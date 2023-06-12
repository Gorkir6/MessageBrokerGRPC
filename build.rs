//En este archivo se le va a dar a tonic el como hacer build, esto para generar el paquete del
//proto. 
//

fn main() -> Result<(), Box<dyn std::error::Error>>{
    tonic_build::compile_protos("proto/MessageBroker.proto")?;
    Ok(())
}
