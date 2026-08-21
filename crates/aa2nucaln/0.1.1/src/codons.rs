// eugh
pub fn get_codon_string(aa: u8) -> Vec<&'static str> {
    let x = match aa {
        65u8 => vec![
            // A
            "GCT", "GCC", "GCA", "GCG", "GCT", "GCC", "GCA", "GCG", "GCT", "GCC", "GCA", "GCG",
        ],
        67u8 => vec![
            // C
            "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC",
        ],
        68u8 => vec![
            // D
            "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC",
        ],
        69u8 => vec![
            // E
            "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG",
        ],
        70u8 => vec![
            // F
            "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC",
        ],
        71u8 => vec![
            // G
            "GGT", "GGC", "GGA", "GGG", "GGT", "GGC", "GGA", "GGG", "GGT", "GGC", "GGA", "GGG",
        ],
        72u8 => vec![
            // H
            "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC",
        ],
        73u8 => vec![
            // I
            "ATT", "ATC", "ATA", "ATT", "ATC", "ATA", "ATT", "ATC", "ATA", "ATT", "ATC", "ATA",
        ],
        75u8 => vec![
            // K
            "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG",
        ],
        76u8 => vec![
            // L
            "TTA", "TTG", "CTT", "CTC", "CTA", "CTG", "TTA", "TTG", "CTT", "CTC", "CTA", "CTG",
        ],
        77u8 => vec![
            // M
            "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG",
        ],
        78u8 => vec![
            // N
            "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC",
        ],
        80u8 => vec![
            // P
            "CCT", "CCC", "CCA", "CCG", "CCT", "CCC", "CCA", "CCG", "CCT", "CCC", "CCA", "CCG",
        ],
        81u8 => vec![
            // Q
            "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG",
        ],
        82u8 => vec![
            // R
            "CGT", "CGC", "CGA", "CGG", "AGA", "AGG", "CGT", "CGC", "CGA", "CGG", "AGA", "AGG",
        ],
        83u8 => vec![
            // S
            "TCT", "TCC", "TCA", "TCG", "AGT", "AGC", "TCT", "TCC", "TCA", "TCG", "AGT", "AGC",
        ],
        84u8 => vec![
            // T
            "ACT", "ACC", "ACA", "ACG", "ACT", "ACC", "ACA", "ACG", "ACT", "ACC", "ACA", "ACG",
        ],
        86u8 => vec![
            // V
            "GTT", "GTC", "GTA", "GTG", "GTT", "GTC", "GTA", "GTG", "GTT", "GTC", "GTA", "GTG",
        ],
        87u8 => vec![
            // W
            "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG",
        ],
        89u8 => vec![
            // Y
            "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC",
        ],
        _ => vec![
            "---", "---", "---", "---", "---", "---", "---", "---", "---", "---", "---", "---",
        ],
    };
    x
}

