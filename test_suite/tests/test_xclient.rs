#[cfg(test)]
mod tests {
    use mul_model::{ArithAddArgs, ArithAddReply};
    use rpcx::*;

    use std::{
        collections::HashMap,
        io::{BufReader, BufWriter, Write},
        net::{Shutdown, SocketAddr, TcpListener, TcpStream},
        os::unix::io::{AsRawFd, RawFd},
        thread,
    };

    fn add(args: ArithAddArgs) -> ArithAddReply {
        ArithAddReply { c: args.a + args.b }
    }

    fn mul(args: ArithAddArgs) -> ArithAddReply {
        ArithAddReply { c: args.a * args.b }
    }

    #[test]
    fn test_xclient_and_server() {
        let mut rpc_server = Server::new("0.0.0.0:8972".to_owned(), 0);
        register_func!(rpc_server, "Arith", "Add", add, ArithAddArgs, ArithAddReply);
        register_func!(rpc_server, "Arith", "Mul", mul, ArithAddArgs, ArithAddReply);

        let addr = rpc_server
            .addr
            .parse::<SocketAddr>()
            .map_err(|err| Error::new(ErrorKind::Other, err))
            .unwrap();

        let listener = TcpListener::bind(&addr).unwrap();
        let raw_fd = listener.as_raw_fd();
        let handler = thread::spawn(move || {
            &rpc_server.start_with_listener(listener).unwrap();
        });

        let mut c: Client = Client::new("127.0.0.1:8972");
        c.start().map_err(|err| println!("{}", err)).unwrap();
        c.opt.serialize_type = SerializeType::JSON;
        c.opt.compress_type = CompressType::Gzip;
        let mut a = 1;
        for _ in 0..10 {
            let service_path = String::from("Arith");
            let service_method = String::from("Mul");
            let metadata = HashMap::new();
            let args = ArithAddArgs { a, b: 10 };
            a += 1;

            let reply: Option<Result<ArithAddReply>> =
                c.call(&service_path, &service_method, false, &metadata, &args);
            if reply.is_none() {
                continue;
            }

            let result_reply = reply.unwrap();
            match result_reply {
                Ok(r) => assert!(r.c == (a - 1) * 10),
                Err(err) => assert!(false, err),
            }
        }
        // clean
        unsafe {
            libc::close(raw_fd);
        }

        let _ = handler.join();
    }
}
