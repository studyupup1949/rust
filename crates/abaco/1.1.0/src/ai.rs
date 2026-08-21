//! Natural language math parsing — "what is 15% of 230", "convert 5 km to miles".
//!
//! Feature-gated behind `ai`. Provides [`NlParser`] for parsing natural language
//! into structured [`ParsedQuery`] types, [`CalculationHistory`] for tracking
//! past calculations, and [`CurrencyConverter`] for live exchange rates.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use tracing::{debug, instrument, warn};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Could not parse natural language input: {0}")]
    ParseError(String),
    #[error("Unsupported query type")]
    UnsupportedQuery,
    #[error("Currency error: {0}")]
    CurrencyError(String),
    #[error("HTTP error: {0}")]
    HttpError(String),
}

pub type Result<T> = std::result::Result<T, AiError>;

/// What the user intended from natural language.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParsedQuery {
    /// A math expression to evaluate (e.g., "what is 15% of 230").
    Calculation(String),
    /// A unit conversion (e.g., "convert 5 km to miles").
    Conversion {
        value: f64,
        from: String,
        to: String,
    },
    /// A currency conversion (e.g., "100 usd in eur").
    CurrencyConversion {
        value: f64,
        from: String,
        to: String,
    },
}

/// Natural language parser for math and conversion queries.
pub struct NlParser;

impl NlParser {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Parse a natural language input into a structured query.
    #[must_use = "parsing has no side effects"]
    pub fn parse_natural(&self, input: &str) -> Result<ParsedQuery> {
        let input = input.trim().to_lowercase();

        // Try currency conversion: "100 usd in eur", "100 usd to eur"
        if let Some(q) = self.try_parse_currency(&input) {
            return Ok(q);
        }

        // Try unit conversion: "convert 5 km to miles", "5 km to miles", "5 km in miles"
        if let Some(q) = self.try_parse_conversion(&input) {
            return Ok(q);
        }

        // Try calculation: "what is 15% of 230", "calculate 2 + 3"
        if let Some(q) = self.try_parse_calculation(&input) {
            return Ok(q);
        }

        // If it looks like a math expression, treat it as calculation
        if input
            .chars()
            .any(|c| c.is_ascii_digit() || "+-*/^%().".contains(c))
        {
            return Ok(ParsedQuery::Calculation(input));
        }

        Err(AiError::ParseError(input))
    }

    fn try_parse_currency(&self, input: &str) -> Option<ParsedQuery> {
        let currencies = [
            "usd", "eur", "gbp", "jpy", "cad", "aud", "chf", "cny", "inr", "krw", "brl", "mxn",
            "sek", "nok", "dkk", "nzd", "sgd", "hkd", "try", "rub", "zar", "pln", "thb", "twd",
        ];

        // Pattern: "<number> <currency> to/in <currency>"
        let words: Vec<&str> = input.split_whitespace().collect();
        if words.len() >= 4
            && let Ok(value) = words[0].parse::<f64>()
        {
            let from = words[1];
            let connector = words[2];
            let to = words[3];
            if (connector == "to" || connector == "in")
                && currencies.contains(&from)
                && currencies.contains(&to)
            {
                return Some(ParsedQuery::CurrencyConversion {
                    value,
                    from: from.to_uppercase(),
                    to: to.to_uppercase(),
                });
            }
        }

        None
    }

    fn try_parse_conversion(&self, input: &str) -> Option<ParsedQuery> {
        let input = input.strip_prefix("convert ").unwrap_or(input);

        // Pattern: "<number> <unit> to/in <unit>"
        let words: Vec<&str> = input.split_whitespace().collect();
        if words.len() >= 4
            && let Ok(value) = words[0].parse::<f64>()
        {
            // Find "to" or "in" separator
            for i in 2..words.len() {
                if words[i] == "to" || words[i] == "in" {
                    let from = words[1..i].join(" ");
                    let to = words[i + 1..].join(" ");
                    if !from.is_empty() && !to.is_empty() {
                        return Some(ParsedQuery::Conversion { value, from, to });
                    }
                }
            }
        }

        None
    }

