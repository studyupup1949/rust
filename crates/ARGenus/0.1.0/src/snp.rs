
use rustc_hash::FxHashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq)]
pub enum SnpStatus {

    NotApplicable,

    Confirmed,

    WildType,

    NovelVariant(char),

    NotCovered,

    Unverified(String),
}

impl std::fmt::Display for SnpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnpStatus::NotApplicable => write!(f, "Acquired"),
            SnpStatus::Confirmed => write!(f, "Confirmed"),
            SnpStatus::WildType => write!(f, "WildType"),
            SnpStatus::NovelVariant(aa) => write!(f, "Novel({})", aa),
            SnpStatus::NotCovered => write!(f, "NotCovered"),
            SnpStatus::Unverified(reason) => write!(f, "Unverified({})", reason),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SnpInfo {

    #[allow(dead_code)]
    pub gene: String,

    pub wildtype: char,

    pub position: usize,

    pub mutant: char,
}

impl SnpInfo {

    pub fn nucleotide_range(&self) -> (usize, usize) {
        let start = (self.position - 1) * 3;
        let end = start + 3;
        (start, end)
    }
}

static CODON_TABLE: LazyLock<FxHashMap<&'static str, char>> = LazyLock::new(|| {
    let mut table = FxHashMap::default();

    table.insert("TTT", 'F'); table.insert("TTC", 'F');

    table.insert("TTA", 'L'); table.insert("TTG", 'L');
    table.insert("CTT", 'L'); table.insert("CTC", 'L');
    table.insert("CTA", 'L'); table.insert("CTG", 'L');

    table.insert("ATT", 'I'); table.insert("ATC", 'I'); table.insert("ATA", 'I');

    table.insert("ATG", 'M');

    table.insert("GTT", 'V'); table.insert("GTC", 'V');
    table.insert("GTA", 'V'); table.insert("GTG", 'V');

    table.insert("TCT", 'S'); table.insert("TCC", 'S');
    table.insert("TCA", 'S'); table.insert("TCG", 'S');
    table.insert("AGT", 'S'); table.insert("AGC", 'S');

    table.insert("CCT", 'P'); table.insert("CCC", 'P');
    table.insert("CCA", 'P'); table.insert("CCG", 'P');

    table.insert("ACT", 'T'); table.insert("ACC", 'T');
    table.insert("ACA", 'T'); table.insert("ACG", 'T');

    table.insert("GCT", 'A'); table.insert("GCC", 'A');
    table.insert("GCA", 'A'); table.insert("GCG", 'A');

    table.insert("TAT", 'Y'); table.insert("TAC", 'Y');

    table.insert("TAA", '*'); table.insert("TAG", '*'); table.insert("TGA", '*');

    table.insert("CAT", 'H'); table.insert("CAC", 'H');

    table.insert("CAA", 'Q'); table.insert("CAG", 'Q');

    table.insert("AAT", 'N'); table.insert("AAC", 'N');

    table.insert("AAA", 'K'); table.insert("AAG", 'K');

    table.insert("GAT", 'D'); table.insert("GAC", 'D');

    table.insert("GAA", 'E'); table.insert("GAG", 'E');

    table.insert("TGT", 'C'); table.insert("TGC", 'C');

    table.insert("TGG", 'W');

    table.insert("CGT", 'R'); table.insert("CGC", 'R');
    table.insert("CGA", 'R'); table.insert("CGG", 'R');
    table.insert("AGA", 'R'); table.insert("AGG", 'R');

    table.insert("GGT", 'G'); table.insert("GGC", 'G');
    table.insert("GGA", 'G'); table.insert("GGG", 'G');
    table
});

pub fn translate_codon(codon: &str) -> Option<char> {
    let upper = codon.to_uppercase();
    CODON_TABLE.get(upper.as_str()).copied()
}

#[allow(dead_code)]
pub fn is_snp_gene(gene_name: &str) -> bool {
    parse_snp_info(gene_name).is_some()
}

pub fn parse_snp_info(gene_name: &str) -> Option<SnpInfo> {

    let name = gene_name.split('|').next().unwrap_or(gene_name);

    let parts: Vec<&str> = name.rsplitn(2, '_').collect();
    if parts.len() != 2 {
        return None;
    }

    let snp_part = parts[0];
    let gene = parts[1];

    if snp_part.len() < 3 {
        return None;
    }

    let chars: Vec<char> = snp_part.chars().collect();

    let wildtype = chars[0].to_ascii_uppercase();
    if !is_amino_acid(wildtype) {
        return None;
    }

    let mutant = chars[chars.len() - 1].to_ascii_uppercase();
    if !is_amino_acid(mutant) {
        return None;
    }

    let pos_str: String = chars[1..chars.len()-1].iter().collect();
    let position: usize = pos_str.parse().ok()?;

    if position == 0 {
        return None;
    }

    Some(SnpInfo {
        gene: gene.to_string(),
        wildtype,
        position,
        mutant,
    })
}

fn is_amino_acid(c: char) -> bool {
    matches!(c, 'A' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'K' | 'L' |
                'M' | 'N' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'V' | 'W' | 'Y')
}

