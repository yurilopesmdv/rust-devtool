# Arquitetura atual do pacote

[Guia de uso](../README.md) · [Especificação](README.md)

`rust-boilerplate` é um pacote Cargo independente de `rust-devtool`. Não importa módulos dos utilitários, não altera seu dispatcher e possui seu próprio executável e biblioteca. Ambos compartilham apenas o workspace/lockfile, como as aulas.

## Caminho de uma operação

```text
CLI/flags ─────────────┐
                      ├── Recipe tipada ── validação/planner ── Plan ── texto/JSON
TOML/schema validado ─┘                          ↑
                                           catálogo embutido
```

`cli.rs` interpreta argumentos com Clap e aplica somente flags presentes sobre a receita. `config.rs` desserializa TOML via Serde, rejeita campos desconhecidos e verifica versões. IDs de templates são strings consultadas no catálogo: não são um enum de linguagens fechado.

`catalog.rs` carrega metadados validados. `build.rs` descobre automaticamente os arquivos `.toml` de `catalog/`, ordena-os e embute seu conteúdo durante a compilação. O catálogo distribuído acompanha a versão do binário; editar arquivos ao lado de um binário já instalado não altera o comportamento dele.

`planner.rs` valida papéis, destinos, layouts, escolhas de serviços e capacidades. Calcula caminhos a partir dos metadados, com caminhos condicionais por integração. Não executa comandos externos, não consulta runtimes nem escreve no destino. Consultas de filesystem servem para verificar destinos e resolver ancestrais existentes.

`doctor.rs` executa consultas de diagnóstico. A geração tem sua própria camada de operações em `src/generator/`: runtime, Git/GitHub e instalação. Usa `std::process::Command` com programa/argumentos separados e stdin fechado. Os comandos de versão vêm do catálogo confiável embutido ou da lista interna de ferramentas de infraestrutura; receitas não podem introduzir comandos arbitrários.

`main.rs` cuida da apresentação e dos códigos de saída. Planos e diagnósticos podem ser consumidos em JSON sem interpretar texto de terminal. A biblioteca permite testar o núcleo sem simular entrada interativa.

## Camadas da geração

O assistente já produz a mesma Recipe das flags. O renderer usa fontes/lockfiles embutidos e arquivos condicionais; o writer usa staging e rename sem substituição. O runtime instala apenas quando escolhido, com nvm/Node já existentes ou preparados após autorização. VCS trata Git local, GitHub privado e push em etapas separadas.

Veja [manutenção por agentes](manutencao-por-agentes.md) para os módulos, contratos e testes.

Nenhuma dessas camadas inicia infraestrutura. Os arquivos de Compose/Kubernetes e seus guias pertencem ao projeto gerado. As execuções avulsas de utilitários continuam exclusivamente em devtool.

As capacidades das oito bases têm adapters em src/generator/integrations, com fontes e lockfiles de apoio em integrations/. Status ready indica a base gerável; planning-only descreve a saída sem efeitos de plan. Novas stacks precisam de adapter próprio para anunciar integrações. O núcleo continua independente de devtool.

O renderer também compõe docs/ARQUITETURA.md e passa os fontes por generator/format.rs antes da publicação: dprint JS/TS embutido; rustfmt para Rust. Python usa fontes/fragmentos previamente formatados e uma matriz validada com Ruff. Os adapters selecionam lockfiles exatos por combinação; o script de validação descobre as stacks prontas no catálogo e escolhe checks pelo gerenciador.
