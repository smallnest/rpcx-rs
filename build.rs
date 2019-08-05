[package]
name = "rpcx-rs"
version = "0.1.0"
authors = ["The rpcx-rs Project Developers", "smallnest@gmail.com"]
license = "MIT"
readme = "README.md"
repository = "https://github.com/smallnest/rust-rs"
documentation = "https://docs.rs/rust-rs/"
homepage = "https://crates.io/crates/rust-rs"
keywords = ["rpc", "network", "microservice"]
categories = ["network-programming"]
autobenches = true
edition = "2018"

[workspace]
members = [
    "rpcx_protocol",
    "rpcx_derive",
    "rpcx_client",
    "rpcx_server",
    "examples/client_call_mul",
    "examples/client_call_mul_async",
]
 
[profile.release]
debug = true

[profile.bench]
debug = true 


