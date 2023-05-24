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
    let c = context.lock();
    let mut ctx = c.unwrap();

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
      
      let n = ctx.log.len().clone();
      
      let mut last_log: Option<(i32, Operation)>;
      if ctx.log.len() > 0 {
        last_log = Some(ctx.log[ctx.log.len()-1].clone());
      } else {
        last_log = None;
      }
        
        let add_node_request = &OperationRequest{
            operations: vec![(ctx.term.clone(), add_node_operation.clone())],
            sender: ctx.address.clone(),
            previous_log_entry: last_log.clone(),
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
              } else if operation_response.note == "Error : Different last log" {
                // check log (consistent/not) if inconsistent do operation until consistent then send response
                let mut flag = false;
                let mut log = ctx.log.clone();
                let n = log.len() as i32;
                let mut idx = n-1;
                let mut operation_request: OperationRequest = OperationRequest{
                  operations: vec![(ctx.term.clone(), add_node_operation.clone())],
                  sender: ctx.address.clone(),
                  previous_log_entry: last_log.clone(),
                  term: ctx.term.clone(),
                };
                if idx < 0 {
                  flag = true;
                } else {
                  operation_request.operations.insert(0, log[idx as usize].clone());
                  idx -= 1;
                  if idx == -1 {
                    last_log = None;
                  } else {
                    last_log = Some(log[idx as usize].clone());
                  }
                }  
                while !flag {
                  let request = post(&operation_response.address, OPERATION_ROUTE, &serde_json::to_string(&operation_request).unwrap()).await;
                  match request {
                    Ok(sk) => {
                      // println!("{:?}", operation_request);
                      let response = sk.json::<OperationResponse>().await.unwrap();
                      // println!("{:?}", response);
                      if response.accepted {
                        flag = true;
                      } else {
                        if idx < 0 {
                          flag = true;
                        } else {
                          operation_request.operations.insert(0, log[idx as usize].clone());
                          idx -= 1;
                          // println!("{}", idx);
                          if idx == -1 {
                            last_log = None;
                          } else {
                            last_log = Some(log[idx as usize].clone());
                          }
                          operation_request.previous_log_entry = last_log.clone();
                        }  
                      }
                    },
                    Err(e) => {
                      println!("{:?}", e);
                      flag = true;
                    }
                  }
                }
              }
            },
            Err(e) => {
              print!("{:?}", e);
            }
          }
        }

        let term = ctx.term.clone();
        ctx.log.push((term, add_node_operation.clone()));
        ctx.peers.push(new_node.sender.clone());
        res = true;        

    } else {
        res = false;
    }
    
    println!("Register node {} success.\n", new_node.sender.clone());
    println!("Peers : {:?}\n", ctx.peers.clone());
    // println!("Log : {:?}\n", ctx.log.clone());

    HttpResponse::Ok().body(serde_json::to_string(&RegisterResponse {
      accepted: res,
      term: ctx.term.clone(),
      peers: peers.clone(),
      log: ctx.log.clone(),
      queue: ctx.queue.clone()
    }).unwrap())
}