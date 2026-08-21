use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use scraper::{Html, Selector};
use serde_yaml::{Mapping, Value};
use url::Url;

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

#[derive(Debug, Clone)]
struct ScannedPage {
    page_id: String,
    title: String,
    path: String,
    breadcrumb_paths: Vec<String>,
    nav_candidates: Vec<NavCandidate>,
    has_docs_index_candidate: bool,
    dialogs: Vec<ScannedDialog>,
    opens: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct NavCandidate {
    label: Option<String>,
    targets: Vec<ScannedTarget>,
}

#[derive(Debug, Clone)]
struct NavCandidateMeta {
    selector: String,
}

#[derive(Debug, Clone)]
struct ScannedTarget {
    path: String,
    title: String,
}

#[derive(Debug, Clone)]
struct PageLocation {
    path: String,
    host: Option<String>,
}

#[derive(Debug, Default)]
struct NavCluster {
    owners: BTreeSet<String>,
    label: Option<String>,
    paths: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct LayoutGroup {
    layout_id: String,
    members: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct ScannedDialog {
    dialog_id: String,
    dom_id: Option<String>,
    dialog_kind: String,
    title: String,
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

pub fn scan_html_paths<I, P>(paths: I) -> Result<Document, ValidationError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut raw_pages = Vec::new();

    for path in paths {
        let path_ref = path.as_ref();
        let input = fs::read(path_ref).map_err(|err| {
            ValidationError::new(format!("failed to read `{}`: {err}", path_ref.display()))
        })?;
        let input = String::from_utf8_lossy(&input).into_owned();
        let html = Html::parse_document(&input);
        let title = extract_title(&html).unwrap_or_else(|| infer_title_from_path(path_ref));
        let location = extract_page_location(&html, path_ref);
        let page_id = make_identifier_for_page(&title, &location.path, "Page");
        raw_pages.push((page_id, title, location, html));
    }

    if raw_pages.is_empty() {
        return Err(ValidationError::new("no html files matched input"));
    }

    let pages = raw_pages
        .into_iter()
        .map(|(page_id, title, location, html)| {
            let dialogs = extract_dialogs(&html, &title);
            let opens = extract_dialog_opens(&html, &dialogs);
            ScannedPage {
                breadcrumb_paths: extract_breadcrumb_paths(&html, location.host.as_deref()),
                nav_candidates: extract_nav_candidates(
                    &html,
                    location.host.as_deref(),
                    &location.path,
                ),
                has_docs_index_candidate: has_large_docs_index_candidate(
                    &html,
                    location.host.as_deref(),
                    &location.path,
                ),
                dialogs,
                opens,
                page_id,
                title,
                path: location.path,
            }
        })
        .collect::<Vec<_>>();

    Ok(document_from_scanned_pages(&pages))
}

pub fn render_document(doc: &Document) -> String {
    let mut out = String::new();
    if let Some(app) = &doc.app {
        out.push_str("app: ");
        out.push_str(&yaml_scalar(app));
        out.push('\n');
        out.push('\n');
    }
    render_tree_section(&mut out, "drill", &doc.drill);
    out.push('\n');
    render_tree_section(&mut out, "inherit", &doc.inherit);
    out.push('\n');
    render_nav_section(&mut out, &doc.nav);
    out.push('\n');
    render_node_section(&mut out, &doc.node);
    if !doc.groups.is_empty() {
        out.push('\n');
        render_groups_section(&mut out, &doc.groups);
    }
    out
}

fn document_from_scanned_pages(pages: &[ScannedPage]) -> Document {
    let mut used_ids = pages
        .iter()
        .map(|page| page.page_id.clone())
        .collect::<BTreeSet<_>>();
    let mut pages_by_path = pages
        .iter()
        .cloned()
        .map(|page| (page.path.clone(), page))
        .collect::<BTreeMap<_, _>>();

    for page in pages {
        for candidate in &page.nav_candidates {
            for target in &candidate.targets {
                pages_by_path.entry(target.path.clone()).or_insert_with(|| {
                    let page_id = unique_page_id(&target.title, &target.path, &mut used_ids);
                    ScannedPage {
                        page_id,
                        title: target.title.clone(),
                        path: target.path.clone(),
                        breadcrumb_paths: Vec::new(),
                        nav_candidates: Vec::new(),
                        has_docs_index_candidate: false,
                        dialogs: Vec::new(),
                        opens: BTreeSet::new(),
                    }
                });
            }
        }
    }

    let scanned_pages = pages_by_path.values().cloned().collect::<Vec<_>>();
    let page_id_by_path = pages_by_path
        .iter()
        .map(|(path, page)| (path.clone(), page.page_id.clone()))
        .collect::<BTreeMap<_, _>>();

    let input_page_ids = pages
        .iter()
        .map(|page| page.page_id.clone())
        .collect::<BTreeSet<_>>();
    let input_page_count = input_page_ids.len();

    let mut nav_clusters = BTreeMap::<BTreeSet<String>, NavCluster>::new();
    for page in pages {
        for candidate in &page.nav_candidates {
            let targets = candidate
                .targets
                .iter()
                .filter_map(|target| page_id_by_path.get(&target.path).cloned())
                .collect::<BTreeSet<_>>();
            if targets.len() < 2 {
                continue;
            }
            let cluster = nav_clusters.entry(targets).or_default();
            cluster.owners.insert(page.page_id.clone());
            cluster
                .paths
                .extend(candidate.targets.iter().map(|target| target.path.clone()));
            if cluster.label.is_none() {
                cluster.label = candidate.label.clone();
            }
        }
    }

    let mut nav = BTreeMap::new();
    let mut nav_usage = BTreeMap::<String, BTreeSet<String>>::new();
    let mut root_nav_ids = BTreeSet::new();
    let mut next_nav_idx = 1;
    for (targets, cluster) in prune_redundant_nav_clusters(nav_clusters) {
        let preferred_nav_id = classify_nav_cluster(&cluster, input_page_count, targets.len());
        let nav_id = allocate_nav_id(preferred_nav_id, &nav, &mut next_nav_idx);
        if should_attach_nav_to_root_layout(&nav_id, &cluster, input_page_count, targets.len()) {
            root_nav_ids.insert(nav_id.clone());
        }
        nav.insert(nav_id.clone(), targets);
        for owner in cluster.owners {
            nav_usage.entry(owner).or_default().insert(nav_id.clone());
        }
    }

    let layout_groups = infer_layout_groups(&scanned_pages, &nav, &root_nav_ids);
    let drill = build_drill_section(&scanned_pages);
    let section_page_ids = collect_section_page_ids(&drill);
    let layout_open_ids = infer_layout_dialogs(pages, &layout_groups, &input_page_ids);
    let layout_promoted_open_ids = layout_open_ids
        .values()
        .flat_map(|opens| opens.iter().cloned())
        .collect::<BTreeSet<_>>();
    let root_open_ids = infer_root_layout_dialogs(pages, &input_page_ids)
        .difference(&layout_promoted_open_ids)
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut node = BTreeMap::new();
    let mut root_layout = NodeSpec::default();
    if !root_nav_ids.is_empty() {
        root_layout
            .attrs
            .insert("nav".to_string(), AttrValue::Vector(root_nav_ids.clone()));
    }
    if !root_open_ids.is_empty() {
        root_layout.attrs.insert(
            "opens".to_string(),
            AttrValue::Vector(root_open_ids.clone()),
        );
    }
    root_layout
        .attrs
        .insert("kind".to_string(), AttrValue::Scalar("layout".to_string()));
    node.insert("RootLayout".to_string(), root_layout);

    for group in &layout_groups {
        let mut attrs = BTreeMap::new();
        let layout_navs = nav
            .iter()
            .filter_map(|(nav_id, targets)| {
                if targets.len() >= 2 && targets.is_subset(&group.members) {
                    Some(nav_id.clone())
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        if !layout_navs.is_empty() {
            attrs.insert("nav".to_string(), AttrValue::Vector(layout_navs));
        }
        if let Some(opens) = layout_open_ids.get(&group.layout_id) {
            if !opens.is_empty() {
                attrs.insert("opens".to_string(), AttrValue::Vector(opens.clone()));
            }
        }
        attrs.insert("kind".to_string(), AttrValue::Scalar("layout".to_string()));
        node.insert(group.layout_id.clone(), NodeSpec { attrs });
    }

    for page in &scanned_pages {
        let mut attrs = BTreeMap::new();
        let kind = infer_node_kind(page, &section_page_ids);
        attrs.insert("kind".to_string(), AttrValue::Scalar(kind.to_string()));
        attrs.insert("title".to_string(), AttrValue::Scalar(page.title.clone()));
        attrs.insert("path".to_string(), AttrValue::Scalar(page.path.clone()));
        if let Some(nav_ids) = nav_usage.get(&page.page_id) {
            let filtered = nav_ids
                .iter()
                .filter(|nav_id| !root_nav_ids.contains(*nav_id))
                .cloned()
                .collect::<BTreeSet<_>>();
            if !filtered.is_empty() {
                attrs.insert("nav".to_string(), AttrValue::Vector(filtered));
            }
        }
        let layout_level_opens = layout_groups
            .iter()
            .find(|group| group.members.contains(&page.page_id))
            .and_then(|group| layout_open_ids.get(&group.layout_id))
            .cloned()
            .unwrap_or_default();
        let filtered_opens = page
            .opens
            .difference(&root_open_ids)
            .filter(|dialog_id| !layout_level_opens.contains(*dialog_id))
            .cloned()
            .collect::<BTreeSet<_>>();
        if !filtered_opens.is_empty() {
            attrs.insert("opens".to_string(), AttrValue::Vector(filtered_opens));
        }
        node.insert(page.page_id.clone(), NodeSpec { attrs });
    }

    for page in &scanned_pages {
        for dialog in &page.dialogs {
            let mut attrs = BTreeMap::new();
            attrs.insert("kind".to_string(), AttrValue::Scalar("dialog".to_string()));
            attrs.insert(
                "dialog-kind".to_string(),
                AttrValue::Scalar(dialog.dialog_kind.clone()),
            );
            attrs.insert("title".to_string(), AttrValue::Scalar(dialog.title.clone()));
            node.insert(dialog.dialog_id.clone(), NodeSpec { attrs });
        }
    }

    let inherit = build_inherit_section(&scanned_pages, &layout_groups);

    Document {
        app: None,
        drill,
        inherit,
        nav,
        node,
        groups: Vec::new(),
    }
}

fn extract_title(html: &Html) -> Option<String> {
    let title_selector = Selector::parse("title").expect("title selector");
    html.select(&title_selector)
        .next()
        .map(|title| collapse_text(title.text().collect::<String>()))
        .filter(|title| !title.is_empty())
}

fn extract_page_location(html: &Html, path: &Path) -> PageLocation {
    if let Some(href) = extract_canonical_href(html) {
        if let Ok(url) = Url::parse(&href) {
            return PageLocation {
                path: normalize_path(url.path()),
                host: url.host_str().map(ToOwned::to_owned),
            };
        }
        return PageLocation {
            path: normalize_relative_href_to_path(&href).unwrap_or_else(|| infer_page_path(path)),
            host: None,
        };
    }
    PageLocation {
        path: infer_page_path(path),
        host: None,
    }
}

fn extract_canonical_href(html: &Html) -> Option<String> {
    let canonical_selector = Selector::parse("link[rel=canonical]").expect("canonical selector");
    if let Some(href) = html
        .select(&canonical_selector)
        .next()
        .and_then(|node| node.value().attr("href"))
    {
        return Some(href.to_string());
    }

    let og_selector =
        Selector::parse("meta[property='og:url'], meta[name='og:url']").expect("og:url selector");
    html.select(&og_selector)
        .next()
        .and_then(|node| node.value().attr("content"))
        .map(ToOwned::to_owned)
}

fn infer_title_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Page");
    stem.replace(['-', '_', '.'], " ")
}

fn infer_page_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("page");
    if stem.eq_ignore_ascii_case("index")
        || stem.eq_ignore_ascii_case("home")
        || stem.ends_with("-home")
        || stem.ends_with("_home")
    {
        "/".to_string()
    } else {
        format!("/{}", stem.replace([' ', '_'], "-").to_ascii_lowercase())
    }
}

fn extract_nav_candidates(
    html: &Html,
    page_host: Option<&str>,
    page_path: &str,
) -> Vec<NavCandidate> {
    let selectors = [
        "nav",
        "[role='navigation']",
        "[role='tablist']",
        "ul[id*='nav']",
        "ol[id*='nav']",
        "header",
        "footer",
    ];
    let mut candidates = Vec::new();
    for selector_text in selectors {
        let selector = Selector::parse(selector_text).expect("selector");
        let link_selector = Selector::parse("a[href]").expect("anchor selector");
        for container in html.select(&selector) {
            if matches!(selector_text, "header" | "footer")
                && contains_nested_nav_container(&container)
            {
                continue;
            }
            let mut targets = BTreeMap::new();
            for link in container.select(&link_selector) {
                let Some(href) = link.value().attr("href") else {
                    continue;
                };
                let Some(path) = normalize_internal_href_to_path(href, page_host) else {
                    continue;
                };
                let title = extract_link_title(&link, &path);
                if !is_probably_page_target(&path, &title, &link) {
                    continue;
                }
                targets
                    .entry(path.clone())
                    .or_insert(ScannedTarget { path, title });
            }
            if targets.len() < 2 {
                continue;
            }
            let meta = NavCandidateMeta {
                selector: selector_text.to_string(),
            };
            let candidate = NavCandidate {
                label: extract_container_label(&container),
                targets: targets.into_values().collect(),
            };
            if !candidate.targets.is_empty()
                && !should_skip_nav_candidate(&candidate, &meta, page_path)
            {
                candidates.push(candidate);
            }
        }
    }
    candidates.sort_by(|left, right| {
        left.targets
            .iter()
            .map(|target| target.path.as_str())
            .collect::<Vec<_>>()
            .cmp(
                &right
                    .targets
                    .iter()
                    .map(|target| target.path.as_str())
                    .collect::<Vec<_>>(),
            )
    });
    candidates.dedup_by(|left, right| {
        left.targets
            .iter()
            .map(|target| target.path.as_str())
            .eq(right.targets.iter().map(|target| target.path.as_str()))
    });
    candidates
}

fn extract_breadcrumb_paths(html: &Html, page_host: Option<&str>) -> Vec<String> {
    let selectors = [
        "nav[aria-label*='breadcrumb' i] a[href]",
        "[role='navigation'][aria-label*='breadcrumb' i] a[href]",
        "ol.breadcrumb a[href]",
        "ul.breadcrumb a[href]",
        "[data-testid*='breadcrumb' i] a[href]",
    ];
    for selector_text in selectors {
        let selector = Selector::parse(selector_text).expect("breadcrumb selector");
        let mut paths = Vec::new();
        for link in html.select(&selector) {
            let Some(href) = link.value().attr("href") else {
                continue;
            };
            let Some(path) = normalize_internal_href_to_path(href, page_host) else {
                continue;
            };
            if paths.last() != Some(&path) {
                paths.push(path);
            }
        }
        if !paths.is_empty() {
            return paths;
        }
    }
    Vec::new()
}

fn normalize_internal_href_to_path(href: &str, page_host: Option<&str>) -> Option<String> {
    let trimmed = href.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with("javascript:")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("tel:")
    {
        return None;
    }

    if let Ok(url) = Url::parse(trimmed) {
        let Some(page_host) = page_host else {
            return None;
        };
        if url.host_str() != Some(page_host) {
            return None;
        }
        return Some(normalize_path(url.path()));
    }

    normalize_relative_href_to_path(trimmed)
}

fn normalize_relative_href_to_path(href: &str) -> Option<String> {
    let path = href.split('#').next().unwrap_or(href);
    let path = path.split('?').next().unwrap_or(path).trim();
    if path.is_empty() || path == "." || path.starts_with("//") {
        return None;
    }
    Some(if path.starts_with('/') {
        normalize_path(path)
    } else {
        normalize_path(&format!("/{path}"))
    })
}

fn extract_container_label(element: &scraper::ElementRef<'_>) -> Option<String> {
    for attr in ["aria-label", "id", "data-testid"] {
        if let Some(value) = element
            .value()
            .attr(attr)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(value.to_string());
        }
    }
    element
        .value()
        .attr("class")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn extract_link_title(link: &scraper::ElementRef<'_>, path: &str) -> String {
    let text = collapse_text(link.text().collect::<String>());
    if !text.is_empty() {
        return text;
    }
    for attr in ["aria-label", "title", "alt"] {
        if let Some(value) = link
            .value()
            .attr(attr)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return value.to_string();
        }
    }
    infer_title_from_route(path)
}

fn contains_nested_nav_container(element: &scraper::ElementRef<'_>) -> bool {
    let selector =
        Selector::parse("nav, [role='navigation'], [role='tablist'], ul[id*='nav'], ol[id*='nav']")
            .expect("nested nav selector");
    element.select(&selector).next().is_some()
}

fn infer_title_from_route(path: &str) -> String {
    if path == "/" {
        return "Home".to_string();
    }
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.replace(['-', '_', '.'], " "))
        .map(|segment| {
            let mut chars = segment.chars();
            let mut out = String::new();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
            }
            out.push_str(chars.as_str());
            out
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_probably_page_target(path: &str, title: &str, link: &scraper::ElementRef<'_>) -> bool {
    if !is_probably_page_path(path) {
        return false;
    }

    let lowered_title = title.to_ascii_lowercase();
    let action_labels = [
        "login",
        "log in",
        "logout",
        "log out",
        "sign in",
        "sign up",
        "register",
        "cart",
        "checkout",
        "purchase",
        "buy now",
        "add to cart",
        "ログイン",
        "ログアウト",
        "会員登録",
        "新規取得",
        "カート",
        "購入",
        "注文する",
    ];
    if action_labels
        .iter()
        .any(|label| lowered_title.contains(&label.to_ascii_lowercase()))
    {
        return false;
    }

    let rel = link
        .value()
        .attr("rel")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if rel.contains("nofollow") {
        let utility_labels = ["detail", "details", "詳", "取得", "応募", "claim"];
        if utility_labels
            .iter()
            .any(|label| lowered_title.contains(&label.to_ascii_lowercase()))
        {
            return false;
        }
    }

    true
}

fn is_probably_page_path(path: &str) -> bool {
    let lowered = path.to_ascii_lowercase();
    let blocked_fragments = [
        "/login",
        "/logout",
        "/signup",
        "/signin",
        "/register",
        "/registration",
        "/cgi-bin/",
        "/cart",
        "/checkout",
        "/purchase",
        "/order/",
        "/basket",
        "/issues/new/choose",
        "/profile/",
        "/channel/",
    ];
    if blocked_fragments
        .iter()
        .any(|fragment| lowered.contains(fragment))
    {
        return false;
    }
    true
}

fn should_skip_nav_candidate(
    candidate: &NavCandidate,
    meta: &NavCandidateMeta,
    page_path: &str,
) -> bool {
    if candidate.targets.len() < 2 {
        return true;
    }
    if is_locale_switcher_candidate(candidate, meta) {
        return true;
    }
    if is_footer_directory_candidate(candidate, meta) {
        return true;
    }
    if is_docs_index_candidate(candidate, page_path) {
        return true;
    }
    false
}

fn is_locale_switcher_candidate(candidate: &NavCandidate, meta: &NavCandidateMeta) -> bool {
    let label = candidate
        .label
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if label.contains("locale") || label.contains("language") || label.contains("lang") {
        return true;
    }
    let locale_titles = candidate
        .targets
        .iter()
        .filter(|target| is_locale_like_title(&target.title))
        .count();
    locale_titles >= 3 && locale_titles * 2 >= candidate.targets.len()
        || meta.selector == "footer" && locale_titles >= 2 && candidate.targets.len() >= 4
}

fn is_locale_like_title(title: &str) -> bool {
    let lowered = title.trim().to_ascii_lowercase();
    matches!(
        lowered.as_str(),
        "english"
            | "deutsch"
            | "français"
            | "francais"
            | "español"
            | "espanol"
            | "italiano"
            | "nederlands"
            | "português"
            | "portugues"
            | "svenska"
            | "日本"
            | "日本語"
            | "thai"
            | "ไทย"
    )
}

fn is_footer_directory_candidate(candidate: &NavCandidate, meta: &NavCandidateMeta) -> bool {
    meta.selector == "footer" && candidate.targets.len() >= 16
}

fn is_docs_index_candidate(candidate: &NavCandidate, page_path: &str) -> bool {
    if candidate.targets.len() < 30 {
        return false;
    }
    let segments = split_path_segments(page_path);
    if segments.len() < 2 {
        return false;
    }
    let prefixes = (2..=segments.len())
        .map(|len| format!("/{}", segments[..len].join("/")))
        .collect::<Vec<_>>();
    prefixes.into_iter().any(|prefix| {
        let matching = candidate
            .targets
            .iter()
            .filter(|target| target.path.starts_with(&prefix))
            .count();
        matching * 4 >= candidate.targets.len() * 3
    })
}

fn has_large_docs_index_candidate(html: &Html, page_host: Option<&str>, page_path: &str) -> bool {
    let selectors = [
        "nav",
        "[role='navigation']",
        "[role='tablist']",
        "ul[id*='nav']",
        "ol[id*='nav']",
    ];
    let link_selector = Selector::parse("a[href]").expect("anchor selector");
    for selector_text in selectors {
        let selector = Selector::parse(selector_text).expect("selector");
        for container in html.select(&selector) {
            let mut targets = BTreeMap::new();
            for link in container.select(&link_selector) {
                let Some(href) = link.value().attr("href") else {
                    continue;
                };
                let Some(path) = normalize_internal_href_to_path(href, page_host) else {
                    continue;
                };
                let title = extract_link_title(&link, &path);
                targets
                    .entry(path.clone())
                    .or_insert(ScannedTarget { path, title });
            }
            let candidate = NavCandidate {
                label: extract_container_label(&container),
                targets: targets.into_values().collect(),
            };
            if is_docs_index_candidate(&candidate, page_path) {
                return true;
            }
        }
    }
    false
}

fn extract_dialogs(html: &Html, page_title: &str) -> Vec<ScannedDialog> {
    let selectors = [
        "dialog",
        "[role='dialog']",
        "[role='alertdialog']",
        "[aria-modal='true']",
    ];
    let heading_selector =
        Selector::parse("h1, h2, h3, [aria-label], [title]").expect("dialog heading selector");
    let mut dialogs = Vec::new();
    let mut used_ids = BTreeSet::new();
    for selector_text in selectors {
        let selector = Selector::parse(selector_text).expect("dialog selector");
        for element in html.select(&selector) {
            let title = extract_dialog_title(&element, &heading_selector, page_title);
            let dom_id = element.value().attr("id").map(ToOwned::to_owned);
            let base = element
                .value()
                .attr("id")
                .map(|id| make_identifier(id, ""))
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| make_identifier(&title, "Dialog"));
            let dialog_kind = classify_dialog_kind(&element, dom_id.as_deref(), &title);
            let mut dialog_id = if base.ends_with("Dialog") {
                base
            } else {
                format!("{base}Dialog")
            };
            if dialog_id.is_empty() {
                dialog_id = "Dialog".to_string();
            }
            if !used_ids.insert(dialog_id.clone()) {
                for index in 2.. {
                    let candidate = format!("{dialog_id}{index}");
                    if used_ids.insert(candidate.clone()) {
                        dialog_id = candidate;
                        break;
                    }
                }
            }
            dialogs.push(ScannedDialog {
                dialog_id,
                dom_id,
                dialog_kind,
                title,
            });
        }
    }
    dialogs
}

fn classify_dialog_kind(
    element: &scraper::ElementRef<'_>,
    dom_id: Option<&str>,
    title: &str,
) -> String {
    let form_selector =
        Selector::parse("form, input, textarea, select").expect("dialog form selector");
    if element.select(&form_selector).next().is_some() {
        return "form".to_string();
    }
    let lowered = format!(
        "{} {}",
        dom_id.unwrap_or_default().to_ascii_lowercase(),
        title.to_ascii_lowercase()
    );
    if lowered.contains("confirm") || lowered.contains("確認") || lowered.contains("delete") {
        return "confirm".to_string();
    }
    if lowered.contains("alert") || lowered.contains("warning") || lowered.contains("error") {
        return "alert".to_string();
    }
    if lowered.contains("cookie") || lowered.contains("consent") || lowered.contains("privacy") {
        return "consent".to_string();
    }
    if lowered.contains("menu") || lowered.contains("drawer") || lowered.contains("sheet") {
        return "sheet".to_string();
    }
    if lowered.contains("picker") || lowered.contains("select") || lowered.contains("choose") {
        return "picker".to_string();
    }
    if lowered.contains("promo") || lowered.contains("campaign") || lowered.contains("coupon") {
        return "promo".to_string();
    }
    "generic".to_string()
}

fn extract_dialog_title(
    element: &scraper::ElementRef<'_>,
    heading_selector: &Selector,
    page_title: &str,
) -> String {
    for attr in ["aria-label", "title"] {
        if let Some(value) = element
            .value()
            .attr(attr)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return value.to_string();
        }
    }
    if let Some(heading) = element.select(heading_selector).next() {
        let heading_text = collapse_text(heading.text().collect::<String>());
        if !heading_text.is_empty() {
            return heading_text;
        }
        for attr in ["aria-label", "title"] {
            if let Some(value) = heading
                .value()
                .attr(attr)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                return value.to_string();
            }
        }
    }
    let text = collapse_text(element.text().collect::<String>());
    if !text.is_empty() {
        return text
            .split_whitespace()
            .take(6)
            .collect::<Vec<_>>()
            .join(" ");
    }
    format!("{page_title} dialog")
}

fn extract_dialog_opens(html: &Html, dialogs: &[ScannedDialog]) -> BTreeSet<String> {
    if dialogs.is_empty() {
        return BTreeSet::new();
    }
    let dialog_ids_by_dom_id = build_dialog_dom_id_map(html, dialogs);
    if dialog_ids_by_dom_id.is_empty() {
        return BTreeSet::new();
    }
    let selector = Selector::parse(
        "button[aria-controls], a[aria-controls], [aria-haspopup='dialog'][aria-controls], a[href^='#'], [data-dialog], [data-dialog-target], [data-modal-target]",
    )
    .expect("dialog trigger selector");
    let mut opens = BTreeSet::new();
    for trigger in html.select(&selector) {
        for attr in [
            "aria-controls",
            "data-dialog",
            "data-dialog-target",
            "data-modal-target",
        ] {
            if let Some(target) = trigger.value().attr(attr) {
                if let Some(dialog_id) =
                    resolve_dialog_id_from_target(target, &dialog_ids_by_dom_id)
                {
                    opens.insert(dialog_id);
                }
            }
        }
        if let Some(href) = trigger.value().attr("href") {
            if let Some(dialog_id) = resolve_dialog_id_from_target(href, &dialog_ids_by_dom_id) {
                opens.insert(dialog_id);
            }
        }
    }
    opens
}

fn build_dialog_dom_id_map(_html: &Html, dialogs: &[ScannedDialog]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for dialog in dialogs {
        if let Some(dom_id) = &dialog.dom_id {
            map.insert(dom_id.clone(), dialog.dialog_id.clone());
        }
    }
    map
}

fn infer_root_layout_dialogs(
    pages: &[ScannedPage],
    input_page_ids: &BTreeSet<String>,
) -> BTreeSet<String> {
    let input_pages = pages
        .iter()
        .filter(|page| input_page_ids.contains(&page.page_id))
        .collect::<Vec<_>>();
    if input_pages.len() < 2 {
        return BTreeSet::new();
    }
    let mut iter = input_pages.into_iter().map(|page| page.opens.clone());
    let Some(mut shared) = iter.next() else {
        return BTreeSet::new();
    };
    for opens in iter {
        shared = shared.intersection(&opens).cloned().collect();
    }
    shared
}

fn infer_layout_dialogs(
    pages: &[ScannedPage],
    layout_groups: &[LayoutGroup],
    input_page_ids: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let page_map = pages
        .iter()
        .map(|page| (page.page_id.clone(), page))
        .collect::<BTreeMap<_, _>>();
    let mut out = BTreeMap::new();
    for group in layout_groups {
        let members = group
            .members
            .iter()
            .filter(|page_id| input_page_ids.contains(*page_id))
            .filter_map(|page_id| page_map.get(page_id).map(|page| page.opens.clone()))
            .collect::<Vec<_>>();
        if members.len() < 2 {
            continue;
        }
        let mut shared = members[0].clone();
        for opens in members.iter().skip(1) {
            shared = shared.intersection(opens).cloned().collect();
        }
        if !shared.is_empty() {
            out.insert(group.layout_id.clone(), shared);
        }
    }
    out
}

fn resolve_dialog_id_from_target(
    target: &str,
    dialog_ids_by_dom_id: &BTreeMap<String, String>,
) -> Option<String> {
    let cleaned = target
        .trim()
        .trim_start_matches('#')
        .trim_start_matches('[')
        .trim_end_matches(']');
    if cleaned.is_empty() {
        return None;
    }
    dialog_ids_by_dom_id.get(cleaned).cloned()
}

fn infer_node_kind(page: &ScannedPage, section_page_ids: &BTreeSet<String>) -> &'static str {
    if section_page_ids.contains(&page.page_id) {
        return "section";
    }
    if is_probably_action_node(&page.path, &page.title) {
        return "action";
    }
    if page.has_docs_index_candidate {
        return "index";
    }
    "page"
}

