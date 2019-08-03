use rpcx_client::Client;

use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::io::Error;
use std::io::Result;
use std::sync::Arc;
use std::thread;
use std::time;

use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use rpcx_client::{ArcReply, Arg, Reply};

use rpcx_protocol::SerializeType;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
struct ArithAddArgs {
    #[serde(rename = "A")]
    a: u64,
    #[serde(rename = "B")]
    b: u64,
}

impl Arg for ArithAddArgs {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|err| Error::from(err))
    }
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()> {
        let arg: ArithAddArgs = serde_json::from_slice(data)?;
        *self = arg;
        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
struct ArithAddReply {
    #[serde(rename = "C")]
    c: u64,
}

impl Reply for ArithAddReply {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|err| Error::from(err))
    }
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()> {
        let reply: ArithAddReply = serde_json::from_slice(data)?;
        *self = reply;

        Ok(())
    }
}

pub fn main() {
    let mut c: Client = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();

    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Mul");
        let metadata = HashMap::new();
        let args = ArithAddArgs { a: 10, b: 20 };
        let mut reply: ArithAddReply = Default::default();
        let arcReply: ArcReply = Arc::new(RefCell::new(Box::new(reply)));

        c.send(
            service_path,
            service_method,
            metadata,
            &args,
            Some(arcReply.clone()),
        );

        thread::sleep(time::Duration::from_millis(10 * 1000));
    }
}
