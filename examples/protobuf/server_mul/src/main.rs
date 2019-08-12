use mul_model_proto::{ProtoArgs, ProtoReply};
use rpcx::*;

fn add(args: ProtoArgs) -> ProtoReply {
    let mut rt: ProtoReply = Default::default();
    rt.set_C(args.A + args.B);
    rt
}

fn mul(args: ProtoArgs) -> ProtoReply {
    let mut rt: ProtoReply = Default::default();
    rt.set_C(args.A * args.B);
    rt
}

fn main() {
    let mut rpc_server = Server::new("0.0.0.0:8972".to_owned(), 0);
    register_func!(rpc_server, "Arith", "Add", add, ProtoArgs, ProtoReply);

    register_func!(rpc_server, "Arith", "Mul", mul, ProtoArgs, ProtoReply);

    rpc_server.start().unwrap();
}
