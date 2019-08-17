use std::collections::hash_map::HashMap;

use mul_model_proto::*;
use rpcx::Client;
use rpcx::RpcxClient;
use rpcx::{Result, SerializeType,CompressType};

pub fn main() {
    let mut c: Client = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();
    c.opt.serialize_type = SerializeType::Protobuf;
    c.opt.compress_type = CompressType::Gzip;

    let mut a = 1;
    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Mul");
        let metadata = HashMap::new();
        let mut args = ProtoArgs::new();
        &args.set_A(a);
        &args.set_B(20);
        a = a + 1;

        let reply: Option<Result<ProtoReply>> =
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
