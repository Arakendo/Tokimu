use std::{env, fs, path::PathBuf, process};

use xml_tools::{
    parse_xml_document, ExpandedName, XmlDiagnostic, XmlDocument, XmlDocumentId, XmlNodeId,
    XmlNodeKind, XmlOptions, XmlSourceId,
};

const SOURCE: XmlSourceId = XmlSourceId::new(1);
const DOCUMENT: XmlDocumentId = XmlDocumentId::new(1);

fn main() {
    if let Err(error) = run() {
        eprintln!("hello-xml-inspect: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let path = input_path()?;
    let input =
        fs::read_to_string(&path).map_err(|error| format!("read '{}': {error}", path.display()))?;
    let document = parse_xml_document(DOCUMENT, SOURCE, &input, XmlOptions::default())
        .map_err(format_diagnostic)?;

    println!("source: {}", path.display());
    println!("bytes: {}", input.len());
    println!("roots: {}", document.roots().len());
    println!();
    for root in document.roots() {
        print_node(&document, *root, 0)?;
    }
    Ok(())
}

fn input_path() -> Result<PathBuf, String> {
    let mut args = env::args_os();
    let _program = args.next();
    let path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/sample.xml"));
    if args.next().is_some() {
        return Err("usage: cargo run -p hello-xml-inspect -- [path/to/document.xml]".to_owned());
    }
    Ok(path)
}

fn print_node(document: &XmlDocument, node: XmlNodeId, depth: usize) -> Result<(), String> {
    let indent = "  ".repeat(depth);
    let span = document
        .node_span(node)
        .ok_or_else(|| "document returned an invalid node span".to_owned())?;
    match document
        .node_kind(node)
        .ok_or_else(|| "document returned an invalid node kind".to_owned())?
    {
        XmlNodeKind::Element {
            name,
            lexical_prefix,
            attributes,
        } => {
            print!(
                "{indent}element {}",
                display_name(name, lexical_prefix.as_deref())
            );
            print_span(span.start, span.end);
            println!();
            for attribute in attributes {
                println!(
                    "{indent}  attribute {} = {:?} [{}..{}]",
                    display_name(&attribute.name, attribute.lexical_prefix.as_deref()),
                    attribute.value,
                    attribute.span.start,
                    attribute.span.end,
                );
            }
        }
        XmlNodeKind::Text { text } => {
            println!("{indent}text {:?} [{}..{}]", text, span.start, span.end);
        }
        XmlNodeKind::Comment { text } => {
            println!("{indent}comment {:?} [{}..{}]", text, span.start, span.end);
        }
        XmlNodeKind::ProcessingInstruction { target, data } => {
            println!(
                "{indent}processing-instruction {target:?} {:?} [{}..{}]",
                data, span.start, span.end
            );
        }
    }

    for child in document
        .children(node)
        .ok_or_else(|| "document returned invalid child nodes".to_owned())?
    {
        print_node(document, *child, depth + 1)?;
    }
    Ok(())
}

fn display_name(name: &ExpandedName, lexical_prefix: Option<&str>) -> String {
    match (lexical_prefix, name.namespace_uri.as_deref()) {
        (Some(prefix), Some(namespace)) => {
            format!("{prefix}:{} {{{namespace}}}", name.local_name)
        }
        (None, Some(namespace)) => format!("{} {{{namespace}}}", name.local_name),
        (_, None) => name.local_name.clone(),
    }
}

fn print_span(start: usize, end: usize) {
    print!(" [{start}..{end}]");
}

fn format_diagnostic(diagnostic: XmlDiagnostic) -> String {
    let span = diagnostic
        .span
        .map(|span| format!(" [{}..{}]", span.start, span.end))
        .unwrap_or_default();
    format!(
        "{:?}/{:?}{span}: {}",
        diagnostic.category, diagnostic.code, diagnostic.message
    )
}
