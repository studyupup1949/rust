use std::collections::HashMap;

/// 计算 vec 里出现次数最多的元素
pub fn count_show<T: Eq + std::hash::Hash + Clone>(v: &Vec<T>) -> Option<T> {
    let mut hm: HashMap<&T, i32> = HashMap::new();
    for item in v {
        let num = hm.entry(item).or_insert(0);
        *num += 1;
    }

    let mut most: Option<T> = None;
    let mut max: &i32 = &0;
    for (&k, val) in &hm {
        if val > max {
            most = Some(k.clone());
            max = val;
        }
    }
    most
}
