/*
ACID by Alexander Abraham,
a.k.a. "The Black Unicorn", a.k.a. "Angeldust Duke".
Licensed under the MIT license.
*/

use std::fs;
use fs_extra;
use std::env;
use kstring::*;
use cleasy::App;
use regex::Regex;
use std::fs::File;
use std::fs::write;
use liquid::object;
use std::path::Path;
use git2::Repository;
use std::fs::read_dir;
use liquid::ValueView;
use colored::Colorize;
use std::path::PathBuf;
use file_serve::Server;
use chrono::prelude::*;
use std::process::exit;
use liquid::ObjectView;
use std::fs::create_dir;
use fs_extra::dir::copy;
use serde_json::from_str;
use liquid::ParserBuilder;
use fs_extra::dir::move_dir;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use angelmarkup::map_to_aml;
use std::collections::HashMap;
use fs_extra::file::move_file;
use chrono::offset::LocalResult;
use fs_extra::file::CopyOptions;
use extract_frontmatter::Extractor;
use angelmarkup::serialize as aml_serialize;

/// Clones a GitHub repository from "repo" into "target_dir".
fn clone_repo(repo: String, target_dir: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let repo = match Repository::clone(&repo, target_dir) {
        Ok(_x) => result.push(true),
        Err(_e) => result.push(false)
    };
    return result[0];
}

/// Constants for Acid websites.
fn acid_constants() -> HashMap<String, String> {
    let mut constants: HashMap<String, String> = HashMap::new();
    constants.insert(String::from("config_file_path"),String::from("config.aml"));
    constants.insert(String::from("layouts_dir"),String::from("layouts"));
    constants.insert(String::from("posts_dir"),String::from("posts"));
    constants.insert(String::from("assets_dir"),String::from("assets"));
    constants.insert(String::from("pages_dir"),String::from("pages"));
    constants.insert(String::from("index_path"),String::from("index.markdown"));
    constants.insert(String::from("index_output_path"),String::from("index.html"));
    constants.insert(String::from("build_dir"),String::from("build"));
    constants.insert(String::from("post_url_key"),String::from("url"));
    constants.insert(String::from("theme_temp_path"),String::from(".theme"));
    constants.insert(String::from("git_ignore_path"),String::from(".gitignore"));
    constants.insert(String::from("default_post_title"),String::from("Welcome"));
    constants.insert(String::from("stylesheet_name"),String::from("styles.css"));
    constants.insert(String::from("blog_layout"),String::from("blog.html"));
    constants.insert(String::from("page_layout"),String::from("page.html"));
    constants.insert(String::from("post_layout"),String::from("post.html"));
    constants.insert(String::from("version"),String::from("1.1.1"));
    constants.insert(String::from("name"),String::from("Acid"));
    constants.insert(String::from("author"),String::from("Alexander Abraham"));
    return constants;
}

// Returns a vector of strings from a character split for a string.
/// Both the string and split character have to be strings.
fn clean_split(subject: String, split_char: String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for item in subject.split(&split_char) {
        let new_item: String = item.to_string();
        result.push(new_item);
    }
    return result;
}

// Checks whether a file exists and
/// returns a boolean to that effect.
fn file_is(filename: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let contents = read_to_string(filename);
    match contents {
        Ok(_n) => result.push(true),
        Err(_x) => result.push(false)
    }
    return result[0];
}

/// Tries to create a file and returns
/// a boolean depending on whether the
/// operation succeeded.
fn create_file(filename: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let new_file = File::create(filename);
    match new_file {
        Ok(_n) => result.push(true),
        Err(_x) => result.push(false)
    }
    return result[0];
}

/// Tries to write to a file and returns
/// a boolean depending on whether the
/// operation succeeded.
fn write_to_file(filename: String, contents: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let fname_copy: String = filename.clone();
    if file_is(filename) == true {
        let write_op = write(fname_copy, contents);
        match write_op {
            Ok(_n) => result.push(true),
            Err(_x) => result.push(false)
        }
    }
    return result[0];
}

/// Tries to read a file and return
/// its contents.
fn read_file(filename: String) -> String {
    let mut result: String = String::from("");
    let fname_copy: String = filename.clone();
    if file_is(filename) == true {
        result = read_to_string(fname_copy).unwrap();
    }
    else {}
    return result;
}

/// Reads a aml string and returns a [HashMap] from it.
fn get_aml(subject: String) -> HashMap<String, String> {
    return aml_serialize(subject);
}

/// Getting site settings.
fn get_site_config(config_path: String) -> HashMap<String, String> {
    let result = get_aml(read_file(config_path));
    return result;
}

