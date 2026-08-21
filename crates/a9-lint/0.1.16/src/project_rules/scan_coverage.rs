use std::path::{Path, PathBuf};

use crate::{
    ProjectRule, Rule as RuleTrait, Violation,
    workspace::{discover_scan_dirs, workspace_members},
};

pub struct Rule;

impl RuleTrait for Rule {
    fn name(&self) -> &'static str {
        "scan-coverage"
    }

    fn description(&self) -> &'static str {
        "Every workspace member's src/ must be reachable by the effective scan dirs"
    }
}

impl ProjectRule for Rule {
    fn detect(&self, project_root: &Path) -> Vec<Violation> {
        let members = workspace_members(project_root);

        if members.is_empty() {
            return vec![];
        }

        let scan_dirs = discover_scan_dirs(project_root);

        let mut violations = vec![];

        for member in &members {
            let member_src = project_root.join(member).join("src");

            if !member_src.exists() {
                continue;
            }

            let covered = scan_dirs
                .iter()
                .any(|scan| is_covered(project_root, scan, &member_src));

            if covered {
                continue;
            }

            violations
                .push(Violation {
                    line: 0,
                    message: format!(
                        "workspace member `{member}/src` is not covered by any scan dir; add `{member}/src` to `scan` in `[workspace.metadata.a9-lint]`"
                    ),
                    fixable: false,
                });
        }

        violations
    }
}

fn is_covered(root: &Path, scan_dir: &str, member_src: &PathBuf) -> bool {
    let scan_abs = root.join(scan_dir);

    member_src.starts_with(&scan_abs) || scan_abs.starts_with(member_src)
}
