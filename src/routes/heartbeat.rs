use crate::prelude::*;
use std::{time::{Duration, SystemTime}, fmt};
#[derive(serde::Deserialize, serde::Serialize, Debug)]

pub struct HeartbeatRequest{
  term: i32
}

// Asumsi : dikirimkan oleh leader doang
pub async fn heartbeat(context: web::Data<Arc<Mutex<NodeInfo>>>, heartbeat_request: web::Json<HeartbeatRequest>) -> impl Responder {
  let mut ctx: MutexGuard<NodeInfo> = context.lock().unwrap();
  let mut resp:bool = false;
  let term = heartbeat_request.term.clone();

  if (ctx.term == term) {
    // Lanjut
    if(ctx.node_type == NodeType::Follower) {
      if (SystemTime::now() - ctx.last_heartbeat_received < election_timeout){
        ctx.last_heartbeat_received = SystemTime::now();
      }
      println!("heartbeat_response: ok");
      resp = true;
    }else{
      resp = false;
    }
  } else{
    // Tolak
    resp = false;
  }

  HttpResponse::Ok().body(serde_json::to_string(&OperationResponse { 
    ok: resp
  }).unwrap())
}