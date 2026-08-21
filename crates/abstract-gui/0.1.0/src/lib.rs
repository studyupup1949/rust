use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde_yaml::{Mapping, Value};

#[derive(Debug, Clone)]
pub struct Document {
    pub app: Option<String>,
    pub drill: TreeSection,
    pub inherit: TreeSection,
    pub nav: BTreeMap<String, BTreeSet<String>>,
    pub node: BTreeMap<String, NodeSpec>,
    pub groups: Vec<GroupSpec>,
}

#[derive(Debug, Clone, Default)]
pub struct NodeSpec {
    pub attrs: BTreeMap<String, AttrValue>,
}

#[derive(Debug, Clone)]
pub enum AttrValue {
    Scalar(String),
    Vector(BTreeSet<String>),
}

#[derive(Debug, Clone)]
pub struct GroupSpec {
    pub id: String,
    pub members: BTreeSet<String>,
}

pub type TreeSection = BTreeMap<String, Vec<TreeChild>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeChild {
    Leaf(String),
    Branch(String, Vec<TreeChild>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn load_document_from_path(path: impl AsRef<Path>) -> Result<Document, ValidationError> {
    let mut stack = Vec::new();
    load_document_from_path_inner(path.as_ref(), &mut stack)
}

pub fn load_documents_from_paths<I, P>(paths: I) -> Result<Document, ValidationError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut merged = Document {
        app: None,
        drill: BTreeMap::new(),
        inherit: BTreeMap::new(),
        nav: BTreeMap::new(),
        node: BTreeMap::new(),
        groups: Vec::new(),
    };
    let mut saw_any = false;

    for path in paths {
        saw_any = true;
        let doc = load_document_from_path(path)?;
        merge_document(&mut merged, doc);
    }

    if !saw_any {
        return Err(ValidationError::new("no .gui files matched input"));
    }

    Ok(merged)
}

pub fn parse_document(input: &str) -> Result<Document, ValidationError> {
    let normalized = normalize_tree_shorthand(input);
    let root: Value = serde_yaml::from_str(&normalized)
        .map_err(|err| ValidationError::new(format!("YAML parse error: {err}")))?;
    let root_map = root
        .as_mapping()
        .ok_or_else(|| ValidationError::new("top level must be a mapping"))?;

    let app = optional_string(root_map, "app")?;
    let drill = parse_tree_section(root_map, "drill")?;
    let inherit = parse_tree_section(root_map, "inherit")?;
    let nav = parse_nav_section(root_map, "nav")?;
    let node = parse_node_section(root_map, "node")?;
    let groups = parse_groups(root_map, "groups")?;

    Ok(Document {
        app,
        drill,
        inherit,
        nav,
        node,
        groups,
    })
}

pub fn validate_document(doc: &Document) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    let inherit_leaves = collect_inherit_leaves(&doc.inherit, &mut errors);
    let drill_nodes = collect_all_nodes(&doc.drill, true, &mut errors);
    let pages = page_nodes_from_sets(&inherit_leaves, &drill_nodes);

    for node in &drill_nodes {
        if !pages.contains(node) {
            errors.push(ValidationError::new(format!(
                "drill node `{node}` must be page-like"
            )));
        }
    }

    for (nav_id, targets) in &doc.nav {
        if targets.is_empty() {
            errors.push(ValidationError::new(format!(
                "nav `{nav_id}` must not be empty"
            )));
        }
        for target in targets {
            if !pages.contains(target) {
                errors.push(ValidationError::new(format!(
                    "nav `{nav_id}` target `{target}` must be a page"
                )));
            }
        }
    }

    for (node_id, spec) in &doc.node {
        if let Some(AttrValue::Vector(nav_ids)) = spec.attrs.get("nav") {
            for nav_id in nav_ids {
                if !doc.nav.contains_key(nav_id) {
                    errors.push(ValidationError::new(format!(
                        "node `{node_id}` references unknown nav `{nav_id}`"
                    )));
                }
            }
        }
    }

    for group in &doc.groups {
        for member in &group.members {
            if !pages.contains(member) {
                errors.push(ValidationError::new(format!(
                    "group `{}` member `{member}` must be a page",
                    group.id
                )));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn page_nodes(doc: &Document) -> Result<BTreeSet<String>, Vec<ValidationError>> {
    let mut errors = Vec::new();
    let inherit_leaves = collect_inherit_leaves(&doc.inherit, &mut errors);
    let drill_nodes = collect_all_nodes(&doc.drill, true, &mut errors);
    if errors.is_empty() {
        Ok(page_nodes_from_sets(&inherit_leaves, &drill_nodes))
    } else {
        Err(errors)
    }
}

fn load_document_from_path_inner(
    path: &Path,
    stack: &mut Vec<PathBuf>,
) -> Result<Document, ValidationError> {
    let canonical = path.canonicalize().map_err(|err| {
        ValidationError::new(format!("failed to open `{}`: {err}", path.display()))
    })?;
    if stack.contains(&canonical) {
        let mut chain = stack
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>();
        chain.push(canonical.display().to_string());
        return Err(ValidationError::new(format!(
            "import cycle detected: {}",
            chain.join(" -> ")
        )));
    }
    stack.push(canonical.clone());

    let input = fs::read_to_string(&canonical).map_err(|err| {
        ValidationError::new(format!("failed to read `{}`: {err}", canonical.display()))
    })?;

    let (imports, body) = preprocess_source(&input).map_err(|err| {
        ValidationError::new(format!("{} in `{}`", err.message, canonical.display()))
    })?;

    let mut doc = Document {
        app: None,
        drill: BTreeMap::new(),
        inherit: BTreeMap::new(),
        nav: BTreeMap::new(),
        node: BTreeMap::new(),
        groups: Vec::new(),
    };

    let base_dir = canonical.parent().unwrap_or_else(|| Path::new("."));
    for import in imports {
        let import_path = base_dir.join(import);
        let imported = load_document_from_path_inner(&import_path, stack)?;
        merge_document(&mut doc, imported);
    }

    let current = parse_document(&body)?;
    merge_document(&mut doc, current);
    stack.pop();
    Ok(doc)
}

fn optional_string(root: &Mapping, key: &str) -> Result<Option<String>, ValidationError> {
    match root.get(Value::String(key.to_string())) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ValidationError::new(format!("`{key}` must be a string"))),
        None => Ok(None),
    }
}

fn preprocess_source(input: &str) -> Result<(Vec<String>, String), ValidationError> {
    let mut imports = Vec::new();
    let mut body = String::new();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#import") {
            let rest = trimmed.trim_start_matches("#import").trim();
            let Some(path) = parse_import_target(rest) else {
                return Err(ValidationError::new(
                    "invalid #import syntax; expected #import \"path.gui\"",
                ));
            };
            imports.push(path);
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        body.push_str(line);
        body.push('\n');
    }
    Ok((imports, normalize_tree_shorthand(&body)))
}

fn normalize_tree_shorthand(input: &str) -> String {
    let mut out = String::new();
    let mut in_tree_section = false;

    for line in input.lines() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if indent == 0 {
            in_tree_section = matches!(trimmed, "drill:" | "inherit:");
        }

        if in_tree_section && should_promote_tree_leaf(line) {
            out.push_str(line);
            out.push(':');
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }

    out
}

fn should_promote_tree_leaf(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with('-')
        && !trimmed.contains(':')
        && !trimmed.starts_with('#')
}

fn parse_import_target(rest: &str) -> Option<String> {
    let mut chars = rest.chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut out = String::new();
    for ch in chars {
        if ch == '"' {
            return Some(out);
        }
        out.push(ch);
    }
    None
}

fn merge_document(into: &mut Document, other: Document) {
    if other.app.is_some() {
        into.app = other.app;
    }
    merge_tree_section(&mut into.drill, other.drill);
    merge_tree_section(&mut into.inherit, other.inherit);
    merge_nav_section(&mut into.nav, other.nav);
    merge_node_section(&mut into.node, other.node);
    merge_groups(&mut into.groups, other.groups);
}

fn merge_tree_section(into: &mut TreeSection, other: TreeSection) {
    for (key, value) in other {
        into.entry(key).or_default().extend(value);
    }
}

fn merge_nav_section(
    into: &mut BTreeMap<String, BTreeSet<String>>,
    other: BTreeMap<String, BTreeSet<String>>,
) {
    for (key, value) in other {
        into.entry(key).or_default().extend(value);
    }
}

fn merge_node_section(into: &mut BTreeMap<String, NodeSpec>, other: BTreeMap<String, NodeSpec>) {
    for (node_id, spec) in other {
        let entry = into.entry(node_id).or_default();
        for (attr_name, attr_value) in spec.attrs {
            match (entry.attrs.get_mut(&attr_name), attr_value) {
                (Some(AttrValue::Vector(existing)), AttrValue::Vector(next)) => {
                    existing.extend(next);
                }
                (_, next) => {
                    entry.attrs.insert(attr_name, next);
                }
            }
        }
    }
}

fn merge_groups(into: &mut Vec<GroupSpec>, other: Vec<GroupSpec>) {
    let mut index = into
        .iter()
        .enumerate()
        .map(|(idx, group)| (group.id.clone(), idx))
        .collect::<BTreeMap<_, _>>();
    for group in other {
        if let Some(existing_idx) = index.get(&group.id).copied() {
            into[existing_idx].members.extend(group.members);
        } else {
            index.insert(group.id.clone(), into.len());
            into.push(group);
        }
    }
}

fn parse_tree_section(root: &Mapping, key: &str) -> Result<TreeSection, ValidationError> {
    let Some(value) = root.get(Value::String(key.to_string())) else {
        return Ok(BTreeMap::new());
    };
    let mapping = value
        .as_mapping()
        .ok_or_else(|| ValidationError::new(format!("`{key}` must be a mapping")))?;
    let mut section = BTreeMap::new();
    for (node_key, children_value) in mapping {
        let node_id = expect_string(node_key, key)?;
        let children = parse_tree_children(children_value, key)?;
        section.insert(node_id, children);
    }
    Ok(section)
}

fn parse_tree_children(value: &Value, section: &str) -> Result<Vec<TreeChild>, ValidationError> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::Mapping(map) => parse_tree_mapping_children(map, section),
        Value::Sequence(seq) => parse_tree_sequence_children(seq, section),
        _ => Err(ValidationError::new(format!(
            "entries in `{section}` must contain a child mapping"
        ))),
    }
}

