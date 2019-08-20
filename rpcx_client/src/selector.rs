use rand::prelude::*;
use rand::Rng;
use rpcx_protocol::{RpcxParam, SerializeType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub trait ClientSelector {
    fn select(
        &mut self,
        service_path: &String,
        service_method: &String,
        args: &dyn RpcxParam,
    ) -> String;
    fn update_server(&self, servers: &HashMap<String, String>);
}

pub struct RandomSelector {
    pub servers: Arc<RwLock<Vec<String>>>,
    rnd: ThreadRng,
}

impl RandomSelector {
    pub fn new() -> Self {
        RandomSelector {
            servers: Arc::new(RwLock::new(Vec::new())),
            rnd: thread_rng(),
        }
    }
}

impl ClientSelector for RandomSelector {
    fn select(
        &mut self,
        _service_path: &String,
        _service_method: &String,
        _args: &dyn RpcxParam,
    ) -> String {
        let servers = (*self).servers.read().unwrap();
        let size = servers.len();
        let idx = (*self).rnd.gen_range(0, size);
        let s = &servers[idx];
        String::from(s)
    }
    fn update_server(&self, map: &HashMap<String, String>) {
        let mut servers = (*self).servers.write().unwrap();
        for k in map.keys() {
            servers.push(String::from(k));
        }
    }
}

pub struct RoundbinSelector {
    pub servers: Arc<RwLock<Vec<String>>>,
    index: usize,
}

impl RoundbinSelector {
    pub fn new() -> Self {
        RoundbinSelector {
            servers: Arc::new(RwLock::new(Vec::new())),
            index: 0,
        }
    }
}

impl ClientSelector for RoundbinSelector {
    fn select(
        &mut self,
        _service_path: &String,
        _service_method: &String,
        _args: &dyn RpcxParam,
    ) -> String {
        let servers = (*self).servers.read().unwrap();
        let size = servers.len();
        self.index = (self.index + 1) % size;
        let s = &servers[self.index];
        String::from(s)
    }
    fn update_server(&self, map: &HashMap<String, String>) {
        let mut servers = (*self).servers.write().unwrap();
        for k in map.keys() {
            servers.push(String::from(k));
        }
    }
}

pub struct ConsistentHashSelector {
    pub servers: Arc<RwLock<Vec<String>>>,
}

impl ConsistentHashSelector {
    pub fn new() -> Self {
        ConsistentHashSelector {
            servers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

fn hash_request(
    data: &mut Vec<u8>,
    service_path: &String,
    service_method: &String,
    args: &dyn RpcxParam,
) {
    data.extend(service_path.clone().into_bytes());
    data.extend(service_method.clone().into_bytes());
    data.extend(args.into_bytes(SerializeType::JSON).unwrap());
}
impl ClientSelector for ConsistentHashSelector {
    fn select(
        &mut self,
        service_path: &String,
        service_method: &String,
        args: &dyn RpcxParam,
    ) -> String {
        let servers = (*self).servers.read().unwrap();
        let size = servers.len();

        // let data = Vec::new(service_path.len() + service_method.len());
        let jh = jumphash::JumpHasher::new();
        let mut data = Vec::new();
        hash_request(&mut data, service_path, service_method, args);
        let index = jh.slot(&data, size as u32);
        let s = &servers[index as usize];
        String::from(s)
    }
    fn update_server(&self, map: &HashMap<String, String>) {
        let mut servers = (*self).servers.write().unwrap();
        for k in map.keys() {
            servers.push(String::from(k));
        }
    }
}
