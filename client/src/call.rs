use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;

pub trait Arg: Debug {}

pub trait Reply: Debug {}

pub type ArcReply = Arc<RefCell<Box<dyn Reply>>>;

#[derive(Debug)]
pub struct Call {
    pub seq: u64,
    pub error: String,
    pub reply: ArcReply,
}

impl Call {
    pub fn new(seq: u64, ar: ArcReply) -> Self {
        Call {
            seq: seq,
            error: String::new(),
            reply: ar,
        }
    }
}
