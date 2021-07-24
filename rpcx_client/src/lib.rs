pub mod client;
pub mod discovery;
pub mod selector;
pub mod xclient;

pub use client::*;
pub use discovery::*;
pub use selector::*;
pub use xclient::*;

use async_trait::async_trait;

use rpcx_protocol::{CallFuture, Metadata, Result, RpcxParam};

#[async_trait]
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

    fn send<T>(
        &mut self,
        service_method: &str,
        is_oneway: bool,
        metadata: &Metadata,
        args: &dyn RpcxParam,
    ) -> CallFuture
    where
        T: RpcxParam + Default + Sync + Send + 'static;
}
