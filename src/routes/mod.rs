use actix_web::{App, web};

pub mod ok;
pub mod ok_post;

pub const OK_ROUTE: &str = "/ok";
pub const OK_POST_ROUTE: &str = "/okPost";