use actix_web::{Responder, HttpResponse};

pub async fn ok() -> impl Responder {
  HttpResponse::Ok().body("Service running")
}