use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use std::net::SocketAddr;
use std::thread;

use rpcx_protocol::call::*;
use rpcx_protocol::*;
use std::io::{BufReader, BufWriter, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

pub type RpcxFn = fn(&[u8], SerializeType) -> Result<Vec<u8>>;

pub struct Server {
    pub addr: String,
    pub services: Arc<RwLock<HashMap<String, Box<RpcxFn>>>>,
}

impl Server {
    pub fn new(s: String) -> Self {
        Server {
            addr: s,
            services: Arc::new(RwLock::new(HashMap::new())),
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

    pub fn start(&self) -> Result<()> {
        let addr = self
            .addr
            .parse::<SocketAddr>()
            .map_err(|err| Error::new(ErrorKind::Other, err))?;

        let listener = TcpListener::bind(&addr)?;
        println!("Listening on: {}", addr);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let response = b"Hello World";
                    Server::process(&self.services, stream);
                }
                Err(e) => {
                    println!("Unable to connect: {}", e);
                }
            }
        }

        Ok(())
    }

    fn process(services: &Arc<RwLock<HashMap<String, Box<RpcxFn>>>>, stream: TcpStream) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let services_cloned = services.clone();
        
        thread::spawn(move || loop {
            let mut msg = Message::new();
            match msg.decode(&mut reader) {
                Ok(()) => {
                    let service_path = &msg.service_path;
                        let service_method = &msg.service_method;
                        let seq =& msg.get_seq();
                        let st = (&msg).get_serialize_type().unwrap();
                        let mut payload = Vec::with_capacity(msg.payload.len());
                        payload.extend_from_slice(&msg.payload);

                        let key = format!("{}.{}", service_path, service_method);

                    thread::spawn(move || {
                        
                        let map = services_cloned.read().unwrap();
                        match map.get(&key) {
                            Some(box_fn) => {
                                let f = **box_fn;
                                let reply = f(&payload,st).unwrap();
                            },
                            None => {

                            },
                        }

                    }); 
                }
                Err(error) => {
                    println!("failed to read: {}", error.to_string());
                    stream.shutdown(Shutdown::Both).unwrap();
                    return;
                }
            }
        });
    }
}

#[macro_export]
macro_rules! register_func {
    ($rpc_server:expr, $service_path:expr, $service_method:expr, $service_fn:expr, $arg_type:expr, $reply_type:expr) => {{
        let f: RpcxFn = |x, st| {
            let mut args: ArithAddArgs = Default::default();
            args.from_slice(st, x)?;
            let reply: ArithAddReply = $service_fn(args);
            reply.into_bytes(st)
        };
        $rpc_server.register_fn($service_path.to_string(), $service_method.to_string(), f);
    }};
}
