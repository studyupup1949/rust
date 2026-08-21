ADIF-IO
=======

![](https://img.shields.io/badge/No_AI_Code-passed-brightgreen)

ADIF-IO is a Rust crate to read and write ADIF formated files (.adi, .adif) for storing hamradio QSO data.

This project was my first practical use of Rust. It started as a completely new implementation based largely on Regex.

I was just curious how much faster ADIF can be read with Rust.
But this was so slow (slower than Python) that I ported the parsing code from my library PyADIF-File.

10,000 QSOs are now parsed in ~ 115 ms. With PyADIF-File it takes ~ 400 ms.

Meanwhile, I have completely rewritten the crate (except the parsing part) as my Rust journey went on.

Unlike PyADIF-File, it features type collation to the ADIF type, depending on the field name.
Maybe I'll port this to PyADIF-File?


API usage example
-----------------

    use std::fs;
    use adif_io::{DeserializeADI, Doc};
    
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
        println!("QSO count: {}", doc.iter_record().count());
        doc.iter_record().enumerate().for_each(|(i, qso)| println!("QSO {}: {:?}", i+1, qso));
    
        // Get a QSO and modify data
        let mut qso = doc.get_record_mut(5).expect("no QSO available");
        qso.insert("NOTES", "Test data".into());
    
        // Create a `Record` and add it, case for field names does not matter
        let qso = Record::from(vec![
            ("QSO_DATE", "20231009"),
            ("TIME_ON", "1245"),
            ("Call", "DK5XXX"),
            ("NAME", "Chris"),  // Upper case field name inserted
        ]);
        println!("Name: {:?}", qso.get("name").unwrap());  // Accessed field name with lower case
        doc.add_record(qso);
    }

Serialize/Deserialize to/from other formats
-------------------------------------------

ADIF-IO provides serialization/deserialization with feature `serde_impl` via [Serde](https://serde.rs/).
The result is an ADIF typed serialization (with the supported types).
Also see [adi2json](examples/adi2json.rs) and [json2adi](examples/json2adi.rs) as examples.

Example serde_impl

    "records": [
    {
      "CALL": {
        "String": "AA0AA/P"
      },
      "QSO_DATE": {
        "Date": "2019-01-30"
      },
      "TIME_ON": {
        "Time": "19:25"
      },

With the feature `serde_loose` you will get a loosely typed serialization
with a nicer representation but loosing most of the typing.

Example serde_loose

    "records": [
    {
      "CALL": "AA0AA/P",
      "QSO_DATE": "20190130",
      "TIME_ON": "1925",

Sorry, for playing around with features. Just learning and experimenting.


Use the crate
-------------

Add the following line to your `Cargo.toml`

    adif_io = { git = "https://codeberg.org/dragoncode/adif_io.git", tag = "v0.2.0" }

Change the tag name to the preferred release.

The crate provides two Serde implementations as separate features `serde_impl` and `serde_loose` (see above).

    adif_io = { git = "https://codeberg.org/dragoncode/adif_io.git", tag = "v0.2.0", features = ["serde_impl"]}

If you need API documentation run

    cargo doc --features serde_impl

License
-------

Written by Andreas Schawo, licensed under CC BY-SA 4.0.
To view a copy of this license, visit http://creativecommons.org/licenses/by-sa/4.0/