    fn try_parse_calculation(&self, input: &str) -> Option<ParsedQuery> {
        // "what is X", "what's X", "calculate X", "compute X", "eval X"
        let prefixes = [
            "what is ",
            "what's ",
            "calculate ",
            "compute ",
            "eval ",
            "how much is ",
        ];

        for prefix in &prefixes {
            if let Some(rest) = input.strip_prefix(prefix) {
                // Handle "X% of Y" -> "Y * X / 100"
                if let Some(pct_expr) = self.try_parse_percent_of(rest) {
                    return Some(ParsedQuery::Calculation(pct_expr));
                }
                return Some(ParsedQuery::Calculation(rest.to_string()));
            }
        }

        // Handle standalone "X% of Y"
        if let Some(pct_expr) = self.try_parse_percent_of(input) {
            return Some(ParsedQuery::Calculation(pct_expr));
        }

        None
    }

    fn try_parse_percent_of(&self, input: &str) -> Option<String> {
        // "15% of 230" -> "230 * 15 / 100"
        let parts: Vec<&str> = input.splitn(2, "% of ").collect();
        if parts.len() == 2 {
            let pct = parts[0].trim();
            let base = parts[1].trim();
            if pct.parse::<f64>().is_ok() {
                return Some(format!("{base} * {pct} / 100"));
            }
        }
        None
    }
}

impl Default for NlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// History of calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationHistory {
    entries: VecDeque<HistoryEntry>,
    max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub input: String,
    pub result: String,
    pub timestamp: String,
}

impl CalculationHistory {
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
        }
    }

    pub fn push(&mut self, input: &str, result: &str) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(HistoryEntry {
            input: input.to_string(),
            result: result.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        });
    }

    #[must_use]
    pub fn entries(&self) -> &VecDeque<HistoryEntry> {
        &self.entries
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Serialize history to a JSON string.
    pub fn to_json(&self) -> std::result::Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Deserialize history from a JSON string.
    pub fn from_json(json: &str) -> std::result::Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    /// Save history to a JSON file.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::result::Result<(), String> {
        let json = self.to_json()?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    /// Load history from a JSON file.
    pub fn load_from_file(path: &std::path::Path) -> std::result::Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_json(&json)
    }
}

impl Default for CalculationHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

// ── Currency conversion ─────────────────────────────────────────────────────

/// Cached exchange rates with a timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RateCache {
    /// Base currency code (rates are relative to this).
    base: String,
    /// Currency code → rate (1 base = rate target).
    rates: HashMap<String, f64>,
    /// When these rates were fetched (RFC 3339).
    fetched_at: String,
}

/// Live currency converter with rate caching.
///
/// Fetches exchange rates from a hoosh service endpoint and caches them
/// in memory. Cache TTL is configurable (default: 1 hour).
pub struct CurrencyConverter {
    /// Base URL of the rate service (e.g., `http://localhost:8088`).
    base_url: String,
    /// Cache TTL in seconds.
    cache_ttl_secs: i64,
    /// Cached rates (behind a mutex for interior mutability).
    cache: Mutex<Option<RateCache>>,
    /// HTTP client.
    client: reqwest::Client,
}

/// Result of a currency conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyResult {
    pub from_value: f64,
    pub from_currency: String,
    pub to_value: f64,
    pub to_currency: String,
    pub rate: f64,
}

impl std::fmt::Display for CurrencyResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} = {} {} (rate: {})",
            self.from_value, self.from_currency, self.to_value, self.to_currency, self.rate
        )
    }
}

/// Response shape expected from the hoosh rate service.
#[derive(Debug, Deserialize)]
struct RateResponse {
    base: String,
    rates: HashMap<String, f64>,
}

