// use seraphim::generate;

// fn init_logger() {
//     flexi_logger::Logger::with_env()
//         // .format(|record: &flexi_logger::Record| format!("{}", &record.args()))
//         .duplicate_to_stderr(flexi_logger::Duplicate::Debug)
//         .start()
//         .unwrap();
// }

// fn main() {
//     init_logger();
//     let generator = generate::Generator::new(generate::Config::from_args());
//     generator::run()
// }