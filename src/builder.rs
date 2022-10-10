use path_absolutize::*;
use rslint_parser::{parse_module, parse_module_lossy, SyntaxKind, SyntaxNode};
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
    let mut modules = collect_modules(graph);
    let module_map = to_module_map(&mut modules);
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

fn to_module_map(modules: &mut Vec<Module>) -> String {
    let mut module_map = String::from("{");
    for module in modules.iter_mut() {
        transform_module_interface(module);
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
    let mut _iter = |_node: &SyntaxNode| -> bool {
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
        return true;
    };
    parse_iterate_module(content, &mut _iter);
    return sources;
}

// converts commonjs imports to require, exports to module exports
fn transform_module_interface(module: &mut Module) {
    // need to copy since closure changes module
    let mod_copy = copy_module(&module);
    let mut _iter = |_node: &SyntaxNode| -> bool {
        if _node.kind() == SyntaxKind::IMPORT_DECL {
            let mut _import_node = _node.first_child();
            'import: loop {
                while let Some(_in) = _import_node {
                    match _in.kind() {
                        SyntaxKind::LITERAL => {
                            // import a from "b"
                            // straight from name to literal, no need to change
                            let src = _in
                                .text()
                                .to_string()
                                .replace(&['\'', '\"', ' ', '\t'][..], "")
                                .to_owned();
                            let abs_path = Path::new(&module.file_path)
                                .parent()
                                .unwrap()
                                .join(src)
                                .absolutize()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string();
                            let var_name = _in.prev_sibling().unwrap().text().to_string();
                            let new_stmt =
                                format!("const {{default: {}}} = require(\"{}\");", var_name, abs_path);
                            module.module_content = module
                                .module_content
                                .replace(&_node.text().to_string(), &new_stmt);
                            break 'import;
                        }
                        // import a, {b, c} from "d"
                        // or
                        // import {b, c} from "d"
                        SyntaxKind::NAMED_IMPORTS => {
                            let mut vars = _in.text().to_string().replace(&['{', '}'][..], "");
                            if let Some(v) = _in.prev_sibling() {
                                let mut new_var = String::from("default: ");
                                new_var.push_str(&v.text().to_string());
                                new_var.push_str(&format!(",{}", vars));
                                vars = new_var;
                            }
                            let src = _in
                                .next_sibling()
                                .unwrap()
                                .text()
                                .to_string()
                                .replace(&['\'', '\"', ' ', '\t'][..], "")
                                .to_owned();
                            let abs_path = Path::new(&module.file_path)
                                .parent()
                                .unwrap()
                                .join(src)
                                .absolutize()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string();
                            let new_stmt =
                                format!("const {{{}}} = require(\"{}\");", vars, abs_path);
                            module.module_content = module
                                .module_content
                                .replace(&_node.text().to_string(), &new_stmt);
                            break 'import;
                        }
                        _ => _import_node = _in.next_sibling(),
                    }
                }
                break 'import;
            }
        } else if _node.kind() == SyntaxKind::EXPORT_DECL {
            // export { name }
            // or
            // export const name = obj;
            let mut _export_node = _node.first_child();
            'export: loop {
                while let Some(_en) = _export_node {
                    match _en.kind() {
                        SyntaxKind::EXPORT_NAMED => {
                            // export { name }
                            let vars_str = _en.text().to_string().replace(&['{', '}'][..], "");
                            let vars = vars_str.split(",");
                            let mut new_stmt = String::from("");
                            for var in vars {
                                let var_trim = var.replace(&[' ', ';'][..], "");
                                new_stmt.push_str(
                                    &format!("exports.{} = {};\n", var_trim, var_trim)[..],
                                );
                            }
                            module.module_content = module
                                .module_content
                                .replace(&_node.text().to_string(), &new_stmt);
                            break 'export;
                        }
                        SyntaxKind::VAR_DECL => {
                            // export const name = obj;
                            let mut new_stmt = String::from("");
                            let stmt = _en
                                .text()
                                .to_string()
                                .replace(&[' ', ';'][..], "")
                                .replace("let", "")
                                .replace("const", "")
                                .replace("var", "");
                            let decls = stmt.split(",");
                            for decl in decls {
                                let mut decl_split = decl.split("=");
                                let (name, value) =
                                    (decl_split.next().unwrap(), decl_split.next().unwrap());
                                new_stmt.push_str(&format!("exports.{} = {};\n", name, value)[..]);
                            }
                            module.module_content = module
                                .module_content
                                .replace(&_node.text().to_string(), &new_stmt);
                            break 'export;
                        }
                        _ => _export_node = _en.next_sibling(),
                    }
                }
                break 'export;
            }
        } else if _node.kind() == SyntaxKind::EXPORT_DEFAULT_EXPR {
            // export default name
            let var = _node.first_child().unwrap().text();
            let new_stmt = String::from(format!("exports.default = {};\n", var));
            module.module_content = module
                .module_content
                .replace(&_node.text().to_string(), &new_stmt);
        } else {
            // println!("{:?}", _node);
        }
        return true;
    };
    parse_iterate_module(&mod_copy.module_content.to_string(), &mut _iter);
}

fn parse_iterate_module<F: FnMut(&SyntaxNode) -> bool>(content: &String, cb: &mut F) -> () {
    let parse = parse_module(content, 0);
    let mut syntax_node = parse.syntax().first_child();
    loop {
        let mut _node = syntax_node.unwrap();
        let cont = cb(&_node);
        if !cont {
            break;
        }
        syntax_node = match _node.next_sibling() {
            Some(next) => Some(next),
            _ => break,
        }
    }
}