fn is_probably_action_node(path: &str, title: &str) -> bool {
    let lowered_path = path.to_ascii_lowercase();
    let lowered_title = title.to_ascii_lowercase();
    let action_fragments = [
        "/contact/sales",
        "/contact",
        "/demo",
        "/start",
        "/signup",
        "/register",
    ];
    if action_fragments
        .iter()
        .any(|fragment| lowered_path.contains(fragment))
    {
        return true;
    }
    let action_titles = [
        "contact sales",
        "contact us",
        "get started",
        "start now",
        "book a demo",
        "営業にお問い合わせ",
        "お問い合わせ",
        "今すぐ始める",
    ];
    action_titles
        .iter()
        .any(|fragment| lowered_title.contains(&fragment.to_ascii_lowercase()))
}

fn suggest_nav_id(label: &str) -> Option<String> {
    let lowered = label.to_ascii_lowercase();
    if lowered.contains("footer") {
        Some("FooterNav".to_string())
    } else if lowered.contains("account")
        || lowered.contains("mypage")
        || lowered.contains("my-page")
    {
        Some("AccountNav".to_string())
    } else if lowered.contains("category") || lowered.contains("tab") {
        Some("CategoryNav".to_string())
    } else if lowered.contains("header") || lowered.contains("global") {
        Some("GlobalNav".to_string())
    } else {
        None
    }
}

