use crate::prelude::*;
use actix_web::rt::Runtime;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RegisterResponse {
  accepted: bool
}

pub async fn register(context: web::Data<Arc<Mutex<NodeInfo>>>, new_node: NodeInfo) -> impl Responder {
    let ctx = context.lock().unwrap();

    let mut res = false;

    if ctx.address == ctx.leader {
        let mut runtime = Runtime::new().unwrap();
        let requests = runtime.block_on(post_many(ctx.peers.clone(), "/operation", ""));
        
        // TODO: check log (consistent/not) if inconsistent do operation until consistent then send response
    }
    else {
        res = false;
    }

    HttpResponse::Ok().body(serde_json::to_string(&RegisterResponse {
        accepted: res
    }).unwrap())
}