use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use crate::config::{Config, Dotfile, DotfileType, Variable};
use crate::template;

pub struct Installer {
    config: Config,
    profile: String,
    /// Absolute path to the directory containing config.yaml
    config_dir: PathBuf,
    silent: bool,
}

impl Installer {
    pub fn new(config: Config, profile: String, config_dir: PathBuf, silent: bool) -> Self {
        Self {
            config,
            profile,
            config_dir,
            silent,
        }
    }

    pub fn install(&self) -> Result<(), String> {
        let start = std::time::Instant::now();
        let dotfile_names = self.resolve_dotfiles()?;

        for name in &dotfile_names {
            // safe: validate() guarantees all profile dotfile refs exist
            let dotfile = &self.config.dotfiles[name];

            if let Err(e) = self.install_dotfile(name, dotfile) {
                Self::err(&e);
                return Err(e);
            }
        }

        self.log(&format!(
            "installed {} dotfiles in {:.2?}",
            dotfile_names.len(),
            start.elapsed()
        ));

        Ok(())
    }

    fn log(&self, _msg: &str) {
        #[cfg(not(coverage))]
        if !self.silent {
            println!("{_msg}");
        }
    }

    fn err(msg: &str) {
        eprintln!("{msg}");
    }

    fn install_dotfile(&self, name: &str, dotfile: &Dotfile) -> Result<(), String> {
        match dotfile.dtype {
            DotfileType::Link => self.install_link(name, dotfile),
            DotfileType::Copy => self.install_copy(name, dotfile),
            DotfileType::Template => self.install_template(name, dotfile),
            DotfileType::LinkChildren => self.install_link_children(name, dotfile),
        }
    }