fn classify_nav_cluster(
    cluster: &NavCluster,
    input_page_count: usize,
    target_count: usize,
) -> Option<String> {
    if let Some(label_based) = cluster.label.as_deref().and_then(suggest_nav_id) {
        return Some(label_based);
    }

    let category_like = cluster
        .paths
        .iter()
        .filter(|path| {
            path == &&"/" || path.starts_with("/category/") || path.starts_with("/promotion/event/")
        })
        .count();
    let account_like = cluster
        .paths
        .iter()
        .filter(|path| path.starts_with("/my"))
        .count();

    if account_like == cluster.paths.len() && account_like >= 2 {
        return Some("AccountNav".to_string());
    }
    if category_like == cluster.paths.len() && category_like >= 2 {
        return Some("CategoryNav".to_string());
    }
    if cluster.owners.len() == input_page_count && input_page_count > 1 {
        return Some("GlobalNav".to_string());
    }
    if input_page_count == 1 && target_count >= 4 {
        return Some("GlobalNav".to_string());
    }
    None
}

fn should_attach_nav_to_root_layout(
    nav_id: &str,
    cluster: &NavCluster,
    input_page_count: usize,
    target_count: usize,
) -> bool {
    if nav_id == "FooterNav" || nav_id == "AccountNav" {
        return false;
    }
    if cluster.owners.len() == input_page_count && input_page_count > 0 {
        return true;
    }
    input_page_count == 1 && target_count >= 4
}

