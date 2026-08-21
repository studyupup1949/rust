


macro_rules! format_number_int {
    ($x: expr, $p: expr, $t: expr, $d: expr) => {
        {
            let mut x = $x;
            let precision = $p;
            let thousand = $t;
            let decimal = $d;

            let mut result: String = "".to_string();
            let mut minus: bool = false;

            if x < 0 {
                minus = true;
                x *= -1;
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

macro_rules! format_number_uint {
    ($x: expr, $p: expr, $t: expr, $d: expr) => {
        {
            let mut x = $x;
            let precision = $p;
            let thousand = $t;
            let decimal = $d;

            let mut result: String = "".to_string();

            while x>=1000  {
                result = format!("{}{:03}{}", thousand, x%1000, result);
                x /= 1000;
            }
            result = format!("{}{}", x, result);
        
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

pub(super) use format_number_int;
pub(super) use format_number_uint;
pub(super) use format_number_float;