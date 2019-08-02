use rpcx_client::Client;

use std::thread;
use std::time;
use std::collections::hash_map::HashMap;

#[allow(unused_imports)]
use rpcx_client::{Arg,Reply};

#[derive(Default,Debug,Copy,Clone)]
struct ArithAddArgs {
    a: u64,
    b: u64,
}

#[derive(Default,Debug,Copy,Clone)]
struct ArithAddReply {
    c: u64,
}

pub fn main() {
    let mut c: Client = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();

    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Add");
        let metadata = HashMap::new();
        let args = ArithAddArgs{a:10,b:20};
        let reply: ArithAddReply = Default::default();

        c.send(service_path, service_method, metadata, args, reply);

        thread::sleep(time::Duration::from_millis(10 * 1000));
    }
}
