use std::fmt;
use std::mem;
use std::ptr;
use std::iter::FromIterator;

use std::collections::BTreeMap;

use vec_map::VecMap;

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

    fn take(&mut self) -> Rawlink<T> {
        mem::replace(self, Rawlink::none())
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
enum Node<'a, T: 'a> {
    Leaf {
        /// the edge label
        data: &'a [T],
        /// text start_pos of every text at this node: { text index in root: start_pos }
        starts: VecMap<usize>,
        /// text terminates at this node, suffix offset: { text index in root: suffix offset}
        terminates: VecMap<usize>
    },
    Internal {
        data: &'a [T],
        starts: VecMap<usize>,
        terminates: VecMap<usize>,
        children: BTreeMap<T, Node<'a, T>>,
        suffix_link: Rawlink<Node<'a, T>>
    },
    Root { children: BTreeMap<T, Node<'a, T>> }
}

impl<'a, T: Ord + Copy + fmt::Debug> Node<'a, T> {
    pub fn root() -> Node<'a, T> {
        Root { children: BTreeMap::new() }
    }

    pub fn leaf(data: &'a [T], txt_idx: usize, rank: usize, start_pos: usize) -> Node<'a, T> {
        Leaf {
            data: data,
            starts: VecMap::from_iter(vec![(txt_idx, start_pos)]),
            terminates: VecMap::from_iter(vec![(txt_idx, rank)])
        }
    }

    pub fn internal(data: &'a [T], txt_idx: usize, start_pos: usize) -> Node<T> {
        Internal {
            data: data,
            terminates: VecMap::new(),
            starts: VecMap::from_iter(vec![(txt_idx, start_pos)]),
            children: BTreeMap::new(),
            suffix_link: Rawlink::none()
        }
    }

    pub fn add_child(&mut self, x: Node<'a, T>) {
        match *self {
            Root { ref mut children } => {
                children.insert(x.head(), x);
            },
            Internal { ref mut children, .. } => {
                children.insert(x.head(), x);
            },
            Leaf { .. } => panic!("leaf can't have a child")
        }
    }

    fn shrink(&mut self, offset: usize) {
        match *self {
            Leaf { ref mut data, .. } => {
                *data = &data[offset..]
            },
            Internal { ref mut data, .. } => {
                *data = &data[offset..]
            },
            Root { .. } => panic!("can't shrink root node")
        }
    }

    fn truncated_internal(&mut self, txt_idx: usize, offset: usize) -> Node<'a, T> {
        assert!(offset < self.data().len());
        match *self {
            Leaf { ref data, ref mut starts, .. } => {
                let new_starts = starts.clone();
                for (_key, value) in starts.iter_mut() {
                    *value += offset;
                }
                Internal {
                    data: &data[..offset],
                    starts: new_starts,
                    terminates: VecMap::new(),
                    children: BTreeMap::new(),
                    suffix_link: Rawlink::none()
                }
            },
            Internal { ref data, ref mut starts, ref terminates, ref mut suffix_link, .. } => {
                let new_starts = starts.clone();
                let new_suffix_link = suffix_link;
                for (_key, value) in starts.iter_mut() {
                    *value += offset;
                }
                Internal {
                    data: &data[..offset],
                    starts: new_starts,
                    terminates: VecMap::new(),
                    children:: BTreeMap::new(),
                    suffix_link: suffix_link.take()
                }
            }
        }
    }

    pub fn split_at(&mut self, offset: usize) {
        let new = Node::internal(&self.data()[0..offset]);
        let mut old = mem::replace(self, new);
        if let Internal { ref mut suffix_link, .. } = *self {
            *suffix_link = old.suffix_link();
        }
        if let Internal { ref mut suffix_link, .. } = old {
            *suffix_link = Rawlink::none();
        }
        old.shrink(offset);
        self.add_child(old);
    }

    #[inline]
    pub fn data(&self) -> &'a [T] {
        match *self {
            Internal { data, .. } => data,
            Leaf { data, .. } => data,
            _ => panic!("root hava no data label")
        }
    }

    pub fn head(&self) -> T {
        match *self {
            Internal { data, .. } => data[0],
            Leaf { data, .. } => data[0],
            _ => panic!("root have no head")
        }
    }

    pub fn iter_children<'t>(&'t self) -> ::std::collections::btree_map::Values<'t, T, Node<'a, T>> {
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
            Leaf { ref data, .. } => data.len(),
            Internal { ref data, .. } => data.len(),
            Root { .. } => 0,
        }
    }

    fn add_suffix_link(&mut self, slink: Rawlink<Node<'a, T>>) {
        match *self {
            Internal { ref mut suffix_link, .. } => {
                *suffix_link = slink;
            }
            _ => {}
        }
    }

    fn suffix_link(&self) -> Rawlink<Node<'a, T>> {
        match *self {
            Internal { suffix_link, .. } => {
                suffix_link.clone()
            }
            _ => {
                Rawlink::none()
            }
        }
    }

    pub fn mut_child_starts_with<'t>(&'t mut self, c: &T) -> Option<&'t mut Node<'a, T>> {
        match *self {
            Root { ref mut children } => children.get_mut(c),
            Internal { ref mut children, .. } => children.get_mut(c),
            Leaf { .. } => panic!("leaf have no children")
        }
    }
    pub fn child_starts_with(&self, c: &T) -> Option<&Node<'a, T>> {
        match *self {
            Root { ref children } => children.get(c),
            Internal { ref children, .. } => children.get(c),
            Leaf { .. } => None
        }
    }
}

