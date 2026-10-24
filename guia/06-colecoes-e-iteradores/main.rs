use std::collections::HashMap;

fn main() {
    let mut arquivos: Vec<String> = vec!["main.rs".into(), "README.md".into()];
    arquivos.push("lib.rs".into());
    let rust: Vec<&str> = arquivos
        .iter()
        .map(String::as_str)
        .filter(|nome| nome.ends_with(".rs"))
        .collect();
    println!("Fontes emprestados: {rust:?}");
    println!("Primeiro: {:?}", arquivos.first()); // Option, sem índice inválido.

    let mut frequencias = HashMap::new();
    for nivel in ["info", "erro", "info", "aviso"] {
        *frequencias.entry(nivel).or_insert(0) += 1;
    }
    // HashMap não garante a ordem de iteração. Ordenamos para exibir.
    let mut resumo: Vec<_> = frequencias.into_iter().collect();
    resumo.sort_by_key(|(nivel, _)| *nivel);
    println!("Logs: {resumo:?}");

    let total_bytes: usize = arquivos.iter().map(|nome| nome.len()).sum();
    println!("Bytes nos nomes: {total_bytes}");
    let maiusculos: Vec<String> = arquivos
        .into_iter()
        .map(|nome| nome.to_uppercase())
        .collect();
    // arquivos foi consumido por into_iter().
    println!("Nomes transformados: {maiusculos:?}");
}