fn parse_tree_mapping_children(
    map: &Mapping,
    section: &str,
) -> Result<Vec<TreeChild>, ValidationError> {
    let mut children = Vec::new();
    for (child_key, child_value) in map {
        let child_id = expect_string(child_key, section)?;
        let nested = parse_tree_children(child_value, section)?;
        if nested.is_empty() {
            children.push(TreeChild::Leaf(child_id));
        } else {
            children.push(TreeChild::Branch(child_id, nested));
        }
    }
    Ok(children)
}

fn parse_tree_sequence_children(
    seq: &[Value],
    section: &str,
) -> Result<Vec<TreeChild>, ValidationError> {
    let mut children = Vec::new();
    for item in seq {
        match item {
            Value::String(id) => children.push(TreeChild::Leaf(id.clone())),
            Value::Mapping(map) => {
                if map.len() != 1 {
                    return Err(ValidationError::new(format!(
                        "branch entries in `{section}` must contain exactly one key"
                    )));
                }
                let (branch_key, branch_value) = map.iter().next().expect("single entry");
                let branch_id = expect_string(branch_key, section)?;
                let branch_children = parse_tree_children(branch_value, section)?;
                if branch_children.is_empty() {
                    children.push(TreeChild::Leaf(branch_id));
                } else {
                    children.push(TreeChild::Branch(branch_id, branch_children));
                }
            }
            _ => {
                return Err(ValidationError::new(format!(
                    "entries in `{section}` must be strings or single-key mappings"
                )))
            }
        }
    }
    Ok(children)
}

