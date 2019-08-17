use std::collections::hash_map::HashMap;

use mul_model::*;
use rpcx::Client;
use rpcx::RpcxClient;
use rpcx::{CompressType, Result, SerializeType};

pub fn main() {
    let mut c: Client = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();
    c.opt.serialize_type = SerializeType::JSON;
    c.opt.compress_type = CompressType::Gzip;
    let mut a = 1;
    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Mul");
        let metadata = HashMap::new();
        let args = ArithAddArgs { a: a, b: 10 };
        a = a + 1;

        let reply: Option<Result<ArithAddReply>> =
            c.call(&service_path, &service_method, false, &metadata, &args);
        if reply.is_none() {
            continue;
        }

        let result_reply = reply.unwrap();
        match result_reply {
            Ok(r) => println!("received: {:?}", r),
            Err(err) => println!("received err:{}", err),
        }
    }
}
