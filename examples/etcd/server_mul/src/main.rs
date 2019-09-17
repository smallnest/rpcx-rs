use etcd::Client as EtcdClient;
use mul_model::{ArithAddArgs, ArithAddReply};
use rpcx::*;
use std::time;

fn add(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a + args.b }
}

fn mul(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a * args.b }
}

fn main() {
    // etcd plugin
    let etcd_client = EtcdClient::new(&["http://127.0.0.1:2379"], None).unwrap();
    let p = EtcdRegister::new(
        etcd_client,
        "/rpcx_test".to_owned(),
        "127.0.0.1:8972".to_owned(),
        time::Duration::new(5, 0),
    );

    let mut rpc_server = Server::new("0.0.0.0:8972".to_owned(), 0);
    rpc_server.add_register_plugin(Box::new(p));

    register_func!(
        rpc_server,
        "Arith",
        "Add",
        add,
        "".to_owned(),
        ArithAddArgs,
        ArithAddReply
    );

    register_func!(
        rpc_server,
        "Arith",
        "Mul",
        mul,
        "".to_owned(),
        ArithAddArgs,
        ArithAddReply
    );

    rpc_server.start().unwrap();
}