impl CurrencyConverter {
    /// Create a new converter pointing at the given hoosh base URL.
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            cache_ttl_secs: 3600,
            cache: Mutex::new(None),
            client: reqwest::Client::new(),
        }
    }

    /// Create a converter with a custom cache TTL (in seconds).
    #[must_use]
    pub fn with_ttl(mut self, ttl_secs: i64) -> Self {
        self.cache_ttl_secs = ttl_secs;
        self
    }

    /// Check if the cached rates are still valid.
    fn cache_valid(&self) -> bool {
        let guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref cache) = *guard
            && let Ok(fetched) = chrono::DateTime::parse_from_rfc3339(&cache.fetched_at)
        {
            let age = Utc::now().signed_duration_since(fetched);
            return age.num_seconds() < self.cache_ttl_secs;
        }
        false
    }

    /// Fetch fresh rates from the hoosh service.
    #[instrument(skip(self))]
    async fn fetch_rates(&self) -> Result<RateCache> {
        let url = format!("{}/rates", self.base_url);
        debug!(url, "fetching exchange rates");

        let resp = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AiError::HttpError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            warn!(%status, "rate service returned error");
            return Err(AiError::HttpError(format!("HTTP {status}")));
        }

        let body: RateResponse = resp
            .json()
            .await
            .map_err(|e| AiError::CurrencyError(format!("invalid rate response: {e}")))?;

        let cache = RateCache {
            base: body.base,
            rates: body.rates,
            fetched_at: Utc::now().to_rfc3339(),
        };

        // Update the cache
        let mut guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(cache.clone());

        debug!(base = cache.base, count = cache.rates.len(), "rates cached");
        Ok(cache)
    }

    /// Get rates, using cache if valid or fetching fresh ones.
    async fn get_rates(&self) -> Result<RateCache> {
        if self.cache_valid() {
            let guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(ref cache) = *guard {
                return Ok(cache.clone());
            }
        }
        // Try to fetch fresh rates
        match self.fetch_rates().await {
            Ok(cache) => Ok(cache),
            Err(e) => {
                // Offline fallback: use stale cache if available
                let guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(ref cache) = *guard {
                    warn!("using stale cached rates (fetch failed: {e})");
                    return Ok(cache.clone());
                }
                Err(e)
            }
        }
    }

    /// Convert between currencies.
    #[instrument(skip(self), fields(from, to))]
    pub async fn convert(&self, value: f64, from: &str, to: &str) -> Result<CurrencyResult> {
        let from = from.to_uppercase();
        let to = to.to_uppercase();

        let cache = self.get_rates().await?;

        // Get rates relative to the base currency
        let from_rate = if from == cache.base {
            1.0
        } else {
            *cache
                .rates
                .get(&from)
                .ok_or_else(|| AiError::CurrencyError(format!("unknown currency: {from}")))?
        };

        let to_rate = if to == cache.base {
            1.0
        } else {
            *cache
                .rates
                .get(&to)
                .ok_or_else(|| AiError::CurrencyError(format!("unknown currency: {to}")))?
        };

        // Convert: from → base → to
        let rate = to_rate / from_rate;
        let result = value * rate;

        debug!(value, %from, %to, rate, result, "currency conversion");

        Ok(CurrencyResult {
            from_value: value,
            from_currency: from,
            to_value: result,
            to_currency: to,
            rate,
        })
    }

    /// Manually set cached rates (useful for testing or offline mode).
    pub fn set_rates(&self, base: &str, rates: HashMap<String, f64>) {
        let cache = RateCache {
            base: base.to_uppercase(),
            rates,
            fetched_at: Utc::now().to_rfc3339(),
        };
        let mut guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(cache);
    }
}

