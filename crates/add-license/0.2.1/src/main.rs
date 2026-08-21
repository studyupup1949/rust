use std::env;
use std::fs::File;
use std::io::Write;

mod toml_file;

use toml_file::TomlFile;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] != "add-license" {
        std::process::exit(1);
    }

    let mut toml_file = TomlFile::read();

    match args.len() {
        2 => handle_no_args(&mut toml_file),
        3 => handle_three_args(&mut toml_file, &args),
        4 => handle_four_args(&mut toml_file, &args),
        5 => handle_five_args(&mut toml_file, &args),
        _ => {
            std::process::exit(1);
        }
    }
}

// 'cargo add-license'
// This command creates the LICENSE file based on the 'license' attribute inside Cargo.toml
// If you have the 'author' and 'year' attributes it will also replace the <year> and
// <authors> inside the LICENSE file.
fn handle_no_args(toml_file: &mut TomlFile) {
    if toml_file.license.is_none() {
        std::process::exit(1);
    }
    let license = supported_license(toml_file.license.as_ref().unwrap());
    create_license_file(license, &toml_file.authors, &toml_file.year);
}

// Either: 
// 'cargo add-license MIT' 
// 'cargo add-license -y:2024' 
// 'cargo add-license -a:Mary_Johnson,Carl_Menger' 
// This command replaces the 'license', 'year' or 'authors' attributes inside your Cargo.toml
// The first one also create the license file and replace <year>, <authors> inside the file with 
// the 'year' and 'author' attributes of Cargo.toml, if they are not empty.
fn handle_three_args(toml_file: &mut TomlFile, args: &[String]) {
    if ["APACHE2", "GPL3", "MIT", "BSD2", "BSD3", "AGPL3", "GPL2", "LGPL2.1"].contains(&args[2].as_str()) {
        let license = args[2].clone();
        toml_file.license = Some(
            convert_license_name(&license[..]).to_owned()
        );
        toml_file.write();
        create_license_file(&license, &toml_file.authors, &toml_file.year);
    } else if args[2].starts_with("-y:") {
        let year = args[2][3..].to_string();
        toml_file.year = Some(year);
        toml_file.write();
    } else if args[2].starts_with("-a:") {
        let authors_str = &args[2][3..];
        let authors: Vec<String> = authors_str.split(',').map(|s| s.replace('_', " ")).collect();
        toml_file.authors = Some(authors);
        toml_file.write();
    }    
}

// Either: 
// 'cargo add-license MIT -y:2024'
// 'cargo add-license MIT -a:Marques'
// 'cargo add-license -y:2024 -a:Marques'
// 'cargo add-license -a:Marques -y:2024'
fn handle_four_args(toml_file: &mut TomlFile, args: &[String]) {
    if ["APACHE2", "GPL3", "MIT", "BSD2", "BSD3", "AGPL3", "GPL2", "LGPL2.1"].contains(&args[2].as_str()) {
        if args[3].starts_with("-y:") {
            let license = args[2].clone();
            let year = args[3][3..].to_string();
            toml_file.license = Some(
                convert_license_name(&license[..]).to_owned()
            );
            toml_file.year = Some(year);
            toml_file.write();
            create_license_file(&license, &toml_file.authors, &toml_file.year);
        } else if args[3].starts_with("-a:") {
            let license = args[2].clone();
            let authors: Vec<String> = args[3][3..].split(',').map(|s| s.replace('_', " ")).collect();
            toml_file.license = Some(
                convert_license_name(&license[..]).to_owned()
            );
            toml_file.authors = Some(authors);
            toml_file.write();
            create_license_file(&license, &toml_file.authors, &toml_file.year);
        }
    } else if args[2].starts_with("-a:") && args[3].starts_with("-y:") {
        let authors: Vec<String> = args[2][3..].split(',').map(|s| s.replace('_', " ")).collect();
        let year = args[3][3..].to_string();
        toml_file.authors = Some(authors);
        toml_file.year = Some(year);
        toml_file.write();
    } else if args[2].starts_with("-y:") && args[3].starts_with("-a:") {
        let year = args[2][3..].to_string();
        let authors: Vec<String> = args[3][3..].split(',').map(|s| s.replace('_', " ")).collect();
        toml_file.year = Some(year);
        toml_file.authors = Some(authors);
        toml_file.write();
    }
}