// match upper AND lower case?
pub fn get_codon_string_both(aa: u8) -> Vec<&'static str> {
    let x = match aa {
        65u8 => vec![
            // A
            "GCT", "GCC", "GCA", "GCG", "GCT", "GCC", "GCA", "GCG", "GCT", "GCC", "GCA", "GCG",
        ],
        67u8 => vec![
            // C
            "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC",
        ],
        68u8 => vec![
            // D
            "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC",
        ],
        69u8 => vec![
            // E
            "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG",
        ],
        70u8 => vec![
            // F
            "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC",
        ],
        71u8 => vec![
            // G
            "GGT", "GGC", "GGA", "GGG", "GGT", "GGC", "GGA", "GGG", "GGT", "GGC", "GGA", "GGG",
        ],
        72u8 => vec![
            // H
            "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC",
        ],
        73u8 => vec![
            // I
            "ATT", "ATC", "ATA", "ATT", "ATC", "ATA", "ATT", "ATC", "ATA", "ATT", "ATC", "ATA",
        ],
        75u8 => vec![
            // K
            "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG",
        ],
        76u8 => vec![
            // L
            "TTA", "TTG", "CTT", "CTC", "CTA", "CTG", "TTA", "TTG", "CTT", "CTC", "CTA", "CTG",
        ],
        77u8 => vec![
            // M
            "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG",
        ],
        78u8 => vec![
            // N
            "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC",
        ],
        80u8 => vec![
            // P
            "CCT", "CCC", "CCA", "CCG", "CCT", "CCC", "CCA", "CCG", "CCT", "CCC", "CCA", "CCG",
        ],
        81u8 => vec![
            // Q
            "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG",
        ],
        82u8 => vec![
            // R
            "CGT", "CGC", "CGA", "CGG", "AGA", "AGG", "CGT", "CGC", "CGA", "CGG", "AGA", "AGG",
        ],
        83u8 => vec![
            // S
            "TCT", "TCC", "TCA", "TCG", "AGT", "AGC", "TCT", "TCC", "TCA", "TCG", "AGT", "AGC",
        ],
        84u8 => vec![
            // T
            "ACT", "ACC", "ACA", "ACG", "ACT", "ACC", "ACA", "ACG", "ACT", "ACC", "ACA", "ACG",
        ],
        86u8 => vec![
            // V
            "GTT", "GTC", "GTA", "GTG", "GTT", "GTC", "GTA", "GTG", "GTT", "GTC", "GTA", "GTG",
        ],
        87u8 => vec![
            // W
            "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG",
        ],
        89u8 => vec![
            // Y
            "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC",
        ],
        97u8 => vec![
            // a
            "GCT", "GCC", "GCA", "GCG", "GCT", "GCC", "GCA", "GCG", "GCT", "GCC", "GCA", "GCG",
        ],
        99u8 => vec![
            // c
            "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC", "TGT", "TGC",
        ],
        100u8 => vec![
            // d
            "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC", "GAT", "GAC",
        ],
        101u8 => vec![
            // e
            "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG", "GAA", "GAG",
        ],
        102u8 => vec![
            // f
            "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC", "TTT", "TTC",
        ],
        103u8 => vec![
            // g
            "GGT", "GGC", "GGA", "GGG", "GGT", "GGC", "GGA", "GGG", "GGT", "GGC", "GGA", "GGG",
        ],
        104u8 => vec![
            // h
            "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC", "CAT", "CAC",
        ],
        105u8 => vec![
            // i
            "ATT", "ATC", "ATA", "ATT", "ATC", "ATA", "ATT", "ATC", "ATA", "ATT", "ATC", "ATA",
        ],
        107u8 => vec![
            // k
            "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG", "AAA", "AAG",
        ],
        108u8 => vec![
            // l
            "TTA", "TTG", "CTT", "CTC", "CTA", "CTG", "TTA", "TTG", "CTT", "CTC", "CTA", "CTG",
        ],
        109u8 => vec![
            // m
            "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG", "ATG",
        ],
        110u8 => vec![
            // n
            "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC", "AAT", "AAC",
        ],
        112u8 => vec![
            // p
            "CCT", "CCC", "CCA", "CCG", "CCT", "CCC", "CCA", "CCG", "CCT", "CCC", "CCA", "CCG",
        ],
        113u8 => vec![
            // q
            "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG", "CAA", "CAG",
        ],
        114u8 => vec![
            // r
            "CGT", "CGC", "CGA", "CGG", "AGA", "AGG", "CGT", "CGC", "CGA", "CGG", "AGA", "AGG",
        ],
        115u8 => vec![
            // s
            "TCT", "TCC", "TCA", "TCG", "AGT", "AGC", "TCT", "TCC", "TCA", "TCG", "AGT", "AGC",
        ],
        116u8 => vec![
            // t
            "ACT", "ACC", "ACA", "ACG", "ACT", "ACC", "ACA", "ACG", "ACT", "ACC", "ACA", "ACG",
        ],
        118u8 => vec![
            // v
            "GTT", "GTC", "GTA", "GTG", "GTT", "GTC", "GTA", "GTG", "GTT", "GTC", "GTA", "GTG",
        ],
        119u8 => vec![
            // w
            "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG", "TGG",
        ],
        121u8 => vec![
            // y
            "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC", "TAT", "TAC",
        ],
        _ => vec![
            "---", "---", "---", "---", "---", "---", "---", "---", "---", "---", "---", "---",
        ],
    };
    x
}