/// Serializes Markdown content saved in "md_string" into a [HashMap].
fn serialize_front_matter(md_string: String) -> HashMap<String,String> {
    let mut result: HashMap<String, String> = HashMap::new();
    let mut extractor = Extractor::new(&md_string);
    extractor.select_by_terminator("---");
    let im_output: String = extractor.extract();
    let mut cleaned_front_matter_vector: Vec<String> = clean_split(im_output, String::from("\n"));
    cleaned_front_matter_vector.remove(0);
    for item in cleaned_front_matter_vector.clone() {
        let split_items = clean_split(item, String::from(":"));
        result.insert(String::from(split_items[0].clone()),String::from(split_items[1].clone()));
    }
    let content: &str = extractor.remove().trim();
    result.insert(String::from("content"), markdown::to_html(&content.to_owned()));
    return result;
}

/// Checks whether a directory exists and returns
/// a boolean to that effect.
fn dir_is(path: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    if Path::new(&path).exists() {
        result.push(true);
    }
    else {
        result.push(false);
    }
return result[0];
}

/// Deletes the directory at "path"
/// and returns a boolean depending
/// on whether the operation succeeded.
fn clean(path: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let del_op = remove_dir_all(path);
    match del_op {
        Ok(_x) => result.push(true),
        Err(_e) => result.push(false)
    }
    return result[0];
}

/// Lists files with their full relative path
/// in "dir".
fn raw_list_files(dir: String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    if Path::new(&dir).exists() {
        let paths = read_dir(&dir).unwrap();
        for path in paths {
            let raw_path = path.unwrap().path().display().to_string();
            result.push(raw_path);
        }
    }
    else {}
    return result;
}

/// A struct to contain template data for the main layout.
#[derive(Debug)]
#[derive(ObjectView,ValueView)]
struct HomeContext {
    site: HashMap<String, String>,
    posts: Vec<HashMap<String, String>>,
}

/// A struct to contain template data for pages.
#[derive(Debug)]
#[derive(ObjectView,ValueView)]
struct PageContext {
    site: HashMap<String, String>,
    page: HashMap<String, String>
}

/// Fills a liquid template at "template_path" with a context object for content pages.
fn fill_template_page(template_path: String, context: PageContext) -> String {
    let template_path_clone_one = template_path.clone();
    let template_path_clone_two = template_path_clone_one.clone();
    let mut result_string: String = String::from("");
    let mut result_vec: Vec<String> = Vec::new();
    let liquid_string = ParserBuilder::with_stdlib().build().unwrap().parse(&read_file(template_path_clone_one)).unwrap();
    let globals = object!(context);
    let output = liquid_string.render(&globals);
    match output {
        Ok(_x) => result_vec.push(_x),
        Err(_e) => {
            println!("{}", format!("There was an error in your template, \'{}\'!",template_path_clone_two).red().to_string());
            println!("Error: {}", _e);
            exit(0);
        }

    }
    return result_vec[0].clone();
}

/// Fills a liquid template at "template_path" with a context object for an Acid project's "index.html".
fn fill_template_home(template_path: String, context: HomeContext) -> String {
    let template_path_clone_one = template_path.clone();
    let template_path_clone_two = template_path_clone_one.clone();
    let mut result_string: String = String::from("");
    let mut result_vec: Vec<String> = Vec::new();
    let liquid_string = ParserBuilder::with_stdlib().build().unwrap().parse(&read_file(template_path_clone_one)).unwrap();
    let globals = object!(context);
    let output = liquid_string.render(&globals);
    match output {
        Ok(_x) => result_vec.push(_x),
        Err(_e) => {
            println!("{}", format!("There was an error in your template, \'{}\'!",template_path_clone_two).red().to_string());
            println!("Error: {}", _e);
            exit(0);
        }

    }
    return result_vec[0].clone();
}

/// Tries to create a new directory and returns
/// a boolean depending on whether the
/// operation succeeded.
fn create_directory(path: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let new_dir = create_dir(path);
    match new_dir {
        Ok(_n) => result.push(true),
        Err(_x) => result.push(false)
    }
    return result[0];
}

/// Tries to move a file from "src" to "target"
/// and returns a boolean depending on whether the
/// operation succeeded.
fn file_move(src: String, target: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let options = CopyOptions::new();
    let move_op = move_file(src, target, &options);
    match move_op {
        Ok(_n) => result.push(true),
        Err(_x) => result.push(false)
    }
    return result[0];
}

/// Attempts to move a directory from "src" to "target".
/// A boolean is returned depending on whether the operation
/// suceeded.
fn dir_move(src: String, target: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let options = fs_extra::dir::CopyOptions::new();
    let move_op = move_dir(src, target, &options);
    match move_op {
        Ok(_n) => result.push(true),
        Err(_x) => result.push(false)
    }
    return result[0];
}

