use std::{sync::{Arc, mpsc}, thread};

use crate::utils::vixerr::Error;

type TaskFn = Box<dyn FnOnce() + Send + 'static>;

pub struct Serial {
    sender: mpsc::Sender<TaskFn>,
}

impl Serial {
    pub fn new() -> Arc<Serial> {
        let (sender, receiver) = mpsc::channel::<TaskFn>();
        thread::spawn(move || {
            while let Ok(task_fn) = receiver.recv() {
                task_fn();
            }
        });

        Arc::new(Serial { sender })
    }

    pub fn submit<ResultT: Send + 'static>(&self, task_fn: Box<dyn FnOnce() -> Result<ResultT, Error> + Send + 'static>) -> Result<ResultT, Error> {
        let (session_sender, session_receiver) = mpsc::channel::<Result<ResultT, Error>>();
        self.sender.send(Box::new(move || {
            let res = task_fn();
            session_sender.send(res).unwrap();
        })).unwrap();

        session_receiver.recv().unwrap()
    }
}
