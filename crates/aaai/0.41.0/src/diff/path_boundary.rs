//! Capability-scoped discovery and reads for user-selected folders.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
#[cfg(not(windows))]
use cap_fs_ext::DirExt;
#[cfg(windows)]
use cap_fs_ext::OpenOptionsMaybeDirExt;
#[cfg(windows)]
use cap_fs_ext::OsMetadataExt;
#[cfg(unix)]
use cap_fs_ext::OpenOptionsSyncExt;
use cap_fs_ext::{FileTypeExt, FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
#[cfg(windows)]
use cap_std::fs::OpenOptionsExt;
use cap_std::fs::{Dir, OpenOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Identity {
    device: u64,
    inode: u64,
}

#[derive(Clone)]
pub(super) struct FileRef {
    parent: Arc<Dir>,
    name: OsString,
    identity: Identity,
    root_device: u64,
}

#[derive(Clone, Debug)]
pub(super) struct PathIssue {
    pub(super) code: &'static str,
    pub(super) detail: &'static str,
    pub(super) unreadable: bool,
}

#[derive(Clone)]
pub(super) enum Node {
    Directory,
    File(FileRef),
    Issue(PathIssue),
}

#[derive(Clone)]
pub(super) struct ObservedPath {
    pub(super) display: String,
    pub(super) node: Node,
}

pub(super) type PathMap = BTreeMap<PathBuf, ObservedPath>;

const ROOT_ERROR: &str =
    "[AAAI-ROOT-UNAVAILABLE] Selected folder must be a physical readable directory";

pub(super) fn collect(root: &Path) -> anyhow::Result<PathMap> {
    collect_with_hook(root, &|_, _| Ok(()))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpenPhase {
    Enumerate,
    Directory,
}

fn collect_with_hook(
    root: &Path,
    before_open: &dyn Fn(&Path, OpenPhase) -> std::io::Result<()>,
) -> anyhow::Result<PathMap> {
    let root = open_selected_root(root)?;
    let root_device = identity(&root.dir_metadata().context(ROOT_ERROR)?).device;
    let root = Arc::new(root);
    let mut paths = BTreeMap::new();
    walk(&root, Path::new(""), root_device, &mut paths, before_open)
        .map_err(|path_issue| anyhow::anyhow!(issue_text(&path_issue)))?;
    Ok(paths)
}

fn open_selected_root(path: &Path) -> anyhow::Result<Dir> {
    if path.as_os_str().is_empty() || path.components().next_back() == Some(Component::ParentDir) {
        anyhow::bail!(ROOT_ERROR);
    }

    // Retain the ambient authority only for acquisition of the selected root.
    // When there is a final name, open that one name without following it.
    if let Some(name) = path.file_name() {
        let parent_path = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent = Dir::open_ambient_dir(parent_path, ambient_authority()).context(ROOT_ERROR)?;
        open_directory_nofollow(&parent, name).context(ROOT_ERROR)
    } else {
        Dir::open_ambient_dir(path, ambient_authority()).context(ROOT_ERROR)
    }
}

fn walk(
    dir: &Arc<Dir>,
    relative_dir: &Path,
    root_device: u64,
    paths: &mut PathMap,
    before_open: &dyn Fn(&Path, OpenPhase) -> std::io::Result<()>,
) -> Result<(), PathIssue> {
    before_open(relative_dir, OpenPhase::Enumerate).map_err(|_| {
        issue(
            "AAAI-PATH-READ",
            "The directory could not be enumerated.",
            true,
        )
    })?;
    let entries = dir
        .entries()
        .map_err(|_| issue("AAAI-PATH-READ", "The directory could not be enumerated.", true))?;

    for entry in entries {
        let entry = entry.map_err(|_| {
            issue(
                "AAAI-PATH-READ",
                "A directory entry could not be enumerated.",
                true,
            )
        })?;
        let name = entry.file_name();
        let relative = relative_dir.join(&name);
        let display = display_path(&relative);

        let metadata = match dir.symlink_metadata(&name) {
            Ok(metadata) => metadata,
            Err(_) => {
                paths.insert(
                    relative,
                    ObservedPath {
                        display,
                        node: Node::Issue(issue(
                            "AAAI-PATH-METADATA",
                            "Entry metadata is unavailable.",
                            true,
                        )),
                    },
                );
                continue;
            }
        };
        let file_type = metadata.file_type();

        match entry_is_windows_reparse(dir, &name) {
            Ok(true) => {
                paths.insert(
                    relative,
                    ObservedPath {
                        display,
                        node: Node::Issue(issue(
                            "AAAI-PATH-REPARSE",
                            "Windows reparse points are not read.",
                            false,
                        )),
                    },
                );
                continue;
            }
            Err(_) => {
                paths.insert(
                    relative,
                    ObservedPath {
                        display,
                        node: Node::Issue(issue(
                            "AAAI-PATH-METADATA",
                            "Entry metadata is unavailable.",
                            true,
                        )),
                    },
                );
                continue;
            }
            Ok(false) => {}
        }

        if file_type.is_symlink() {
            paths.insert(
                relative,
                ObservedPath {
                    display,
                    node: Node::Issue(issue(
                        "AAAI-PATH-LINK",
                        "Link-like entries are not followed.",
                        false,
                    )),
                },
            );
            continue;
        }

        if file_type.is_dir() {
            let opened = before_open(&relative, OpenPhase::Directory)
                .and_then(|()| open_directory_nofollow(dir, &name));
            match classify_directory(dir, &name, root_device, opened) {
                Ok(child) => {
                    let child = Arc::new(child);
                    paths.insert(
                        relative.clone(),
                        ObservedPath {
                            display: display.clone(),
                            node: Node::Directory,
                        },
                    );
                    if let Err(path_issue) =
                        walk(&child, &relative, root_device, paths, before_open)
                    {
                        paths.insert(
                            relative,
                            ObservedPath {
                                display,
                                node: Node::Issue(path_issue),
                            },
                        );
                    }
                }
                Err(path_issue) => {
                    paths.insert(
                        relative,
                        ObservedPath {
                            display,
                            node: Node::Issue(path_issue),
                        },
                    );
                }
            }
            continue;
        }

        if file_type.is_file() {
            paths.insert(
                relative,
                ObservedPath {
                    display,
                    node: Node::File(FileRef {
                        parent: Arc::clone(dir),
                        name,
                        identity: identity(&metadata),
                        root_device,
                    }),
                },
            );
            continue;
        }

        let special = file_type.is_fifo()
            || file_type.is_socket()
            || file_type.is_block_device()
            || file_type.is_char_device();
        let detail = if special {
            "Special filesystem objects are not read."
        } else {
            "Unsupported filesystem objects are not read."
        };
        paths.insert(
            relative,
            ObservedPath {
                display,
                node: Node::Issue(issue("AAAI-PATH-SPECIAL", detail, false)),
            },
        );
    }
    Ok(())
}

fn classify_directory(
    parent: &Dir,
    name: &OsStr,
    root_device: u64,
    opened: std::io::Result<Dir>,
) -> Result<Dir, PathIssue> {
    let child = opened.map_err(|error| classify_open_failure(parent, name, error, true))?;
    let metadata = child.dir_metadata().map_err(|_| {
        issue(
            "AAAI-PATH-METADATA",
            "Directory metadata is unavailable.",
            true,
        )
    })?;
    if identity(&metadata).device != root_device {
        return Err(issue(
            "AAAI-PATH-XDEV",
            "Cross-filesystem descent is not allowed.",
            false,
        ));
    }
    Ok(child)
}

fn issue(code: &'static str, detail: &'static str, unreadable: bool) -> PathIssue {
    PathIssue {
        code,
        detail,
        unreadable,
    }
}

fn classify_open_failure(
    parent: &Dir,
    name: &OsStr,
    error: std::io::Error,
    expected_directory: bool,
) -> PathIssue {
    if error.kind() == std::io::ErrorKind::NotFound {
        return issue(
            "AAAI-PATH-RACE",
            "The entry was removed before it could be opened.",
            false,
        );
    }

    match parent.symlink_metadata(name) {
        Err(metadata_error) if metadata_error.kind() == std::io::ErrorKind::NotFound => issue(
            "AAAI-PATH-RACE",
            "The entry was removed before it could be opened.",
            false,
        ),
        Ok(metadata) => {
            let reparse = entry_is_windows_reparse(parent, name).unwrap_or(false);
            let wrong_kind = if expected_directory {
                !metadata.is_dir()
            } else {
                !metadata.is_file()
            };
            if metadata.file_type().is_symlink() || reparse || wrong_kind {
                issue(
                    "AAAI-PATH-RACE",
                    "The entry kind changed before it could be opened.",
                    false,
                )
            } else {
                issue(
                    "AAAI-PATH-READ",
                    "The entry could not be opened for reading.",
                    true,
                )
            }
        }
        Err(_) => issue(
            "AAAI-PATH-READ",
            "The entry could not be opened for reading.",
            true,
        ),
    }
}

pub(super) fn read_file(file: &FileRef) -> Result<Vec<u8>, PathIssue> {
    read_file_with(file, || Ok(()))
}

fn read_file_with(
    file: &FileRef,
    before_open: impl FnOnce() -> std::io::Result<()>,
) -> Result<Vec<u8>, PathIssue> {

    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    #[cfg(unix)]
    options.nonblock(true);
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;

        options
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .maybe_dir(true);
    }

    let cap_file = before_open()
        .and_then(|()| file.parent.open_with(&file.name, &options))
        .map_err(|error| classify_open_failure(&file.parent, &file.name, error, false))?;
    let metadata = cap_file.metadata().map_err(|_| {
        issue(
            "AAAI-PATH-METADATA",
            "Opened-file metadata is unavailable.",
            true,
        )
    })?;

    if windows_reparse(&metadata) {
        return Err(issue(
            "AAAI-PATH-RACE",
            "The file changed to a reparse point before it was read.",
            false,
        ));
    }
    if !metadata.is_file() {
        return Err(issue(
            "AAAI-PATH-RACE",
            "The entry is no longer a regular file.",
            false,
        ));
    }
    let opened_identity = identity(&metadata);
    if opened_identity.device != file.root_device || opened_identity != file.identity {
        return Err(issue(
            "AAAI-PATH-RACE",
            "The file identity changed before it was read.",
            false,
        ));
    }

    // Content is read from the exact handle whose type and identity were checked.
    let mut bytes = Vec::new();
    cap_file
        .take(u64::MAX)
        .read_to_end(&mut bytes)
        .map_err(|_| {
            issue(
                "AAAI-PATH-READ",
                "The opened regular file could not be read.",
                true,
            )
        })?;
    Ok(bytes)
}

#[cfg(unix)]
fn open_directory_nofollow(parent: &Dir, name: &OsStr) -> std::io::Result<Dir> {
    // cap-fs-ext implements this with directory-required, no-follow semantics.
    parent.open_dir_nofollow(name)
}

#[cfg(windows)]
fn open_directory_nofollow(parent: &Dir, name: &OsStr) -> std::io::Result<Dir> {
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT,
    };

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .follow(FollowSymlinks::No)
        .maybe_dir(true);
    let file = parent.open_with(name, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_dir()
        || OsMetadataExt::file_attributes(&metadata) & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "not a physical directory",
        ));
    }
    Ok(Dir::from_std_file(file.into_std()))
}

