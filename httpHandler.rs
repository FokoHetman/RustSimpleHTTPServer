use std::{
  net::TcpStream,
  io::{Read, Write},
  str,
};

#[derive(Debug)]
pub struct Response {
  pub header: String,
  pub content: String,
  pub attachment: String,
}

impl Response {
  pub fn new(response: String) -> Response {

    let mut contents = String::from(response.split("\r\nContent-Length:").collect::<Vec<&str>>()[1].split("\r\n\r\n").collect::<Vec<&str>>()[1].to_string());

    let mut head = response.split("Content-Disposition:").collect::<Vec<&str>>()[0].to_string();
    let mut attachments = response.split("filename=\"").collect::<Vec<&str>>()[1].split("\"").collect::<Vec<&str>>()[0].to_string();

    return Response { header: head, content: contents, attachment: attachments };
  }
}

pub fn make_request(host: &str, path: &str) -> Result<String, String> {

  let mut stream = TcpStream::connect(host).map_err(|e| e.to_string())?;


  let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
  println!("Sending Request:\n{}", request);
  stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;

  let mut buffer = Vec::new();
  stream.read_to_end(&mut buffer).map_err(|e| e.to_string())?;


  let response = str::from_utf8(&buffer).map_err(|e| e.to_string())?.to_string();

  Ok(response)
}
