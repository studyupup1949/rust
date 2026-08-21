use crate::error::AcoustixError;

/// Number of consecutive hypothesis words checked for reconstructing one
/// reference word (e.g. "off" + "loading" -> "offloading"). Fixed at 2,
/// matching every real compound-word ASR split observed so far widen
/// only if a real 3-way split is measured.
const COMPOUND_SPLIT_WINDOW: usize = 2;

/// British `-ise` family suffixes mapped to their American `-ize` spelling
/// (checked longest-first so e.g. "isation" doesn't get short-circuited by
/// the shorter "ise" pattern). whisper.cpp transcribes British-spelled
/// source text ("personalised") in American spelling ("personalized")
/// regardless of what was actually spoken -- a standard, closed spelling
/// variance, not a content or synthesis error.
const BRITISH_ISE_SUFFIXES: &[(&str, &str)] = &[
    ("isations", "izations"),
    ("isation", "ization"),
    ("isable", "izable"),
    ("isers", "izers"),
    ("ising", "izing"),
    ("ised", "ized"),
    ("iser", "izer"),
    ("ise", "ize"),
];

fn normalize_british_ize_spelling(word: &str) -> String {
    for (british, american) in BRITISH_ISE_SUFFIXES {
        if let Some(stem) = word.strip_suffix(british) {
            return format!("{stem}{american}");
        }
    }
    word.to_string()
}

/// Tokenizes text by first treating hyphens and Unicode dashes as word
/// separators (ASR never reconstructs hyphens, so a written hyphenated
/// compound like "AI-narrated" is always transcribed as separate
/// space-separated words "AI narrated" -- stripping the hyphen as plain
/// punctuation instead would glue it into one token that can never match),
/// then splitting on whitespace, deunicode-normalizing each word to its
/// closest ASCII representation (so remaining Unicode punctuation like
/// curly quotes U+2018/2019/201C/201D collapse to their ASCII equivalents
/// before the punctuation filter below runs -- `is_ascii_punctuation`
/// alone never matches those, so a source/ASR-transcript pair differing
/// only in quote-mark style previously produced spurious WER errors),
/// stripping remaining ASCII punctuation, lowercasing, and normalizing
/// British `-ise` spellings to their American `-ize` equivalent.
fn tokenize(text: &str) -> Vec<String> {
    text.replace(
        [
            '-', '\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}', '\u{2014}',
        ],
        " ",
    )
    .split_whitespace()
    .map(|w| {
        let cleaned = deunicode::deunicode(w)
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect::<String>()
            .to_lowercase();
        normalize_british_ize_spelling(&cleaned)
    })
    .filter(|w| !w.is_empty())
    .collect()
}

/// Computes the Word Error Rate (WER) between a reference transcript and a hypothesis transcript
/// using the Levenshtein distance dynamic programming algorithm.
///
/// Returns the WER as a float (typically >= 0.0, where 0.0 is a perfect match).
pub fn word_error_rate(reference: &str, hypothesis: &str) -> Result<f32, AcoustixError> {
    let ref_words = tokenize(reference);
    let hyp_words = tokenize(hypothesis);

    if ref_words.is_empty() {
        return Err(AcoustixError::InvalidParameter(
            "Reference text must contain at least one valid word".to_string(),
        ));
    }

    let n = ref_words.len();
    let m = hyp_words.len();

    // 2D DP table
    let mut dp = vec![vec![0; m + 1]; n + 1];

    for i in 0..=n {
        dp[i][0] = i;
    }
    for j in 0..=m {
        dp[0][j] = j;
    }

    for i in 1..=n {
        for j in 1..=m {
            let cost = if ref_words[i - 1] == hyp_words[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1) // Deletion
                .min(dp[i][j - 1] + 1) // Insertion
                .min(dp[i - 1][j - 1] + cost); // Substitution

            // ASR never reconstructs a compound word's internal boundary --
            // ordinary uncommon or CamelCase compounds ("offloading",
            // "QuietScroll") are transcribed as separate words the same way
            // hyphenated ones are (see `tokenize`'s hyphen handling above).
            // If concatenating the last `COMPOUND_SPLIT_WINDOW` hypothesis
            // words exactly reconstructs this reference word, treat it as a
            // zero-cost match (a segmentation difference, not a content
            // error) rather than an insertion+substitution.
            if j >= COMPOUND_SPLIT_WINDOW {
                let merged: String = hyp_words[j - COMPOUND_SPLIT_WINDOW..j].concat();
                if merged == ref_words[i - 1] {
                    dp[i][j] = dp[i][j].min(dp[i - 1][j - COMPOUND_SPLIT_WINDOW]);
                }
            }
        }
    }

    let edits = dp[n][m];
    Ok(edits as f32 / n as f32)
}

