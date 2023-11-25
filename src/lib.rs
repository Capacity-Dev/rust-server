
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Task = Box<dyn FnOnce() + Send + 'static>;
enum Message {
    NewTask(Task),
    Stop,
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
pub struct ThreadPool {
    sender: mpsc::Sender<Message>,
    workers: Vec<Worker>,
}

impl Worker {
    fn new(id: usize, message: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = Some(thread::spawn(move || loop {
            let received = message.lock().unwrap().recv().unwrap();

            match received {
                Message::NewTask(task) => {
                    println!("Thread {} is processing a task...",id);
                    task();
                }
                Message::Stop => {
                    println!("Thread {} is Stopping...",id);
                    break;
                }
            }
        }));
        Worker { id, thread }
    }
}

impl ThreadPool {
    pub fn new(len: usize) -> ThreadPool {
        assert!(len > 0);
        let mut workers = Vec::with_capacity(len);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..len {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }
        ThreadPool { workers, sender }
    }
    pub fn exec<F>(&self, closure: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Message::NewTask(Box::new(closure));
        self.sender.send(task).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Dropping thread pool");
        for _ in &mut self.workers {
            self.sender.send(Message::Stop).unwrap();
        }
        for worker in &mut self.workers {
            println!("Dropping worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

