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
    resolve_paths_by_extension(
        paths,
        &["gui"],
        "no .gui files found under current directory",
    )
}

fn collect_files_with_extensions(
    dir: &Path,
    extensions: &[&str],
    out: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files_with_extensions(&path, extensions, out)?;
            continue;
        }
        if file_type.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    extensions
                        .iter()
                        .any(|candidate| ext.eq_ignore_ascii_case(candidate))
                })
        {
            out.push(path);
        }
    }
    Ok(())
}

fn resolve_scan_input_paths(paths: Vec<String>) -> Result<Vec<PathBuf>, String> {
    resolve_paths_by_extension(
        paths,
        &["html", "htm"],
        "scan requires at least one html file",
    )
}

fn resolve_paths_by_extension(
    paths: Vec<String>,
    extensions: &[&str],
    empty_message: &str,
) -> Result<Vec<PathBuf>, String> {
    if paths.is_empty() {
        let mut discovered = Vec::new();
        collect_files_with_extensions(Path::new("."), extensions, &mut discovered)
            .map_err(|err| err.to_string())?;
        discovered.sort();
        if discovered.is_empty() {
            return Err(empty_message.to_string());
        }
        return Ok(discovered);
    }

    let mut resolved = Vec::new();
    for raw_path in paths {
        let path = PathBuf::from(raw_path);
        if path.is_dir() {
            collect_files_with_extensions(&path, extensions, &mut resolved)
                .map_err(|err| err.to_string())?;
        } else {
            resolved.push(path);
        }
    }
    resolved.sort();
    resolved.dedup();
    if resolved.is_empty() {
        return Err(empty_message.to_string());
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::{resolve_input_paths, resolve_scan_input_paths};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn resolve_input_paths_expands_gui_directories() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-main-gui-dir-{unique}"));
        fs::create_dir_all(dir.join("nested")).expect("mkdir");
        fs::write(
            dir.join("a.gui"),
            "drill:\n  Home:\ninherit:\n  RootLayout:\n    Home:\n",
        )
        .expect("write a");
        fs::write(
            dir.join("nested").join("b.gui"),
            "drill:\n  Page:\ninherit:\n  RootLayout:\n    Page:\n",
        )
        .expect("write b");
        fs::write(dir.join("nested").join("ignore.txt"), "x").expect("write txt");

        let resolved =
            resolve_input_paths(vec![dir.to_string_lossy().into_owned()]).expect("resolve");
        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().any(|path| path.ends_with("a.gui")));
        assert!(resolved.iter().any(|path| path.ends_with("b.gui")));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_scan_input_paths_expands_html_directories() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-main-html-dir-{unique}"));
        fs::create_dir_all(dir.join("nested")).expect("mkdir");
        fs::write(dir.join("a.html"), "<html></html>").expect("write a");
        fs::write(dir.join("nested").join("b.htm"), "<html></html>").expect("write b");
        fs::write(dir.join("nested").join("ignore.gui"), "x").expect("write ignore");

        let resolved =
            resolve_scan_input_paths(vec![dir.to_string_lossy().into_owned()]).expect("resolve");
        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().any(|path| path.ends_with("a.html")));
        assert!(resolved.iter().any(|path| path.ends_with("b.htm")));

        fs::remove_dir_all(&dir).ok();
    }
}
