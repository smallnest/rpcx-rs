use std::collections::HashMap;

use super::client::Client;
use rpcx_protocol::RpcxParam;

pub trait ClientSelector {
    fn select(service_path: String, service_method: String, args: &dyn RpcxParam) -> String;
    fn update_server(servers: HashMap<String, String>);
}

pub trait ServiceDiscovery {
    fn get_services() -> [(String, String)];
    fn close();
}

pub struct XClient<T: ClientSelector> {
    clients: Vec<Client>,
    selector: T,
}
