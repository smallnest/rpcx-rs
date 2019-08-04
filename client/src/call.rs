use futures::{Async, Future, Poll};
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::Result;
use std::sync::Arc;
use std::sync::Mutex;

use rpcx_protocol::SerializeType;

pub trait Arg: Debug {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>>;
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()>;
}

pub trait Reply: Debug {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>>;
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()>;
}

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
            state: 0,
            error: String::new(),
            reply_data: Vec::new(),
        }
    }
}

pub type ArcCall = Arc<Mutex<RefCell<Call>>>;

pub struct CallFuture {
    pub arc_call: Option<ArcCall>,
}

impl CallFuture {
    pub fn new(opt: Option<ArcCall>) -> Self {
        CallFuture { arc_call: opt }
    }
}

impl Future for CallFuture {
    type Item = Option<ArcCall>;
    type Error = String;

    fn poll(&mut self) -> Poll<Option<ArcCall>, String> {
        if self.arc_call.is_none() {
            return Ok(Async::Ready(None));
        }

        let arc_call = self.arc_call.as_ref().unwrap().clone();
        loop {
            let state = arc_call.lock().unwrap().get_mut().state;
            match state {
                // 0 => Ok(Async::NotReady),
                0 => {}
                1 => return Ok(Async::Ready(Some(arc_call))),
                _ => return Err(String::from(&arc_call.lock().unwrap().get_mut().error)),
            }
        }
    }
}

unsafe impl Send for Call {}
unsafe impl Sync for Call {}
