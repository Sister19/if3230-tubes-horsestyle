use std::{sync::{Arc, Mutex}, thread, time::{Duration, SystemTime}, fmt};

use actix_web::rt::Runtime;

use crate::{prelude::*, get_port};
#[derive(Clone, PartialEq, Debug)]
pub enum NodeType {
  Follower,
  Leader,
  Candidate
}

#[derive(Clone, Debug)]
pub struct NodeInfo {
  pub last_heartbeat_received: SystemTime,
  pub election_timeout: Duration,
  pub term: i32,
  pub address: String,
  pub leader: String,
  pub node_type: NodeType,
  pub peers: Vec<String>,
  pub log: Vec<(i32, Operation)>,
  pub queue: Vec<String>,
}

impl NodeInfo {
  pub fn new(address: String, leader: String) -> Self {
    let random_number = rand::Rng::gen_range(&mut rand::thread_rng(), 300..500);
    NodeInfo {
      last_heartbeat_received: SystemTime::now(),
      election_timeout: Duration::from_millis(random_number),
      term: 0,
      address: address,
      leader: leader,
      node_type: NodeType::Follower,
      peers: Vec::new(),
      log: Vec::new(),
      queue: Vec::new(),
    }
  }

  pub fn start_node(&mut self) {
    let context: web::Data<Arc<Mutex<NodeInfo>>> = web::Data::new(Arc::new(Mutex::new(self.clone())));
    
    println!("====================");
    println!("Node Info : \n");
    println!("- Addresss : {}", self.address);
    println!("- Term : {}", self.term);
    println!("- Leader : {}", self.leader);
    println!("====================");
    
    self.run_loop_thread(context.clone());
    self.run_service_thread(context.clone());
  }
  
  fn run_service_thread(&mut self, context: web::Data<Arc<Mutex<NodeInfo>>>) {
    System::new().block_on(async {
        let port = get_port();
        println!("RUNNING PORT: {}\n", port);
        HttpServer::new(move || {
            App::new()
            .app_data(context.clone())
            .route(OK_ROUTE, web::get().to(ok::ok))
            .route(OK_POST_ROUTE, web::post().to(ok_post::ok_post))
            .route(OPERATION_ROUTE, web::post().to(operation::operation))
            .route(REQUEST_LOG_ROUTE, web::get().to(request_log::request_log))
        })
        .bind(self.address.clone())?
        .run()
        .await
    }).unwrap();
  }

  async fn run_loop_thread(&self, context: web::Data<Arc<Mutex<NodeInfo>>>) {
      std::thread::spawn(move || {
          loop {
            let mut node = context.lock().unwrap();
            match node.node_type {
              NodeType::Follower =>  {
                if SystemTime::now().duration_since(node.last_heartbeat_received).unwrap() > node.election_timeout {
                  node.node_type = NodeType::Candidate;
                  node.term += 1;
                  let mut runtime = Runtime::new().unwrap();
                  let results = runtime.block_on(post_many(node.peers.clone(), "/requestVote", ""));
                }
              },
              _ => {}
            }
          }
      });
  }
  

}
