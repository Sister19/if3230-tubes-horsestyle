use actix_web::{App, web};

pub mod ok;
pub mod ok_post;
pub mod operation;

pub const OK_ROUTE: &str = "/ok";
pub const OK_POST_ROUTE: &str = "/okPost";
pub const OPERATION_ROUTE: &str = "/operation";