fn allocate_nav_id(
    preferred: Option<String>,
    nav: &BTreeMap<String, BTreeSet<String>>,
    next_nav_idx: &mut usize,
) -> String {
    if let Some(preferred) = preferred {
        if !nav.contains_key(&preferred) {
            return preferred;
        }
    }
    loop {
        let candidate = format!("Nav{next_nav_idx}");
        *next_nav_idx += 1;
        if !nav.contains_key(&candidate) {
            return candidate;
        }
    }
}

fn prune_redundant_nav_clusters(
    nav_clusters: BTreeMap<BTreeSet<String>, NavCluster>,
) -> Vec<(BTreeSet<String>, NavCluster)> {
    let clusters = nav_clusters.into_iter().collect::<Vec<_>>();
    let mut keep = vec![true; clusters.len()];
    for i in 0..clusters.len() {
        for j in 0..clusters.len() {
            if i == j || !keep[i] {
                continue;
            }
            let (targets_i, cluster_i) = &clusters[i];
            let (targets_j, cluster_j) = &clusters[j];
            if targets_i.len() >= targets_j.len() {
                continue;
            }
            if cluster_i.owners != cluster_j.owners {
                continue;
            }
            let intersection = targets_i.intersection(targets_j).count();
            if intersection == targets_i.len() && intersection * 100 >= targets_j.len() * 75 {
                keep[i] = false;
            }
        }
    }
    clusters
        .into_iter()
        .enumerate()
        .filter_map(|(idx, cluster)| keep[idx].then_some(cluster))
        .collect()
}

