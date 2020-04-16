# rpcx-rs

[![Build Status](https://travis-ci.org/smallnest/rpcx-rs.svg?branch=master)](https://travis-ci.org/smallnest/rpcx-rs)
[![Crate](https://img.shields.io/crates/v/rpcx.svg)](https://crates.io/crates/rpcx)
[![API](https://docs.rs/rpcx/badge.svg)](https://docs.rs/rpcx)

Rust library for [rpcx](https://rpcx.site) rpc/microservice framework.

Use the **simplest** style to explore Rust function as cross-platform rpc services.

If you can write Rust functions, you can write rpc services. It is so easy.


see all exampes: [rpcx-rs-examples](https://github.com/smallnest/rpcx-rs/tree/master/examples).

## Roadmap

###  0.1.x

protocol and client/server lib.

- [x] Protocol
- [x] Client (call synchronous/asynchronous)
- [x] support JSON, MessagePack and Protobuf
- [x] Service implementation

### 0.2.x

- [ ] Service discovery
  - [x] static multiple peers 
  - [x] etcd
  - [ ] consul
- [ ] service governance
 - [ ] Select Mode
   - [x] RandomSelect,
   - [x] RoundRobin
   - [x] WeightedRoundRobin
   - [ ] WeightedICMP
   - [x] ConsistentHash
   - [ ] Closest
   - [ ] Custiomized
 - [ ] Faile Mode
   - [x] Failover
   - [x] Failfast
   - [x] Failtry


### 0.3.x

- [ ] plugins
- [ ] document
- [ ] unit tests and integration tests
- [ ] other features like implementation in Go

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
rpcx = "0.2.0"
```

## Example

### Write the Argument and the Reply

First you should write the argument and the reply. They are used by rpc services and clients.

```rust
use std::error::Error as StdError;

use rmp_serde as rmps; 
use serde::{Deserialize, Serialize};

use rpcx_derive::*;
use rpcx_protocol::{Error, ErrorKind, Result, RpcxParam, SerializeType};

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
```

You must add `RpcxParam`、`Serialize`、`Deserialize` and `Default` traits in `derive`. Rpcx can add hepler methods for serialization.

If not, you need to implement `RpcxParam` and `Default` mannually.

Here we defined `ArithAddArgs` as the argument type and `ArithAddReply` as the reply type.

### Implement the server

```rust
use mul_model::{ArithAddArgs, ArithAddReply};
use rpcx::*;

fn add(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a + args.b }
}

fn mul(args: ArithAddArgs) -> ArithAddReply {
    ArithAddReply { c: args.a * args.b }
}

fn main() {
    let mut rpc_server = Server::new("127.0.0.1:8972".to_owned());
    register_func!(
        rpc_server,
        "Arith",
        "Add",
        add,
        ArithAddArgs,
        ArithAddReply
    );

    register_func!(
        rpc_server,
        "Arith",
        "Mul",
        mul,
        ArithAddArgs,
        ArithAddReply
    );

    rpc_server.start().unwrap();
}
```
Here we implement two services: `add` and `mul`. And we use `register_func!` macro to register them with their expored names(`service_path` and `service_method`). Clients can use the name to access them.

### Implement client

Here we use one client to access `Arith.Mul` service in a loop.

```rust
use std::collections::hash_map::HashMap;

use mul_model::*;
use rpcx::Client;
use rpcx::{Result, SerializeType};

pub fn main() {
    let mut c: Client = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();
    c.opt.serialize_type = SerializeType::JSON;

    let mut a = 1;
    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Mul");
        let metadata = HashMap::new();
        let args = ArithAddArgs { a: a, b: 10 };
        a += 1;

        let reply: Option<Result<ArithAddReply>> =
            c.call(service_path, service_method, false, metadata, &args);
        match reply {
            Some(Ok(r)) => println!("received: {:?}", r),
            Some(Err(err)) => println!("received err:{}", err),
            None => {}
        }
    }
}
```

Actually you can use this client to access rpcx services implemented by other program languages such as [service in go](https://github.com/rpcx-ecosystem/rpcx-examples3/tree/master/102basic).


As you see, only after three steps you have expored Rust functions (`add` and `mul`) as rpc services.

You can find more examples at [rpcx-rs/examples](https://github.com/smallnest/rpcx-rs/examples)

## License

rpcx-rs is distributed under the terms of both the MIT license.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
