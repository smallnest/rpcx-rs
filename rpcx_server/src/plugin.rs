use super::{RpcxFn, Server};
use etcd::{kv, Client};
#[allow(unused_imports)]
use futures::future::Future;
use hyper::client::HttpConnector;
use rpcx_protocol::*;
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};
use tokio::runtime::Runtime;

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

#[allow(dead_code)]
pub struct EtcdRegister {
    client: Client<HttpConnector>,
    base_path: String,
    service_addr: String,
    services: Arc<RwLock<HashMap<String, String>>>,
    update_interval: Duration,
}

impl EtcdRegister {
    pub fn new(
        client: Client<HttpConnector>,
        base_path: String,
        service_addr: String,
        update_interval: Duration,
    ) -> Self {
        let services = Arc::new(RwLock::new(HashMap::new()));
        let service_cloned = services.clone();
        let etcd_client_cloned = client.clone();
        let base_path_cloned = base_path.clone();
        let service_addr_cloned = service_addr.clone();
        let update_interval_cloned = update_interval;

        thread::spawn(move || loop {
            thread::sleep(update_interval_cloned);
            Self::refresh(
                service_cloned.clone(),
                etcd_client_cloned.clone(),
                base_path_cloned.clone(),
                service_addr_cloned.clone(),
                update_interval_cloned,
            );
        });
        let r = EtcdRegister {
            client,
            base_path,
            service_addr,
            update_interval,
            services: services.clone(),
        };
        r
    }

    fn refresh(
        services_arc: Arc<RwLock<HashMap<String, String>>>,
        etc_client: Client<HttpConnector>,
        base_path: String,
        service_addr: String,
        update_interval: Duration,
    ) {
        let services = services_arc.read().unwrap();
        for (k, v) in services.iter() {
            match Self::refresh_fn(
                &etc_client,
                base_path.clone(),
                service_addr.clone(),
                update_interval,
                k.as_str(),
                v.clone().as_str(),
            ) {
                Ok(_) => {}
                Err(err) => eprintln!("failed to refresh {}. err: {}", k.as_str(), err),
            }
        }
    }

    fn refresh_fn(
        etcd_client: &Client<HttpConnector>,
        base_path: String,
        service_addr: String,
        update_interval: Duration,
        service_path: &str,
        meta: &str,
    ) -> Result<()> {
        let mut key: String = base_path.clone();
        key.push('/');
        key.push_str(service_path);

        // update this node
        // "<base_path>/<service_path>/<service_addr>"
        key.push('/');
        key.push_str(service_addr.as_str());
        let op = kv::set(
            etcd_client,
            key.as_str(),
            meta,
            Some(update_interval.as_secs() * 2),
        );
        match Runtime::new().unwrap().block_on(op) {
            Ok(_) => {}
            Err(err) => match &err[0] {
                etcd::Error::Api(api_err) => {
                    if api_err.error_code != 105 {
                        return Err(Error::from(format!("{:?}", err)));
                    }
                }
                _ => {
                    return Err(Error::from(format!("{:?}", err)));
                }
            },
        }
        Ok(())
    }
}
impl RegisterPlugin for EtcdRegister {
    fn register_fn(&mut self, service_path: &str, _: &str, meta: String, _: RpcxFn) -> Result<()> {
        if self.services.read().unwrap().get(service_path).is_some() {
            return Ok(());
        }
        let mut key: String = self.base_path.clone();
        key.push('/');
        key.push_str(service_path);

        // check base_path existence
        let op = kv::create_dir(
            &self.client,
            self.base_path.as_str(),
            Some(self.update_interval.as_secs()),
        );
        match Runtime::new().unwrap().block_on(op) {
            Ok(_) => {}
            Err(err) => match &err[0] {
                etcd::Error::Api(api_err) => {
                    if api_err.error_code != 105 {
                        return Err(Error::from(format!("{:?}", err)));
                    }
                }
                _ => {
                    return Err(Error::from(format!("{:?}", err)));
                }
            },
        }
        // check service existence
        let op = kv::create_dir(
            &self.client,
            key.as_str(),
            Some(self.update_interval.as_secs()),
        );
        match Runtime::new().unwrap().block_on(op) {
            Ok(_) => {}
            Err(err) => match &err[0] {
                etcd::Error::Api(api_err) => {
                    if api_err.error_code != 105 {
                        return Err(Error::from(format!("{:?}", err)));
                    }
                }
                _ => {
                    return Err(Error::from(format!("{:?}", err)));
                }
            },
        }
        // add this node
        // "<base_path>/<service_path>/<service_addr>"
        key.push('/');
        key.push_str(self.service_addr.as_str());
        let op = kv::set(
            &self.client,
            key.as_str(),
            meta.as_str(),
            Some(self.update_interval.as_secs() * 2),
        );
        match Runtime::new().unwrap().block_on(op) {
            Ok(_) => println!("succeed to register: {}", key.as_str()),
            Err(err) => match &err[0] {
                etcd::Error::Api(api_err) => {
                    if api_err.error_code != 105 {
                        return Err(Error::from(format!(
                            "failed to register:{}, err:{:?}",
                            key.as_str(),
                            err
                        )));
                    }
                }
                _ => {
                    return Err(Error::from(format!(
                        "failed to set:{}, err:{:?}",
                        key.as_str(),
                        err
                    )));
                }
            },
        }

        // record this service
        self.services
            .write()
            .unwrap()
            .insert(service_path.to_owned(), meta);
        Ok(())
    }
}
