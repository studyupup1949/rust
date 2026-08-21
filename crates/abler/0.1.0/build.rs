use std::{env, fs, io::{self, Write as _}, path, fmt};

fn main() {
    build_abler_phf_sets().expect("Building Abler PHF set should succeed");
}

#[derive(Debug)]
enum BuildAblerPhfSetsError {
    GetEnv(env::VarError),
    CreateFile(io::Error),
    WriteToFile(io::Error)
}

impl fmt::Display for BuildAblerPhfSetsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GetEnv(err) => write!(f, "failed reading environment variable: {}", err),
            Self::CreateFile(err) => write!(f, "failed creating file: {}", err),
            Self::WriteToFile(err) => write!(f, "failed writing to file: {}", err),
        }
    }
}

include!{"src/cis.rs"}

include!{"src/kind.rs"}

fn build_abler_phf_sets() -> Result<(), BuildAblerPhfSetsError> {

    let env_path = env::var("OUT_DIR").map_err(BuildAblerPhfSetsError::GetEnv)?;
    let path = path::Path::new(&env_path).join("abler_set.rs");

    let mut file = io::BufWriter::new(fs::File::create(&path).map_err(BuildAblerPhfSetsError::CreateFile)?);
    
    let mut strs_set = phf_codegen::Map::new();

    for &kind in Kind::VARIANTS {
        let [negative, positive] = kind.names();
        strs_set.entry(Cis(*negative), "false");
        strs_set.entry(Cis(*positive), "true");
    }

    write!(
        &mut file,
        "impl Abler {{\n\tconst ALIASES: &'static phf::Map<cis::Cis<'static>, bool> = &{};\n}}",
        strs_set.build(),
    ).map_err(BuildAblerPhfSetsError::WriteToFile)
}