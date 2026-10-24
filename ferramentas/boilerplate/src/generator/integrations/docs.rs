use super::*;
pub fn write(
    r: &Recipe,
    files: &mut Files,
    has_api: bool,
    monitored: bool,
    vault: bool,
    namespace: &str,
    name: &str,
) {
    let db = if has_api {
        r.backend.database.unwrap_or(Database::None)
    } else {
        Database::None
    };
    let cache = has_api && r.backend.cache == Some(Cache::Redis);
    if r.infrastructure.containers != Some(Containers::Compose)
        && r.infrastructure.kubernetes != Some(Kubernetes::Manifests)
        && !monitored
        && !vault
        && db == Database::None
        && !cache
    {
        return;
    }
    let mut text = String::from(
        "# Operação do ambiente gerado\n\nEste documento acompanha apenas os módulos selecionados. As configurações são bases de desenvolvimento: ajuste TLS, domínio, backup, capacidade e políticas para seu ambiente. O gerador não inicia serviços nem opera clusters.\n\n",
    );
    text.push_str("## Dependências do ambiente\n\nA geração offline de JS/TS/Python requer o binário boilerplate; fontes Rust também requerem rustfmt no PATH. Para instalar/executar a aplicação, use o runtime e os comandos do README da aplicação. Não há instalação global de frameworks.\n\n");
    if r.infrastructure.containers == Some(Containers::Compose) {
        text.push_str("Docker/Compose: siga a [instalação oficial no Ubuntu](https://docs.docker.com/engine/install/ubuntu/) compatível com sua distribuição WSL; valide `docker --version`, `docker compose version` e `docker info` antes de iniciar containers.\n\n");
    }
    if r.infrastructure.kubernetes == Some(Kubernetes::Manifests) {
        text.push_str("kubectl: siga a [instalação oficial Linux](https://kubernetes.io/docs/tasks/tools/install-kubectl-linux/). O cluster, kubeconfig, StorageClass e permissões são pré-requisitos externos.\n\n");
    }
    if vault {
        text.push_str("Vault CLI para bootstrap: [instalação oficial Linux](https://developer.hashicorp.com/vault/install). O Agent já faz parte dos arquivos de Compose/Kubernetes escolhidos.\n\n");
    }
    if has_api && (db != Database::None || cache) {
        text.push_str("## Dados e configuração\n\nA aplicação lê DATABASE_URL e/ou REDIS_URL (somente os módulos presentes). .env.example da aplicação usa localhost; infra/.env.example usa nomes internos para containers/Kubernetes. Não misture esses endereços. Preencha credenciais próprias, com percent-encoding nos componentes de senha das URLs. Serviços externos não geram instâncias locais: configure seus URLs e TLS. PostgreSQL aceita sslmode=verify-full; MongoDB suporta mongodb+srv/TLS; Redis suporta rediss. Não desative a validação de certificados.\n\n");
    }
    if r.infrastructure.containers == Some(Containers::Compose) {
        text.push_str("## Compose\n\nNa raiz do repositório:\n\n```sh\ncp infra/.env.example infra/.env\n# Edite infra/.env: substitua TODOS os valores replace-me e endereços externos.\ndocker compose --env-file infra/.env config --quiet\ndocker compose --env-file infra/.env up --build -d\ndocker compose --env-file infra/.env logs --tail=100\n# Parar mantendo volumes:\ndocker compose --env-file infra/.env down\n```\n\nEm uma seleção sem dados/segredos/monitoria, infra/.env.example pode estar vazio. Os serviços publicados ficam vinculados a 127.0.0.1. Nunca use down -v se quiser manter os dados. Em monorepo o Compose coordena as duas aplicações; em split cada raiz tem seu Compose e o browser usa as portas publicadas no host.\n\n");
    }
    if db != Database::None {
        text.push_str("## Preparação de dados\n\nO startup abre a conexão e valida configuração, mas não aplica alterações de schema silenciosamente. Prepare o banco antes de chamar os exemplos. A tabela/coleção usa uma entrada `default`; GET /api/v1/example lê e POST /api/v1/example aceita JSON `{\"message\":\"texto\"}` (1 a 200 caracteres), com parâmetros SQL/driver.\n\n");
        let node = r
            .backend
            .template
            .as_deref()
            .is_some_and(|s| s != "rust-axum");
        if r.backend.template.as_deref() == Some("python-fastapi") {
            text.push_str("Na pasta da API: `uv run --locked --env-file .env python -m app.infrastructure.prepare`. Em Compose: `docker compose --env-file infra/.env run --rm api python -m app.infrastructure.prepare`. Python usa `uv run --locked --env-file .env pytest` para persistência real; sem URLs de teste os testes de integração são pulados. FastAPI retorna 422 para payload inválido.\n\n");
        } else if node {
            text.push_str("Na pasta da API: `npm run build && npm run db:prepare`. Em Compose: `docker compose --env-file infra/.env run --rm api npm run db:prepare` (imagem já compilada).\n\n");
        } else {
            text.push_str("Na pasta da API: `cargo run --example prepare`. Em Compose: `docker compose --env-file infra/.env run --rm api /app/prepare`.\n\n");
        }
        text.push_str(if db==Database::Postgres{"A preparação SQL é idempotente para a tabela demonstrativa; migrations/001_examples.sql documenta o schema. Para evoluir schemas reais, adicione migrações versionadas antes de alterar dados existentes.\n\n"}else{"A preparação MongoDB cria o índice e a entrada demonstrativa idempotentemente. O driver usa a base indicada na URI, e o exemplo valida os campos de entrada. Não há migrações SQL.\n\n"});
        text.push_str("`/health` indica processo ativo; `/ready` consulta o banco obrigatório e retorna 503 quando indisponível. Testes de persistência exigem o banco configurado e preparado: Node habilita o teste HTTP quando DATABASE_URL ou DATABASE_URL_FILE estiver presente; Rust usa `cargo test -- --ignored`. A suíte padrão não simula persistência como se fosse um banco real.\n\n");
    }
    if cache {
        text.push_str("## Cache Redis\n\nLeituras do exemplo usam chave `example` e TTL de 30s e prefixo do projeto; gravações invalidam a chave. Redis indisponível permite consultar o banco/origem sem cache; credencial/URL ausente é erro de configuração. O cache não é a fonte da informação e não determina a readiness. Não coloque dados sensíveis nesse exemplo sem definir sua política.\n\n");
    }
    if monitored {
        text.push_str("## Prometheus e Grafana\n\nCompose: Prometheus em http://localhost:9090 e Grafana em http://localhost:3002 (usuário admin, senha definida em infra/.env). Datasource e dashboard são provisionados por arquivos; confira Targets do Prometheus e o painel Application. Em Kubernetes, faça port-forward dos Services prometheus:9090 e grafana:3000. Não há Prometheus Operator ou CRDs obrigatórios.\n\nBackends expõem /metrics com contagem, duração e status HTTP; labels usam padrões de rota, não URLs com IDs. Em Next standalone são métricas de processo; frontend estático usa stub_status do nginx + exporter, com tráfego/conexões reais (sem inventar latência/status que o stub_status não mede). Quando há API no fullstack, o painel HTTP monitora a API.\n\nPara binários locais sem containers, use infra/monitoring/prometheus.local.yml e ajuste o datasource Grafana para localhost:9090. O provisionamento distribuído aponta para o nome interno prometheus, adequado a Compose/Kubernetes. Dados da monitoria Kubernetes usam emptyDir no overlay dev e são perdidos ao recriar pods; configure armazenamento/retention no seu ambiente.\n\n");
    }
    if r.infrastructure.kubernetes == Some(Kubernetes::Manifests) {
        text.push_str(&format!("## Kubernetes\n\nÉ necessário kubectl, um cluster configurado, StorageClass para serviços locais com PVC e as imagens publicadas ou carregadas no cluster. O gerador não cria cluster, registry, DNS ou ingress controller. Sem imagem explícita, construa a imagem `{}-api:0.1.0`/`{}-web:0.1.0` correspondente ao Deployment; imagens fornecidas devem implementar o contrato de configuração e os binários de preparação.\n\n```sh\nkubectl kustomize infra/kubernetes/overlays/dev > /tmp/{name}-rendered.yaml\n# Revise o resultado, imagens, namespace, recursos e endpoints.\nkubectl create namespace {namespace} --dry-run=client -o yaml | kubectl apply -f -\n# Quando houver credenciais: preencha infra/.env e crie o Secret fora do Git.\nkubectl -n {namespace} create secret generic app-secrets --from-env-file=infra/.env --dry-run=client -o yaml | kubectl apply -f -\nkubectl apply -k infra/kubernetes/overlays/dev\nkubectl -n {namespace} get pods,svc,pvc\n```\n\nNão versione o YAML resultante do Secret. Valores são fornecidos pelo operador e não estão nos manifests gerados. Sem serviços que necessitem credenciais, o passo do Secret não se aplica.\n\nAcesso local: port-forward de svc/api 3000:3000 e de svc/web 5173:80 (3001:80 para Next), em terminais separados. O frontend usa a origem local da API. Para domínio real, ajuste URLs de build VITE_API_URL/NEXT_PUBLIC_API_URL e FRONTEND_ORIGIN, reconstrua imagens e configure seu ingress/TLS.\n\n",name,name));
        if db != Database::None {
            text.push_str(&format!("Prepare dados por Job explícito: `kubectl apply -f infra/kubernetes/jobs/prepare.yaml`, depois `kubectl -n {namespace} logs job/{name}-prepare`. Para repetir após conclusão, remova somente esse Job e aplique novamente.\n\n"));
        }
    }
    if vault {
        text.push_str(&format!(r#"## Vault: bootstrap e entrega por arquivos

Não há root token/senha embutidos. O Vault local usa armazenamento persistente e inicia selado/não inicializado; o operador deve inicializar, guardar recovery/unseal keys fora do repo e deselar. TLS desativado é exclusivamente para esse laboratório. Vault externo exige endereço/certificados/autenticação adequados; edite o endereço em infra/vault/agent.kubernetes.hcl antes de aplicar, quando externo.

### Compose

1. Prepare infra/.env e, se local, inicie somente Vault: `docker compose --env-file infra/.env up -d vault`.
2. Com Vault CLI instalado, exporte VAULT_ADDR=http://127.0.0.1:8200 para o operador (o Agent usa o endereço interno de infra/.env). Execute `vault operator init` e `vault operator unseal`, conforme o quorum retornado. Faça login sem gravar o token em arquivos versionados.
3. Habilite KV v2 e AppRole se ainda não existirem; não reconfigure mounts existentes sem revisar:

```sh
vault secrets enable -path=kv kv-v2
vault auth enable approle
vault policy write {name}-app infra/vault/app-policy.hcl
vault write auth/approle/role/{name}-app token_policies={name}-app token_ttl=1h token_max_ttl=4h
mkdir -p infra/vault/local
vault read -field=role_id auth/approle/role/{name}-app/role-id > infra/vault/local/role-id
vault write -field=secret_id -f auth/approle/role/{name}-app/secret-id > infra/vault/local/secret-id
chmod 600 infra/vault/local/role-id infra/vault/local/secret-id
```

4. Preencha o segredo `kv/{name}` com os campos que os templates solicitam: database_url, redis_url, ou app_secret quando não há banco/cache. Os URLs usados pelo Agent Compose devem apontar aos nomes internos database/cache. Prefira `vault kv put kv/{name} @arquivo.json`, com arquivo ignorado em infra/vault/local; não coloque senhas literais no histórico.
5. Inicie o restante. O Agent de uma execução renderiza arquivos, encerra com sucesso e só então a API inicia. Se o bootstrap falhar, consulte os logs do vault-agent e refaça somente após corrigir a identidade/policy/segredo. A API não recebe token de Vault.

### Kubernetes

O initContainer Vault Agent autentica usando o ServiceAccount api e o método Kubernetes do Vault; não requer Vault Injector ou webhook. Configure `auth/kubernetes`, o endereço/CA do API server, o token reviewer e a role `{name}-api` vinculada ao ServiceAccount api no namespace `{namespace}` e à policy {name}-app. Um Vault local recebe ServiceAccount vault e ClusterRoleBinding de TokenReview; é necessário permissão para aplicar esse recurso. Configure o método com o token projetado do próprio servidor segundo as instruções do Vault para esse ambiente.

```sh
vault auth enable kubernetes
# Vault rodando dentro do cluster usa o token e a CA do próprio ServiceAccount:
vault write auth/kubernetes/config kubernetes_host=https://kubernetes.default.svc
# Para Vault externo, forneça explicitamente host, CA e token reviewer adequados:
# vault write auth/kubernetes/config kubernetes_host="$KUBE_HOST" kubernetes_ca_cert=@"$KUBE_CA_FILE" token_reviewer_jwt=@"$KUBE_REVIEWER_FILE"
vault policy write {name}-app infra/vault/app-policy.hcl
vault write auth/kubernetes/role/{name}-api bound_service_account_names=api bound_service_account_namespaces={namespace} policies={name}-app ttl=1h
```

O initContainer escreve em emptyDir em memória; a API monta os arquivos read-only. DATABASE_URL_FILE/REDIS_URL_FILE (ou APP_SECRET_FILE no exemplo sem dados) são lidos e validados antes do listener. Valor direto e arquivo simultâneos causam erro, em vez de escolher um silenciosamente. APP_SECRET é um exemplo de configuração e não implementa autenticação da API. Segredos são lidos no bootstrap; após rotação, reinicie os pods/containers da API. Esta entrega não promete hot reload de segredos.

"#));
    }
    let path = PathBuf::from("docs/OPERACAO.md");
    files.insert(path, text);
    files.entry(PathBuf::from("README.md")).or_default().push_str("\nVeja [Operação e integrações](docs/OPERACAO.md) para configuração, bootstrap, validação e recuperação dos módulos escolhidos.\n");
}
