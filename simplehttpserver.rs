// FOK's GRILLIN'
// Feel free to edit how server works.
// However edited servers may not be accepted as official, without being monitored.

mod threading;
use threading::ThreadPool;
use std::{
  env,
  net::{TcpListener, TcpStream},
  io,
  fs,
  path::{Path, PathBuf},
  io::{prelude::*, BufReader},
  process::Command,
  thread,
  sync::{mpsc, Arc, Mutex},

  time::Duration,
};

fn main() {
  let args: Vec<_> = env::args().collect();
  let mut ip = String::from("0.0.0.0:");
  if args.len()>1 {
    ip+=&args[1];
  } else {
    ip+="8000";
  }

  let listener = TcpListener::bind(&ip.clone()).unwrap();
  let pool = ThreadPool::new(12);
  println!("Listening on: {}", ip);
  for stream in listener.incoming() {
    let stream = stream.unwrap();
    pool.execute(|| {handle_connection(stream);});
  }

}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
  let buf_reader = BufReader::new(&mut stream);
  let request_line = buf_reader.lines().next().unwrap().unwrap();
  let mut root = "./";
  println!("{:#?}", request_line);

  let htmlreqs: Vec<&str> = ["GET / HTTP/1.1", "GET /sleep HTTP/1.1"].to_vec();
  //let mut all_reqs: Vec<(&str, (&str, &str))> = [("GET / HTTP/1.1", ("HTTP/1.1 200 OK", "templates/index.html"))].to_vec();
  let mut length: usize = 0;
  let mut contents: String = String::new();
  let mut status_line: &str = "";
  let mut filename: &str = "";
  let mut is_file: bool = false;
  let mut openable_file: String = String::new();
  println!("Syncing files...");
  let mut path = Path::new(".");
  let mut all_files: Vec<PathBuf> = list_all_dir(PathBuf::from(root))?;
  let mut files: Vec<PathBuf> = list_dir(PathBuf::from(root))?;
  println!("{:#?}", files);
  let mut attachment=String::new();

  let (status_line, filename) = match &request_line[..] {
    
    "GET / HTTP/1.1" => {
      ("HTTP/1.1 200 OK", "index.html")
    }
    "GET /sleep HTTP/1.1" => {
      thread::sleep(Duration::from_secs(5));
      ("HTTP/1.1 200 OK", "index.html")
    },
    _ => {
      //verify if request_line.replace("GET", "").replace("HTTP/1.1", "") exists as a file. If it does, copy it into stream.
      let mut returned: (&str, &str) = ("HTTP/1.1 404 NOT FOUND", "/404.html");
      for x in all_files.clone() {
        println!("File: {:#?}", x);
        let strx = "".to_owned() + &x.into_os_string().into_string().unwrap().replace("./", "/");
        // println!("{}:{}", strx, "");
        // println!("{strx}, {request_line} {}", request_line.contains(&strx));
        println!("strx: {}\nreqlineFiltered: {}", strx, request_line.replace("GET", "").replace(" HTTP/1.1", "").to_string());
        if request_line.replace("GET ", "").replace(" HTTP/1.1", "").replace("/","") == strx.replace("/","") && &strx != "." {
          //let mut fx = fs::File::open(".".to_owned() + &strx)?;
          //io::copy(&mut fx, &mut stream);
          let mut count = 0;
          for i in strx.chars() {
            if count>0 {
              attachment+=i.to_string().as_str();
            }
            count+=1;
          }

          println!("Handling a file {}", ".".to_owned() + &strx);
          files = list_dir(PathBuf::from(".".to_owned() + &strx))?;
          is_file = !PathBuf::from(".".to_owned() + &strx).is_dir();
          println!("{}", is_file);
          if is_file {
            openable_file = ".".to_owned() + &strx;
          }
          returned = ("HTTP/1.1 200 OK", "index.html");
          break;
        }
      }

//   ("HTTP/1.1 404 NOT FOUND", "templates/404.html");
     returned
    },
  };
  println!("{}", filename);

  let mut response = String::new();

//  println!("onefileissued::{:#?}::{}", files.clone(), files.clone().len());
  if is_file {
    println!("{}", openable_file);
    contents = fs::read_to_string(openable_file).unwrap();

  } else {
    contents = String::from("<html>\n<body>\n  Directory: \n");
    //contents = fs::read_to_string(filename).unwrap();
    contents+=&format!("{}\n", request_line.split(" ").collect::<Vec<&str>>()[1]);
    for i in files.clone() {
      contents+=&("<li><a href=/".to_owned() + &i.display().to_string() + ">" + &i.display().to_string().replace(root, "").replace(&move_dir_back(i.display().to_string().replace(root, "")), "") + "</a>");
//      println!("{}", contents);
    }
    println!("{:#?}", move_dir_back(request_line.split(" ").collect::<Vec<&str>>()[1].to_string()));
    contents+=&("<li><a href=/.".to_owned() + &move_dir_back(request_line.split(" ").collect::<Vec<&str>>()[1].to_string()) + ">..</a>");
    contents+="</body></html>";
  }
  length = contents.len();

  response=format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
  stream.write_all(response.as_bytes()).unwrap();

  //stream.write_all(response.as_bytes()).unwrap();
  Ok(())
}

fn move_dir_back(path: String) -> String {
  let mut result = String::new();

  let splt = path.split("/").collect::<Vec<&str>>();
  for i in 0..splt.len() {
    if i!=splt.len()-1 {

      result.push_str(splt[i]);
      result+="/";
    }
  }
  return result;
}

fn list_dir(path: PathBuf) -> io::Result<Vec<PathBuf>> {
  let mut result: Vec<PathBuf> = [].to_vec();

  if path.is_dir() {
    for entry in fs::read_dir(path.clone())? {
      let mut npath = entry?.path();
//      if npath.is_dir() {
//        result.append(&mut list_dir(npath.clone())?);
//      } else {
        result.push(npath.clone());
//      }
    }
  }
  result.push(path.clone());


  return Ok(result)
}

fn list_all_dir(path: PathBuf) -> io::Result<Vec<PathBuf>> {
  let mut result: Vec<PathBuf> = [].to_vec();

  if path.is_dir() {
    for entry in fs::read_dir(path.clone())? {
      let mut npath = entry?.path();
      if npath.is_dir() {
        result.append(&mut list_all_dir(npath.clone())?);
      } else {
        result.push(npath.clone());
      }
    }
  }
  result.push(path.clone());


  return Ok(result)
}

