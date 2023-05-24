use std::time::SystemTime;

use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ReqVoteRequest {
  pub candidate: String,
  pub term: i32
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ReqVoteResponse {
  pub accepted: bool
}

pub async fn request_vote(context: web::Data<Arc<Mutex<NodeInfo>>>, reqvote_request: web::Json<ReqVoteRequest>) -> impl Responder {
    let c = context.lock();
    let mut ctx = c.unwrap();

    let candidate = reqvote_request.candidate.clone();
    let term = reqvote_request.term.clone();

    // initialize response
    let mut res = false;

    println!("====================");
    println!("POST : Request Vote\n");
    println!("Candidate : {}", candidate);
    println!("Term : {}\n", term);
    
    ctx.election_status = true;

    if (ctx.term < term) {
        res = true;
        println!("Request vote sent.");
    }
    else {
        res = false;
    }
    
    if ctx.node_type.clone() == NodeType::Candidate {
        ctx.node_type = NodeType::Follower;
        ctx.term -= 1;
        let random_number = rand::Rng::gen_range(&mut rand::thread_rng(), 30000..45000);
        ctx.last_heartbeat_received = SystemTime::now();
        ctx.election_timeout = Duration::from_millis(random_number);
    }
    // response
    HttpResponse::Ok().body(serde_json::to_string(&ReqVoteResponse {
        accepted: res
    }).unwrap())
}