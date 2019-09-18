#[cfg(test)]
mod tests {
    use mul_model::{ArithAddArgs, ArithAddReply};
    use rpcx::*;

    use std::{
        collections::HashMap,
        net::{SocketAddr, TcpListener},
        os::unix::io::AsRawFd,
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
        // setup server
        let mut rpc_server = Server::new("0.0.0.0:8972".to_owned(), 0);
        register_func!(
            rpc_server,
            "Arith",
            "Add",
            add,
            "weight=10".to_owned(),
            ArithAddArgs,
            ArithAddReply
        );
        register_func!(
            rpc_server,
            "Arith",
            "Mul",
            mul,
            "weight=10".to_owned(),
            ArithAddArgs,
            ArithAddReply
        );

        let addr = rpc_server
            .addr
            .parse::<SocketAddr>()
            .map_err(|err| Error::new(ErrorKind::Other, err))
            .unwrap();

        let listener = TcpListener::bind(&addr).unwrap();
        let raw_fd = listener.as_raw_fd();
        let handler = thread::spawn(move || match rpc_server.start_with_listener(listener) {
            Ok(()) => {}
            Err(err) => println!("{}", err),
        });

        // setup client

        // use static server
        let mut servers = HashMap::new();
        servers.insert("tcp@127.0.0.1:8972".to_owned(), "weight=10".to_owned());
        let selector = WeightedSelector::new();

        // set discovery with static peers
        let disc = StaticDiscovery::new();
        disc.add_selector(&selector);
        disc.update_servers(&servers);

        // init xclient
        let mut opt: Opt = Default::default();
        opt.serialize_type = SerializeType::JSON;
        opt.compress_type = CompressType::Gzip;
        let mut xc = XClient::new(
            String::from("Arith"),
            FailMode::Failfast,
            Box::new(selector),
            opt,
        );

        let mut a = 1;
        for _ in 0..10 {
            let service_method = String::from("Mul");
            let metadata = HashMap::new();
            let args = ArithAddArgs { a, b: 10 };
            a += 1;

            let reply: Option<Result<ArithAddReply>> =
                xc.call(&service_method, false, &metadata, &args);
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
        drop(xc);
        unsafe {
            libc::close(raw_fd);
        }

        let _ = handler.join();
    }
}
