use tokio::time::sleep;
use chrono::{DateTime,Local};
use std::fs::File;
use std::io::{self,Write};
use std::time::Duration;
use futures::Stream;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::pin::Pin;
use tonic::{Request, Response, Status};
mod m_broker{
    tonic::include_proto!("message_broker");
}

struct Cliente{
    pub usuario: m_broker::User,
    pub conectado: bool,
    pub stream: Option<Arc<mpsc::Sender<Result<m_broker::Message,Status>>>>,
}
struct Topic {
    pub mensajes: Vec<m_broker::Message>,
    pub suscriptores: HashMap<String, Cliente>,
}

impl Default for Topic{
    fn default() -> Self{
        Topic{
            mensajes: Vec::new(),
            suscriptores: HashMap::new(),
        }
    }
}

struct Log {
    file: File,
}

impl Default for Log {
    fn default() -> Self {
        let file = match File::create("logs.txt") {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Error creating file: {}", err);
                File::open("logs.txt").expect("Failed to open file")
            }
        };

        Self {
            file,
        }
    }
}

//Implementacion del server.
//
#[derive(Default)]
struct BrokerTrait{
    topics: Arc<Mutex<HashMap<String,Topic>>>,
    users: Arc<Mutex<HashMap<String,m_broker::User>>>,
    log: Arc<std::sync::Mutex<Log>>,
}

impl BrokerTrait{
    async fn remove_client_from_topic(&self, topic_name: String, client_id: String) {
        let mut topics = self.topics.lock().await;
        println!("Entra a remove cliente");
        if let Some(topic) = topics.get_mut(&topic_name) {
            match topic.suscriptores.get_mut(&client_id){
                Some(cl) => {
                    cl.conectado = false;
                    cl.stream = None;
                }
                None => {println!("El usuario no existe");}
                
            }
        }
    }

    async fn client_disconnected(&self,client_id: String){
        let mut users = self.users.lock().await;
        if users.contains_key(&client_id){
            users.remove(&client_id);
        }
        let mut topics = self.topics.lock().await;
        for (_,topic) in topics.iter_mut(){
            if topic.suscriptores.contains_key(&client_id){
                topic.suscriptores.remove(&client_id);
            }
        }
        let date = Local::now();
        self.write_on_log(format!("{}: Cliente desconectado\n",date)).await;
    }

   
    async fn write_on_log(&self, data: String) {
        let log = Arc::clone(&self.log);
        let _ = log.lock();
        let result = tokio::task::spawn_blocking(move || {
            let mut log = log.lock().unwrap();
            log.file.write_all(data.as_bytes())?;
            Ok::<(), io::Error>(())
        })
        .await;

        if let Err(err) = result {
            eprintln!("Error writing to log: {}", err);
        }
    }

}
#[tonic::async_trait]
impl m_broker::broker_server::Broker for Arc<BrokerTrait>{


    async fn subscribe(
        &self,
        request: Request<m_broker::SubscriptionRequest>,
    )-> Result<Response<m_broker::SubscriptionResponse>, Status>{
        let request_data = request.into_inner();
        let (request_user, request_topic) = (request_data.user.unwrap(),request_data.topic.clone());
        let mut topics = self.topics.lock().await;
        
        if topics.contains_key(&request_topic){
            let topic = topics.get_mut(&request_topic).unwrap();
            let cliente = Cliente{
                usuario: request_user.clone(),
                conectado: false,
                stream: None,
            };
            match topic.suscriptores.insert(request_user.id.clone().trim().to_string(),cliente){
                Some(_c) => {
                    return Err(Status::already_exists("El usuario se encuentra ya suscrito al topic"));
                }
                None => {
                    let date = Local::now();
                    self.write_on_log(format!("{}: Cliente {} se suscribio al topic {}\n",date,request_user.id.clone().trim().to_string(),request_topic.clone())).await;
                }
            }
        }else{
            println!("DEBUG: Topic not found: {}", request_topic); 
            return Err(Status::invalid_argument("No existe el topic"));
        } 
        let response = m_broker::SubscriptionResponse{
            success: true
        };
        Ok(Response::new(response))
        
    }

