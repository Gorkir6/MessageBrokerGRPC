use std::io::{self,Write};
use tokio_stream::StreamExt;
use tonic::transport::Channel;
mod m_broker{
    tonic::include_proto!("message_broker");
}

struct Client{
    pub usuario: m_broker::User,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Conectarse al servidor de gRPC
    let channel = Channel::from_static("http://127.0.0.1:50051").connect().await?;
    let mut client =m_broker::broker_client::BrokerClient::new(channel);
    let mut cliente_name = String::new();
    let mut cliente_id = String::new();
    loop{
        cliente_id = "".to_string();
        cliente_name = "".to_string();
        println!("Ingrese su nombre");
        io::stdout().flush()?;
        io::stdin().read_line(&mut cliente_name)?; 
        println!("Ingese su id: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut cliente_id)?; 
        let request = tonic::Request::new(m_broker::RegisterRequest {
            usuario: Some(m_broker::User {
                id: cliente_id.trim_end().to_string(),
                nombre: cliente_name.trim_end().to_string(),
            }),
        });        
        match client.register(request).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                tokio::spawn(async move{
                    while let Some(_item) = stream.next().await{
                        
                    }
                });
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
    let mut selected_topic = String::new();
    println!("--- Topics disponibles ---");
    for (index,topic) in response.topics.iter().enumerate() {
        println!("{}: {}",index+1, topic);
    }
    loop{
        println!("Seleccione el topic:");
        let mut topic_name = String::new();
        io::stdout().flush()?;
        io::stdin().read_line(&mut topic_name)?;
        match topic_name.trim().parse::<usize>(){
            Ok(i)=>{
                let topic_i = i-1;
                if topic_i < response.topics.len(){
                    selected_topic = response.topics[topic_i].clone();
                    println!("Topic elegido: {}",selected_topic.clone());
                    break;
                }else{
                    println!("Seleccione una opcion valida");
                }
            }
            Err(_) => {println!("Error de tipo");}
        }
    }    
    let request = tonic::Request::new(m_broker::SubscriptionRequest {
        topic: selected_topic.trim().to_string(),
        user: Some(usuario),
    });
    match client.subscribe(request).await{
        Ok(_response) => {
            println!("Suscripción exitosa");
        }
        Err(error) => {
            println!("Error: {}",error.message());
        }
    }
    Ok(())
}

async fn view(mut client: m_broker::broker_client::BrokerClient<Channel>, usuario: m_broker::User) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(m_broker::GetTopicRequest {id: usuario.id.clone(),});
    let response = client.get_topics(request).await?.into_inner();
    if response.topics.is_empty(){
        println!("No se encuentra suscrito a ningun topic");
        return Ok(());
    }
    let mut selected_topic = String::new();//recomendable usar Option.
    println!("--- Topics suscritos ---");
    for (index,topic) in response.topics.iter().enumerate() {
        println!("{}: {}",index+1, topic);
    }
    loop{
        println!("Seleccione el topic que desea ver:");
        let mut topic_name = String::new();
        io::stdout().flush()?;
        io::stdin().read_line(&mut topic_name)?;
        match topic_name.trim().parse::<usize>(){
            Ok(i)=>{
                let topic_i = i-1;
                if topic_i < response.topics.len(){
                    selected_topic = response.topics[topic_i].clone();
                    println!("Topic elegido: {}",selected_topic.clone());
                    break;
                }else{
                    println!("Seleccione una opcion valida");
                }
            }
            Err(_) => {println!("Error de tipo");}
        }
    }
    let request = tonic::Request::new(m_broker::GetMessageRequest {
        topic: selected_topic.trim().to_string(),
        id: usuario.id.clone(),
    });
    match client.get_messages(request).await{
        Ok(response)=>{
            let mut stream = response.into_inner();
            println!("---Mensajes en el topic ---");
            println!("Presion ctrl+c para salir");
            loop{
                tokio::select! {
                    item = stream.next() => {
                        if let Some(item) = item {
                            println!("{}: {}", item.as_ref().unwrap().id.clone().trim().to_string(), item.as_ref().unwrap().contenido.clone());
                        }
                    }
                    _ = tokio::signal::ctrl_c() => break,
                }
            }
        }
        Err(error) => {
            println!("Erro: {}",error.message());
        }
    }
  
    Ok(())
}

async fn post(mut client: m_broker::broker_client::BrokerClient<Channel>, usuario: m_broker::User) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(m_broker::GetTopicRequest {id: usuario.id.clone()});
    let response = client.get_topics(request).await?.into_inner();
    if response.topics.is_empty(){
        println!("No se encuentra suscrito a ningun topic");
        return Ok(());
    }
    let mut selected_topic = String::new();
    println!("--- Topics suscritos ---");
    for (index,topic) in response.topics.iter().enumerate() {
        println!("{}: {}",index+1, topic);
    }
    loop{
        println!("Seleccione el topic:");
        let mut topic_name = String::new();
        io::stdout().flush()?;
        io::stdin().read_line(&mut topic_name)?;
        match topic_name.trim().parse::<usize>(){
            Ok(i)=>{
                let topic_i = i-1;
                if topic_i < response.topics.len(){
                    selected_topic = response.topics[topic_i].clone();
                    println!("Topic elegido: {}",selected_topic.clone());
                    break;
                }else{
                    println!("Seleccione una opcion valida");
                }
            }
            Err(_) => {println!("Error de tipo");}
        }
    }
    let mut message = String::new();
    loop{
        println!("Ingrese el mensaje:");
        io::stdout().flush()?;
        io::stdin().read_line(&mut message)?;
        if !message.trim().to_string().is_empty(){
            break;
        }
    }
    let request = tonic::Request::new(m_broker::MessageRequest {
        topic: selected_topic.trim().to_string(),
        mensaje: Some(m_broker::Message{id: usuario.id.clone(),contenido:message.clone().trim().to_string()}),
    });
    match client.post_message(request).await{
        Ok(_response) => {
            println!("Mensaje publicado!");
        }
        Err(error) =>{
            println!("Error: {}", error.message());
        }
    }
    Ok(())
}
