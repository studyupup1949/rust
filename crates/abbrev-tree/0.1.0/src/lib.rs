use std::fmt;
use std::mem;

pub struct AbbrevTree<T> {
    v: Vec<(String, AbbrevTree<T>)>,
    pub data: T,
}

impl<T: Default> AbbrevTree<T> {
    pub fn new() -> Self {
        AbbrevTree { v: Vec::new(), data: Default::default() }
    }

    // TODO: Recursion is probably bad but oh well.
    pub fn add(&mut self, item: &str, data: T) {
        // Find match.
        for (chunk, subtree) in &mut self.v {
            let prefix_len = common_prefix_length(chunk, item);
            if prefix_len > 0 {
                if prefix_len == chunk.len() {
                    // Full match. Recurse.
                    if subtree.v.len() == 0 {
                        let d = mem::replace(
                            &mut subtree.data, Default::default()
                        );
                        subtree.v.push((
                            "".to_string(),
                            AbbrevTree { v: Vec::new(), data: d },
                        ))
                    }
                    return subtree.add(&item[prefix_len..], data);
                } else {
                    // Partial match. Split and then add.
                    let chunk_suffix = chunk.split_off(prefix_len);
                    let d = mem::replace(&mut subtree.data, Default::default());
                    let v: Vec<_> = subtree.v.drain(..).collect();
                    subtree.v.push((
                        chunk_suffix,
                        AbbrevTree { v, data: d },
                    ));
                    return subtree.add(&item[prefix_len..], data);
                }
            }
        }

        // Else add new subtree.
        self.v.push((
            item.to_string(),
            AbbrevTree { v: Vec::new(), data },
        ));
    }

    pub fn complete<'d>(&'d self, item: &str) -> Vec<(String, &'d T)> {
        let mut v = Vec::new();
        self._complete("", item, &mut v);
        v
    }

    fn _complete<'d>(
        &'d self, left: &str, item: &str, v: &mut Vec<(String, &'d T)>
    ) {
        if self.v.len() == 0 && item == "" {
            v.push((left.to_string(), &self.data));
        }

        for (chunk, subtree) in &self.v {
            let prefix_len = common_prefix_length(chunk, item);
            // TODO: Make sure this makes sense.
            if item == "" || item.len() == prefix_len
                    || chunk.len() == prefix_len {
                let mut s = left.to_string();
                s.push_str(chunk);
                subtree._complete(&s, &item[prefix_len..], v);
            }
        }
    }

    pub fn get_mut<'d>(&'d mut self, item: &str) -> Option<&'d mut T> {
        self._get_mut("", item)
    }

    fn _get_mut<'d>(&'d mut self, left: &str, item: &str) -> Option<&'d mut T> {
        if self.v.len() == 0 && item == "" {
            // We're a leaf and item is exhausted.
            return Some(&mut self.data);
        }

        for (chunk, subtree) in &mut self.v {
            let prefix_len = common_prefix_length(chunk, item);
            if prefix_len == chunk.len() {
                let mut s = left.to_string();
                s.push_str(chunk);
                match subtree._get_mut(&s, &item[prefix_len..]) {
                    Some(d) => return Some(d),
                    None => (),
                }
            }
        }

        None
    }
}

