use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

use abstract_gui::{
    load_documents_from_paths, page_nodes, render_document, scan_html_paths, validate_document,
    Document, TreeChild, TreeSection,
};

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "gui".to_string());
    let Some(command) = args.next() else {
        print_usage(&program);
        process::exit(2);
    };
    if !matches!(
        command.as_str(),
        "check" | "page" | "drill" | "inherit" | "node" | "nav" | "scan"
    ) {
        eprintln!("unknown command: {command}");
        print_usage(&program);
        process::exit(2);
    }
    let raw_paths = args.collect::<Vec<_>>();
    let paths = match if command == "scan" {
        resolve_scan_input_paths(raw_paths)
    } else {
        resolve_input_paths(raw_paths)
    } {
        Ok(paths) => paths,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };

    match command.as_str() {
        "check" => {
            let _doc = load_and_validate(&paths);
            println!("ok");
        }
        "page" => {
            let doc = load_and_validate(&paths);
            match page_nodes(&doc) {
                Ok(pages) => {
                    for page in pages {
                        println!("{page}");
                    }
                }
                Err(errors) => {
                    for err in errors {
                        eprintln!("validation error: {}", err.message);
                    }
                    process::exit(1);
                }
            }
        }
        "drill" => {
            let doc = load_and_validate(&paths);
            print_tree_section(&doc.drill);
        }
        "inherit" => {
            let doc = load_and_validate(&paths);
            print_tree_section(&doc.inherit);
        }
        "node" => {
            let doc = load_and_validate(&paths);
            for node_id in doc.node.keys() {
                println!("{node_id}");
            }
        }
        "nav" => {
            let doc = load_and_validate(&paths);
            for nav_id in doc.nav.keys() {
                println!("{nav_id}");
            }
        }
        "scan" => {
            let doc = match scan_html_paths(paths.iter()) {
                Ok(doc) => doc,
                Err(err) => {
                    eprintln!("scan error: {}", err.message);
                    process::exit(1);
                }
            };
            if let Err(errors) = validate_document(&doc) {
                for err in errors {
                    eprintln!("validation error: {}", err.message);
                }
                process::exit(1);
            }
            print!("{}", render_document(&doc));
        }
        _ => unreachable!(),
    }
}

fn print_usage(program: &str) {
    eprintln!("usage: {program} check <file.gui> [more.gui ...]");
    eprintln!("       {program} page [file.gui ...]");
    eprintln!("       {program} drill [file.gui ...]");
    eprintln!("       {program} inherit [file.gui ...]");
    eprintln!("       {program} node [file.gui ...]");
    eprintln!("       {program} nav [file.gui ...]");
    eprintln!("       {program} scan <file.html> [more.html ...]");
}

fn load_and_validate(paths: &[PathBuf]) -> Document {
    let doc = match load_documents_from_paths(paths.iter()) {
        Ok(doc) => doc,
        Err(err) => {
            eprintln!("syntax error: {}", err.message);
            process::exit(1);
        }
    };
    if let Err(errors) = validate_document(&doc) {
        for err in errors {
            eprintln!("validation error: {}", err.message);
        }
        process::exit(1);
    }
    doc
}

fn print_tree_section(section: &TreeSection) {
    for (root, children) in section {
        println!("{root}");
        print_tree_children(children, 1);
    }
}

fn print_tree_children(children: &[TreeChild], depth: usize) {
    for child in children {
        let indent = "  ".repeat(depth);
        match child {
            TreeChild::Leaf(id) => println!("{indent}{id}"),
            TreeChild::Branch(id, nested) => {
                println!("{indent}{id}");
                print_tree_children(nested, depth + 1);
            }
        }
    }
}

fn resolve_input_paths(paths: Vec<String>) -> Result<Vec<PathBuf>, String> {
    if paths.is_empty() {
        let mut discovered = Vec::new();
        collect_gui_files(Path::new("."), &mut discovered).map_err(|err| err.to_string())?;
        discovered.sort();
        if discovered.is_empty() {
            return Err("no .gui files found under current directory".to_string());
        }
        return Ok(discovered);
    }

    Ok(paths.into_iter().map(PathBuf::from).collect())
}

fn collect_gui_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_gui_files(&path, out)?;
            continue;
        }
        if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("gui") {
            out.push(path);
        }
    }
    Ok(())
}

fn resolve_scan_input_paths(paths: Vec<String>) -> Result<Vec<PathBuf>, String> {
    if paths.is_empty() {
        return Err("scan requires at least one html file".to_string());
    }
    Ok(paths.into_iter().map(PathBuf::from).collect())
}
