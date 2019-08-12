
use  protoc_rust::Customize;

fn main() {

    protoc_rust::run(protoc_rust::Args {
	    out_dir: "src/",
	    input: &["protos/arith.proto"],
	    includes: &["protos"],
	    customize: Customize {
            // serde_derive_cfg: None,
            // serde_derive: Some(true),
	      ..Default::default()
	    },
	}).expect("protoc");
  }
 