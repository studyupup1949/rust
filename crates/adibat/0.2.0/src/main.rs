use std::io::Read;
use std::fs::File;
use std::collections::HashMap;
use regex;

fn print_usage() {
    println!("adi [file with reference energy] [other files]")
}

fn get_energy_in_line(line: &str) -> f64{
    let floet = regex::Regex::new(r"[0-9]\.[0-9]+").unwrap();
    line.split_whitespace().find(|s| floet.is_match(s)).unwrap().parse::<f64>().unwrap()
}

fn get_irrep_in_line(line: &str) -> &str {
    line.split_whitespace().nth(line.split_whitespace().position(|s| s == "DFTCI").unwrap()-1).unwrap()
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    #[test]
    fn test_get_energy_in_line() {
       assert_eq!(5.0, get_energy_in_line("#1 DFTCI 5.0 6.0 8.0"));
       assert_eq!(5.0, get_energy_in_line("# 1 DFTCI 5.0 6.0 8.0"))
    }
    #[test]
    fn test_get_irrep_in_line() {
       assert_eq!("12a", get_irrep_in_line("# 1 12a DFTCI 5.0 6.0 8.0"));
       assert_eq!("12a", get_irrep_in_line("#12 12a DFTCI 5.0 6.0 8.0"))
    }
}

fn main() -> Result<(), std::io::Error>{
    let mut passed_args: std::env::Args = std::env::args();
    // get rid of program call
    passed_args.next();
    let mut result_map: Vec<(String, Vec<(String, f64)>)> = Vec::new();
    let ref_ener: f64;
    let enl = regex::Regex::new("^#.*DFTCI").unwrap();
    if let Some(r) = passed_args.next() {
        let mut fh = File::open(r.clone())?;
        let mut string_buffer = String::new();
        fh.read_to_string(&mut string_buffer)?;
        let dftci_line = string_buffer.lines().find(|s| {
           enl.is_match(s)
        }).unwrap();
        ref_ener = get_energy_in_line(dftci_line);
        // result_vector.push((r, get_irrep_in_line(dftci_line).to_owned(), ref_ener));
    } else {
            print_usage();
            return Ok(());
    }
    while let Some(o) = passed_args.next() {
        let mut fh = File::open(o.clone())?;
        let mut sb = String::new();
        fh.read_to_string(&mut sb)?;
        let line_iter = sb.lines();
        let mut irrep_and_energies: Vec<(String, f64)> = Vec::new();
        for l in line_iter {
            if enl.is_match(l) {
                    let irrp = get_irrep_in_line(l).to_owned();
                    let enrg = get_energy_in_line(l) -ref_ener;
                    irrep_and_energies.push((irrp, enrg));
            }
        }
       result_map.push((o.clone(), irrep_and_energies));
    }
    for (file, v) in result_map.iter() {
        println!("{:^8}", file);
        println!("{:=^8}", "=");
        for (state, _) in v {
            print!("{:-^8}|", state);
        }
        println!();
        for (_, energ) in v {
            print!("{:^8.3}|", energ*27.2114);
        }
        println!();
        println!("{:=^8}", "=");
    }
    Ok(())
}
