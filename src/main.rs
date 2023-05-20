mod routes;
mod node;
mod operation;
mod request;
mod prelude {
    pub use std::{env};
    pub use actix_web::{HttpServer, main, App, web, rt::System, HttpResponse, Responder};
    pub use std::{ops::Add, error::Error, io};
    pub use std::sync::{Arc, Mutex};
    pub use crate::routes::*;
    pub use crate::operation::*;
    pub use crate::node::*;
    pub use crate::request::*;
    pub use serde::*;
}


use prelude::*;
const DEFAULT_PORT: u16 = 8000;
const DEFAULT_IP: &str = "127.0.0.1";

fn main() {
    let address = format!("{}:{}", DEFAULT_IP, get_port());
    let mut node: NodeInfo;
    if let Ok(leader) = get_leader() {
        node = NodeInfo::new(address.clone(), leader);
    } else {
        node = NodeInfo::new(address.clone(), address.clone());
    }
    node.start_node();
}

pub fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return DEFAULT_PORT;
    }
    args[1].parse().unwrap_or_else(|_| {
        println!("ARGS ERROR: Port cannot be read");
        println!("RESOLVE: using default port {}", DEFAULT_PORT);
        DEFAULT_PORT
    })
}

pub fn get_leader() -> Result<String, io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return Err(io::Error::new(io::ErrorKind::Other, "No leader address provided"));
    }
    Ok(args[2].parse().unwrap())
}

