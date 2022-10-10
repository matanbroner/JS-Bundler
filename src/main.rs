mod builder;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || (args.len() == 1 && &args[1] == "-h") {
        println!("usage: bundler [entry point] [output directory]");
        return;
    }
    builder::build(&String::from(&args[1]), &String::from(&args[2]));
}
