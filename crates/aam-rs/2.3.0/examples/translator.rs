use aam_rs::translator::TOMLTranslator;

fn main() {
    let toml_data = r#"
host = "example.com"
port = 443

[database]
url = "postgresql://user:pass@localhost/db"
max_connections = 20

[logging]
level = "debug"
to_file = true
rotation = 7
"#;

    let builders = TOMLTranslator::toml_to_aam(toml_data).expect("Failed to translate TOML to AAM");
    let aam_strings: Vec<String> = builders.into_iter().map(|b| b.build()).collect();

    for (i, content) in aam_strings.iter().enumerate() {
        println!("=== Module {i} ===\n{content}\n");
    }
}
