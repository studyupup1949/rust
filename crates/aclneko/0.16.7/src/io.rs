use crate::acl::Acl;

use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{stdin, BufRead, BufReader};
use std::str;
use std::str::FromStr;

pub const POLICY_FILE_PATH: &str = "/etc/caitsith/policy/current";
pub const PATCH_DIR: &str = "/etc/caitsith/patch";
pub const POLICY_INTERFACE_PATH: &str = "/sys/kernel/security/caitsith/policy";
pub const QUERY_INTERFACE_PATH: &str = "/sys/kernel/security/caitsith/query";

pub fn open_syspolicy_write() -> Result<File, std::io::Error> {
    OpenOptions::new().write(true).open(POLICY_INTERFACE_PATH)
}

pub fn open_syspolicy_read() -> Result<File, std::io::Error> {
    OpenOptions::new().read(true).open(POLICY_INTERFACE_PATH)
}

/// read_policy_file tries to read policy file with given file name.
///
pub fn read_policy_file(file: &str) -> Result<Acl, String> {
    let mut policy_file = BufReader::new(File::open(file).map_err(|e| e.to_string())?);
    let mut buf = vec![];
    _ = policy_file.read_until(0, &mut buf);
    match Acl::from_str(str::from_utf8(&buf).unwrap()) {
        Ok(a) => Ok(a),
        Err(e) => Err(e.to_string()),
    }
}

pub fn apply_acl(acl: Acl) -> Result<(), String> {
    let mut fp = match open_syspolicy_write() {
        Ok(f) => f,
        Err(e) => return Err(e.to_string()),
    };
    let text = format!("{}", acl) + "\n";
    //     print!("{}", text);
    match fp.write(text.as_bytes()) {
        Ok(_) => {}
        Err(e) => return Err(e.to_string()),
    }
    Ok(())
}

pub fn apply_acl_atomic(acl: Acl) -> Result<(), String> {
    match acl.is_atomic() {
        true => apply_acl(acl),
        false => Err(String::from("Selected ACL is not atomic")),
    }
}

pub fn apply_acl_stdin() -> Result<(), String> {
    let mut reader = stdin();
    let mut s = vec![];
    _ = reader.read_to_end(&mut s);
    match Acl::from_str(&str::from_utf8(&s).unwrap()) {
        Ok(acl) => apply_acl(acl),
        Err(e) => Err(e.to_string()),
    }
}

/// clear_acl remove Acl specified with the headers in supplied Acl instance.
///
pub fn clear_acl(acl: &Acl) -> Result<(), String> {
    let mut fp = open_syspolicy_write().map_err(|e| e.to_string())?;
    for h in acl.parse_acl_headers() {
        let mut text = format!("delete {} acl {}", h.priority, h.op.as_str());
        for attr in &h.attr {
            text += &format!(" {}{}{}", attr.0.as_str(), attr.1.as_str(), attr.2);
        }
        text += "\n";
        //         print!("{}", text);
        match fp.write(text.as_bytes()) {
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

/// clear_rule removes rules which is contained in the patch.
/// Every header should be ensured to be defined in both of patch and the system.
///
pub fn clear_rule(patch: &Acl, dst: &Acl) -> Result<(), String> {
    //clear all acl which has cooresponding header with a patch
    let mut fp = open_syspolicy_write().map_err(|e| e.to_string())?;
    let mut new_acls = vec![];

    for h in patch.parse_acl_headers() {
        eprintln!("header: {}", h);
        let removing_block = patch
            .parse_acl_block_by_header(format!("{}", &h).as_str())
            .unwrap();
        let dst_block = dst
            .parse_acl_block_by_header(format!("{}", &h).as_str())
            .unwrap();

        for i in 0..removing_block.rule.len() {
            for j in 0..dst_block.rule.len() {
                if removing_block.rule[i] == dst_block.rule[j] {
                    break;
                } else {
                    new_acls.push(dst_block);
                }
            }
        }
    }

    clear_acl(&patch)?;
    fp.write(format!("{}", Acl::from(new_acls)).as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// remove_acl checks a patch has cooresponding header for target Acl and
/// removes ACLs from current system policy with target headers.
/// Use unmerge_acl to rule-based unmerging for ACLs.
///
pub fn remove_acl(patch: &Acl, from: &Acl) -> Result<(), String> {
    for h in patch.parse_acl_headers() {
        if from.has_header(&h) == false {
            return Err(format!("no such header: {}", h));
        }
    }
    clear_acl(&patch)
}

pub fn remove_acl_atomic(acl: &Acl, from: &Acl) -> Result<(), String> {
    match acl.is_atomic() {
        true => remove_acl(acl, from),
        false => Err(String::from("Selected ACL is not atomic")),
    }
}

pub fn clear_acl_from_stdin() -> Result<(), String> {
    let mut reader = stdin();
    let mut s = vec![];
    _ = reader.read_to_end(&mut s);
    match Acl::from_str(&str::from_utf8(&s).unwrap()) {
        Ok(acl) => clear_acl(&acl),
        Err(e) => Err(e.to_string()),
    }
}

/// unmerge_acl checks a patch has cooresponding header for target Acl and
/// remove for each rule defined in the patch from ACL blocks.
///
pub fn unmerge_acl(patch: &Acl, from: &Acl) -> Result<(), String> {
    for h in patch.parse_acl_headers() {
        if from.has_header(&h) == false {
            return Err(format!("no such header: {}", h));
        }
    }
    clear_rule(patch, from)
}

/// unmerge_acl_from_stdin reads policy from stdin and unmerges cooresponding rules
/// from target Acl.
///
pub fn unmerge_acl_from_stdin(from: &Acl) -> Result<(), String> {
    let mut reader = stdin();
    let mut s = vec![];
    _ = reader.read_to_end(&mut s);
    match Acl::from_str(&str::from_utf8(&s).unwrap()) {
        Ok(acl) => clear_rule(&acl, from),
        Err(e) => Err(e.to_string()),
    }
}

/// list_registered_patches lists predefined patches under PATCH_DIR.
///
pub fn list_registered_patches() -> Result<(), String> {
    eprintln!("\x1B[33m[registered patches]\x1B[0m");
    if let Ok(d) = fs::read_dir(PATCH_DIR) {
        for f in d {
            if let Ok(f) = f {
                println!("{}", f.file_name().to_str().unwrap_or("-"));
            }
        }
        Ok(())
    } else {
        Err(String::from("cannot read patches"))
    }
}
