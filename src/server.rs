#[path = "utils/queue.rs"] mod queue;

use mBroker::User;
use queue::Queue;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc,Mutex};
use tonic::{transport::Server, Request, Response, Status};
mod mBroker{
    tonic::include_proto!("message_broker");
}

struct Topic {
    pub nombre: String,
    pub mensajes: Vec<mBroker::Message>,
    pub suscriptores: HashSet<String>,
}

impl Default for Topic{
    fn default() -> Self{
        Topic{
            nombre: String::new(),
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
impl mBroker::broker_server::Broker for BrokerTrait{
        
    async fn suscribe(
        &self,
        request: Request<mBroker::SuscriptionRequest>,
    )-> Result<Response<mBroker::SuscriptionResponse>, Status>{
        let request_data = request.into_inner();
        let (request_id, request_topic) = (request_data.id.clone(),request_data.topic.clone());
        let mut topics = self.topics.lock().unwrap();
        
        if topics.contains_key(&request_topic){
            let topic = topics.get_mut(&request_topic).unwrap();
            let insertado = topic.suscriptores.insert(request_id.clone());
            if insertado == false{
                return Err(Status::already_exists("El usuario se encuentra ya suscrito al topic"));
            }
        }
        
        let response = mBroker::SuscriptionResponse{
            success: true
        };
        Ok(Response::new(response))
        
    }

    
    async fn register(
        &self,
        request: Request<mBroker::RegisterRequest>,
    ) -> Result<Response<mBroker::RegisterResponse>, Status> {
        let usuario = match request.into_inner().usuario.clone() {
            Some(usuario) => usuario,
            None => return Err(Status::invalid_argument("User information is missing")),
        };

        let mut users = self.users.lock().unwrap();

        if users.contains_key(&usuario.id) {
            return Err(Status::already_exists("User already exists"));
        }

        users.insert(usuario.id.clone(),usuario);

        let response = mBroker::RegisterResponse {
            success: true,
        };

        Ok(Response::new(response))
    }


    async fn get_messages(
        &self,
        request: Request<mBroker::GetMessageRequest>
    )-> Result<Response<mBroker::GetMessageResponse>,Status>{
        let request_data = request.into_inner();    
        let (request_id, request_topic) = (request_data.id, request_data.topic);
        let mut response_messages = Vec::new();
        let topics = self.topics.lock().unwrap();
        
        match topics.get_mut(&request_topic) {
            Some(topic) => {
                if topic.suscriptores.contains(&request_id) {
                    let response_messages = topic.mensajes.clone();
                    // Rest of your code here

                } else {
                    let error_message = format!("El usuario {}, no se encuentra suscrito al topic",request_id.clone());
                    return Err(Status::permission_denied(error_message));
                }
            }
            None => {
                return Err(Status::not_found("No existe el topic"));
            }
        }

        let response = mBroker::GetMessageResponse{
            messages: response_messages
        };
        Ok(Response::new(response))
    }

    async fn get_all_topics(
        &self, 
        request: Request<mBroker::GetAllTopicRequest>,
    ) -> Result<Response<mBroker::GetTopicResponse>,Status>{
        let topics = self.topics.lock().unwrap();
        let mut response_topics = vec![];
        for topic in topics.iter(){
            response_topics.push(topic.nombre.clone());
        }
        let response = mBroker::GetTopicResponse{
            topics: response_topics
        };
        Ok(Response::new(response))
    }

    async fn get_topics(
        &self,
        request: Request<mBroker::GetTopicRequest>,
    ) -> Result<Response<mBroker::GetTopicResponse>,Status>{
        let topics = self.topics.lock().unwrap();
        let request_topic  = request.into_inner();
        let user_id = request_topic.id;
        let mut response_topics = vec![];
        for topic in topics.iter(){
            if topic.suscriptores.contains(&user_id){
                response_topics.push(topic.nombre.clone());
            }
        }
        let response = mBroker::GetTopicResponse{
            topics: response_topics
        };
        Ok(Response::new(response))
        
    }

    async fn post_message(
        &self,
        request: Request<mBroker::MessageRequest>,
    ) -> Result<Response<mBroker::MessageResponse>,Status>{
        let request_content = request.into_inner();
        let (request_topic, request_mensaje) = (request_content.topic, request_content.mensaje.unwrap());
        let (msg_contenido, msg_id) = (request_mensaje.contenido, request_mensaje.id);
        let response = mBroker::MessageResponse{
            success: true
        };
        Ok(Response::new(response))
    }
}

fn main() {
    println!("Hello, world!");
}
