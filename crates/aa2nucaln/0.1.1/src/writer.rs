use crate::matrix;
use anyhow::Result;
use std::io::{BufRead, Read};

pub fn write_faa<R: Read + BufRead>(faa: bio::io::fasta::Reader<R>, ignore: bool) -> Result<()> {
    for record in faa.records() {
        let fasta_record = record?;
        let matrix = matrix::create_matrix(fasta_record.seq(), ignore);
        let t_mat = matrix::transpose(matrix)?;
        for (i, seq) in t_mat.iter().enumerate() {
            println!(">{}:{}\n{}", fasta_record.id(), i + 1usize, seq.join(""));
        }
    }

    Ok(())
}
