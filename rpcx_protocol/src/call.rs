use crate::Result;

use std::{
    cell::RefCell,
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use crate::SerializeType;

use bytes::BytesMut;

use super::Error;

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
    pub task: Option<Waker>,
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
            seq,
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

pub fn get_result<T>(arc_call: Option<ArcCall>, st: SerializeType) -> Result<T>
where
    T: RpcxParam + Default,
{
    if arc_call.is_none() {
        return Err(Error::from("reply is empty"));
    }
    let arc_call_1 = arc_call.unwrap().clone();
    let mut arc_call_2 = arc_call_1.lock().unwrap();
    let arc_call_3 = arc_call_2.get_mut();
    let reply_data = &arc_call_3.reply_data;
    if !arc_call_3.error.is_empty() {
        let err = &arc_call_3.error;
        return Err(Error::from(String::from(err)));
    }

    let mut reply: T = Default::default();
    match reply.from_slice(st, &reply_data) {
        Ok(()) => Ok(reply),
        Err(err) => Err(err),
    }
}
pub struct CallFuture {
    pub arc_call: Option<ArcCall>,
}

impl CallFuture {
    pub fn new(opt: Option<ArcCall>) -> Self {
        CallFuture { arc_call: opt }
    }
}

impl Future for CallFuture {
    type Output = Option<ArcCall>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.arc_call.is_none() {
            return Poll::Ready(None);
        }

        let arc_call = self.arc_call.as_ref().unwrap().clone();
        let mut arc_call_1 = arc_call.lock().unwrap();
        let state = &arc_call_1.get_mut().state;
        let mut status = state.lock().expect("!lock");
        if status.ready {
            Poll::Ready(Some(arc_call.clone()))
        } else {
            status.task = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

unsafe impl Send for Call {}
unsafe impl Sync for Call {}