impl Default for CurrencyConverter {
    fn default() -> Self {
        Self::new("http://localhost:8088")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> NlParser {
        NlParser::new()
    }

    #[test]
    fn test_parse_what_is() {
        let q = parser().parse_natural("what is 2 + 3").unwrap();
        assert_eq!(q, ParsedQuery::Calculation("2 + 3".to_string()));
    }

    #[test]
    fn test_parse_calculate() {
        let q = parser().parse_natural("calculate 10 * 5").unwrap();
        assert_eq!(q, ParsedQuery::Calculation("10 * 5".to_string()));
    }

    #[test]
    fn test_parse_percent_of() {
        let q = parser().parse_natural("what is 15% of 230").unwrap();
        match q {
            ParsedQuery::Calculation(expr) => {
                assert!(expr.contains("230"));
                assert!(expr.contains("15"));
                assert!(expr.contains("100"));
            }
            other => panic!("Expected Calculation, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_conversion() {
        let q = parser().parse_natural("convert 5 km to miles").unwrap();
        assert_eq!(
            q,
            ParsedQuery::Conversion {
                value: 5.0,
                from: "km".to_string(),
                to: "miles".to_string()
            }
        );
    }

    #[test]
    fn test_parse_conversion_in() {
        let q = parser().parse_natural("10 meters in feet").unwrap();
        assert_eq!(
            q,
            ParsedQuery::Conversion {
                value: 10.0,
                from: "meters".to_string(),
                to: "feet".to_string()
            }
        );
    }

    #[test]
    fn test_parse_currency() {
        let q = parser().parse_natural("100 usd to eur").unwrap();
        assert_eq!(
            q,
            ParsedQuery::CurrencyConversion {
                value: 100.0,
                from: "USD".to_string(),
                to: "EUR".to_string()
            }
        );
    }

    #[test]
    fn test_parse_raw_expression() {
        let q = parser().parse_natural("3.14 * 2").unwrap();
        assert_eq!(q, ParsedQuery::Calculation("3.14 * 2".to_string()));
    }

    #[test]
    fn test_history_push() {
        let mut h = CalculationHistory::new(100);
        h.push("2 + 3", "5");
        assert_eq!(h.len(), 1);
        assert_eq!(h.entries()[0].input, "2 + 3");
    }

    #[test]
    fn test_history_limit() {
        let mut h = CalculationHistory::new(3);
        h.push("1", "1");
        h.push("2", "2");
        h.push("3", "3");
        h.push("4", "4");
        assert_eq!(h.len(), 3);
        assert_eq!(h.entries()[0].input, "2");
    }

    #[test]
    fn test_history_clear() {
        let mut h = CalculationHistory::new(100);
        h.push("1", "1");
        h.push("2", "2");
        h.clear();
        assert!(h.is_empty());
    }

    #[test]
    fn test_parse_how_much_is() {
        let q = parser().parse_natural("how much is 5 + 3").unwrap();
        assert_eq!(q, ParsedQuery::Calculation("5 + 3".to_string()));
    }

    #[test]
    fn test_parse_compute() {
        let q = parser().parse_natural("compute sqrt(16)").unwrap();
        assert_eq!(q, ParsedQuery::Calculation("sqrt(16)".to_string()));
    }

    #[test]
    fn test_parse_eval_prefix() {
        let q = parser().parse_natural("eval 2^10").unwrap();
        assert_eq!(q, ParsedQuery::Calculation("2^10".to_string()));
    }

    #[test]
    fn test_standalone_percent_of() {
        let q = parser().parse_natural("15% of 200").unwrap();
        match q {
            ParsedQuery::Calculation(expr) => {
                assert!(expr.contains("200"));
                assert!(expr.contains("15"));
            }
            other => panic!("Expected Calculation, got {other:?}"),
        }
    }

    #[test]
    fn test_currency_with_in() {
        let q = parser().parse_natural("50 gbp in jpy").unwrap();
        assert_eq!(
            q,
            ParsedQuery::CurrencyConversion {
                value: 50.0,
                from: "GBP".to_string(),
                to: "JPY".to_string()
            }
        );
    }

    #[test]
    fn test_conversion_without_prefix() {
        let q = parser().parse_natural("100 fahrenheit to celsius").unwrap();
        assert_eq!(
            q,
            ParsedQuery::Conversion {
                value: 100.0,
                from: "fahrenheit".to_string(),
                to: "celsius".to_string()
            }
        );
    }

    #[test]
    fn test_unparseable_input() {
        assert!(parser().parse_natural("hello world").is_err());
    }

    #[test]
    fn test_number_only_input() {
        let q = parser().parse_natural("42").unwrap();
        assert_eq!(q, ParsedQuery::Calculation("42".to_string()));
    }

    #[test]
    fn test_history_default() {
        let h = CalculationHistory::default();
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn test_nl_parser_default() {
        let p: NlParser = Default::default();
        let q = p.parse_natural("what is 1 + 1").unwrap();
        assert_eq!(q, ParsedQuery::Calculation("1 + 1".to_string()));
    }

    #[test]
    fn test_history_entries_accessor() {
        let mut h = CalculationHistory::new(10);
        h.push("a", "b");
        h.push("c", "d");
        assert_eq!(h.entries().len(), 2);
        assert_eq!(h.entries()[0].input, "a");
        assert_eq!(h.entries()[0].result, "b");
        assert_eq!(h.entries()[1].input, "c");
    }

    // --- Currency converter ---

    fn test_converter() -> CurrencyConverter {
        let c = CurrencyConverter::new("http://localhost:8088");
        let mut rates = HashMap::new();
        rates.insert("EUR".to_string(), 0.92);
        rates.insert("GBP".to_string(), 0.79);
        rates.insert("JPY".to_string(), 149.50);
        rates.insert("CAD".to_string(), 1.36);
        c.set_rates("USD", rates);
        c
    }

    #[tokio::test]
    async fn test_currency_usd_to_eur() {
        let c = test_converter();
        let r = c.convert(100.0, "USD", "EUR").await.unwrap();
        assert!((r.to_value - 92.0).abs() < 0.01);
        assert!((r.rate - 0.92).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_currency_eur_to_usd() {
        let c = test_converter();
        let r = c.convert(100.0, "EUR", "USD").await.unwrap();
        // 100 EUR = 100 / 0.92 USD ≈ 108.70
        assert!((r.to_value - 108.70).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_currency_cross_rate() {
        let c = test_converter();
        // EUR → JPY: rate = 149.50 / 0.92 ≈ 162.50
        let r = c.convert(1.0, "EUR", "JPY").await.unwrap();
        assert!((r.to_value - 162.50).abs() < 0.5);
    }

    #[tokio::test]
    async fn test_currency_same() {
        let c = test_converter();
        let r = c.convert(100.0, "USD", "USD").await.unwrap();
        assert!((r.to_value - 100.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_currency_unknown_errors() {
        let c = test_converter();
        let r = c.convert(100.0, "USD", "XYZ").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn test_currency_case_insensitive() {
        let c = test_converter();
        let r = c.convert(100.0, "usd", "eur").await.unwrap();
        assert!((r.to_value - 92.0).abs() < 0.01);
    }

    #[test]
    fn test_currency_display() {
        let r = CurrencyResult {
            from_value: 100.0,
            from_currency: "USD".to_string(),
            to_value: 92.0,
            to_currency: "EUR".to_string(),
            rate: 0.92,
        };
        let s = r.to_string();
        assert!(s.contains("100"));
        assert!(s.contains("USD"));
        assert!(s.contains("EUR"));
    }

    #[test]
    fn test_currency_converter_default() {
        let c = CurrencyConverter::default();
        assert!(!c.cache_valid());
    }

    #[test]
    fn test_currency_cache_valid_after_set() {
        let c = test_converter();
        assert!(c.cache_valid());
    }

    #[test]
    fn test_currency_with_ttl() {
        let c = CurrencyConverter::new("http://localhost:8088").with_ttl(60);
        assert_eq!(c.cache_ttl_secs, 60);
    }

    // --- History persistence ---

    #[test]
    fn test_history_json_roundtrip() {
        let mut h = CalculationHistory::new(100);
        h.push("2 + 3", "5");
        h.push("sqrt(16)", "4");
        let json = h.to_json().unwrap();
        let h2 = CalculationHistory::from_json(&json).unwrap();
        assert_eq!(h2.len(), 2);
        assert_eq!(h2.entries()[0].input, "2 + 3");
        assert_eq!(h2.entries()[1].result, "4");
    }

    #[test]
    fn test_history_file_roundtrip() {
        let mut h = CalculationHistory::new(100);
        h.push("1 + 1", "2");
        let dir = std::env::temp_dir();
        let path = dir.join("abaco_test_history.json");
        h.save_to_file(&path).unwrap();
        let h2 = CalculationHistory::load_from_file(&path).unwrap();
        assert_eq!(h2.len(), 1);
        assert_eq!(h2.entries()[0].input, "1 + 1");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_history_empty_json() {
        let h = CalculationHistory::new(50);
        let json = h.to_json().unwrap();
        let h2 = CalculationHistory::from_json(&json).unwrap();
        assert!(h2.is_empty());
    }
}
