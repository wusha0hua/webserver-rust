use std::{io::{Error, Read, Write}, net::{TcpListener, TcpStream}, sync::Mutex, thread};
use std::sync::Arc;

use crate::threadpool::ThreadPool;

const LOCALHOST: &str = "127.0.0.1";

pub struct Server {
    ip: String,
    port: u16,
    listener: TcpListener,
    threadpool: ThreadPool,
    callback: Arc<Mutex<dyn Fn(Vec<u8>) -> Vec<u8> + Send + Sync + 'static>>,
}

impl Server {
    pub fn new(port: u16, callback: Box<dyn Fn(Vec<u8>) -> Vec<u8> + Send + Sync + 'static>) -> Result<Self, Error> {
        let size = match thread::available_parallelism() {
            Ok(size) => size.get(),
            Err(_) => 12,
        };
        let listener = TcpListener::bind(format!("{}:{}", LOCALHOST, port))?;
        let callback = Arc::new(Mutex::new(callback));
        let server = Self { 
            ip: LOCALHOST.to_string(), 
            port, 
            listener, 
            threadpool: ThreadPool::new(size), 
            callback,
        };
        Ok(server)
    }
    pub fn start(&mut self) {
        for stream in self.listener.incoming() {
            if let Ok(stream) = stream {
                let callback = self.callback.clone();
                self.threadpool.execute(move || {
                    Self::handle_connect(callback, stream);
                });
            }
        }
    }
    fn handle_connect(f: Arc<Mutex<dyn Fn(Vec<u8>) -> Vec<u8> + Send + Sync + 'static>>, mut stream: TcpStream) {
        loop {
            let mut read_buffer = Vec::new();
            if loop {
                let mut buff  = [0; 1024];
                let count = stream.read(&mut buff).unwrap();
                if count == 1024 {
                    read_buffer.append(&mut buff.into_iter().take(count).collect());
                } else {
                    break count;
                }
            } == 0 {
                break;
            }
            let write_buffer = match f.lock() {
                Ok(f) => f(read_buffer),
                Err(err) => {
                    println!("{:?}", err);
                    Vec::new()
                }
            };
            if write_buffer.len() != 0 {
                stream.write(&write_buffer).unwrap();
            }
        }
    }
}
