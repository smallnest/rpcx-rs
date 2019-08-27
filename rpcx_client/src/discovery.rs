use super::selector::ClientSelector;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, RwLock},
};

pub trait Discovery<'a> {
    fn get_services(&self) -> HashMap<String, String>;
    fn add_selector(&'a self, s: &'a dyn ClientSelector);
    fn close(&self);
}

#[derive(Default)]
pub struct StaticDiscovery<'a> {
    servers: HashMap<String, String>,
    selectors: Arc<RwLock<Vec<&'a dyn ClientSelector>>>,
}

impl<'a> StaticDiscovery<'a> {
    pub fn new() -> StaticDiscovery<'a> {
        StaticDiscovery {
            servers: HashMap::new(),
            selectors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn update_servers(&self, servers: &HashMap<String, String>) {
        let selectors = (*self).selectors.write().unwrap();
        let v = selectors.deref();
        for s in v {
            s.update_server(servers)
        }
    }
}

impl<'a> Discovery<'a> for StaticDiscovery<'a> {
    fn get_services(&self) -> HashMap<String, String> {
        let mut servers = HashMap::new();
        for (k, v) in &self.servers {
            servers.insert(k.clone(), v.clone());
        }
        servers
    }

    fn add_selector(&'a self, s: &'a dyn ClientSelector) {
        let mut selectors = (*self).selectors.write().unwrap();
        selectors.push(s);
    }
    fn close(&self) {}
}
