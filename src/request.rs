use std::ops::Add;

use reqwest::{Client, Response, Error, Result};

use crate::prelude::*;

pub async fn get(address: String, path: String) -> Result<Response> {
  let client = Client::new();
  let url = address.add(&path);
  let response = client.get(url).send().await?;
  Ok(response)
}

pub async fn post(address: String, path: String, body: String) -> Result<Response> {
  let client = Client::new();
  let url = address.add(&path);
  let response = client.post(url).body(body).send().await?;
  Ok(response)
}