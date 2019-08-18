use crate::Result;
use futures::task::{current, Task};
use futures::{Async, Future, Poll};
use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

use crate::SerializeType;

use bytes::BytesMut;

pub trait RpcxParam: Debug {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>>;
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()>;
}

impl RpcxParam for BytesMut {
    fn into_bytes(&self, _: SerializeType) -> Result<Vec<u8>> {
        let rt = self.to_vec();
        Ok(rt)
    }
    fn from_slice(&mut self, _: SerializeType, data: &[u8]) -> Result<()> {
        (*self).extend_from_slice(data);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Status {
    pub ready: bool,
    pub task: Option<Task>,
}

#[derive(Debug)]
pub struct Call {
    pub seq: u64,
    pub is_client_error: bool,
    pub state: Arc<Mutex<Status>>,
    pub error: String,
    pub reply_data: Vec<u8>,
}

impl Call {
    pub fn new(seq: u64) -> Self {
        Call {
            seq: seq,
            is_client_error: true,
            state: Arc::new(Mutex::new(Status {
                ready: false,
                task: None,
            })),
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
        let mut arc_call_1 = arc_call.lock().unwrap();
        let state = &arc_call_1.get_mut().state;
        let mut status = state.lock().expect("!lock");
        if status.ready {
            Ok(Async::Ready(Some(arc_call.clone())))
        } else {
            status.task = Some(current());
            Ok(Async::NotReady)
        }
    }
}

unsafe impl Send for Call {}
unsafe impl Sync for Call {}