fn parse_nav_section(
    root: &Mapping,
    key: &str,
) -> Result<BTreeMap<String, BTreeSet<String>>, ValidationError> {
    let Some(value) = root.get(Value::String(key.to_string())) else {
        return Ok(BTreeMap::new());
    };
    let mapping = value
        .as_mapping()
        .ok_or_else(|| ValidationError::new(format!("`{key}` must be a mapping")))?;
    let mut navs = BTreeMap::new();
    for (nav_key, nav_value) in mapping {
        let nav_id = expect_string(nav_key, key)?;
        let members = parse_string_set(nav_value, &format!("nav `{nav_id}`"))?;
        navs.insert(nav_id, members);
    }
    Ok(navs)
}

fn parse_node_section(
    root: &Mapping,
    key: &str,
) -> Result<BTreeMap<String, NodeSpec>, ValidationError> {
    let Some(value) = root.get(Value::String(key.to_string())) else {
        return Ok(BTreeMap::new());
    };
    let mapping = value
        .as_mapping()
        .ok_or_else(|| ValidationError::new(format!("`{key}` must be a mapping")))?;
    let mut nodes = BTreeMap::new();
    for (node_key, node_value) in mapping {
        let node_id = expect_string(node_key, key)?;
        let attrs_map = node_value.as_mapping().ok_or_else(|| {
            ValidationError::new(format!("node `{node_id}` must be a mapping of attributes"))
        })?;
        let mut attrs = BTreeMap::new();
        for (attr_key, attr_value) in attrs_map {
            let attr_name = expect_string(attr_key, &format!("node `{node_id}`"))?;
            let parsed = match attr_value {
                Value::Sequence(_) => AttrValue::Vector(parse_string_set(
                    attr_value,
                    &format!("node `{node_id}` attribute `{attr_name}`"),
                )?),
                Value::String(value) => AttrValue::Scalar(value.clone()),
                Value::Number(value) => AttrValue::Scalar(value.to_string()),
                Value::Bool(value) => AttrValue::Scalar(value.to_string()),
                Value::Null => AttrValue::Scalar("null".to_string()),
                Value::Mapping(_) => {
                    return Err(ValidationError::new(format!(
                        "node `{node_id}` attribute `{attr_name}` must be scalar or vector"
                    )))
                }
                Value::Tagged(_) => {
                    return Err(ValidationError::new(format!(
                        "node `{node_id}` attribute `{attr_name}` must not use tags"
                    )))
                }
            };
            attrs.insert(attr_name, parsed);
        }
        nodes.insert(node_id, NodeSpec { attrs });
    }
    Ok(nodes)
}

