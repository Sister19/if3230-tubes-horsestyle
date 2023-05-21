use std::{ops::Add, pin::Pin, error::Error};

use futures::Future;
use reqwest::{Client, Response};

use crate::prelude::*;

pub async fn get(address: String, path: String) -> Result<Response, Box<dyn Error>> {
  let client = Client::new();
  let url = format!("{}{}", address, path);
  let response = client.get(url).send().await?;
  Ok(response)
}


pub async fn post(address: &str, path: &str, body: &str) -> Result<Response, Box<dyn Error>> {
  let client = Client::new();
  let url = format!("{}{}", address, path);
  println!("{}", url);
  let response = client.post(&url).body(body.to_owned()).send().await?;
  println!("tes");
  println!("Response: {:?}", response);
  Ok(response)
}

pub async fn post_many(addresses: Vec<String>, path: &str, body: &String) -> Vec<Result<Response, Box<dyn Error>>> {
  let requests = addresses.iter().map(|address| post(address, path, body));
  futures::future::join_all(requests).await
}