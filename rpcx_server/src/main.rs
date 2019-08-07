/// for tech experiment
///
use std::error::Error as StdError;

use rmp_serde as rmps;
use serde::{Deserialize, Serialize};

use rpcx_derive::*;
use rpcx_protocol::{Error, ErrorKind, Result, RpcxParam, SerializeType};
use rpcx_server::*;

#[derive(RpcxParam, Default, Debug, Copy, Clone, Serialize, Deserialize)]
struct ArithAddArgs {
    #[serde(rename = "A")]
    a: u64,
    #[serde(rename = "B")]
    b: u64,
}
#[derive(RpcxParam, Default, Debug, Copy, Clone, Serialize, Deserialize)]
struct ArithAddReply {
    #[serde(rename = "C")]
    c: u64,
}

fn test(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a + args.b }
}

fn main() {
    let mut rpc_server = Server::new("0.0.0.0".to_owned());
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
    let reply = f(s.as_ref(),SerializeType::JSON).unwrap();
    println!("reply:{}", String::from_utf8(reply).unwrap());
}
