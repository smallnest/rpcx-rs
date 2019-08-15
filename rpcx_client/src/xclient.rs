use std::collections::HashMap;

use super::selector::ClientSelector;

use super::client::{Client, Opt};
use super::RpcxClient;
use futures::future;
use futures::Future;
use rpcx_protocol::{Error, Metadata, Result, RpcxParam};
use std::boxed::Box;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::RwLock;

pub trait ServiceDiscovery {
    fn get_services() -> [(String, String)];
    fn close();
}

pub enum FailMode {
    //Failover selects another server automaticaly
    Failover,
    //Failfast returns error immediately
    Failfast,
    //Failtry use current client again
    Failtry,
    //Failbackup select another server if the first server doesn't respon in specified time and use the fast response.
    Failbackup,
}

pub enum SelectMode {
    //RandomSelect is selecting randomly
    RandomSelect,
    //RoundRobin is selecting by round robin
    RoundRobin,
    //WeightedRoundRobin is selecting by weighted round robin
    WeightedRoundRobin,
    //WeightedICMP is selecting by weighted Ping time
    WeightedICMP,
    //ConsistentHash is selecting by hashing
    ConsistentHash,
    //Closest is selecting the closest server
    Closest,
    // SelectByUser is selecting by implementation of users
    SelectByUser,
}

pub struct XClient<S: ClientSelector> {
    pub opt: Opt,
    fail_mode: FailMode,
    selector_mode: SelectMode,
    clients: Arc<RwLock<HashMap<String, RefCell<Client>>>>,
    selector: S,
}

impl<S: ClientSelector> XClient<S> {
    pub fn new(fm: FailMode, sm: SelectMode, s: S, opt: Opt) -> Self {
        XClient {
            fail_mode: fm,
            selector_mode: sm,
            selector: s,
            clients: Arc::new(RwLock::new(HashMap::new())),
            opt: opt,
        }
    }
}

impl<S: ClientSelector> RpcxClient for XClient<S> {
    fn call<T>(
        &mut self,
        service_path: String,
        service_method: String,
        is_oneway: bool,
        metadata: Metadata,
        args: &dyn RpcxParam,
    ) -> Option<Result<T>>
    where
        T: RpcxParam + Default,
    {
        // get a key from selector
        let selector = &mut (self.selector);
        let k = selector.select(&service_path, &service_method, args);
        if k.is_empty() {
            return Some(Err(Error::from("server not found".to_owned())));
        }

        let mut clients_guard = self.clients.write().unwrap();
        let mut client = clients_guard.get_mut(&k);
        if client.is_none() {
            match clients_guard.get(&k) {
                Some(_) => {}
                None => {
                    let mut items: Vec<&str> = k.split("@").collect();
                    if items.len() == 1 {
                        items.insert(0, "tcp");
                    }
                    let mut created_client = Client::new(&items[1]);
                    created_client.opt = self.opt;
                    match created_client.start() {
                        Ok(_) => {
                            clients_guard.insert(k.clone(), RefCell::new(created_client));
                        }
                        Err(err) => return Some(Err(err)),
                    }
                }
            }
        }

        client = clients_guard.get_mut(&k);
        if client.is_none() {
            return Some(Err(Error::from("client still not found".to_owned())));
        }

        // invoke this client
        let mut selected_client = client.unwrap().borrow_mut();
        let rt =
            (*selected_client).call::<T>(service_path, service_method, is_oneway, metadata, args);

        match &self.fail_mode {
            Failover => {}
            Failfast => {}
            Failtry => {}
            Failbackup => {}
        }

        rt
    }
    fn acall<T>(
        &mut self,
        service_path: String,
        service_method: String,
        metadata: Metadata,
        args: &dyn RpcxParam,
    ) -> Box<dyn Future<Item = Result<T>, Error = Error> + Send + Sync>
    where
        T: RpcxParam + Default + Sync + Send + 'static,
    {
        // get a key from selector
        let k = self.selector.select(&service_path, &service_method, args);
        if k.is_empty() {
            return Box::new(future::err(Error::from("server not found".to_owned())));
        }

        let clients_guard = self.clients.read().unwrap();
        let mut client = clients_guard.get(&k);

        if client.is_none() {
            let mut clients_w_guard = self.clients.write().unwrap();
            match clients_w_guard.get(&k) {
                Some(_) => {}
                None => {
                    let mut items: Vec<&str> = k.split("@").collect();
                    if items.len() == 1 {
                        items.insert(0, "tcp");
                    }
                    let mut created_client = Client::new(&items[1]);
                    match created_client.start() {
                        Ok(_) => {
                            clients_w_guard.insert(k.clone(), RefCell::new(created_client));
                        }
                        Err(err) => return Box::new(future::err(err)),
                    }
                }
            }
        }

        client = clients_guard.get(&k);
        if client.is_none() {
            return Box::new(future::err(Error::from(
                "client still not found".to_owned(),
            )));
        }

        // invoke this client
        let mut selected_client = client.unwrap().borrow_mut();
        let rt = (*selected_client).acall::<T>(service_path, service_method, metadata, args);

        match &self.fail_mode {
            Failover => {}
            Failfast => {}
            Failtry => {}
            Failbackup => {}
        }

        rt
    }
}