/// Creates a boilerplate Acid theme at "project_path".
fn scaffold_theme(project_path: String) {
    let project_path_clone_one: String = project_path.clone();
    let project_path_clone_two: String = project_path_clone_one.clone();
    let project_path_clone_three: String = project_path_clone_two.clone();
    let project_path_clone_four: String = project_path_clone_three.clone();
    let project_path_clone_five: String = project_path_clone_four.clone();
    let project_path_clone_six: String = project_path_clone_five.clone();
    let project_path_clone_seven: String = project_path_clone_six.clone();
    let project_path_clone_eight: String = project_path_clone_seven.clone();
    let project_path_clone_nine: String = project_path_clone_eight.clone();
    let project_path_clone_ten: String = project_path_clone_nine.clone();
    let mut config_map: HashMap<String, String> = HashMap::new();
    config_map.insert(String::from("name"), project_path_clone_one);
    config_map.insert(String::from("version"), String::from("1.0.0"));
    config_map.insert(String::from("type"), String::from("theme"));
    config_map.insert(String::from("assets_path"), acid_constants()["assets_dir"].clone());
    let aml_string: String = map_to_aml(config_map);
    let git_ignore_path: String = format!("{}/{}", project_path, acid_constants()["git_ignore_path"].clone());
    let config_path: String = format!("{}/{}", project_path, acid_constants()["config_file_path"].clone());
    let readme_path: String = format!("{}/README.markdown", project_path_clone_three);
    let css_path: String = format!("{}/{}/{}",project_path_clone_nine,acid_constants()["assets_dir"].clone(),acid_constants()["stylesheet_name"].clone());
    let blog_layout_path: String = format!("{}/{}/{}", project_path_clone_four, acid_constants()["layouts_dir"].clone(), acid_constants()["blog_layout"].clone());
    let page_layout_path: String = format!("{}/{}/{}", project_path_clone_five, acid_constants()["layouts_dir"].clone(), acid_constants()["page_layout"].clone());
    let post_layout_path: String = format!("{}/{}/{}", project_path_clone_six, acid_constants()["layouts_dir"].clone(), acid_constants()["post_layout"].clone());
    let config_path_clone: String = config_path.clone();
    let readme_path_clone: String = readme_path.clone();
    let blog_layout_path_clone: String = blog_layout_path.clone();
    let page_layout_path_clone: String = page_layout_path.clone();
    let post_layout_path_clone: String = post_layout_path.clone();
    let git_ignore_path_clone: String = git_ignore_path.clone();
    let css_path_clone: String = css_path.clone();
    let layout_contents: String = String::from("<!--Put your Liquid layout into this file.-->");
    let css_contents: String = String::from("/*Put your theme\'s styles into this file.*/");
    let layout_contents_clone_one: String = layout_contents.clone();
    let layout_contents_clone_two: String = layout_contents_clone_one.clone();
    let readme_contents: String = format!("# {}\n\nDescribe your theme here.", project_path_clone_ten);
    let git_ignore_contents: String = format!(".DS_Store\n");
    let layout_path = format!("{}/{}", project_path_clone_seven, acid_constants()["layouts_dir"].clone());
    let assets_path = format!("{}/{}", project_path_clone_eight, acid_constants()["assets_dir"].clone());
    create_dir(project_path_clone_nine);
    create_dir(layout_path);
    create_dir(assets_path);
    create_file(config_path);
    write_to_file(config_path_clone, aml_string);
    create_file(readme_path);
    write_to_file(readme_path_clone, readme_contents);
    create_file(git_ignore_path);
    write_to_file(git_ignore_path_clone, git_ignore_contents);
    create_file(blog_layout_path);
    write_to_file(blog_layout_path_clone, layout_contents);
    create_file(page_layout_path);
    write_to_file(page_layout_path_clone, layout_contents_clone_one);
    create_file(post_layout_path);
    write_to_file(post_layout_path_clone, layout_contents_clone_two);
    create_file(css_path);
    write_to_file(css_path_clone, css_contents);
}