/// Computes the Character Error Rate (CER) between reference and hypothesis text.
pub fn character_error_rate(reference: &str, hypothesis: &str) -> Result<f32, AcoustixError> {
    let ref_chars: Vec<char> = reference
        .chars()
        .filter(|c| !c.is_ascii_punctuation())
        .collect();
    let hyp_chars: Vec<char> = hypothesis
        .chars()
        .filter(|c| !c.is_ascii_punctuation())
        .collect();

    if ref_chars.is_empty() {
        return Err(AcoustixError::InvalidParameter(
            "Reference text must contain at least one character".to_string(),
        ));
    }

    let n = ref_chars.len();
    let m = hyp_chars.len();

    let mut dp = vec![vec![0; m + 1]; n + 1];

    for i in 0..=n {
        dp[i][0] = i;
    }
    for j in 0..=m {
        dp[0][j] = j;
    }

    for i in 1..=n {
        for j in 1..=m {
            let cost = if ref_chars[i - 1].to_lowercase().to_string()
                == hyp_chars[j - 1].to_lowercase().to_string()
            {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    let edits = dp[n][m];
    Ok(edits as f32 / n as f32)
}

/// Computes the Speaker Similarity (SIM) as the cosine similarity between two speaker embedding vectors.
///
/// Returns a score in the range `[-1.0, 1.0]`, where `1.0` indicates identical vectors.
pub fn cosine_similarity(vec_a: &[f32], vec_b: &[f32]) -> Result<f32, AcoustixError> {
    if vec_a.is_empty() || vec_b.is_empty() {
        return Err(AcoustixError::EmptySignal(
            "Embedding vectors cannot be empty".to_string(),
        ));
    }
    if vec_a.len() != vec_b.len() {
        return Err(AcoustixError::InvalidParameter(
            "Embedding vector lengths must match".to_string(),
        ));
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (&x, &y) in vec_a.iter().zip(vec_b.iter()) {
        dot_product += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a <= 1e-10 || norm_b <= 1e-10 {
        return Err(AcoustixError::InvalidParameter(
            "Embedding vector magnitude is zero".to_string(),
        ));
    }

    Ok(dot_product / (norm_a.sqrt() * norm_b.sqrt()))
}

/// Computes Speaker Attribution Accuracy (ACC) between actual speaker sequence and predicted sequence.
///
/// Returns a score in the range `[0.0, 1.0]`.
pub fn speaker_attribution_accuracy(
    actual: &[String],
    predicted: &[String],
) -> Result<f32, AcoustixError> {
    if actual.is_empty() || predicted.is_empty() {
        return Err(AcoustixError::EmptySignal(
            "Speaker label lists cannot be empty".to_string(),
        ));
    }
    if actual.len() != predicted.len() {
        return Err(AcoustixError::InvalidParameter(
            "Actual and predicted speaker label lists must have the same length".to_string(),
        ));
    }

    let correct = actual
        .iter()
        .zip(predicted.iter())
        .filter(|(a, p)| a == p)
        .count();
    Ok(correct as f32 / actual.len() as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wer() {
        let reference = "The quick brown fox jumps over the lazy dog";
        let hypothesis = "The quick brown fox jump over lazy dog";

        let wer = word_error_rate(reference, hypothesis).unwrap();
        // Deletion: "the" (1), Substitution: "jumps" -> "jump" (1). Total edits = 2.
        // Reference words = 9. WER = 2 / 9 = 0.2222
        assert!((wer - 0.222222).abs() < 1e-4);
    }

    #[test]
    fn word_error_rate_ignores_unicode_punctuation_differences() {
        // Curly quotes/apostrophe (typical whisper.cpp ASR output) vs straight
        // quotes (typical fixture source text) must not count as WER errors --
        // is_ascii_punctuation alone never matches U+2019/U+201C/U+201D.
        let reference = "the dog's toy is \"red\"";
        let hypothesis = "the dog\u{2019}s toy is \u{201C}red\u{201D}"; // curly ' and " "
        let wer = word_error_rate(reference, hypothesis).expect("non-empty reference");
        assert_eq!(
            wer, 0.0,
            "curly vs straight quotes should not produce WER errors, got {wer}"
        );
    }

    #[test]
    fn word_error_rate_treats_hyphens_as_word_separators() {
        // Real-world case: ASR never reconstructs hyphens, so a hyphenated compound in written source text
        // ("AI-narrated") is always transcribed as separate
        // space-separated words ("AI narrated"). Naively stripping the
        // hyphen as plain punctuation instead glues it into one token
        // ("ainarrated"), which can never match the natural two-word
        // transcription -- inflating WER on any hyphenated compound.
        let reference = "Transform content into curated, AI-narrated audio scrolls.";
        let hypothesis = "Transform content into curated AI narrated audio scrolls.";
        let wer = word_error_rate(reference, hypothesis).expect("non-empty reference");
        assert_eq!(
            wer, 0.0,
            "hyphenated compound should match its natural two-word ASR transcription, got WER {wer}"
        );
    }

    #[test]
    fn word_error_rate_treats_asr_compound_word_splits_as_matches() {
        // Real-world cases whisper.cpp never reconstructs a compound word's internal boundary,
        // even with no hyphen or other written signal to normalize -- "offloading" comes
        // back as "off loading", "performative" as "perform ative", and the
        // CamelCase brand name "QuietScroll" as "Quiet Scroll". None of these
        // are content errors; the transcribed words concatenate exactly back
        // to the reference word.
        let reference = "Cognitive offloading is a performative act, says QuietScroll.";
        let hypothesis = "Cognitive off loading is a perform ative act says Quiet Scroll";
        let wer = word_error_rate(reference, hypothesis).expect("non-empty reference");
        assert_eq!(
            wer, 0.0,
            "compound-word ASR splits should not count as WER errors, got {wer}"
        );
    }

    #[test]
    fn word_error_rate_still_counts_genuine_two_word_substitutions() {
        // Guard against the compound-split tolerance masking a real error:
        // two hypothesis words that happen to concatenate to something other
        // than the reference word must still cost edits normally.
        let reference = "the cat sat down";
        let hypothesis = "the dog ran down";
        let wer = word_error_rate(reference, hypothesis).expect("non-empty reference");
        // "cat"->"dog" and "sat"->"ran": 2 substitutions / 4 reference words.
        assert!(
            (wer - 0.5).abs() < 1e-4,
            "unrelated word substitutions must still count as errors, got {wer}"
        );
    }

    #[test]
    fn word_error_rate_treats_british_ize_spelling_as_a_match() {
        // Real-world case source text
        // uses British spelling ("personalised"), whisper.cpp always
        // transcribes American spelling ("personalized") regardless of what
        // was actually spoken -- a closed, standard spelling variance, not
        // a content error.
        let reference = "Transform content into personalised audio scrolls.";
        let hypothesis = "Transform content into personalized audio scrolls";
        let wer = word_error_rate(reference, hypothesis).expect("non-empty reference");
        assert_eq!(
            wer, 0.0,
            "British/American -ise/-ize spelling should not count as a WER error, got {wer}"
        );
    }

    #[test]
    fn test_cer() {
        let reference = "cat";
        let hypothesis = "cot";
        let cer = character_error_rate(reference, hypothesis).unwrap();
        assert!((cer - 0.333333).abs() < 1e-4);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![2.0, 4.0, 6.0]; // Colinear vector
        let sim = cosine_similarity(&a, &b).unwrap();
        assert!((sim - 1.0).abs() < 1e-5);

        let c = vec![-1.0, -2.0, -3.0]; // Opposite vector
        let sim = cosine_similarity(&a, &c).unwrap();
        assert!((sim - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_acc() {
        let actual = vec!["spk1".to_string(), "spk1".to_string(), "spk2".to_string()];
        let predicted = vec!["spk1".to_string(), "spk2".to_string(), "spk2".to_string()];
        let acc = speaker_attribution_accuracy(&actual, &predicted).unwrap();
        assert!((acc - 0.666667).abs() < 1e-4);
    }
}
