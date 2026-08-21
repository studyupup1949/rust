/*
ACID by Alexander Abraham,
a.k.a. "The Black Unicorn", a.k.a. "Angeldust Duke".
Licensed under the MIT license.
*/

use std::env;
use kstring::*;
use std::fs::File;
use std::fs::write;
use liquid::object;
use std::path::Path;
use std::fs::read_dir;
use liquid::ValueView;
use colored::Colorize;
use std::process::exit;
use liquid::ObjectView;
use std::fs::create_dir;
use fs_extra::dir::copy;
use serde_json::from_str;
use liquid::ParserBuilder;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::collections::HashMap;
use fs_extra::file::move_file;
use fs_extra::file::CopyOptions;
use extract_frontmatter::Extractor;

/// Constants for Acid websites.
fn acid_constants() -> HashMap<String, String> {
    let mut constants: HashMap<String, String> = HashMap::new();
    constants.insert(String::from("config_file_path"),String::from("config.json"));
    constants.insert(String::from("layouts_dir"),String::from("layouts"));
    constants.insert(String::from("posts_dir"),String::from("posts"));
    constants.insert(String::from("pages_dir"),String::from("pages"));
    constants.insert(String::from("index_path"),String::from("index.markdown"));
    constants.insert(String::from("index_output_path"),String::from("index.html"));
    constants.insert(String::from("build_dir"),String::from("build"));
    constants.insert(String::from("post_url_key"),String::from("url"));
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

/// Reads a JSON string and returns a [HashMap] from it.
fn get_json(subject: String) -> HashMap<String, String> {
    return from_str(&subject).unwrap();
}

/// Getting site settings.
fn get_site_config(config_path: String) -> HashMap<String, String> {
    let result = get_json(read_file(config_path));
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
        Err(_e) => println!("{}", format!("There was an error in your template, \'{}\'!",template_path_clone_two).red().to_string())

    }
    if result_vec.len() == 0 {
        println!("{}", format!("There was an error in your template!").red().to_string());
        exit(0);
    }
    else {
        result_string = result_vec[0].clone();
    }
    return result_string;
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
        Err(_e) => println!("{}", format!("There was an error in your template, \'{}\'!",template_path_clone_two).red().to_string())

    }
    if result_vec.len() == 0 {
        println!("{}", format!("There was an error in your template!").red().to_string());
        exit(0);
    }
    else {
        result_string = result_vec[0].clone();
    }
    return result_string;
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
    let config_path: String = format!("{}/{}", project_path_clone_one, acid_constants()["config_file_path"].clone());
    let layouts_path: String = format!("{}/{}", project_path_clone_two, acid_constants()["layouts_dir"].clone());
    let posts_path: String = format!("{}/{}", project_path_clone_three, acid_constants()["posts_dir"].clone());
    let pages_path: String = format!("{}/{}", project_path_clone_four, acid_constants()["pages_dir"].clone());
    let build_dir: String = format!("{}/{}", project_path_clone_seven, acid_constants()["build_dir"].clone());
    let build_dir_clone = build_dir.clone();
    if file_is(config_path) && dir_is(layouts_path) && dir_is(posts_path) && dir_is(pages_path) {
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
            copy_assets(project_path_clone_eleven);
        }
        println!("{}", format!("Done.").green().to_string());
    }
    else {
        println!("{}", format!("One or more project-critical files or directories could not be found!").red().to_string());
        exit(0);
    }
}

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

/// Acid's command-line interface.
fn cli(){
    let args: Vec<String> = env::args().collect();
    let arg_len = args.len();
    if arg_len == 3 {
        if args[1].clone() == "build" && dir_is(args[2].clone()){
            toolchain(args[2].clone());
        }
        else if args[1].clone() == "clean" && dir_is(args[2].clone()) {
            clean(format!("{}/{}",args[2].clone(), acid_constants()["build_dir"].clone()));
        }
        else {
            println!("{}", format!("The project directory \'{}\' does not exist!", args[2].clone()).red().to_string());
            exit(0);
        }
    }
    else {
        println!("{}", format!("Wrong usage!").red().to_string());
        exit(0);
    }
}

/// The main entry point for
/// the Rust compiler.
fn main(){
    cli();
}
