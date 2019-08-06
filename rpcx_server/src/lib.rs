use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use rpcx_protocol::Result;

pub type RpcxFn = fn(&[u8]) -> Result<Vec<u8>>;

pub struct Server {
    pub services: Arc<RwLock<HashMap<String, Box<RpcxFn>>>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
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
}

#[macro_export]
macro_rules! register_func {
    ($rpc_server:expr, $service_path:expr, $service_method:expr, $service_fn:expr, $arg_type:expr, $reply_type:expr) => {{
        let f: RpcxFn = |x| {
            let mut args: ArithAddArgs = Default::default();
            args.from_slice(SerializeType::JSON, x)?;
            let reply: ArithAddReply = $service_fn(args);
            reply.into_bytes(SerializeType::JSON)
        };
        $rpc_server.register_fn($service_path.to_string(), $service_method.to_string(), f);
    }};
}
