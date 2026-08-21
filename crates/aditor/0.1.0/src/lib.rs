/// 给定一个时间戳（秒），加上指定秒数，返回新的时间戳（秒）
pub fn add_seconds(timestamp: u64, seconds: u64) -> u64 {
    timestamp.saturating_add(seconds)
}

/// 给定一个时间戳（秒），减去指定秒数，返回新的时间戳（秒）
pub fn sub_seconds(timestamp: u64, seconds: u64) -> u64 {
    timestamp.saturating_sub(seconds)
}

/// 计算两个时间戳（秒）之间的差值（秒）
pub fn diff_seconds(ts1: u64, ts2: u64) -> u64 {
    if ts1 > ts2 {
        ts1 - ts2
    } else {
        ts2 - ts1
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_seconds() {
        assert_eq!(add_seconds(1000, 60), 1060);
    }

    #[test]
    fn test_sub_seconds() {
        assert_eq!(sub_seconds(1000, 60), 940);
        assert_eq!(sub_seconds(50, 100), 0); // 不会为负数
    }

    #[test]
    fn test_diff_seconds() {
        assert_eq!(diff_seconds(1000, 800), 200);
        assert_eq!(diff_seconds(800, 1000), 200);
        assert_eq!(diff_seconds(1000, 1000), 0);
    }
}
