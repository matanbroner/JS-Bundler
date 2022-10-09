mod builder;
mod parser;
use rslint_parser::{ast::IfStmt, parse_module, AstNode, SyntaxKind, SyntaxNodeExt};

fn main() {
    println!("Hello, world!");
    builder::build(&String::from("/Users/matanbroner/Documents/Side Projects/js_bundler_rust/bundler/test/shapes/index.js"), &String::from("/Users/matanbroner/Documents/Side Projects/js_bundler_rust/bundler/test"));
}
