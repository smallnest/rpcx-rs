use rpcx_client::Client;

use std::thread;
use std::time;

pub fn main() {
    let mut c = Client::new("127.0.0.1:8972");
    c.start().map_err(|err| println!("{}", err)).unwrap();

    thread::sleep(time::Duration::from_millis(60 * 1000));
}