/// Creates a boilerplate Acid site at "project_path".
fn scaffold_site(project_path: String) {
    let project_path_clone_one: String = project_path.clone();
    let project_path_clone_two: String = project_path_clone_one.clone();
    let project_path_clone_three: String = project_path_clone_two.clone();
    let project_path_clone_four: String = project_path_clone_three.clone();
    let project_path_clone_five: String = project_path_clone_four.clone();
    let project_path_clone_six: String = project_path_clone_five.clone();
    let project_path_clone_seven: String = project_path_clone_six.clone();
    let project_path_clone_eight: String = project_path_clone_seven.clone();
    let project_path_clone_nine: String = project_path_clone_eight.clone();
    let project_path_clone_ten: String = project_path_clone_nine.clone();
    let markdown_about_name: String = String::from("about.markdown");
    let utc: DateTime<Utc> = Utc::now();
    let current_date: String = format!("{}", clean_split(clean_split(utc.to_string(), String::from(" "))[0].clone(), String::from(" ")).join("-"));
    let markdown_post_name: String = format!("{}-{}.markdown", current_date, acid_constants()["default_post_title"].clone());
    let config_path: String = format!("{}/{}", project_path, acid_constants()["config_file_path"].clone());
    let index_path: String = format!("{}/{}", project_path_clone_one, acid_constants()["index_path"].clone());
    let git_ignore_path: String = format!("{}/{}", project_path_clone_two, acid_constants()["git_ignore_path"].clone());
    let sample_post: String = format!("{}/{}/{}", project_path_clone_three, acid_constants()["posts_dir"].clone(), markdown_post_name);
    let about_page: String = format!("{}/{}/{}", project_path_clone_four, acid_constants()["pages_dir"].clone(), markdown_about_name);
    let readme_path: String = format!("{}/README.markdown", project_path_clone_eight);
    let config_path_clone: String = config_path.clone();
    let index_path_clone: String = index_path.clone();
    let git_ignore_path_clone: String = git_ignore_path.clone();
    let sample_post_clone: String = sample_post.clone();
    let about_page_clone: String = about_page.clone();
    let readme_path_clone: String = readme_path.clone();
    let posts_dir: String = format!("{}/{}", project_path_clone_five, acid_constants()["posts_dir"].clone());
    let pages_dir: String = format!("{}/{}", project_path_clone_six, acid_constants()["pages_dir"].clone());
    let mut config_map: HashMap<String, String> = HashMap::new();
    config_map.insert(String::from("title"), project_path_clone_seven);
    config_map.insert(String::from("has_assets"), String::from("true"));
    config_map.insert(String::from("assets_path"), acid_constants()["assets_dir"].clone());
    config_map.insert(String::from("baseurl"), String::from("/"));
    config_map.insert(String::from("description"), String::from("The description of your site goes here."));
    config_map.insert(String::from("theme"), String::from("https://github.com/iamtheblackunicorn/acid-tripping"));
    config_map.insert(String::from("use_remote_theme"), String::from("true"));
    config_map.insert(String::from("type"), String::from("site"));
    config_map.insert(String::from("keywords"),String::from("acid cms site blog"));
    config_map.insert(String::from("profile_pic"),String::from("https://raw.githubusercontent.com/iamtheblackunicorn/acid/main/assets/images/logo/logo.png"));
    config_map.insert(String::from("fifty_seven_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-57x57.png"));
    config_map.insert(String::from("sixty_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-60x60.png"));
    config_map.insert(String::from("seven_two_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-72x72.png"));
    config_map.insert(String::from("seven_six_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-76x76.png"));
    config_map.insert(String::from("one_one_four_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-114x114.png"));
    config_map.insert(String::from("one_two_zero_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-120x120.png"));
    config_map.insert(String::from("one_four_four_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-144x144.png"));
    config_map.insert(String::from("one_five_two_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-152x152.png"));
    config_map.insert(String::from("one_eight_zero_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/apple-icon-180x180.png"));
    config_map.insert(String::from("one_nine_two_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/android-icon-192x192.png"));
    config_map.insert(String::from("thirty_two_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/favicon-32x32.png"));
    config_map.insert(String::from("nine_six_icon"),String::from("https://blckunicorn.art/acid/assets/favicons/favicon-96x96.png"));
    config_map.insert(String::from("social_media_image_title"),String::from("Acid, a static-site generator."));
    config_map.insert(String::from("social_media_image"),String::from("https://raw.githubusercontent.com/iamtheblackunicorn/acid/main/assets/images/logo/banner.png"));
    config_map.insert(String::from("google_analytics_id"),String::from("WHATEVER"));
    config_map.insert(String::from("viewText"),String::from("READ ME"));
    let aml_string: String = map_to_aml(config_map);
    let git_ignore_contents: String = String::from("/.theme\n/build\n.DS_Store");
    let post_contents: String = String::from("---\ntitle:Welcome\nlayout:post\ndescription:A short welcome post.\n---\n\n## Your post\nYour post\'s contents goes here.");
    let index_contents: String = String::from("---\ntitle:My Blog\nlayout:blog\n---");
    let about_contents: String = String::from("---\ntitle:About\nlayout:page\ndescription:About me.\n---\n\n## About\nWrite something about yourself here.");
    let readme_contents: String = format!("# {}\n\nDescribe your site here.", project_path_clone_nine);
    create_dir(project_path_clone_ten);
    create_dir(posts_dir);
    create_dir(pages_dir);
    create_file(config_path);
    write_to_file(config_path_clone, aml_string);
    create_file(git_ignore_path);
    write_to_file(git_ignore_path_clone, git_ignore_contents);
    create_file(index_path);
    write_to_file(index_path_clone, index_contents);
    create_file(readme_path);
    write_to_file(readme_path_clone, readme_contents);
    create_file(sample_post);
    write_to_file(sample_post_clone, post_contents);
    create_file(about_page);
    write_to_file(about_page_clone, about_contents);
}

