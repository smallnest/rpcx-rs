use mul_model::{ArithAddArgs, ArithAddReply};
use rpcx::*;

fn test(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a + args.b }
}

fn main() {
    let mut rpc_server = Server::new("0.0.0.0:0".to_owned(),0);
    register_func!(
        rpc_server,
        "Arith",
        "Add",
        test,
        ArithAddArgs,
        ArithAddReply
    );

    let f = rpc_server
        .get_fn(String::from("Arith"), String::from("Add"))
        .unwrap();
    let s = String::from(r#"{"A":1,"B":2}"#);
    let reply = f(s.as_ref(), SerializeType::JSON).unwrap();
    println!("reply:{}", String::from_utf8(reply).unwrap());
}
