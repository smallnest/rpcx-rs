use mul_model::{ArithAddArgs, ArithAddReply};
use rpcx_protocol::{RpcxParam, SerializeType};
use rpcx_server::*;

fn add(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a + args.b }
}

fn mul(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a * args.b }
}

fn main() {
    let mut rpc_server = Server::new("127.0.0.1:8972".to_owned());
    register_func!(
        rpc_server,
        "Arith",
        "Add",
        add,
        ArithAddArgs,
        ArithAddReply
    );

    register_func!(
        rpc_server,
        "Arith",
        "Mul",
        mul,
        ArithAddArgs,
        ArithAddReply
    );

    rpc_server.start().unwrap();
}
