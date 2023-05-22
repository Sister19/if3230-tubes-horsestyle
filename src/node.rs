use std::{sync::{Arc, Mutex}, thread, time::{Duration, SystemTime}, fmt};

use actix_web::rt::Runtime;

use crate::{prelude::*, get_port, routes::{register::{RegisterRequest, RegisterResponse}, heartbeat::HeartbeatRequest}};
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum NodeType {
  Follower,
  Leader,
  Candidate
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

    if self.address == self.leader {
      self.node_type = NodeType::Leader;
    } else {
      println!("====================");
      println!("Registering this node to the term ...\n");
      
      let register_request = &RegisterRequest {
        sender: self.address.clone(),
        term: self.term.clone()
      };

      let mut runtime = Runtime::new().unwrap();
      let result = runtime.block_on(post(&self.leader, REGISTER_ROUTE, &serde_json::to_string(&register_request).unwrap()));
  
      match result {
        Ok(sk) => { 
          System::new().block_on(async {
            let register_response = sk.json::<RegisterResponse>().await.unwrap();
            // println!("{:?}", register_response);

            if register_response.accepted {
              self.term = register_response.term.clone();
              self.peers.push(self.leader.clone());
              for peer in register_response.peers {
                self.peers.push(peer.clone());
              }
              self.log = register_response.log.clone();
              self.queue = register_response.queue.clone();
            }
          });
        },
        Err(e) => {
          print!("{:?}", e);
        }
        
      }

      println!("Node registered. \n");

    }

    println!("====================");
    println!("Node Info : \n");
    println!("- Addresss : {}", self.address.clone());
    println!("- Term : {}", self.term.clone());
    println!("- Leader : {}", self.leader.clone());
    println!("- Peers : {:?}", self.peers.clone());
    println!("====================");
    
    let context: web::Data<Arc<Mutex<NodeInfo>>> = web::Data::new(Arc::new(Mutex::new(self.clone())));
    let context_service = context.clone();
    std::thread::spawn(move || {
      loop {
          let mut node = context.lock().unwrap();
          match node.node_type {
            NodeType::Follower =>  {
              if SystemTime::now().duration_since(node.last_heartbeat_received).unwrap() > node.election_timeout {
                node.node_type = NodeType::Candidate;
                node.term += 1;
                let mut runtime = Runtime::new().unwrap();
                let results = runtime.block_on(post_many(node.peers.clone(), "/requestVote", &String::from("")));
              }
            },
            NodeType::Leader => {
              let mut heartbeat_request = HeartbeatRequest {
                term: node.term.clone()
              };
              let mut runtime = Runtime::new().unwrap();
              let results = runtime.block_on(post_many(node.peers.clone(), HEARTBEAT_ROUTE, &serde_json::to_string(&heartbeat_request).unwrap()));
              for result in results {
                match result {
                  Ok(sk) => {
                    println!("{:?}", sk);
                  },
                  Err(e) => {
                    println!("{:?}", e);
                  }
                }
              }
            },
            _ => {}
          }
        }
    });
    self.run_service_thread(context_service.clone());
  }
  
  fn run_service_thread(&mut self, context: web::Data<Arc<Mutex<NodeInfo>>>) {
    System::new().block_on(async {
        println!("Oke");
        let port = get_port();
        println!("RUNNING PORT: {}\n", port);
        HttpServer::new(move || {
            App::new()
            .app_data(context.clone())
            .route(OK_ROUTE, web::get().to(ok::ok))
            .route(OK_POST_ROUTE, web::post().to(ok_post::ok_post))
            .route(OPERATION_ROUTE, web::post().to(operation::operation))
            .route(REQUEST_LOG_ROUTE, web::get().to(request_log::request_log))
            .route(REQUEST_VOTE_ROUTE, web::post().to(request_vote::request_vote))
            .route(REGISTER_ROUTE, web::post().to(register::register))
            .route(EXECUTE_ROUTE, web::post().to(execute::execute))
            .route(HEARTBEAT_ROUTE, web::post().to(heartbeat::heartbeat))
        })
        .bind(self.address.clone())?
        .run()
        .await
    }).unwrap();
  }
}
