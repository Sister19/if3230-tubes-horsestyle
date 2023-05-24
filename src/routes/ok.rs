use crate::prelude::*;

pub async fn ok(context: web::Data<Arc<Mutex<NodeInfo>>>) -> impl Responder {
  let ctx = context.lock().unwrap();
  HttpResponse::Ok().body((ctx.node_type == NodeType::Leader).to_string())
}