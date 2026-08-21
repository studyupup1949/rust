use aam_rs::aaml::AAML;

fn main() {
    let parser = match AAML::parse(include_str!("standard.aam")) {
        Ok(aaml) => aaml,
        Err(e) => {eprint!("{}", e); return;}
    };

    if let Some(a) = parser.find_obj("a") {
        println!("{}", a);
    } else {
        println!("a not found");
    }

    if let Some(d) = parser.find_obj("c") {
        println!("{}", d);
        if let Some(e) = parser.find_obj(&**d) {
            println!("{}", e);
        }
    } else {
        println!("a not found");
    }
}