#![cfg_attr(feature="nightly", feature(alloc_system))]
#[cfg(feature="nightly")]
extern crate alloc_system;
extern crate random;
extern crate tensorflow;

use random::Source;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::result::Result;
use std::path::Path;
use std::process::exit;
use tensorflow::Code;
use tensorflow::Graph;
use tensorflow::ImportGraphDefOptions;
use tensorflow::Session;
use tensorflow::SessionOptions;
use tensorflow::Status;
use tensorflow::StepWithGraph;
use tensorflow::Tensor;

fn main() {
    // Putting the main code in another function serves two purposes:
    // 1. We can use the `?` operator.
    // 2. We can call exit safely, which does not run any destructors.
    exit(match run() {
        Ok(_) => 0,
        Err(e) => {
            println!("{}", e);
            1
        }
    }
}


fn run() -> Result<(), Box<Error>> {
    let filename = "examples/regression-model/model.pb"; // y = w * x + b
    if !Path::new(filename).exists() {
        return Err(Box::new(Status::new_set(Code::NotFound,
                                            &format!("Run 'python regression.py' to generate \
                                                      {} and try again.",
                                                     filename))
            .unwrap()));
    }
    // Load the computation graph defined by regression.py.
    let mut graph = Graph::new();
    let mut proto = Vec::new();
    File::open(filename)?.read_to_end(&mut proto)?;
    graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;
    let mut session = Session::new(&SessionOptions::new(), &graph)?;
    let op_x = graph.operation_by_name_required("x")?;
    let op_y = graph.operation_by_name_required("y")?;
    let op_init = graph.operation_by_name_required("init")?;
    let op_train = graph.operation_by_name_required("train")?;
    let op_w = graph.operation_by_name_required("w")?;
let op_b = graph.operation_by_name_required("b")?;

}