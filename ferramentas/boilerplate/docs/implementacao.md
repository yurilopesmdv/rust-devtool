# Plano de implementação e validação

[Índice](README.md). Etapas 1–3 entregues: núcleo, assistente e geração das seis bases, instalação de dependências e Git/GitHub. As etapas de dados/cache, Kubernetes, monitoria e Vault também estão implementadas. A execução Docker/cluster depende do ambiente de destino; veja validacao.md.

## Organização interna

O pacote independente mantém catálogo e planejamento separados dos utilitários de devtool e das aulas:

```text
ferramentas/boilerplate/
  build.rs                      # embute os TOMLs do catálogo na compilação
  catalog/                      # um arquivo de metadados por stack
  src/
    lib.rs                      # núcleo reutilizável
    main.rs                     # saída, códigos e despacho da CLI
    cli.rs                      # argumentos e precedência sobre receitas
    config.rs                   # opções tipadas e schema TOML
    catalog.rs                  # metadados, capacidades e validação do catálogo
    planner.rs                  # validação e plano sem escrita de projetos
    doctor.rs                   # consultas de versão de ferramentas
  recipes/
  tests/
  docs/
```

Renderização, escrita, instalação, Git/GitHub e assistente já estão em módulos de `src/generator/`. As fontes e lockfiles estão em `templates/<id>/`, separados do catálogo. Não será necessário colocar lógica do gerador em `ferramentas/devtool/src`.

O catálogo já é embutido no binário; sua execução não depende do cwd nem de acesso ao clone. Veja a [arquitetura do pacote](arquitetura-do-pacote.md) e [como adicionar uma stack](adicionar-stack.md).

Instaladores oficiais de Vite/Next/Nest podem ajudar a atualizar bases durante a manutenção. Invocá-los no runtime deve ser um modo explícito posterior, porque baixar um scaffolder é diferente de renderizar offline e pode instalar pacotes ou alterar defaults. A proposta inicial é distribuir bases testadas, com versões fixadas, e registrar sua origem.

## Configuração única

Wizard, flags e receita alimentam a mesma representação tipada. O planner resolve integrações e valida capacidades antes de tocar o destino. Toda omissão é resolvida no plano; nenhum renderer pode reativar uma escolha `none`.

O catálogo descreve: ID/versão, linguagem/framework, runtimes, gerenciador de pacotes, capacidades, dependências compatíveis, opções de instalação, arquivos condicionais e documentação. Schema de receita é versionado independentemente da versão da CLI.

Cada extensão deve implementar os pontos de integração concretos: imports, bootstrap, configuração, shutdown, arquivos de dependência, testes e documentação. Não basta copiar um YAML. O planejador atual detecta destinos inválidos/sobrepostos e valida dependências entre escolhas. A detecção de conflitos de arquivos renderizados e placeholders pendentes será implementada com o renderer.

Publicar arquivos via staging no mesmo filesystem, verificando que o destino continua ausente. No layout de dois repositórios, renderizar/validar ambos antes de publicar; documentar recuperação se a publicação do segundo falhar. Nunca apagar diretórios preexistentes nem usar um rollback que remova arquivos que não pertencem à execução.

## Entregas incrementais

| Etapa | Entrega | Critério de conclusão |
| --- | --- | --- |
| 0 — concluída | Especificação e catálogo pretendido | Fluxo, opções, limites e documentação registrados |
| 1 — concluída | Núcleo Rust + templates list/show + plan + doctor | Receitas/flags validadas, plano sem efeitos e requisitos por operação |
| 2 — concluída | new + assistente + Express/TS e React/Vite | Geração individual e fullstack; testes/build reais das bases; instalação opcional |
| 3 — concluída | Vanilla, Next, Nest e Axum | Seis bases e nove pares mínimos validados; organização respeita cada stack |
| 4 — implementada | PostgreSQL, MongoDB e Redis | Integrações por backend e ausência completa quando desativadas |
| 5 — gerada/validada como configuração | Docker/Compose e contratos web/API | Imagens construídas, conexão válida e redes documentadas para os dois layouts |
| 6 — gerada/validada como configuração | Kubernetes | Manifests válidos, saúde/Services/imagens/volumes consistentes e guia manual |
| 7 — implementada | Prometheus/Grafana | Métricas reais coletáveis, targets e dashboards funcionais |
| 8 — implementada | Vault | Aplicação lê segredos pelo contrato; integração Compose/Kubernetes validada |
| 9 — documentação entregue | Distribuição e documentação final | Instalação/distribuição do boilerplate, independência da devtool e guias completos revisados |

As etapas organizam o trabalho; não retiram opções do escopo final aprovado. Só anunciar uma integração como disponível depois de validá-la. Não transformar uma fase parcial em um catálogo que aceita flags sem implementar seu efeito.

## Estratégia de testes

- Unitários: validação de escolhas, `none`, dependências de módulos, nomes/caminhos inválidos, precedência de flags/receitas e ausência de interação em CI.
- Geração em diretórios temporários: rejeitar destino existente/symlink, diretórios sobrepostos, cancelamento, falhas de escrita, placeholders não resolvidos e proteção de conteúdo preexistente.
- Conteúdo: caminhos, imports, manifestos de dependências e documentação refletem escolhas. Testar explicitamente que banco/cache/monitoria/Vault desativados não aparecem.
- Bases: instalar com versões controladas e executar build, testes e lint de cada stack, incluindo biblioteca/API Rust e health HTTP real.
- Combinações: nove pares mínimos; cada backend com cada banco; cache ligado/desligado; ambos os layouts; Kubernetes com imagem existente e com Dockerfile; serviços locais/externos; combinações críticas completas.
- Infraestrutura: validar Compose, construir imagens e validar/renderizar Kubernetes. YAML sintaticamente válido sozinho não comprova conexão funcional.
- Integração em ambiente de teste isolado: frontend alcança API; dados/cache funcionam; métricas chegam ao Prometheus e datasource/dashboard ao Grafana; segredo chega ao processo pelo Vault.
- Instalação: simular e testar falhas do gerenciador preservando arquivos; não iniciar serviços como efeito lateral.
- Documentação: links locais, comandos no diretório indicado, portas consistentes e guias condicionais. Smoke tests reais devem acompanhar os comandos publicados.

Execuções de teste de infraestrutura pertencem a ambiente isolado de validação/CI, não ao comando que gera projetos. Não operar cluster pessoal ou serviço externo como efeito de um teste padrão.

## Critério de funcionalidade pronta

Uma combinação só está pronta quando: arquivos são gerados corretamente; dependências e build passam; comportamento demonstrativo funciona; opções ausentes não deixam código residual; os passos documentados reproduzem o ambiente; e os limites estão explícitos. As aulas e os comandos existentes continuam verificados.

O núcleo já tem testes de validação, nove pares fullstack, extensão declarativa de linguagem, omissão de módulos, caminhos/symlinks, receitas, precedência e CLI sem ferramentas externas. Os testes de geração também verificam arquivos, layouts, ESM/CommonJS, staging, ausência de escrita para adapters indisponíveis e Git/GitHub com comandos simulados. As quatro bases Node foram instaladas e passaram em typecheck/test/build; Rust/Axum passou em test/Clippy. As combinações de dados foram executadas com serviços locais autenticados e Vault Agent real; containers/cluster não foram executados. Consulte [validação](validacao.md) para a evidência e reprodução.
