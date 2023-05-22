use crate::prelude::*;

use super::operation::{OperationRequest, OperationResponse, OperationError};

/*
- /execute
    - struct Request {Operation: enum (Queue, Dequeue), Content: String}
    - kalo leader:
        - kirim /operation ke semua follower
            - cek apakah ada response mengandung INCONSISTENT_LOG
                - kalo ada diloop sampe accept
                    - last operation
                    - kalau ada, kirim /operation dengan Operations = Log[last_operation-1: last_operation], last_operation = last_operation - 1 PreviousLogEntry = Log[last_operation - 1]
        - kalau 50% + 1 ack, commit
            - kirim /operation commit
    - else:
        - tolak 
*/
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ExecuteRequest{
  pub operation_type: OperationType,
  pub content: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ExecuteResponse{
  pub accepted: bool,
}

// Prekondisi : operation_to_execute nya hanya Queue dan Dequeue
pub async fn execute(context: web::Data<Arc<Mutex<NodeInfo>>>, operation_to_execute: web::Json<ExecuteRequest>) -> impl Responder {
  let mut ctx = context.lock().unwrap();
  
  println!("====================");
  println!("EXECUTE : Operation\n");
  println!("Operation : {:?}", operation_to_execute.operation_type.clone());
  println!("Content : {:?}\n", operation_to_execute.content.clone().unwrap());

  let term = ctx.term.clone();
  let mut result = false;
  let operation = &Operation {
    operation_type: operation_to_execute.operation_type.clone(),
    content: Some(operation_to_execute.content.clone().unwrap()),
    is_committed: None
  };
  
  let mut last_log: Option<(i32, Operation)>;
  if ctx.log.len() > 0 {
    last_log = Some(ctx.log[ctx.log.len()-1].clone());
  } else {
    last_log = None;
  }

  if ctx.node_type == NodeType::Leader {
    // kirim /operation ke semua follower
    let mut operation_request = OperationRequest{
      operations: vec![(ctx.term.clone(), operation.clone())],
      sender: ctx.address.clone(),
      previous_log_entry: last_log,
      term: ctx.term.clone(),
    };

    // println!("{:?}", operation_request);

    let mut responses = post_many(ctx.peers.clone(), OPERATION_ROUTE, &serde_json::to_string(&operation_request).unwrap()).await;
    ctx.log.push((term, operation.clone()));

    let mut success = 0;
    let mut count = 0;
    for mut response in responses {
      match response {
        Ok(sk) => {
          let operation_response = sk.json::<OperationResponse>().await.unwrap();
          count += 1;
          if operation_response.accepted {
            success += 1;
          }else{
            if (operation_response.flag == OperationError::InconsistentLog){
              // ... Masalah blocking dan cuma ngirim operation (termnya kenapa ngga)

              let mut flag = false;
              let mut log = ctx.log.clone();
              let n = log.len() as i32;
              let mut idx = n-1;
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
                      success += 1;
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
          }
        },
        Err(e) => {
          count += 1;
          print!("{:?}", e);
        }
      }
    };

    if (success >= (count / 2 + 1)) { // handle commit yang gagal di term sebelumnya(?) - udah dihandle inconsistent log
      let last_log_idx = ctx.log.len()-1;
      last_log = Some(ctx.log[ctx.log.len()-1].clone());
      if ctx.log[last_log_idx].1.operation_type == OperationType::Queue {
        let el = ctx.log[last_log_idx].1.content.clone().unwrap();
        ctx.queue.push(el.clone());
        println!("Queue : enqueue \"{}\" to the queue\n", el);
      } else if ctx.log[last_log_idx].1.operation_type == OperationType::Dequeue {
        let el = ctx.queue.remove(0);
        println!("Queue : dequeue {} from the queue\n", el);
      }
      ctx.log[last_log_idx].1.is_committed = Some(true);
      println!("Commit : Commit applied \nQueue : {:?}\n", ctx.queue);
      
      let commit_operation = &Operation {
        operation_type: OperationType::Commit,
        content: None,
        is_committed: Some(true)
      };
      let commit_request= &OperationRequest{
        operations: vec![(ctx.term.clone(), commit_operation.clone())],
        sender: ctx.address.clone(),
        previous_log_entry: last_log.clone(),
        term: ctx.term.clone(),
      };
      if ctx.log.len() > 0 {
        last_log = Some(ctx.log[ctx.log.len()-1].clone());
      } else {
        last_log = None;
      }
      ctx.log.push((term, commit_operation.clone()));
      post_many(ctx.peers.clone(), OPERATION_ROUTE, &serde_json::to_string(&commit_request).unwrap()).await;
      result = true;
    }
  }

  // println!("{:?}", ctx.log);

  // response
  HttpResponse::Ok().body(serde_json::to_string(&ExecuteResponse { 
    accepted: result
    }).unwrap())
}
