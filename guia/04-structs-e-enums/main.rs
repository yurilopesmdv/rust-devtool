#[derive(Debug)]
struct Configuracao {
    nome: String,
    silencioso: bool,
}

impl Configuracao {
    fn nova(nome: &str) -> Self {
        Self {
            nome: nome.to_owned(),
            silencioso: false,
        }
    }

    fn exibir(&self) {
        if !self.silencioso {
            println!("Ferramenta: {}", self.nome);
        }
    }
}

#[derive(Debug)]
enum Comando {
    Ajuda,
    Hash { caminho: String },
}

fn executar(comando: Comando) {
    match comando {
        Comando::Ajuda => println!("Use hash <arquivo>"),
        Comando::Hash { caminho } => println!("Arquivo selecionado: {caminho}"),
    }
}

fn main() {
    let config = Configuracao::nova("devtool");
    config.exibir();
    println!("Debug: {config:?}");
    executar(Comando::Ajuda);
    executar(Comando::Hash {
        caminho: String::from("Cargo.toml"),
    });
    let arquivo: Option<&str> = Some("Cargo.toml");
    if let Some(caminho) = arquivo {
        println!("Argumento opcional presente: {caminho}");
    }
    let argumentos = ["devtool"];
    println!(
        "Ausente: {}",
        argumentos.get(1).unwrap_or(&"nenhum arquivo")
    );
}
