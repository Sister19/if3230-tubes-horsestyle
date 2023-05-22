use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ReqVoteRequest {
  candidate: String,
  last_log_entry: Option<(i32, Operation)>,
  term: i32
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ReqVoteResponse {
  accepted: bool
}

pub async fn request_vote(context: web::Data<Arc<Mutex<NodeInfo>>>, reqvote_request: web::Json<ReqVoteRequest>) -> impl Responder {
    let ctx = context.lock().unwrap();

    let candidate = reqvote_request.candidate.clone();
    let last_log_entry = reqvote_request.last_log_entry.clone();
    let term = reqvote_request.term.clone();

    // initialize response
    let mut res = false;

    println!("====================");
    println!("POST : Request Vote\n");
    println!("Candidate : {}", candidate);
    println!("Term : {}\n", term);

    if (ctx.term == term) {
        let last_log = ctx.log[ctx.log.len() - 1].clone();
        if (last_log_entry.clone().unwrap().0 <= last_log.0) {
            res = true;
            print!("Request vote sent.")
        }
        else {
            res = false;
        }
    }
    else {
        res = false;
    }

    // response
    HttpResponse::Ok().body(serde_json::to_string(&ReqVoteResponse {
        accepted: res
    }).unwrap())
}