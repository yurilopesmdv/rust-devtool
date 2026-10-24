fn main() {
    let mut name = String::from("DevTool");

    let reference = &mut name;

    reference.push_str(" CLI");

    println!("{}", reference);
}