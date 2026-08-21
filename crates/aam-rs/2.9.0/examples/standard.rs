use aam_rs::aam::AAM;

fn main() {
    let parser = match AAM::parse(include_str!("standard.aam")) {
        Ok(aaml) => aaml,
        Err(e) => {
            eprint!("{:?}", e);
            return;
        }
    };

    let a_hits = parser.find("a");
    println!("{:?}", a_hits);

    let c_hits = parser.find("c");
    println!("{:?}", c_hits);

    if let Some((_, deep_ref)) = c_hits.first() {
        println!("{:?}", parser.find(deep_ref));
    }
}
