use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

// Job is a type alias for a trait object that holds the type of closure that
// execute receives.
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        let (sender, receiver) = mpsc::channel();
        
        // Place receiver in an Arc and a Mutex
        // Arc ensures that we can copy receiver so that multiple workers own the receiver safely
        // Mutex will ensure that only one worker gets a job from the receiver at a time.
        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size); // Stores each new Worker in a vector.
        
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        ThreadPool {
            workers,
            sender: Some(sender)
        }
    }
    
    // implement execute method with an interface similar to thread::spawn
    // should take the closure it's given and gives it to an idle thread in
    // the pool to run.
    // define execute method on the ThreadPool to take a closure as a param.
    // The closure is of Type or Kind FnOnce since running a request will only execute
    // the request's closure one time.
    // we also need Send to transfer the closure from one thread to another and
    // we must bound the lifetime as 'static since we do not know how long the thread
    // will take to execute.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Create a new Job instance by passing the closure f to a new instance
        // of Box aliased by job.
        let job = Box::new(f);
        
        // send that job down the sending end of the channel
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop (&mut self) {
        drop(self.sender.take());
        
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// Define worker struct that holds and id of type usize and a JoinHandles<()>
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    // Define a Worker::new fn that takes an id number and returns a Worker
    // instance that holds the id and a thread spawned with an empty closure.
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // We FIRST call lock() on receiver to acquire the mutex and then call unwrap to panic on any errors.
            // Second we call recv to receive Job from the channel.
            match receiver.lock().unwrap().recv() {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
            
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });
        
        Worker {
            id,
            thread: Some(thread)
        }
    }
}

