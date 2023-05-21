use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct OperationRequest {
  pub operations: Vec<Operation>,
  pub sender: String,
  pub previous_log_entry: Option<(i32, Operation)>,
  pub term: i32
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct OperationResponse {
  pub accepted: bool,
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

  println!("====================");
  println!("POST : Operation\n");
  println!("Sender : {}", sender);
  println!("Term : {}\n", term);

  if sender != ctx.leader {
    result = false;
    note = format!("Error : Sender is not a Leader");
  } else {
    if ctx.term == term {
      if ctx.node_type == NodeType::Candidate {
        result = false;
        note = format!("Error : I'm a new candidate");
      } else if ctx.node_type == NodeType::Follower {

        // cek last log
        let mut flag = false;
        if ctx.log.len() == 0 {
          flag = true;
        } else {
          let last_log = ctx.log[ctx.log.len()-1].clone();
          if (previous_log_entry.clone().unwrap().0 == last_log.0) && (previous_log_entry.clone().unwrap().1.is_equal(last_log.1.clone())) {
            flag = true;
          } else {
            result = false;
            note = format!("Error : Different last log");
          }
        }
        
        println!("Operations running ...\n");
        
        // jika last log sama
        if flag {
          
          let mut new_idx: i32;
          for operation in operations {
            new_idx = ctx.log.len() as i32;
          
            // execute operation
            if operation.operation_type == OperationType::Queue {
              
              let new_operation = (new_idx, operation.clone());
              ctx.log.push(new_operation);
              let el = operation.content.unwrap();
              println!("Log : add enqueue \"{}\" to the queue\n", el);
  
            } else if operation.operation_type == OperationType::Dequeue {
              
              let new_operation = (new_idx, operation.clone());
              ctx.log.push(new_operation);
              println!("Log : add dequeue from the queue\n");
            
            } else if operation.operation_type == OperationType::AddNode {
              
              let node = operation.clone().content.unwrap();
              ctx.peers.push(node.clone());
              println!("AddNode : add new node \"{}\" to peers", node);
              println!("Peers : {:?}\n", ctx.peers.clone());
            
            } else if operation.operation_type == OperationType::ChangeLeader {
              
              let new_leader = operation.clone().content.unwrap();
              let old_leader = ctx.leader.clone();
              let random_number = rand::Rng::gen_range(&mut rand::thread_rng(), 300..500);
              ctx.leader = new_leader.clone();
              ctx.election_timeout = Duration::from_millis(random_number);
              println!("ChangeLeader : change leader from \"{}\" to \"{}\"\n", old_leader, new_leader);
            
            } else if operation.operation_type == OperationType::Commit {
              
              let last_log_idx = ctx.log.len()-1;
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
  
            } else if operation.operation_type == OperationType::None {
              println!("None\n");
            }
  
          }

          result = true;
          note = format!("Operation Success");
          println!("Operations end.\n");
        }

      } else {
        result = false;
        note = format!("Error : This node is a leader");
      }
    } else {
      result = false;
      note = format!("Error : Different owner term ({}) from sender term ({})", ctx.term, term);
    }
  }

  // response
  HttpResponse::Ok().body(serde_json::to_string(&OperationResponse { 
    accepted: result,
    note: note
    }).unwrap())
}