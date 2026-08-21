
// Basic mappings for Sanskrit Phonemes to integers
// Consonants (Vyambjana) - 33 Standard
const CONSONANTS: [&str; 33] = [
    "k", "kh", "g", "gh", "ṅ",
    "c", "ch", "j", "jh", "ñ",
    "ṭ", "ṭh", "ḍ", "ḍh", "ṇ",
    "t", "th", "d", "dh", "n",
    "p", "ph", "b", "bh", "m",
    "y", "r", "l", "v",
    "ś", "ṣ", "s", "h"
];

// Matras (Svara) - 16 Standard
// Short (Laghu): a, i, u, ṛ, ḷ
// Long (Guru): ā, ī, ū, ṝ, ḹ, e, ai, o, au, aṃ, aḥ
const MATRAS: [&str; 16] = [
    "a", "ā", "i", "ī", "u", "ū", "ṛ", "ṝ", 
    "ḷ", "ḹ", "e", "ai", "o", "au", "aṃ", "aḥ"
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatraWeight {
    Laghu, // Short
    Guru,  // Long
}



#[derive(Debug, Clone, PartialEq)]
pub struct Syllable {
    pub value: u16, // The integer value in Z_q
}

impl Syllable {
    pub fn new(val: u16) -> self::Syllable {
        Syllable { value: val }
    }

    pub fn to_sanskrit(&self) -> String {
        let v = self.value as usize;
        let c_idx = v / 16;
        let m_idx = v % 16;
        
        // Use modulo to wrap around for larger Q values to maintain uniform distribution across consonants
        let c = CONSONANTS[c_idx % 33];
        
        // Balanced Mapping for Crypto Uniformity:
        // Indices 0..8 (8 total) -> Map to Laghu (size 5)
        // Indices 8..16 (8 total) -> Map to Guru (size 11)
        let m_str = if m_idx < 8 {
             // Weights (0-7): Laghu
             // Cycle through the 5 Laghus: a, i, u, r, l (indices 0, 2, 4, 6, 8 in original array)
             let lags = [0, 2, 4, 6, 8];
             MATRAS[lags[m_idx % 5]]
        } else {
             // Weights (8-15): Guru
             // Cycle through the 11 Gurus
             let gurs = [1, 3, 5, 7, 9, 10, 11, 12, 13, 14, 15];
             MATRAS[gurs[(m_idx - 8) % 11]]
        };

        format!("{}{}", c, m_str)
    }
}

pub fn get_matra_weight(idx: usize) -> MatraWeight {
    if idx < 8 {
        MatraWeight::Laghu
    } else {
        MatraWeight::Guru
    }
}

pub fn encode_vector(vec: &[u16], use_meter: bool) -> String {
    let mut result = Vec::new();
    // In Chhandas (Meter) mode, we use the 'break' (Yati) as a dimension separator.
    // Standard Anushtubh meter has 4 quarters (padas) of 8 syllables.
    // For PoC, we can insert a danda '।' every 8 syllables if meter is enabled.
    
    for (i, val) in vec.iter().enumerate() {
        result.push(Syllable::new(*val).to_sanskrit());
        if use_meter && (i + 1) % 8 == 0 {
            result.push("।".to_string());
        }
    }
    result.join(" ")
}

pub fn decode_verse(verse: &str) -> Vec<u16> {
    // This is a naive decoder for the PoC
    // It assumes space separation and exact matches
    // In a real system, we'd have a robust parser
    
    verse.split_whitespace()
        .map(|s| {
            // Reverse lookup fallback
            // This is O(N) basic search, okay for PoC
            let mut val = 0;
            // Try to match easiest way: iterate all combos
            for c in 0..33 {
                for m_idx in 0..16 {
                   let candidate_val = (c * 16 + m_idx) as u16;
                   let built = Syllable::new(candidate_val).to_sanskrit();
                   if built == s {
                       return candidate_val;
                   }
                }
            }
            0 // Fail safe
        })
        .collect()
}
