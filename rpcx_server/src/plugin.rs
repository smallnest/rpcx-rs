use super::{RpcxFn, Server};
#[allow(unused_imports)]
use rpcx_protocol::*;
use std::net::TcpStream;
impl Server {
    pub fn add_register_plugin(&mut self, p: Box<dyn RegisterPlugin + Send + Sync>) {
        let mut plugins = self.register_plugins.write().unwrap();
        plugins.push(p);
    }
    pub fn add_connect_plugin(&mut self, p: Box<dyn ConnectPlugin + Send + Sync>) {
        let mut plugins = self.connect_plugins.write().unwrap();
        plugins.push(p);
    }
}

pub trait RegisterPlugin {
    fn register_fn(
        &mut self,
        service_path: &str,
        service_method: &str,
        meta: String,
        f: RpcxFn,
    ) -> Result<()>;
}

pub trait ConnectPlugin {
    fn connected(&mut self, conn: &TcpStream) -> Result<()>;
}