/// Tries to copy a folder from "src" to "target"
/// and returns a boolean depending on whether the
/// operation succeeded.
fn folder_copy(src: String, target: String) -> bool {
    let mut result: Vec<bool> = Vec::new();
    let options = fs_extra::dir::CopyOptions::new();
    let copy_op = copy(src, target, &options);
    match copy_op {
        Ok(_n) => result.push(true),
        Err(_x) => result.push(false)
    }
    return result[0];
}

/// Generates the website's content pages.
fn generate_pages(project_path: String) {
    let project_path_clone = project_path.clone();
    let project_path_clone_one = project_path_clone.clone();
    let project_path_clone_two = project_path_clone_one.clone();
    let project_path_clone_three = project_path_clone_two.clone();
    let pages_path = format!("{}/{}", project_path,acid_constants()["pages_dir"].clone());
    let config_path = format!("{}/{}", project_path_clone_two, acid_constants()["config_file_path"].clone());
    let built_page_path = format!("{}/{}/{}", project_path_clone, acid_constants()["build_dir"].clone(),acid_constants()["pages_dir"].clone());
    let built_page_path_clone = built_page_path.clone();
    create_dir(built_page_path);
    let markdown_page_list: Vec<String> = raw_list_files(pages_path);
    let config = get_site_config(config_path);
    for markdown_page in markdown_page_list {
        let markdown_page_clone = markdown_page.clone();
        let markdown_page_clone_one = markdown_page_clone.clone();
        let path_vector: Vec<String> = clean_split(markdown_page_clone, String::from("/"));
        let base_fname: String = path_vector[path_vector.len()-1].clone();
        let base_file_name: String = clean_split(base_fname, String::from("."))[0].clone();
        let new_file_name: String = format!("{}.html", base_file_name);
        let new_file_name_clone: String = new_file_name.clone();
        let new_file_name_clone_one: String = new_file_name_clone.clone();
        let new_file_name_clone_two: String = new_file_name_clone_one.clone();
        let serialized_page = serialize_front_matter(read_file(markdown_page));
        let page_current_path: String = format!("{}/{}", project_path_clone_three, new_file_name_clone_one);
        let page_current_path_clone: String = page_current_path.clone();
        let page_current_path_clone_one = page_current_path_clone.clone();
        let page_target_path: String = format!("{}/{}", built_page_path_clone, new_file_name_clone_two);
        if serialized_page.clone().contains_key("layout") {
            let layout_path = format!("{}/{}/{}.html", project_path_clone_one, acid_constants()["layouts_dir"].clone(), serialized_page["layout"]);
            let page_context = PageContext{
                site: config.clone(),
                page: serialized_page
            };
            let output: String = fill_template_page(layout_path, page_context);
            create_file(page_current_path_clone);
            write_to_file(page_current_path_clone_one, output);
            file_move(page_current_path, page_target_path);
        }
        else {
            let error_message: String = format!("Layout for \'{}\' not found!", markdown_page_clone_one).red().to_string();
            println!("{}", error_message);
            exit(0);
        }
    }
}

/// A function fetch and use third-party themes.
fn use_theme(project_path: String) {
    let project_path_clone: String = project_path.clone();
    let project_path_clone_one: String = project_path_clone.clone();
    let project_path_clone_two: String = project_path_clone_one.clone();
    let local_theme_dir: String = format!("{}/{}", project_path, acid_constants()["theme_temp_path"].clone());
    let config_path: String = format!("{}/{}", project_path_clone, acid_constants()["config_file_path"].clone());
    let config_path_clone: String = config_path.clone();
    let config_info: HashMap<String, String> = get_site_config(config_path);
    if config_info["use_remote_theme"].clone() == "true" && config_info.clone().contains_key("theme") && config_info["type"].clone() == "site" {
        let repo_url: String = config_info["theme"].clone();
        let local_theme_dir_clone: String = local_theme_dir.clone();
        let local_theme_dir_clone_one: String = local_theme_dir_clone.clone();
        let local_theme_dir_clone_two: String = local_theme_dir_clone_one.clone();
        let local_theme_dir_clone_three: String = local_theme_dir_clone_two.clone();
        let theme_config_path: String = format!("{}/{}", local_theme_dir_clone_three,acid_constants()["config_file_path"].clone());
        let theme_config_path_clone: String = theme_config_path.clone();
        create_directory(local_theme_dir);
        clone_repo(repo_url, local_theme_dir_clone);
        let theme_config: HashMap<String,String> = get_site_config(theme_config_path);
        if file_is(theme_config_path_clone) && theme_config.clone().contains_key("assets_path") && theme_config.clone().contains_key("type") && theme_config["type"].clone() == "theme" {
            let theme_layouts_path: String = format!("{}/{}", local_theme_dir_clone_one, acid_constants()["layouts_dir"].clone());
            let theme_assets_path: String = format!("{}/{}", local_theme_dir_clone_two, theme_config["assets_path"].clone());
            let theme_layouts_path_target: String = format!("{}", project_path_clone_one);
            let theme_assets_path_target: String = format!("{}", project_path_clone_two);
            dir_move(theme_layouts_path, theme_layouts_path_target);
            dir_move(theme_assets_path, theme_assets_path_target);
        }
        else {
            println!("{}", format!("The theme is configured incorrectly or could not be found.").red().to_string());
        }
    }
    else {}
}

