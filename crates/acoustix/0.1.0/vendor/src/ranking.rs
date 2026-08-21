use crate::error::AcoustixError;
use crate::quality::{log_spectral_distance, segmental_snr};
use crate::similarity::compute_mcd_between_signals;

/// The type of objective speech metric to use for evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Mel Cepstral Distortion (MCD) (lower is better).
    Mcd,
    /// Log Spectral Distance (LSD) (lower is better).
    Lsd,
    /// Segmental Signal-to-Noise Ratio (SegSNR) (higher is better).
    SegSnr,
}

/// The result of an automated pairwise preference comparison.
#[derive(Debug, Clone)]
pub struct PreferenceResult {
    /// The score achieved by Model A.
    pub score_a: f32,
    /// The score achieved by Model B.
    pub score_b: f32,
    /// The label of the preferred model ("A", "B", or "Tie").
    pub preferred_model: String,
}

/// Compares two model signals against a reference signal using a specific metric.
/// Declares which one is preferred based on metric minimization (MCD, LSD) or maximization (SegSNR).
pub fn compare_preference(
    ref_sig: &[f32],
    test_a: &[f32],
    test_b: &[f32],
    metric: MetricType,
    sample_rate: u32,
    frame_len: usize,
    overlap: usize,
) -> Result<PreferenceResult, AcoustixError> {
    let score_a = match metric {
        MetricType::Mcd => compute_mcd_between_signals(
            ref_sig,
            test_a,
            sample_rate,
            frame_len,
            overlap,
            20,
            13,
            true,
        )?,
        MetricType::Lsd => log_spectral_distance(ref_sig, test_a, frame_len, overlap)?,
        MetricType::SegSnr => {
            segmental_snr(ref_sig, test_a, frame_len, overlap, -10.0, 35.0, 1e-4)?
        }
    };

    let score_b = match metric {
        MetricType::Mcd => compute_mcd_between_signals(
            ref_sig,
            test_b,
            sample_rate,
            frame_len,
            overlap,
            20,
            13,
            true,
        )?,
        MetricType::Lsd => log_spectral_distance(ref_sig, test_b, frame_len, overlap)?,
        MetricType::SegSnr => {
            segmental_snr(ref_sig, test_b, frame_len, overlap, -10.0, 35.0, 1e-4)?
        }
    };

    let preferred = match metric {
        MetricType::Mcd | MetricType::Lsd => {
            if score_a < score_b {
                "A".to_string()
            } else if score_b < score_a {
                "B".to_string()
            } else {
                "Tie".to_string()
            }
        }
        MetricType::SegSnr => {
            if score_a > score_b {
                "A".to_string()
            } else if score_b > score_a {
                "B".to_string()
            } else {
                "Tie".to_string()
            }
        }
    };

    Ok(PreferenceResult {
        score_a,
        score_b,
        preferred_model: preferred,
    })
}

/// The raw objective scores computed for a given model.
#[derive(Debug, Clone)]
pub struct ModelEvaluation {
    /// The name/identifier of the model.
    pub model_name: String,
    /// Mel Cepstral Distortion (MCD) score.
    pub mcd: f32,
    /// Log Spectral Distance (LSD) score.
    pub lsd: f32,
    /// Segmental SNR (SegSNR) score.
    pub seg_snr: f32,
}

/// A model rank assignment with aggregated composite score.
#[derive(Debug, Clone)]
pub struct RankedModel {
    /// The name/identifier of the ranked model.
    pub model_name: String,
    /// The computed rank (1 is best, higher is worse).
    pub rank: usize,
    /// The composite weighted score (lower is better, range [0, total_weight]).
    pub aggregated_score: f32,
}

