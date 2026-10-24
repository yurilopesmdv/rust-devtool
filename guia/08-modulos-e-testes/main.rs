mod comando; // Carrega comando.rs nesta pasta.

use comando::{Comando, interpretar};

fn main() {
    match interpretar(&["hash", "Cargo.toml"]) {
        Ok(Comando::Ajuda) => println!("Ajuda solicitada"),
        Ok(Comando::Hash(caminho)) => println!("Calcular hash de {caminho}"),
        Err(erro) => eprintln!("Erro: {erro}"),
    }
}
