use rpcx_protocol::RpcxParam;
use std::collections::HashMap;
use rand::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::sync::{Arc,RwLock};


pub trait ClientSelector {
    fn select(&mut self, service_path: &String, service_method: &String, args: &dyn RpcxParam) -> String;
    fn update_server(&self,servers: HashMap<String, String>);
}

pub struct RandomSelector {
    pub servers: Arc<RwLock<Vec<String>>>,
    rnd: ThreadRng,
}

impl RandomSelector {
    pub fn new() -> Self {
        RandomSelector{
            servers: Arc::new(RwLock::new(Vec::new())),
            rnd: thread_rng(),
        }
    }
}

impl ClientSelector for RandomSelector{

    fn select(&mut self, service_path: &String, service_method: &String, args: &dyn RpcxParam) -> String {
        let servers = (*self).servers.read().unwrap();
        let size = servers.len();
        let idx = (*self).rnd.gen_range(0, size);
        let s = &servers[idx];
        String::from(s)

    }
    fn update_server(&self,map: HashMap<String, String>) {
        let mut servers = (*self).servers.write().unwrap();
        for k in map.keys() {
            servers.push(String::from(k));
        }
    }
}