use std::error::Error as StdError;

use rmp_serde as rmps; 
use serde::{Deserialize, Serialize};

use rpcx::*;

#[derive(RpcxParam, Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ArithAddArgs {
    #[serde(rename = "A")]
    pub a: u64,
    #[serde(rename = "B")]
    pub b: u64,
}
#[derive(RpcxParam, Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ArithAddReply {
    #[serde(rename = "C")]
    pub c: u64,
}
