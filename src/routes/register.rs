use crate::prelude::*;
use actix_web::rt::Runtime;

use super::operation::OperationRequest;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RegisterResponse {
  accepted: bool,
  node: NodeInfo
}

pub async fn register(context: web::Data<Arc<Mutex<NodeInfo>>>, new_node: web::Json<NodeInfo>) -> impl Responder {
    let mut ctx = context.lock().unwrap();

    println!("====================");
    println!("POST : REGISTER\n");
    println!("Sender : {}", new_node.address.clone());
    println!("Term : {}\n", new_node.term.clone());

    let mut res = false;

    let add_node_operation = &Operation {
        operation_type: OperationType::AddNode,
        content: Some(String::from(new_node.address.clone())),
        is_committed: None
    };
    if ctx.address == ctx.leader {
        let add_node_request = &OperationRequest{
            operations: vec![add_node_operation.clone()],
            sender: ctx.address.clone(),
            previous_log_entry: None,
            term: ctx.term.clone(),
        };

        let requests = post_many(ctx.peers.clone(), "/operation", &serde_json::to_string(&add_node_request).unwrap()).await;
        ctx.peers.push(new_node.address.clone());
        println!("{:?}", ctx.peers);
        // TODO: check log (consistent/not) if inconsistent do operation until consistent then send response
    }
    else {
        res = false;
    }
    
    println!("Register node {} success.\n", new_node.address.clone());

    HttpResponse::Ok().body(serde_json::to_string(&RegisterResponse {
        accepted: true,
        node: ctx.clone()
    }).unwrap())
}