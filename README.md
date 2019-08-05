# rpcx-rs

[![Build Status](https://travis-ci.org/smallnest/rpcx-rs.svg?branch=master)](https://travis-ci.org/smallnest/rpcx-rs)
[![Crate](https://img.shields.io/crates/v/rpcx-rs.svg)](https://crates.io/crates/rpcx-rs)
[![API](https://docs.rs/rpcx-rs/badge.svg)](https://docs.rs/rpcx-rs)

Rust library for [rpcx](https://rpcx.site) rpc/microservice framework.


## Roadmap

###  0.1.x

protocol and client lib.

- [x] Protocol
- [x] Client (call synchronous/asynchronous)
- [x] support JSON and MessagePack

### 0.2.x

server lib. You can register services bu Rust and  they can be invoked by other languages.

- [ ] Service implementation

### 0.3.x

- [ ] Service discovery and service governance: support etcd and consul
- [ ] Plugins
- [ ] Other features like implementation in Go

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
rpcx_protocol = "0.1.0"
rpcx_derive = "0.1.0"
rpcx_client = "0.1.0"
```

## Example

Roadmap only supports client development so you need start a server by [go server implementation](https://github.com/rpcx-ecosystem/rpcx-examples3/tree/master/102basic).

Write a client:

```rust
use rpcx_client::Client;

use std::collections::hash_map::HashMap;
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

        let reply: Option<Result<ArithAddReply>> =
            c.call(service_path, service_method, false, metadata, &args);
        if reply.is_none() {
            continue;
        }

        let result_reply = reply.unwrap();
        match result_reply {
            Ok(r) => println!("received: {:?}", r),
            Err(err) => println!("received err:{}", err),
        }
    }
}
```

## License

rpcx-rs is distributed under the terms of both the MIT license.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