pub fn verify_snp(
    contig_seq: &str,
    gene_name: &str,
    ref_start: usize,
    ref_end: usize,
    contig_start: usize,
    contig_end: usize,
    strand: char,
) -> SnpStatus {

    let snp_info = match parse_snp_info(gene_name) {
        Some(info) => info,
        None => return SnpStatus::NotApplicable,
    };

    let (snp_nt_start, snp_nt_end) = snp_info.nucleotide_range();

    if snp_nt_start < ref_start || snp_nt_end > ref_end {
        return SnpStatus::NotCovered;
    }

    let offset_in_ref = snp_nt_start - ref_start;
    let contig_snp_start = if strand == '+' {
        contig_start + offset_in_ref
    } else {

        contig_end - offset_in_ref - 3
    };
    let contig_snp_end = contig_snp_start + 3;

    if contig_snp_end > contig_seq.len() {
        return SnpStatus::NotCovered;
    }

    let codon_seq = &contig_seq[contig_snp_start..contig_snp_end];

    let codon = if strand == '-' {
        reverse_complement(codon_seq)
    } else {
        codon_seq.to_uppercase()
    };

    let amino_acid = match translate_codon(&codon) {
        Some(aa) => aa,
        None => return SnpStatus::Unverified(format!("invalid_codon:{}", codon)),
    };

    if amino_acid == snp_info.mutant {
        SnpStatus::Confirmed
    } else if amino_acid == snp_info.wildtype {
        SnpStatus::WildType
    } else {
        SnpStatus::NovelVariant(amino_acid)
    }
}

fn reverse_complement(seq: &str) -> String {
    seq.chars()
        .rev()
        .map(|c| match c.to_ascii_uppercase() {
            'A' => 'T',
            'T' => 'A',
            'G' => 'C',
            'C' => 'G',
            _ => 'N',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_snp_info() {

        let info = parse_snp_info("rpoB_V146F").unwrap();
        assert_eq!(info.gene, "rpoB");
        assert_eq!(info.wildtype, 'V');
        assert_eq!(info.position, 146);
        assert_eq!(info.mutant, 'F');

        let info = parse_snp_info("gyrA_S83L|QUINOLONE|QUINOLONE|J01M").unwrap();
        assert_eq!(info.gene, "gyrA");
        assert_eq!(info.wildtype, 'S');
        assert_eq!(info.position, 83);
        assert_eq!(info.mutant, 'L');

        let info = parse_snp_info("test_A1G").unwrap();
        assert_eq!(info.position, 1);

        assert!(parse_snp_info("blaTEM-1").is_none());
        assert!(parse_snp_info("tet(M)").is_none());
    }

    #[test]
    fn test_is_snp_gene() {
        assert!(is_snp_gene("rpoB_V146F"));
        assert!(is_snp_gene("gyrA_S83L|QUINOLONE"));
        assert!(!is_snp_gene("blaTEM-1"));
        assert!(!is_snp_gene("tet(M)"));
    }

    #[test]
    fn test_translate_codon() {
        assert_eq!(translate_codon("ATG"), Some('M'));
        assert_eq!(translate_codon("TTT"), Some('F'));
        assert_eq!(translate_codon("TTC"), Some('F'));
        assert_eq!(translate_codon("GTT"), Some('V'));
        assert_eq!(translate_codon("TAA"), Some('*'));
        assert_eq!(translate_codon("XXX"), None);
    }

    #[test]
    fn test_nucleotide_range() {
        let info = SnpInfo {
            gene: "test".to_string(),
            wildtype: 'V',
            position: 146,
            mutant: 'F',
        };

        assert_eq!(info.nucleotide_range(), (435, 438));

        let info2 = SnpInfo {
            gene: "test".to_string(),
            wildtype: 'A',
            position: 1,
            mutant: 'B',
        };
        assert_eq!(info2.nucleotide_range(), (0, 3));
    }

    #[test]
    fn test_verify_snp() {

        let contig = "ATGTTTGGG";
        let status = verify_snp(
            contig,
            "test_V2F",
            0, 9,
            0, 9,
            '+',
        );
        assert_eq!(status, SnpStatus::Confirmed);

        let contig = "ATGGTTGGG";
        let status = verify_snp(
            contig,
            "test_V2F",
            0, 9,
            0, 9,
            '+',
        );
        assert_eq!(status, SnpStatus::WildType);

        let contig = "ATGCTTGGG";
        let status = verify_snp(
            contig,
            "test_V2F",
            0, 9,
            0, 9,
            '+',
        );
        assert_eq!(status, SnpStatus::NovelVariant('L'));
    }

    #[test]
    fn test_snp_status_display() {
        assert_eq!(format!("{}", SnpStatus::Confirmed), "Confirmed");
        assert_eq!(format!("{}", SnpStatus::WildType), "WildType");
        assert_eq!(format!("{}", SnpStatus::NovelVariant('L')), "Novel(L)");
        assert_eq!(format!("{}", SnpStatus::NotApplicable), "Acquired");
    }
}
