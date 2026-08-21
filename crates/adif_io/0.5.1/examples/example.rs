use std::fs;
use adif_io::{DeserializeADI, Doc, Record, SerializeADI};

fn main() {
    let content = fs::read_to_string("test_data/big_testfile_1000.adi").expect("error reading ADI file: {err}");
    let mut doc = Doc::new();
    doc.deserialize_adi(&content).expect("could not deserialize from ADI");

    // Header info from file
    let header = doc.header();
    println!("Comment  : {}", header.comment());
    println!("Prog ID  : {}", header.program_id());
    println!("Prog Ver : {}", header.program_ver());

    // Count QSOs and print them
    println!("QSO count: {}", doc.iter_records().count());
    doc.iter_records().enumerate().for_each(|(i, qso)| println!("QSO {}: {}", i+1, qso));
    println!("---\n");

    // Get 6th QSO and modify data
    let qso = doc.get_record_mut(5).expect("no QSO available");
    qso.insert("NOTES", "New data".into());  // NOTES field
    qso["Call"] = "AB1CDE".into();  // Change callsign

    // Create a `Record` and add it, case for field names does not matter
    let qso = Record::from(vec![
        ("QSO_DATE", "20231009"),
        ("TIME_ON", "1245"),
        ("Call", "DK5XXX"),
        ("NAME", "Chris"),  // Upper case field name inserted
    ]);

    println!("QSO : {}", qso);
    println!("Name: {}", qso["name"]);  // Accessed field name with lower case
    println!("Date: {}", qso["qso_DATE"]);  // Accessed field name with mixed case

    // print as debug with type info for fields
    println!("QSO : {:?}", qso);
    println!("Name: {:?}", qso["NAME"]);
    println!("Date: {:?}", qso["QSO_DATE"]);

    doc.add_record(qso);

    // Serialize and write
    fs::write("example.adi", doc.serialize_adi()).expect("could not write ADI output file");
}