    type RegisterStream  = Pin<Box<dyn Stream<Item = Result<m_broker::RegisterResponse, Status>> + Send>>;    
    async fn register(
        self: &Arc<BrokerTrait>,
        request: Request<m_broker::RegisterRequest>,
    ) -> Result<Response<Self::RegisterStream>, Status> {
        let usuario = match request.into_inner().usuario.clone() {
            Some(usuario) => usuario,
            None => return Err(Status::invalid_argument("Datos incompletos")),
        };
        if usuario.id.trim().to_string().is_empty()||usuario.nombre.trim().to_string().is_empty(){
            return Err(Status::invalid_argument("No puede ser vacio"));
        }
        let mut users = self.users.lock().await;

        if users.contains_key(&usuario.id) {
            return Err(Status::already_exists("Usuario existente"));
        }
        let self_arc = Arc::clone(&self);
        let (stream_tx, stream_rx) = mpsc::channel(1);
        users.insert(usuario.id.clone(),usuario.clone());
        tokio::spawn(async move{
             loop{
                    match stream_tx.send(Ok(m_broker::RegisterResponse{})).await{
                        Ok(_) => {}
                        Err(_msg) => {
                            let _ = &self_arc.client_disconnected(usuario.id.clone()).await;
                            break;
                        }
                    }
                    sleep(Duration::from_secs(2)).await;
                }
        });
        let date = Local::now();
        self.write_on_log(format!("{}: Cliente conectado\n",date)).await;
        let output_stream = ReceiverStream::new(stream_rx);
        Ok(Response::new(Box::pin(output_stream) as Self::RegisterStream))
 
    }

    
    type GetMessagesStream  = Pin<Box<dyn Stream<Item = Result<m_broker::Message, Status>> + Send>>;

    async fn get_messages(
        self: &Arc<BrokerTrait>,
        request: Request<m_broker::GetMessageRequest>
    )-> Result<Response<Self::GetMessagesStream>,Status>{
        let request_data = request.into_inner();    
        let (request_id, request_topic) = (request_data.id, request_data.topic);
        let mut topics = self.topics.lock().await;
        let topic = match topics.get_mut(&request_topic){
            Some(topic) => topic,
            None => return Err(Status::not_found("No existe el topic")),
        };
        let self_arc = Arc::clone(&self);
            //Se genera el stream de mensajes. 
            //Primero se van a enviar los mensajes que se encuentran guardados en el
            //servidor. 
            //Despues de esto se van a enviar los mensajes por medio de los que envien los
            //clientes al topic 
            //Se genera el tx, rx y se guarda en un hashmap.
            //Esto funciona de la siguiente forma 
            let (stream_tx, stream_rx) = mpsc::channel(128);
            let (tx,mut rx) = mpsc::channel::<Result<m_broker::Message, Status>>(128);
            let current_client = match topic.suscriptores.get_mut(&request_id.trim().to_string()){
                Some(c) => {
                    c
                }
                None => {
                     let error_message = format!("El usuario {}, no se encuentra suscrito al topic",request_id.clone());
                     return Err(Status::permission_denied(error_message));

                }
            };
            let messages_clone = topic.mensajes.clone();
            current_client.stream = Some(Arc::new(tx));
            current_client.conectado = true;
            tokio::spawn(async move{ 

                for message in messages_clone{
                    match stream_tx.send(Ok(message)).await{
                        Ok(_)=>{}
                        Err(_) => {}
                    }
                }            
                while let Some(msg) = rx.recv().await{
                    match stream_tx.send(msg).await{
                        Ok(_) => {println!("Mensaje enviado correctamente");}
                        Err(_msg) => {
                            println!("Error enviando el mensaje al cliente");
                            let _ = &self_arc.remove_client_from_topic(request_topic.clone(), request_id.clone()).await;
                            break;
                        }
                    }
                }
            });
            let output_stream = ReceiverStream::new(stream_rx);
            Ok(Response::new(Box::pin(output_stream) as Self::GetMessagesStream))
 
            

        
    }

