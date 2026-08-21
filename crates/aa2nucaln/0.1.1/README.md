# aa2nucaln

Rust implementation of the amino acid alignment to a nucleotide alignment described here: https://academic.oup.com/bioinformatics/article/31/11/1836/2365396.

Primarily for use with hmmer.

## Usage

`aa2nucaln ./path/to/fasta > output.fna`

Output is to STDOUT.

Output of `aa2nucaln` help page:

```txt
Max Brown <mb39@sanger.ac.uk>
Convert an amino acid alignment to a nucelotide alignment.
For more info, see: https://academic.oup.com/bioinformatics/article/31/11/1836/2365396

USAGE:
    aa2nucaln [OPTIONS] [fasta]

ARGS:
    <fasta>    The reference amino acid alignment file in fasta format.

OPTIONS:
    -h, --help               Print help information
    -i, --ignore <ignore>    Should the lower-case letters in an amino acid alignment be ignored?
                             [default: true]
    -V, --version            Print version information
```