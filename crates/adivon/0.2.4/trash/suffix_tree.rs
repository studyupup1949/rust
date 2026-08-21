use std::fmt;
use std::mem;
use std::ptr;

use std::collections::btree_map::BTreeMap;
use super::Queue;

pub use self::Node::*;

#[allow(raw_pointer_derive)]
#[derive(Debug)]
struct Rawlink<T> {
    p: *mut T,
}

impl<T> Copy for Rawlink<T> {}

impl<T> Clone for Rawlink<T> {
    fn clone(&self) -> Rawlink<T> { *self }
}

/// Rawlink is a type like Option<T> but for holding a raw mutable pointer.
impl<T> Rawlink<T> {
    /// Like `Option::None` for Rawlink.
    fn none() -> Rawlink<T> {
        Rawlink{p: ptr::null_mut()}
    }

    /// Like `Option::Some` for Rawlink
    fn some(n: &mut T) -> Rawlink<T> {
        Rawlink{p: n}
    }

    fn is_null(&self) -> bool {
        self.p.is_null()
    }

    /// Convert the `Rawlink` into an immutable Option value.
    fn resolve<'a>(&self) -> Option<&'a T> {
        if self.p.is_null() {
            None
        } else {
            Some(unsafe { &*self.p })
        }
    }

    /// Convert the `Rawlink` into a mutable Option value.
    fn resolve_mut<'a>(&mut self) -> Option<&'a mut T> {
        if self.p.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.p })
        }
    }
}

/// a node in SuffixTree
#[derive(Debug, Clone)]
enum Node<T> {
    /// Leaf node
    Leaf {
        start: usize,
        /// which strings terminates at this node
        from: Vec<usize>
    },
    Internal {
        start: usize,
        end: usize,
        /// which strings terminates at this node
        from: Vec<usize>,
        children: BTreeMap<T, Node<T>>,
        suffix_link: Rawlink<Node<T>>
    },
    Root { children: BTreeMap<T, Node<T>> }
}

impl<T: Ord + Copy + fmt::Debug> Node<T> {
    pub fn root() -> Node<T> {
        Root { children: BTreeMap::new() }
    }

    pub fn leaf(start: usize, from: Vec<usize>) -> Node<T> {
        Leaf { start: start, from: from }
    }

    pub fn internal(start: usize, end: usize, from: Vec<usize>) -> Node<T> {
        Internal {
            start: start, end: end, from: from,
            children: BTreeMap::new(),
            suffix_link: Rawlink::none() }
    }

    pub fn add_child(&mut self, x: Node<T>, string: &[T]) {
        match *self {
            Root { ref mut children } => {
                children.insert(x.head(string), x);
            },
            Internal { ref mut children, .. } => {
                children.insert(x.head(string), x);
            },
            Leaf { .. } => panic!("leaf can't have a child")
        }
    }

    fn start(&self) -> usize {
        match *self {
            Leaf { start, .. } => {
                start
            },
            Internal { start, .. } => {
                start
            },
            Root { .. } => 0
        }
    }

    fn shrink(&mut self, offset: usize) {
        match *self {
            Leaf { ref mut start, .. } => {
                *start += offset;
            },
            Internal { ref mut start, ref end, .. } => {
                *start += offset;
                assert!(*start < *end);
            },
            Root { .. } => panic!("can't shrink root node")
        }
    }

    pub fn from(&self) -> Vec<usize> {
        match *self {
            Leaf { ref from, .. } => {
                from.clone()
            },
            Internal { ref from, .. } => {
                from.clone()
            }
            _ => panic!("calling from() on wrong node")
        }
    }

    pub fn split_at(&mut self, offset: usize, seq: &[T]) {
        let new = Node::internal(self.start(), self.start()+offset, self.from().clone());
        let mut old = mem::replace(self, new);
        if let Internal { ref mut suffix_link, .. } = *self {
            *suffix_link = old.suffix_link();
        }
        if let Internal { ref mut suffix_link, .. } = old {
            *suffix_link = Rawlink::none();
        }
        old.shrink(offset);

        println!("crash at 3?");
        self.add_child(old, seq);
    }

