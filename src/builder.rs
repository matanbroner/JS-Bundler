use rslint_parser::{parse_module, SyntaxKind};
use std::fs;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
struct Module {
    file_path: String,
    module_content: String,
    dependencies: Vec<Module>,
}

fn new_module(file_path: &String) -> Module {
    println!("creating module from: {file_path}");
    let contents = fs::read_to_string(file_path).expect("error reading");
    let imports = parse_module_imports(&contents, &file_path);
    let mut mods = Vec::new();
    for import in imports {
        // get dirname + imported file path
        let path = Path::new(file_path).parent().unwrap().join(import);
        match path.to_str() {
            Some(s) => mods.push(new_module(&String::from(s))),
            None => panic!("cannot convert path to string")
        }
    }
    return Module {
        file_path: file_path.to_string(),
        module_content: contents,
        dependencies: mods,
    };
}

fn create_dependency_graph(entry_file: &String) -> Module {
    let root = new_module(&entry_file);
    return root;
}

pub fn build(entry_file: &String, output_folder: &String) -> () {
    let graph = create_dependency_graph(&entry_file);
    println!("{:?}", graph);
    // let output_files = bundle(graph);
    // for out_file in output_files {
    //     // create the full path
    //     let path = Path::new(output_folder).join(out_file.name);
    //     let mut file = fs::File::create(path).expect(format!("error creating {}", path.display()));
    //     file.write_all(out_file.content);
    // }
}

// Helpers

// terribly inefficient way of parsing the
// module for its imports, due to a lack of knowledge using rslist_parse
// TODO: fix
fn parse_module_imports(content: &String, requestor_path: &String) -> Vec<String> {
    let mut sources = Vec::new();
    let parse = parse_module(content, 0);
    let mut syntax_node = parse.syntax().first_child();
    loop {
        let mut _node = syntax_node.unwrap();
        if _node.kind() == SyntaxKind::IMPORT_DECL {
            let mut _import_node = _node.first_child();
            'import: loop {
                while let Some(_in) = _import_node {
                    if _in.kind() == SyntaxKind::LITERAL {
                        let src = _in
                            .text()
                            .to_string()
                            .replace(&['\'', '\"', ' ', '\t'][..], "")
                            .to_owned();
                        sources.push(src);
                        break 'import;
                    }
                    _import_node = _in.next_sibling();
                }
            }
        }
        syntax_node = match _node.next_sibling() {
            Some(next) => Some(next),
            _ => break,
        }
    }
    return sources;
}