#[cfg(not(any(unix, windows)))]
fn open_directory_nofollow(parent: &Dir, name: &OsStr) -> std::io::Result<Dir> {
    parent.open_dir_nofollow(name)
}

#[cfg(windows)]
fn windows_reparse(metadata: &cap_std::fs::Metadata) -> bool {
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
    OsMetadataExt::file_attributes(metadata) & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn windows_reparse(_metadata: &cap_std::fs::Metadata) -> bool {
    false
}

#[cfg(windows)]
fn entry_is_windows_reparse(parent: &Dir, name: &OsStr) -> std::io::Result<bool> {
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;

    let mut options = OpenOptions::new();
    options
        .access_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .follow(FollowSymlinks::No)
        .maybe_dir(true);
    let file = parent.open_with(name, &options)?;
    let metadata = file.metadata()?;
    Ok(windows_reparse(&metadata))
}

#[cfg(not(windows))]
fn entry_is_windows_reparse(_parent: &Dir, _name: &OsStr) -> std::io::Result<bool> {
    Ok(false)
}

fn identity(metadata: &cap_std::fs::Metadata) -> Identity {
    Identity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}

pub(super) fn issue_text(issue: &PathIssue) -> String {
    format!("[{}] {}", issue.code, issue.detail)
}

fn display_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(display_component(name)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn display_component(name: &OsStr) -> String {
    if let Some(text) = name.to_str() {
        let mut display = String::new();
        for character in text.chars() {
            if character == '%' || character == '\\' || character.is_control() {
                let mut buffer = [0; 4];
                for byte in character.encode_utf8(&mut buffer).as_bytes() {
                    display.push_str(&format!("%{byte:02X}"));
                }
            } else {
                display.push(character);
            }
        }
        return display;
    }

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return name
            .as_bytes()
            .iter()
            .map(|byte| format!("%{byte:02X}"))
            .collect();
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        return name
            .encode_wide()
            .map(|unit| format!("%u{unit:04X}"))
            .collect();
    }
    #[allow(unreachable_code)]
    String::from("%INVALID")
}

#[cfg(test)]
mod tests;
