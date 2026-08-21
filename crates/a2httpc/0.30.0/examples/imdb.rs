use std::fs::File;

fn main() -> a2httpc::Result {
    env_logger::init();

    let resp = a2httpc::get("https://datasets.imdbws.com/title.basics.tsv.gz").send()?;
    println!("Status: {:?}", resp.status());
    println!("Headers:\n{:#?}", resp.headers());
    if resp.is_success() {
        let file = File::create("title.basics.tsv.gz")?;
        let n = resp.write_to(file)?;
        println!("Wrote {n} bytes to the file.");
    }

    Ok(())
}
