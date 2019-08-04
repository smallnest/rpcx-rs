use rpcx_client::Client;

use std::collections::hash_map::HashMap;
use std::error::Error as StdError;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;

use futures::future::*;
use rmp_serde as rmps;
use rmp_serde::decode::*;
use rmp_serde::encode::*;
use serde::{Deserialize, Serialize};

use rpcx_protocol::SerializeType;
use rpcx_protocol::{Arg, Reply};

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
            SerializeType::MsgPack => {
                rmps::to_vec(self).map_err(|err| Error::new(ErrorKind::Other, err.description()))
            }
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
            SerializeType::MsgPack => {
                let arg: ArithAddArgs = rmps::from_slice(data)
                    .map_err(|err| Error::new(ErrorKind::Other, err.description()))?;
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
            SerializeType::MsgPack => {
                rmps::to_vec(self).map_err(|err| Error::new(ErrorKind::Other, err.description()))
            }
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
            SerializeType::JSON => {
                let reply: ArithAddReply = rmps::from_slice(data)
                    .map_err(|err| Error::new(ErrorKind::Other, err.description()))?;
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
    c.opt.serialize_type = SerializeType::MsgPack;

    let mut a = 1;
    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Mul");
        let metadata = HashMap::new();
        let args = ArithAddArgs { a: a, b: 10 };
        a = a + 1;

        let f = c.send(service_path, service_method, false, false, metadata, &args);

        let arc_call = f.wait().unwrap();
        let arc_call_1 = arc_call.unwrap().clone();
        let mut arc_call_2 = arc_call_1.lock().unwrap();
        let arc_call_3 = arc_call_2.get_mut();
        let reply_data = &arc_call_3.reply_data;

        if arc_call_3.error.len() > 0 {
            println!("received err:{}", &arc_call_3.error)
        } else {
            let mut reply: ArithAddReply = Default::default();
            reply.from_slice(SerializeType::JSON, &reply_data).unwrap();
            println!("received: {:?}", &reply);
        }
    }
}