/// Generates the posts for an Acid project
/// at "project_path" and the project's "index.html".
fn generate_posts_and_index(project_path: String){
    let project_path_clone: String = project_path.clone();
    let project_path_clone_one: String = project_path_clone.clone();
    let project_path_clone_two: String = project_path_clone_one.clone();
    let project_path_clone_three: String = project_path_clone_two.clone();
    let project_path_clone_four: String = project_path_clone_three.clone();
    let project_path_clone_five: String = project_path_clone_four.clone();
    let project_path_clone_six: String = project_path_clone_five.clone();
    let project_path_clone_seven: String = project_path_clone_six.clone();
    let project_path_clone_eight: String = project_path_clone_seven.clone();
    let build_path = format!("{}/{}", project_path_clone_four,acid_constants()["build_dir"].clone());
    let build_path_clone = build_path.clone();
    create_directory(build_path_clone);
    let built_posts_path = format!("{}/{}",build_path,acid_constants()["posts_dir"].clone());
    let built_posts_path_clone = built_posts_path.clone();
    create_directory(built_posts_path);
    let config: HashMap<String, String> = get_site_config(format!("{}/{}", project_path_clone_one,acid_constants()["config_file_path"]));
    let posts_path: String = format!("{}/{}", project_path, acid_constants()["posts_dir"]);
    let config_clone = config.clone();
    let markdown_posts: Vec<String> = raw_list_files(posts_path);
    let mut index_context: Vec<HashMap<String, String>> = Vec::new();
    for markdown_post in markdown_posts {
        let markdown_post_clone = markdown_post.clone();
        let markdown_post_clone_one = markdown_post_clone.clone();
        let mut serialized_post: HashMap<String,String> = serialize_front_matter(read_file(markdown_post));
        let serialized_post_clone = serialized_post.clone();
        let path_vector: Vec<String> = clean_split(markdown_post_clone, String::from("/"));
        let base_fname: String = path_vector[path_vector.len()-1].clone();
        let base_file_name: String = clean_split(base_fname, String::from("."))[0].clone();
        let new_file_name: String = format!("{}.html", base_file_name);
        let new_file_name_clone: String = new_file_name.clone();
        let new_file_name_clone_one: String = new_file_name_clone.clone();
        let new_file_name_clone_two: String = new_file_name_clone_one.clone();
        let new_file_name_clone_three: String = new_file_name_clone_two.clone();
        let post_url: String = format!("{}/{}", acid_constants()["posts_dir"],new_file_name_clone_one);
        let target_path = format!("{}/{}", built_posts_path_clone, new_file_name_clone_two);
        let current_path = format!("{}/{}", project_path_clone_five,new_file_name_clone_three);
        let current_path_clone_one = current_path.clone();
        let current_path_clone_two = current_path_clone_one.clone();
        serialized_post.insert(acid_constants()["post_url_key"].clone(),post_url);
        if serialized_post.clone().contains_key("layout") {
            let layout_path: String = format!("{}/{}/{}.html", project_path_clone,acid_constants()["layouts_dir"],serialized_post["layout"].clone());
            let page_context = PageContext{
                site: config.clone(),
                page: serialized_post_clone
            };
            let output = fill_template_page(layout_path, page_context);
            create_file(current_path_clone_one);
            write_to_file(current_path_clone_two, output);
            file_move(current_path, target_path);
            index_context.push(serialized_post);
        }
        else {
            let error_message: String = format!("Layout for \'{}\' not found!", markdown_post_clone_one).red().to_string();
            println!("{}", error_message);
            exit(0);
        }
    }
    let index_path: String = format!("{}/{}",project_path_clone_eight,acid_constants()["index_path"].clone());
    let index_path_clone: String = index_path.clone();
    if file_is(index_path) {
        let serialized_index: HashMap<String, String> = serialize_front_matter(read_file(index_path_clone));
        if serialized_index.contains_key("layout") {
            let index_layout_path = format!("{}/{}/{}.html",project_path_clone_three,acid_constants()["layouts_dir"].clone(),serialized_index["layout"]);
            let new_index_context = HomeContext{
                site: config_clone,
                posts: index_context
            };
            let current_index_path = format!("{}/{}", project_path_clone_six, acid_constants()["index_output_path"].clone());
            let current_index_path_clone_one = current_index_path.clone();
            let current_index_path_clone_two = current_index_path_clone_one.clone();
            let target_index_path = format!("{}/{}/{}", project_path_clone_seven, acid_constants()["build_dir"].clone(),acid_constants()["index_output_path"].clone());
            let index_output = fill_template_home(index_layout_path, new_index_context);
            create_file(current_index_path_clone_one);
            write_to_file(current_index_path_clone_two,index_output);
            file_move(current_index_path, target_index_path);
        }
        else {
            let error_message: String = format!("Layout for \'{}\' not found!", acid_constants()["index_path"].clone()).red().to_string();
            println!("{}", error_message);
            exit(0);
        }
    }
    else{
        let err_msg: String = format!("No \'{}\' found!",acid_constants()["index_path"]).red().to_string();
        println!("{}", err_msg);
        exit(0);
    }
}

