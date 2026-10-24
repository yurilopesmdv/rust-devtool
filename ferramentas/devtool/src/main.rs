use std::env;
mod commands;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("DevTool CLI");
        println!("Uso: devtool <comando>");
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "hello" => {
            println!("Olá! Bem-vindo ao DevTool CLI!");
        }
        "version" => {
            println!("DevTool CLI versão 0.0.1");
        }
        "help" => {
            println!("Comandos disponíveis:");
            println!("  hello   - Exibe uma mensagem de boas-vindas");
            println!("  version - Exibe a versão do DevTool CLI");
            println!("  help    - Exibe esta mensagem de ajuda");
            println!("  hash <arquivo> - Calcula o SHA-256 de um arquivo");
        }

        "json" => {
            if args.len() < 4 {
                eprintln!("Uso: devtool json format <arquivo>");
                std::process::exit(1);
            }

            let action = args[2].as_str();
            let file_path = args[3].as_str();

            match action {
                "format" => match commands::json::format_json(file_path) {
                    Ok(formatted) => println!("{}", formatted),
                    Err(error) => {
                        eprintln!("Erro: {}", error);
                        std::process::exit(1);
                    }
                },

                _ => {
                    eprintln!("Ação JSON desconhecida: {}", action);
                    std::process::exit(1);
                }
            }
        }

        "hash" => {
            if args.len() < 3 {
                eprintln!("Uso: devtool hash <arquivo>");
                std::process::exit(1);
            }

            let file_path = args[2].as_str();

            match commands::hash::calculate_hash(file_path) {
                Ok(hash) => println!("SHA-256: {}", hash),
                Err(error) => {
                    eprintln!("Erro ao calcular hash: {}", error);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            println!("Comando desconhecido: {}", command);
        }
    }
}
