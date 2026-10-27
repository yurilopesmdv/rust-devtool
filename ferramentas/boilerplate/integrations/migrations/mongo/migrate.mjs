import { MongoClient } from 'mongodb';
import { createHash, randomUUID } from 'node:crypto';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const owner = randomUUID();
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
const args = process.argv.slice(2);
let interrupted = 0;
process.on('SIGINT', () => {
  interrupted = 130;
});
process.on('SIGTERM', () => {
  interrupted = 143;
});

function configuration() {
  const prefix = process.env.MIGRATION_DATABASE_URL || process.env.MIGRATION_DATABASE_URL_FILE
    ? 'MIGRATION_DATABASE_URL'
    : 'DATABASE_URL';
  const direct = process.env[prefix];
  const file = process.env[`${prefix}_FILE`];
  if (direct && file) throw new Error('Configuração ambígua');
  const url = direct || (file && readFileSync(file, 'utf8').replace(/[\r\n]+$/, ''));
  if (!url || !/^mongodb(?:\+srv)?:\/\/[^/]+\/[^?/#]+(?:\?|$)/.test(url)) {
    throw new Error('URI Mongo obrigatória');
  }
  return url;
}

function load() {
  const ids = new Set();
  return readdirSync(join(root, 'migrations/mongo'), { withFileTypes: true })
    .filter(entry => entry.name.endsWith('.json')).sort((a, b) => a.name.localeCompare(b.name))
    .map(entry => {
      if (!entry.isFile() || !/^[0-9]{14}_[a-z0-9_-]+\.json$/.test(entry.name)) {
        throw new Error('Arquivo inválido');
      }
      const id = entry.name.slice(0, 14);
      if (ids.has(id)) throw new Error('Versão duplicada');
      ids.add(id);
      const bytes = readFileSync(join(root, 'migrations/mongo', entry.name));
      if (bytes.length > 1048576 || bytes.subarray(0, 3).equals(Buffer.from([239, 187, 191]))) {
        throw new Error('Arquivo inválido');
      }
      const spec = JSON.parse(bytes.toString('utf8'));
      if (!spec || typeof spec !== 'object' || Array.isArray(spec)) {
        throw new Error('Objeto obrigatório');
      }
      const keys = spec.operation === 'bootstrap_examples'
        ? ['operation']
        : ['operation', 'source', 'target', 'batch_size'];
      if (Object.keys(spec).some(key => !keys.includes(key))) throw new Error('Opção desconhecida');
      if (spec.operation === 'copy_field') {
        if (
          ![spec.source, spec.target].every(field =>
            typeof field === 'string' && /^[a-z][a-z0-9_]{0,63}$/.test(field)
          ) || spec.source === spec.target || !Number.isInteger(spec.batch_size)
          || spec.batch_size < 1 || spec.batch_size > 1000
        ) throw new Error('Backfill inválido');
      } else if (spec.operation !== 'bootstrap_examples') throw new Error('Operação desconhecida');
      return { id, checksum: createHash('sha256').update(bytes).digest('hex'), spec };
    });
}

async function run(client, files) {
  const database = client.db();
  await database.command({ ping: 1 });
  const locks = database.collection('_boilerplate_migration_lock');
  const ledger = database.collection('_boilerplate_migrations');
  const examples = database.collection('examples');
  if (args[0] === 'unlock') {
    if (
      args.length !== 3 || args[2] !== '--confirm-stopped' || !/^[a-zA-Z0-9-]{1,128}$/.test(args[1])
    ) throw new Error('Confirmação obrigatória');
    const result = await locks.deleteOne({ _id: 'runner', owner: args[1] });
    if (result.deletedCount !== 1) throw new Error('Reserva não corresponde');
    console.log(JSON.stringify({ unlocked: true }));
    return;
  }
  if (args.length !== 1 || !['status', 'up'].includes(args[0]) || files.length === 0) {
    throw new Error('Uso: status|up|unlock OWNER --confirm-stopped');
  }
  const records = await ledger.find({}).toArray();
  for (const row of records) {
    if (!files.some(file => file.id === row._id && file.checksum === row.checksum)) {
      throw new Error('Checksum divergente ou arquivo removido');
    }
  }
  if (args[0] === 'status') {
    const reservation = await locks.findOne({ _id: 'runner' });
    console.log(
      JSON.stringify({
        reserved_by: reservation?.owner ?? null,
        migrations: files.map(file => {
          const row = records.find(record => record._id === file.id);
          return {
            version: file.id,
            status: row?.status ?? 'pending',
            checkpoint: Number(row?.checkpoint ?? 0),
          };
        }),
      }),
    );
    return;
  }
  let acquired = false;
  try {
    const deadline = Date.now() + 30000;
    while (!acquired) {
      if (interrupted || Date.now() >= deadline) throw new Error('Reserva indisponível');
      try {
        await locks.insertOne({ _id: 'runner', owner, started_at: new Date() });
        acquired = true;
      } catch (error) {
        if (error.code !== 11000) throw error;
        await sleep(50);
      }
    }
    // Recheck history under the reservation, since another runner may have completed while waiting.
    const current = await ledger.find({}).toArray();
    for (const row of current) {
      if (!files.some(file => file.id === row._id && file.checksum === row.checksum)) {
        throw new Error('Histórico divergente');
      }
    }
    const highest = current.filter(row =>
      row.status === 'applied'
    ).map(row => row._id).sort().at(-1) ?? '';
    for (const file of files) {
      const existing = await ledger.findOne({ _id: file.id });
      if (existing?.status === 'applied') continue;
      if (file.id < highest) throw new Error('Migration fora de ordem');
      await ledger.updateOne({ _id: file.id }, {
        $setOnInsert: { checksum: file.checksum, checkpoint: 0, created_at: new Date() },
        $set: { status: 'running', owner, updated_at: new Date() },
      }, { upsert: true });
      try {
        if (file.spec.operation === 'bootstrap_examples') {
          await examples.createIndex({ message: 1 });
          await examples.updateOne({ _id: 'default' }, {
            $setOnInsert: { message: 'API funcionando' },
          }, { upsert: true });
        } else {
          while (true) {
            if (interrupted || !await locks.findOne({ _id: 'runner', owner })) {
              throw new Error('Execução interrompida');
            }
            const docs = await examples.find({ [file.spec.target]: { $exists: false } }).sort({
              _id: 1,
            }).limit(file.spec.batch_size).toArray();
            if (docs.length === 0) break;
            for (const doc of docs) {
              if (typeof doc[file.spec.source] !== 'string') {
                throw new Error('Fonte do backfill precisa ser string');
              }
              const changed = await examples.updateOne({
                _id: doc._id,
                [file.spec.target]: { $exists: false },
              }, { $set: { [file.spec.target]: doc[file.spec.source] } });
              await ledger.updateOne({ _id: file.id, owner }, {
                $inc: { checkpoint: changed.modifiedCount },
                $set: { updated_at: new Date() },
              });
            }
          }
        }
        if (interrupted || !await locks.findOne({ _id: 'runner', owner })) {
          throw new Error('Reserva perdida');
        }
        await ledger.updateOne({ _id: file.id, owner }, {
          $set: { status: 'applied', updated_at: new Date() },
          $unset: { error_code: '' },
        });
      } catch (error) {
        await ledger.updateOne({ _id: file.id, owner }, {
          $set: { status: 'failed', error_code: 'execution_failed', updated_at: new Date() },
        }).catch(() => {});
        throw error;
      }
    }
    console.log(JSON.stringify({ applied: true }));
  } finally {
    if (acquired) await locks.deleteOne({ _id: 'runner', owner });
  }
}

let client;
try {
  const files = args[0] === 'unlock' ? [] : load();
  client = new MongoClient(configuration(), {
    serverSelectionTimeoutMS: 5000,
    connectTimeoutMS: 5000,
    socketTimeoutMS: 5000,
  });
  await run(client, files);
} catch {
  console.error(
    'Migration Mongo falhou; verifique configuração, reserva, checksum e dados. Consulte docs/MIGRATIONS.md.',
  );
  process.exitCode = interrupted || 1;
} finally {
  if (client) {
    await client.close().catch(() => {
      process.exitCode = process.exitCode || 1;
      console.error('Falha ao encerrar cliente Mongo');
    });
  }
}
