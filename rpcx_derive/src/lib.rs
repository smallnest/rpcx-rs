extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use rmp_serde as rmps;
use rmp_serde::decode::*;
use rmp_serde::encode::*;
use serde::{Deserialize, Serialize};

use rpcx_protocol::call::*;

#[proc_macro_derive(RpcxParam)]
pub fn rpcx_param(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl RpcxParam for #name {
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
                        let param: Self = serde_json::from_slice(data)?;
                        *self = param;
                        Ok(())
                    }
                    SerializeType::MsgPack => {
                        let param: Self = rmps::from_slice(data)
                            .map_err(|err| Error::new(ErrorKind::Other, err.description()))?;
                        *self = param;
                        Ok(())
                    }
                    _ => Err(Error::new(ErrorKind::Other, "unknown format")),
                }
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
