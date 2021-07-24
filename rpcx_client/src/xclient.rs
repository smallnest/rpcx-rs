#![allow(non_snake_case)]

use std::collections::HashMap;

use super::selector::ClientSelector;

use super::{
    client::{Client, Opt},
    RpcxClient,
};

use rpcx_protocol::{call::*, CallFuture, Error, ErrorKind, Metadata, Result, RpcxParam};
use std::{
    boxed::Box,
    cell::RefCell,
    sync::{Arc, Mutex, RwLock, RwLockWriteGuard},
};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Copy, Clone, Display, PartialEq, EnumIter, EnumString)]
pub enum FailMode {
    //Failover selects another server automaticaly
    Failover = 0,
    //Failfast returns error immediately
    Failfast = 1,
    //Failtry use current client again
    Failtry = 2,
    //Failbackup select another server if the first server doesn't respon in specified time and
    // use the fast response.
    Failbackup = 3,
}

#[derive(Debug, Copy, Clone, Display, PartialEq, EnumIter, EnumString)]
pub enum SelectMode {
    //RandomSelect is selecting randomly
    RandomSelect = 0,
    //RoundRobin is selecting by round robin
    RoundRobin = 1,
    //WeightedRoundRobin is selecting by weighted round robin
    WeightedRoundRobin = 2,
    //WeightedICMP is selecting by weighted Ping time
    WeightedICMP = 3,
    //ConsistentHash is selecting by hashing
    ConsistentHash = 4,
    //Closest is selecting the closest server
    Closest = 5,
    // SelectByUser is selecting by implementation of users
    SelectByUser = 1000,
}

pub struct XClient<S: ClientSelector> {
    pub opt: Opt,
    service_path: String,
    fail_mode: FailMode,
    clients: Arc<RwLock<HashMap<String, RefCell<Client>>>>,
    selector: Box<S>,
}

unsafe impl<S: ClientSelector> Send for XClient<S> {}
unsafe impl<S: ClientSelector> Sync for XClient<S> {}

impl<S: ClientSelector> XClient<S> {
    pub fn new(service_path: String, fm: FailMode, s: Box<S>, opt: Opt) -> Self {
        XClient {
            service_path,
            fail_mode: fm,
            selector: s,
            clients: Arc::new(RwLock::new(HashMap::new())),
            opt,
        }
    }

