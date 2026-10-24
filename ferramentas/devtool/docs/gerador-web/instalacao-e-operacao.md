# Instalação, dependências e documentação de operação

[Índice](README.md). Esta página distingue comandos existentes de instruções planejadas.

## O que funciona hoje

Com Rust/Cargo instalados, na raiz deste repositório:

```sh
cargo run -p rust-devtool -- help
cargo run -p rust-devtool -- hash ferramentas/devtool/fixtures/exemplo.txt
cargo run -p rust-devtool -- json format ferramentas/devtool/fixtures/exemplo.json
```

Ainda não há binário chamado `devtool` com o gerador, instalador próprio ou templates. Não use os exemplos da especificação esperando gerar projetos nesta revisão.

## Distribuição planejada

Adicionar o nome de binário `devtool` sem remover a compatibilidade com `rust-devtool`. Após essa implementação, a instalação local com Cargo deverá ser documentada usando o caminho real do pacote. Um instalador opcional POSIX sh poderá orientar o build/instalação e verificar o PATH, sem alterar arquivos de configuração do shell automaticamente.

A experiência deve seguir o padrão encontrado no repositório `tools`: comando no PATH, utilitário documentado, dependências verificadas e modo interativo. O núcleo Rust evita tornar Bash ou `fzf` obrigatórios para a geração. MacOS e Windows nativo entram apenas quando seus testes forem incluídos; o alvo inicial é Linux/WSL.

## Dependências por operação

| Operação | Requisito |
| --- | --- |
| Gerar arquivos com binário distribuído | Binário devtool; sem Node/Docker obrigatórios para renderizar |
| Compilar o gerador localmente | Rust/Cargo e componentes conforme o guia do repositório |
| Executar frontend vanilla | Navegador e servidor HTTP local documentado |
| Instalar/executar stacks Node | Node e gerenciador validado pelo template |
| Instalar/executar API Rust | Toolchain Rust compatível com o template |
| Executar ambiente Compose | Docker Engine/alternativa compatível e plugin Compose |
| Construir/publicar imagens | Ferramenta de build e registry quando necessário |
| Usar manifests Kubernetes | kubectl, cluster/contexto e requisitos do overlay |
| Integrar Vault/monitoria externa | Instância, acesso e configuração já preparados |

Apenas gerar arquivos não exige daemons. `doctor` deve diferenciar dependência ausente para geração, instalação ou execução; não exigir Kubernetes quando a opção estiver desativada e não sugerir que um Docker daemon inacessível impede renderizar um Dockerfile.

Versões mínimas e comandos de instalação serão fixados no catálogo junto dos templates validados. Não publicar uma tabela de versões presumidas antes de testar a combinação. Documentar links oficiais e instruções específicas para Linux/WSL; instalações de ferramentas não devem usar sudo automaticamente.

## Documentação obrigatória dentro de cada projeto gerado

O README é renderizado de acordo com a seleção; não é uma lista genérica de todos os serviços possíveis. Deve conter:

1. Resumo da stack, versões, arquitetura e árvore de diretórios realmente gerada.
2. Requisitos para gerar, instalar, executar e testar, com links e comandos de instalação do ambiente suportado.
3. Configuração local: arquivos a copiar, variáveis obrigatórias, valores de exemplo sem credenciais reais e validação de erros.
4. Comandos de instalação/build/dev/test/lint e resultado esperado, com diretório de execução explícito.
5. Portas/URLs, rotas de exemplo, health checks e integração frontend/API.
6. Comandos de dados: migrations SQL ou preparação de índices/documentos conforme o banco; comandos de cache quando necessário.
7. Guia Docker/Compose, se selecionado: preparar valores, construir, iniciar, verificar logs, parar e preservar volumes.
8. Guia Kubernetes, se selecionado: requisitos, referências de imagem, configuração, renderização, revisão e aplicação manual, além de verificação e remoção dos recursos do projeto.
9. Guia de monitoria, se selecionada: endereço do endpoint, targets, datasource, dashboard e como verificar a coleta.
10. Guia de Vault, se selecionado: identidade/policy/path, bootstrap manual, entrega de segredos, validação e limites de rotação.
11. Erros comuns e recuperação: porta ocupada, serviço inacessível, credenciais ausentes, CORS, imagem não encontrada, storage e namespaces.
12. Limitações conhecidas e decisões para levar o laboratório a outro ambiente.

Documentação complementar deve estar em `docs/` apenas quando a opção estiver ativa: arquitetura, desenvolvimento, integração fullstack, dados, containers, Kubernetes, monitoria e segredos. Evitar links a arquivos omitidos.

## Instalação de dependências ao final da geração

O assistente apresenta o comando e pede a escolha instalar/pular. Para vanilla sem pacotes, informa que a etapa não se aplica. No fullstack, mostra os diretórios e operações de cada aplicação; uma falha não é anunciada como sucesso total.

Instalar dependências pode executar scripts definidos pelo ecossistema. O resumo informa esse efeito e as versões previstas. A instalação não inicia serviços, aplica migrations, baixa charts ou configura um cluster. Quando o usuário pula, o projeto ainda deve conter instruções suficientes para executar a instalação posteriormente.

Sem terminal interativo, não perguntar. `--install yes` autoriza a instalação de dependências da aplicação; `--install no` é o padrão de automação. Uma falha de instalação retorna erro, preserva o projeto e imprime o diretório e o comando para repetir.