fn unique_page_id(title: &str, path: &str, used_ids: &mut BTreeSet<String>) -> String {
    let base = make_identifier_for_page(title, path, "Page");
    if used_ids.insert(base.clone()) {
        return base;
    }
    for index in 2.. {
        let candidate = format!("{base}{index}");
        if used_ids.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!()
}

fn make_identifier_for_page(input: &str, path: &str, fallback: &str) -> String {
    let from_title = make_identifier(input, "");
    if !from_title.is_empty() {
        return from_title;
    }
    let from_path = make_identifier(&infer_title_from_route(path), "");
    if !from_path.is_empty() {
        return from_path;
    }
    fallback.to_string()
}

fn infer_layout_groups(
    pages: &[ScannedPage],
    nav: &BTreeMap<String, BTreeSet<String>>,
    root_nav_ids: &BTreeSet<String>,
) -> Vec<LayoutGroup> {
    let mut groups = Vec::new();
    let mut claimed_pages = BTreeSet::new();

    for (nav_id, targets) in nav {
        if root_nav_ids.contains(nav_id) || targets.len() < 2 {
            continue;
        }
        claimed_pages.extend(targets.iter().cloned());
        groups.push(LayoutGroup {
            layout_id: infer_layout_id_for_nav(nav_id),
            members: targets.clone(),
        });
    }

    let mut members_by_segment = BTreeMap::<String, BTreeSet<String>>::new();
    for page in pages {
        if claimed_pages.contains(&page.page_id) {
            continue;
        }
        let segments = split_path_segments(&page.path);
        if let Some(first) = segments.first() {
            members_by_segment
                .entry((*first).to_string())
                .or_default()
                .insert(page.page_id.clone());
        }
    }

    groups.extend(
        members_by_segment
            .into_iter()
            .filter(|(_segment, members)| members.len() >= 2)
            .map(|(segment, members)| LayoutGroup {
                layout_id: infer_layout_id_for_segment(&segment),
                members,
            }),
    );

    groups
}

fn infer_layout_id_for_nav(nav_id: &str) -> String {
    if let Some(base) = nav_id.strip_suffix("Nav") {
        format!("{base}Layout")
    } else {
        format!("{nav_id}Layout")
    }
}

fn infer_layout_id_for_segment(segment: &str) -> String {
    match segment {
        "my" => "AccountLayout".to_string(),
        "category" => "CategoryLayout".to_string(),
        other => {
            let mut id = make_identifier(&infer_title_from_route(&format!("/{other}")), "Layout");
            if id.is_empty() {
                id = "SectionLayout".to_string();
            }
            if !id.ends_with("Layout") {
                id.push_str("Layout");
            }
            id
        }
    }
}

fn build_inherit_section(pages: &[ScannedPage], layout_groups: &[LayoutGroup]) -> TreeSection {
    let mut claimed_pages = BTreeSet::new();
    let mut children = Vec::new();

    for group in layout_groups {
        let mut members = group.members.iter().cloned().collect::<Vec<_>>();
        members.sort();
        claimed_pages.extend(members.iter().cloned());
        children.push(TreeChild::Branch(
            group.layout_id.clone(),
            members.into_iter().map(TreeChild::Leaf).collect(),
        ));
    }

    let mut root_pages = pages
        .iter()
        .map(|page| page.page_id.clone())
        .filter(|page_id| !claimed_pages.contains(page_id))
        .collect::<Vec<_>>();
    root_pages.sort();
    children.extend(root_pages.into_iter().map(TreeChild::Leaf));

    BTreeMap::from([("RootLayout".to_string(), children)])
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/".to_string();
    }
    let trimmed = trimmed.trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

fn collapse_text(text: String) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn make_identifier(input: &str, fallback: &str) -> String {
    let parts = input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let mut out = String::new();
            if let Some(first) = chars.next() {
                out.push(first.to_ascii_uppercase());
            }
            out.push_str(&chars.as_str().to_ascii_lowercase());
            out
        })
        .collect::<String>();
    if parts.is_empty() {
        fallback.to_string()
    } else if parts.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        format!("N{parts}")
    } else {
        parts
    }
}

