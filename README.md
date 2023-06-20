# Proyecto 2 Sistemas Operativos 
## Gorki Romero, John Rojas, Camilo Gonzalez

## Este proyecto no se ha probado en windows. Se recomienda usar en linux.

# Como correrlo 
## Requerimientos:
### Rust
### Instalacion
```ssh 
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
### Cargo
Suele venir en la instalacion de rust, si se tiene un error 
```ssh
    sudo apt install cargo 
```
### Para el proto
```ssh
    sudo apt install -y protobuf-compiler libprotobuf-dev
```

## Ejecutar cliente y servidor
```ssh
    cargo run --bin broker-server
    cargo run --bin broker-cliente
```

# Referencias utilizadas para el desarrollo del proyecto. 
- Se tomó la decisión de aprender rust para el desarrollo de este proyecto por lo que el uso del libro de la documentación de rust fue esencial para tener una noción del lenguaje. https://doc.rust-lang.org/book/
- Se utilizó el siguiente video para lograr comprender la comunicación con grpc utilizando proto. https://www.youtube.com/watch?v=JkSa-qA2jnY
- Se utilizaron los ejemplos proporsionados por la documentación de Tonic para comprender el streaming. https://github.com/hyperium/tonic/tree/master/examples/src
- Se planea continuar con el proyecto para crear una aplicacion de escritorio con rust al terminar el semestre.
- Considerar el uso de crossbeam-channel(revisar si es funcional con tonic). https://crates.io/crates/crossbeam-channel
