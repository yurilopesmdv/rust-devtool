# Instalação, dependências e documentação de operação

[Índice](README.md). Esta página distingue comandos existentes de instruções planejadas.

## O que funciona hoje

Dentro de WSL/Linux, com Rust/Cargo instalados, na raiz deste repositório:

```sh
cargo run -p rust-boilerplate -- templates list
cargo run -p rust-boilerplate -- plan minha-api --type backend --backend express-ts
cargo run -p rust-boilerplate -- doctor --template express-ts
```

Para instalação no PATH e exemplos completos, consulte o [README do pacote](../README.md). As oito bases já são geráveis com `new`, incluindo instalação local de dependências quando escolhida; veja [criação e instalação](criacao-e-instalacao.md) e [Git/GitHub](git-e-github.md). A devtool continua separada e seus comandos estão no [guia dos utilitários](../../devtool/README.md).

## Distribuição

O executável `boilerplate` pertence ao pacote `rust-boilerplate`. Pode ser instalado com `cargo install --path ferramentas/boilerplate --locked`, a partir da raiz. Um instalador auxiliar poderá ser adicionado depois; não é necessário para usar o pacote hoje.

A experiência segue os exemplos do repositório `tools`: comando no PATH, documentação por ferramenta e dependências verificadas. O núcleo Rust não exige Bash ou fzf para planejar. O ambiente de execução e validação é WSL/Linux; Windows nativo fica fora do escopo.

## Dependências por operação

| Operação | Requisito |
| --- | --- |
| Gerar arquivos com binário distribuído | Binário boilerplate para JS/TS/Python; Rust também requer rustfmt; sem daemon Docker |
| Compilar o gerador localmente | Rust/Cargo e componentes conforme o guia do repositório |
| Executar frontend vanilla | Navegador e servidor HTTP local documentado |
| Instalar/executar stacks Node | Node e gerenciador validado pelo template |
| Instalar/executar API Rust | Toolchain Rust compatível com o template |
| Instalar/executar API Python | uv; Python 3.13 é selecionado/obtido pelo uv |
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

O assistente apresenta o comando e pede a escolha instalar/pular. Em vanilla, a instalação prepara somente as ferramentas locais de desenvolvimento. No fullstack, mostra os diretórios e operações de cada aplicação; uma falha não é anunciada como sucesso total.

Instalar dependências pode executar scripts definidos pelo ecossistema. O resumo informa esse efeito e as versões previstas. A instalação não inicia serviços, aplica migrations, baixa charts ou configura um cluster. Quando o usuário pula, o projeto ainda deve conter instruções suficientes para executar a instalação posteriormente.

Sem terminal interativo, não perguntar. `--install yes` autoriza a instalação de dependências da aplicação; é obrigatório informar `--install yes` ou `--install no` em automação. Uma falha de instalação retorna erro, preserva o projeto e imprime o diretório e o comando para repetir.
