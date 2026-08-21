/// 数值格式化工具
///
/// 根据数值大小自动选择合适的显示格式：
/// - < 1,000: 整数 (如 847)
/// - 1,000 ~ 999,999: 千分位逗号 (如 12,847)
/// - 1,000,000 ~ 999,999,999: 缩写 + 1位小数 (如 12.8M)
/// - 1,000,000,000 ~ 999,999,999,999: 缩写 + 2位小数 (如 1.28B)
/// - 1,000,000,000,000+: 科学计数法 (如 1.28e12)

/// 将 f64 数值格式化为人类可读的字符串
pub fn format_number(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }

    let negative = n < 0.0;
    let abs = n.abs();
    let prefix = if negative { "-" } else { "" };

    if abs < 1_000.0 {
        // < 1,000: 整数
        format!("{}{}", prefix, abs as u64)
    } else if abs < 1_000_000.0 {
        // 1,000 ~ 999,999: 千分位逗号
        let int_val = abs as u64;
        format!("{}{}", prefix, format_with_commas(int_val))
    } else if abs < 1_000_000_000.0 {
        // 1M ~ 999M: 缩写 + 1位小数
        let val = abs / 1_000_000.0;
        format!("{}{:.1}M", prefix, val)
    } else if abs < 1_000_000_000_000.0 {
        // 1B ~ 999B: 缩写 + 2位小数
        let val = abs / 1_000_000_000.0;
        format!("{}{:.2}B", prefix, val)
    } else {
        // 1T+: 科学计数法
        let exp = abs.log10().floor() as i32;
        let mantissa = abs / 10_f64.powi(exp);
        format!("{}{:.2}e{}", prefix, mantissa, exp)
    }
}

/// 为整数添加千分位逗号
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + (len - 1) / 3);

    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(format_number(0.0), "0");
    }

    #[test]
    fn test_small_integers() {
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(847.0), "847");
        assert_eq!(format_number(999.0), "999");
    }

    #[test]
    fn test_thousands_with_commas() {
        assert_eq!(format_number(1_000.0), "1,000");
        assert_eq!(format_number(12_847.0), "12,847");
        assert_eq!(format_number(999_999.0), "999,999");
        assert_eq!(format_number(100_000.0), "100,000");
    }

    #[test]
    fn test_millions_abbreviation() {
        assert_eq!(format_number(1_000_000.0), "1.0M");
        assert_eq!(format_number(12_800_000.0), "12.8M");
        assert_eq!(format_number(999_000_000.0), "999.0M");
        assert_eq!(format_number(1_500_000.0), "1.5M");
    }

    #[test]
    fn test_billions_abbreviation() {
        assert_eq!(format_number(1_000_000_000.0), "1.00B");
        assert_eq!(format_number(1_280_000_000.0), "1.28B");
        assert_eq!(format_number(999_000_000_000.0), "999.00B");
    }

    #[test]
    fn test_trillions_scientific() {
        assert_eq!(format_number(1_000_000_000_000.0), "1.00e12");
        assert_eq!(format_number(1_280_000_000_000.0), "1.28e12");
        assert_eq!(format_number(5_500_000_000_000_000.0), "5.50e15");
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(format_number(-42.0), "-42");
        assert_eq!(format_number(-12_847.0), "-12,847");
        assert_eq!(format_number(-1_500_000.0), "-1.5M");
        assert_eq!(format_number(-1_280_000_000.0), "-1.28B");
        assert_eq!(format_number(-1_000_000_000_000.0), "-1.00e12");
    }

    #[test]
    fn test_fractional_small_numbers() {
        // 小于 1000 的小数应截断为整数
        assert_eq!(format_number(3.7), "3");
        assert_eq!(format_number(99.9), "99");
    }
}
