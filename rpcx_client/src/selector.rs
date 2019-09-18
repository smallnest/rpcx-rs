use qstring::QString;
use rand::{prelude::*, Rng};
use rpcx_protocol::{RpcxParam, SerializeType};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use weighted_rs::*;

pub trait ClientSelector {
    fn select(&mut self, service_path: &str, service_method: &str, args: &dyn RpcxParam) -> String;
    fn update_server(&self, servers: &HashMap<String, String>);
}

#[derive(Default)]
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

unsafe impl Sync for RandomSelector {}
unsafe impl Send for RandomSelector {}

impl ClientSelector for RandomSelector {
    fn select(
        &mut self,
        _service_path: &str,
        _service_method: &str,
        _args: &dyn RpcxParam,
    ) -> String {
        let servers = (*self).servers.read().unwrap();
        let size = servers.len();
        if size == 0 {
            return String::new();
        }
        let idx = (*self).rnd.gen_range(0, size);
        let s = &servers[idx];
        String::from(s)
    }
    fn update_server(&self, map: &HashMap<String, String>) {
        let mut servers = self.servers.write().unwrap();
        servers.clear();
        for k in map.keys() {
            servers.push(String::from(k));
        }
    }
}

#[derive(Default)]
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

unsafe impl Sync for RoundbinSelector {}
unsafe impl Send for RoundbinSelector {}

impl ClientSelector for RoundbinSelector {
    fn select(
        &mut self,
        _service_path: &str,
        _service_method: &str,
        _args: &dyn RpcxParam,
    ) -> String {
        let servers = (*self).servers.read().unwrap();
        let size = servers.len();
        if size == 0 {
            return String::new();
        }
        self.index = (self.index + 1) % size;
        let s = &servers[self.index];
        String::from(s)
    }
    fn update_server(&self, map: &HashMap<String, String>) {
        let mut servers = self.servers.write().unwrap();
        servers.clear();
        for k in map.keys() {
            servers.push(String::from(k));
        }
    }
}

#[derive(Default)]
pub struct WeightedSelector {
    pub servers: Arc<RwLock<SmoothWeight<String>>>,
}

impl WeightedSelector {
    pub fn new() -> Self {
        WeightedSelector {
            servers: Arc::new(RwLock::new(SmoothWeight::new())),
        }
    }
}

unsafe impl Sync for WeightedSelector {}
unsafe impl Send for WeightedSelector {}

impl ClientSelector for WeightedSelector {
    fn select(
        &mut self,
        _service_path: &str,
        _service_method: &str,
        _args: &dyn RpcxParam,
    ) -> String {
        let mut servers = self.servers.write().unwrap();
        let mut sw = servers.next();
        match &mut sw {
            Some(s) => s.clone(),
            None => String::new(),
        }
    }
    fn update_server(&self, map: &HashMap<String, String>) {
        let mut servers = self.servers.write().unwrap();

        servers.reset();
        for (k, v) in map.iter() {
            let qs = QString::from(v.as_str());
            if let Some(val) = qs.get("weight") {
                if let Ok(w) = val.parse::<isize>() {
                    servers.add(k.clone(), w);
                } else {
                    servers.add(k.clone(), 1);
                }
            } else {
                servers.add(k.clone(), 1);
            }
        }
    }
}

#[derive(Default)]
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
    service_path: &str,
    service_method: &str,
    args: &dyn RpcxParam,
) {
    data.extend(service_path.to_string().into_bytes());
    data.extend(service_method.to_string().into_bytes());
    data.extend(args.into_bytes(SerializeType::JSON).unwrap());
}
impl ClientSelector for ConsistentHashSelector {
    fn select(&mut self, service_path: &str, service_method: &str, args: &dyn RpcxParam) -> String {
        let servers = (*self).servers.read().unwrap();
        let size = servers.len();
        if size == 0 {
            return String::new();
        }
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
