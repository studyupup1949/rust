use ext::{no_std::pipelines::tap::Tap, standard::functions::{ext::StdAnyExt, fun::read_line}};
use itertools::Itertools;
use std::process;

fn main() {
    loop {
        read_line()
            .tap(|s| if s.trim_end().is_empty() { process::exit(0) })
            .chars()
            // 阶段1：分组潜在数字段: 判断是否是合法的数字起始字符
            .chunk_by(|&c| c.is_ascii_digit() || c == '+' || c == '-' || c == '.')
            .into_iter()
            // 过滤无效字符组
            .filter_map(|(valid, group)| valid.then(|| group.collect::<String>()))
            // 阶段2：转换数字段为实际数字
            .flat_map(|segment| {
                // 从最长到最短尝试所有前缀
                (1..=segment.len())
                    .rev()
                    .find_map(|len| segment[..len].parse::<f64>().ok())
            })
            .sum::<f64>()
            .echo();
    }
}