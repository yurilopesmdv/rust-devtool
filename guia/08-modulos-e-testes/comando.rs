#[derive(Debug, PartialEq)]
pub enum Comando<'a> {
    Ajuda,
    Hash(&'a str),
}

pub fn interpretar<'a>(args: &[&'a str]) -> Result<Comando<'a>, &'static str> {
    match args {
        [] | ["help"] => Ok(Comando::Ajuda),
        ["hash", caminho] if !caminho.is_empty() => Ok(Comando::Hash(caminho)),
        ["hash"] => Err("informe um arquivo"),
        _ => Err("comando ou argumentos inválidos"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ajuda_na_ausencia_de_comando() {
        assert_eq!(interpretar(&[]), Ok(Comando::Ajuda));
        assert_eq!(interpretar(&["help"]), Ok(Comando::Ajuda));
    }

    #[test]
    fn hash_exige_um_unico_caminho_nao_vazio() {
        assert_eq!(interpretar(&["hash", "a.txt"]), Ok(Comando::Hash("a.txt")));
        assert!(interpretar(&["hash"]).is_err());
        assert!(interpretar(&["hash", ""]).is_err());
        assert!(interpretar(&["hash", "a.txt", "extra"]).is_err());
        assert!(interpretar(&["inexistente"]).is_err());
    }
}
