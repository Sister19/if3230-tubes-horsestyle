use std::{sync::{Arc, Mutex}, thread, time::Duration};

use crate::{prelude::*, get_port};
#[derive(Clone)]
pub enum NodeType {
  Follower,
  Leader,
  Candidate
}

#[derive(Clone)]
pub struct NodeInfo {
  pub term: i32,
  address: String,
  pub leader: String,
  pub node_type: NodeType,
  pub peers: Vec<String>,
  pub log: Vec<(i32, Operation)>,
  pub value: String
}

impl NodeInfo {
  pub fn new(address: String, leader: String) -> Self {
    NodeInfo {
      term: 0,
      address: address,
      leader: leader,
      node_type: NodeType::Follower,
      peers: Vec::new(),
      log: Vec::new(),
      value: String::new()
    }
  }

  pub fn start_node(&mut self) {
    let context: web::Data<Arc<Mutex<NodeInfo>>> = web::Data::new(Arc::new(Mutex::new(self.clone())));
    self.run_loop_thread(context.clone());
    self.run_service_thread(context.clone());
  }
  
  fn run_service_thread(&mut self, context: web::Data<Arc<Mutex<NodeInfo>>>) {
    System::new().block_on(async {
        let port = get_port();
        println!("RUNNING PORT: {}", port);
        HttpServer::new(move || {
            App::new()
            .app_data(context.clone())
            .route(OK_ROUTE, web::get().to(ok::ok))
            .route(OK_POST_ROUTE, web::post().to(ok_post::ok_post))
        })
        .bind(self.address.clone())?
        .run()
        .await
    }).unwrap();
  }

  fn run_loop_thread(&self, context: web::Data<Arc<Mutex<NodeInfo>>>) {
      std::thread::spawn(move || {
          loop {
            println!("{}", context.lock().unwrap().value.clone());
            thread::sleep(Duration::from_secs(5));
          }
      });
  }
  

}
