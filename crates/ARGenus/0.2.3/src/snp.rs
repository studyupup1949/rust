//! SNP Verification Module
//!
//! Verifies point mutations in SNP-based ARG genes after targeted assembly.
//! Parses SNP information from gene names and checks if the mutation exists
//! in the assembled contig sequence.
//!
//! # SNP Gene Name Format
//! SNP-based ARG names follow the pattern: `gene_WtPosVar`
//! - `gene`: Base gene name (e.g., "rpoB", "gyrA")
//! - `Wt`: Wild-type amino acid (single letter)
//! - `Pos`: Position in the protein sequence
//! - `Var`: Variant (mutant) amino acid
//!
//! Examples: `rpoB_V146F`, `gyrA_S83L`, `parC_S80I`

use rustc_hash::FxHashMap;
use std::sync::LazyLock;

// ============================================================================
// SNP Status
// ============================================================================

/// Result of SNP verification.
#[derive(Debug, Clone, PartialEq)]
pub enum SnpStatus {
    /// Not a SNP-based gene (acquired gene)
    NotApplicable,
    /// SNP confirmed - mutant allele present
    Confirmed,
    /// Wild-type allele found - no resistance mutation
    WildType,
    /// Different amino acid found at position
    NovelVariant(char),
    /// SNP position not covered by contig
    NotCovered,
    /// Could not verify (alignment issue, etc.)
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

// ============================================================================
// SNP Info
// ============================================================================

/// Parsed SNP information from gene name.
#[derive(Debug, Clone)]
pub struct SnpInfo {
    /// Base gene name (e.g., "rpoB")
    #[allow(dead_code)]
    pub gene: String,
    /// Wild-type amino acid
    pub wildtype: char,
    /// Position in protein sequence (1-based)
    pub position: usize,
    /// Mutant amino acid
    pub mutant: char,
}

impl SnpInfo {
    /// Calculates the nucleotide position range for this SNP (0-based).
    /// Returns (start, end) for the codon.
    pub fn nucleotide_range(&self) -> (usize, usize) {
        let start = (self.position - 1) * 3;
        let end = start + 3;
        (start, end)
    }
}

// ============================================================================
// Codon Table
// ============================================================================

/// Standard genetic code codon table.
static CODON_TABLE: LazyLock<FxHashMap<&'static str, char>> = LazyLock::new(|| {
    let mut table = FxHashMap::default();
    // Phenylalanine (F)
    table.insert("TTT", 'F'); table.insert("TTC", 'F');
    // Leucine (L)
    table.insert("TTA", 'L'); table.insert("TTG", 'L');
    table.insert("CTT", 'L'); table.insert("CTC", 'L');
    table.insert("CTA", 'L'); table.insert("CTG", 'L');
    // Isoleucine (I)
    table.insert("ATT", 'I'); table.insert("ATC", 'I'); table.insert("ATA", 'I');
    // Methionine (M) - Start
    table.insert("ATG", 'M');
    // Valine (V)
    table.insert("GTT", 'V'); table.insert("GTC", 'V');
    table.insert("GTA", 'V'); table.insert("GTG", 'V');
    // Serine (S)
    table.insert("TCT", 'S'); table.insert("TCC", 'S');
    table.insert("TCA", 'S'); table.insert("TCG", 'S');
    table.insert("AGT", 'S'); table.insert("AGC", 'S');
    // Proline (P)
    table.insert("CCT", 'P'); table.insert("CCC", 'P');
    table.insert("CCA", 'P'); table.insert("CCG", 'P');
    // Threonine (T)
    table.insert("ACT", 'T'); table.insert("ACC", 'T');
    table.insert("ACA", 'T'); table.insert("ACG", 'T');
    // Alanine (A)
    table.insert("GCT", 'A'); table.insert("GCC", 'A');
    table.insert("GCA", 'A'); table.insert("GCG", 'A');
    // Tyrosine (Y)
    table.insert("TAT", 'Y'); table.insert("TAC", 'Y');
    // Stop codons (*)
    table.insert("TAA", '*'); table.insert("TAG", '*'); table.insert("TGA", '*');
    // Histidine (H)
    table.insert("CAT", 'H'); table.insert("CAC", 'H');
    // Glutamine (Q)
    table.insert("CAA", 'Q'); table.insert("CAG", 'Q');
    // Asparagine (N)
    table.insert("AAT", 'N'); table.insert("AAC", 'N');
    // Lysine (K)
    table.insert("AAA", 'K'); table.insert("AAG", 'K');
    // Aspartic acid (D)
    table.insert("GAT", 'D'); table.insert("GAC", 'D');
    // Glutamic acid (E)
    table.insert("GAA", 'E'); table.insert("GAG", 'E');
    // Cysteine (C)
    table.insert("TGT", 'C'); table.insert("TGC", 'C');
    // Tryptophan (W)
    table.insert("TGG", 'W');
    // Arginine (R)
    table.insert("CGT", 'R'); table.insert("CGC", 'R');
    table.insert("CGA", 'R'); table.insert("CGG", 'R');
    table.insert("AGA", 'R'); table.insert("AGG", 'R');
    // Glycine (G)
    table.insert("GGT", 'G'); table.insert("GGC", 'G');
    table.insert("GGA", 'G'); table.insert("GGG", 'G');
    table
});

