use crate::prelude::*;
use std::{time::{Duration, SystemTime}, fmt};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeartbeatRequest{
  pub term: i32,
  pub address: String
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeartbeatResponse{
  pub accepted: bool
}

// Asumsi : dikirimkan oleh leader doang
pub async fn heartbeat(context: web::Data<Arc<Mutex<NodeInfo>>>, heartbeat_request: web::Json<HeartbeatRequest>) -> impl Responder {
  let mut ctx = context.lock().unwrap();
  let mut resp:bool = false;
  let term = heartbeat_request.term.clone();
  let address = heartbeat_request.address.clone();

  if ctx.node_type.clone() == NodeType::Candidate {
    ctx.term -= 1;
    let random_number = rand::Rng::gen_range(&mut rand::thread_rng(), 1000..5000);
    ctx.last_heartbeat_received = SystemTime::now();
    ctx.election_timeout = Duration::from_millis(random_number);
  }

  if ctx.leader != address {
    ctx.leader = address.clone();
  }

  if (ctx.term == term) {
    // println!("{:?} < {:?}", ctx.last_heartbeat_received.elapsed().unwrap(), ctx.election_timeout);
    // println!("{}", ctx.last_heartbeat_received.elapsed().unwrap() < ctx.election_timeout);
    if (ctx.node_type == NodeType::Follower) {
      ctx.last_heartbeat_received = SystemTime::now();
      resp = true;
    } else{
      resp = false;
    }
  } else {
    // Tolak
    println!("heartbeat_response: not ok");
    resp = false;
  }

  HttpResponse::Ok().body(serde_json::to_string(&HeartbeatResponse { 
    accepted: resp
  }).unwrap())
}