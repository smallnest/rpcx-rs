/// for tech experiment
///
use std::error::Error as StdError;

use rmp_serde as rmps;
use serde::{Deserialize, Serialize};

use rpcx_derive::*;
use rpcx_protocol::{Error, ErrorKind, Result, RpcxParam, SerializeType};

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

type RpcxFn = fn(&[u8]) -> Result<Vec<u8>>;

fn test(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a + args.b }
}

fn main() {
    let f: RpcxFn = |x| {
        let mut args: ArithAddArgs = Default::default();
        args.from_slice(SerializeType::JSON, x)?;
        let reply: ArithAddReply = test(args);
        reply.into_bytes(SerializeType::JSON)
    };

    let s = String::from(r#"{"A":1,"B":2}"#);
    let reply = f(s.as_ref()).unwrap();
    println!("reply:{}", String::from_utf8(reply).unwrap());
}

fn fun_test() {
    let y = 2;
    //static dispatch
    fun_test_impl(5, times2);
    fun_test_impl(5, |x| 2 * x);
    fun_test_impl(5, |x| y * x);
    //dynamic dispatch
    fun_test_dyn(5, &times2);
    fun_test_dyn(5, &|x| 2 * x);
    fun_test_dyn(5, &|x| y * x);
    //C-like pointer to function
    fun_test_ptr(5, times2);
    fun_test_ptr(5, |x| 2 * x); //ok: empty capture set
                                // fun_test_ptr(5, |x| y*x); //error: expected fn pointer, found closure
}

fn fun_test_impl(value: i32, f: impl Fn(i32) -> i32) -> i32 {
    println!("{}", f(value));
    value
}
fn fun_test_dyn(value: i32, f: &dyn Fn(i32) -> i32) -> i32 {
    println!("{}", f(value));
    value
}
fn fun_test_ptr(value: i32, f: fn(i32) -> i32) -> i32 {
    println!("{}", f(value));
    value
}

fn times2(value: i32) -> i32 {
    2 * value
}
