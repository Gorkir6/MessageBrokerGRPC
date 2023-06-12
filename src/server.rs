#[path = "utils/queue.rs"] mod queue;

use queue::Queue;


struct Mensaje{
    id: String,
    contenido: String,
}

struct Topic {
    nombre: String,
    mensajes: Queue<Mensaje>,
}
//Implementacion del server.
//


fn main() {
    println!("Hello, world!");
}
