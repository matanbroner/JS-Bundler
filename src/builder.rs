use path_absolutize::*;
use rslint_parser::{parse_module, SyntaxKind};
use std::fs;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug, Clone)]
struct Module {
    file_path: String,
    module_content: String,
    dependencies: Vec<Module>,
}

fn new_module(file_path: &String) -> Module {
    let abs_path = Path::new(file_path)
    .absolutize()
    .unwrap()
    .to_str()
    .unwrap()
    .to_string();
    println!("creating module from: {abs_path}");
    let contents = fs::read_to_string(file_path).expect("error reading");
    let imports = parse_module_imports(&contents);
    let mut mods = Vec::new();
    for import in imports {
        // get dirname + imported file path
        let path = Path::new(file_path).parent().unwrap().join(import);
        match path.to_str() {
            Some(s) => mods.push(new_module(&String::from(s))),
            None => panic!("cannot convert path to string"),
        }
    }
    return Module {
        file_path: abs_path,
        module_content: contents,
        dependencies: mods,
    };
}

fn copy_module(module: &Module) -> Module {
    let mut dependencies: Vec<Module> = Vec::new();
    for dep in &module.dependencies {
        dependencies.push(copy_module(dep));
    }
    let _module = Module {
        file_path: module.file_path.clone(),
        module_content: module.module_content.clone(),
        dependencies: dependencies,
    };
    return _module;
}

fn create_dependency_graph(entry_file: &String) -> Module {
    let root = new_module(&entry_file);
    return root;
}

fn bundle(graph: Module) -> (String, String) {
    let modules = collect_modules(graph);
    let module_map = to_module_map(&modules);
    let module_code = add_runtime(&module_map, &modules.first().unwrap().file_path);
    return (String::from("bundle.js"), module_code);
}

fn collect_modules(graph: Module) -> Vec<Module> {
    let mut mods: Vec<Module> = Vec::new();
    collect(graph, &mut mods);

    fn collect(module: Module, mods: &mut Vec<Module>) {
        // TODO: how can we avoid this, figure out Rust borrow checker
        // to avoid deep copy of the module
        mods.push(copy_module(&module));
        for dep in module.dependencies {
            collect(dep, mods);
        }
    }
    return mods;
}

fn to_module_map(modules: &Vec<Module>) -> String {
    let mut module_map = String::from("{");
    for module in modules {
        transform_module_interface(&module);
        module_map.push_str(
            &format!(
                "\"{}\": function(exports, require) {{  {} }},",
                module.file_path, module.module_content
            )[..],
        );
    }
    module_map.push_str("}");
    return module_map;
}

fn add_runtime(module_map: &String, entry_point: &String) -> String {
    let runtime = String::from(format!(
        "
    const modules = {};
    const entry = \"{}\";
    function webpackStart({{ modules, entry }}) {{
      const moduleCache = {{}};
      const require = moduleName => {{
        // if in cache, return the cached version
        if (moduleCache[moduleName]) {{
          return moduleCache[moduleName];
        }}
        const exports = {{}};
        // this will prevent infinite \"require\" loop
        // from circular dependencies
        moduleCache[moduleName] = exports;
    
        // \"require\"-ing the module,
        // exported stuff will assigned to \"exports\"
        modules[moduleName](exports, require);
        return moduleCache[moduleName];
      }};
    
      // start the program
      require(entry);
    }}

    webpackStart({{ modules, entry }});
    ",
        module_map, entry_point
    ));
    return runtime;
}

pub fn build(entry_file: &String, output_folder: &String) -> () {
    let graph = create_dependency_graph(&entry_file);
    // println!("{:?}", graph);
    let (file_name, code) = bundle(graph);
    // create the full path
    let path = Path::new(output_folder).join(file_name);
    let mut file = fs::File::create(path).expect("error creating output path");
    file.write_all(code.as_bytes())
        .expect("error writing to output file");
}

// Helpers

// terribly inefficient way of parsing the
// module for its imports, due to a lack of knowledge using rslist_parse
// TODO: fix
fn parse_module_imports(content: &String) -> Vec<String> {
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

fn transform_module_interface(module: &Module) {
    
}
