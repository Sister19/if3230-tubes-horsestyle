use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OkRequest {
  value: String
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OkResponse {
  value: String
}

pub async fn ok_post(context: web::Data<Arc<Mutex<NodeInfo>>>,ok_request: web::Json<OkRequest>) -> impl Responder {
  let c = context.lock();
  let mut ctx = c.unwrap();  println!("ok_post");
  HttpResponse::Ok().body(serde_json::to_string(&OkResponse { value: ctx.queue.join("") }).unwrap())
}