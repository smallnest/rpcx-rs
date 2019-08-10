use mul_model::{ArithAddArgs, ArithAddReply};
use rpcx::*;

fn add(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a + args.b }
}

fn mul(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a * args.b }
}

fn main() {
    let mut rpc_server = Server::new("0.0.0.0:8972".to_owned(), 0);
    register_func!(rpc_server, "Arith", "Add", add, ArithAddArgs, ArithAddReply);

    register_func!(rpc_server, "Arith", "Mul", mul, ArithAddArgs, ArithAddReply);

    rpc_server.start().unwrap();
}
