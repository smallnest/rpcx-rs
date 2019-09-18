pub mod client;
pub mod discovery;
pub mod selector;
pub mod xclient;

pub use client::*;
pub use discovery::*;
pub use selector::*;
pub use xclient::*;

use futures::Future;
use rpcx_protocol::{Error, Metadata, Result, RpcxParam};

pub trait RpcxClient {
    fn call<T>(
        &mut self,
        service_method: &str,
        is_oneway: bool,
        metadata: &Metadata,
        args: &dyn RpcxParam,
    ) -> Option<Result<T>>
    where
        T: RpcxParam + Default;

    fn acall<T>(
        &mut self,
        service_method: &str,
        metadata: &Metadata,
        args: &dyn RpcxParam,
    ) -> Box<dyn Future<Item = Result<T>, Error = Error> + Send + Sync>
    where
        T: RpcxParam + Default + Sync + Send + 'static;
}
