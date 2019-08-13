use rpcx_protocol::RpcxParam;
use std::collections::HashMap;

pub trait ClientSelector {
    fn select(&self, service_path: &String, service_method: &String, args: &dyn RpcxParam) -> String;
    fn update_server(&self,servers: HashMap<String, String>);
}