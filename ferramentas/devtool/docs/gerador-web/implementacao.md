# Plano de implementação e validação

[Índice](README.md). Nenhuma etapa de código do gerador foi entregue nesta especificação.

## Organização interna proposta

Evoluir o pacote existente sem misturar templates com as aulas:

```text
ferramentas/devtool/
  src/
    commands/                   # preserva hash/json; acrescenta novos comandos
    generator/
      config.*                  # escolhas tipadas, receitas e versão de schema
      catalog.*                 # metadados e capacidades dos templates
      validation.*              # restrições e combinações suportadas
      planner.*                 # plano determinístico, sem efeitos
      renderer.*                # arquivos, substituições e omissão de módulos
      writer.*                  # staging, publicação e limpeza de falhas
      installer.*               # processos de instalação selecionados
      wizard.*                  # perguntas convertidas na mesma configuração
  templates/
    frontends/
    backends/
    integrations/               # dados, cache, containers, Kubernetes e operação
  tests/
  docs/gerador-web/
```

São caminhos previstos, ainda não criados como módulos. Templates usados pelo binário devem acompanhar a distribuição: não depender do cwd ou de um clone do repositório. Avaliar inclusão no binário ou pacote de catálogo versionado; a primeira implementação deve privilegiar templates locais versionados e geração sem rede.

Instaladores oficiais de Vite/Next/Nest podem ajudar a atualizar bases durante a manutenção. Invocá-los no runtime deve ser um modo explícito posterior, porque baixar um scaffolder é diferente de renderizar offline e pode instalar pacotes ou alterar defaults. A proposta inicial é distribuir bases testadas, com versões fixadas, e registrar sua origem.

## Configuração única

Wizard, flags e receita alimentam a mesma representação tipada. O planner resolve integrações e valida capacidades antes de tocar o destino. Toda omissão é resolvida no plano; nenhum renderer pode reativar uma escolha `none`.

O catálogo descreve: ID/versão, linguagem/framework, runtimes, gerenciador de pacotes, capacidades, dependências compatíveis, opções de instalação, arquivos condicionais e documentação. Schema de receita é versionado independentemente da versão da CLI.

Cada extensão deve implementar os pontos de integração concretos: imports, bootstrap, configuração, shutdown, arquivos de dependência, testes e documentação. Não basta copiar um YAML. O planejador detecta conflitos de caminho, placeholders pendentes e referências a serviços omitidos.

Publicar arquivos via staging no mesmo filesystem, verificando que o destino continua ausente. No layout de dois repositórios, renderizar/validar ambos antes de publicar; documentar recuperação se a publicação do segundo falhar. Nunca apagar diretórios preexistentes nem usar um rollback que remova arquivos que não pertencem à execução.

## Entregas incrementais

| Etapa | Entrega | Critério de conclusão |
| --- | --- | --- |
| 0 | Especificação e catálogo pretendido | Fluxo, opções, limites e documentação registrados |
| 1 | Núcleo Rust + templates list/show + plan + doctor | Receitas/flags validadas, plano sem efeitos e requisitos por operação |
| 2 | new + assistente + Express/TS e React/Vite | Geração individual e fullstack; testes/build reais das bases; instalação opcional |
| 3 | Vanilla, Next, Nest e Axum | Seis bases e nove pares mínimos validados; organização respeita cada stack |
| 4 | PostgreSQL, MongoDB e Redis | Integrações por backend e ausência completa quando desativadas |
| 5 | Docker/Compose e contratos web/API | Imagens construídas, conexão válida e redes documentadas para os dois layouts |
| 6 | Kubernetes | Manifests válidos, saúde/Services/imagens/volumes consistentes e guia manual |
| 7 | Prometheus/Grafana | Métricas reais coletáveis, targets e dashboards funcionais |
| 8 | Vault | Aplicação lê segredos pelo contrato; integração Compose/Kubernetes validada |
| 9 | Distribuição e documentação final | Instalação de devtool, compatibilidade anterior e guias completos revisados |

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

Não há testes de runtime novos nesta fase documental. Quando começar a implementação, manter os checks do workspace e acrescentar testes conforme as responsabilidades forem implementadas.
