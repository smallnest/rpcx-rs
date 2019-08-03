use rpcx_client::Client;

use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::sync::Arc;
use std::thread;
use std::time;

#[allow(unused_imports)]
use rpcx_client::{ArcReply, Arg, Reply};

#[derive(Default, Debug, Copy, Clone)]
struct ArithAddArgs {
    a: u64,
    b: u64,
}

impl Arg for ArithAddArgs {}

#[derive(Default, Debug, Copy, Clone)]
struct ArithAddReply {
    c: u64,
}

impl Reply for ArithAddReply {}

pub fn main() {
    let mut c: Client = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();

    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Add");
        let metadata = HashMap::new();
        let args = ArithAddArgs { a: 10, b: 20 };
        let mut reply: ArithAddReply = Default::default();
        let arcReply: ArcReply = Arc::new(RefCell::new(Box::new(reply)));

        c.send(
            service_path,
            service_method,
            metadata,
            &args,
            Some(arcReply.clone()),
        );

        thread::sleep(time::Duration::from_millis(10 * 1000));
    }
}
