mod routes;
mod node;
mod operation;
mod prelude {
    pub use std::{env};
    pub use actix_web::{HttpServer, main, App, web, rt::System};
    pub use crate::routes::*;
    pub use crate::operation::*;
    pub use crate::node::*;
}

use prelude::*;
const DEFAULT_PORT: u16 = 8080;

fn main() {
    node_loop();
    node_service();
}

fn node_service() {
    System::new().block_on(async {
        let port = get_port();
        println!("RUNNING PORT: {}", port);
        HttpServer::new(|| {
            App::new()
            .route(OK_ROUTE, web::get().to(ok::ok))
        })
        .bind(("127.0.0.1", port))?
        .run()
        .await
    }).unwrap();
}

fn node_loop() {
    std::thread::spawn(|| {
        loop {
            
        }
    });
}

fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return 8080;
    }
    args[1].parse().unwrap_or_else(|_| {
        println!("ARGS ERROR: Port cannot be read");
        println!("RESOLVE: using default port {}", DEFAULT_PORT);
        DEFAULT_PORT
    })
}