fn build_drill_section(pages: &[ScannedPage]) -> TreeSection {
    let page_id_by_path = pages
        .iter()
        .map(|page| (page.path.clone(), page.page_id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut parent_by_id = BTreeMap::<String, Option<String>>::new();
    for page in pages {
        let segments = split_path_segments(&page.path);
        let parent = infer_drill_parent(
            &page.path,
            &segments,
            &page.breadcrumb_paths,
            &page_id_by_path,
        )
        .filter(|parent_id| parent_id != &page.page_id);
        parent_by_id.insert(page.page_id.clone(), parent);
    }

    let mut children_by_parent = BTreeMap::<String, Vec<String>>::new();
    let mut roots = Vec::new();
    for page in pages {
        match parent_by_id
            .get(&page.page_id)
            .and_then(|parent| parent.clone())
        {
            Some(parent) => children_by_parent
                .entry(parent)
                .or_default()
                .push(page.page_id.clone()),
            None => roots.push(page.page_id.clone()),
        }
    }

    let pages_by_id = pages
        .iter()
        .map(|page| (page.page_id.clone(), page))
        .collect::<BTreeMap<_, _>>();

    roots.sort_by_key(|page_id| pages_by_id.get(page_id).map(|page| page.path.clone()));
    let mut section = BTreeMap::new();
    for root in roots {
        section.insert(
            root.clone(),
            build_drill_children(&root, &children_by_parent, &pages_by_id),
        );
    }
    section
}

fn infer_drill_parent(
    path: &str,
    segments: &[&str],
    breadcrumb_paths: &[String],
    page_id_by_path: &BTreeMap<String, String>,
) -> Option<String> {
    if path == "/" || segments.is_empty() {
        return None;
    }

    if let Some(parent_id) = infer_breadcrumb_parent(path, breadcrumb_paths, page_id_by_path) {
        return Some(parent_id);
    }

    if segments.len() >= 2 {
        let direct_parent_path = format!("/{}", segments[..segments.len() - 1].join("/"));
        if let Some(parent_id) = page_id_by_path.get(&direct_parent_path) {
            return Some(parent_id.clone());
        }
    }

    let section_root_path = format!("/{}", segments[0]);
    if let Some(parent_id) = page_id_by_path.get(&section_root_path) {
        return Some(parent_id.clone());
    }

    None
}

fn infer_breadcrumb_parent(
    path: &str,
    breadcrumb_paths: &[String],
    page_id_by_path: &BTreeMap<String, String>,
) -> Option<String> {
    let current_index = breadcrumb_paths.iter().position(|item| item == path)?;
    if current_index == 0 {
        return None;
    }
    breadcrumb_paths[..current_index]
        .iter()
        .rev()
        .find_map(|parent_path| page_id_by_path.get(parent_path).cloned())
}

fn build_drill_children(
    page_id: &str,
    children_by_parent: &BTreeMap<String, Vec<String>>,
    pages_by_id: &BTreeMap<String, &ScannedPage>,
) -> Vec<TreeChild> {
    let mut children = children_by_parent.get(page_id).cloned().unwrap_or_default();
    children.sort_by_key(|child| pages_by_id.get(child).map(|page| page.path.clone()));
    children
        .into_iter()
        .map(|child| {
            let nested = build_drill_children(&child, children_by_parent, pages_by_id);
            if nested.is_empty() {
                TreeChild::Leaf(child)
            } else {
                TreeChild::Branch(child, nested)
            }
        })
        .collect()
}

fn collect_section_page_ids(section: &TreeSection) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (root, children) in section {
        if !children.is_empty() {
            out.insert(root.clone());
        }
        collect_section_page_ids_from_children(children, &mut out);
    }
    out
}

fn collect_section_page_ids_from_children(children: &[TreeChild], out: &mut BTreeSet<String>) {
    for child in children {
        if let TreeChild::Branch(id, nested) = child {
            out.insert(id.clone());
            collect_section_page_ids_from_children(nested, out);
        }
    }
}

fn split_path_segments(path: &str) -> Vec<&str> {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn render_tree_section(out: &mut String, section_name: &str, section: &TreeSection) {
    out.push_str(section_name);
    out.push_str(":\n");
    for (root, children) in section {
        render_tree_entry(out, 1, root, children);
    }
}

fn render_tree_entry(out: &mut String, depth: usize, node_id: &str, children: &[TreeChild]) {
    let indent = "  ".repeat(depth);
    out.push_str(&indent);
    out.push_str(node_id);
    out.push_str(":\n");
    for child in children {
        match child {
            TreeChild::Leaf(id) => render_tree_entry(out, depth + 1, id, &[]),
            TreeChild::Branch(id, nested) => render_tree_entry(out, depth + 1, id, nested),
        }
    }
}

fn render_nav_section(out: &mut String, nav: &BTreeMap<String, BTreeSet<String>>) {
    if nav.is_empty() {
        out.push_str("nav: {}\n");
        return;
    }
    out.push_str("nav:\n");
    for (nav_id, targets) in nav {
        out.push_str("  ");
        out.push_str(nav_id);
        out.push_str(":\n");
        for target in targets {
            out.push_str("    - ");
            out.push_str(target);
            out.push('\n');
        }
    }
}

fn render_node_section(out: &mut String, node: &BTreeMap<String, NodeSpec>) {
    out.push_str("node:\n");
    for (node_id, spec) in node {
        out.push_str("  ");
        out.push_str(node_id);
        out.push_str(":\n");
        for (attr_name, attr_value) in &spec.attrs {
            out.push_str("    ");
            out.push_str(attr_name);
            out.push_str(": ");
            match attr_value {
                AttrValue::Scalar(value) => out.push_str(&yaml_scalar(value)),
                AttrValue::Vector(values) => {
                    out.push('[');
                    out.push_str(&values.iter().cloned().collect::<Vec<_>>().join(", "));
                    out.push(']');
                }
            }
            out.push('\n');
        }
    }
}

fn render_groups_section(out: &mut String, groups: &[GroupSpec]) {
    out.push_str("groups:\n");
    for group in groups {
        out.push_str("  - id: ");
        out.push_str(&yaml_scalar(&group.id));
        out.push('\n');
        out.push_str("    members: [");
        out.push_str(&group.members.iter().cloned().collect::<Vec<_>>().join(", "));
        out.push_str("]\n");
    }
}

fn yaml_scalar(value: &str) -> String {
    serde_yaml::to_string(&Value::String(value.to_string()))
        .expect("yaml string")
        .trim()
        .to_string()
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
        if let Some(AttrValue::Vector(dialog_ids)) = spec.attrs.get("opens") {
            for dialog_id in dialog_ids {
                let Some(dialog_spec) = doc.node.get(dialog_id) else {
                    errors.push(ValidationError::new(format!(
                        "node `{node_id}` references unknown dialog `{dialog_id}`"
                    )));
                    continue;
                };
                let dialog_kind = match dialog_spec.attrs.get("kind") {
                    Some(AttrValue::Scalar(kind)) => kind.as_str(),
                    _ => "",
                };
                if dialog_kind != "dialog" {
                    errors.push(ValidationError::new(format!(
                        "node `{node_id}` opens target `{dialog_id}` must be a dialog"
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
        collections::{BTreeMap, BTreeSet},
        fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        build_drill_section, infer_layout_groups, infer_page_path, load_document_from_path,
        load_documents_from_paths, parse_document, prune_redundant_nav_clusters, render_document,
        scan_html_paths, validate_document, AttrValue, NavCluster, ScannedPage, TreeChild,
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

    #[test]
    fn scans_html_pages_into_gui_document() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let home = dir.join("home.html");
        let products = dir.join("products.html");
        fs::write(
            &home,
            r#"<!doctype html>
<html>
  <head>
    <title>Home</title>
    <link rel="canonical" href="https://example.test/" />
  </head>
  <body>
    <header>
      <nav>
        <a href="/">Home</a>
        <a href="/products">Products</a>
      </nav>
    </header>
    <main><h1>Home</h1></main>
  </body>
</html>"#,
        )
        .expect("write home");
        fs::write(
            &products,
            r#"<!doctype html>
<html>
  <head>
    <title>Products</title>
    <link rel="canonical" href="https://example.test/products" />
  </head>
  <body>
    <header>
      <nav>
        <a href="/">Home</a>
        <a href="/products">Products</a>
      </nav>
    </header>
    <main><h1>Products</h1></main>
  </body>
</html>"#,
        )
        .expect("write products");

        let doc = scan_html_paths([&home, &products]).expect("scan");
        validate_document(&doc).expect("validate scan result");
        assert!(doc.nav.contains_key("GlobalNav"));
        assert!(doc.node.contains_key("Home"));
        assert!(doc.node.contains_key("Products"));

        let rendered = render_document(&doc);
        let reparsed = parse_document(&rendered).expect("reparse rendered");
        validate_document(&reparsed).expect("revalidate rendered");
    }

    #[test]
    fn scans_single_complex_html_into_page_stubs_and_nav() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-complex-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let home = dir.join("complex-home.html");
        fs::write(
            &home,
            r#"<!doctype html>
<html>
  <head>
    <title>Example Shop</title>
    <link rel="canonical" href="https://shop.example.test/" />
  </head>
  <body>
    <header>
      <div role="tablist" aria-label="category tabs">
        <a href="https://shop.example.test/">Home</a>
        <a href="https://shop.example.test/category/fashion">Fashion</a>
        <a href="https://shop.example.test/category/cosme">Cosme</a>
        <a href="https://shop.example.test/ranking">Ranking</a>
      </div>
      <div>
        <a href="https://external.example.test/help">Help</a>
      </div>
    </header>
    <main><h1>Example Shop</h1></main>
  </body>
</html>"#,
        )
        .expect("write home");

        let doc = scan_html_paths([&home]).expect("scan");
        validate_document(&doc).expect("validate scan result");
        assert!(doc.nav.contains_key("CategoryNav"));
        assert!(doc.node.contains_key("ExampleShop"));
        assert!(doc.node.contains_key("Fashion"));
        assert!(doc.node.contains_key("Cosme"));
        assert!(doc.node.contains_key("Ranking"));
        assert_eq!(doc.nav["CategoryNav"].len(), 4);
        assert!(!doc.node["ExampleShop"].attrs.contains_key("nav"));
    }

    #[test]
    fn drill_prefers_direct_parent_then_section_root() {
        let home = ScannedPage {
            page_id: "Home".to_string(),
            title: "Home".to_string(),
            path: "/".to_string(),
            breadcrumb_paths: Vec::new(),
            nav_candidates: Vec::new(),
            has_docs_index_candidate: false,
            dialogs: Vec::new(),
            opens: BTreeSet::new(),
        };
        let docs = ScannedPage {
            page_id: "Docs".to_string(),
            title: "Docs".to_string(),
            path: "/docs".to_string(),
            breadcrumb_paths: Vec::new(),
            nav_candidates: Vec::new(),
            has_docs_index_candidate: false,
            dialogs: Vec::new(),
            opens: BTreeSet::new(),
        };
        let guide = ScannedPage {
            page_id: "Guide".to_string(),
            title: "Guide".to_string(),
            path: "/docs/guide".to_string(),
            breadcrumb_paths: Vec::new(),
            nav_candidates: Vec::new(),
            has_docs_index_candidate: false,
            dialogs: Vec::new(),
            opens: BTreeSet::new(),
        };
        let advanced = ScannedPage {
            page_id: "Advanced".to_string(),
            title: "Advanced".to_string(),
            path: "/docs/category/advanced".to_string(),
            breadcrumb_paths: Vec::new(),
            nav_candidates: Vec::new(),
            has_docs_index_candidate: false,
            dialogs: Vec::new(),
            opens: BTreeSet::new(),
        };

        let drill = build_drill_section(&[home, docs, guide, advanced]);
        let docs_children = drill.get("Docs").expect("docs root");
        assert!(docs_children.contains(&TreeChild::Leaf("Guide".to_string())));
        assert!(docs_children.contains(&TreeChild::Leaf("Advanced".to_string())));
    }

    #[test]
    fn scan_excludes_action_like_links_from_stub_pages() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-action-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let home = dir.join("action-home.html");
        fs::write(
            &home,
            r#"<!doctype html>
<html>
  <head>
    <title>Example Portal</title>
    <link rel="canonical" href="https://portal.example.test/" />
  </head>
  <body>
    <header>
      <nav aria-label="main nav">
        <a href="https://portal.example.test/">Home</a>
        <a href="https://portal.example.test/docs">Docs</a>
        <a href="https://portal.example.test/login" rel="nofollow">ログイン</a>
        <a href="https://portal.example.test/cart" rel="nofollow">カート</a>
      </nav>
    </header>
  </body>
</html>"#,
        )
        .expect("write home");

        let doc = scan_html_paths([&home]).expect("scan");
        validate_document(&doc).expect("validate scan result");
        assert!(doc.node.contains_key("Docs"));
        assert!(!doc.node.contains_key("Login"));
        assert!(!doc.node.contains_key("Cart"));
    }

    #[test]
    fn scan_marks_branch_pages_as_sections() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-section-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let about = dir.join("about.html");
        let apps = dir.join("apps.html");
        fs::write(
            &about,
            r#"<!doctype html>
<html>
  <head>
    <title>About</title>
    <link rel="canonical" href="https://example.test/about" />
  </head>
  <body>
    <nav aria-label="about nav">
      <a href="https://example.test/about">About</a>
      <a href="https://example.test/about/apps">Applications</a>
    </nav>
  </body>
</html>"#,
        )
        .expect("write about");
        fs::write(
            &apps,
            r#"<!doctype html>
<html>
  <head>
    <title>Applications</title>
    <link rel="canonical" href="https://example.test/about/apps" />
  </head>
  <body><main>Applications</main></body>
</html>"#,
        )
        .expect("write apps");

        let doc = scan_html_paths([&about, &apps]).expect("scan");
        validate_document(&doc).expect("validate scan result");
        let about_kind = match doc.node["About"].attrs.get("kind") {
            Some(AttrValue::Scalar(value)) => value.as_str(),
            _ => "",
        };
        let apps_kind = match doc.node["Applications"].attrs.get("kind") {
            Some(AttrValue::Scalar(value)) => value.as_str(),
            _ => "",
        };
        assert_eq!(about_kind, "section");
        assert_eq!(apps_kind, "page");
    }

    #[test]
    fn breadcrumb_parent_beats_path_prefix_parent() {
        let root = ScannedPage {
            page_id: "Docs".to_string(),
            title: "Docs".to_string(),
            path: "/docs".to_string(),
            breadcrumb_paths: Vec::new(),
            nav_candidates: Vec::new(),
            has_docs_index_candidate: false,
            dialogs: Vec::new(),
            opens: BTreeSet::new(),
        };
        let guide = ScannedPage {
            page_id: "Guide".to_string(),
            title: "Guide".to_string(),
            path: "/docs/guide".to_string(),
            breadcrumb_paths: vec!["/docs".to_string(), "/docs/guide".to_string()],
            nav_candidates: Vec::new(),
            has_docs_index_candidate: false,
            dialogs: Vec::new(),
            opens: BTreeSet::new(),
        };
        let faq = ScannedPage {
            page_id: "Faq".to_string(),
            title: "Faq".to_string(),
            path: "/docs/guide/faq".to_string(),
            breadcrumb_paths: vec![
                "/docs".to_string(),
                "/docs/guide".to_string(),
                "/docs/guide/faq".to_string(),
            ],
            nav_candidates: Vec::new(),
            has_docs_index_candidate: false,
            dialogs: Vec::new(),
            opens: BTreeSet::new(),
        };

        let drill = build_drill_section(&[root, guide, faq]);
        let docs_children = drill.get("Docs").expect("docs root");
        assert!(docs_children.contains(&TreeChild::Branch(
            "Guide".to_string(),
            vec![TreeChild::Leaf("Faq".to_string())],
        )));
    }

    #[test]
    fn infer_layout_groups_prefers_non_root_nav_targets() {
        let pages = vec![
            ScannedPage {
                page_id: "Home".to_string(),
                title: "Home".to_string(),
                path: "/".to_string(),
                breadcrumb_paths: Vec::new(),
                nav_candidates: Vec::new(),
                has_docs_index_candidate: false,
                dialogs: Vec::new(),
                opens: BTreeSet::new(),
            },
            ScannedPage {
                page_id: "My".to_string(),
                title: "My".to_string(),
                path: "/my".to_string(),
                breadcrumb_paths: Vec::new(),
                nav_candidates: Vec::new(),
                has_docs_index_candidate: false,
                dialogs: Vec::new(),
                opens: BTreeSet::new(),
            },
            ScannedPage {
                page_id: "MyOrders".to_string(),
                title: "My Orders".to_string(),
                path: "/my/orders".to_string(),
                breadcrumb_paths: Vec::new(),
                nav_candidates: Vec::new(),
                has_docs_index_candidate: false,
                dialogs: Vec::new(),
                opens: BTreeSet::new(),
            },
        ];
        let nav = BTreeMap::from([
            (
                "GlobalNav".to_string(),
                BTreeSet::from(["Home".to_string(), "My".to_string()]),
            ),
            (
                "AccountNav".to_string(),
                BTreeSet::from(["My".to_string(), "MyOrders".to_string()]),
            ),
        ]);
        let root_nav_ids = BTreeSet::from(["GlobalNav".to_string()]);

        let groups = infer_layout_groups(&pages, &nav, &root_nav_ids);
        assert!(groups.iter().any(|group| {
            group.layout_id == "AccountLayout"
                && group.members == BTreeSet::from(["My".to_string(), "MyOrders".to_string()])
        }));
    }

    #[test]
    fn scan_ignores_locale_switcher_and_huge_footer_directory() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-locale-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let home = dir.join("locale-home.html");
        fs::write(
            &home,
            r#"<!doctype html>
<html>
  <head>
    <title>Stripe Like</title>
    <link rel="canonical" href="https://example.test/" />
  </head>
  <body>
    <footer>
      <div class="footer-links">
        <a href="https://example.test/docs">Docs</a>
        <a href="https://example.test/pricing">Pricing</a>
        <a href="https://example.test/about">About</a>
        <a href="https://example.test/blog">Blog</a>
        <a href="https://example.test/customers">Customers</a>
        <a href="https://example.test/partners">Partners</a>
        <a href="https://example.test/startups">Startups</a>
        <a href="https://example.test/enterprise">Enterprise</a>
        <a href="https://example.test/security">Security</a>
        <a href="https://example.test/tax">Tax</a>
        <a href="https://example.test/billing">Billing</a>
        <a href="https://example.test/connect">Connect</a>
        <a href="https://example.test/payments">Payments</a>
        <a href="https://example.test/invoicing">Invoicing</a>
        <a href="https://example.test/atlas">Atlas</a>
        <a href="https://example.test/issuing">Issuing</a>
      </div>
      <div class="locale-switcher" aria-label="locale switcher">
        <a href="https://example.test/en">English</a>
        <a href="https://example.test/de">Deutsch</a>
        <a href="https://example.test/fr">Français</a>
        <a href="https://example.test/ja">日本語</a>
      </div>
    </footer>
  </body>
</html>"#,
        )
        .expect("write home");

        let doc = scan_html_paths([&home]).expect("scan");
        assert!(doc.nav.is_empty());
        assert_eq!(doc.node.len(), 2);
        assert!(doc.node.contains_key("RootLayout"));
        assert!(doc.node.contains_key("StripeLike"));
    }

    #[test]
    fn scan_rejects_absolute_external_links_when_page_host_unknown() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-external-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let home = dir.join("rust-like.html");
        fs::write(
            &home,
            r#"<!doctype html>
<html>
  <head>
    <title>Rust Like</title>
  </head>
  <body>
    <header>
      <nav>
        <a href="/learn">Learn</a>
        <a href="/community">Community</a>
        <a href="https://github.com/example/site/issues/new/choose">File an issue!</a>
        <a href="https://www.youtube.com/channel/UC123">Watch videos</a>
      </nav>
    </header>
  </body>
</html>"#,
        )
        .expect("write home");

        let doc = scan_html_paths([&home]).expect("scan");
        assert!(doc.node.contains_key("Learn"));
        assert!(doc.node.contains_key("Community"));
        assert!(!doc.node.contains_key("FileAnIssue"));
        assert!(!doc.node.contains_key("WatchVideos"));
    }

    #[test]
    fn scan_skips_huge_docs_index_candidate() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-docs-index-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let home = dir.join("docs.html");
        let links = (0..32)
            .map(|idx| format!("<a href=\"https://example.test/en-US/docs/Web/JavaScript/Ref{idx}\">Ref {idx}</a>"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(
            &home,
            format!(
                "<!doctype html><html><head><title>JS</title><link rel=\"canonical\" href=\"https://example.test/en-US/docs/Web/JavaScript\" /></head><body><nav>{links}</nav></body></html>"
            ),
        )
        .expect("write home");

        let doc = scan_html_paths([&home]).expect("scan");
        assert!(doc.nav.is_empty());
        assert_eq!(doc.node.len(), 2);
        assert!(doc.node.contains_key("RootLayout"));
        assert!(doc.node.contains_key("Js"));
        let js_kind = match doc.node["Js"].attrs.get("kind") {
            Some(AttrValue::Scalar(value)) => value.as_str(),
            _ => "",
        };
        assert_eq!(js_kind, "index");
    }

    #[test]
    fn scan_marks_sales_cta_stub_as_action() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-sales-action-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let home = dir.join("pricing-home.html");
        fs::write(
            &home,
            r#"<!doctype html>
<html>
  <head>
    <title>Pricing</title>
    <link rel="canonical" href="https://example.test/pricing" />
  </head>
  <body>
    <header>
      <nav>
        <a href="https://example.test/pricing">Pricing</a>
        <a href="https://example.test/contact/sales">Contact sales</a>
        <a href="https://example.test/docs">Docs</a>
      </nav>
    </header>
  </body>
</html>"#,
        )
        .expect("write home");

        let doc = scan_html_paths([&home]).expect("scan");
        let action_kind = match doc.node["ContactSales"].attrs.get("kind") {
            Some(AttrValue::Scalar(value)) => value.as_str(),
            _ => "",
        };
        assert_eq!(action_kind, "action");
    }

    #[test]
    fn scan_extracts_dialog_nodes_and_opens_relation() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-dialog-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let home = dir.join("dialog-home.html");
        fs::write(
            &home,
            r#"<!doctype html>
<html>
  <head>
    <title>Checkout</title>
    <link rel="canonical" href="https://example.test/checkout" />
  </head>
  <body>
    <button aria-controls="coupon-modal" aria-haspopup="dialog">Open coupon</button>
    <dialog id="coupon-modal" aria-label="Coupon modal">
      <h2>Coupon modal</h2>
      <button>Apply coupon</button>
    </dialog>
  </body>
</html>"#,
        )
        .expect("write home");

        let doc = scan_html_paths([&home]).expect("scan");
        validate_document(&doc).expect("validate scan result");
        let dialog_kind = match doc.node["CouponModalDialog"].attrs.get("kind") {
            Some(AttrValue::Scalar(value)) => value.as_str(),
            _ => "",
        };
        assert_eq!(dialog_kind, "dialog");
        let dialog_subkind = match doc.node["CouponModalDialog"].attrs.get("dialog-kind") {
            Some(AttrValue::Scalar(value)) => value.as_str(),
            _ => "",
        };
        assert_eq!(dialog_subkind, "promo");
        let opens = match doc.node["Checkout"].attrs.get("opens") {
            Some(AttrValue::Vector(values)) => values,
            _ => panic!("missing opens"),
        };
        assert!(opens.contains("CouponModalDialog"));
    }

    #[test]
    fn scan_promotes_shared_dialog_to_layout_opens() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gui-scan-dialog-layout-test-{unique}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let my = dir.join("my.html");
        let orders = dir.join("orders.html");
        let body = r#"
  <body>
    <button aria-controls="cookie-consent" aria-haspopup="dialog">Cookie settings</button>
    <dialog id="cookie-consent" aria-label="Cookie consent">
      <form>
        <button>Save</button>
      </form>
    </dialog>
  </body>
"#;
        fs::write(
            &my,
            format!(
                "<!doctype html><html><head><title>My</title><link rel=\"canonical\" href=\"https://example.test/my\" /></head>{body}</html>"
            ),
        )
        .expect("write my");
        fs::write(
            &orders,
            format!(
                "<!doctype html><html><head><title>My Orders</title><link rel=\"canonical\" href=\"https://example.test/my/orders\" /></head>{body}</html>"
            ),
        )
        .expect("write orders");

        let doc = scan_html_paths([&my, &orders]).expect("scan");
        validate_document(&doc).expect("validate scan result");
        let layout_opens = match doc.node["AccountLayout"].attrs.get("opens") {
            Some(AttrValue::Vector(values)) => values,
            _ => panic!("missing layout opens"),
        };
        assert!(layout_opens.contains("CookieConsentDialog"));
        assert!(!doc.node["My"].attrs.contains_key("opens"));
        assert!(!doc.node["MyOrders"].attrs.contains_key("opens"));
        let dialog_subkind = match doc.node["CookieConsentDialog"].attrs.get("dialog-kind") {
            Some(AttrValue::Scalar(value)) => value.as_str(),
            _ => "",
        };
        assert_eq!(dialog_subkind, "form");
    }

    #[test]
    fn infer_page_path_treats_home_stem_as_root() {
        assert_eq!(infer_page_path(Path::new("rust-home.html")), "/");
        assert_eq!(infer_page_path(Path::new("home.html")), "/");
    }

    #[test]
    fn prune_redundant_nav_clusters_drops_near_subset_duplicate() {
        let mut input = BTreeMap::new();
        input.insert(
            BTreeSet::from([
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
            ]),
            NavCluster {
                owners: BTreeSet::from(["Home".to_string()]),
                label: Some("global".to_string()),
                paths: BTreeSet::new(),
            },
        );
        input.insert(
            BTreeSet::from(["A".to_string(), "B".to_string(), "C".to_string()]),
            NavCluster {
                owners: BTreeSet::from(["Home".to_string()]),
                label: Some("global duplicate".to_string()),
                paths: BTreeSet::new(),
            },
        );

        let pruned = prune_redundant_nav_clusters(input);
        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0].0.len(), 4);
    }
}
