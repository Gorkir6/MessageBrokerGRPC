use m_broker::User;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc,Mutex};
use tonic::{Request, Response, Status};
mod m_broker{
    tonic::include_proto!("message_broker");
}

struct Topic {
    pub mensajes: Vec<m_broker::Message>,
    pub suscriptores: HashSet<String>,
}

impl Default for Topic{
    fn default() -> Self{
        Topic{
            mensajes: Vec::new(),
            suscriptores: HashSet::new(),
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

#[tonic::async_trait]
impl m_broker::broker_server::Broker for BrokerTrait{
        
    async fn suscribe(
        &self,
        request: Request<m_broker::SuscriptionRequest>,
    )-> Result<Response<m_broker::SuscriptionResponse>, Status>{
        let request_data = request.into_inner();
        let (request_id, request_topic) = (request_data.id.clone(),request_data.topic.clone());
        let mut topics = self.topics.lock().unwrap();
        
        if topics.contains_key(&request_topic){
            let topic = topics.get_mut(&request_topic).unwrap();
            let insertado = topic.suscriptores.insert(request_id.clone());
            if !insertado{
                return Err(Status::already_exists("El usuario se encuentra ya suscrito al topic"));
            }
        }else{
            println!("DEBUG: Topic not found: {}", request_topic); 
            return Err(Status::invalid_argument("No existe el topic"));
        }
        println!("El usuario se suscribio al topic!");
        println!("Los usuarios suscritos son: ");
        for suscriber in topics.get(&request_topic).unwrap().suscriptores.iter(){
            println!("{}",suscriber);
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

        let mut users = self.users.lock().unwrap();

        if users.contains_key(&usuario.id) {
            return Err(Status::already_exists("User already exists"));
        }

        users.insert(usuario.id.clone(),usuario);

        let response = m_broker::RegisterResponse {
            success: true,
        };

        Ok(Response::new(response))
    }


    async fn get_messages(
        &self,
        request: Request<m_broker::GetMessageRequest>
    )-> Result<Response<m_broker::GetMessageResponse>,Status>{
        let request_data = request.into_inner();    
        let (request_id, request_topic) = (request_data.id, request_data.topic);
        let mut topics = self.topics.lock().unwrap();
        
        match topics.get_mut(&request_topic) {
            Some(topic) => {
                if topic.suscriptores.contains(&request_id) {
                    let response = m_broker::GetMessageResponse{
                        messages: topic.mensajes.clone()
                    };
                    Ok(Response::new(response))                
                } else {
                    let error_message = format!("El usuario {}, no se encuentra suscrito al topic",request_id.clone());
                    return Err(Status::permission_denied(error_message));
                }
            }
            None => {
                return Err(Status::not_found("No existe el topic"));
            }
        }

        
    }

    async fn get_all_topics(
        &self, 
        _request: Request<m_broker::GetAllTopicRequest>,
    ) -> Result<Response<m_broker::GetTopicResponse>,Status>{
        let topics = self.topics.lock().unwrap();
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
        let topics = self.topics.lock().unwrap();
        let request_topic  = request.into_inner();
        let user_id = request_topic.id;
        let mut response_topics = vec![];
        for (topic_name,topic) in topics.iter(){
            if topic.suscriptores.contains(&user_id){
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
        let mut topics = self.topics.lock().unwrap();
        let request_content = request.into_inner();
        let (request_topic, request_mensaje) = (request_content.topic, request_content.mensaje.unwrap());
        //Ver si si existe el topic y si si esta suscrito el usuario.
        match topics.get_mut(&request_topic){
            Some(topic)=>{
                if topic.suscriptores.contains(&request_mensaje.id){
                    //Si esta suscrito
                    topic.mensajes.push(request_mensaje);
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
                    suscriptores: HashSet::new(),
                },
            );
            topics.insert(
                "Politics".to_string(),
                Topic {
                    mensajes: vec![],
                    suscriptores: HashSet::new(),
                },
            );
            topics.insert(
                "Memes".to_string(),
                Topic {
                    mensajes: vec![],
                    suscriptores: HashSet::new(),
                },
            );
            topics
        })),
        users: Arc::new(Mutex::new(HashMap::new())),
    };
    tonic::transport::Server::builder().add_service(m_broker::broker_server::BrokerServer::new(broker)).serve(addr).await?;
    Ok(())
}