/// Does some "pre-flight" checks on an Acid project
/// and compiles the project at "project_path/build".
fn toolchain(project_path: String){
    let project_path_clone_one: String = project_path.clone();
    let project_path_clone_two: String = project_path_clone_one.clone();
    let project_path_clone_three: String = project_path_clone_two.clone();
    let project_path_clone_four: String = project_path_clone_three.clone();
    let project_path_clone_five: String = project_path_clone_four.clone();
    let project_path_clone_six: String = project_path_clone_five.clone();
    let project_path_clone_seven: String = project_path_clone_six.clone();
    let project_path_clone_eight: String =  project_path_clone_seven.clone();
    let project_path_clone_nine: String = project_path_clone_eight.clone();
    let project_path_clone_ten: String = project_path_clone_nine.clone();
    let project_path_clone_eleven: String = project_path_clone_ten.clone();
    let project_path_clone_twelve: String = project_path_clone_eleven.clone();
    let project_path_clone_thirteen: String = project_path_clone_twelve.clone();
    let project_path_clone_fourteen: String = project_path_clone_thirteen.clone();
    let project_path_clone_fifteen: String = project_path_clone_fourteen.clone();
    let config_path: String = format!("{}/{}", project_path_clone_one, acid_constants()["config_file_path"].clone());
    let layouts_path: String = format!("{}/{}", project_path_clone_two, acid_constants()["layouts_dir"].clone());
    let posts_path: String = format!("{}/{}", project_path_clone_three, acid_constants()["posts_dir"].clone());
    let posts_path_clone_one: String = posts_path.clone();
    let posts_path_clone_two: String = posts_path_clone_one.clone();
    let pages_path: String = format!("{}/{}", project_path_clone_four, acid_constants()["pages_dir"].clone());
    let pages_path_clone_one: String = pages_path.clone();
    let pages_path_clone_two: String = pages_path_clone_one.clone();
    let build_dir: String = format!("{}/{}", project_path_clone_seven, acid_constants()["build_dir"].clone());
    let temp_theme_dir: String = format!("{}/{}", project_path_clone_seven, acid_constants()["theme_temp_path"].clone());
    let build_dir_clone = build_dir.clone();
    let config_path_clone = config_path.clone();
    let config_path_clone_one = config_path_clone.clone();
    let config_path_clone_two = config_path_clone_one.clone();
    let config_path_clone_three = config_path_clone_two.clone();
    let config_info: HashMap<String, String> = get_site_config(config_path_clone);
    let layout_path_clone: String = layouts_path.clone();
    let layout_path_clone_one: String = layout_path_clone.clone();
    if file_is(config_path_clone_one) && dir_is(posts_path) && dir_is(pages_path) && dir_is(layouts_path) && config_info.clone().contains_key("theme") == false && config_info.clone().contains_key("use_remote_theme") == false && config_info["type"].clone() == "site" {
        println!("{}", format!("Compiling your site!").cyan().to_string());
        if dir_is(build_dir) {
            clean(build_dir_clone);
            generate_posts_and_index(project_path_clone_five);
            generate_pages(project_path_clone_six);
            copy_assets(project_path_clone_ten);
        }
        else {
            generate_posts_and_index(project_path_clone_eight);
            generate_pages(project_path_clone_nine);
            copy_assets(project_path_clone_fifteen);
        }
        println!("{}", format!("Done.").green().to_string());
    }
    else if dir_is(layout_path_clone) == false && file_is(config_path_clone_two) && dir_is(posts_path_clone_one) && dir_is(pages_path_clone_one) && config_info.clone().contains_key("theme") && config_info["use_remote_theme"].clone() == "true" && config_info["type"].clone() == "site" {
        println!("{}", format!("Compiling your site!").cyan().to_string());
        if dir_is(build_dir) {
            clean(build_dir_clone);
            clean(temp_theme_dir);
            use_theme(project_path_clone_twelve);
            generate_posts_and_index(project_path_clone_five);
            generate_pages(project_path_clone_six);
            copy_assets(project_path_clone_fourteen);
        }
        else {
            use_theme(project_path_clone_thirteen);
            generate_posts_and_index(project_path_clone_eight);
            generate_pages(project_path_clone_nine);
            copy_assets(project_path_clone_ten);
        }
        println!("{}", format!("Done.").green().to_string());
    }
    else if dir_is(layout_path_clone_one) == true && file_is(config_path_clone_three) && dir_is(posts_path_clone_two) && dir_is(pages_path_clone_two) && config_info.clone().contains_key("theme") && config_info["use_remote_theme"].clone() == "true" {
        println!("{}", format!("You have set the \'use_remote_theme\' flag to \'true\'. This is not allowed with custom layouts.").red().to_string());
        exit(0);
    }
    else {
        println!("{}", format!("One or more project-critical files or directories could not be found!").red().to_string());
        exit(0);
    }
}

