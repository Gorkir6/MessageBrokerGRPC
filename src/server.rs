use m_broker::User;
use futures::Stream;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
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
//Implementacion del server.
//
#[derive(Default)]
struct BrokerTrait{
    topics: Arc<Mutex<HashMap<String,Topic>>>,
    users: Arc<Mutex<HashMap<String,User>>>
}

impl BrokerTrait{
async fn remove_client(&self, topic_name: String, client_id: String) {
        let mut topics = self.topics.lock().await;
        if let Some(topic) = topics.get_mut(&topic_name) {
            topic.suscriptores.remove(&client_id);
        }
    }
}
#[tonic::async_trait]
impl m_broker::broker_server::Broker for BrokerTrait{

    async fn suscribe(
        &self,
        request: Request<m_broker::SuscriptionRequest>,
    )-> Result<Response<m_broker::SuscriptionResponse>, Status>{
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
            match topic.suscriptores.insert(request_user.id.clone(),cliente){
                Some(_c) => {
                    return Err(Status::already_exists("El usuario se encuentra ya suscrito al topic"));
                }
                None => {
                    
                }
            }
        }else{
            println!("DEBUG: Topic not found: {}", request_topic); 
            return Err(Status::invalid_argument("No existe el topic"));
        } 
        let response = m_broker::SuscriptionResponse{
            success: true
        };
        Ok(Response::new(response))
        
    }

    
    async fn register(
        &self,
        request: Request<m_broker::RegisterRequest>,
    ) -> Result<Response<m_broker::RegisterResponse>, Status> {
        let usuario = match request.into_inner().usuario.clone() {
            Some(usuario) => usuario,
            None => return Err(Status::invalid_argument("User information is missing")),
        };

        let mut users = self.users.lock().await;

        if users.contains_key(&usuario.id) {
            return Err(Status::already_exists("User already exists"));
        }

        users.insert(usuario.id.clone(),usuario);

        let response = m_broker::RegisterResponse {
            success: true,
        };

        Ok(Response::new(response))
    }

    
    type GetMessagesStream  = Pin<Box<dyn Stream<Item = Result<m_broker::Message, Status>> + Send>>;

    async fn get_messages(
        &self,
        request: Request<m_broker::GetMessageRequest>
    )-> Result<Response<Self::GetMessagesStream>,Status>{
        let request_data = request.into_inner();    
        let (request_id, request_topic) = (request_data.id, request_data.topic);
        let mut topics = self.topics.lock().await;
        let topic = match topics.get_mut(&request_topic){
            Some(topic) => topic,
            None => return Err(Status::not_found("No existe el topic")),
        };
        
            //Se genera el stream de mensajes. 
            //Primero se van a enviar los mensajes que se encuentran guardados en el
            //servidor. 
            //Despues de esto se van a enviar los mensajes por medio de los que envien los
            //clientes al topic 
            //Se genera el tx, rx y se guarda en un hashmap.
            //Esto funciona de la siguiente forma 
            let (stream_tx, stream_rx) = mpsc::channel(128);
            let (tx,mut rx) = mpsc::channel::<Result<m_broker::Message, Status>>(128);
            let current_client = match topic.suscriptores.get_mut(&request_id){
                Some(c) => {
                    c
                }
                None => {
                     let error_message = format!("El usuario {}, no se encuentra suscrito al topic",request_id.clone());
                     return Err(Status::permission_denied(error_message));

                }
            };
            current_client.stream = Some(Arc::new(tx));
            tokio::spawn(async move{ 
                while let Some(msg) = rx.recv().await{
                    match stream_tx.send(msg).await{
                        Ok(_) => {println!("Mensaje enviado correctamente");}
                        Err(_msg) => {
                            println!("Error enviando el mensaje al cliente");
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
        let user_id = request_topic.id;
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
                if topic.suscriptores.contains_key(&request_mensaje.id){
                    //Si esta suscrito
                    topic.mensajes.push(request_mensaje.clone());
                    //Se envia el mensaje a todos los clientes que se encuentren viendo el topic.
                    //Utilizando el stream.
                    for (_,sub) in &topic.suscriptores{
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
    let broker = BrokerTrait {
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
    };
    tonic::transport::Server::builder().add_service(m_broker::broker_server::BrokerServer::new(broker)).serve(addr).await?;
    Ok(())
}
