use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum ErroLimite {
    NumeroInvalido,
    ForaDoIntervalo,
}

impl fmt::Display for ErroLimite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NumeroInvalido => write!(f, "informe um inteiro positivo"),
            Self::ForaDoIntervalo => write!(f, "o limite deve estar entre 1 e 100"),
        }
    }
}

impl Error for ErroLimite {}

fn ler_limite(entrada: Option<&str>) -> Result<u32, ErroLimite> {
    let numero = entrada
        .unwrap_or("10")
        .parse::<u32>()
        .map_err(|_| ErroLimite::NumeroInvalido)?;
    if !(1..=100).contains(&numero) {
        return Err(ErroLimite::ForaDoIntervalo);
    }
    Ok(numero)
}

fn main() {
    for entrada in [None, Some("25"), Some("zero"), Some("101")] {
        match ler_limite(entrada) {
            Ok(limite) => println!("Limite: {limite}"),
            Err(erro) => eprintln!("Erro: {erro}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usa_padrao_e_aceita_as_fronteiras() {
        assert_eq!(ler_limite(None), Ok(10));
        assert_eq!(ler_limite(Some("1")), Ok(1));
        assert_eq!(ler_limite(Some("100")), Ok(100));
    }

    #[test]
    fn distingue_texto_invalido_de_numero_fora_do_intervalo() {
        assert_eq!(ler_limite(Some("abc")), Err(ErroLimite::NumeroInvalido));
        assert_eq!(ler_limite(Some("-1")), Err(ErroLimite::NumeroInvalido));
        for entrada in ["0", "101"] {
            assert_eq!(ler_limite(Some(entrada)), Err(ErroLimite::ForaDoIntervalo));
        }
    }
}