    /// Resolve src, create parent dirs, remove existing dst (only files/symlinks, NEVER directories)
    fn prepare_dst(&self, name: &str, dotfile: &Dotfile) -> Result<PathBuf, String> {
        let src = self.resolve_src(&dotfile.src);
        let dst = &dotfile.dst;

        if !src.exists() {
            return Err(format!(
                "dotfile '{name}': src does not exist: {}",
                src.display()
            ));
        }

        // safe: validate() ensures dst is never empty
        let parent = dst.parent().expect("dst must have a parent");
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "dotfile '{name}': failed to create dir {}: {e}",
                parent.display()
            )
        })?;

        // only remove files and symlinks, NEVER real directories
        if let Ok(meta) = dst.symlink_metadata()
            && (meta.file_type().is_symlink() || meta.is_file())
        {
            fs::remove_file(dst).map_err(|e| {
                format!(
                    "dotfile '{name}': failed to remove existing {}: {e}",
                    dst.display()
                )
            })?;
        }

        Ok(src)
    }

    fn install_link(&self, name: &str, dotfile: &Dotfile) -> Result<(), String> {
        let src = self.prepare_dst(name, dotfile)?;
        let dst = &dotfile.dst;

        self.log(&format!("link: {} -> {}", dst.display(), src.display()));

        unix_fs::symlink(&src, dst).map_err(|e| {
            format!(
                "dotfile '{name}': failed to symlink {} -> {}: {e}",
                dst.display(),
                src.display()
            )
        })?;

        Ok(())
    }

    fn install_copy(&self, name: &str, dotfile: &Dotfile) -> Result<(), String> {
        let src = self.resolve_src(&dotfile.src);
        let dst = &dotfile.dst;

        if !src.exists() {
            return Err(format!(
                "dotfile '{name}': src does not exist: {}",
                src.display()
            ));
        }

        self.log(&format!("copy: {} -> {}", src.display(), dst.display()));

        if src.is_dir() {
            copy_dir_recursive(&src, dst).map_err(|e| {
                format!(
                    "dotfile '{name}': failed to copy {} -> {}: {e}",
                    src.display(),
                    dst.display()
                )
            })?;
        } else {
            // for single files, create parent and overwrite
            let parent = dst.parent().expect("dst must have a parent");
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "dotfile '{name}': failed to create dir {}: {e}",
                    parent.display()
                )
            })?;
            fs::copy(&src, dst).map_err(|e| {
                format!(
                    "dotfile '{name}': failed to copy {} -> {}: {e}",
                    src.display(),
                    dst.display()
                )
            })?;
        }

        Ok(())
    }

    fn install_template(&self, name: &str, dotfile: &Dotfile) -> Result<(), String> {
        let src = self.prepare_dst(name, dotfile)?;
        let dst = &dotfile.dst;

        let variables = self.resolve_variables();

        if src.is_dir() {
            self.install_template_dir(name, &src, dst, &variables)?;
        } else {
            self.install_template_file(name, &src, dst, &variables)?;
        }

        Ok(())
    }

    fn install_template_file(
        &self,
        name: &str,
        src: &Path,
        dst: &Path,
        variables: &std::collections::HashMap<String, Variable>,
    ) -> Result<(), String> {
        let content = fs::read_to_string(src)
            .map_err(|e| format!("dotfile '{name}': failed to read {}: {e}", src.display()))?;

        let rendered = template::render(&content, variables, &self.profile)?;

        self.log(&format!("template: {} -> {}", src.display(), dst.display()));

        fs::write(dst, rendered)
            .map_err(|e| format!("dotfile '{name}': failed to write {}: {e}", dst.display()))?;

        Ok(())
    }

    fn install_template_dir(
        &self,
        name: &str,
        src: &Path,
        dst: &Path,
        variables: &std::collections::HashMap<String, Variable>,
    ) -> Result<(), String> {
        fs::create_dir_all(dst).map_err(|e| {
            format!(
                "dotfile '{name}': failed to create dir {}: {e}",
                dst.display()
            )
        })?;

        let entries = fs::read_dir(src).map_err(|e| {
            format!(
                "dotfile '{name}': failed to read dir {}: {e}",
                src.display()
            )
        })?;

        for entry in entries {
            let msg = format!("failed to read dir entry in {}", src.display());
            let entry = entry.expect(&msg);

            let child_src = entry.path();
            let child_dst = dst.join(entry.file_name());

            if child_src.is_dir() {
                self.install_template_dir(name, &child_src, &child_dst, variables)?;
            } else {
                self.install_template_file(name, &child_src, &child_dst, variables)?;
            }
        }

        Ok(())
    }

    /// Merge global variables with profile variables (profile overrides global)
    fn resolve_variables(&self) -> std::collections::HashMap<String, Variable> {
        let mut merged = self.config.variables.clone();

        // safe: install() already validates the profile exists
        let profile = self
            .config
            .profiles
            .get(&self.profile)
            .expect("profile must exist");
        for (key, value) in &profile.variables {
            merged.insert(key.clone(), value.clone());
        }

        merged
    }

    fn install_link_children(&self, name: &str, dotfile: &Dotfile) -> Result<(), String> {
        let src = self.resolve_src(&dotfile.src);
        let dst = &dotfile.dst;

        if !src.exists() {
            return Err(format!(
                "dotfile '{name}': src does not exist: {}",
                src.display()
            ));
        }

        if !src.is_dir() {
            return Err(format!(
                "dotfile '{name}': link_children requires a directory, got: {}",
                src.display()
            ));
        }

        fs::create_dir_all(dst).map_err(|e| {
            format!(
                "dotfile '{name}': failed to create dir {}: {e}",
                dst.display()
            )
        })?;

        let entries = fs::read_dir(&src).map_err(|e| {
            format!(
                "dotfile '{name}': failed to read dir {}: {e}",
                src.display()
            )
        })?;

        for entry in entries {
            let msg = format!("failed to read dir entry in {}", src.display());
            let entry = entry.expect(&msg);

            let child_src = entry.path();
            let child_dst = dst.join(entry.file_name());

            // remove existing child dst
            if child_dst.exists() || child_dst.symlink_metadata().is_ok() {
                let msg = format!("failed to remove existing {}", child_dst.display());
                remove_path(&child_dst).expect(&msg);
            }

            self.log(&format!(
                "link_children: {} -> {}",
                child_dst.display(),
                child_src.display()
            ));

            unix_fs::symlink(&child_src, &child_dst).map_err(|e| {
                format!(
                    "dotfile '{name}': failed to symlink {} -> {}: {e}",
                    child_dst.display(),
                    child_src.display()
                )
            })?;
        }

        Ok(())
    }

    /// Resolve dotfile src path relative to config_dir/dotpath
    fn resolve_src(&self, src: &Path) -> PathBuf {
        self.config_dir.join(&self.config.dotpath).join(src)
    }

    /// Collect all dotfile names for the profile, resolving includes recursively
    fn resolve_dotfiles(&self) -> Result<Vec<String>, String> {
        let mut result = Vec::new();
        let mut visited = Vec::new();
        self.collect_dotfiles(&self.profile, &mut result, &mut visited)?;
        Ok(result)
    }

    fn collect_dotfiles(
        &self,
        profile_name: &str,
        result: &mut Vec<String>,
        visited: &mut Vec<String>,
    ) -> Result<(), String> {
        if visited.contains(&profile_name.to_string()) {
            return Err(format!(
                "circular profile include detected: '{profile_name}'"
            ));
        }
        visited.push(profile_name.to_string());

        let profile = self
            .config
            .profiles
            .get(profile_name)
            .ok_or_else(|| format!("profile '{profile_name}' not found"))?;

        // resolve includes first (base dotfiles come before profile's own)
        for include in &profile.include {
            self.collect_dotfiles(include, result, visited)?;
        }

        for name in &profile.dotfiles {
            if !result.contains(name) {
                result.push(name.clone());
            }
        }

        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let msg = format!("failed to read dir entry in {}", src.display());
        let entry = entry.expect(&msg);
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<(), std::io::Error> {
    let meta = path.symlink_metadata()?;
    if meta.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}