#[derive(Debug)]
pub struct SuffixTree<'a, T: Sized + 'a> {
    txts: Vec<&'a [T]>,
    root: Node<'a, T>
}

impl<'a, T: Ord + Copy + fmt::Debug> SuffixTree<'a, T> {
    pub fn new(txt: &'a [T]) -> SuffixTree<'a, T> {
        let mut st = SuffixTree {
            txts: vec![],
            root: Node::root()
        };
        st.add(txt);
        st
    }

    /// check if a string query is a substring
    // pub fn contains(&self, query: &[T]) -> bool {
    //     let text = self.txts;
    //     let mut x = Some(&self.root);
    //     let nquery = query.len();
    //     let mut pos = 0;
    //     while !x.map_or(true, |n| n.is_leaf()) && pos < nquery {
    //         x = x.unwrap().child_starts_with(&query[pos]);
    //         if let Some(ref node) = x {
    //             let label = node.slice();
    //             let nlabel = label.len();
    //             if nlabel <= query[pos..].len() {
    //                 if label == &query[pos.. pos + nlabel] {
    //                     pos += nlabel;
    //                 } else {
    //                     return false;
    //                 }
    //             } else {
    //                 return label.starts_with(&query[pos..]);
    //             }
    //         }
    //     }
    //     pos == nquery
    // }

    pub fn add(&mut self, txt: &'a [T]) {
        self.ukkonen95(txt)
    }

    // http://stackoverflow.com/questions/9452701/ukkonens-suffix-tree-algorithm-in-plain-english
    // http://pastie.org/5925812
    // Ukkonen (1995)
    fn ukkonen95(&mut self, txt: &'a [T]) {
        let root_link = Rawlink::some(&mut self.root);
        let txt_idx = self.txts.len();
        // active point
        let mut active_node = root_link;
        let mut active_edge: usize = 0; //  0 used for null
        let mut active_length = 0;
        // how many to be inserted
        let mut remainder = 0;
        for (pos, &c) in txt.iter().enumerate() {
            remainder += 1;
            let mut need_suffix_link: Rawlink<Node<T>> = Rawlink::none();

            while remainder > 0 {
                if active_length == 0 { active_edge = pos }
                if active_node.resolve().map_or(false, |n| n.child_starts_with(&txt[active_edge]).is_none()) {
                    active_node.resolve_mut().map(|n| n.add_child(Node::leaf(&txt[pos..], txt_idx, pos)));
                    need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(active_node));
                    need_suffix_link = active_node;
                } else if let Some(ref mut next) = active_node.resolve_mut().unwrap().mut_child_starts_with(&txt[active_edge]) {
                    // walk down
                    if active_length >= next.length(pos) {
                        active_edge += next.length(pos);
                        active_length -= next.length(pos);
                        active_node = Rawlink::some(next);
                        continue;
                    }
                    if next.data()[active_length] == c {
                        active_length += 1;
                        need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(active_node));
                        break;
                    }
                    next.split_at(active_length);
                    next.add_child(Node::leaf(&txt[pos..], txt_idx, pos));
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
                    dot.push_str(&format!("  {} -> {} [ label = \"{:?}\"];\n", pid, nid, node.data()));
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
        write!(f, "SuffixTree(txts: {:?})", self.txts)
    }
}


#[test]
fn test_suffix_tree() {
    let s = "abcabxac".chars().collect::<Vec<char>>();
    let st = SuffixTree::new(&s);
    println!("==================================================");
    println!("got => {}", st);
    println!("dot =>\n{}", st.to_dot());
}


// #[test]
// fn test_suffix_tree_contains() {
//     let s = b"abcabxabcdaabab";
//     let st = SuffixTree::new(s);

//     assert!(st.contains(b"abc"));
//     assert!(st.contains(b""));
//     assert!(st.contains(b"b"));
//     assert!(!st.contains(b"y"));
//     assert!(st.contains(b"abcabxabcdaabab"));
//     assert!( st.contains(b"bxabcdaa"));
//     assert!(!st.contains(b"bxabadaa"));
// }
