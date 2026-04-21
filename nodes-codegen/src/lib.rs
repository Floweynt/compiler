use tree_sitter_nodes::LANGUAGE;

pub fn parse(src: &str) {
    let mut parser = tree_sitter::Parser::new();
    
    parser
        .set_language(&LANGUAGE.into())
        .expect("Error loading Nodes parser");
    
    parser.set_logger(Some(Box::new(|log_type, str| {

    })));

    parser.parse(src, None);
}

