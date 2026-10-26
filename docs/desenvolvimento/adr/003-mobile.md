# ADR 003 — Entrada futura de mobile

Estado: contrato de descoberta; framework/plataformas aguardam a fase mobile com o usuário.

## Escopo

O produto atual continua web. Não registrar IDs mobile ready, instalar SDKs, iniciar emuladores, criar contas cloud ou gerar arquivos de assinatura nesta fase. A CLI continua sendo executada no WSL/Linux; futuros builds precisam informar claramente onde são executáveis.

## Decisões a tomar com o usuário no início da fase

| Escolha | Pergunta | Efeito no boilerplate |
| --- | --- | --- |
| Plataformas | Android, iOS ou ambas? Também haverá versão web? | SDKs, runners e testes necessários |
| Framework | React Native com Expo, React Native com projeto nativo ou Flutter? | Linguagem, manifesto, navegação, testes e dependências |
| Integrações nativas | Há módulos/permissões que exigem projeto nativo próprio? | Customização e manutenção de Android/iOS |
| Build | Local, runner mantido pelo usuário ou serviço remoto optativo? | Requisitos, contas, custo e autorização explícita |
| API | Backend existente ou composição fullstack? Como o dispositivo acessa a API? | URL por ambiente, rede, TLS e diagnóstico |
| Distribuição | Só desenvolvimento ou futura loja? | Identificadores e assinatura em etapa separada |

React Native/Expo é candidato por aproveitar TypeScript, mas isso não decide a preferência do usuário. Flutter exige Dart e outra cadeia de build; o contrato de qualidade continua equivalente. A [documentação de ambiente React Native](https://reactnative.dev/docs/set-up-your-environment) deve orientar a matriz de SDKs. Build iOS local requer ambiente Apple apropriado; não prometer compilação iOS dentro do WSL. [EAS Build](https://docs.expo.dev/build/introduction/) é uma possibilidade remota, a ser escolhida e autorizada separadamente, não um passo automático de geração.

## Arquitetura e configuração propostas

Organização por funcionalidade: screens, components, navigation, services de API e contratos; configuração de ambientes fica fora das regras de UI. Armazenamento local, autenticação, analytics e notificações são adapters optativos, cada um com testes e permissões documentados. Não incluir todas as integrações no pacote mínimo.

URL acessível pelo dispositivo não é necessariamente localhost do WSL. Definir estratégia de desenvolvimento por dispositivo/emulador e documentar o endereço efetivo. Nenhuma chave secreta de backend entra no bundle mobile. Entrega de segredos, armazenamento seguro e refresh de sessão exigem desenho específico na fase correspondente.

## Microentregas e aceite

- MB001: coletar as decisões acima e registrar matriz host/plataforma/framework e versões fixadas; não começar pelo SDK mais complexo sem necessidade.
- MB002: base mínima com uma tela e serviço de API configurável, lint/typecheck/formatação/testes, build ou execução real em uma plataforma escolhida e documentação de pré-requisitos.
- MB003: navegação e composição com backend existente/gerado; provar acesso de dispositivo e tratar erro/offline sem simular conectividade como sucesso.
- MB004: um adapter por vez (armazenamento/auth/notificação), com permissões explícitas e testes; none omite código e configuração.
- MB005: assinatura/distribuição opcionais, credenciais fora do Git e execução remota somente por escolha explícita; nunca publicar em loja como efeito de new.

A expansão pode exigir nova família de projeto no schema da receita. Essa alteração terá validação e migração próprias, sem reutilizar frontend web para anunciar uma compatibilidade mobile que ainda não existe.
