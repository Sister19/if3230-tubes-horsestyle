use std::{io::{self, Write}, error::Error};

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

pub const EXECUTE_ROUTE: &str = "/execute";
pub const REQUEST_LOG_ROUTE: &str = "/requestLog";


#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ExecuteRequest{
  pub operation_type: OperationType,
  pub content: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ExecuteResponse{
  pub accepted: bool,
  pub content: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize, Debug)]
pub enum OperationType {
  Queue,
  Dequeue,
  AddNode,
  ChangeLeader,
  Commit,
  None
}

#[tokio::main]
async fn main() {
    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim_end_matches('\n').trim();

        if input == "exit" {
            break;
        }
        let commands: Vec<String> = input.split(" ").map(|c| c.to_string()).collect();
        println!("{:?}", commands);
        if commands[0] == "queue" {
          let execution_request = &ExecuteRequest{
            operation_type: OperationType::Queue,
            content: Some(commands[1].clone()),
          };
          let response = post(commands[2].as_str(), EXECUTE_ROUTE, &serde_json::to_string(&execution_request).unwrap()).await;
          match response {
            Ok(sk) => {
              let execution_response = sk.json::<ExecuteResponse>().await.unwrap();
              println!("Accepted : {}", execution_response.accepted);
              println!("Content : {:?}", execution_response.content);
            },
            Err(e) => {
              println!("Error : {:?}", e);
            }
          };
        } else if commands[0] == "dequeue" {
          let execution_request = &ExecuteRequest{
            operation_type: OperationType::Dequeue,
            content: None,
          };
          let response = post(commands[1].as_str(), EXECUTE_ROUTE, &serde_json::to_string(&execution_request).unwrap()).await;
          match response {
            Ok(sk) => {
              let execution_response = sk.json::<ExecuteResponse>().await.unwrap();
              println!("Accepted : {}", execution_response.accepted);
              println!("Content : {:?}", execution_response.content);
            },
            Err(e) => {
              println!("Error : {:?}", e);
            }
          };
        } else if commands[0] == "request_log" {
          let response = get(commands[1].clone(), REQUEST_LOG_ROUTE.to_string()).await;
          match response {
            Ok(sk) => {
              let execution_response = sk.text().await.unwrap();
              println!("Log : {:?}", execution_response);
            },
            Err(e) => {
              println!("Error : {:?}", e);
            }
          };
        }
    }
}

async fn post(address: &str, path: &str, body: &str) -> Result<Response, Box<dyn Error>> {
  let client = Client::new();
  let url = format!("http://{}{}", address, path);
  // println!("{}", url);
  // println!("{}", body);
  match client.post(&url).body(body.to_owned()).header("content-type", "application/json").send().await {
    Ok(response) => {
     return Ok(response);
    },
    Err(e) => {
      return Err(Box::new(e));
    }
  };
}
async fn get(address: String, path: String) -> Result<Response, Box<dyn Error>> {
  let client = Client::new();
  let url = format!("http://{}{}", address, path);
  let response = client.get(url).send().await?;
  Ok(response)
}