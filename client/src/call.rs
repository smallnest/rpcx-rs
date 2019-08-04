use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Result;
use std::sync::Arc;

use rpcx_protocol::SerializeType;

pub trait Arg: Debug {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>>;
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()>;
}

pub trait Reply: Debug {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>>;
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()>;
}

pub type ArcResp = Arc<RefCell<Vec<u8>>>;

#[derive(Debug)]
pub struct Call {
    pub seq: u64,
    pub error: String,
    pub reply: ArcResp,
}

impl Call {
    pub fn new(seq: u64, ar: ArcResp) -> Self {
        Call {
            seq: seq,
            error: String::new(),
            reply: ar,
        }
    }
}

unsafe impl Send for Call{}
unsafe impl Sync for Call{}
