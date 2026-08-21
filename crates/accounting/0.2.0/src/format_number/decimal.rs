
use super::FormatNumber;
use rust_decimal::Decimal;


impl FormatNumber for Decimal {
    fn format_number(&self, precision: usize, thousand: &str, decimal: &str) -> String {
        
        let x = *self;
        let precision = precision as u32;
        let v:Vec<char> = x.round_dp(precision).to_string().chars().collect();

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