    pub fn slice<'a>(&self, seq: &'a[&'a [T]]) -> &'a [T] {
        match *self {
            Internal { start, end, .. } => &seq[0][start..end],
            Leaf { start, .. } => &seq[0][start..],
            _ => panic!("root can't seq")
        }
    }

    pub fn head(&self, seq: &[T]) -> T {
        match *self {
            Internal { start, .. } => seq[start],
            Leaf { start, .. } => seq[start],
            _ => panic!("root have no head")
        }
    }

    pub fn iter_children<'t>(&'t self) -> ::std::collections::btree_map::Values<'t, T, Node<T>> {
        match *self {
            Root { ref children } => children.values(),
            Internal { ref children, .. } => children.values(),
            Leaf { .. } => panic!("leaf have no children")
        }
    }

    pub fn is_leaf(&self) -> bool {
        if let Leaf { .. } = *self { true } else { false }
    }

    pub fn is_root(&self) -> bool {
        if let Root { .. } = *self { true } else { false }
    }

    pub fn is_internal(&self) -> bool {
        if let Internal { .. } = *self { true } else { false }
    }

    fn length(&self, pos: usize) -> usize {
        match *self {
            Leaf { ref start, .. } => pos - start,
            Internal { ref start, ref end, .. } => {
                if *end < pos {
                    *end - *start
                } else {
                    pos - *start
                }
            },
            Root { .. } => 0,
        }
    }

    fn add_suffix_link(&mut self, slink: Rawlink<Node<T>>) {
        match *self {
            Internal { ref mut suffix_link, .. } => {
                *suffix_link = slink;
            }
            _ => {}
        }
    }

    fn suffix_link(&self) -> Rawlink<Node<T>> {
        match *self {
            Internal { suffix_link, .. } => {
                suffix_link.clone()
            }
            _ => {
                Rawlink::none()
            }
        }
    }

    pub fn mut_child_starts_with<'t>(&'t mut self, c: &T) -> Option<&'t mut Node<T>> {
        match *self {
            Root { ref mut children } => children.get_mut(c),
            Internal { ref mut children, .. } => children.get_mut(c),
            Leaf { .. } => panic!("leaf have no children")
        }
    }
    pub fn child_starts_with(&self, c: &T) -> Option<&Node<T>> {
        match *self {
            Root { ref children } => children.get(c),
            Internal { ref children, .. } => children.get(c),
            Leaf { .. } => None
        }
    }
}

#[derive(Debug)]
pub struct SuffixTree<'a, T: Sized + 'a> {
    origs: Vec<&'a [T]>,
    root: Node<T>
}

