// FOK's GRILLIN'
// Feel free to edit how server works.
// However edited servers may not be accepted as official, without being monitored.


use std::{
  thread,
  sync::{mpsc, Arc, Mutex},
};


pub struct ThreadPool {
  minions: Vec<Minion>,
  sender: mpsc::Sender<Job>,
}
type Job = Box<dyn FnOnce() + Send + 'static>;


impl ThreadPool {
  pub fn new(size: usize) -> ThreadPool {

    let (sender, receiver) = mpsc::channel();

    let receiver = Arc::new(Mutex::new(receiver));

    let mut minions = Vec::with_capacity(size);

    for id in 0..size {
      minions.push(Minion::new(id, Arc::clone(&receiver)));
    }

    ThreadPool {minions, sender}
  }
//  fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {

//  }
  pub fn execute<F>(&self, f: F)
   where F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f);
    self.sender.send(job).unwrap();
  }
}

struct Minion {
  id: usize,
  thread: thread::JoinHandle<()>,
}

impl Minion {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Minion {
    let thread = thread::spawn(move || loop {
      let job = receiver.lock().unwrap().recv().unwrap();

      println!("Minion {id} is handling new job...");
      job();
    });

    Minion {id, thread}
  }
}
