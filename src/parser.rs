use rslint_parser::{parse_module, SyntaxKind};

// terribly inefficient way of parsing the
// module for its imports, due to a lack of knowledge using rslist_parse
// TODO: fix
pub fn parse_module_imports(content: &str) -> Vec<String> {
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
                        sources.push(_in.text().to_string().replace(&['\'', '\"', ' ', '\t'][..], ""));
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
