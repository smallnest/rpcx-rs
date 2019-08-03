use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Result, Write};
use std::net::Shutdown;
use std::net::TcpStream;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use rpcx_protocol::message::{Message, MessageType, Metadata, RpcxMessage};

pub mod call;
pub use call::*;

#[derive(Debug, Default)]
struct RpcData {
    seq: u64,
    data: Vec<u8>,
}

/// a direct client to connect rpcx services.
#[derive(Debug)]
pub struct Client {
    addr: &'static str,
    stream: Option<TcpStream>,
    seq: Arc<AtomicU64>,
    chan_sender: Sender<RpcData>,
    chan_receiver: Arc<Mutex<Receiver<RpcData>>>,
    calls: Arc<Mutex<HashMap<u64, Call>>>,
}

impl Client {
    pub fn new(addr: &'static str) -> Client {
        let (sender, receiver) = mpsc::channel();

        Client {
            addr: addr,
            stream: None,
            seq: Arc::new(AtomicU64::new(1)),
            chan_sender: sender,
            chan_receiver: Arc::new(Mutex::new(receiver)),
            calls: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn start(&mut self) -> Result<()> {
        let stream = TcpStream::connect(self.addr)?;
        let read_stream = stream.try_clone()?;
        let write_stream = stream.try_clone()?;
        self.stream = Some(stream);

        thread::spawn(move || {
            let mut reader = BufReader::new(read_stream.try_clone().unwrap());

            loop {
                let mut msg = Message::new();
                match msg.decode(&mut reader) {
                    Ok(()) => {
                        println!("{:?}", msg);
                    }
                    Err(error) => {
                        println!("failed to read: {}", error.to_string());
                        read_stream.shutdown(Shutdown::Both).unwrap();
                    }
                }
            }
        });

        let chan_receiver = self.chan_receiver.clone();

        thread::spawn(move || {
            let mut writer = BufWriter::new(write_stream.try_clone().unwrap());
            loop {
                match chan_receiver.lock().unwrap().recv() {
                    Err(error) => {
                        println!("failed to fetch RpcData: {}", error.to_string());
                        write_stream.shutdown(Shutdown::Both).unwrap();
                        return;
                    }
                    Ok(rpcdata) => {
                        println!("{:?}", &rpcdata.data);
                        match writer.write_all(rpcdata.data.as_slice()) {
                            Ok(()) => {
                                println!("wrote");
                            }
                            Err(error) => {
                                println!("failed to write: {}", error.to_string());
                                write_stream.shutdown(Shutdown::Both).unwrap();
                            }
                        }

                        match writer.flush() {
                            Ok(()) => {
                                println!("flushed");
                            }
                            Err(error) => {
                                println!("failed to flush: {}", error.to_string());
                                write_stream.shutdown(Shutdown::Both).unwrap();
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn send(
        &mut self,
        service_path: String,
        service_method: String,
        metadata: Metadata,
        args: &dyn Arg,
        reply: Option<ArcReply>,
    ) {
        let mut req = Message::new();
        req.set_version(0);
        req.set_message_type(MessageType::Request);
        req.service_path = service_path.clone();
        req.service_method = service_method.clone();
        req.metadata.replace(metadata);

        let seq = self.seq.clone().fetch_add(1, Ordering::SeqCst);
        if reply.is_some() {
            let ar = reply.unwrap();
            let callback = call::Call::new(seq, ar.clone());
            self.calls.clone().lock().unwrap().insert(seq, callback);
        }
        let data = vec![
            8, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 39, 0, 0, 0, 5, 65, 114, 105, 116, 104,
            0, 0, 0, 3, 77, 117, 108, 0, 0, 0, 0, 0, 0, 0, 15, 123, 34, 65, 34, 58, 49, 48, 44, 34,
            66, 34, 58, 50, 48, 125,
        ];
        let send_data = RpcData {
            seq: seq,
            data: data,
        };

        self.chan_sender.clone().send(send_data).unwrap();
    }
}
