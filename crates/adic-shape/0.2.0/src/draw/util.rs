pub (crate) fn str_digits(x: f64, precision: usize) -> String {
    format!("{x:.precision$}").trim_end_matches('0').trim_end_matches('.').to_string()
}
