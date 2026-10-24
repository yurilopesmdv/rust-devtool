# Ferramentas para desenvolvimento

Este diretório recebe os utilitários de terminal construídos ao aplicar o que foi aprendido no [guia](../guia/README.md).

A [devtool](devtool/README.md) já contém os experimentos de hash SHA-256 e formatação de JSON. O comportamento desses comandos foi preservado nesta reorganização.

Para adicionar uma ferramenta no futuro, crie um pacote em uma subpasta e registre seu caminho em `members` no `Cargo.toml` da raiz. Cada ferramenta deve ter seu próprio README com objetivo, exemplos de uso e testes dos comportamentos relevantes. Bibliotecas compartilhadas podem ganhar uma pasta `crates/` quando houver necessidade real de reutilização.

Ideias para depois da trilha: busca em logs, inspeção de configurações, validação de JSON e resumo de arquivos. A instalação de comandos no terminal fica para essa etapa futura.
