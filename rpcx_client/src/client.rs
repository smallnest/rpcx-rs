use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io::{BufReader, BufWriter, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SendError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use futures::future::*;

use super::RpcxClient;
use rpcx_protocol::call::*;
use rpcx_protocol::*;

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
    addr: String,
    stream: Option<TcpStream>,
    seq: Arc<AtomicU64>,
    chan_sender: Sender<RpcData>,
    chan_receiver: Arc<Mutex<Receiver<RpcData>>>,
    calls: Arc<Mutex<HashMap<u64, ArcCall>>>,
}

impl Client {
    pub fn new(addr: &str) -> Client {
        let (sender, receiver) = mpsc::channel();

        Client {
            opt: Default::default(),
            addr: String::from(addr),
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
            stream = TcpStream::connect(self.addr.as_str())?;
        } else {
            let socket_addr: SocketAddr = self
                .addr 
                .parse()
                .map_err(|err| Error::new(ErrorKind::Network, err))?;
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
                    Ok(()) => match calls.lock().unwrap().remove(&msg.get_seq()) {
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
                        Self::drain_calls(calls, error);
                        read_stream.shutdown(Shutdown::Both).unwrap();
                        return;
                    }
                }
            }
        });

        let chan_receiver = self.chan_receiver.clone();
        let send_calls = self.calls.clone();
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
                                Self::drain_calls(send_calls.clone(), error);
                                write_stream.shutdown(Shutdown::Both).unwrap();
                                return;
                            }
                        }

                        match writer.flush() {
                            Ok(()) => {
                                println!("flushed");
                            }
                            Err(error) => {
                                println!("failed to flush: {}", error.to_string());
                                Self::drain_calls(send_calls.clone(), error);
                                write_stream.shutdown(Shutdown::Both).unwrap();
                                return;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
    pub fn send(
        &self,
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

        let mut call_future = CallFuture::new(None);
        if !is_oneway && !is_heartbeat {
            let callback = Call::new(seq);
            let arc_call = Arc::new(Mutex::new(RefCell::from(callback)));
            self.calls
                .clone()
                .lock()
                .unwrap()
                .insert(seq, arc_call.clone());

            call_future = CallFuture::new(Some(arc_call));
        }

        let send_data = RpcData {
            seq: seq,
            data: data,
        };
        match self.chan_sender.clone().send(send_data) {
            Ok(_) => {}
            Err(err) => self.remove_call_with_senderr(err),
        }

        call_future
    }

    fn remove_call_with_senderr(&self, err: SendError<RpcData>) {
        let seq = err.0.seq;
        let calls = self.calls.clone();
        let mut m = calls.lock().unwrap();
        match m.remove(&seq) {
            Some(call) => {
                let internal_call_cloned = call.clone();
                let mut internal_call_mutex = internal_call_cloned.lock().unwrap();
                let internal_call = internal_call_mutex.get_mut();
                internal_call.error = String::from(err.description());
                let mut status = internal_call.state.lock().unwrap();
                status.ready = true;
                if let Some(ref task) = status.task {
                    task.notify()
                }
            }
            None => {}
        }
    }

    fn drain_calls<T: StdError>(calls: Arc<Mutex<HashMap<u64, ArcCall>>>, err: T) {
        let mut m = calls.lock().unwrap();
        for (_, call) in m.drain().take(1) {
            let internal_call_cloned = call.clone();
            let mut internal_call_mutex = internal_call_cloned.lock().unwrap();
            let internal_call = internal_call_mutex.get_mut();
            internal_call.error = String::from(err.description());
            let mut status = internal_call.state.lock().unwrap();
            status.ready = true;
            if let Some(ref task) = status.task {
                task.notify()
            }
        }
    }

    #[allow(dead_code)]
    fn remove_call_with_err<T: StdError>(&mut self, seq: u64, err: T) {
        let calls = self.calls.clone();
        let m = calls.lock().unwrap();
        match m.get(&seq) {
            Some(call) => {
                let internal_call_cloned = call.clone();
                let mut internal_call_mutex = internal_call_cloned.lock().unwrap();
                let internal_call = internal_call_mutex.get_mut();
                internal_call.error = String::from(err.description());
                let mut status = internal_call.state.lock().unwrap();
                status.ready = true;
                if let Some(ref task) = status.task {
                    task.notify()
                }
            }
            None => {}
        }
    }
}

impl RpcxClient for Client {
    fn call<T>(
        &mut self,
        service_path: String,
        service_method: String, 
        is_oneway: bool,
        metadata: Metadata,
        args: &dyn RpcxParam,
    ) -> Option<Result<T>>
    where
        T: RpcxParam + Default,
    {
        let f = self.send(
            service_path,
            service_method,
            is_oneway,
            false,
            metadata,
            args,
        );

        if is_oneway {
            return None;
        }

        let arc_call = f.wait().unwrap();
        let arc_call_1 = arc_call.unwrap().clone();
        let mut arc_call_2 = arc_call_1.lock().unwrap();
        let arc_call_3 = arc_call_2.get_mut();
        let reply_data = &arc_call_3.reply_data;

        if arc_call_3.error.len() > 0 {
            let err = &arc_call_3.error;
            return Some(Err(Error::from(String::from(err))));
        }

        let mut reply: T = Default::default();
        match reply.from_slice(self.opt.serialize_type, &reply_data) {
            Ok(()) => Some(Ok(reply)),
            Err(err) => Some(Err(Error::from(err))),
        }
    }

    fn acall<T>(
        &mut self,
        service_path: String,
        service_method: String,
        metadata: Metadata,
        args: &dyn RpcxParam,
    ) -> Box<dyn Future<Item = Result<T>, Error = Error> + Send + Sync>
    where
        T: RpcxParam + Default,
    {
        let f = self.send(service_path, service_method, false, false, metadata, args);

        let st = self.opt.serialize_type;
        let rt = f
            .map(move |opt_arc_call| {
                let arc_call_1 = opt_arc_call.unwrap().clone();
                let mut arc_call_2 = arc_call_1.lock().unwrap();
                let arc_call_3 = arc_call_2.get_mut();
                let reply_data = &arc_call_3.reply_data;

                if arc_call_3.error.len() > 0 {
                    let err = &arc_call_3.error;
                    return Err(Error::from(String::from(err)));
                }

                let mut reply: T = Default::default();
                match reply.from_slice(st, &reply_data) {
                    Ok(()) => return Ok(reply),
                    Err(err) => return Err(Error::from(err)),
                }
            })
            .map_err(|err| Error::from(err));

        Box::new(rt)
    }
}
