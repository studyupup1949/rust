use crate::error::AcoustixError;

/// Tokenizes text by splitting on whitespace, converting to lowercase, and removing punctuation.
fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| !c.is_ascii_punctuation())
                .collect::<String>()
                .to_lowercase()
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
