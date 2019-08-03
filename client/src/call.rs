use std::fmt::Debug;
use std::cell::RefCell;

pub trait Arg: Default + Debug {}

pub trait Reply: Default + Debug {}

impl<T: Default + Debug> Arg for T {}
impl<T: Default + Debug> Reply for T {}

#[derive(Default, Debug)]
pub struct Call<T: Arg, U: Reply> {
    pub service_path: String,
    pub service_method: String,
    pub seq: u64,
    pub args: T,
    pub reply: Option<RefCell<U>>,
    pub error: String,
}

impl<T: Arg, U: Reply> Call<T, U> {
    pub fn new() -> Self {
        Default::default()
    }
}
