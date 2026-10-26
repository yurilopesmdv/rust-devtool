# ADR 002 — Contrato para Java/Spring

Estado: aceito para a futura fase Java; nenhum template anunciado no catálogo.

## Decisão

Entregar primeiro uma API HTTP mínima com Spring MVC, organização por funcionalidade e sem banco. Repository e infraestrutura entram em adapter posterior. Não adicionar JPA/ORM, segurança de login, Kafka ou um cluster como requisito da base.

| Ponto | Padrão proposto | Critério antes de implementar |
| --- | --- | --- |
| JDK | LTS de distribuição OpenJDK mantida, instalada no Linux/WSL | Escolher versão exata após verificar suporte da versão Spring e executar javac/test/build; doctor verifica major e origem informada pelo usuário |
| Framework | Spring Boot estável com BOM versionado | Consultar requisitos oficiais e conferir transitivas; sem SNAPSHOT/latest e sem copiar grafo de outro framework |
| Build | Maven Wrapper por projeto | Versionar mvnw e propriedades com distribuição/checksum fixados; executar sem Maven global em HOME temporário |
| Layout | src/main/java com pacote válido e módulos por funcionalidade | groupId/artifactId/package perguntados ou derivados com validação; hífen do nome do projeto não vira pacote Java inválido |
| Testes | JUnit e teste HTTP do framework, primeiro sem serviços | Health/readiness, validação Unicode, CORS, configuração inválida, erro sanitizado e SIGTERM |
| Formatação/lint | Spotless com formatter fixado, compilador e verificações estáticas revisadas | Um comando verify reproduz o CI; plugins versionados e nenhuma correção silenciosa em modo check |
| Dados | Adapter PostgreSQL com pool e repository explícitos | Preparação versionável/explicitamente separada de startup; testes com banco descartável |

Maven foi preferido pela convenção de layout e lifecycle previsíveis para a primeira base. Gradle foi considerado por sua DSL e flexibilidade, mas adicionaria outra matriz de wrapper/plugins antes de haver necessidade. O wrapper permite executar o build com a distribuição do projeto, conforme a [documentação Apache](https://maven.apache.org/tools/wrapper/). Só o script Linux é distribuído nesta ferramenta WSL; não adicionar arquivos nativos Windows por padrão.

Não fixar aqui uma combinação JDK/Spring ainda não testada. A fonte de decisão será a [matriz oficial de requisitos Spring Boot](https://docs.spring.io/spring-boot/system-requirements.html), consultada novamente no início da implementação. A versão escolhida deverá aparecer em manifesto, wrapper, doctor, Docker/CI e documentação antes de marcar ready.

## Microentregas e aceite

- J001: inventário das versões e licenças, escolha JDK/BOM/wrapper/formatter, prova de build em /tmp sem ferramentas globais e sem execução de aplicações externas.
- J002: base sem dados, fontes formatados, testes HTTP e encerramento real, geração isolada e fullstack nos dois layouts; publicar ID ready somente após evidência.
- J003: PostgreSQL opcional, pool e repository com queries, preparação explícita, readiness/shutdown e persistência real; none não carrega driver.
- J004: Compose/Kubernetes e CI opcionais, smoke de imagem não root e política de atualização do grafo Maven.

A primeira entrega não precisa oferecer várias arquiteturas. O mapa controller → service → repository é o padrão comum; detalhes do Spring ficam no adapter e não em switches no assistente Rust. O catálogo descreve requisitos e capacidades efetivamente testados.
