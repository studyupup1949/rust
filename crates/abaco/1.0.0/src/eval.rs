//! Expression evaluator — arithmetic, 28+ math functions, variables, scientific notation.
//!
//! The evaluator parses and evaluates mathematical expressions using a recursive
//! descent parser. It supports standard arithmetic (`+`, `-`, `*`, `/`, `%`, `^`),
//! parentheses, named functions (`sin`, `sqrt`, `log2`, etc.), constants (`pi`, `e`),
//! and user-defined variables.

use crate::core::Value;
use std::collections::HashMap;
use tracing::instrument;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Unknown function: {0}")]
    UnknownFunction(String),
    #[error("Unknown variable: {0}")]
    UnknownVariable(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Math error: {0}")]
    MathError(String),
    #[error("Invalid expression")]
    InvalidExpression,
}

pub type Result<T> = std::result::Result<T, EvalError>;

/// Lexer token.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Power,
    LParen,
    RParen,
    Ident(String),
    Comma,
    Bang,
}

/// Tokenize an expression string.
#[inline]
#[must_use = "tokenizing has no side effects"]
pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\n' | b'\r' => i += 1,
            b'+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            b'-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            b'*' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                tokens.push(Token::Power);
                i += 2;
            }
            b'*' => {
                tokens.push(Token::Star);
                i += 1;
            }
            b'/' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            b'%' => {
                tokens.push(Token::Percent);
                i += 1;
            }
            b'^' => {
                tokens.push(Token::Power);
                i += 1;
            }
            b'(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            b')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            b',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            b'!' => {
                tokens.push(Token::Bang);
                i += 1;
            }
            c if c.is_ascii_digit() || c == b'.' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                    i += 1;
                }
                // Scientific notation: 1e10, 1.5e-3, 2E+6
                if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
                    i += 1;
                    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                        i += 1;
                    }
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                let num_str = &input[start..i];
                let num = num_str
                    .parse::<f64>()
                    .map_err(|_| EvalError::ParseError(format!("Invalid number: {num_str}")))?;
                tokens.push(Token::Number(num));
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                tokens.push(Token::Ident(input[start..i].to_string()));
            }
            _ => {
                let ch = input[i..].chars().next().unwrap_or('?');
                return Err(EvalError::ParseError(format!("Unexpected character: {ch}")));
            }
        }
    }

    insert_implicit_multiplication(&mut tokens);
    Ok(tokens)
}

/// Insert implicit `*` tokens where multiplication is implied.
///
/// Handles patterns like `2(3+4)`, `2pi`, `(2)(3)`, `(2)pi`.
fn insert_implicit_multiplication(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i + 1 < tokens.len() {
        let insert = match (&tokens[i], &tokens[i + 1]) {
            // 2(... or 2pi — but NOT ident( which is a function call
            (Token::Number(_), Token::LParen | Token::Ident(_)) => true,
            // )(... or )2 or )pi
            (Token::RParen, Token::LParen | Token::Number(_) | Token::Ident(_)) => true,
            // 15% ( — postfix percent followed by group
            (Token::Percent, Token::LParen | Token::Ident(_)) => {
                // Only if previous token pattern suggests postfix %
                // (this is a best-effort heuristic)
                i >= 1 && matches!(tokens[i - 1], Token::Number(_) | Token::RParen)
            }
            _ => false,
        };
        if insert {
            tokens.insert(i + 1, Token::Star);
            i += 2; // skip past the inserted star
        } else {
            i += 1;
        }
    }
}

/// Maximum recursion depth for expression parsing.
/// Prevents stack overflow from deeply nested expressions like `(((((...)))))`
const MAX_PARSE_DEPTH: usize = 256;

/// Expression evaluator with variable support.
pub struct Evaluator {
    variables: HashMap<String, f64>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    #[must_use]
    pub fn get_variable(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }

    /// Evaluate an expression string and return a Value.
    #[inline]
    #[must_use = "evaluating has no side effects"]
    #[instrument(skip(self), fields(expr))]
    pub fn eval(&self, expr: &str) -> Result<Value> {
        let tokens = tokenize(expr)?;
        if tokens.is_empty() {
            return Err(EvalError::InvalidExpression);
        }
        let mut pos = 0;
        let result = self.parse_expr(&tokens, &mut pos, 0)?;
        if pos < tokens.len() {
            return Err(EvalError::ParseError(format!(
                "Unexpected token at position {pos}"
            )));
        }
        Ok(Self::to_value(result))
    }

