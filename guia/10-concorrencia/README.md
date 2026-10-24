# 10 — Ownership entre threads

[Índice da trilha](../README.md) · [Código executável](main.rs)

```sh
cargo run -p guia-rust --example concorrencia
```

Entender como posse e comunicação se combinam ao executar trabalho em paralelo.

- `thread::spawn` inicia uma thread do sistema operacional. `move` transfere os valores capturados pela closure; isso permite que ela possua os dados enquanto executa.
- O canal `mpsc` tem múltiplos produtores e um consumidor. Clonamos o remetente para cada worker, não os dados das tarefas.
- O iterador do receptor espera mensagens até todos os remetentes serem descartados. `drop(envio)` encerra o remetente que ficou na thread principal; esquecê-lo faz este programa esperar indefinidamente.
- `join()` espera o fim da thread e informa se ela entrou em panic. A ordem de chegada das mensagens é indeterminada; o exemplo ordena antes de imprimir.
- `Send` descreve tipos que podem ser transferidos entre threads; `Sync` descreve tipos que podem ser referenciados de forma compartilhada entre threads. São traits que o compilador usa para verificar segurança.
- `Arc<T>` fornece posse compartilhada com contagem atômica; `Mutex<T>` protege acesso exclusivo a dados compartilhados. Estude-os quando o problema exigir estado compartilhado.

### Comparação com TypeScript / Node.js

`Promise.all` coordena promises; não faz trabalho síncrono de CPU executar em várias threads por si só. `worker_threads` se aproxima mais do mecanismo usado neste exemplo, embora a troca e propriedade de dados sejam diferentes.

`async`/`await` em Rust trabalha com futures e não equivale a iniciar uma thread. Para executar futures você precisa de um executor; escolher um runtime e explorar I/O assíncrono fica como continuação da trilha.

### Experimente

1. Adicione mais nomes e confirme que todas as mensagens chegam.
2. Remova temporariamente `drop(envio)`, observe a espera e interrompa com Ctrl+C; depois restaure.
3. Explique por que uma referência a uma `String` local não pode ser enviada livremente para uma thread que pode sobreviver ao escopo.

Avance quando conseguir identificar quem possui cada nome, cada remetente e o receptor. A saída ordenada original contém `lib.rs` com 6 bytes e `main.rs` com 7. Criar uma thread por nome é apenas demonstração; uma ferramenta real deve limitar o número de workers.

Próximo passo: [projeto de consolidação](../projeto-final.md).
