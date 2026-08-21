mod tabs;

mod read;

use read::read;
use tabs::*;

use std::io::Write as _;

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=build/main.rs");
    println!("cargo:rerun-if-changed=build/read.rs");
    println!("cargo:rerun-if-changed=build/tabs.rs");
    let class_registry = read::<_, ClassIdRegistration>("ACI-Registry/class-id-registry.csv")?;
    let vendor_registry = read::<_, VendorIdRegistration>("ACI-Registry/vendor-id-registry.csv")?;

    let mut subclass_sets = Vec::new();

    for dir in std::fs::read_dir("ACI-Registry/well-known-subclasses")? {
        let dir = dir?;

        let path = dir.path();

        if path
            .file_name()
            .expect("huh")
            .to_string_lossy()
            .ends_with(".csv")
        {
            let stem = path
                .file_stem()
                .expect("we know this doesn't end in `/`, ok?")
                .to_str()
                .unwrap();

            let name = internal_name_to_variant(stem);

            let subclasses = read::<_, WellKnownSubclassRegistration>(&path)?;

            subclass_sets.push((name, subclasses));
        }
    }

    let mut base = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    base.push("tables-generated.rs");

    println!("cargo:rustc-env=TABLES_GENERATED={}", base.display());

    let mut output = std::fs::File::create(base)?;

    writeln!(output, "classes!{{")?;

    for class in class_registry {
        writeln!(output, "{}", class)?;
    }
    writeln!(output, "}}")?;

    writeln!(output, "vendors!{{")?;

    for class in vendor_registry {
        writeln!(output, "{}", class)?;
    }
    writeln!(output, "}}")?;

    for (class, subclasses) in subclass_sets {
        if !subclasses.is_empty() {
            writeln!(
                output,
                "well_known_subclass!{{{}Subclass @ {}:{{",
                class, class
            )?;
            for subclass in subclasses {
                writeln!(output, "{}", subclass)?;
            }
            writeln!(output, "}} }}")?;
        }
    }

    if std::env::var_os("CARGO_FEATURE_NON_AUTHORATIVE").is_some() {
        let mut base = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

        base.push("non-auth-tables-generated.rs");

        println!(
            "cargo:rustc-env=NON_AUTH_TABLES_GENERATED={}",
            base.display()
        );

        let mut output = std::fs::File::create(base)?;

        let mut non_auth_subclass_sets = Vec::new();
        let mut non_auth_product_sets = Vec::new();
        for dir in std::fs::read_dir("aci-tables/subclasses")? {
            let dir = dir?;

            let path = dir.path();

            if path
                .file_name()
                .expect("huh")
                .to_string_lossy()
                .ends_with(".csv")
            {
                let stem = path
                    .file_stem()
                    .expect("we know this doesn't end in `/`, ok?")
                    .to_str()
                    .unwrap();

                let name = internal_name_to_variant(stem);

                let subclasses = read::<_, OtherSubclassProductInfo>(&path)?;

                non_auth_subclass_sets.push((name, subclasses));
            }
        }

        for dir in std::fs::read_dir("aci-tables/products")? {
            let dir = dir?;

            let path = dir.path();

            if path
                .file_name()
                .expect("huh")
                .to_string_lossy()
                .ends_with(".csv")
            {
                let stem = path
                    .file_stem()
                    .expect("we know this doesn't end in `/`, ok?")
                    .to_str()
                    .unwrap();

                let name = internal_name_to_variant(stem);

                let subclasses = read::<_, OtherSubclassProductInfo>(&path)?;

                non_auth_product_sets.push((name, subclasses));
            }
        }

        for (class, subclasses) in non_auth_subclass_sets {
            if !subclasses.is_empty() {
                writeln!(
                    output,
                    "non_auth_subclass!{{{}Subclass @ {}:{{",
                    class, class
                )?;
                for subclass in subclasses {
                    writeln!(output, "{}", subclass)?;
                }
                writeln!(output, "}} }}")?;
            }
        }

        for (class, subclasses) in non_auth_product_sets {
            if !subclasses.is_empty() {
                writeln!(output, "non_auth_product!{{{}Product @ {}:{{", class, class)?;
                for subclass in subclasses {
                    writeln!(output, "{}", subclass)?;
                }
                writeln!(output, "}} }}")?;
            }
        }
    }

    Ok(())
}
