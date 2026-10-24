# Catálogo e arquitetura

[Índice](README.md). Estruturas e integrações abaixo são contratos para a implementação futura.

## Catálogo inicial

| ID | Tipo | Base | Organização |
| --- | --- | --- | --- |
| `vanilla-js` | Frontend | HTML, CSS, módulos JavaScript; sem bundler obrigatório | assets, scripts, styles e views |
| `react-vite-ts` | Frontend | React + TypeScript + Vite | app, pages, components, features, services e styles |
| `next-ts` | Frontend | Next.js + TypeScript | App Router em app; components, features e serviços separados |
| `express-ts` | Backend | Node.js + TypeScript + Express | camadas organizadas por funcionalidade |
| `nest-ts` | Backend | Node.js + TypeScript + NestJS | módulos do Nest com controllers, providers e adapters |
| `rust-axum` | Backend | Rust + Axum + Tokio | módulos Rust com camadas explícitas e composição no bootstrap |

O catálogo inicial combina qualquer frontend com qualquer backend para fullstack. Os testes devem verificar as nove combinações em sua configuração mínima. Combinações com extensões entram no catálogo somente quando possuem validação própria.

## Estrutura backend padronizada

Modelo conceitual para Express; adaptar nomes de arquivos e módulos às convenções de Nest e Rust:

```text
src/
  app.*                     # composição HTTP, sem iniciar listener nos testes
  server.*                  # bootstrap, shutdown e listener
  config/                   # validação das configurações selecionadas
  http/
    routes.*                # composição das APIs e versionamento /api/v1
    middlewares/            # erros, request ID, logs e CORS quando necessário
  modules/
    example/
      controllers/          # traduz HTTP para chamadas de serviço
      services/             # regras e casos de uso
      dto/                  # validação de entrada e saída
      repositories/         # somente com persistência selecionada
  infrastructure/
    database/               # somente com banco
    cache/                  # somente com Redis
    observability/          # somente com monitoria
    secrets/                # somente com Vault
  tests/
```

Controllers chamam services; services usam contratos de repository quando há persistência; adapters implementam a comunicação com serviços externos. Configuração de pool/conexão não fica em controller. Não criar repositories vazios, conexões de banco ou entidades persistentes quando o banco for `none`.

Um backend mínimo oferece endpoint demonstrativo e health checks, erro HTTP consistente, logs estruturados, configuração validada e shutdown controlado. A seleção de persistência acrescenta uma fatia demonstrativa de leitura/escrita com teste de integração; sem persistência, o exemplo permanece stateless, sem fingir um banco em memória.

Nest mantém seus `modules`, controllers e providers, injetando implementações de repositório por tokens. Rust usa `mod`, structs/traits e estado compartilhado do Axum; não reproduz decorators ou herança do ecossistema Node. O guia gerado explica a localização equivalente de cada camada.

## Frontend

React/Vite terá `src/app`, `src/pages`, `src/components`, `src/features`, `src/services` e `src/styles`. Componentes compartilhados não dependem de páginas; serviços concentram acesso à API; componentes não recebem credenciais de infraestrutura.

Next usa `src/app` para rotas/layouts, preservando as convenções do App Router, com `src/components`, `src/features` e `src/services` para organização. Não criar `pages/` apenas para imitar o React/Vite. Código exclusivo de servidor e variáveis privadas não entram em módulos de cliente.

Vanilla usa `index.html`, `assets/` e `src/scripts`, `src/styles`, `src/views`. Oferece um servidor HTTP local documentado, sem adicionar React/Vite. Não exigir Node para abrir um projeto estático simples; instalação de dependências pode ser não aplicável. Não prometer um roteador de SPA que não foi selecionado.

## Integrações de dados

| Escolha | Conteúdo exigido |
| --- | --- |
| `none` | Nenhuma biblioteca, env, configuração ou inicialização de banco/cache |
| PostgreSQL | Driver/ORM definido por stack, pool, schema/migração, exemplo e teste |
| MongoDB | Driver/ODM definido por stack, conexão, validação/modelo, índices e exemplo/teste |
| Redis | Cliente por stack, conexão, timeout, política de falha e exemplo de TTL |

Sugestão de apresentação: PostgreSQL para dados relacionais e transações; MongoDB quando a aplicação pede documentos e seu modelo de consulta foi considerado. Nenhum deve ser obrigatório. Redis é cache, não a fonte primária da informação; o boilerplate deve definir quando uma falha de cache permite seguir consultando o serviço principal.

As bibliotecas, versões e licenças serão avaliadas e registradas antes da implementação dos adapters. Não selecionar ORM/ODM apenas para preencher uma pasta. MongoDB não deve receber um fluxo de migrações SQL fictício.

## Fullstack e repositórios separados

```text
# Monorepo
plataforma/
  apps/web/
  apps/api/
  infra/                    # apenas os módulos escolhidos
  docs/
  boilerplate.toml              # receita sem segredos

# Separados: duas raízes, cada uma com docs e receita própria
plataforma-web/
plataforma-api/
```

No monorepo, criar scripts de coordenação somente para ferramentas necessárias. Não impor gerenciador de workspaces Node ao backend Rust. No layout separado, cada projeto constrói e inicia independentemente, com contratos de URL/CORS compatíveis; nenhum arquivo pode depender de um caminho absoluto na máquina que gerou os projetos.

O modo separado gera diretórios preparados para serem repositórios independentes; não cria repositórios remotos nem pressupõe organizações GitHub/GitLab. Os guias explicam a integração entre equipes e ambientes.
