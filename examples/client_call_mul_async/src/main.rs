use std::collections::hash_map::HashMap;

use mul_model::*;
use rpcx::*;

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut c: Client = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();
    c.opt.serialize_type = SerializeType::MsgPack;

    let mut a = 1;
    loop {
        let service_path = String::from("Arith");
        let service_method = String::from("Mul");
        let metadata = HashMap::new();
        let args = ArithAddArgs { a, b: 10 };
        a += 1;

        let resp = c
            .send(
                &service_path,
                &service_method,
                false,
                false,
                &metadata,
                &args,
            )
            .await;

        let reply: Result<ArithAddReply> = get_result(resp, SerializeType::SerializeNone);
        match reply {
            Ok(r) => println!("received: {:?}", r),
            Err(err) => println!("received err:{}", err),
        }
    }
}
