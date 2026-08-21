use cargo_metadata;
use cargo_metadata::Metadata;
use lazy_static::lazy_static;
use std::{
    env, fmt,
    fmt::Display,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "expander",
    about = "Add snippets of ac-library-rs to cargo-compete workspace"
)]
struct App {
    #[structopt(short, long, required = true)]
    target_file_name: String,
    #[structopt(short, long)]
    doc_hidden: bool,
    #[structopt(global = true)]
    module_ids: Vec<Module>,
    #[structopt(short, long)]
    all: bool,
}

#[derive(Debug, Clone)]
enum Module {
    Convolution,
    FenwickTree,
    Dsu,
    LazySegTree,
    Math,
    MaxFlow,
    MinCostFlow,
    ModInt,
    Scc,
    SegTree,
    String,
    Twosat,
    Unknoun(String),
}

const ALL_MODULE: [Module; 12] = [
    Module::Convolution,
    Module::FenwickTree,
    Module::Dsu,
    Module::LazySegTree,
    Module::Math,
    Module::MaxFlow,
    Module::MinCostFlow,
    Module::ModInt,
    Module::Scc,
    Module::SegTree,
    Module::String,
    Module::Twosat,
];

impl FromStr for Module {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "convolution" => Ok(Module::Convolution),
            "fenwicktree" => Ok(Module::FenwickTree),
            "dsu" => Ok(Module::Dsu),
            "lazysegtree" => Ok(Module::LazySegTree),
            "math" => Ok(Module::Math),
            "maxflow" => Ok(Module::MaxFlow),
            "mincostflow" => Ok(Module::MinCostFlow),
            "modint" => Ok(Module::ModInt),
            "scc" => Ok(Module::Scc),
            "segtree" => Ok(Module::SegTree),
            "string" => Ok(Module::String),
            "twosat" => Ok(Module::Twosat),
            _ => Ok(Module::Unknoun(s.to_string())),
        }
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Module::Unknoun(_) = self {
            Err(fmt::Error)
        } else {
            write!(
                f,
                "{}",
                match self {
                    Module::Convolution => "convolution",
                    Module::FenwickTree => "fenwicktree",
                    Module::Dsu => "dsu",
                    Module::LazySegTree => "segtree",
                    Module::Math => "math",
                    Module::MaxFlow => "maxflow",
                    Module::MinCostFlow => "mincostflow",
                    Module::ModInt => "modint",
                    Module::Scc => "scc",
                    Module::SegTree => "segtree",
                    Module::String => "string",
                    Module::Twosat => "twosat",
                    Module::Unknoun(_) => unreachable!(),
                }
            )
        }
    }
}

lazy_static! {
    static ref AC_LIBRARY_RS_HOME: String = env::var("AC_LIBRARY_RS_HOME")
        .expect("environment variable $AC_LIBRARY_RS_HOME isn' t set");
}

fn get_metadata() -> cargo_metadata::Metadata {
    cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("failed to get workspace metadata") // return;
}

fn get_excutable_bin_names(result: &Metadata) -> Vec<String> {
    // if cd is compete workspace?
    // let result = cargo_metadata::MetadataCommand::new()
    //     .no_deps()
    //     .exec()
    //     .expect("failed to get workspace metadata"); // return;

    let metadata = result.packages[0].metadata.clone();

    let config = metadata
        .get("cargo-compete")
        .expect("current dir is not cargo compete workspace");

    config
        .get("bin")
        .unwrap()
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.clone())
        .collect()
}

fn get_workspace_root(result: &Metadata) -> PathBuf {
    result.workspace_root.clone()
}

fn try_to_string(m: &Module) -> Option<String> {
    match m {
        Module::Convolution => Some("convolution".to_string()),
        Module::FenwickTree => Some("fenwicktree".to_string()),
        Module::Dsu => Some("dsu".to_string()),
        Module::LazySegTree => Some("segtree".to_string()),
        Module::Math => Some("math".to_string()),
        Module::MaxFlow => Some("maxflow".to_string()),
        Module::MinCostFlow => Some("mincostflow".to_string()),
        Module::ModInt => Some("modint".to_string()),
        Module::Scc => Some("scc".to_string()),
        Module::SegTree => Some("segtree".to_string()),
        Module::String => Some("string".to_string()),
        Module::Twosat => Some("twosat".to_string()),
        Module::Unknoun(_) => None,
    }
}

fn exe_script(modules: &[Module]) -> String {
    String::from_utf8(
        Command::new(format!("{}/expand.py", *AC_LIBRARY_RS_HOME).as_str())
            .args(
                modules
                    .iter()
                    .filter_map(|x| try_to_string(x))
                    .collect::<Vec<String>>(),
            )
            .output()
            .expect("failed to execute expand.py")
            .stdout,
    )
    .expect("failed to convert bytes into String")
}

fn remove_comment_line(s: String) -> String {
    let mut res = String::new();
    for l in s.lines() {
        // this is a comment line
        if l.trim().starts_with("//") {
            continue;
        }
        // this is an empty line
        if l.chars().all(|x| x.is_whitespace()) {
            continue;
        }
        res += l;
        res += "\n";
    }
    res
}

fn main() {
    // parse command line args
    let App {
        target_file_name,
        doc_hidden,
        mut module_ids,
        all,
    } = App::from_args();

    if all {
        module_ids = ALL_MODULE.to_vec();
    }

    if module_ids.is_empty() {
        return;
    }

    let cargo_metadata_result = get_metadata();
    // check if current dir is cargo-compete workspace
    let bins = get_excutable_bin_names(&cargo_metadata_result);
    let target_contest_root = get_workspace_root(&cargo_metadata_result);

    // checl if entered output file is available
    assert!(bins.contains(&target_file_name.to_lowercase()));

    // execute expand.py
    env::set_current_dir(&Path::new(&AC_LIBRARY_RS_HOME.as_str()))
        .expect("cannot `cd $AC_LIBRARY_RS_HOME`");
    let mut output = exe_script(&module_ids);
    // dbg!(output);

    // remove doc and empty lines
    if doc_hidden {
        output = remove_comment_line(output);
    }

    let target_path = {
        let mut r = target_contest_root;
        r.push("src");
        r.push("bin");
        r.push(format!("{}.rs", target_file_name));
        r
    };

    // dbg!(target_path);

    // let mut target_path = env::current_dir().expect("failed to get current dir");
    // target_path.push("src");
    // target_path.push("bin");
    // target_path.push(format!("{}.rs", target_file_name));
    let mut output_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(target_path)
        .expect("failed to open target file");

    // dbg!(&output);
    write!(&mut output_file, "{}", output).expect("failed to write modules");

    let mut modules = String::new();
    for m in module_ids.iter().filter_map(|x| try_to_string(x)) {
        modules += m.as_str();
        modules += ", ";
    }
    modules.pop();
    modules.pop();
    println!("Successfully wrote ac-library-rs modules: {}", modules);
}
