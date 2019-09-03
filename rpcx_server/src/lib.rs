use std::{
    boxed::Box,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use std::net::SocketAddr;

use rpcx_protocol::*;
use std::{
    io::{BufReader, BufWriter, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

use std::{
    os::unix::io::{AsRawFd, RawFd},
    thread,
};

use scoped_threadpool::Pool;

pub mod plugin;
pub use plugin::*;

pub type RpcxFn = fn(&[u8], SerializeType) -> Result<Vec<u8>>;
pub struct Server {
    pub addr: String,
    raw_fd: Option<RawFd>,
    pub services: Arc<RwLock<HashMap<String, Box<RpcxFn>>>>,
    thread_number: u32,
    register_plugins: Arc<RwLock<Vec<Box<dyn RegisterPlugin + Send + Sync>>>>,
}

impl Server {
    pub fn new(s: String, n: u32) -> Self {
        let mut thread_number = n;
        if n == 0 {
            thread_number = num_cpus::get() as u32;
            thread_number *= 2;
        }
        Server {
            addr: s,
            services: Arc::new(RwLock::new(HashMap::new())),
            thread_number,
            register_plugins: Arc::new(RwLock::new(Vec::new())),
            raw_fd: None,
        }
    }

    pub fn register_fn(&mut self, service_path: String, service_method: String, f: RpcxFn) {
        let key = format!("{}.{}", service_path, service_method);
        let services = self.services.clone();
        let mut map = services.write().unwrap();
        map.insert(key, Box::new(f));
    }

    pub fn get_fn(&self, service_path: String, service_method: String) -> Option<RpcxFn> {
        let key = format!("{}.{}", service_path, service_method);
        let map = self.services.read().unwrap();
        let box_fn = map.get(&key)?;
        Some(**box_fn)
    }

    pub fn start_with_listener(&self, listener: TcpListener) -> Result<()> {
        let thread_number = self.thread_number;

        'accept_loop: for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // for p in &mut self.connect_plugins {
                    //     if p.connected(&stream) {
                    //         break 'accept_loop;
                    //     }
                    // }

                    let services_cloned = self.services.clone();
                    thread::spawn(move || {
                        Server::process(thread_number, services_cloned, stream);
                    });
                }
                Err(e) => {
                    println!("Unable to accept: {}", e);
                    return Err(Error::new(ErrorKind::Network, e));
                }
            }
        }

        Ok(())
    }
    pub fn start(&mut self) -> Result<()> {
        let addr = self
            .addr
            .parse::<SocketAddr>()
            .map_err(|err| Error::new(ErrorKind::Other, err))?;

        let listener = TcpListener::bind(&addr)?;
        println!("Listening on: {}", addr);

        self.raw_fd = Some(listener.as_raw_fd());

        self.start_with_listener(listener)
    }

    pub fn close(&self) {
        if let Some(raw_fd) = self.raw_fd {
            unsafe {
                libc::close(raw_fd);
            }
        }
    }
    fn process(
        thread_number: u32,
        service: Arc<RwLock<HashMap<String, Box<RpcxFn>>>>,
        stream: TcpStream,
    ) {
        let services_cloned = service;
        let local_stream = stream.try_clone().unwrap();

        let mut pool = Pool::new(thread_number);
        pool.scoped(|scoped| {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut msg = Message::new();
                match msg.decode(&mut reader) {
                    Ok(()) => {
                        let service_path = &msg.service_path;
                        let service_method = &msg.service_method;
                        let key = format!("{}.{}", service_path, service_method);
                        let map = &services_cloned.read().unwrap();
                        match map.get(&key) {
                            Some(box_fn) => {
                                let f = **box_fn;
                                let local_stream_in_child = local_stream.try_clone().unwrap();

                                scoped.execute(move || {
                                    invoke_fn(local_stream_in_child.try_clone().unwrap(), msg, f)
                                });
                            }
                            None => {
                                let err = format!("service {} not found", key);
                                let reply_msg = msg.get_reply().unwrap();
                                let mut metadata = reply_msg.metadata.borrow_mut();
                                (*metadata).insert(SERVICE_ERROR.to_string(), err);
                                drop(metadata);
                                let data = reply_msg.encode();
                                let mut writer = BufWriter::new(local_stream.try_clone().unwrap());
                                writer.write_all(&data).unwrap();
                                writer.flush().unwrap();
                            }
                        }
                    }
                    Err(error) => {
                        println!("failed to read: {}", error.to_string());
                        match local_stream.shutdown(Shutdown::Both) {
                            Ok(()) => {
                                println!("client {} is closed", local_stream.peer_addr().unwrap())
                            }
                            Err(_) => {
                                println!("client {} is closed", local_stream.peer_addr().unwrap())
                            }
                        }
                        return;
                    }
                }
            }
        });
    }
}

fn invoke_fn(stream: TcpStream, msg: Message, f: RpcxFn) {
    let mut reply_msg = msg.get_reply().unwrap();
    let reply = f(&msg.payload, msg.get_serialize_type().unwrap()).unwrap();
    reply_msg.payload = reply;
    let data = reply_msg.encode();

    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    match writer.write_all(&data) {
        Ok(()) => {}
        Err(_err) => {}
    }
    match writer.flush() {
        Ok(()) => {}
        Err(_err) => {}
    }
}

#[macro_export]
macro_rules! register_func {
    ($rpc_server:expr, $service_path:expr, $service_method:expr, $service_fn:expr, $arg_type:ty, $reply_type:ty) => {{
        let f: RpcxFn = |x, st| {
            // TODO change ProtoArgs to $arg_typ
            let mut args: $arg_type = Default::default();
            args.from_slice(st, x)?;
            let reply: $reply_type = $service_fn(args);
            reply.into_bytes(st)
        };
        $rpc_server.register_fn($service_path.to_string(), $service_method.to_string(), f);
    }};
}
