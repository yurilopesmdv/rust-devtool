# Infraestrutura opcional e conexão entre aplicações

[Índice](README.md). Contratos de geração; o gerador não executará a infraestrutura.

## Composição por seleção

| Seleção | Gerar | Omitir se desativada |
| --- | --- | --- |
| Docker/Compose | Dockerfile por aplicação selecionada, .dockerignore, compose e guia | Imagens e arquivos Compose |
| Kubernetes | Kustomize base/overlay, workloads, Services e configuração | Toda a árvore de manifests |
| PostgreSQL/MongoDB | Integração da aplicação e recursos locais quando pedidos | Drivers, conexões, envs, serviços e volumes |
| Redis | Cliente e configuração; serviço local se pedido | Código de cache, variáveis e recursos Redis |
| Prometheus/Grafana | Instrumentação suportada, coleta, datasource e dashboards | Bibliotecas de métricas, endpoint e arquivos de monitoria |
| Vault | Leitura de arquivos de segredos, integração e guia de bootstrap | Clientes, templates e referências ao Vault |

Não manter todos os serviços em um Compose ocultos por profiles quando o usuário escolheu `none`: o conteúdo deve ser omitido. Profiles podem organizar o que foi selecionado, por exemplo separar monitoria de serviços da aplicação. Esse uso é compatível com os [profiles do Compose](https://docs.docker.com/compose/how-tos/profiles/).

## Docker e desenvolvimento local

Dockerfile com build separado da execução quando aplicável, imagem de runtime apropriada e processo sem root quando suportado. Imagens e ferramentas terão versões explícitas no catálogo. Aplicações escutam na interface correta dentro do container, encerram com sinais e expõem health checks.

Compose inclui somente aplicações e serviços escolhidos; nomes, portas, volumes e health checks precisam ser consistentes. Endereços entre containers usam nomes de serviço, não `localhost`. Serviços locais sensíveis ficam acessíveis pela máquina local apenas quando necessário; credenciais são fornecidas por arquivos ignorados ou segredos montados, nunca fixadas no template.

Bancos e Redis locais usam armazenamento/volumes de desenvolvimento conforme o serviço. O guia distingue parar containers de remover volumes. Migrations e preparação de índices são comandos explícitos, sem executar alterações destrutivas silenciosamente no startup.

Sem Docker, a aplicação continua executável com runtime local. Serviços selecionados devem ter instruções de instalação ou apontar para instâncias externas. Escolher banco sem Docker não habilita Docker implicitamente.

## Kubernetes

Gerar Kustomize com `base` e overlay `dev`, usando o [suporte de Kustomize do kubectl](https://kubernetes.io/docs/tasks/manage-kubernetes-objects/kustomization/). O overlay de produção, quando houver, exige valores fornecidos pelo usuário e validação específica; não é uma cópia do laboratório marcada como pronta para produção.

A base contém Deployment/Service das aplicações, ConfigMaps não secretos, portas, probes e requests/limits iniciais documentados. Ingress é opcional e depende de controller/domínios existentes; o template não instala implicitamente um ingress controller. O guia de desenvolvimento inclui acesso por port-forward quando não houver ingress.

Uma imagem é pré-requisito: permitir Dockerfile gerado ou referência de imagem já existente. Kubernetes sem Compose é válido. Usar tags/digests explícitos; o guia mostra como construir/publicar/carregar imagens e preencher referências, sem executar esses passos.

Para banco/cache/monitoria/Vault, distinguir `local` (recursos de laboratório no ambiente alvo) de `external` (configuração para instâncias existentes). Banco local em Kubernetes requer volume, armazenamento compatível e documentação de inicialização. Não pressupor StorageClass. Recursos de estado não são modelados como cópias stateless da API.

Não criar cluster ou recursos de nuvem. Não aplicar manifests. Dependências de cluster precisam aparecer no README e no plano: storage, DNS/ingress quando escolhido, autenticação Kubernetes do Vault quando escolhida. A primeira versão deve preferir recursos nativos; `ServiceMonitor` só pode ser gerado com suporte explícito ao Prometheus Operator.

## URLs, frontend/backend e CORS

| Origem da chamada | Endereço esperado |
| --- | --- |
| Navegador em desenvolvimento | URL publicada na máquina, ou caminho relativo servido por proxy |
| Container → API | Nome do serviço Compose e porta interna |
| Aplicação servidor → API no cluster | DNS do Service Kubernetes |
| Navegador → API no cluster | Origem pública/ingress ou acesso local documentado por port-forward |

Nunca expor `http://api:porta` ou DNS interno Kubernetes como URL a ser usada pelo navegador. CORS usa uma lista explícita de origens; não combinar credenciais com origem irrestrita. Os guias mostram como trocar URLs entre desenvolvimento e implantação.

Vite injeta variáveis de cliente no build; Next também distingue variáveis públicas de configurações do servidor. O template documenta quando reconstruir a imagem e quando uma configuração de runtime é suportada. Segredos de banco/cache/Vault não podem ter prefixos de exposição ao browser.

No monorepo, Compose e manifests podem conectar serviços pelos seus nomes. Em repositórios separados, cada Compose tem sua própria rede por padrão: gerar instruções explícitas para criar uma rede externa compartilhada, ou usar URLs acessíveis pelo host. Em Kubernetes, usar namespace e nomes de Service combinados. O contrato de integração acompanha os dois projetos.

## Monitoria: Prometheus e Grafana

Módulo opcional completo: métricas HTTP de contagem, duração e erros; endpoint de coleta; configuração de scrape do Prometheus; datasource Grafana; dashboard provisionado com painéis de tráfego, latência e falhas.

As métricas precisam vir da aplicação ou servidor web real, não de um painel sem fonte de dados. Evitar labels com URLs não normalizadas, IDs de usuário e outros valores sem limite. Logs básicos continuam existindo sem a opção de monitoria; tracing distribuído e agregação de logs são futuras extensões.

Frontend estático não roda um endpoint de métricas de processo Node. Quando servido em container, as métricas podem vir do servidor HTTP/exporter escolhido; sem servidor instrumentado, não oferecer um painel HTTP fictício. Métricas de experiência no browser ficam fora desta primeira camada.

No Kubernetes, permitir conectar a uma stack existente; se a opção local for selecionada, gerar recursos de laboratório e instruções de instalação. Monitoria sem Docker/Kubernetes gera configuração para binários locais ou serviços externos, com os pré-requisitos documentados.

O contrato de coleta segue os [primeiros passos do Prometheus](https://prometheus.io/docs/introduction/first_steps/); datasource e dashboards seguem o [provisionamento por arquivos do Grafana](https://grafana.com/docs/grafana/latest/administration/provisioning/).

## Variáveis de ambiente e Vault

A base de toda aplicação tem validação de configuração e `.env.example` com nomes e placeholders aplicáveis. `.env` real é ignorado; nenhuma opção cria senha real versionada. Dados não secretos e segredos têm contratos separados.

Vault é opcional. Contrato proposto: suporte explícito a arquivos de segredo montados, com configurações como `DATABASE_URL_FILE` quando houver banco. A aplicação lê o arquivo no bootstrap, valida o valor e não o imprime. Arquivo e variável direta simultâneos para a mesma chave devem produzir erro de ambiguidade; a documentação explica o fluxo escolhido.

No Kubernetes, um initContainer Vault Agent autentica pelo método Kubernetes e renderiza arquivos em emptyDir em memória. A API monta esses arquivos read-only. Não há dependência do Injector: a identidade/role/policy e o método de autenticação precisam ser configurados pelo operador conforme o guia gerado.

No Compose, o laboratório pode fornecer Vault/Agent e um procedimento manual de bootstrap, entrega de credenciais e renderização em volume compartilhado. Não embutir root token nem marcar o modo dev do Vault como configuração de produção. Para Vault externo, gerar apenas o contrato e as configurações de integração aplicáveis.

O guia explica que este primeiro adapter lê no startup; rotação sem reinício exige mecanismo posterior próprio. Templates Kubernetes não guardam segredos reais em YAML: o [objeto Secret](https://kubernetes.io/docs/concepts/configuration/secret/) não torna o conteúdo seguro apenas por codificá-lo em base64.

## Contratos de disponibilidade

Health de processo deve ser separado da readiness das dependências exigidas pelo serviço. Um banco obrigatório indisponível pode impedir readiness; Redis usado como cache tolerante a falha não deve causar indisponibilidade total sem uma razão documentada. Configuração de segredos ausente precisa falhar de modo claro antes de aceitar tráfego.

Cada combinação tem um checklist funcional no README gerado: aplicação inicia, frontend alcança API, API acessa os serviços selecionados, saúde responde, métricas são coletadas e segredos chegam pelo mecanismo escolhido. Esses passos são executados pelo usuário, não pelo gerador.