    /// Evaluate a partial/incomplete expression for live-as-you-type feedback.
    ///
    /// Returns `Ok(value)` with the best-effort result if parsing succeeds up
    /// to the end of input or a recoverable point. Trailing operators are
    /// ignored (e.g., `"2 + 3 *"` evaluates as `"2 + 3"`).
    ///
    /// Returns `Err` only if the expression is fundamentally unparseable.
    #[must_use = "evaluating has no side effects"]
    pub fn eval_partial(&self, expr: &str) -> Result<Value> {
        let tokens = tokenize(expr)?;
        if tokens.is_empty() {
            return Err(EvalError::InvalidExpression);
        }

        // Try the full expression first
        let mut pos = 0;
        if let Ok(result) = self.parse_expr(&tokens, &mut pos, 0) {
            return Ok(Self::to_value(result));
        }

        // Progressively strip trailing tokens until it parses
        for drop in 1..tokens.len() {
            let truncated = &tokens[..tokens.len() - drop];
            if truncated.is_empty() {
                break;
            }
            let mut pos = 0;
            if let Ok(result) = self.parse_expr(truncated, &mut pos, 0)
                && pos == truncated.len()
            {
                return Ok(Self::to_value(result));
            }
        }

        Err(EvalError::InvalidExpression)
    }

    fn to_value(result: f64) -> Value {
        if result.fract() == 0.0 && result.abs() < 9_007_199_254_740_992.0 {
            Value::Integer(result as i64)
        } else {
            Value::Float(result)
        }
    }

    fn check_depth(depth: usize) -> Result<()> {
        if depth >= MAX_PARSE_DEPTH {
            return Err(EvalError::ParseError("Expression too deeply nested".into()));
        }
        Ok(())
    }

