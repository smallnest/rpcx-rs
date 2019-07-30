use std::io::{self, BufReader, BufWriter, Read, Result, Write};
use std::net::Shutdown;
use std::net::TcpStream;
use std::process;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time;

use rpcx_protocol::message::{Message, MessageType, Metadata, RpcxMessage};

pub mod call;

use call::*;

/// a direct client to connect rpcx services.
#[derive(Debug)]
pub struct Client<T:Arg, U:Reply> {
    addr: &'static str,
    stream: Option<TcpStream>,
    seq: Arc<AtomicU64>,
    chan_sender: Sender<call::Call<T, U>>,
    chan_receiver: Receiver<call::Call<T, U>>,
}

impl<T: Arg, U: Reply> Client<T, U> {
    pub fn new(addr: &'static str) -> Client<T, U> {
        let (sender, receiver) = mpsc::channel();

        Client {
            addr: addr,
            stream: None,
            seq: Arc::new(AtomicU64::new(1)),
            chan_sender: sender,
            chan_receiver: receiver,
        }
    }
    pub fn start(&mut self) -> Result<()> {
        let stream = TcpStream::connect(self.addr)?;
        let read_stream = stream.try_clone()?;
        let write_stream = stream.try_clone()?;
        self.stream = Some(stream);

        thread::spawn(move || {
            let mut client_buffer = [0u8; 1024];
            let mut reader = BufReader::new(read_stream.try_clone().unwrap());

            let mut stream = read_stream.try_clone().unwrap();

            let mut msg = Message::new();
            msg.decode(&mut stream).unwrap();

            loop {
                match reader.read(&mut client_buffer[0..]) {
                    Ok(n) => {
                        if n == 0 {
                            process::exit(0);
                        } else {
                            io::stdout().write(&client_buffer).unwrap();
                            io::stdout().flush().unwrap();
                        }
                    }
                    Err(error) => {
                        println!("failed to read: {}", error.to_string());
                        read_stream.shutdown(Shutdown::Both).unwrap();
                    }
                }
            }
        });

        thread::spawn(move || {
            let msg_data: [u8; 47] = [
                8, 0, 0, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 31, 0, 0, 0, 5, 65, 114, 105, 116,
                104, 0, 0, 0, 3, 77, 117, 108, 0, 0, 0, 0, 0, 0, 0, 7, 130, 161, 65, 10, 161, 66,
                20,
            ];

            let mut msg = Message::new();
            let mut data = &msg_data[..] as &[u8];
            msg.decode(&mut data).unwrap();

            let mut writer = BufWriter::new(write_stream.try_clone().unwrap());
            loop {
                let req = msg.encode();
                match writer.write_all(req.as_slice()) {
                    Ok(()) => {
                        println!("wrote");
                    }
                    Err(error) => {
                        println!("failed to write: {}", error.to_string());
                        write_stream.shutdown(Shutdown::Both).unwrap();
                    }
                }

                match writer.flush() {
                    Ok(()) => {
                        println!("flushed");
                    }
                    Err(error) => {
                        println!("failed to flush: {}", error.to_string());
                        write_stream.shutdown(Shutdown::Both).unwrap();
                    }
                }

                thread::sleep(time::Duration::from_millis(1000));
            }
        });

        Ok(())
    }

    pub fn async_send(
        &mut self,
        service_path: String,
        service_method: String,
        metadata: Metadata,
        args: T,
        reply: U,
    ) {
        let mut req = Message::new();
        req.set_version(0);
        req.set_message_type(MessageType::Request);
        req.service_path = service_path.clone();
        req.service_method = service_method.clone();
        req.metadata.replace(metadata);

        let mut callback = call::Call::<T, U>::new();
        callback.service_path = service_path.clone();
        callback.service_method = service_method.clone();
        callback.args = args;
        callback.reply = reply;
        callback.seq = self.seq.clone().fetch_add(1, Ordering::SeqCst);

        self.chan_sender.send(callback).unwrap();
        println!("{:?}",self.chan_receiver.recv().unwrap());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
