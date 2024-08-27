use std::{error::Error, sync::{Arc, Mutex}};

use tokio::{self, io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpListener, runtime::Runtime};

const LOCALHOST: &str = "127.0.0.1";

pub struct  AsyncSercer {
    ip: String,
    port: u16,
    callback: Arc<Mutex<dyn Fn(Vec<u8>) -> Vec<u8> + Send + Sync + 'static>>,
}

impl AsyncSercer {
    pub fn new(port: u16, callback: Box<dyn Fn(Vec<u8>) -> Vec<u8> + Send + Sync + 'static>) -> Self {
        Self {
            ip: LOCALHOST.to_string(),
            port,
            callback: Arc::new(Mutex::new(callback)),
        }
    }
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let runtime = Runtime::new()?;
        runtime.block_on(async {
            let listener = TcpListener::bind(format!("{}:{}", &self.ip, self.port)).await?;
            loop {
                let (mut socket, addr) = listener.accept().await?;
                let f = Arc::clone(&self.callback);
                println!("new connect: {:?}", addr);
                tokio::spawn(async move {
                    let (reader, mut writer) = socket.split();
                    let mut reader = BufReader::new(reader);
                    loop {
                        let mut msg = String::new();
                        let count = if let Ok(count) = reader.read_line(&mut msg).await {
                            count
                        } else {
                            break;
                        };
                        if count == 0 {
                            println!("{:?}: disconnect", addr);
                            break;
                        }
                        let write_buff = if let Ok(f) = f.lock() {
                            f(msg.into_bytes())
                        } else {
                            Vec::new()
                        }; 
                        if write_buff.len() != 0 {
                            if let Err(e) = writer.write_all(&write_buff).await {
                                println!("write to {:?} error: {:?}", addr, e);
                            }
                        }
                    }
                });
            }
        })
    }
}
