use crate::{prelude::*, routes::operation::OperationResponse};
use actix_web::rt::Runtime;

use super::operation::OperationRequest;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RegisterResponse {
  pub accepted: bool,
  pub term: i32,
  pub peers: Vec<String>,
  pub log: Vec<(i32, Operation)>,
  pub queue: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RegisterRequest {
  pub sender: String,
  pub term: i32
}

pub async fn register(context: web::Data<Arc<Mutex<NodeInfo>>>, new_node: web::Json<RegisterRequest>) -> impl Responder {
    let mut ctx = context.lock().unwrap();

    let peers = ctx.peers.clone();

    println!("====================");
    println!("POST : REGISTER\n");
    println!("Sender : {}", new_node.sender.clone());
    println!("Term : {}\n", new_node.term.clone());

    let mut res = false;

    let add_node_operation = &Operation {
        operation_type: OperationType::AddNode,
        content: Some(String::from(new_node.sender.clone())),
        is_committed: None
    };
    if ctx.address == ctx.leader {
        let add_node_request = &OperationRequest{
            operations: vec![add_node_operation.clone()],
            sender: ctx.address.clone(),
            previous_log_entry: None,
            term: ctx.term.clone(),
        };

        let responses = post_many(ctx.peers.clone(), OPERATION_ROUTE, &serde_json::to_string(&add_node_request).unwrap()).await;
        
        let mut count = 0;
        for response in responses {
          match response {
            Ok(sk) => {
              let operation_response = sk.json::<OperationResponse>().await.unwrap();
              if operation_response.accepted {
                count += 1;
              }
            },
            Err(e) => {
              print!("{:?}", e);
            }
          }
        }

        ctx.peers.push(new_node.sender.clone());
        res = true;
        // TODO: check log (consistent/not) if inconsistent do operation until consistent then send response
    } else {
        res = false;
    }
    
    println!("Register node {} success.\n", new_node.sender.clone());
    println!("Peers : {:?}\n", ctx.peers.clone());

    HttpResponse::Ok().body(serde_json::to_string(&RegisterResponse {
      accepted: res,
      term: ctx.term.clone(),
      peers: peers.clone(),
      log: ctx.log.clone(),
      queue: ctx.queue.clone()
    }).unwrap())
}