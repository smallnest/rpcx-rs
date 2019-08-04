use rpcx_client::Client;

use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::sync::Arc;
use std::thread;
use std::time;

use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use rpcx_client::{ArcResp, Arg, Reply};

use rpcx_protocol::{CompressType, SerializeType};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
struct ArithAddArgs {
    #[serde(rename = "A")]
    a: u64,
    #[serde(rename = "B")]
    b: u64,
}

impl Arg for ArithAddArgs {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>> {
        match st {
            SerializeType::JSON => serde_json::to_vec(self).map_err(|err| Error::from(err)),
            _ => Err(Error::new(ErrorKind::Other, "unknown format")),
        }
    }
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()> {
        match st {
            SerializeType::JSON => {
                let arg: ArithAddArgs = serde_json::from_slice(data)?;
                *self = arg;
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::Other, "unknown format")),
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
struct ArithAddReply {
    #[serde(rename = "C")]
    c: u64,
}

impl Reply for ArithAddReply {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>> {
        match st {
            SerializeType::JSON => serde_json::to_vec(self).map_err(|err| Error::from(err)),
            _ => Err(Error::new(ErrorKind::Other, "unknown format")),
        }
    }
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()> {
        match st {
            SerializeType::JSON => {
                let reply: ArithAddReply = serde_json::from_slice(data)?;
                *self = reply;
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::Other, "unknown format")),
        }
    }
}

pub fn main() {
    let mut c: Client = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();

    let mut a = 1;
    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Mul");
        let metadata = HashMap::new();
        let args = ArithAddArgs { a: a, b: 10 };
        a = a + 1;

        let resp_data = Vec::new();
        let arc_resp = Arc::new(RefCell::new(resp_data));

        c.send(
            service_path,
            service_method,
            SerializeType::JSON,
            CompressType::CompressNone,
            metadata,
            &args,
            Some(arc_resp.clone()),
        );

        thread::sleep(time::Duration::from_millis(5 * 1000));

        let mut reply: ArithAddReply = Default::default();
        reply
            .from_slice(SerializeType::JSON(), &arc_resp.borrow())
            .unwrap();
        println!("received: {:?}", &reply);
    }
}