    async fn get_all_topics(
        &self, 
        _request: Request<m_broker::GetAllTopicRequest>,
    ) -> Result<Response<m_broker::GetTopicResponse>,Status>{
        let topics = self.topics.lock().await;
        let mut response_topics = vec![];
        for (topic_name, _topic) in topics.iter(){
            response_topics.push(topic_name.clone());
        }
        let response = m_broker::GetTopicResponse{
            topics: response_topics
        };
        Ok(Response::new(response))
    }

    async fn get_topics(
        &self,
        request: Request<m_broker::GetTopicRequest>,
    ) -> Result<Response<m_broker::GetTopicResponse>,Status>{
        let topics = self.topics.lock().await;
        let request_topic  = request.into_inner();
        let user_id = request_topic.id.trim().to_string();
        let mut response_topics = vec![];
        for (topic_name,topic) in topics.iter(){
            if topic.suscriptores.contains_key(&user_id){
                response_topics.push(topic_name.clone());
            }
        }
        let response = m_broker::GetTopicResponse{
            topics: response_topics
        };
        Ok(Response::new(response))
        
    }

    async fn post_message(
        &self,
        request: Request<m_broker::MessageRequest>,
    ) -> Result<Response<m_broker::MessageResponse>,Status>{
        let mut topics = self.topics.lock().await;
        let request_content = request.into_inner();
        let (request_topic, request_mensaje) = (request_content.topic, request_content.mensaje.unwrap());
        //Ver si si existe el topic y si si esta suscrito el usuario.
        match topics.get_mut(&request_topic){
            Some(topic)=>{
                if topic.suscriptores.contains_key(&request_mensaje.id.trim().to_string()){
                    //Si esta suscrito
                    topic.mensajes.push(request_mensaje.clone());
                    let date = Local::now();
                    self.write_on_log(format!("{}: Mensaje enviado al topic {}\n",date,request_topic.clone())).await;
                    //Se envia el mensaje a todos los clientes que se encuentren viendo el topic.
                    //Utilizando el stream.
                    for (_,sub) in &topic.suscriptores{
                        if sub.conectado{
                            match &sub.stream{
                                Some(sub) => {
                                    match sub.send(Ok(request_mensaje.clone())).await{
                                        Ok(_) => {}
                                        Err(_) => {}
                                    }
                                }
                                None => { println!("No hay");
                                }
                            };
                        }                        
                    }
                    let response = m_broker::MessageResponse{
                        success: true
                    };
                    Ok(Response::new(response))
                }else{
                    return Err(Status::permission_denied("El usuario no esta suscrito al topic. Debe suscribirse para poder enviar mensajes"));
                }
            }
            None => {
                return Err(Status::not_found("El topic no existe"));
            }

        }
    }
}
#[tokio::main()]
async fn main() -> Result<(),Box<dyn std::error::Error>>{
    let addr = "127.0.0.1:50051".parse()?;
    
    println!("Server escuchando en {}", addr);
    let broker = Arc::new(BrokerTrait {
        topics: Arc::new(Mutex::new({
            let mut topics = HashMap::new();
            topics.insert(
                "Pc".to_string(),
                Topic {
                        mensajes: vec![],
                        suscriptores: HashMap::new(),
                    },
            );
            topics.insert(
                "Politics".to_string(),
                Topic {
                    mensajes: vec![],
                    suscriptores: HashMap::new(),
                },
            );
            topics.insert(
                "Memes".to_string(),
                Topic {
                    mensajes: vec![],
                    suscriptores: HashMap::new(),
                },
            );
            topics
        })),
        users: Arc::new(Mutex::new(HashMap::new())),
        log: Arc::new(std::sync::Mutex::new(Log::default())),
    });
    tonic::transport::Server::builder().add_service(m_broker::broker_server::BrokerServer::new(broker)).serve(addr).await?;
    Ok(())
}
