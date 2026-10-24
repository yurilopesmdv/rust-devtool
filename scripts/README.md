# Scripts de desenvolvimento

[Contribuição](../CONTRIBUTING.md) · [Microtasks](../docs/desenvolvimento/README.md)

Os scripts não são comandos da devtool nem templates de aplicação; são ferramentas para manter este repositório. Execute a partir da raiz, salvo indicação diferente.

## Workspace Rust

```sh
sh scripts/check-workspace.sh
sh scripts/check-workspace.sh --offline
sh scripts/tests/check-workspace.sh
```

Verifica formatação, todos os targets de teste, doctests e Clippy. Funciona por caminho absoluto a partir de outro diretório. Não instala componentes nem inicia serviços.

## Links locais de Markdown

Python 3.8+ com biblioteca padrão, sem pip:

```sh
python3 scripts/check_docs.py
python3 -m unittest discover -s scripts/tests -p 'test_check_docs.py'
```

Verifica arquivos/diretórios de destino e fragmentos de arquivos Markdown. Suporta links inline, imagens, referências, caminhos relativos ou relativos à raiz, URLs percent-encoded e destinos entre `<...>` com espaços. Âncoras vêm de títulos ATX, títulos Setext simples, títulos repetidos e atributos HTML id/name. Links fora da raiz do repositório são rejeitados.

Não acessa a rede: esquemas externos e URLs //host são ignorados. Blocos cercados por crases/til e comentários HTML não são inspecionados; links em código inline também são ignorados. Não percorre target, node_modules, .venv, caches e symlinks de diretório. O utilitário cobre os formatos de documentação usados aqui; não substitui um renderer CommonMark completo nem verifica fragmentos de arquivos HTML/JSX.

O checker retorna 1 se há link local inválido e mostra arquivo, linha e destino; argumentos inválidos retornam 2. Para uma árvore de teste, use --root /caminho/da/arvore. O workflow Quality executa seus testes e o check em um job próprio, sem exigir toolchain Rust.

## Aplicações geradas

O runner específico permanece em [validate-generated.py](../ferramentas/boilerplate/scripts/validate-generated.py). Consulte [validação de boilerplates](../ferramentas/boilerplate/docs/validacao.md) para seleção de stacks, modo mínimo e serviços descartáveis.
