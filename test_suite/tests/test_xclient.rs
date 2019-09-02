#[cfg(test)]
mod tests {
    use mul_model::{ArithAddArgs, ArithAddReply};
    use rpcx::*;

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
        crossbeam::scope(|scope| {
            scope.spawn(|| {
                &rpc_server.start().unwrap();
            });
        });

        &rpc_server.close();
    }
}
