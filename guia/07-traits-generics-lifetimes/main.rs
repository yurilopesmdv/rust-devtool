trait Resumo {
    fn resumir(&self) -> String;
}

struct Tarefa {
    nome: String,
}

impl Resumo for Tarefa {
    fn resumir(&self) -> String {
        format!("Tarefa: {}", self.nome)
    }
}

// T é escolhido em tempo de compilação: despacho estático.
fn imprimir<T: Resumo>(item: &T) {
    println!("{}", item.resumir());
}

// Despacho dinâmico: implementações distintas podem usar a mesma interface.
fn imprimir_dinamico(item: &dyn Resumo) {
    println!("{}", item.resumir());
}

// A saída referencia um trecho da própria entrada; não aloca outra String.
fn primeiro_campo<'a>(linha: &'a str, separador: &str) -> &'a str {
    linha.split(separador).next().unwrap_or(linha)
}

fn main() {
    let tarefa = Tarefa {
        nome: String::from("validar configuração"),
    };
    imprimir(&tarefa);
    imprimir_dinamico(&tarefa);
    let linha = String::from("erro: arquivo ausente");
    let campo = primeiro_campo(&linha, ":");
    println!("Primeiro campo: {campo}");
}
