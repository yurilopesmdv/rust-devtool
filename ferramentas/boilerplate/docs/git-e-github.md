# Git local, GitHub privado e push

[Guia de uso](../README.md) · [Criação e instalação](criacao-e-instalacao.md)

## Git local por padrão

`new` verifica se Git está disponível, cria o projeto e inicializa um repositório com branch `main`. A implementação usa `git init` e `git symbolic-ref`, compatíveis com a versão Git instalada no WSL de validação. Não cria commit nem remote por padrão. Use `--git no` para desativar.

Monorepo tem um Git na raiz; split tem dois repositórios independentes. Não é criado Git dentro das aplicações de um monorepo. Configurações globais de usuário, email, templates, credenciais e aliases não são alteradas.

## GitHub é uma escolha separada

Em TTY, após gerar os fontes e tratar dependências, perguntar se deve criar o(s) repositório(s) privado(s). A resposta padrão é não. Em automação, GitHub também fica desativado sem `--github yes`.

Quando selecionado, verificar `gh --version` e `gh auth status --hostname github.com`, sem exibir tokens. Falta de gh/autenticação preserva os projetos locais e informa `gh auth login` como próximo passo. A ferramenta não instala gh nem inicia login automaticamente.

O nome remoto padrão vem do nome informado para o projeto, não do diretório de destino. Para personalizar: `--github-repo owner/repo`; split usa `--github-web owner/web` e `--github-api owner/api`. Sem owner, gh usa o contexto da conta autenticada. Em modo interativo o nome também pode ser revisado antes de criar.

Exemplo de criação privada **sem push**:

```sh
boilerplate new minha-api --type backend --backend express-ts \
  --destination ./minha-api --install no \
  --github yes --github-repo minha-org/minha-api --push no --non-interactive
```

O comando chama `gh repo create <nome> --private --source . --remote origin` na raiz de cada projeto. Não usa `--public`, não reutiliza silenciosamente um remote/repositório existente e não apaga repositórios em caso de falha. A visibilidade pública ainda não é uma opção desta interface.

## Push explícito

Em TTY, uma pergunta separada autoriza commit inicial e push, com padrão não. Em automação use `--github yes --push yes`; uma combinação conflitante retorna erro. Antes de criar os remotes com push, verificar se nome/email Git estão configurados em cada raiz. Nada é gravado na configuração global.

Quando autorizado, preparar os arquivos gerados respeitando .gitignore, fazer commit inicial e usar `gh repo create ... --push`. O fluxo não inicia um push posterior implícito. Se o remote for criado e uma etapa seguinte falhar, ele fica preservado; o diagnóstico exige recuperação manual, sem remover recursos remotos.

## Diagnóstico

```sh
boilerplate doctor --template express-ts --git --github
```

Doctor consulta instalação/versão de Git e gh e o estado de autenticação. Não cria repositórios nem publica conteúdo. Os testes deste projeto usam gh simulado para verificar argumentos, autenticação e ausência de push; nenhuma conta ou repositório real é utilizado nos testes.

A criação de remotes segue a [documentação oficial de gh repo create](https://cli.github.com/manual/gh_repo_create).