/// Aggregates multiple evaluation metrics across candidate models and ranks them.
/// Normalizes metrics to a 0.0 (best) to 1.0 (worst) scale, and computes a weighted sum.
pub fn rank_models(
    evaluations: &[ModelEvaluation],
    w_mcd: f32,
    w_lsd: f32,
    w_snr: f32,
) -> Vec<RankedModel> {
    if evaluations.is_empty() {
        return Vec::new();
    }

    // Find min and max for normalization
    let mut min_mcd = f32::INFINITY;
    let mut max_mcd = f32::NEG_INFINITY;
    let mut min_lsd = f32::INFINITY;
    let mut max_lsd = f32::NEG_INFINITY;
    let mut min_snr = f32::INFINITY;
    let mut max_snr = f32::NEG_INFINITY;

    for eval in evaluations {
        if eval.mcd < min_mcd {
            min_mcd = eval.mcd;
        }
        if eval.mcd > max_mcd {
            max_mcd = eval.mcd;
        }
        if eval.lsd < min_lsd {
            min_lsd = eval.lsd;
        }
        if eval.lsd > max_lsd {
            max_lsd = eval.lsd;
        }
        if eval.seg_snr < min_snr {
            min_snr = eval.seg_snr;
        }
        if eval.seg_snr > max_snr {
            max_snr = eval.seg_snr;
        }
    }

    let diff_mcd = max_mcd - min_mcd;
    let diff_lsd = max_lsd - min_lsd;
    let diff_snr = max_snr - min_snr;

    let mut scored_models: Vec<(String, f32)> = evaluations
        .iter()
        .map(|eval| {
            let norm_mcd = if diff_mcd > 1e-6 {
                (eval.mcd - min_mcd) / diff_mcd
            } else {
                0.0
            };
            let norm_lsd = if diff_lsd > 1e-6 {
                (eval.lsd - min_lsd) / diff_lsd
            } else {
                0.0
            };
            // For SNR, higher is better, so norm = (max - val) / diff makes higher SNR map to 0.0 (best)
            let norm_snr = if diff_snr > 1e-6 {
                (max_snr - eval.seg_snr) / diff_snr
            } else {
                0.0
            };

            let score = w_mcd * norm_mcd + w_lsd * norm_lsd + w_snr * norm_snr;
            (eval.model_name.clone(), score)
        })
        .collect();

    // Sort by composite score (ascending: lower score is better)
    scored_models.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranked = Vec::with_capacity(scored_models.len());
    for (i, (name, score)) in scored_models.into_iter().enumerate() {
        ranked.push(RankedModel {
            model_name: name,
            rank: i + 1,
            aggregated_score: score,
        });
    }

    ranked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_preference_mcd() {
        let ref_sig = vec![0.1; 1000];
        let test_a = vec![0.1; 1000]; // MCD = 0 (perfect)

        let mut test_b = vec![0.1; 1000];
        for i in (0..1000).step_by(2) {
            test_b[i] = -0.1; // MCD > 0
        }

        let res = compare_preference(&ref_sig, &test_a, &test_b, MetricType::Mcd, 16000, 256, 128)
            .unwrap();
        assert_eq!(res.preferred_model, "A");
        assert!(res.score_a < res.score_b);
    }

    #[test]
    fn test_compare_preference_snr() {
        let ref_sig = vec![0.5; 4];
        let test_a = vec![0.49; 4]; // SNR is very high
        let test_b = vec![0.4; 4]; // SNR is lower

        let res = compare_preference(&ref_sig, &test_a, &test_b, MetricType::SegSnr, 16000, 4, 0)
            .unwrap();
        assert_eq!(res.preferred_model, "A");
        assert!(res.score_a > res.score_b);
    }

    #[test]
    fn test_rank_models() {
        let evals = vec![
            ModelEvaluation {
                model_name: "Model_A".to_string(),
                mcd: 1.0,      // Best
                lsd: 1.5,      // Mid
                seg_snr: 20.0, // Best
            },
            ModelEvaluation {
                model_name: "Model_B".to_string(),
                mcd: 3.0,     // Worst
                lsd: 4.0,     // Worst
                seg_snr: 5.0, // Worst
            },
            ModelEvaluation {
                model_name: "Model_C".to_string(),
                mcd: 2.0,      // Mid
                lsd: 1.0,      // Best
                seg_snr: 15.0, // Mid
            },
        ];

        let ranked = rank_models(&evals, 1.0, 1.0, 1.0);
        assert_eq!(ranked.len(), 3);

        // Model_A or Model_C should be rank 1 (since Model_B is worst in all dimensions)
        assert_eq!(ranked[2].model_name, "Model_B");
        assert_eq!(ranked[2].rank, 3);

        assert_eq!(ranked[0].rank, 1);
    }
}
