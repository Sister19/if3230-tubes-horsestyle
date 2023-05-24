use std::{ops::Add, pin::Pin, error::Error};

use futures::Future;
use reqwest::{Client, Response};

use crate::prelude::*;

pub async fn get(address: String, path: String) -> Result<Response, Box<dyn Error>> {
  let client = Client::new();
  let url = format!("http://{}{}", address, path);
  let response = client.get(url).send().await?;
  Ok(response)
}


pub async fn post(address: &str, path: &str, body: &str) -> Result<Response, Box<dyn Error>> {
  let client = Client::builder().timeout(Duration::from_secs(20)).build().unwrap();
  let url = format!("http://{}{}", address, path);
  // println!("{}", url);
  // println!("{}", body);
  match client.post(&url).body(body.to_owned()).header("content-type", "application/json").send().await {
    Ok(response) => {
      // println!("Response: {:?}", response);
      return Ok(response);
    },
    Err(e) => {
      // println!("Error: {:?}", e);
      return Err(Box::new(e));
    }
  };
}

pub async fn post_many(addresses: Vec<String>, path: &str, body: &String) -> Vec<Result<Response, Box<dyn Error>>> {
  let requests = addresses.iter().map(|address| {
    // println!("{}", body);
    post(address, path, body)
  } );
  futures::future::join_all(requests).await
}