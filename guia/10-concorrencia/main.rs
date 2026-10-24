use std::{sync::mpsc, thread};

fn main() {
    let tarefas = vec![String::from("main.rs"), String::from("lib.rs")];
    let (envio, recebimento) = mpsc::channel();
    let mut workers = Vec::new();

    for nome in tarefas {
        let envio_worker = envio.clone();
        workers.push(thread::spawn(move || {
            // O worker recebe a posse do nome e de um remetente do canal.
            let bytes = nome.len();
            envio_worker
                .send((nome, bytes))
                .expect("o receptor permanece aberto enquanto os workers executam");
        }));
    }
    drop(envio); // Sem isso, o receptor esperaria este remetente se encerrar.
    let mut resultados: Vec<_> = recebimento.into_iter().collect();
    for worker in workers {
        worker.join().expect("worker não deve entrar em panic");
    }
    resultados.sort_by(|a, b| a.0.cmp(&b.0));
    println!("Bytes por nome: {resultados:?}");
}
