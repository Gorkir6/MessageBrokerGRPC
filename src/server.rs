use utils::queue;



struct Mensaje{
    id: String,
    contenido: String,
}

#[derive(Debug)]
struct Topic {
    nombre: String,
    mensajes: utils::Queue<Mensaje>,
}



fn main() {
    println!("Hello, world!");
}
