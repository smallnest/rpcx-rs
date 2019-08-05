use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Result, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rpcx_protocol::call::*;
use rpcx_protocol::message::*;

#[derive(Debug, Copy, Clone)]
pub struct Opt {
    pub compress_type: CompressType,
    pub serialize_type: SerializeType,
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub nodelay: Option<bool>,
    pub ttl: Option<u32>,
}

impl Default for Opt {
    fn default() -> Self {
        Opt {
            compress_type: CompressType::CompressNone,
            serialize_type: SerializeType::JSON,
            connect_timeout: Default::default(),
            read_timeout: Default::default(),
            write_timeout: Default::default(),
            nodelay: None,
            ttl: None,
        }
    }
}

#[derive(Debug, Default)]
struct RpcData {
    seq: u64,
    data: Vec<u8>,
}

/// a direct client to connect rpcx services.
#[derive(Debug)]
pub struct Client {
    pub opt: Opt,
    addr: &'static str,
    stream: Option<TcpStream>,
    seq: Arc<AtomicU64>,
    chan_sender: Sender<RpcData>,
    chan_receiver: Arc<Mutex<Receiver<RpcData>>>,
    calls: Arc<Mutex<HashMap<u64, ArcCall>>>,
}

impl Client {
    pub fn new(addr: &'static str) -> Client {
        let (sender, receiver) = mpsc::channel();

        Client {
            opt: Default::default(),
            addr: addr,
            stream: None,
            seq: Arc::new(AtomicU64::new(0)),
            chan_sender: sender,
            chan_receiver: Arc::new(Mutex::new(receiver)),
            calls: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn start(&mut self) -> Result<()> {
        let stream: TcpStream;

        if self.opt.connect_timeout.as_millis() == 0 {
            stream = TcpStream::connect(self.addr)?;
        } else {
            let socket_addr: SocketAddr = self
                .addr
                .parse()
                .map_err(|err| Error::new(ErrorKind::Other, err))?;
            stream = TcpStream::connect_timeout(&socket_addr, self.opt.connect_timeout)?;
        }

        if self.opt.read_timeout.as_millis() > 0 {
            stream.set_read_timeout(Some(self.opt.read_timeout))?;
        }
        if self.opt.write_timeout.as_millis() > 0 {
            stream.set_write_timeout(Some(self.opt.read_timeout))?;
        }

        if self.opt.nodelay.is_some() {
            stream.set_nodelay(self.opt.nodelay.unwrap())?;
        }
        if self.opt.ttl.is_some() {
            stream.set_ttl(self.opt.ttl.unwrap())?;
        }
        let read_stream = stream.try_clone()?;
        let write_stream = stream.try_clone()?;
        self.stream = Some(stream);

        let calls = self.calls.clone();
        thread::spawn(move || {
            let mut reader = BufReader::new(read_stream.try_clone().unwrap());

            loop {
                let mut msg = Message::new();
                match msg.decode(&mut reader) {
                    Ok(()) => match calls.lock().unwrap().get(&msg.get_seq()) {
                        Some(call) => {
                            let internal_call_cloned = call.clone();
                            let mut internal_call_mutex = internal_call_cloned.lock().unwrap();
                            let internal_call = internal_call_mutex.get_mut();
                            if let Some(MessageStatusType::Error) = msg.get_message_status_type() {
                                internal_call.error = msg.get_error().unwrap_or("".to_owned());
                            } else {
                                internal_call.reply_data.extend_from_slice(&msg.payload);
                            }

                            let mut status = internal_call.state.lock().unwrap();
                            status.ready = true;
                            if let Some(ref task) = status.task {
                                task.notify()
                            }
                        }
                        None => {}
                    },
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
        is_oneway: bool,
        is_heartbeat: bool,
        metadata: Metadata,
        args: &dyn RpcxParam,
    ) -> CallFuture {
        let seq = self.seq.clone().fetch_add(1, Ordering::SeqCst);

        let mut req = Message::new();
        req.set_version(0);
        req.set_message_type(MessageType::Request);
        req.set_serialize_type(self.opt.serialize_type);
        req.set_compress_type(self.opt.compress_type);
        req.set_seq(seq);
        req.service_path = service_path.clone();
        req.service_method = service_method.clone();
        req.metadata.replace(metadata);
        let payload = args.into_bytes(self.opt.serialize_type).unwrap();
        req.payload = payload;

        let data = req.encode();

        let send_data = RpcData {
            seq: seq,
            data: data,
        };
        self.chan_sender.clone().send(send_data).unwrap();

        if !is_oneway && !is_heartbeat {
            let callback = Call::new(seq);
            let arc_call = Arc::new(Mutex::new(RefCell::from(callback)));
            self.calls
                .clone()
                .lock()
                .unwrap()
                .insert(seq, arc_call.clone());

            return CallFuture::new(Some(arc_call));
        }

        CallFuture::new(None)
    }
}