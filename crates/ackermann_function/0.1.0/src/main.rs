/// Ackermann Function — compute A(m, n) with memoisation.
///
/// A(0, n)   = n + 1
/// A(m, 0)   = A(m−1, 1)           for m > 0
/// A(m, n)   = A(m−1, A(m, n−1))   for m, n > 0
///
/// Demonstrates extreme recursion depth; uses an explicit stack to
/// avoid blowing the call stack and a HashMap for memoisation.

use std::collections::HashMap;

type Key = (u32, u64);

fn ackermann(m: u32, n: u64) -> u64 {
    let mut memo: HashMap<Key, u64> = HashMap::new();
    let mut stack: Vec<Key> = vec![(m, n)];

    while let Some(key @ (cm, cn)) = stack.last().copied() {
        if let Some(&val) = memo.get(&key) {
            stack.pop();
            // Feed result back into parent
            if let Some(parent) = stack.last_mut() {
                if parent.0 == cm + 1 && parent.1 == u64::MAX {
                    *parent = (cm, val);
                }
            }
            continue;
        }

        if cm == 0 {
            let val = cn + 1;
            memo.insert(key, val);
            stack.pop();
            if let Some(parent) = stack.last_mut() {
                if parent.0 == 1 && parent.1 == u64::MAX {
                    *parent = (0, val);
                }
            }
        } else if cn == 0 {
            // A(m, 0) = A(m-1, 1)
            stack.pop();
            stack.push((cm, u64::MAX)); // sentinel: need A(cm-1, ?)
            stack.push((cm - 1, 1));
        } else {
            // A(m, n) = A(m-1, A(m, n-1))
            // Check if A(m, n-1) is known
            if let Some(&inner) = memo.get(&(cm, cn - 1)) {
                stack.pop();
                stack.push((cm - 1, inner));
            } else {
                stack.push((cm, cn - 1));
            }
        }
    }

    memo[&(m, n)]
}

fn main() {
    println!("Ackermann Function A(m, n)");
    println!();
    println!("{:>4}  {:>4}  {:>12}  {}", "m", "n", "A(m,n)", "digits");
    println!("{}", "─".repeat(48));

    let cases: Vec<(u32, u64)> = vec![
        (0, 0), (0, 5), (0, 10),
        (1, 1), (1, 5), (1, 10),
        (2, 2), (2, 5), (2, 10),
        (3, 3), (3, 5), (3, 6),
        (4, 1),
    ];

    for (m, n) in cases {
        let val = ackermann(m, n);
        let digits = if val < 10 { 1 } else { (val as f64).log10() as usize + 1 };
        println!("{m:>4}  {n:>4}  {val:>12}  {digits} digits");
    }

    // Show growth rate
    println!("\nA(4, 2) is too large to compute directly — it has 19729 decimal digits.");
}
