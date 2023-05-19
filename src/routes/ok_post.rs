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
  let mut ctx = context.lock().unwrap();
  ctx.value = ok_request.value.clone();
  HttpResponse::Ok().body(serde_json::to_string(&OkResponse { value: ctx.value.clone() }).unwrap())
}