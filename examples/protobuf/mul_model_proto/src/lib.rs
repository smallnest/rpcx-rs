pub mod arith;

pub use arith::*;

use protobuf::Message;
use rpcx::{Error, ErrorKind, Result, RpcxParam, SerializeType};

impl RpcxParam for ProtoArgs {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>> {
        match st {
            SerializeType::Protobuf => self
                .write_to_bytes()
                .map_err(|err| Error::new(ErrorKind::Serialization, err)),
            _ => Err(Error::new(ErrorKind::Other, "unknown format")),
        }
    }
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()> {
        match st {
            SerializeType::Protobuf => self
                .merge_from_bytes(data)
                .map_err(|err| Error::new(ErrorKind::Serialization, err)),
            _ => Err(Error::new(ErrorKind::Other, "unknown format")),
        }
    }
}

impl RpcxParam for ProtoReply {
    fn into_bytes(&self, st: SerializeType) -> Result<Vec<u8>> {
        match st {
            SerializeType::Protobuf => self
                .write_to_bytes()
                .map_err(|err| Error::new(ErrorKind::Serialization, err)),
            _ => Err(Error::new(ErrorKind::Other, "unknown format")),
        }
    }
    fn from_slice(&mut self, st: SerializeType, data: &[u8]) -> Result<()> {
        match st {
            SerializeType::Protobuf => self
                .merge_from_bytes(data)
                .map_err(|err| Error::new(ErrorKind::Serialization, err)),
            _ => Err(Error::new(ErrorKind::Other, "unknown format")),
        }
    }
}
