// Variáveis, tipos, tuplas, arrays, funções e expressões.
const LIMITE: u32 = 10;

fn somar(a: u32, b: u32) -> u32 {
    a + b // Sem ponto e vírgula: esta expressão é o retorno.
}

fn main() {
    let ferramenta = "devtool"; // &str inferido
    let mut execucoes: u32 = 1;
    execucoes += 1;
    let porta = "8080";
    let porta: u16 = porta.parse().expect("a porta do exemplo é válida");
    let ativo: bool = true;
    let versao: (u8, u8, u8) = (0, 1, 0);
    let comandos: [&str; 3] = ["help", "hash", "json"];
    let status = if ativo { "ativo" } else { "inativo" };
    println!("{ferramenta} {versao:?}: {status}, porta {porta}");
    println!("Comandos: {comandos:?}");
    println!("Execuções: {}, limite: {LIMITE}", somar(execucoes, 1));
}