impl<T> fmt::Debug for AbbrevTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME: We should check for f.alternate().
        let mut stack = vec![self.v.iter()];
        let mut first = true;
        while stack.len() > 0 {
            match stack.last_mut().unwrap().next() {
                Some(x) => {
                    if !first {
                        write!(f, "\n")?;
                    }
                    write!(
                        f,
                        "{}{:?}",
                        " ".repeat(2 * (stack.len()-1)),
                        x.0,
                    )?;
                    stack.push((x.1).v.iter());
                },
                None => { stack.pop(); },
            }
            first = false;
        }
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_abbrev_tree() {
    let mut t = AbbrevTree::new();
    println!("{:?}", t);
    assert_eq!(t.v.len(), 0);

    t.add("cat", ());
    println!("{:?}", t);
    assert_eq!(t.v.len(), 1);
    assert_eq!(t.v[0].0, "cat");
    assert_eq!((t.v[0].1).v.len(), 0);

    t.add("cargo", ());
    println!("{:?}", t);
    assert_eq!(t.v.len(), 1);
    assert_eq!(t.v[0].0, "ca");
    assert_eq!((t.v[0].1).v.len(), 2);
    assert_eq!((t.v[0].1).v[0].0, "t");
    assert_eq!(((t.v[0].1).v[0].1).v.len(), 0);
    assert_eq!((t.v[0].1).v[1].0, "rgo");
    assert_eq!(((t.v[0].1).v[1].1).v.len(), 0);

    t.add("chmod", ());
    println!("{:?}", t);
    assert_eq!(t.v.len(), 1);
    assert_eq!(t.v[0].0, "c");

    t.add("chown", ());
    println!("{:?}", t);
    assert_eq!(t.v.len(), 1);
    assert_eq!(t.v[0].0, "c");

    t.add("ls", ());
    println!("{:?}", t);
    assert_eq!(t.v.len(), 2);
    assert_eq!(t.v[0].0, "c");
    assert_eq!(t.v[1].0, "ls");

    t.add("lshw", ());
    println!("{:?}", t);

    fn first<A, B, I: IntoIterator<Item = (A, B)>>(i: I) -> Vec<A> {
        i.into_iter().map(|x: (_, _)| x.0).collect()
    }

    assert_eq!(first(t.complete("c")), vec![
        "cat".to_string(),
        "cargo".to_string(),
        "chmod".to_string(),
        "chown".to_string(),
    ]);
    assert_eq!(first(t.complete("ca")), vec![
        "cat".to_string(),
        "cargo".to_string(),
    ]);
    assert_eq!(first(t.complete("cat")), vec!["cat".to_string()]);
    assert_eq!(first(t.complete("ch")), vec![
        "chmod".to_string(),
        "chown".to_string(),
    ]);
    assert_eq!(first(t.complete("cho")), vec!["chown".to_string()]);
    assert_eq!(first(t.complete("chow")), vec!["chown".to_string()]);
    assert_eq!(first(t.complete("chown")), vec!["chown".to_string()]);
    assert_eq!(first(t.complete("l")), vec![
        "ls".to_string(),
        "lshw".to_string(),
    ]);
    assert_eq!(first(t.complete("ls")), vec![
        "ls".to_string(),
        "lshw".to_string(),
    ]);
    assert_eq!(first(t.complete("lsh")), vec!["lshw".to_string()]);
    assert_eq!(first(t.complete("lshw")), vec!["lshw".to_string()]);
    assert_eq!(first(t.complete("x")), Vec::<String>::new());
    assert_eq!(first(t.complete("xyz")), Vec::<String>::new());

    assert!(t.get_mut("c").is_none());
    assert!(t.get_mut("ca").is_none());
    t.get_mut("cat").unwrap();
    t.get_mut("cargo").unwrap();
    t.get_mut("chmod").unwrap();
    t.get_mut("chown").unwrap();
    t.get_mut("ls").unwrap();
    t.get_mut("lshw").unwrap();
    assert!(t.get_mut("xyz").is_none());
}

fn common_prefix_length(a: &str, b: &str) -> usize {
    let mut aa = a.char_indices();
    let mut bb = b.char_indices();

    loop {
        match (aa.next(), bb.next()) {
            (Some((ai, ac)), Some((_, bc))) =>
                if ac != bc {
                    return ai;
                },
            (None, Some((bi, _))) => return bi,
            (Some((ai, _)), None) => return ai,
            (None, None) => return a.len(),
        }
    }
}

#[cfg(test)]
#[test]
fn test_common_prefix_length() {
    assert_eq!(common_prefix_length("", "foo"), 0);
    assert_eq!(common_prefix_length("foo", "foo"), 3);
    assert_eq!(common_prefix_length("foo", "foobar"), 3);
    assert_eq!(common_prefix_length("foobar", "foo"), 3);
    assert_eq!(common_prefix_length("foobar", "bar"), 0);
    assert_eq!(common_prefix_length("foobar", "foofuzz"), 3);
}
