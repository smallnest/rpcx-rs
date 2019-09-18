use std::collections::hash_map::HashMap;

use mul_model::*;
use rpcx::*;

pub fn main() {
    let mut servers = HashMap::new();
    servers.insert("tcp@127.0.0.1:8972".to_owned(), "".to_owned());
    let selector = RandomSelector::new();

    let disc = StaticDiscovery::new();
    disc.add_selector(&selector);
    disc.update_servers(&servers);

    let mut opt: Opt = Default::default();
    opt.serialize_type = SerializeType::JSON;
    opt.compress_type = CompressType::Gzip;
    let mut xc = XClient::new(
        String::from("Arith"),
        FailMode::Failfast,
        Box::new(selector),
        opt,
    );

    let mut a = 1;
    loop {
        let service_method = String::from("Mul");
        let metadata = HashMap::new();
        let args = ArithAddArgs { a, b: 10 };
        a += 1;

        let reply: Option<Result<ArithAddReply>> =
            xc.call(&service_method, false, &metadata, &args);
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
