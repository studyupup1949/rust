use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Could not parse natural language input: {0}")]
    ParseError(String),
    #[error("Unsupported query type")]
    UnsupportedQuery,
}

pub type Result<T> = std::result::Result<T, AiError>;

/// What the user intended from natural language.
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
    pub fn new() -> Self {
        Self
    }

    /// Parse a natural language input into a structured query.
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

    pub fn entries(&self) -> &VecDeque<HistoryEntry> {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for CalculationHistory {
    fn default() -> Self {
        Self::new(100)
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
}
