
use super::FormatNumber;

macro_rules! format_number_int {
    ($x: expr, $p: expr, $t: expr, $d: expr) => {
        {
            let mut x = $x;
            let precision = $p;
            let thousand = $t;
            let decimal = $d;

            let mut result: String = "".to_string();
            let mut minus: bool = false;

            if let Some(neg) = x.checked_neg() {
                if neg > 0 {
                    x = neg;
                    minus = true;
                }
            }

            while x>=1000  {
                result = format!("{}{:03}{}", thousand, x%1000, result);
                x /= 1000;
            }
            result = format!("{}{}", x, result);

            if minus {
                result = format!("-{}", result);
            }
        
            if precision > 0 {
                result = format!("{}{}{}", result, decimal, "0".repeat(precision))
            } 

            result
        }
    };
}


macro_rules! format_number_float {
    ($x: expr, $p: expr, $t: expr, $d: expr) => {
        {
            let x = $x;
            let precision = $p;
            let thousand = $t;
            let decimal = $d;

            let v:Vec<char> = format!("{0:.1$}", x, precision).chars().collect();
    
            let l;
            if let Some(index) = v.iter().position(|&r| r == '.') {
                l = index - 1;
            } else {
                l = v.len() - 1;
            }
            
            let mut buffer = String::new();
            let mut j = 0;
            for i in (0..=l).rev() {
                j += 1;
                buffer.push(v[i]);
                if j==3 && i>0 && !(i==1 && v[0] == '-') {
                    buffer.push(',');
                    j = 0;
                }
            }

            let mut result: String = buffer.chars().rev().collect();
            
            if thousand != "," {
                result = result.replace(",", thousand);
            }

            let mut extra: String = v[l+1..v.len()].into_iter().collect();
            if decimal != "." {
                extra = extra.replace(".", decimal);
            }

            format!("{}{}", result, extra)
        }
    }
    
}

impl FormatNumber for i8 {
    fn format_number(&self, precision: usize, _: &str, decimal: &str) -> String {
        if precision > 0 {
            format!("{}{}{}", *self, decimal, "0".repeat(precision))
        } else {
            self.to_string()
        }
    }
}

impl FormatNumber for u8 {
    fn format_number(&self, precision: usize, _: &str, decimal: &str) -> String {
        if precision > 0 {
            format!("{}{}{}", *self, decimal, "0".repeat(precision))
        } else {
            self.to_string()
        }
    }
}

impl FormatNumber for i16 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for i32 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for i64 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for i128 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for isize {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for u16 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for u32 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for u64 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for u128 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for usize {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_int!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for f32 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_float!(*self, precision, thousand, decimal)
    }
}

impl FormatNumber for f64 {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        format_number_float!(*self, precision, thousand, decimal)
    }
}