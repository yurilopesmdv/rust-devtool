# Gerador de projetos web — especificação

Status: catálogo, planejamento, doctor, geração (`new`), assistente, instalação de dependências e Git/GitHub estão implementados em pacote independente. As seis bases e arquivos Compose estão disponíveis; os adapters de dados/cache/Kubernetes/monitoria/Vault também estão implementados. A execução de infraestrutura continua manual. Consulte o [README do pacote](../README.md) para os comandos disponíveis.

## Objetivo e escopo definido

Criar rapidamente uma base web executável, com estrutura consistente e integrações opcionais reais. A ferramenta tem seu próprio pacote Rust, separado dos utilitários de devtool e do [guia de aprendizado](../../../guia/README.md).

- Frontend: HTML/CSS/JavaScript sem framework, React com Vite ou Next.js.
- Backend: Node.js/TypeScript com Express, NestJS ou Rust com Axum.
- Backend padronizado em camadas: controllers, services, repositories, middlewares, rotas HTTP e configuração. A organização física respeita cada framework.
- Dados: nenhum, PostgreSQL ou MongoDB. Cache: nenhum ou Redis. Bibliotecas de integração serão fixadas e testadas por stack, sem outro questionário de ORM.
- Docker/Compose, Kubernetes, Prometheus/Grafana e Vault são opcionais independentes, sujeitos a compatibilidade explícita.
- Frontend e backend podem ser criados individualmente ou juntos. Fullstack permite combinar as três opções de frontend com as três de backend, no mesmo repositório ou em dois diretórios para repositórios separados.
- Uma opção desativada não gera seu código, configuração, variável de ambiente, dependência, serviço ou instrução de instalação.
- Ao final, perguntar se deve instalar as dependências da aplicação. A execução da aplicação e da infraestrutura fica fora do gerador: não iniciar containers, criar clusters, aplicar manifests nem inicializar Vault.
- Mobile e novas linguagens ficam para futuras versões do catálogo.

A decisão final de não executar infraestrutura substitui a proposta inicial de perguntar se deveria iniciá-la.

## Documentos

| Documento | Conteúdo |
| --- | --- |
| [Fluxo e comandos](fluxo-e-comandos.md) | Perguntas, flags, receitas e comportamento de erros |
| [Catálogo e arquitetura](catalogo-e-arquitetura.md) | Estruturas por stack, integrações e combinações |
| [Infraestrutura](infraestrutura.md) | Docker, Kubernetes, dados, monitoria, segredos e conexões |
| [Instalação e operação](instalacao-e-operacao.md) | Dependências e contrato dos guias gerados |
| [Plano de implementação](implementacao.md) | Módulos Rust, entregas, testes e critérios de conclusão |
| [Fontes e decisões](referencias.md) | Documentação oficial consultada e justificativas |

## Funcionalidades disponíveis

Veja [integrações](integracoes.md) e [validação](validacao.md).

Consulte [criação e instalação](criacao-e-instalacao.md), [Git/GitHub](git-e-github.md), [manutenção por agentes](manutencao-por-agentes.md), a [arquitetura do pacote](arquitetura-do-pacote.md) e o [guia de extensão de linguagens](adicionar-stack.md). O pacote é independente de devtool e executa em WSL/Linux.

## Exemplo de resultado esperado

Uma escolha de React/Vite + Express/TS, PostgreSQL, Redis, Docker/Compose, Kubernetes, monitoria e Vault deve gerar: duas aplicações conectadas; configuração de dados/cache; imagens; ambiente Compose; manifests de laboratório; coleta de métricas; dashboards provisionados; integração de leitura de segredos; e guias para instalar ferramentas, preparar valores locais e executar cada parte.

A mesma escolha sem banco, cache e infraestrutura deve gerar somente as aplicações, os testes, a configuração básica e sua documentação. Não deve haver imports quebrados ou parâmetros obrigatórios de serviços omitidos.

As combinações das bases podem ser geradas nos dois layouts. O exemplo completo com dados/monitoria/segredos é gerável; é preciso preencher as configurações e executar seu bootstrap no ambiente de destino. Gerar manifests de laboratório não equivale a implantar um ambiente de produção.
