pub const fn char2num(c: char) -> isize {
    match c {
        ' ' => 0,
        // NOTE(Able): Why does it jump to 53 here? MY REASONS ARE BEYOND YOUR UNDERSTANDING MORTAL
        '/' => 53,
        '\\' => 54,
        '.' => 55,
        'U' => -210,
        'A'..='Z' => -(c as isize) + 64,
        'a'..='z' => (c as isize) - 96,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn str_to_base55() {
        let chrs: Vec<isize> = "AbleScript".chars().map(char2num).collect();
        assert_eq!(chrs, &[-1, 2, 12, 5, -19, 3, 18, 9, 16, 20]);
    }
}
