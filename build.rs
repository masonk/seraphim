extern crate flexi_logger;
extern crate protoc_rust;
#[macro_use]
extern crate log;

fn init_logger() {
    flexi_logger::Logger::with_env()
        // .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
        .o_duplicate_info(true)
        .start()
        .unwrap();
}

fn main() {
    regenerate_rust_gencode();
}

// This doesn't need to be done on every build; only when the example.proto or feature.proto
// changes, which should be quite rare, or when rust-protobuf is upgraded.
// Moreover, running this requires `protoc` in $PATH, which otherwise isn't a dependency
// of this package. So, regeneration is disabled for must builds.
fn regenerate_rust_gencode() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/tictactoe/gen",
        input: &[
            "src/third_party/tensorflow/core/example/example.proto",
            "src/third_party/tensorflow/core/example/feature.proto",
        ],
        customize: protoc_rust::Customize {
            ..Default::default()
        },
        includes: &["src"],
    })
    .expect("protoc");
}
