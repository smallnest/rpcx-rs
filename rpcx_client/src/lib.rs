pub mod client;
pub mod xclient;
pub mod selector;

pub use client::*;
pub use xclient::*;
pub use selector::*;

use futures::Future;
use rpcx_protocol::{Error, Metadata, Result, RpcxParam};

pub trait RpcxClient {
    fn call<T>(
        &mut self,
        service_path: String,
        service_method: String,
        is_oneway: bool,
        metadata: Metadata,
        args: &dyn RpcxParam,
    ) -> Option<Result<T>>
    where
        T: RpcxParam + Default;

    fn acall<T>(
        &mut self,
        service_path: String,
        service_method: String,
        metadata: Metadata,
        args: &dyn RpcxParam,
    ) -> Box<dyn Future<Item = Result<T>, Error = Error> + Send + Sync>
    where
        T: RpcxParam + Default;
}
