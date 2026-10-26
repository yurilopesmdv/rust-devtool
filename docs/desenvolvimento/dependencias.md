# Atualização de dependências

Atualizar o gerador e atualizar o código distribuído são entregas diferentes. Dependabot propõe Cargo do workspace e actions semanalmente; não faz merge automático. Templates npm/uv/Cargo com placeholders são atualizados pelos [comandos de snapshots](../../ferramentas/boilerplate/docs/locks.md), em lotes por stack. Não criar bots que instalem versões latest em cada geração.

## Fluxo de revisão

1. Abrir lote pequeno (um ecossistema/stack ou atualização coordenada de peers), registrar motivo, origem e releases consultadas. Major exige revisão de API e contratos; atualização de segurança pode furar a cadência semanal.
2. Alterar manifesto e resolver o lock pelo comando versionado. Usar ferramentas compatíveis com as versões do catálogo. Não apagar lock inteiro, usar legacy-peer-deps ou incluir dependência opcional desativada para contornar conflito.
3. Revisar dependências transitivas adicionadas/removidas, checksums/integrities, scripts de instalação e licença. Consultar advisories do ecossistema (RustSec/Cargo audit, npm audit e auditor Python em ambiente temporário, com ferramenta fixada/revisada). Registrar data e limites; um scan sem alertas não prova ausência de vulnerabilidades.
4. Pedir o plano de checks e executar a matriz afetada. Preservar evidência local e aguardar os jobs remotos aplicáveis antes de merge. Não apresentar o plano como execução dos checks.
5. Se houver exceção, registrar pacote/versão/advisory ou licença, alcance/exposição, justificativa, compensação, responsável e prazo de reavaliação. Não silenciar globalmente categorias inteiras nem usar exceção sem prazo para distribuir uma base.

```sh
# Arquivos tracked modificados contra HEAD (ou --base REF):
python3 scripts/dependency_review.py
# Simulação explícita, sem alterar dependências:
python3 scripts/dependency_review.py --files \
  ferramentas/boilerplate/templates/express-ts/package.json \
  ferramentas/boilerplate/templates/express-ts/package-lock.json
```

O relatório JSON distingue workspace, actions e os três grupos de templates; informa comandos e revisão manual. Não inclui arquivos untracked automaticamente: passe --files para uma stack nova. O relatório é conservador e não substitui revisão do diff de adapters. Mudança de infraestrutura requer também checks Compose/Kustomize e, quando aplicável, o smoke Docker optativo. Alterações de catálogo/contratos compartilham risco entre stacks e exigem revisar a matriz inteira.

## Política por origem

| Origem | Publicação | Verificação mínima |
| --- | --- | --- |
| Cargo workspace | Cargo.toml/Cargo.lock em commit próprio | fmt, todos os testes, Clippy e regressões do CLI |
| npm templates | Manifestos base + locks base/superset | npm ci sem ignorar peers, lint/format/typecheck/build/test, ESM/CommonJS quando suportado e adapters afetados |
| uv templates | pyproject + 12 snapshots + cópia base | uv sync --locked, Ruff, pytest e variantes com Vault/adapters afetados |
| Rust templates | Manifesto + 12 snapshots | cargo --locked, fmt/test/Clippy e adapters afetados |
| Go templates | go.mod/go.sum do adapter | go mod verify, gofmt/test -race/vet/build, base sem driver e PostgreSQL real |
| Actions | SHA imutável com release legível no comentário | release oficial, permissões mínimas, actionlint e execução remota |
| Imagens | tags/digests e documentação de upgrade | origem oficial, release, arquitetura Linux e smoke no daemon descartável |

Para um patch de TypeScript, a simulação deve produzir diff do manifesto e lock e apontar os checks npm; o lint com typescript-eslint confirma compatibilidade. Não distribuir uma versão simulada ou não publicada. O mesmo princípio vale para mudanças dos drivers: resolução do grafo, testes reais de persistência e verificação de readiness/shutdown antes de marcar como validado.
