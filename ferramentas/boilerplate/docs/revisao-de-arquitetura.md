# Revisão das camadas e conexões

[Catálogo](catalogo-e-arquitetura.md) · [Qualidade](qualidade.md) · [Validação](validacao.md)

A revisão de uma API Nest com PostgreSQL, Redis, Compose, Kubernetes, monitoria e Vault encontrou uma separação incompleta: o repository só reexportava funções, e o arquivo de cliente da infraestrutura executava consultas do exemplo. Não havia consulta dentro da leitura de variáveis, mas a localização da lógica de acesso aos dados dificultava entender as camadas.

## Responsabilidades atuais

| Camada | Responsabilidade |
| --- | --- |
| Configuração | Ler/validar ambiente e arquivos de segredos, sem queries |
| Cliente de banco | Criar pool/conexão, verificar disponibilidade e fechar recursos |
| Schema/prepare | Preparar o exemplo por comando explícito; não no startup |
| Repository | SQL parametrizado ou operações do driver MongoDB do módulo |
| Service | Caso de uso, coordenação do repository e cache |
| Controller/router | HTTP, validação da entrada e tradução da resposta |
| Composição/lifecycle | Registrar dependências e abrir/fechar recursos selecionados |

O fluxo é controller/router → service → repository → cliente de banco. SELECT 1/ping de readiness são verificações de conectividade, não consultas do domínio. Os repositories agora contêm as consultas reais em Express, Nest, Rust e Python.

Nest usa `ExampleModule` por funcionalidade e injeta `ExampleRepository` em `ExampleService`. `InfrastructureModule` registra o lifecycle com os hooks do framework. O teste do service usa um repository falso sem abrir conexão, enquanto a suíte de integração verifica o driver real. Não é imposta Clean Architecture ou uma hierarquia de abstrações vazias.

Rust também tem arquivos distintos de controller, service e repository. Python usa APIRouter/Pydantic/Depends e recursos pertencentes à instância FastAPI via lifespan/app.state. Cada API inclui seu próprio `docs/ARQUITETURA.md`.

## Outras correções

- A invalidação do cache ocorre depois que a escrita tem sucesso. O cache continua eventualmente consistente, com TTL de 30s; isso não resolve a corrida entre leitores e escritores concorrentes para casos que exijam consistência forte.
- Drivers Node têm limites de seleção/conexão e consulta/socket. A readiness verifica o banco obrigatório; Redis pode falhar sem derrubar o acesso à origem.
- Preparação de schema, credenciais, bootstrap Vault e execução de infraestrutura continuam explícitos.
- Dockerfiles Node instalam com npm ci, respeitando os locks; os READMEs usam o env-file do Compose.
- Express preserva HTTP 400 para JSON inválido e 413 para payload grande; erros internos não imprimem objetos de erro com configuração do driver.
- A escolha de painel de processo Node depende do runtime Next, em vez da presença de npm para ferramentas do frontend estático.
- Arquivos gerados e templates receberam indentação consistente e checks por linguagem. TypeScript/parser de lint são fixados em versões compatíveis.

## Limites

A base implementa um exemplo pequeno; não adiciona autenticação/autorização, regras de negócio da sua aplicação ou uma plataforma completa de migrações. Conexões Node/Rust são compartilhadas por processo; aplicações concorrentes com configurações diferentes devem usar processos separados. Python isola recursos por instância.

Containers e deploys Kubernetes precisam de verificação operacional no ambiente de destino. Os testes locais de persistência, Agent e configuração não comprovam um deploy de produção. Veja as evidências em validacao.md.
