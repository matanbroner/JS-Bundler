mod builder;

fn main() {
    println!("Hello, world!");
    builder::build(&String::from("/Users/matanbroner/Documents/Side Projects/js_bundler_rust/bundler/test/shapes/index.js"), &String::from("/Users/matanbroner/Documents/Side Projects/js_bundler_rust/bundler/test"));
}
