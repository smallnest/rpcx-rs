use std::net::TcpStream;

use super::{RpcxFn, Server};

impl Server {
    pub fn add_register_plugin(&mut self, p: &dyn RegisterPlugin) {}
    pub fn add_connect_plugin(&mut self, p: &dyn ConnectPlugin) {}
}

pub trait RegisterPlugin {
    fn register_fn(&mut self, service_path: &str, service_method: &str, f: RpcxFn) -> bool;
}

pub trait ConnectPlugin {
    fn connected(&mut self, conn: &TcpStream) -> bool;
}