// Either: 
// 'cargo add-license MIT -y:2024 -a:Marques'
// 'cargo add-license MIT -a:Marques -y:2024'
fn handle_five_args(toml_file: &mut TomlFile, args: &[String]) {
    if ["APACHE2", "GPL3", "MIT", "BSD2", "BSD3", "AGPL3", "GPL2", "LGPL2.1"].contains(&args[2].as_str()) {
        if args[3].starts_with("-y:") && args[4].starts_with("-a:") {
            let license = args[2].clone();
            let year = args[3][3..].to_string();
            let authors: Vec<String> = args[4][3..].split(',').map(|s| s.replace('_', " ")).collect();
            toml_file.license = Some(
                convert_license_name(&license[..]).to_owned()
            );
            toml_file.year = Some(year);
            toml_file.authors = Some(authors);
            toml_file.write();
            create_license_file(&license, &toml_file.authors, &toml_file.year);
        } else if args[3].starts_with("-a:") && args[4].starts_with("-y:") {
            let license = args[2].clone();
            let authors: Vec<String> = args[3][3..].split(',').map(|s| s.replace('_', " ")).collect();
            let year = args[4][3..].to_string();
            toml_file.license = Some(
                convert_license_name(&license[..]).to_owned()
            );
            toml_file.authors = Some(authors);
            toml_file.year = Some(year);
            toml_file.write();
            create_license_file(&license, &toml_file.authors, &toml_file.year);
        }
    }
}

fn supported_license(license: &str) -> &str {
    return match license {
        "AGPL-3.0-only"=> "AGPL3",
        "Apache-2.0"   => "APACHE2",
        "BSD-2-Clause" => "BSD2",
        "BSD-3-Clause" => "BSD3",
        "GPL-2.0"      => "GPL2",
        "GPL-3.0"      => "GPL3",
        "LGPL-2.1"     => "LGPL2.1",
        "MIT"          => "MIT",
        _ => { std::process::exit(1); }
    };
}

fn convert_license_name(license: &str) -> &str {
    return match license {
        "AGPL3"   => "AGPL-3.0-only",
        "APACHE2" => "Apache-2.0",
        "BSD2"    => "BSD-2-Clause",
        "BSD3"    => "BSD-3-Clause",
        "GPL2"    => "GPL-2.0",
        "GPL3"    => "GPL-3.0",
        "LGPL2.1" => "LGPL-2.1",
        "MIT"     => "MIT",
        _ => ""
    };
}

fn create_license_file(license: &str, authors: &Option<Vec<String>>, year: &Option<String>) {
    let license_content = match license {
        "AGPL3"   => include_str!("../licenses/agpl-3_0"),
        "APACHE2" => include_str!("../licenses/apache-2_0"),
        "BSD2"    => include_str!("../licenses/bsd-2-clause"),
        "BSD3"    => include_str!("../licenses/bsd-3-clause"),
        "GPL2"    => include_str!("../licenses/gpl-2_0"),
        "GPL3"    => include_str!("../licenses/gpl-3_0"),
        "LGPL2.1" => include_str!("../licenses/lgpl-2_1"),
        "MIT"     => include_str!("../licenses/mit"),
        _ => {
            std::process::exit(1);
        }
    };

    let mut license_content = license_content.to_string();

    if let Some(authors) = authors {
        if !authors.is_empty() {
            let authors_str = authors.join(", ");
            license_content = license_content.replace("<authors>", &authors_str);
        }
    }

    if let Some(year) = year {
        license_content = license_content.replace("<year>", year);
    }

    let current_dir = env::current_dir().unwrap();
    let license_path = current_dir.join("LICENSE");
    let mut file = File::create(license_path).unwrap();
    file.write_all(license_content.as_bytes()).unwrap();
}
