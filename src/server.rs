#[path = "utils/queue.rs"] mod queue;

use queue::Queue;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc,Mutex};
use tonic::{transport::Server, Request, Response, Status};
use message_broker::{
    message_broker_server::{Broker, BrokerServer},
    SuscriptionRequest, SuscriptionResponse,MessageRequest, GetMessageRequest, GetMessageResponse, GetTopicResponse, GetAllTopicRequest,
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
        let topics: self.topics.lock().unwrap();
        let response_topics = vec![];
        while(!topics.is_empty()){
           if Some(topics.dequeue()){

            } 
        }
    }
}

fn main() {
    println!("Hello, world!");
}
