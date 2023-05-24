use std::{sync::{Arc, Mutex}, thread, time::{Duration, SystemTime}, fmt};

use actix_web::rt::Runtime;

use crate::{prelude::*, get_port, routes::{register::{RegisterRequest, RegisterResponse}, heartbeat::{HeartbeatRequest, HeartbeatResponse}, request_vote::{ReqVoteRequest, ReqVoteResponse}, operation::{OperationRequest, OperationResponse}}};
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
  pub election_status: bool
}

impl NodeInfo {
  pub fn new(address: String, leader: String) -> Self {
    let random_number = rand::Rng::gen_range(&mut rand::thread_rng(), 30000..45000);
    let time = SystemTime::now();
    NodeInfo {
      last_heartbeat_received: time,
      election_timeout: Duration::from_millis(random_number),
      term: 0,
      address: address,
      leader: leader,
      node_type: NodeType::Follower,
      peers: Vec::new(),
      log: Vec::new(),
      queue: Vec::new(),
      election_status: false
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
            let temp = sk.json::<RegisterResponse>().await;
            let register_response = temp.unwrap();
            // println!("{:?}", register_response);

            if register_response.accepted {
              self.last_heartbeat_received = SystemTime::now();
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
    println!("- Log : {:?}", self.log.clone());
    println!("====================");
    
    let context: web::Data<Arc<Mutex<NodeInfo>>> = web::Data::new(Arc::new(Mutex::new(self.clone())));
    let context_service = context.clone();
    
    std::thread::spawn(move || {
      loop {
        let mut node = context.lock().unwrap();
        match node.node_type {
          NodeType::Follower =>  {
            check_election_timeout(&mut node);
          },
          NodeType::Leader => {
            if node.last_heartbeat_received.elapsed().unwrap() > Duration::from_millis(2000) {
              heartbeat(node);
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

fn heartbeat(mut node: std::sync::MutexGuard<NodeInfo>) {
  let mut heartbeat_request = HeartbeatRequest {
    term: node.term.clone(),
    address: node.address.clone()
  };
  let mut runtime = Runtime::new().unwrap();
  let results = runtime.block_on(post_many(node.peers.clone(), HEARTBEAT_ROUTE, &serde_json::to_string(&heartbeat_request).unwrap()));
  let mut i = 0;
  let mut new_peers = Vec::new();
  for result in results {
    match result {
      Ok(sk) => {
        // let response = runtime.block_on(sk.json::<HeartbeatResponse>()).unwrap();
        new_peers.push(node.peers[i].clone());
      },
      Err(e) => {
      }
    }
    i += 1;
  }
  node.peers = new_peers;
  node.last_heartbeat_received = SystemTime::now();
}

fn check_election_timeout(node: &mut std::sync::MutexGuard<NodeInfo>) {
  if (SystemTime::now().duration_since(node.last_heartbeat_received).unwrap() > node.election_timeout) && !node.election_status {
    node.node_type = NodeType::Candidate;
    node.term += 1;

    println!("Election timeout passed. New election leader begun.\n");

    let mut runtime = Runtime::new().unwrap();
    let mut request_vote = ReqVoteRequest {
      candidate: node.address.clone(),
      term: node.term.clone()
    };
    let mut success = 0;
    let mut count = 0;
    let results = runtime.block_on(post_many(node.peers.clone(), REQUEST_VOTE_ROUTE, &serde_json::to_string(&request_vote).unwrap()));
    for result in results {
      match result {
        Ok(sk) => {
          let response = runtime.block_on(sk.json::<ReqVoteResponse>()).unwrap();
          println!("{:?}", response.accepted);
          if (response.accepted) {
            success += 1;
          }
          count += 1;
          // println!("{:?}", response);
        },
        Err(e) => {
          // println!("{:?}", e);
        }
      }
    };

    if success >= ((count/2)) {
      node.leader = node.address.clone();

      let change_leader_operation = &Operation {
        operation_type: OperationType::ChangeLeader,
        content: Some(String::from(node.leader.clone())),
        is_committed: None
      };

      let mut last_log: Option<(i32, Operation)>;
      if node.log.len() > 0 {
        last_log = Some(node.log[node.log.len()-1].clone());
      } else {
        last_log = None;
      }

      let change_leader_request = &OperationRequest{
        operations: vec![(node.term.clone(), change_leader_operation.clone())],
        sender: node.address.clone(),
        previous_log_entry: last_log.clone(),
        term: node.term.clone(),
      };

      println!("Sending change leader request...\n");
      let responses = runtime.block_on(post_many(node.peers.clone(), OPERATION_ROUTE, &serde_json::to_string(&change_leader_request).unwrap()));
      
      for response in responses {
        match response {
          Ok(sk) => {
            let temp = runtime.block_on(sk.json::<OperationResponse>());
            let operation_response = temp.unwrap();
            if operation_response.accepted {
              count += 1;
            } else if operation_response.note == "Error : Different last log" {
              // check log (consistent/not) if inconsistent do operation until consistent then send response
              println!("Error : Different last log");

              let mut flag = false;
              let mut log = node.log.clone();
              let n = log.len() as i32;
              let mut idx = n-1;
              let mut operation_request: OperationRequest = OperationRequest{
                operations: vec![(node.term.clone(), change_leader_operation.clone())],
                sender: node.address.clone(),
                previous_log_entry: last_log.clone(),
                term: node.term.clone(),
              };
              if idx < 0 {
                flag = true;
              } else {
                operation_request.operations.insert(0, log[idx as usize].clone());
                idx -= 1;
                if idx == -1 {
                  last_log = None;
                } else {
                  last_log = Some(log[idx as usize].clone());
                }
              }  
              while !flag {
                let request = runtime.block_on(post(&operation_response.address, OPERATION_ROUTE, &serde_json::to_string(&operation_request).unwrap()));
                match request {
                  Ok(sk) => {
                    // println!("{:?}", operation_request);
                    let response = runtime.block_on(sk.json::<OperationResponse>()).unwrap();
                    // println!("{:?}", response);
                    if response.accepted {
                      flag = true;
                    } else {
                      if idx < 0 {
                        flag = true;
                      } else {
                        operation_request.operations.insert(0, log[idx as usize].clone());
                        idx -= 1;
                        // println!("{}", idx);
                        if idx == -1 {
                          last_log = None;
                        } else {
                          last_log = Some(log[idx as usize].clone());
                        }
                        operation_request.previous_log_entry = last_log.clone();
                      }  
                    }
                  },
                  Err(e) => {
                    // println!("{:?}", e);
                    flag = true;
                  }
                }
              }
            }
          },
          Err(e) => {
            // print!("{:?}", e);
          }
        }
      }

      let term = node.term.clone();
      node.node_type = NodeType::Leader;
      node.log.push((term, change_leader_operation.clone()));

      println!("Election leader stop. I am the new leader.");
      return;
      // println!("{:?}", node.term);
    }
  }
}
