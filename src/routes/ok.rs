use crate::prelude::*;

pub async fn ok(context: web::Data<Arc<Mutex<NodeInfo>>>) -> impl Responder {
  let ctx = context.lock().unwrap();
  HttpResponse::Ok().body(ctx.value.clone())
}