fn parse_groups(root: &Mapping, key: &str) -> Result<Vec<GroupSpec>, ValidationError> {
    let Some(value) = root.get(Value::String(key.to_string())) else {
        return Ok(Vec::new());
    };
    let seq = value
        .as_sequence()
        .ok_or_else(|| ValidationError::new(format!("`{key}` must be a sequence")))?;
    let mut groups = Vec::new();
    for item in seq {
        let map = item
            .as_mapping()
            .ok_or_else(|| ValidationError::new("group entries must be mappings"))?;
        let id = required_string(map, "id", "group")?;
        let members_value = map
            .get(Value::String("members".to_string()))
            .ok_or_else(|| ValidationError::new(format!("group `{id}` must define `members`")))?;
        let members = parse_string_set(members_value, &format!("group `{id}` members"))?;
        groups.push(GroupSpec { id, members });
    }
    Ok(groups)
}

fn parse_string_set(value: &Value, context: &str) -> Result<BTreeSet<String>, ValidationError> {
    let seq = value
        .as_sequence()
        .ok_or_else(|| ValidationError::new(format!("{context} must be a sequence")))?;
    let mut out = BTreeSet::new();
    for item in seq {
        let id = item
            .as_str()
            .ok_or_else(|| ValidationError::new(format!("{context} must contain only strings")))?;
        out.insert(id.to_string());
    }
    Ok(out)
}

fn expect_string(value: &Value, context: &str) -> Result<String, ValidationError> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| ValidationError::new(format!("keys in `{context}` must be strings")))
}

fn required_string(map: &Mapping, key: &str, context: &str) -> Result<String, ValidationError> {
    map.get(Value::String(key.to_string()))
        .ok_or_else(|| ValidationError::new(format!("{context} must define `{key}`")))?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| ValidationError::new(format!("{context} field `{key}` must be a string")))
}

fn page_nodes_from_sets(
    inherit_leaves: &BTreeSet<String>,
    drill_nodes: &BTreeSet<String>,
) -> BTreeSet<String> {
    inherit_leaves.union(drill_nodes).cloned().collect()
}

fn collect_inherit_leaves(
    section: &TreeSection,
    errors: &mut Vec<ValidationError>,
) -> BTreeSet<String> {
    let mut leaves = BTreeSet::new();
    for (root, children) in section {
        collect_inherit_leaves_children(root, children, &mut leaves, errors, true);
    }
    leaves
}

fn collect_inherit_leaves_children(
    current: &str,
    children: &[TreeChild],
    leaves: &mut BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
    current_is_non_leaf: bool,
) {
    if children.is_empty() && !current_is_non_leaf && !leaves.insert(current.to_string()) {
        errors.push(ValidationError::new(format!(
            "inherit leaf `{current}` appears more than once"
        )));
    }
    for child in children {
        match child {
            TreeChild::Leaf(id) => {
                if !leaves.insert(id.clone()) {
                    errors.push(ValidationError::new(format!(
                        "inherit leaf `{id}` appears more than once"
                    )));
                }
            }
            TreeChild::Branch(id, grand_children) => {
                collect_inherit_leaves_children(id, grand_children, leaves, errors, false);
            }
        }
    }
}

