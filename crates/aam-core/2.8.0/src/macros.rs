/// Generates an AAM-based config loader struct that reads `.aam` files from
/// a directory and exposes typed field accessors.
///
/// # Syntax
///
/// ```text
/// define_aam_loader! {
///     name: MyLoader,
///     dir: "packages/",
///
///     list: {
///         build_deps: String,
///         run_deps: String,
///     },
///     opt: {
///         updating_time: u64,
///         version: String,
///     },
///     req: {
///         description: String,
///     },
/// }
/// ```
///
/// **Field sections** (all optional, omit any empty section):
/// - `list: { name: InnerType }` → `fn name(&self, id) -> Result<Vec<InnerType>>`
///   Returns empty vec when the key is missing.
/// - `opt: { name: Type }` → `fn name(&self, id) -> Result<Option<Type>>`
///   Returns `None` when the key is missing.
/// - `req: { name: Type }` → `fn name(&self, id) -> Result<Type>`
///   Returns an error when the key is missing.
///
/// **Generated methods** (always present):
/// - `new()`, `load_dir(path)`, `load_aam(path)`
/// - `get_dep(id) -> Option<Arc<AAM>>`
/// - `get_all_ids() -> Vec<String>`
///
/// # Example
///
/// ```
/// use aam_core::define_aam_loader;
///
/// define_aam_loader! {
///     name: Deps,
///     dir: "packages",
///
///     list: {
///         build_deps: String,
///         tags: String,
///     },
///     opt: {
///         updating_time: u64,
///     },
///     req: {
///         description: String,
///     },
/// }
///
/// // The struct Deps is generated with:
/// // - Deps::new()
/// // - deps.load_dir("path/") -> anyhow::Result<()>
/// // - deps.get_dep("id") -> Option<Arc<AAM>>
/// // - deps.get_all_ids() -> Vec<String>
/// // - deps.build_deps("id") -> anyhow::Result<Vec<String>>
/// // - deps.tags("id") -> anyhow::Result<Vec<String>>
/// // - deps.updating_time("id") -> anyhow::Result<Option<u64>>
/// // - deps.description("id") -> anyhow::Result<String>
/// ```
#[macro_export]
macro_rules! define_aam_loader {
    // ── Entry: full form with all sections ──
    {
        name: $struct_name:ident,
        dir: $dir:expr,
        list: { $($list_field:ident: $list_ty:ty),* $(,)? },
        opt: { $($opt_field:ident: $opt_ty:ty),* $(,)? },
        req: { $($req_field:ident: $req_ty:ty),* $(,)? },
        $(,)?
    } => {
        define_aam_loader! {
            @impl
            name: $struct_name,
            list: { $($list_field: $list_ty),* },
            opt: { $($opt_field: $opt_ty),* },
            req: { $($req_field: $req_ty),* },
        }
    };
    // ── Two sections: list + opt ──
    {
        name: $struct_name:ident,
        dir: $dir:expr,
        list: { $($list_field:ident: $list_ty:ty),* $(,)? },
        opt: { $($opt_field:ident: $opt_ty:ty),* $(,)? },
        $(,)?
    } => {
        define_aam_loader! {
            @impl
            name: $struct_name,
            list: { $($list_field: $list_ty),* },
            opt: { $($opt_field: $opt_ty),* },
            req: { },
        }
    };
    // ── Two sections: list + req ──
    {
        name: $struct_name:ident,
        dir: $dir:expr,
        list: { $($list_field:ident: $list_ty:ty),* $(,)? },
        req: { $($req_field:ident: $req_ty:ty),* $(,)? },
        $(,)?
    } => {
        define_aam_loader! {
            @impl
            name: $struct_name,
            list: { $($list_field: $list_ty),* },
            opt: { },
            req: { $($req_field: $req_ty),* },
        }
    };
    // ── Two sections: opt + req ──
    {
        name: $struct_name:ident,
        dir: $dir:expr,
        opt: { $($opt_field:ident: $opt_ty:ty),* $(,)? },
        req: { $($req_field:ident: $req_ty:ty),* $(,)? },
        $(,)?
    } => {
        define_aam_loader! {
            @impl
            name: $struct_name,
            list: { },
            opt: { $($opt_field: $opt_ty),* },
            req: { $($req_field: $req_ty),* },
        }
    };
    // ── One section: list ──
    {
        name: $struct_name:ident,
        dir: $dir:expr,
        list: { $($list_field:ident: $list_ty:ty),* $(,)? },
        $(,)?
    } => {
        define_aam_loader! {
            @impl
            name: $struct_name,
            list: { $($list_field: $list_ty),* },
            opt: { },
            req: { },
        }
    };
    // ── One section: opt ──
    {
        name: $struct_name:ident,
        dir: $dir:expr,
        opt: { $($opt_field:ident: $opt_ty:ty),* $(,)? },
        $(,)?
    } => {
        define_aam_loader! {
            @impl
            name: $struct_name,
            list: { },
            opt: { $($opt_field: $opt_ty),* },
            req: { },
        }
    };
    // ── One section: req ──
    {
        name: $struct_name:ident,
        dir: $dir:expr,
        req: { $($req_field:ident: $req_ty:ty),* $(,)? },
        $(,)?
    } => {
        define_aam_loader! {
            @impl
            name: $struct_name,
            list: { },
            opt: { },
            req: { $($req_field: $req_ty),* },
        }
    };
    // ── No fields ──
    {
        name: $struct_name:ident,
        dir: $dir:expr,
        $(,)?
    } => {
        define_aam_loader! {
            @impl
            name: $struct_name,
            list: { },
            opt: { },
            req: { },
        }
    };
    // ── Internal implementation ──
    {
        @impl
        name: $struct_name:ident,
        list: { $($list_field:ident: $list_ty:ty),* $(,)? },
        opt: { $($opt_field:ident: $opt_ty:ty),* $(,)? },
        req: { $($req_field:ident: $req_ty:ty),* $(,)? },
        $(,)?
    } => {
        pub struct $struct_name {
            deps: ::std::collections::HashMap<String, ::std::sync::Arc<$crate::aam::AAM>>,
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }

        #[allow(dead_code)]
        impl $struct_name {
            pub fn new() -> Self {
                $struct_name {
                    deps: ::std::collections::HashMap::new(),
                }
            }

            pub fn load_dir<P: AsRef<::std::path::Path>>(&mut self, path: P) -> ::anyhow::Result<()> {
                let files: Vec<_> = ::anyhow::Context::with_context(
                    ::std::fs::read_dir(&path),
                    || format!("Failed to read dir: {}", path.as_ref().display())
                )?
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .filter(|p| {
                        p.is_file()
                            && p.extension().and_then(|s| s.to_str()) == Some("aam")
                            && p.file_name().and_then(|s| s.to_str()) != Some("config.aam")
                    })
                    .collect();

                for file in files {
                    self.load_aam(&file)?;
                }

                Ok(())
            }

            pub fn load_aam<P: AsRef<::std::path::Path>>(&mut self, path: P) -> ::anyhow::Result<()> {
                let model = $crate::aam::AAM::load(&path)
                    .map_err(|e| ::anyhow::anyhow!("Failed to load {}: {:?}", path.as_ref().display(), e))?;

                let id = model
                    .get("id")
                    .ok_or_else(|| ::anyhow::anyhow!("No 'id' in {}", path.as_ref().display()))?
                    .to_string();

                self.deps.insert(id, ::std::sync::Arc::new(model));
                Ok(())
            }

            pub fn get_dep(&self, id: &str) -> Option<::std::sync::Arc<$crate::aam::AAM>> {
                self.deps.get(id).cloned()
            }

            pub fn get_all_ids(&self) -> Vec<String> {
                self.deps.keys().cloned().collect()
            }

            // ── List field getters (empty vec when missing) ──
            $(
                pub fn $list_field(&self, id: &str) -> ::anyhow::Result<Vec<$list_ty>> {
                    let target = self
                        .deps
                        .get(id)
                        .ok_or_else(|| ::anyhow::anyhow!("Package '{}' not found", id))?;

                    let Some(raw) = target.get(stringify!($list_field)) else {
                        return Ok(::std::vec::Vec::new());
                    };

                    let parsed = $crate::found_value::FoundValue::new(raw)
                        .parse_list::<$list_ty>();
                    match parsed {
                        Some(Ok(v)) => Ok(v),
                        Some(Err(_)) => Err(::anyhow::anyhow!(
                            "Failed to parse list '{}' in package '{}'. Raw: {:?}",
                            stringify!($list_field), id, raw
                        )),
                        None => Err(::anyhow::anyhow!(
                            "Expected list format for '{}' in package '{}'. Raw: {:?}",
                            stringify!($list_field), id, raw
                        )),
                    }
                }
            )*

            // ── Optional scalar getters (None when missing) ──
            $(
                pub fn $opt_field(&self, id: &str) -> ::anyhow::Result<Option<$opt_ty>> {
                    let target = self
                        .deps
                        .get(id)
                        .ok_or_else(|| ::anyhow::anyhow!("Package '{}' not found", id))?;

                    let Some(raw) = target.get(stringify!($opt_field)) else {
                        return Ok(None);
                    };

                    let parsed = raw.parse::<$opt_ty>().map(Some);
                    ::anyhow::Context::with_context(parsed, || {
                            format!(
                                "Failed to parse '{}' as {} for package '{}'. Raw: {:?}",
                                stringify!($opt_field), stringify!($opt_ty), id, raw
                            )
                        })
                }
            )*

            // ── Required scalar getters (error when missing) ──
            $(
                pub fn $req_field(&self, id: &str) -> ::anyhow::Result<$req_ty> {
                    let target = self
                        .deps
                        .get(id)
                        .ok_or_else(|| ::anyhow::anyhow!("Package '{}' not found", id))?;

                    let raw = target
                        .get(stringify!($req_field))
                        .ok_or_else(|| ::anyhow::anyhow!(
                            "Required field '{}' not found in package '{}'",
                            stringify!($req_field), id
                        ))?;

                    let parsed = raw.parse::<$req_ty>();
                    ::anyhow::Context::with_context(parsed, || {
                            format!(
                                "Failed to parse '{}' as {} for package '{}'. Raw: {:?}",
                                stringify!($req_field), stringify!($req_ty), id, raw
                            )
                        })
                }
            )*
        }
    };
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    define_aam_loader! {
        name: TestDeps,
        dir: "does_not_matter",

        list: {
            build_deps: String,
        },
        opt: {
            version: u64,
        },
        req: {
            name: String,
        },
    }

    #[test]
    fn test_define_aam_loader_basic() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("pkg.aam");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(
            f,
            "id = my-pkg\nname = hello\nbuild_deps = [dep-a, dep-b]\nversion = 42\n"
        )
        .unwrap();
        drop(f);

        let mut deps = TestDeps::new();
        deps.load_aam(&file_path).unwrap();

        let name = deps.name("my-pkg").unwrap();
        assert_eq!(name, "hello");

        let bdeps = deps.build_deps("my-pkg").unwrap();
        assert_eq!(bdeps, vec!["dep-a", "dep-b"]);

        let ver = deps.version("my-pkg").unwrap();
        assert_eq!(ver, Some(42));

        let ids = deps.get_all_ids();
        assert_eq!(ids, vec!["my-pkg"]);
    }

    #[test]
    fn test_define_aam_loader_missing_opt() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("pkg2.aam");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "id = pkg2\nname = test\nbuild_deps = []\n").unwrap();
        drop(f);

        let mut deps = TestDeps::new();
        deps.load_aam(&file_path).unwrap();

        let ver = deps.version("pkg2").unwrap();
        assert_eq!(ver, None);

        let bdeps = deps.build_deps("pkg2").unwrap();
        assert!(bdeps.is_empty());
    }

    #[test]
    fn test_define_aam_loader_load_dir() {
        let temp = tempfile::tempdir().unwrap();
        for i in 1..=3 {
            let file_path = temp.path().join(format!("pkg{i}.aam"));
            let mut f = std::fs::File::create(&file_path).unwrap();
            writeln!(f, "id = pkg{i}\nname = package-{i}\nbuild_deps = []\n").unwrap();
            drop(f);
        }

        let mut deps = TestDeps::new();
        deps.load_dir(temp.path()).unwrap();

        let ids = deps.get_all_ids();
        assert_eq!(ids.len(), 3);
        assert_eq!(deps.name("pkg2").unwrap(), "package-2");
    }
}
