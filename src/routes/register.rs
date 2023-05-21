use crate::prelude::*;
use actix_web::rt::Runtime;

use super::operation::OperationRequest;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RegisterResponse {
  accepted: bool,
  node: NodeInfo
}

pub async fn register(context: web::Data<Arc<Mutex<NodeInfo>>>, new_node: web::Json<NodeInfo>) -> impl Responder {
    let ctx = context.lock().unwrap();

    println!("====================");
    println!("POST : REGISTER\n");
    println!("Sender : {}", new_node.address.clone());
    println!("Term : {}\n", new_node.term.clone());

    let mut res = false;

    let operations: Vec<Operation> = Vec::new();
    let add_node_operation = &Operation {
        operation_type: OperationType::AddNode,
        content: Some(String::from(new_node.address.clone())),
        is_committed: None
    };

    if ctx.address == ctx.leader {
        let mut runtime = Runtime::new().unwrap();

        let addNodeRequest = &OperationRequest{
            operations: operations.clone(),
            sender: ctx.address.clone(),
            previous_log_entry: None,
            term: ctx.term.clone(),
        };

        // let requests = runtime.block_on(post_many(ctx.peers.clone(), "/operation", &serde_json::to_string(&addNodeRequest).unwrap()));
        
        // TODO: check log (consistent/not) if inconsistent do operation until consistent then send response
    }
    else {
        res = false;
    }

    println!("Register node {} success.\n", new_node.address.clone());

    HttpResponse::Ok().body(serde_json::to_string(&RegisterResponse {
        accepted: res,
        node: ctx.clone()
    }).unwrap())
}