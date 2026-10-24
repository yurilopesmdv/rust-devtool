fn descrever(comando: &str) -> &str {
    match comando {
        "hash" => "calcula um resumo de bytes",
        "json" => "formata dados JSON",
        _ => "comando desconhecido",
    }
}

fn main() {
    for (indice, comando) in ["hash", "json", "outro"].iter().enumerate() {
        println!("{}: {comando} — {}", indice + 1, descrever(comando));
    }
    let mut tentativas = 0;
    let resultado = loop {
        tentativas += 1;
        if tentativas == 3 {
            break "pronto"; // loop também pode produzir um valor.
        }
    };
    while tentativas > 0 {
        tentativas -= 1;
    }
    for numero in 1..=3 {
        if numero == 2 {
            continue;
        }
        println!("Passo {numero}");
    }
    println!("{resultado}; contador final: {tentativas}");
}
