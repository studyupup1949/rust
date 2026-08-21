#!/usr/bin/env bash

# requires hmmer && aa2nucaln in your PATH.
# Or edit this file.

mkdir output

for fasta in ./fastas/*.faa; do
    # first of all, make the nucleotide alignments
    file_name=$(basename ${fasta} .faa)
    aa2nucaln --fasta ${fasta} --ignore false > ./output/${file_name}.fna
    # then build the hmm files
    hmmbuild ./output/${file_name}.hmm ./output/${file_name}.fna
done