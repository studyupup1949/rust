//! Structures generated FROM AAM schemas using `schema_to_struct!`.
//!
//! The schema definitions live in `package_schema.aam` (see adjacent file).
//! At build time they are embedded and expanded into Rust types.
//!
//! This eliminates duplication: the schema is the single source of truth
//! for both runtime validation AND compile-time type definitions.

use aam_rs::from_aam::FromAam as _;
use aam_rs::schema_to_struct;

// ── All structs generated from a single schema file ──
// In real code you would use include_str!:
//   schema_to_struct!(include_str!("package_schema.aam"));
// For now, each @schema block must be separate invocations because
// proc-macros require string literals at call site.

schema_to_struct!("@schema Source  { dir: string, branch: string, url: string }");
schema_to_struct!("@schema Build   { system: string, command: string, args: list<string> }");
schema_to_struct!("@schema Install { commands: list<string>, prefix: string }");
schema_to_struct!("@schema Hooks   { pre_build: string, post_install*: string }");
schema_to_struct!(
    "@schema Package {
    id: string, name: string, type: string, version: string,
    source: Source, build: Build, install: Install, hooks: Hooks,
    build_deps: list<string>, run_deps: list<string>,
    updating_time: i32
}"
);

fn main() -> anyhow::Result<()> {
    let content = "
        id = cairo
        name = cairo
        type = core
        version = 0.0.0
        source = { dir = /build/cairo, branch = master, url = https://gitlab.freedesktop.org/cairo/cairo.git }
        build = { system = meson, command = meson, args = [setup, build, install] }
        install = { commands = [build, install], prefix = /hyprland }
        hooks = { pre_build = export CC=gcc, post_install =  }
        build_deps = [pixman, libpng, freetype, fontconfig]
        run_deps = [pixman, libpng, freetype, fontconfig]
        updating_time = 30
    ";

    let pkg = Package::from_aam_str(content)?;

    println!("=== Structs generated from @schemas ===");
    println!();
    println!("Package:");
    println!("  id:           {}", pkg.id);
    println!("  name:         {}", pkg.name);
    println!("  type:         {}", pkg.r#type);
    println!("  version:      {}", pkg.version);
    println!("  source.dir:   {}", pkg.source.dir);
    println!("  source.branch: {}", pkg.source.branch);
    println!("  source.url:   {}", pkg.source.url);
    println!("  build.system: {}", pkg.build.system);
    println!("  build.args:   {:?}", pkg.build.args);
    println!("  install.cmds: {:?}", pkg.install.commands);
    println!("  hooks.pre:    {}", pkg.hooks.pre_build);
    println!("  hooks.post:   {:?}", pkg.hooks.post_install);
    println!("  build_deps:   {:?}", pkg.build_deps);
    println!("  run_deps:     {:?}", pkg.run_deps);
    println!("  updating:     {}s", pkg.updating_time);
    println!();
    println!("(Schema file: examples/package_schema.aam)");

    Ok(())
}
