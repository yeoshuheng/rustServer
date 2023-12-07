use std::sync::{mpsc, Arc, Mutex};
use std::thread::{JoinHandle, spawn};
use std::fmt;
use std::fmt::{Formatter, Display};

pub struct PoolCreationError;

impl Display for PoolCreationError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {write!(f, "Invalid Number of Workers")}
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id : usize,
    thread : Option<JoinHandle<()>>,
    // Option is used here so that we can move the state value out, instead of keeping the thread as None
    // when the worker's job has ended / during clean-up.
}

impl Worker {
    fn new (id : usize, reciever : Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = spawn(move || loop {
            let msg = reciever.lock()
                .expect("Problem recieving lock").recv();
            match msg {Ok(job) => {println!("Worker {id} recieved and executing job"); job();},
                Err(_) => {println!("Worker {id} has no jobs, disconnecting"); break;}}
        }); Worker {id, thread : Some(thread)}
    }

    // Versus:
    // move || while let Ok(job) = reciever.lock().unwrap().recv().unwrap(); job();
    // More ineffective because while let means lock will be still be held, unlike let, until the job ends.
    // This slows down other workers from recieving the same job.
}

pub struct ThreadPool {
    workers : Vec<Worker>,
    sender : mpsc::Sender<Job>
}

impl ThreadPool {
    pub fn new (size : usize) -> ThreadPool {
        assert!(size > 0); ThreadPool::create_workers(size)}
    
    pub fn build (size : usize) -> Result<ThreadPool, PoolCreationError> {
        if size > 0 {return Err(PoolCreationError);} Ok(ThreadPool::create_workers(size))}

    fn create_workers(size : usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(size);
        let (sender, reciever) = mpsc::channel();
        let reciever = Arc::new(Mutex::new(reciever));
        for id in 0..size {workers.push(Worker::new(id, Arc::clone(&reciever)));}
        ThreadPool{workers, sender}
    } 

    pub fn execute<F>(&self, f : F) where F : FnOnce() + Send + 'static {
        let job = Box::new(f); self.sender.send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {println!("Closing worker {}", worker.id);
            // If thread already None -> thread has already been cleaned up.
            // Else we take ownership and join the thread so that it closes after the run.
            if let Some(thread) = worker.thread.take() {thread.join().unwrap();}}
    }
}
