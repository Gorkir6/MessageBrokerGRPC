#[derive(Default)]
#[path = "utils/queue.rs"] mod queue;

use queue::Queue;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc,Mutex};
use tonic::{transport::Server, Request, Response, Status};
use message_broker::{SuscriptionRequest, SuscriptionResponse,MessageRequest, GetMessageRequest, GetMessageResponse, GetTopicResponse, GetAllTopicRequest};
use message_broker_server::{Broker, BrokerServer};

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
struct Broker{
    topics: Arc<Mutex<Queue<Topic>>>,
}

#[tonic::async_trait]
impl MessageBroker for Broker{

    async fn get_all_topic(
        &self, 
        request: Request<GetAllTopicRequest>,
    ) -> Result<Response<GetTopicResponse>,Status>{
        let topics: self.topics.lock().unwrap().copy();
        let mut response_topics = vec![];
        while(!topics.is_empty()){
           let topic = topics.dequeue().unwrap();
           response_topics.push(topics.nombre);
        }
        let response = GetTopicRespose{
            topics
        };
        Ok(Response::new(response))
    }
}

fn main() {
    println!("Hello, world!");
}
