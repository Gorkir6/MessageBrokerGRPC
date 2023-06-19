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
### Cargo(Incluido en rust)
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
-Se tomo la decision de aprender rust para el desarrollo de este proyecto por lo que el uso de https://doc.rust-lang.org/book/ fue esencial para tener una nocion del lenguaje
-Se utilizó el siguiente video para lograr comprender la comunicación con grpc utilizando proto. https://www.youtube.com/watch?v=JkSa-qA2jnY
-Se utilizaron los ejemplos proporsionados por la documentación de Tonic para comprener el streaming. https://github.com/hyperium/tonic/tree/master/examples/src
-Se planea continuar con el proyecto para crear una aplicacion de escritorio con rust al terminar el semestre.
