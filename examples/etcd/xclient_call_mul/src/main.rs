use etcd::Client as EtcdClient;
use mul_model::*;
use rpcx::*;
use std::collections::hash_map::HashMap;

pub fn main() {
    let selector = RandomSelector::new();
    let etcd_client = EtcdClient::new(&["http://127.0.0.1:2379"], None).unwrap();
    let disc = EtcdDiscovery::new(etcd_client, "/rpcx_test".to_owned(), String::from("Arith"));
    disc.add_selector(&selector);
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
