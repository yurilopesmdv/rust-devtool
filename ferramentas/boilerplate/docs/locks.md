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

## Python e npm

Os locks uv e npm continuam distribuídos. Os comandos específicos de manutenção estão nas microtasks T012/T013; até sua entrega, siga a validação em projeto temporário e mantenha manifesto/lock coerentes. Não execute um instalador com versão flutuante em cada geração de aplicação.
