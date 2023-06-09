use actix_web::{App, web};

pub mod ok;
pub mod ok_post;
pub mod operation;
pub mod request_log;
pub mod request_vote;
pub mod register;
pub mod execute;
pub mod heartbeat;

pub const OK_ROUTE: &str = "/ok";
pub const OK_POST_ROUTE: &str = "/okPost";
pub const OPERATION_ROUTE: &str = "/operation";
pub const REQUEST_LOG_ROUTE: &str = "/requestLog";
pub const REQUEST_VOTE_ROUTE: &str = "/requestVote";
pub const REGISTER_ROUTE: &str = "/register";
pub const EXECUTE_ROUTE: &str = "/execute";
pub const HEARTBEAT_ROUTE: &str = "/heartbeat";