    fn get_cached_client<'a>(
        &'a self,
        clients_guard: &'a mut RwLockWriteGuard<HashMap<String, RefCell<Client>>>,
        k: String,
    ) -> Result<&'a mut RefCell<Client>> {
        let client = clients_guard.get_mut(&k);
        if client.is_none() {
            drop(client);
            match clients_guard.get(&k) {
                Some(_) => {}
                None => {
                    let mut items: Vec<&str> = k.split('@').collect();
                    if items.len() == 1 {
                        items.insert(0, "tcp");
                    }
                    let mut created_client = Client::new(&items[1]);
                    created_client.opt = self.opt;
                    match created_client.start() {
                        Ok(_) => {
                            clients_guard.insert(k.clone(), RefCell::new(created_client));
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }

        let client = clients_guard.get_mut(&k);
        match client {
            Some(_) => Ok(client.unwrap()),
            None => Err(Error::from("client still not found".to_owned())),
        }
    }
}

impl<S: ClientSelector> RpcxClient for XClient<S> {
    fn call<T>(
        &mut self,
        service_method: &str,
        is_oneway: bool,
        metadata: &Metadata,
        args: &dyn RpcxParam,
    ) -> Option<Result<T>>
    where
        T: RpcxParam + Default,
    {
        let service_path = self.service_path.as_str();
        // get a key from selector
        let selector = &mut (self.selector);
        let k = selector.select(service_path, service_method, args);
        if k.is_empty() {
            return Some(Err(Error::new(
                ErrorKind::Client,
                "server not found".to_owned(),
            )));
        }

        let mut clients_guard = self.clients.write().unwrap();
        let client = self.get_cached_client(&mut clients_guard, k.clone());
        if let Err(err) = client {
            return Some(Err(Error::new(ErrorKind::Client, err)));
        }
        // invoke this client
        let mut selected_client = client.unwrap().borrow_mut();
        let opt_rt =
            (*selected_client).call::<T>(service_path, service_method, is_oneway, metadata, args);

        if is_oneway {
            return opt_rt;
        }

        let rt = opt_rt.unwrap();

        match rt {
            Err(rt_err) => {
                if rt_err.kind() == ErrorKind::Client {
                    match self.fail_mode {
                        FailMode::Failover => {
                            let mut retry = self.opt.retry;
                            while retry > 0 {
                                retry -= 1;

                                // re-select
                                let mut clients_guard = self.clients.write().unwrap();
                                let client = self.get_cached_client(&mut clients_guard, k.clone());
                                if let Err(err) = client {
                                    return Some(Err(err));
                                }
                                let mut selected_client = client.unwrap().borrow_mut();

                                let opt_rt = (*selected_client).call::<T>(
                                    service_path,
                                    service_method,
                                    is_oneway,
                                    metadata,
                                    args,
                                );
                                let rt = opt_rt.unwrap();
                                if rt.is_ok() {
                                    return Some(rt);
                                }
                                if rt.unwrap_err().kind() == ErrorKind::Client {
                                    continue;
                                }
                            }
                        }
                        FailMode::Failfast => return Some(Err(rt_err)),
                        FailMode::Failtry => {
                            let mut retry = self.opt.retry;
                            while retry > 0 {
                                retry -= 1;
                                let opt_rt = (*selected_client).call::<T>(
                                    service_path,
                                    service_method,
                                    is_oneway,
                                    metadata,
                                    args,
                                );
                                let rt = opt_rt.unwrap();
                                if rt.is_ok() {
                                    return Some(rt);
                                }
                                if rt.unwrap_err().kind() == ErrorKind::Client {
                                    continue;
                                }
                            }
                        }
                        FailMode::Failbackup => {}
                    }
                }

                Some(Err(rt_err))
            }
            Ok(r) => Some(Ok(r)),
        }
    }

    fn send<T>(
        &mut self,
        service_method: &str,
        is_oneway: bool,
        metadata: &Metadata,
        args: &dyn RpcxParam,
    ) -> CallFuture
    where
        T: RpcxParam + Default + Sync + Send + 'static,
    {
        let service_path = self.service_path.as_str();
        // get a key from selector
        let k = self.selector.select(service_path, service_method, args);
        if k.is_empty() {
            let callback = Call::new(0);
            let arc_call = Arc::new(Mutex::new(RefCell::from(callback)));
            let internal_call_cloned = arc_call.clone();
            let mut internal_call_mutex = internal_call_cloned.lock().unwrap();
            let internal_call = internal_call_mutex.get_mut();
            internal_call.error = "server not found".to_owned();
            let mut status = internal_call.state.lock().unwrap();
            status.ready = true;
            if let Some(ref task) = status.task {
                task.clone().wake()
            }

            return CallFuture::new(Some(arc_call));
        }

        let mut clients_guard = self.clients.write().unwrap();
        let client = self.get_cached_client(&mut clients_guard, k.clone());

        if let Err(err) = client {
            let callback = Call::new(0);
            let arc_call = Arc::new(Mutex::new(RefCell::from(callback)));
            let internal_call_cloned = arc_call.clone();
            let mut internal_call_mutex = internal_call_cloned.lock().unwrap();
            let internal_call = internal_call_mutex.get_mut();
            internal_call.error = err.to_string();
            let mut status = internal_call.state.lock().unwrap();
            status.ready = true;
            if let Some(ref task) = status.task {
                task.clone().wake()
            }

            return CallFuture::new(Some(arc_call));
        }

        // invoke this client
        let selected_client = client.unwrap().borrow_mut();

        selected_client.send(
            service_path,
            service_method,
            is_oneway,
            false,
            metadata,
            args,
        )
    }
}
