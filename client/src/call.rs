use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Result;
use std::sync::Arc;
use std::sync::Mutex;
use futures::{Future,Poll,Async};

use rpcx_protocol::SerializeType;

pub trait Arg: Debug {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>>;
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()>;
}

pub trait Reply: Debug {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>>;
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()>;
}



// impl Future for Call
// {
//     type Item = Call;
//     type Error = String;

//     fn poll(&self) -> Poll<Call, String> {
//         match self.state {
//             0 => Ok(Async::NotReady),
//             1 => Ok(Async::Ready(self)),
//             _ => Err(self.error),
//         }
//     }
// }

pub type ArcCall = Arc<Mutex<RefCell<Call>>>;


#[derive(Debug)]
pub struct Call {
    pub seq: u64,
    pub state: u8,
    pub error: String,
    pub reply_data: Vec<u8>,
}

impl Call {
    pub fn new(seq: u64) -> Self {
        Call {
            seq: seq,
            state:0,
            error: String::new(),
            reply_data: Vec::new(),
        }
    }
}


unsafe impl Send for Call{}
unsafe impl Sync for Call{}