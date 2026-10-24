fn tamanho(texto: &str) -> usize {
    texto.len() // Quantidade de bytes, não necessariamente de caracteres.
}

fn adicionar_sufixo(texto: &mut String) {
    texto.push_str(" CLI");
}

fn consumir(texto: String) -> String {
    println!("Recebi a posse de: {texto}");
    texto // Transfere a posse para quem chamou.
}

fn main() {
    let original = String::from("DevTool");
    let mut nome = original; // Move: original não pode mais ser usada.
    println!("Bytes por empréstimo: {}", tamanho(&nome));
    adicionar_sufixo(&mut nome);
    let nome = consumir(nome);
    let copia_independente = nome.clone(); // Cópia explícita dos dados.
    let parte: &str = &nome[..7]; // Fronteira UTF-8 válida em "DevTool CLI".
    println!("Nome: {nome}; parte: {parte}; cópia: {copia_independente}");
    let numero: u32 = 42;
    let copia = numero; // u32 implementa Copy, ao contrário de String.
    println!("Os dois continuam válidos: {numero}, {copia}");
}