impl<'a, T: Ord + Copy + fmt::Debug> SuffixTree<'a, T> {
    pub fn new(text: &'a [T]) -> SuffixTree<'a, T> {
        let mut st = SuffixTree {
            origs: vec![],
            root: Node::root()
        };
        st.add(text);
        st
    }

    pub fn add(&mut self, text: &'a [T]) {
        self.build(text)
    }

    /// check if a string query is a substring
    pub fn contains(&self, query: &[T]) -> bool {
        let text = &self.origs;
        let mut x = Some(&self.root);
        let nquery = query.len();
        let mut pos = 0;
        while !x.map_or(true, |n| n.is_leaf()) && pos < nquery {
            x = x.unwrap().child_starts_with(&query[pos]);
            if let Some(ref node) = x {
                let label = node.slice(text);
                let nlabel = label.len();
                if nlabel <= query[pos..].len() {
                    if label == &query[pos.. pos + nlabel] {
                        pos += nlabel;
                    } else {
                        return false;
                    }
                } else {
                    return label.starts_with(&query[pos..]);
                }
            }
        }
        pos == nquery
    }

    // http://stackoverflow.com/questions/9452701/ukkonens-suffix-tree-algorithm-in-plain-english
    // http://pastie.org/5925812
    fn build(&mut self, text: &'a [T]) {
        let from = self.origs.len();
        self.origs.push(text);

        let root_link = Rawlink::some(&mut self.root);
        // let text = self.origs;
        // active point
        let mut active_node = root_link;
        let mut active_edge: usize = 0; //  0 used for null
        let mut active_length = 0;
        // how many to be inserted
        let mut remainder = 0;
        for (pos, &c) in text.iter().enumerate() {
            remainder += 1;
            let mut need_suffix_link: Rawlink<Node<T>> = Rawlink::none();

            while remainder > 0 {
                if active_length == 0 { active_edge = pos }
                if active_node.resolve().map_or(false, |n| n.child_starts_with(&text[active_edge]).is_none()) {
                    println!("crash at 1?");
                    active_node.resolve_mut().map(|n| n.add_child(Node::leaf(pos, vec![from]), text));
                    need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(active_node));
                    need_suffix_link = active_node;
                } else if let Some(ref mut next) = active_node.resolve_mut().unwrap().mut_child_starts_with(&text[active_edge]) {
                    // walk down
                    if active_length >= next.length(pos) {
                        active_edge += next.length(pos);
                        active_length -= next.length(pos);
                        active_node = Rawlink::some(next);
                        continue;
                    }
                    if text[next.start() + active_length] == c {
                        active_length += 1;
                        need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(active_node));
                        break;
                    }
                    next.split_at(active_length, text);
                    println!("crash at 2?");
                    next.add_child(Node::leaf(pos, vec![from]), text);
                    need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(Rawlink::some(next)));
                    need_suffix_link = Rawlink::some(next);
                }
                remainder -= 1;

                if active_node.resolve().unwrap().is_root() && active_length > 0 { // rule 1
                    active_length -= 1;
                    active_edge = pos - remainder + 1;
                } else {
                    // rule 3
                    let link = active_node.resolve().unwrap().suffix_link();
                    if link.is_null() {
                        active_node = root_link;
                    } else {
                        active_node = link;
                    }

                }
            }
        }
    }
}


fn dot_id<T>(x: &T) -> u64 {
    unsafe {
        mem::transmute::<_, u64>(x)
    }
}

impl<'a, T: Ord + Copy + fmt::Display + fmt::Debug> SuffixTree<'a, T> {
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph G {\n");
        dot.push_str("  node [shape=point];\n");
        let mut queue = Queue::new();
        queue.enqueue(&self.root);
        while !queue.is_empty() {
            let x = queue.dequeue().unwrap();
            let pid = dot_id(x);
            if x.is_leaf() {

            } else if x.is_root() || x.is_internal() {
                for node in x.iter_children() {
                    let nid = dot_id(node);
                    dot.push_str(&format!("  {} -> {} [ label = \"{}\"];\n", pid, nid, node.slice(&self.origs).iter().map(|c| c.to_string()).collect::<Vec<String>>().join(" ")));
                    // x.suffix_link().resolve().map(|n| dot.push_str(&format!("  {} -> {} [ style=dashed ];\n", pid, dot_id(n))));
                    if node.is_internal() {
                        queue.enqueue(node);
                    }
                }
            }
        }
        dot.push_str("}\n");
        dot
    }
}

impl<'a, T: fmt::Display + fmt::Debug> fmt::Display for SuffixTree<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SuffixTree(origs: {:?})", self.origs)
    }
}


#[test]
fn test_suffix_tree() {
    let s = "abcabxabcdabcabxabc".chars().collect::<Vec<char>>();
    let s2 = "abasbasbabasbabasbasbas".chars().collect::<Vec<char>>();
    let mut st = SuffixTree::new(&s);

    st.add(&s2);
    println!("==================================================");
    println!("got => {}", st);
    println!("dot =>\n{}", st.to_dot());
}


#[test]
fn test_suffix_tree_contains() {
    let s = b"abcabxabcdaabab";
    let st = SuffixTree::new(s);

    assert!(st.contains(b"abc"));
    assert!(st.contains(b""));
    assert!(st.contains(b"b"));
    assert!(!st.contains(b"y"));
    assert!(st.contains(b"abcabxabcdaabab"));
    assert!( st.contains(b"bxabcdaa"));
    assert!(!st.contains(b"bxabadaa"));
}
