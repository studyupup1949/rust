use crate::generators::compose;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// return true if created a new compose file
pub fn create_compose_file() -> std::io::Result<()> {

    if !Path::new("compose.yml").exists() {
        println!("creating a11ywatch compose.yml file");
        let mut file = File::create("compose.yml")?;
        file.write_all(&compose::generate_compose().as_bytes())?;
    }

    Ok(())
}