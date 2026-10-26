# Manutenção dos lockfiles

[Guia de validação](validacao.md) · [Manutenção por agentes](manutencao-por-agentes.md)

Locks distribuídos são parte do boilerplate. Atualize manifesto, snapshots e evidência juntos. Execute os comandos desta página na raiz do rust-devtool; as operações de resolução usam diretórios temporários e não iniciam aplicações ou infraestrutura.

## Rust: 12 combinações

Python 3.8+ e Rust/Cargo. A primeira execução pode precisar do registry; --offline exige caches preenchidos.

```sh
python3 ferramentas/boilerplate/scripts/update_rust_locks.py --check --offline
# Depois de mudar as versões/features do template ou adapter:
python3 ferramentas/boilerplate/scripts/update_rust_locks.py --write
# Atualização explícita também das dependências transitivas compatíveis:
python3 ferramentas/boilerplate/scripts/update_rust_locks.py --write --upgrade
```

O example rust_lock_manifests utiliza o mesmo compositor de dependências de generator/integrations/rust.rs. Não existe uma lista de versões copiada para o script Python. São enumerados nenhum/PostgreSQL/MongoDB × cache ligado/desligado × métricas ligadas/desligadas. A base vai para templates/rust-axum/Cargo.lock; as outras onze vão para integrations/rust-locks/.

As dependências diretas do template e adapters usam versões exatas. O comando recusa requisitos flutuantes, resolve em /tmp e verifica os resultados com --locked e fetch para o host Linux, sem buscar fontes de targets Windows. O nome temporário do projeto é substituído por {{PROJECT_NAME}} nos snapshots. O Cargo pode reordenar o bloco do projeto; isso não implica mudança de versão de dependência.

--check não sobrescreve snapshots e retorna 1 se houver diferença. --write resolve todas as combinações antes de gravar diferenças. Por padrão, cargo update --workspace conserva transitivas existentes quando ainda compatíveis; --upgrade precisa de --write e permite resolver novas transitivas compatíveis. Revise o diff e execute os checks das aplicações afetadas antes de distribuir.

Duas checagens consecutivas com os mesmos manifestos/locks/cache devem produzir a mesma saída. Uma atualização legítima pode mudar o grafo; geração determinística não significa que atualizar dependências dispensa revisão.

## Python: 12 combinações, 24 variantes

Requer Python 3.8+ para o script, Rust/Cargo para compor os manifestos, uv 0.13.5 e Python 3.13 para as aplicações. Prepare o interpretador com `uv python install 3.13 --no-bin`. `--offline` exige também os caches de resolução e pacotes preenchidos.

```sh
python3 ferramentas/boilerplate/scripts/update_python_locks.py --check --offline
python3 ferramentas/boilerplate/scripts/update_python_locks.py --write
python3 ferramentas/boilerplate/scripts/update_python_locks.py --write --upgrade
```

O example python_lock_manifests usa o mesmo compositor de generator/integrations/python.rs. O script resolve nenhum/PostgreSQL/MongoDB × cache × métricas em diretórios temporários, executa `uv sync --locked` em cada combinação e normaliza somente o nome do projeto. Vault usa as dependências já presentes na base; suas variantes compartilham esses locks.

São mantidos os doze arquivos em integrations/python-locks/ e a cópia da base em templates/python-fastapi/uv.lock. `--check` não grava snapshots e retorna 1 quando há diferenças. `--write` resolve e instala todas as combinações antes de publicar; conserva o grafo existente quando compatível. `--upgrade` exige `--write` e solicita atualização explícita das transitivas. A instalação ocorre apenas nos ambientes temporários do verificador, sem iniciar serviços.

Depois de alterar adapters, execute também a matriz de aplicações descrita em [Validação](validacao.md), incluindo opções com Vault. O check de locks verifica resolução/instalação; os testes das aplicações verificam comportamento.

## npm

Os locks npm continuam distribuídos. O comando específico está na microtask T013; até sua entrega, valide em projeto temporário e mantenha manifesto/lock coerentes. Não execute um instalador com versão flutuante em cada geração de aplicação.
