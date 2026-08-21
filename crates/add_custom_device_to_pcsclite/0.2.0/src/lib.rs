use plist;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::{fmt, fs};

pub const DEFAULT_VID: &'static str = "0xFEFF";
pub const DEFAULT_PID: &'static str = "0xFCFD";
pub const DEFAULT_NAME: &'static str = "Pico OpenPGP (Dirty VID:PID)";

const INFO_PLIST_PATH_ON_ARCH_LINUX: &'static str =
    "/usr/lib/pcsc/drivers/ifd-ccid.bundle/Contents/Info.plist";

fn get_plist_file() -> Result<plist::Value, String> {
    println!("Use path \"{INFO_PLIST_PATH_ON_ARCH_LINUX}\".");

    match plist::Value::from_file(INFO_PLIST_PATH_ON_ARCH_LINUX) {
        Ok(file) => Ok(file),
        Err(error) => Err(error.to_string()),
    }
}

fn get_array_from_plist<'a, 'b>(
    plist_file: &'a plist::Value,
    array_key: &'b str,
) -> Result<&'a Vec<plist::Value>, &'static str> {
    let array_of_entries: &Vec<plist::Value> = match plist_file.as_dictionary() {
        Some(plist_as_dictionary) => match plist_as_dictionary.get(array_key) {
            Some(array_as_value) => match array_as_value.as_array() {
                Some(array_of_entries) => array_of_entries,
                None => return Err("Can't get array of entries by key from Info.plist file"),
            },
            None => return Err("Can't get Value by key from Info.plist file"),
        },
        None => return Err("Can't get dictionary from Info.plist file"),
    };

    Ok(array_of_entries)
}

enum InfoPlistArrayKey {
    VendorID,
    ProductID,
    ProductName,
}

impl fmt::Display for InfoPlistArrayKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InfoPlistArrayKey::VendorID => write!(formatter, "VID"),
            InfoPlistArrayKey::ProductID => write!(formatter, "PID"),
            InfoPlistArrayKey::ProductName => write!(formatter, "product name"),
        }
    }
}

impl InfoPlistArrayKey {
    fn get_key(&self) -> &str {
        match self {
            InfoPlistArrayKey::VendorID => "ifdVendorID",
            InfoPlistArrayKey::ProductID => "ifdProductID",
            InfoPlistArrayKey::ProductName => "ifdFriendlyName",
        }
    }
}
fn search_for_entry_in_array(
    plist_file: &plist::Value,
    target_id: &String,
    array_key: InfoPlistArrayKey,
) -> Result<Vec<usize>, String> {
    let array_of_ids: &Vec<plist::Value> =
        match get_array_from_plist(&plist_file, array_key.get_key()) {
            Ok(array_of_ids) => array_of_ids,
            Err(error) => {
                eprintln!("Error: \"{}\".", error);
                return Err(format!("Can't get {array_key}s array from Info.plist file"));
            }
        };
    let mut found_id_indexes: Vec<usize> = Vec::new();
    for (index, id) in array_of_ids.iter().enumerate() {
        match id.as_string() {
            Some(id) => {
                if id == target_id {
                    found_id_indexes.push(index);
                    continue;
                } else {
                    continue;
                }
            }
            None => {
                return Err(format!(
                    "Can't get {array_key} from Info.plist file, maybe plist file is corrupted"
                ));
            }
        }
    }

    Ok(found_id_indexes)
}

pub fn get_product_name<'a, 'b>(
    plist_file: &'a plist::Value,
    index: &'b usize,
) -> Result<String, &'static str> {
    let array_of_product_names: &Vec<plist::Value> =
        match get_array_from_plist(&plist_file, InfoPlistArrayKey::ProductName.get_key()) {
            Ok(array_of_product_names) => array_of_product_names,
            Err(error) => {
                eprintln!("Error: \"{}\".", error);
                return Err("Can't get product names array from Info.plist file");
            }
        };

    match array_of_product_names.get(*index) {
        Some(product_name) => match product_name.as_string() {
            Some(product_name) => Ok(product_name.to_string()),
            None => {
                Err("Can't get product name from Info.plist file, maybe plist file is corrupted")
            }
        },
        None => Err("Length of array of product names is less than index"),
    }
}

/// Check is VID:PID entry exists and return the result as (existing, entry index in the array).
fn check_entry_existing<'a, 'b, 'c>(
    plist_file: &'a plist::Value,
    target_vid: &'b String,
    target_pid: &'c String,
) -> Result<(bool, Option<usize>), &'static str> {
    let vid_indexes: Vec<usize> =
        match search_for_entry_in_array(plist_file, target_vid, InfoPlistArrayKey::VendorID) {
            Ok(vid_indexes) => {
                if vid_indexes.is_empty() {
                    println!("VID not found. Not VID - not entry. So, entry not found.");
                    return Ok((false, None));
                };

                vid_indexes
            }
            Err(error) => {
                eprintln!("Error: \"{}\".", error);
                return Err("Can't search for VID");
            }
        };
    println!("Found VIDs at indexes: {:?}.", vid_indexes);

    let pid_indexes: Vec<usize> =
        match search_for_entry_in_array(plist_file, target_pid, InfoPlistArrayKey::ProductID) {
            Ok(pid_indexes) => {
                if pid_indexes.is_empty() {
                    println!("PID not found. Not PID - not entry. So, entry not found.");
                    return Ok((false, None));
                };

                pid_indexes
            }
            Err(error) => {
                eprintln!("Error: \"{}\".", error);
                return Err("Can't search for PID");
            }
        };
    println!("Found PIDs at indexes: {:?}.", pid_indexes);

    let mut filtered_pid_indexes: Vec<usize> = Vec::new();
    for pid_index in &pid_indexes {
        if vid_indexes.contains(pid_index) {
            filtered_pid_indexes.push(pid_index.clone());
        }
    }
    if filtered_pid_indexes.is_empty() {
        println!("No one PID relevant to VID found. So, entry not found.");
        return Ok((false, None));
    } else if filtered_pid_indexes.len() > 1 {
        println!("More than one PID relevant to VID found. Will use first one.");
        return Ok((true, Some(filtered_pid_indexes[0].clone())));
    }

    Ok((true, Some(filtered_pid_indexes[0].clone())))
}

