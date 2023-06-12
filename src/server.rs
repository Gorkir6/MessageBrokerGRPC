#[path = "utils/queue.rs"] mod queue;

use queue::Queue;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc,Mutex};
use tonic::{transport::Server, Request, Response, Status};
mod mBroker{
    tonic::include_proto!("message_broker");
}
struct Mensaje{
    id: String,
    contenido: String,
}

struct Topic {
    nombre: String,
    mensajes: Queue<Mensaje>,
    suscriptores: HashSet<String>,
}

impl Default for Topic{
    fn default() -> Self{
        Topic{
            nombre: String::new(),
            mensajes: Queue::new(),
            suscriptores: HashSet::new(),
        }
    }
}
//Implementacion del server.
//
#[derive(Default)]
struct BrokerTrait{
    topics: Arc<Mutex<Queue<Topic>>>,
}

#[tonic::async_trait]
impl mBroker::broker_server::Broker for BrokerTrait{
 
    async fn get_all_topics(
        &self, 
        request: Request<mBroker::GetAllTopicRequest>,
    ) -> Result<Response<mBroker::GetTopicResponse>,Status>{
        let topics = self.topics.lock().unwrap();
        let mut response_topics = vec![];
        while !topics.is_empty() {
           let topic = topics.dequeue().unwrap();
           response_topics.push(topic.nombre);
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