fn collect_all_nodes(
    section: &TreeSection,
    include_roots: bool,
    errors: &mut Vec<ValidationError>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (root, children) in section {
        if include_roots && !out.insert(root.clone()) {
            errors.push(ValidationError::new(format!(
                "drill node `{root}` appears more than once"
            )));
        }
        collect_nodes_children(children, &mut out, errors, root.as_str(), include_roots);
    }
    out
}

fn collect_nodes_children(
    children: &[TreeChild],
    out: &mut BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
    _parent: &str,
    include_branches: bool,
) {
    for child in children {
        match child {
            TreeChild::Leaf(id) => {
                if !out.insert(id.clone()) {
                    errors.push(ValidationError::new(format!(
                        "drill node `{id}` appears more than once"
                    )));
                }
            }
            TreeChild::Branch(id, nested) => {
                if include_branches && !out.insert(id.clone()) {
                    errors.push(ValidationError::new(format!(
                        "drill node `{id}` appears more than once"
                    )));
                } else if !include_branches && !out.insert(id.clone()) {
                    errors.push(ValidationError::new(format!(
                        "drill node `{id}` appears more than once"
                    )));
                }
                collect_nodes_children(nested, out, errors, id, include_branches);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        load_document_from_path, load_documents_from_paths, parse_document, validate_document,
    };

    const DEMO: &str = include_str!("../examples/demo.gui");

    #[test]
    fn parses_and_validates_demo() {
        let doc = parse_document(DEMO).expect("parse demo");
        validate_document(&doc).expect("validate demo");
    }

    #[test]
    fn rejects_drill_node_missing_from_inherit_leaves() {
        let src = r#"
app: Bad
drill:
  Home:
    - Missing
inherit:
  RootLayout:
    - Home
nav:
  GlobalNav: [Home, Ghost]
node:
  Home:
    path: /
"#;
        let doc = parse_document(src).expect("parse");
        let errors = validate_document(&doc).expect_err("should fail");
        assert!(errors.iter().any(|err| err.message.contains("Ghost")));
    }

    #[test]
    fn loads_imports_and_skips_hash_comments() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-import-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let imported = dir.join("base.gui");
        let root = dir.join("root.gui");
        fs::write(
            &imported,
            "# comment\nnav:\n  GlobalNav: [Home]\nnode:\n  RootLayout:\n    nav: [GlobalNav]\n",
        )
        .expect("write import");
        fs::write(
            &root,
            format!(
                "#import \"{}\"\n# comment\ndrill:\n  Home: []\ninherit:\n  RootLayout:\n    - Home\nnode:\n  Home:\n    path: /\n",
                imported.file_name().expect("name").to_string_lossy()
            ),
        )
        .expect("write root");

        let doc = load_document_from_path(&root).expect("load");
        validate_document(&doc).expect("validate");
        assert!(doc.nav.contains_key("GlobalNav"));
        assert!(doc.node.contains_key("RootLayout"));
    }

    #[test]
    fn parses_mapping_tree_with_leaf_shorthand() {
        let src = r#"
app: Demo
drill:
  Home:
    Products:
      ProductDetail:
        ProductReviews
inherit:
  RootLayout:
    Home
    Products
    ProductDetail
    ProductReviews
node:
  Home:
    path: /
"#;

        let doc = parse_document(src).expect("parse");
        validate_document(&doc).expect("validate");
    }

    #[test]
    fn merges_multiple_documents() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-merge-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let a = dir.join("a.gui");
        let b = dir.join("b.gui");
        fs::write(
            &a,
            "drill:\n  Home:\ninherit:\n  RootLayout:\n    Home:\nnav:\n  GlobalNav: [Home]\n",
        )
        .expect("write a");
        fs::write(
            &b,
            "drill:\n  AdminRoot:\ninherit:\n  AdminShell:\n    AdminRoot:\nnode:\n  RootLayout:\n    nav: [GlobalNav]\n",
        )
        .expect("write b");

        let doc = load_documents_from_paths([&a, &b]).expect("load merged");
        validate_document(&doc).expect("validate merged");
        assert!(doc.drill.contains_key("Home"));
        assert!(doc.drill.contains_key("AdminRoot"));
    }
}