pub fn run_check(target_vid: &String, target_pid: &String) -> Result<(), &'static str> {
    let file: plist::Value = match get_plist_file() {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error: \"{}\".", error);
            return Err("Can't open Info.plist file");
        }
    };

    let entry_index: usize = match check_entry_existing(&file, target_vid, target_pid) {
        Ok((_, entry_index)) => match entry_index {
            Some(entry_index) => entry_index,
            None => return Ok(()),
        },
        Err(error) => {
            eprintln!("Error: \"{}\".", error);
            return Err("Can't check entry existing");
        }
    };

    let product_name: String = match get_product_name(&file, &entry_index) {
        Ok(product_name) => product_name,
        Err(error) => {
            eprintln!("Error: \"{}\".", error);
            return Err("Can't get product name");
        }
    };
    println!("Found VID:PID at index \"{entry_index}\" with product name \"{product_name}\".",);

    Ok(())
}

fn get_mut_array_from_plist<'a, 'b>(
    plist_file: &'a mut plist::Value,
    array_key: &'b str,
) -> Result<&'a mut Vec<plist::Value>, &'static str> {
    let array_of_entries: &mut Vec<plist::Value> = match plist_file.as_dictionary_mut() {
        Some(plist_as_dictionary) => match plist_as_dictionary.get_mut(array_key) {
            Some(array_as_value) => match array_as_value.as_array_mut() {
                Some(array_of_entries) => array_of_entries,
                None => return Err("Can't get array of entries by key from Info.plist file"),
            },
            None => return Err("Can't get Value by key from Info.plist file"),
        },
        None => return Err("Can't get dictionary from Info.plist file"),
    };

    Ok(array_of_entries)
}

pub fn run_add(
    target_vid: &String,
    target_pid: &String,
    target_name: &String,
    check_existing: &bool,
) -> Result<(), &'static str> {
    let mut file: plist::Value = match get_plist_file() {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error: \"{}\".", error);
            return Err("Can't open Info.plist file");
        }
    };

    if *check_existing == true {
        match check_entry_existing(&file, target_vid, target_pid) {
            Ok((existing, _)) => {
                if existing == true {
                    println!("Entry already exists!");
                    return Ok(());
                }
            }
            Err(error) => {
                eprintln!("Error: \"{}\".", error);
                return Err("Can't check entry existing, can't go to next step");
            }
        };
    };

    println!("Patching list of VIDs!");
    let array_of_vids: &mut Vec<plist::Value> =
        match get_mut_array_from_plist(&mut file, "ifdVendorID") {
            Ok(array_of_vids) => array_of_vids,
            Err(error) => {
                eprintln!("Error: \"{}\".", error);
                return Err(
                    "Can't get array of VIDs from Info.plist file, maybe the file is corrupted",
                );
            }
        };
    array_of_vids.push(plist::Value::String(target_vid.to_string()));

    println!("Patching list of PIDs!");
    let array_of_pids: &mut Vec<plist::Value> =
        match get_mut_array_from_plist(&mut file, "ifdProductID") {
            Ok(array_of_pids) => array_of_pids,
            Err(error) => {
                eprintln!("Error: \"{}\".", error);
                return Err(
                    "Can't get array of PIDs from Info.plist file, maybe the file is corrupted",
                );
            }
        };
    array_of_pids.push(plist::Value::String(target_pid.to_string()));

    println!("Patching list of product names!");
    let array_of_product_names: &mut Vec<plist::Value> = match get_mut_array_from_plist(
        &mut file,
        "ifdFriendlyName",
    ) {
        Ok(array_of_product_names) => array_of_product_names,
        Err(error) => {
            eprintln!("Error: \"{}\".", error);
            return Err(
                "Can't get array of product names from Info.plist file, maybe the file is corrupted",
            );
        }
    };
    array_of_product_names.push(plist::Value::String(target_name.to_string()));

    let temp_plist_file_path: PathBuf =
        PathBuf::from(format!("{INFO_PLIST_PATH_ON_ARCH_LINUX}.tmp"));
    if let Err(error) = file.to_file_xml(&temp_plist_file_path) {
        eprintln!("Error: \"{}\".", error);
        return Err("Can't save to disk Info.plist temp file, maybe permission denied");
    };

    if let Err(error) = fs::rename(
        PathBuf::from(INFO_PLIST_PATH_ON_ARCH_LINUX),
        PathBuf::from(format!("{INFO_PLIST_PATH_ON_ARCH_LINUX}.bak")),
    ) {
        eprintln!("Error: \"{}\".", error);
        return Err("Can't create backup of Info.plist file, maybe permission denied");
    };
    println!(
        "Backup of previously version of Info.plist file saved as \"{INFO_PLIST_PATH_ON_ARCH_LINUX}.bak\"."
    );

    if let Err(error) = fs::rename(
        &temp_plist_file_path,
        PathBuf::from(INFO_PLIST_PATH_ON_ARCH_LINUX),
    ) {
        eprintln!("Error: \"{}\".", error);
        return Err("Can't save to disk Info.plist finally file, maybe permission denied");
    };

    Ok(())
}
