use crate::prelude::*;

pub async fn request_log(context: web::Data<Arc<Mutex<NodeInfo>>>) -> impl Responder {
  // unwrap
  let ctx = context.lock().unwrap();
  
  // get log
  println!("====================");
  println!("GET : Request Log\n");
  println!("Log sent.");
  
  // response
  HttpResponse::Ok().body((serde_json::to_string(&ctx.log).unwrap()))
}