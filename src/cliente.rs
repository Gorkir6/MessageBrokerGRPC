use std::io::{self,Write};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tonic::{transport::Channel,Request};
use tokio::signal;
mod m_broker{
    tonic::include_proto!("message_broker");
}

struct Client{
    pub usuario: m_broker::User,
    pub topics: HashMap<String,Vec<m_broker::Message>>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Conectarse al servidor de gRPC
    let channel = Channel::from_static("http://127.0.0.1:50051").connect().await?;
    let mut client =m_broker::broker_client::BrokerClient::new(channel);
    let mut cliente_name = String::new();
    let mut cliente_id = String::new();
    loop{
        println!("Ingrese su nombre");
        io::stdout().flush()?;
        io::stdin().read_line(&mut cliente_name)?;
        println!("Ingese su id: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut cliente_id)?;
        let request = tonic::Request::new(m_broker::RegisterRequest{usuario: Some(m_broker::User{id:cliente_id.clone(), nombre: cliente_name.clone()}),});
        match client.register(request).await {
            Ok(response) => {
                println!("User registration successful: {:?}", response);
                break;
            }
            Err(error) => {
                println!("Error: {}",error.message());
            }
        }
    }
    //Registro exitoso.
    let user = m_broker::User{id:cliente_id.clone(),nombre:cliente_name.clone(),};
    let cliente_actual = Client{
        usuario: user,
        topics: HashMap::new(),
    };
    // Bucle principal del cliente
    loop {
        println!("--- Menu ---");
        println!("1) Suscribirse");
        println!("2) Ver");
        println!("3) Postear");
        println!("4) Salir");

        let mut input = String::new();
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u32>() {
            Ok(choice) => {
                match choice {
                    1 => subscribe(client.clone(), cliente_actual.usuario.clone()).await?,
                    2 => view(client.clone(), cliente_actual.usuario.clone()).await?,
                    3 => post(client.clone(),cliente_actual.usuario.clone()).await?,
                    4 => break,
                    _ => println!("Opción inválida"),
                }
            }
            Err(_) => println!("Opción inválida"),
        }
    }

    Ok(())
}

async fn subscribe(mut client:m_broker::broker_client::BrokerClient<Channel>, usuario: m_broker::User) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(m_broker::GetAllTopicRequest{id: usuario.id.clone(),});
    let response = client.get_all_topics(request).await?.into_inner();
    println!("--- Topics disponibles ---");
    for topic in response.topics {
        println!("{}", topic);
    }

    println!("Ingrese el nombre del topic al que desea suscribirse:");
    let mut topic_name = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut topic_name)?;
    println!("Topic elegido:{} ", topic_name.clone());
    let request = tonic::Request::new(m_broker::SuscriptionRequest {
        topic: topic_name.trim().to_string(),
        user: Some(usuario),
    });
    let _response = client.suscribe(request).await?.into_inner();

    println!("Suscripción exitosa");

    Ok(())
}

async fn view(mut client: m_broker::broker_client::BrokerClient<Channel>, usuario: m_broker::User) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(m_broker::GetTopicRequest {id: usuario.id.clone(),});
    let response = client.get_topics(request).await?.into_inner();

    println!("--- Topics suscritos ---");
    for topic in response.topics {
        println!("{}", topic);
    }

    println!("Ingrese el nombre del topic que desea ver:");
    let mut topic_name = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut topic_name)?;

    let request = tonic::Request::new(m_broker::GetMessageRequest {
        topic: topic_name.trim().to_string(),
        id: usuario.id.clone(),
    });
    let mut stream = client.get_messages(request).await?.into_inner();
    
    println!("--- Mensajes en el topic ---");
    while let Some(item) = stream.next().await{
        println!("{}: {}", item.as_ref().unwrap().id.clone(), item.as_ref().unwrap().contenido.clone());
    }

    Ok(())
}

async fn post(mut client: m_broker::broker_client::BrokerClient<Channel>, usuario: m_broker::User) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(m_broker::GetTopicRequest {id: usuario.id.clone()});
    let response = client.get_topics(request).await?.into_inner();

    println!("--- Topics suscritos ---");
    for topic in response.topics {
        println!("{}", topic);
    }

    println!("Ingrese el nombre del topic en el que desea publicar:");
    let mut topic_name = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut topic_name)?;

    println!("Ingrese el mensaje:");
    let mut message = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut message)?;

    let request = tonic::Request::new(m_broker::MessageRequest {
        topic: topic_name.trim().to_string(),
        mensaje: Some(m_broker::Message{id: usuario.id.clone(),contenido:message.clone()}),
    });
    let _response = client.post_message(request).await?.into_inner();

    println!("Mensaje publicado");

    Ok(())
}