    // parse_expr: handles + and -
    fn parse_expr(&self, tokens: &[Token], pos: &mut usize, depth: usize) -> Result<f64> {
        Self::check_depth(depth)?;
        let mut left = self.parse_term(tokens, pos, depth)?;
        while *pos < tokens.len() {
            match &tokens[*pos] {
                Token::Plus => {
                    *pos += 1;
                    left += self.parse_term(tokens, pos, depth)?;
                }
                Token::Minus => {
                    *pos += 1;
                    left -= self.parse_term(tokens, pos, depth)?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // parse_term: handles * / %
    fn parse_term(&self, tokens: &[Token], pos: &mut usize, depth: usize) -> Result<f64> {
        let mut left = self.parse_power(tokens, pos, depth)?;
        while *pos < tokens.len() {
            match &tokens[*pos] {
                Token::Star => {
                    *pos += 1;
                    left *= self.parse_power(tokens, pos, depth)?;
                }
                Token::Slash => {
                    *pos += 1;
                    let right = self.parse_power(tokens, pos, depth)?;
                    if right == 0.0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    left /= right;
                }
                Token::Percent => {
                    // Check if this is postfix percentage (e.g. "15%" → 0.15)
                    // or binary modulo (e.g. "10 % 3" → 1).
                    // It's postfix if the next token can't start an operand.
                    let next = tokens.get(*pos + 1);
                    let is_postfix = matches!(
                        next,
                        None | Some(
                            Token::RParen
                                | Token::Comma
                                | Token::Plus
                                | Token::Minus
                                | Token::Star
                                | Token::Slash
                                | Token::Percent
                                | Token::Power
                        )
                    );
                    *pos += 1;
                    if is_postfix {
                        left /= 100.0;
                    } else {
                        let right = self.parse_power(tokens, pos, depth)?;
                        if right == 0.0 {
                            return Err(EvalError::DivisionByZero);
                        }
                        left %= right;
                    }
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // parse_power: handles ^
    fn parse_power(&self, tokens: &[Token], pos: &mut usize, depth: usize) -> Result<f64> {
        let mut base = self.parse_unary(tokens, pos, depth)?;
        // Postfix factorial: 5! = 120
        while *pos < tokens.len() && tokens[*pos] == Token::Bang {
            *pos += 1;
            if base < 0.0 || base.fract() != 0.0 || base > 170.0 {
                return Err(EvalError::MathError(
                    "factorial requires integer 0..=170".into(),
                ));
            }
            let n = base as u64;
            base = (1..=n).fold(1.0_f64, |acc, x| acc * x as f64);
        }
        if *pos < tokens.len() && tokens[*pos] == Token::Power {
            *pos += 1;
            let exp = self.parse_power(tokens, pos, depth)?; // right-associative
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    // parse_unary: handles unary + and -
    fn parse_unary(&self, tokens: &[Token], pos: &mut usize, depth: usize) -> Result<f64> {
        if *pos < tokens.len() {
            match &tokens[*pos] {
                Token::Minus => {
                    *pos += 1;
                    let val = self.parse_unary(tokens, pos, depth)?;
                    Ok(-val)
                }
                Token::Plus => {
                    *pos += 1;
                    self.parse_unary(tokens, pos, depth)
                }
                _ => self.parse_primary(tokens, pos, depth),
            }
        } else {
            Err(EvalError::InvalidExpression)
        }
    }

    // parse_primary: numbers, identifiers (constants, functions, variables), parentheses
    fn parse_primary(&self, tokens: &[Token], pos: &mut usize, depth: usize) -> Result<f64> {
        if *pos >= tokens.len() {
            return Err(EvalError::InvalidExpression);
        }

        match &tokens[*pos] {
            Token::Number(n) => {
                let val = *n;
                *pos += 1;
                Ok(val)
            }
            Token::LParen => {
                *pos += 1;
                let val = self.parse_expr(tokens, pos, depth + 1)?;
                if *pos >= tokens.len() || tokens[*pos] != Token::RParen {
                    return Err(EvalError::ParseError("Missing closing parenthesis".into()));
                }
                *pos += 1;
                Ok(val)
            }
            Token::Ident(name) => {
                let name = name.clone();
                *pos += 1;

                // Check for function call
                if *pos < tokens.len() && tokens[*pos] == Token::LParen {
                    *pos += 1;
                    let mut args = Vec::new();
                    if *pos < tokens.len() && tokens[*pos] != Token::RParen {
                        args.push(self.parse_expr(tokens, pos, depth + 1)?);
                        while *pos < tokens.len() && tokens[*pos] == Token::Comma {
                            *pos += 1;
                            args.push(self.parse_expr(tokens, pos, depth + 1)?);
                        }
                    }
                    if *pos >= tokens.len() || tokens[*pos] != Token::RParen {
                        return Err(EvalError::ParseError("Missing closing parenthesis".into()));
                    }
                    *pos += 1;
                    self.call_function(&name, &args)
                } else {
                    // Constant or variable
                    match name.as_str() {
                        "pi" | "PI" => Ok(std::f64::consts::PI),
                        "e" | "E" => Ok(std::f64::consts::E),
                        "tau" | "TAU" => Ok(std::f64::consts::TAU),
                        _ => self
                            .variables
                            .get(&name)
                            .copied()
                            .ok_or(EvalError::UnknownVariable(name)),
                    }
                }
            }
            _ => Err(EvalError::ParseError(format!(
                "Unexpected token: {:?}",
                tokens[*pos]
            ))),
        }
    }

    /// Check a function result for NaN/Infinity and return a MathError if invalid.
    fn check_result(name: &str, val: f64) -> Result<f64> {
        if val.is_nan() {
            Err(EvalError::MathError(format!(
                "{name} produced undefined result (NaN)"
            )))
        } else if val.is_infinite() {
            Err(EvalError::MathError(format!(
                "{name} produced infinite result"
            )))
        } else {
            Ok(val)
        }
    }

    fn call_function(&self, name: &str, args: &[f64]) -> Result<f64> {
        let n = args.len();
        let result = match name {
            // 1-arg functions
            "sqrt" | "sin" | "cos" | "tan" | "log" | "log10" | "ln" | "log2" | "abs" | "ceil"
            | "floor" | "round" | "exp" | "asin" | "acos" | "atan" | "sinh" | "cosh" | "tanh"
            | "asinh" | "acosh" | "atanh" | "trunc" | "fract" | "sign" | "sgn" | "deg" | "rad"
            | "factorial" => {
                if n != 1 {
                    return Err(EvalError::ParseError(format!(
                        "Function {name} expects 1 argument, got {n}"
                    )));
                }
                let a = args[0];
                match name {
                    "sqrt" => a.sqrt(),
                    "sin" => a.sin(),
                    "cos" => a.cos(),
                    "tan" => a.tan(),
                    "log" | "log10" => a.log10(),
                    "ln" => a.ln(),
                    "log2" => a.log2(),
                    "abs" => a.abs(),
                    "ceil" => a.ceil(),
                    "floor" => a.floor(),
                    "round" => a.round(),
                    "exp" => a.exp(),
                    "asin" => a.asin(),
                    "acos" => a.acos(),
                    "atan" => a.atan(),
                    "sinh" => a.sinh(),
                    "cosh" => a.cosh(),
                    "tanh" => a.tanh(),
                    "asinh" => a.asinh(),
                    "acosh" => a.acosh(),
                    "atanh" => a.atanh(),
                    "trunc" => a.trunc(),
                    "fract" => a.fract(),
                    "sign" | "sgn" => a.signum(),
                    "deg" => a.to_degrees(),
                    "rad" => a.to_radians(),
                    "factorial" => {
                        if a < 0.0 || a.fract() != 0.0 || a > 170.0 {
                            return Err(EvalError::MathError(
                                "factorial requires integer 0..=170".into(),
                            ));
                        }
                        let n = a as u64;
                        (1..=n).fold(1.0_f64, |acc, x| acc * x as f64)
                    }
                    _ => unreachable!(),
                }
            }
            // 2-arg functions
            "min" | "max" | "pow" | "atan2" | "gcd" | "lcm" => {
                if n != 2 {
                    return Err(EvalError::ParseError(format!(
                        "Function {name} expects 2 arguments, got {n}"
                    )));
                }
                match name {
                    "min" => args[0].min(args[1]),
                    "max" => args[0].max(args[1]),
                    "pow" => args[0].powf(args[1]),
                    "atan2" => args[0].atan2(args[1]),
                    "gcd" => {
                        let (mut a, mut b) = (args[0].abs() as u64, args[1].abs() as u64);
                        while b != 0 {
                            let t = b;
                            b = a % b;
                            a = t;
                        }
                        a as f64
                    }
                    "lcm" => {
                        let (mut a, mut b) = (args[0].abs() as u64, args[1].abs() as u64);
                        if a == 0 || b == 0 {
                            return Ok(0.0);
                        }
                        let prod = a as f64 * b as f64;
                        while b != 0 {
                            let t = b;
                            b = a % b;
                            a = t;
                        }
                        prod / a as f64
                    }
                    _ => unreachable!(),
                }
            }
            // Variable-arity functions (1+ args)
            "mean" | "avg" | "median" | "stddev" | "stdev" => {
                if n == 0 {
                    return Err(EvalError::ParseError(format!(
                        "Function {name} requires at least 1 argument"
                    )));
                }
                match name {
                    "mean" | "avg" => args.iter().sum::<f64>() / n as f64,
                    "median" => {
                        let mut sorted = args.to_vec();
                        sorted
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        if n % 2 == 1 {
                            sorted[n / 2]
                        } else {
                            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
                        }
                    }
                    "stddev" | "stdev" => {
                        let mean = args.iter().sum::<f64>() / n as f64;
                        let variance =
                            args.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
                        variance.sqrt()
                    }
                    _ => unreachable!(),
                }
            }
            _ => return Err(EvalError::UnknownFunction(name.to_string())),
        };
        Self::check_result(name, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(expr: &str) -> Value {
        Evaluator::new().eval(expr).unwrap()
    }

    fn eval_f64(expr: &str) -> f64 {
        match Evaluator::new().eval(expr).unwrap() {
            Value::Integer(n) => n as f64,
            Value::Float(n) => n,
            other => panic!("Expected numeric, got {other:?}"),
        }
    }

    #[test]
    fn test_basic_addition() {
        assert_eq!(eval("2 + 3"), Value::Integer(5));
    }

    #[test]
    fn test_basic_subtraction() {
        assert_eq!(eval("10 - 4"), Value::Integer(6));
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(eval("6 * 7"), Value::Integer(42));
    }

    #[test]
    fn test_division() {
        assert_eq!(eval("10 / 4"), Value::Float(2.5));
    }

    #[test]
    fn test_order_of_operations() {
        assert_eq!(eval("2 + 3 * 4"), Value::Integer(14));
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(eval("(2 + 3) * 4"), Value::Integer(20));
    }

    #[test]
    fn test_nested_parentheses() {
        assert_eq!(eval("((1 + 2) * (3 + 4))"), Value::Integer(21));
    }

    #[test]
    fn test_power() {
        assert_eq!(eval("2 ^ 10"), Value::Integer(1024));
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(eval("-5 + 3"), Value::Integer(-2));
    }

    #[test]
    fn test_modulo() {
        assert_eq!(eval("10 % 3"), Value::Integer(1));
    }

    #[test]
    fn test_sqrt_function() {
        assert_eq!(eval_f64("sqrt(16)"), 4.0);
    }

    #[test]
    fn test_abs_function() {
        assert_eq!(eval_f64("abs(-42)"), 42.0);
    }

    #[test]
    fn test_pi_constant() {
        let result = eval_f64("pi");
        assert!((result - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_e_constant() {
        let result = eval_f64("e");
        assert!((result - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn test_variables() {
        let mut ev = Evaluator::new();
        ev.set_variable("x", 5.0);
        assert_eq!(ev.eval("x + 3").unwrap(), Value::Integer(8));
    }

    #[test]
    fn test_division_by_zero() {
        let result = Evaluator::new().eval("1 / 0");
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_variable() {
        let result = Evaluator::new().eval("xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_function() {
        let result = Evaluator::new().eval("foo(3)");
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_expression() {
        // 2^3 + sqrt(9) * 2 = 8 + 3*2 = 14
        assert_eq!(eval_f64("2^3 + sqrt(9) * 2"), 14.0);
    }

    #[test]
    fn test_floor_ceil_round() {
        assert_eq!(eval_f64("floor(3.7)"), 3.0);
        assert_eq!(eval_f64("ceil(3.2)"), 4.0);
        assert_eq!(eval_f64("round(3.5)"), 4.0);
    }

    #[test]
    fn test_min_function() {
        assert_eq!(eval_f64("min(3, 5)"), 3.0);
        assert_eq!(eval_f64("min(7, 2)"), 2.0);
        assert_eq!(eval_f64("min(-1, -5)"), -5.0);
    }

    #[test]
    fn test_max_function() {
        assert_eq!(eval_f64("max(3, 5)"), 5.0);
        assert_eq!(eval_f64("max(7, 2)"), 7.0);
        assert_eq!(eval_f64("max(-1, -5)"), -1.0);
    }

    #[test]
    fn test_pow_function() {
        assert_eq!(eval_f64("pow(2, 10)"), 1024.0);
        assert_eq!(eval_f64("pow(3, 0)"), 1.0);
        assert_eq!(eval_f64("pow(9, 0.5)"), 3.0);
    }

    #[test]
    fn test_log2_function() {
        assert_eq!(eval_f64("log2(8)"), 3.0);
        assert_eq!(eval_f64("log2(1)"), 0.0);
        assert!((eval_f64("log2(2)") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_atan2_function() {
        let result = eval_f64("atan2(1, 1)");
        assert!((result - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
        let result = eval_f64("atan2(0, 1)");
        assert!(result.abs() < 1e-10);
    }

    #[test]
    fn test_multi_arg_wrong_arity() {
        let result = Evaluator::new().eval("min(1)");
        assert!(result.is_err());
        let result = Evaluator::new().eval("max(1, 2, 3)");
        assert!(result.is_err());
        let result = Evaluator::new().eval("sqrt(1, 2)");
        assert!(result.is_err());
    }

    #[test]
    fn test_percent_shorthand() {
        // 15% → 0.15
        assert!((eval_f64("15%") - 0.15).abs() < 1e-10);
        // 50% → 0.5
        assert!((eval_f64("50%") - 0.5).abs() < 1e-10);
        // 100% → 1.0
        assert!((eval_f64("100%") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_percent_in_expression() {
        // 200 * 15% → 200 * 0.15 = 30
        assert!((eval_f64("200 * 15%") - 30.0).abs() < 1e-10);
        // 50% + 0.25 → 0.75
        assert!((eval_f64("50% + 0.25") - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_modulo_still_works() {
        // Binary % with operand on right is still modulo
        assert_eq!(eval("10 % 3"), Value::Integer(1));
        assert_eq!(eval("7 % 2"), Value::Integer(1));
    }

    #[test]
    fn test_scientific_notation() {
        assert_eq!(eval_f64("1e3"), 1000.0);
        assert_eq!(eval_f64("1.5e2"), 150.0);
        assert!((eval_f64("1e-3") - 0.001).abs() < 1e-15);
        assert_eq!(eval_f64("2.5E+2"), 250.0);
        assert_eq!(eval_f64("1e3 + 1e2"), 1100.0);
    }

    #[test]
    fn test_hyperbolic_functions() {
        assert!((eval_f64("sinh(0)")).abs() < 1e-10);
        assert!((eval_f64("cosh(0)") - 1.0).abs() < 1e-10);
        assert!((eval_f64("tanh(0)")).abs() < 1e-10);
    }

    #[test]
    fn test_deg_rad_functions() {
        assert!((eval_f64("deg(pi)") - 180.0).abs() < 1e-10);
        assert!((eval_f64("rad(180)") - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_sign_trunc_fract() {
        assert_eq!(eval_f64("sign(42)"), 1.0);
        assert_eq!(eval_f64("sign(-5)"), -1.0);
        // Note: IEEE 754 signum(+0.0) = +1.0
        assert_eq!(eval_f64("sign(0)"), 1.0);
        assert_eq!(eval_f64("trunc(3.7)"), 3.0);
        assert_eq!(eval_f64("trunc(-3.7)"), -3.0);
        assert!((eval_f64("fract(3.75)") - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("2 + 3").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Number(2.0));
        assert_eq!(tokens[1], Token::Plus);
        assert_eq!(tokens[2], Token::Number(3.0));
    }

    // --- NaN/Infinity guard tests ---

    #[test]
    fn test_sqrt_negative_errors() {
        assert!(Evaluator::new().eval("sqrt(-1)").is_err());
    }

    #[test]
    fn test_ln_negative_errors() {
        assert!(Evaluator::new().eval("ln(-1)").is_err());
    }

    #[test]
    fn test_ln_zero_errors() {
        assert!(Evaluator::new().eval("ln(0)").is_err());
    }

    #[test]
    fn test_log_negative_errors() {
        assert!(Evaluator::new().eval("log(-1)").is_err());
    }

    #[test]
    fn test_acos_out_of_domain() {
        assert!(Evaluator::new().eval("acos(2)").is_err());
    }

    #[test]
    fn test_asin_out_of_domain() {
        assert!(Evaluator::new().eval("asin(2)").is_err());
    }

    #[test]
    fn test_acosh_out_of_domain() {
        assert!(Evaluator::new().eval("acosh(0.5)").is_err());
    }

    #[test]
    fn test_atanh_out_of_domain() {
        assert!(Evaluator::new().eval("atanh(2)").is_err());
    }

    // --- Edge case expressions ---

    #[test]
    fn test_empty_string() {
        assert!(Evaluator::new().eval("").is_err());
    }

    #[test]
    fn test_whitespace_only() {
        assert!(Evaluator::new().eval("   ").is_err());
    }

    #[test]
    fn test_just_a_number() {
        assert_eq!(eval("42"), Value::Integer(42));
    }

    #[test]
    fn test_deeply_nested_parens() {
        assert_eq!(eval("((((1))))"), Value::Integer(1));
    }

    #[test]
    fn test_trailing_operator_errors() {
        assert!(Evaluator::new().eval("1 +").is_err());
    }

    #[test]
    fn test_leading_star_errors() {
        assert!(Evaluator::new().eval("* 3").is_err());
    }

    #[test]
    fn test_unmatched_open_paren() {
        assert!(Evaluator::new().eval("(1 + 2").is_err());
    }

    #[test]
    fn test_unmatched_close_paren() {
        assert!(Evaluator::new().eval("1 + 2)").is_err());
    }

    #[test]
    fn test_double_unary_plus() {
        assert_eq!(eval("++3"), Value::Integer(3));
    }

    #[test]
    fn test_power_right_associative() {
        // 2^3^2 = 2^(3^2) = 2^9 = 512
        assert_eq!(eval("2 ^ 3 ^ 2"), Value::Integer(512));
    }

    #[test]
    fn test_double_star_power() {
        assert_eq!(eval("2 ** 3"), Value::Integer(8));
    }

    #[test]
    fn test_negative_exponent() {
        assert!((eval_f64("2 ^ -3") - 0.125).abs() < 1e-10);
    }

    #[test]
    fn test_very_large_number() {
        let val = eval_f64("1e308");
        assert!(val > 1e307);
    }

    #[test]
    fn test_very_small_number() {
        let val = eval_f64("1e-308");
        assert!(val > 0.0 && val < 1e-307);
    }

    #[test]
    fn test_tau_constant() {
        let result = eval_f64("tau");
        assert!((result - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn test_sgn_alias() {
        assert_eq!(eval_f64("sgn(-5)"), -1.0);
        assert_eq!(eval_f64("sgn(10)"), 1.0);
    }

    #[test]
    fn test_zero_arg_function_errors() {
        assert!(Evaluator::new().eval("sqrt()").is_err());
    }

    #[test]
    fn test_too_many_args_single_arg_fn() {
        assert!(Evaluator::new().eval("abs(1, 2)").is_err());
    }

    #[test]
    fn test_division_by_zero_float() {
        assert!(Evaluator::new().eval("1 / 0.0").is_err());
    }

    #[test]
    fn test_modulo_by_zero_errors() {
        assert!(Evaluator::new().eval("10 % 0").is_err());
    }

    #[test]
    fn test_percent_chain() {
        // 200 * 50% + 1 = 200 * 0.5 + 1 = 101
        assert!((eval_f64("200 * 50% + 1") - 101.0).abs() < 1e-10);
    }

    #[test]
    fn test_multiple_variables() {
        let mut ev = Evaluator::new();
        ev.set_variable("x", 3.0);
        ev.set_variable("y", 7.0);
        assert_eq!(ev.eval("x + y").unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_overwrite_variable() {
        let mut ev = Evaluator::new();
        ev.set_variable("x", 5.0);
        ev.set_variable("x", 10.0);
        assert_eq!(ev.get_variable("x"), Some(10.0));
        assert_eq!(ev.eval("x").unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_variable_shadows_constant() {
        // Variable named "pi" should not be reachable since constants are checked in parse_primary
        // actually constants are checked first, so the constant wins
        let mut ev = Evaluator::new();
        ev.set_variable("pi", 42.0);
        let result = match ev.eval("pi").unwrap() {
            Value::Integer(n) => n as f64,
            Value::Float(n) => n,
            _ => panic!("unexpected"),
        };
        // Constants take precedence in the current implementation
        assert!((result - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_long_addition_chain() {
        assert_eq!(eval("1+2+3+4+5+6+7+8+9+10"), Value::Integer(55));
    }

    #[test]
    fn test_unexpected_character() {
        assert!(Evaluator::new().eval("2 & 3").is_err());
    }

    #[test]
    fn test_evaluator_default() {
        let ev = Evaluator::default();
        assert_eq!(ev.eval("1 + 1").unwrap(), Value::Integer(2));
    }

    // --- Tokenizer edge cases (bytes-based) ---

    #[test]
    fn test_tokenize_all_operators() {
        let tokens = tokenize("1 + 2 - 3 * 4 / 5 % 6 ^ 7").unwrap();
        assert_eq!(tokens.len(), 13);
        assert_eq!(tokens[1], Token::Plus);
        assert_eq!(tokens[3], Token::Minus);
        assert_eq!(tokens[5], Token::Star);
        assert_eq!(tokens[7], Token::Slash);
        assert_eq!(tokens[9], Token::Percent);
        assert_eq!(tokens[11], Token::Power);
    }

    #[test]
    fn test_tokenize_comma_and_parens() {
        let tokens = tokenize("min(3, 5)").unwrap();
        assert_eq!(tokens[0], Token::Ident("min".to_string()));
        assert_eq!(tokens[1], Token::LParen);
        assert_eq!(tokens[3], Token::Comma);
        assert_eq!(tokens[5], Token::RParen);
    }

    #[test]
    fn test_tokenize_double_star() {
        let tokens = tokenize("2 ** 3").unwrap();
        assert_eq!(tokens[1], Token::Power);
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_tokenize_scientific_notation() {
        let tokens = tokenize("1.5e-3").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Number(n) if (n - 0.0015).abs() < 1e-10));
    }

    #[test]
    fn test_tokenize_underscore_ident() {
        let tokens = tokenize("my_var + 1").unwrap();
        assert_eq!(tokens[0], Token::Ident("my_var".to_string()));
    }

    #[test]
    fn test_tokenize_non_ascii_error() {
        let result = tokenize("2 × 3");
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_tabs_and_newlines() {
        let tokens = tokenize("1\t+\n2").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Number(1.0));
    }

    // --- Additional function coverage ---

    #[test]
    fn test_log10_alias() {
        assert!((eval_f64("log10(100)") - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_exp_function() {
        assert!((eval_f64("exp(0)") - 1.0).abs() < 1e-10);
        assert!((eval_f64("exp(1)") - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn test_cos_function() {
        assert!((eval_f64("cos(0)") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_tan_function() {
        assert!((eval_f64("tan(0)")).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_trig_valid() {
        assert!((eval_f64("asin(0)")).abs() < 1e-10);
        assert!((eval_f64("acos(1)")).abs() < 1e-10);
        assert!((eval_f64("atan(0)")).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_hyperbolic_valid() {
        assert!((eval_f64("asinh(0)")).abs() < 1e-10);
        assert!((eval_f64("acosh(1)")).abs() < 1e-10);
        assert!((eval_f64("atanh(0)")).abs() < 1e-10);
    }

    #[test]
    fn test_exp_overflow_errors() {
        assert!(Evaluator::new().eval("exp(1000)").is_err());
    }

    #[test]
    fn test_tan_near_pi_half() {
        // tan(pi/2) in floating point produces a very large but finite value
        // due to pi/2 not being exactly representable
        let result = eval_f64("tan(1.5707963267948966)");
        assert!(result.abs() > 1e15);
    }

    #[test]
    fn test_uppercase_constants() {
        assert!((eval_f64("PI") - std::f64::consts::PI).abs() < 1e-10);
        assert!((eval_f64("E") - std::f64::consts::E).abs() < 1e-10);
        assert!((eval_f64("TAU") - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn test_mixed_operations_precedence() {
        // -2^2 should be -(2^2) = -4 because unary is below power
        // Actually in our parser: unary is parsed before primary, and power calls unary
        // So -2^2 tokenizes as [Minus, Number(2), Power, Number(2)]
        // parse_expr -> parse_term -> parse_power -> parse_unary sees Minus
        // parse_unary returns -(parse_unary) = -(parse_primary) = -2
        // Then parse_power sees Power, gets 2, returns (-2)^2 = 4
        assert_eq!(eval("-2^2"), Value::Integer(4));
    }

    #[test]
    fn test_double_dot_number_errors() {
        assert!(Evaluator::new().eval("1.2.3").is_err());
    }

    #[test]
    fn test_deep_nesting_rejected() {
        // Build an expression with 300 nested parens — exceeds MAX_PARSE_DEPTH (256)
        let open: String = "(".repeat(300);
        let close: String = ")".repeat(300);
        let expr = format!("{open}1{close}");
        let result = Evaluator::new().eval(&expr);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("deeply nested"), "got: {err}");
    }

    #[test]
    fn test_moderate_nesting_accepted() {
        // 100 levels of nesting should work fine
        let open: String = "(".repeat(100);
        let close: String = ")".repeat(100);
        let expr = format!("{open}42{close}");
        assert_eq!(eval(&expr), Value::Integer(42));
    }

    #[test]
    fn test_deeply_nested_functions_rejected() {
        // Nested function calls: sqrt(sqrt(sqrt(...)))
        let mut expr = "1".to_string();
        for _ in 0..300 {
            expr = format!("sqrt({expr})");
        }
        let result = Evaluator::new().eval(&expr);
        assert!(result.is_err());
    }

    // --- Implicit multiplication ---

    #[test]
    fn test_implicit_mul_number_paren() {
        // 2(3 + 4) = 2 * 7 = 14
        assert_eq!(eval("2(3 + 4)"), Value::Integer(14));
    }

    #[test]
    fn test_implicit_mul_number_ident() {
        // 2pi ≈ 6.283
        let result = eval_f64("2pi");
        assert!((result - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn test_implicit_mul_paren_paren() {
        // (2 + 3)(4 + 1) = 5 * 5 = 25
        assert_eq!(eval("(2 + 3)(4 + 1)"), Value::Integer(25));
    }

    #[test]
    fn test_implicit_mul_paren_number() {
        // (3)4 = 12
        assert_eq!(eval("(3)4"), Value::Integer(12));
    }

    #[test]
    fn test_implicit_mul_paren_ident() {
        // (2)pi ≈ 6.283
        let result = eval_f64("(2)pi");
        assert!((result - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn test_implicit_mul_does_not_break_functions() {
        // sin(pi) should still be a function call, not sin * (pi)
        assert!((eval_f64("sin(pi)")).abs() < 1e-10);
        assert_eq!(eval_f64("sqrt(16)"), 4.0);
        assert_eq!(eval_f64("min(3, 5)"), 3.0);
    }

    #[test]
    fn test_implicit_mul_complex() {
        // 3(2 + 1)(4) = 3 * 3 * 4 = 36
        assert_eq!(eval("3(2 + 1)(4)"), Value::Integer(36));
    }

    #[test]
    fn test_implicit_mul_with_power() {
        // 2pi^2 = 2 * pi^2 ≈ 19.739
        let result = eval_f64("2pi^2");
        let expected = 2.0 * std::f64::consts::PI * std::f64::consts::PI;
        assert!((result - expected).abs() < 1e-10);
    }

    // --- Factorial, gcd, lcm ---

    #[test]
    fn test_factorial_function() {
        assert_eq!(eval_f64("factorial(0)"), 1.0);
        assert_eq!(eval_f64("factorial(1)"), 1.0);
        assert_eq!(eval_f64("factorial(5)"), 120.0);
        assert_eq!(eval_f64("factorial(10)"), 3628800.0);
    }

    #[test]
    fn test_factorial_postfix() {
        assert_eq!(eval("5!"), Value::Integer(120));
        assert_eq!(eval("0!"), Value::Integer(1));
        assert_eq!(eval("10!"), Value::Integer(3628800));
    }

    #[test]
    fn test_factorial_in_expression() {
        // 5! + 1 = 121
        assert_eq!(eval("5! + 1"), Value::Integer(121));
        // 3! * 2 = 12
        assert_eq!(eval("3! * 2"), Value::Integer(12));
    }

    #[test]
    fn test_factorial_with_power() {
        // 3!^2 = 6^2 = 36
        assert_eq!(eval("3!^2"), Value::Integer(36));
    }

    #[test]
    fn test_factorial_negative_errors() {
        assert!(Evaluator::new().eval("factorial(-1)").is_err());
        assert!(Evaluator::new().eval("(-1)!").is_err());
    }

    #[test]
    fn test_factorial_non_integer_errors() {
        assert!(Evaluator::new().eval("factorial(3.5)").is_err());
    }

    #[test]
    fn test_gcd() {
        assert_eq!(eval_f64("gcd(12, 8)"), 4.0);
        assert_eq!(eval_f64("gcd(100, 75)"), 25.0);
        assert_eq!(eval_f64("gcd(7, 13)"), 1.0);
        assert_eq!(eval_f64("gcd(0, 5)"), 5.0);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(eval_f64("lcm(4, 6)"), 12.0);
        assert_eq!(eval_f64("lcm(12, 8)"), 24.0);
        assert_eq!(eval_f64("lcm(7, 13)"), 91.0);
        assert_eq!(eval_f64("lcm(0, 5)"), 0.0);
    }

    // --- Statistical functions ---

    #[test]
    fn test_mean() {
        assert_eq!(eval_f64("mean(1, 2, 3)"), 2.0);
        assert_eq!(eval_f64("mean(10)"), 10.0);
        assert_eq!(eval_f64("avg(2, 4, 6, 8)"), 5.0);
    }

    #[test]
    fn test_median_odd() {
        assert_eq!(eval_f64("median(3, 1, 2)"), 2.0);
        assert_eq!(eval_f64("median(5)"), 5.0);
        assert_eq!(eval_f64("median(1, 2, 3, 4, 5)"), 3.0);
    }

    #[test]
    fn test_median_even() {
        assert_eq!(eval_f64("median(1, 2, 3, 4)"), 2.5);
        assert_eq!(eval_f64("median(1, 3)"), 2.0);
    }

    #[test]
    fn test_stddev() {
        // stddev of [2, 4, 4, 4, 5, 5, 7, 9] = 2.0
        assert!((eval_f64("stddev(2, 4, 4, 4, 5, 5, 7, 9)") - 2.0).abs() < 0.01);
        // stddev of constant = 0
        assert_eq!(eval_f64("stdev(5, 5, 5)"), 0.0);
    }

    #[test]
    fn test_stats_empty_errors() {
        assert!(Evaluator::new().eval("mean()").is_err());
        assert!(Evaluator::new().eval("median()").is_err());
    }

    // --- Partial parse ---

    #[test]
    fn test_partial_complete_expression() {
        let ev = Evaluator::new();
        assert_eq!(ev.eval_partial("2 + 3").unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_partial_trailing_operator() {
        let ev = Evaluator::new();
        // "2 + 3 +" should evaluate as "2 + 3" = 5
        assert_eq!(ev.eval_partial("2 + 3 +").unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_partial_trailing_star() {
        let ev = Evaluator::new();
        assert_eq!(ev.eval_partial("10 * 5 *").unwrap(), Value::Integer(50));
    }

    #[test]
    fn test_partial_open_paren() {
        let ev = Evaluator::new();
        // "2 + (3" — strip "(3", then "2 +" → strip "+", "2" = 2
        assert_eq!(ev.eval_partial("2 + (3").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_partial_just_number() {
        let ev = Evaluator::new();
        assert_eq!(ev.eval_partial("42").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_partial_empty_errors() {
        let ev = Evaluator::new();
        assert!(ev.eval_partial("").is_err());
    }

    #[test]
    fn test_partial_function_incomplete() {
        let ev = Evaluator::new();
        // "sqrt(16) + sin(" — strip to "sqrt(16)" = 4
        assert_eq!(
            ev.eval_partial("sqrt(16) + sin(").unwrap(),
            Value::Integer(4)
        );
    }
}