/// Copies project assets to the "build" directory.
fn copy_assets(project_path: String) {
    let project_path_clone_one = project_path.clone();
    let project_path_clone_two = project_path_clone_one.clone();
    let project_path_clone_three = project_path_clone_two.clone();
    let config_path = format!("{}/{}", project_path_clone_one, acid_constants()["config_file_path"].clone());
    let config_path_clone = config_path.clone();
    if file_is(config_path) {
        let config: HashMap<String,String> = get_site_config(config_path_clone);
        if config["has_assets"].clone() == "true" && config.contains_key("assets_path") {
            let stand_alone_assets_path: String = config["assets_path"].clone();
            let build_dir_path: String = format!("{}/{}", project_path_clone_two, acid_constants()["build_dir"].clone());
            let old_assets_path: String = format!("{}/{}", project_path_clone_three, stand_alone_assets_path);
            let old_assets_path_clone: String = old_assets_path.clone();
            if dir_is(old_assets_path) {
                folder_copy(old_assets_path_clone, build_dir_path);
            }
            else {
                println!("{}", format!("The assets path does not exist.").red().to_string());
                exit(0);
            }
        }
        else {}
    }
    else {
        println!("{}", format!("Configuration file not found!").red().to_string());
        exit(0);
    }
}

/// Serves the "build" dir
/// on this address: https://localhost:1024.
fn serve_dir(project_path: String) {
    let mut path: PathBuf = PathBuf::new();
    path.push(project_path);
    path.push(acid_constants()["build_dir"].clone());
    let server_instance: Server = Server::new(path);
    println!("{}", format!("Serving your site on address:\n{}\nPress Ctrl+C to quit the server.", server_instance.addr()).cyan().to_string());
    server_instance.serve();
}

/// Acid's command-line interface.
fn cli(){
    let mut acid_cli: App = App::new(
        acid_constants()["name"].clone(),
        acid_constants()["version"].clone(),
        acid_constants()["author"].clone()
    );
    acid_cli.add_arg(
        "build".to_string(),
        "Build an Acid site.".to_string(),
        "true".to_string()
    );
    acid_cli.add_arg(
        "clean".to_string(),
        "Clean a built Acid site.".to_string(),
        "true".to_string()
    );
    acid_cli.add_arg(
        "serve".to_string(),
        "Serve a built Acid site.".to_string(),
        "true".to_string()
    );
    acid_cli.add_arg(
        "theme".to_string(),
        "Serve a built Acid site.".to_string(),
        "true".to_string()
    );
    acid_cli.add_arg(
        "nsite".to_string(),
        "Scaffold a new Acid site.".to_string(),
        "true".to_string()
    );
    if acid_cli.arg_was_used("build".to_string()) && dir_is(acid_cli.get_arg_data("build".to_string())) {
        toolchain(
            acid_cli.get_arg_data("build".to_string())
        );
    }
    else if acid_cli.arg_was_used("clean".to_string()) && dir_is(acid_cli.get_arg_data("clean".to_string())) {
        let build_dir: String = format!(
            "{}/{}",
            acid_cli.get_arg_data("clean".to_string()),
            acid_constants()["build_dir"].clone()
        );
        clean(build_dir);
    }
    else if acid_cli.arg_was_used("serve".to_string()) && dir_is(acid_cli.get_arg_data("serve".to_string())) {
        serve_dir(
            acid_cli.get_arg_data("serve".to_string())
        );
    }
    else if acid_cli.arg_was_used("theme".to_string()) {
        scaffold_theme(
            acid_cli.get_arg_data("theme".to_string())
        );
    }
    else if acid_cli.arg_was_used("nsite".to_string()) {
        scaffold_site(
            acid_cli.get_arg_data("nsite".to_string())
        );
    }
    else if acid_cli.version_is() {
        println!(
            "{}",
            acid_cli.version().cyan().to_string()
        );
    }
    else if acid_cli.help_is() {
        println!(
            "{}",
            acid_cli.help().cyan().to_string()
        );

    }
    else {
        println!(
            "{}",
            acid_cli.help().red().to_string()
        );
    }
}

/// The main entry point for
/// the Rust compiler.
fn main(){
    cli();
}