/// Translates a codon (3 nucleotides) to amino acid.
pub fn translate_codon(codon: &str) -> Option<char> {
    let upper = codon.to_uppercase();
    CODON_TABLE.get(upper.as_str()).copied()
}

// ============================================================================
// SNP Parsing
// ============================================================================

/// Checks if a gene name represents a SNP-based ARG.
///
/// SNP genes have patterns like: gene_X###Y where X and Y are amino acids
/// and ### is a number.
#[allow(dead_code)]
pub fn is_snp_gene(gene_name: &str) -> bool {
    parse_snp_info(gene_name).is_some()
}

/// Parses SNP information from a gene name.
///
/// # Examples
/// ```
/// use argenus::snp::parse_snp_info;
///
/// let info = parse_snp_info("rpoB_V146F").unwrap();
/// assert_eq!(info.gene, "rpoB");
/// assert_eq!(info.wildtype, 'V');
/// assert_eq!(info.position, 146);
/// assert_eq!(info.mutant, 'F');
/// ```
pub fn parse_snp_info(gene_name: &str) -> Option<SnpInfo> {
    // Handle full AMR database format: gene_X###Y|CLASS|SUBCLASS|...
    let name = gene_name.split('|').next().unwrap_or(gene_name);

    // Find the last underscore that precedes a SNP pattern
    let parts: Vec<&str> = name.rsplitn(2, '_').collect();
    if parts.len() != 2 {
        return None;
    }

    let snp_part = parts[0];  // e.g., "V146F"
    let gene = parts[1];      // e.g., "rpoB"

    // SNP pattern: single letter + digits + single letter
    // Minimum length: 3 (e.g., "A1B")
    if snp_part.len() < 3 {
        return None;
    }

    let chars: Vec<char> = snp_part.chars().collect();

    // First char must be amino acid letter
    let wildtype = chars[0].to_ascii_uppercase();
    if !is_amino_acid(wildtype) {
        return None;
    }

    // Last char must be amino acid letter
    let mutant = chars[chars.len() - 1].to_ascii_uppercase();
    if !is_amino_acid(mutant) {
        return None;
    }

    // Middle part must be digits (position)
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

/// Checks if a character is a valid amino acid single-letter code.
fn is_amino_acid(c: char) -> bool {
    matches!(c, 'A' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'K' | 'L' |
                'M' | 'N' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'V' | 'W' | 'Y')
}

// ============================================================================
// SNP Verification
// ============================================================================

/// Verifies SNP presence in a contig aligned to an ARG reference.
///
/// # Arguments
/// * `contig_seq` - The assembled contig sequence
/// * `gene_name` - Full ARG gene name (e.g., "rpoB_V146F|RIFAMYCIN|...")
/// * `ref_start` - Reference (ARG) start position in alignment (0-based)
/// * `ref_end` - Reference (ARG) end position in alignment
/// * `contig_start` - Contig start position in alignment (0-based)
/// * `contig_end` - Contig end position in alignment
/// * `strand` - Alignment strand ('+' or '-')
///
/// # Returns
/// SNP verification status
pub fn verify_snp(
    contig_seq: &str,
    gene_name: &str,
    ref_start: usize,
    ref_end: usize,
    contig_start: usize,
    contig_end: usize,
    strand: char,
) -> SnpStatus {
    // Parse SNP info from gene name
    let snp_info = match parse_snp_info(gene_name) {
        Some(info) => info,
        None => return SnpStatus::NotApplicable,
    };

    // Calculate nucleotide position of SNP in reference
    let (snp_nt_start, snp_nt_end) = snp_info.nucleotide_range();

    // Check if SNP position is covered by alignment
    if snp_nt_start < ref_start || snp_nt_end > ref_end {
        return SnpStatus::NotCovered;
    }

    // Calculate corresponding position in contig
    let offset_in_ref = snp_nt_start - ref_start;
    let contig_snp_start = if strand == '+' {
        contig_start + offset_in_ref
    } else {
        // For reverse strand, positions are mirrored
        contig_end - offset_in_ref - 3
    };
    let contig_snp_end = contig_snp_start + 3;

    // Check bounds
    if contig_snp_end > contig_seq.len() {
        return SnpStatus::NotCovered;
    }

    // Extract codon from contig
    let codon_seq = &contig_seq[contig_snp_start..contig_snp_end];

    // Handle reverse strand - need reverse complement
    let codon = if strand == '-' {
        reverse_complement(codon_seq)
    } else {
        codon_seq.to_uppercase()
    };

    // Translate codon
    let amino_acid = match translate_codon(&codon) {
        Some(aa) => aa,
        None => return SnpStatus::Unverified(format!("invalid_codon:{}", codon)),
    };

    // Compare with expected
    if amino_acid == snp_info.mutant {
        SnpStatus::Confirmed
    } else if amino_acid == snp_info.wildtype {
        SnpStatus::WildType
    } else {
        SnpStatus::NovelVariant(amino_acid)
    }
}

/// Computes the reverse complement of a DNA sequence.
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_snp_info() {
        // Basic SNP pattern
        let info = parse_snp_info("rpoB_V146F").unwrap();
        assert_eq!(info.gene, "rpoB");
        assert_eq!(info.wildtype, 'V');
        assert_eq!(info.position, 146);
        assert_eq!(info.mutant, 'F');

        // With full AMR format
        let info = parse_snp_info("gyrA_S83L|QUINOLONE|QUINOLONE|J01M").unwrap();
        assert_eq!(info.gene, "gyrA");
        assert_eq!(info.wildtype, 'S');
        assert_eq!(info.position, 83);
        assert_eq!(info.mutant, 'L');

        // Single digit position
        let info = parse_snp_info("test_A1G").unwrap();
        assert_eq!(info.position, 1);

        // Non-SNP gene
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
        // Position 146 (1-based) = nucleotides 435-437 (0-based)
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
        // Create a mock contig with known codon at position
        // Position 2 (aa) = nucleotides 3-5 (0-based)
        // TTT = F (Phe), GTT = V (Val)

        // Test confirmed (mutant found)
        let contig = "ATGTTTGGG"; // Met-Phe-Gly
        let status = verify_snp(
            contig,
            "test_V2F",  // V at position 2 mutated to F
            0, 9,        // ref covers full sequence
            0, 9,        // contig covers full sequence
            '+',
        );
        assert_eq!(status, SnpStatus::Confirmed);

        // Test wildtype found
        let contig = "ATGGTTGGG"; // Met-Val-Gly (wildtype V at position 2)
        let status = verify_snp(
            contig,
            "test_V2F",
            0, 9,
            0, 9,
            '+',
        );
        assert_eq!(status, SnpStatus::WildType);

        // Test novel variant
        let contig = "ATGCTTGGG"; // Met-Leu-Gly (neither V nor F)
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
