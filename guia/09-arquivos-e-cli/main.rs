use std::{env, error::Error, fs, process::ExitCode};

fn executar() -> Result<(), Box<dyn Error>> {
    // args_os aceita caminhos que não são Unicode; o termo precisa ser texto UTF-8.
    let mut argumentos = env::args_os().skip(1);
    let termo = argumentos.next().ok_or("uso: busca <termo> <arquivo>")?;
    let termo = termo.into_string().map_err(|_| "o termo deve ser UTF-8")?;
    let caminho = argumentos.next().ok_or("informe o arquivo de entrada")?;
    if argumentos.next().is_some() {
        return Err("argumentos excedentes; uso: busca <termo> <arquivo>".into());
    }
    let conteudo = fs::read_to_string(caminho)?;
    for (indice, linha) in conteudo.lines().enumerate() {
        if linha.contains(&termo) {
            println!("{}:{linha}", indice + 1);
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match executar() {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            eprintln!("Erro: {erro}");
            ExitCode::FAILURE
        }
    }
}
