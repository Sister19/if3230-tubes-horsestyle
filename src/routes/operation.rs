use std::time::{SystemTimeError, SystemTime};

use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
pub enum OperationError{
  InconsistentLog,
  InvalidOperation,
  None
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct OperationRequest {
  pub operations: Vec<(i32, Operation)>,
  pub sender: String,
  pub previous_log_entry: Option<(i32, Operation)>,
  pub term: i32
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct OperationResponse {
  pub accepted: bool,
  pub address: String,
  pub flag: OperationError,
  pub note: String
}

pub async fn operation(context: web::Data<Arc<Mutex<NodeInfo>>>, operation_request: web::Json<OperationRequest>) -> impl Responder {
  // unwrap
  let mut ctx = context.lock().unwrap();

  let operations = operation_request.operations.clone();
  let sender = operation_request.sender.clone();
  let previous_log_entry = operation_request.previous_log_entry.clone();
  let term = operation_request.term.clone();

  // initialize response
  let mut result = false;
  let mut note = String::new();
  let mut err_operation = OperationError::None;

  println!("====================");
  println!("POST : Operation\n");
  println!("Sender : {}", sender);
  println!("Term : {}\n", term);
  println!("Log : {:?}\n", ctx.log.clone());
  println!("Prev Log : {:?}\n", previous_log_entry.clone());
  // println!("{:?}\n", operation_request);

  if sender != ctx.leader && !ctx.election_status {
    result = false;
    note = format!("Error : Sender is not a Leader");
  } else {
    if ctx.term <= term {
      if ctx.node_type == NodeType::Candidate {
        result = false;
        note = format!("Error : I'm a new candidate");
      } else if ctx.node_type == NodeType::Follower || ctx.node_type == NodeType::Leader {

        // cek last log
        // let n = ctx.log.len();
        let mut flag = false;
        if ctx.log.len() == 0 {
          if previous_log_entry.is_none() {
            flag = true;
            ctx.term = term.clone();
          } else {
            flag = false;
            note = format!("Error : Different last log");
          }
        } else {
          let last_log = ctx.log[ctx.log.len()-1].clone();
          if (previous_log_entry.clone().unwrap().0 == last_log.0) && (previous_log_entry.clone().unwrap().1.is_equal(last_log.1.clone())) {
            flag = true;
            ctx.term = term.clone();
          } else {
            result = false;
            err_operation = OperationError::InconsistentLog;
            note = format!("Error : Different last log");
          }
        }
        
        println!("{:?}", flag);
        // jika last log sama
        if flag {

          println!("Operations running ...\n");
          
          for operation in operations {
            let new_operation = operation.clone();
            // execute operation
            if operation.1.operation_type == OperationType::Queue {
              
              let el = operation.1.content.unwrap();
              println!("Log : add enqueue \"{}\" to the queue\n", el);
  
            } else if operation.1.operation_type == OperationType::Dequeue {
              
              println!("Log : add dequeue from the queue\n");
            
            } else if operation.1.operation_type == OperationType::AddNode {
              
              let node = operation.clone().1.content.unwrap();
              ctx.peers.push(node.clone());
              println!("AddNode : add new node \"{}\" to peers", node);
              println!("Peers : {:?}\n", ctx.peers.clone());
            
            } else if operation.1.operation_type == OperationType::ChangeLeader {
              
              ctx.election_status = false;
              let new_leader = operation.clone().1.content.unwrap();
              let old_leader = ctx.leader.clone();
              let random_number = rand::Rng::gen_range(&mut rand::thread_rng(), 3000..7000);
              ctx.leader = new_leader.clone();
              ctx.election_timeout = Duration::from_millis(random_number);
              
              println!("{:?}", ctx.term);
              println!("ChangeLeader : change leader from \"{}\" to \"{}\"\n", old_leader, new_leader);
            
            } else if operation.1.operation_type == OperationType::Commit {
              
              let last_log_idx = ctx.log.len()-1;
              if ctx.log[last_log_idx].1.operation_type == OperationType::Queue {
                let el = ctx.log[last_log_idx].1.content.clone().unwrap();
                ctx.queue.push(el.clone());
                println!("Queue : enqueue \"{}\" to the queue\n", el);
              } else if ctx.log[last_log_idx].1.operation_type == OperationType::Dequeue {
                let mut el = String::new();
                if ctx.queue.len() > 0 {
                  el = ctx.queue.remove(0);
                } else {
                  println!("Queue : queue is empty\n");
                  return HttpResponse::Ok().body(serde_json::to_string(&OperationResponse { 
                    accepted: false,
                    note: String::from("Queue is empty"),
                    address: ctx.address.clone(),
                    flag: OperationError::InvalidOperation
                  }).unwrap());
                }
                println!("Queue : dequeue {} from the queue\n", el);
              }
              ctx.log[last_log_idx].1.is_committed = Some(true);
              println!("Commit : Commit applied \nQueue : {:?}\n", ctx.queue);
  
            } else if operation.1.operation_type == OperationType::None {
              println!("None\n");
            }
            
            if operation.1.operation_type != OperationType::None {
              ctx.log.push(new_operation);
            } 
            
          }

          result = true;
          note = format!("Operation Success");
          println!("Operations end.\n");
          // println!("Log : {:?}\n", ctx.log);
        }

      } else {
        result = false;
        note = format!("Error : This node is a leader");
      }
    } else {
      result = false;
      note = format!("Error : Sender's term ({}) lower than this node ({})", term, ctx.term);
    }
  }

  ctx.last_heartbeat_received = SystemTime::now();

  // response
  HttpResponse::Ok().body(serde_json::to_string(&OperationResponse { 
    accepted: result,
    note: note,
    address: ctx.address.clone(),
    flag: err_operation
  }).unwrap())
}