use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering::Relaxed;
use serde::Serialize;
use serde::Deserialize;
use log::trace;
#[derive(Debug,  Serialize, Deserialize)]
pub struct Trie<V> {
    children: HashMap<char, Node<V>>,
    #[serde(default)]
    node_count: std::sync::atomic::AtomicU32, // TODO: this didn't need to be atomic - had mutability issues to contend with
    #[serde(default)]
    char_count: std::sync::atomic::AtomicU32,
}

impl<V> Trie<V> {
    pub fn new() -> Trie<V> {
        Trie {
            children: Default::default(),
            node_count: Default::default(),
            char_count: Default::default()
        }
    }
    pub fn insert(&mut self, text: &str,
                  optional_associated_value: Option<V>) {
        if text.is_empty() {
            return
        }
        let c = text.chars().next().unwrap();
        if let Some(child) = self.children.get_mut(&c) {
            child.insert(text, optional_associated_value, &self.node_count, &self.char_count) ;
        } else {
            self.node_count.fetch_add(1, Relaxed);
            self.char_count.fetch_add(text.len() as u32, Relaxed);
            self.children.insert(c, Node {
                text: text.to_string(),
                terminal: true,
                children: Default::default(),
                value: optional_associated_value,
                visit_count: Default::default(),
                #[cfg(feature = "tracing")]
                node_id: gen_id(),
                weight: text.len()
            });
        }
    }
    /// removes the text from the trie and compresses nodes along the way
    pub fn remove(&mut self, text: &str) {
        let first = text.chars().next().unwrap();
        if let Some(first) = self.children.get_mut(&first) {
            first.remove(text, &self.node_count, &self.char_count);
        }
    }
    /// returns the suffix tree root for a given prefix
    pub fn suffix_tree(&self, prefix: &str) -> Option<&Node<V>> {
        if prefix.is_empty() {
            return None
        }
        let first = prefix.chars().next().unwrap();
        if let Some(child) = self.children.get(&first) {
            return child.suffix_root(prefix)
        }
        None
    }
    /// returns the suffix tree with the given matching options
    pub fn suffix_tree_with_matching_options(&self, prefix: &str, options: &MatchingOptions) -> Option<&Node<V>> {
        let first = prefix.chars().next().unwrap();
        if let Some(child) = self.children.get(&first) {
            let tagged  = options.tag(prefix);
            return child.suffix_tree_with_options(tagged.chars.as_slice(), options);
        }
        None
    }
    pub fn get_string_suffixes(&self, prefix: &str) -> HashSet<String> {
        let mut coll = Vec::new();
        let mut emit = HashSet::new();
        self.suffix_tree(prefix).map(|t| {
            t.get_string_suffixes(true, prefix, &mut coll, &mut emit)
        });
        emit
    }

    pub fn get_suffixes_values(&self, prefix: &str) -> Option<Vec<Entry<V>>> {
        let mut coll = Vec::new();
        self.suffix_tree(prefix).map(|t| {
            t.get_suffixes(true, prefix,  &mut coll)
        })
    }

    pub fn get_suffixes_with_matching_options(&self, prefix: &str, options: &MatchingOptions) -> Option<Vec<Entry<V>>> {
        let mut coll = Vec::new();
        self.suffix_tree_with_matching_options(prefix, options).map(|t| {
            t.get_suffixes(true, prefix,  &mut coll)
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Node<V> {
    text: String,
    terminal: bool,
    children: HashMap<char, Node<V>>,
    value: Option<V>,
    // for pruning purposes
    #[serde(default)]
    visit_count: std::sync::atomic::AtomicU64, // TODO: this didn't need to be atomic
    #[cfg(feature = "tracing")]
    #[serde(default = "gen_id")]
    node_id: Option<u8>, // TODO: u8 might not be enough ? consider bigger
    #[serde(default)]
    weight: usize,
}

#[cfg(feature = "tracing")]
fn gen_id() -> Option<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Some(rng.gen())
}

impl<V> Debug for Node<V> where V: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("text", &self.text)
            .field("terminal", &self.terminal)
            .field("value", &self.value).field("children", &self.children)
            .field("nested_chars", &self.weight)
            .finish()
    }
}

impl<V> Node<V> {
    pub fn new(string: &str, is_terminal: bool, value: Option<V>) -> Self {
        Node {
            text: string.to_string(),
            terminal: is_terminal,
            children: Default::default(),
            value,
            visit_count: Default::default(),
            #[cfg(feature = "tracing")]
            node_id: gen_id(),
            weight: string.len()
        }
    }

    pub fn remove(&mut self, text: &str,
                  node_count: &std::sync::atomic::AtomicU32,
                  char_count: &std::sync::atomic::AtomicU32) {
        let mut position = 0;
        let mut my_iter = self.text.chars();
        let mut other_iter = text.chars();
        loop {
            let my_char = my_iter.next();
            let other_char = other_iter.next();
            let remaining = grapheme_slicer_until_end(text, position);
            match (my_char, other_char) {
                // this = abcd
                // other = abce
                (Some(x), Some(y)) if x != y => {
                    // stop
                    return
                },
                (Some(x), Some(y)) if x == y => {
                    // more to come
                },
                (Some(_), None) => {
                    return
                },
                (None, Some(y)) => {
                    if let Some(child) = self.children.get_mut(&y) {
                        child.remove(remaining.as_str(), node_count, char_count);
                        if child.children.len() == 0 {
                            node_count.fetch_sub(1, Relaxed);
                            char_count.fetch_sub(child.text.len() as u32, Relaxed);
                            self.weight -= child.weight;
                            if !child.terminal {
                                self.children.remove(&y); // removing dangling child
                            }
                        }
                        break;
                    }
                    // this prefix never existed in this tree
                    return
                },
                (None, None) => {
                    // this case we need to remove this node possibly
                    if self.children.len() == 1 {
                        break
                    }
                    // make zombie to be removed
                    self.terminal = false;
                    return

                },
                _ => {}
            }
            position += 1;
        }
        if self.children.len() == 1 {
            // merge this with the child
            let (_, child) = self.children.drain().next().unwrap();
            node_count.fetch_sub(1, Relaxed);
            // same text which is merged back
            self.text = format!("{}{}", self.text, child.text);
            self.value = child.value;
            self.terminal = child.terminal;
            self.children = child.children;
            return
        }
    }

    pub fn char_weight_of_children(&self) -> usize {
        self.children
            .iter()
            .fold(0, |x, (_,y) | x + y.weight)
    }

    pub fn insert(&mut self, text: &str,
                  value: Option<V>,
                  node_count: &std::sync::atomic::AtomicU32,
                  char_count: &std::sync::atomic::AtomicU32) {
        let mut position = 0;
        let mut child_iter = text.chars();
        let mut this_iter = self.text.chars();
        loop {
            let text_next_char = child_iter.next();
            let this_next_char = this_iter.next();

            match (text_next_char, this_next_char) {
                (Some(x), Some(y)) if x == y => {
                    // continue
                },
                (Some(input_next), Some(this_next)) if input_next != this_next => {
                    // split
                    let current_child_weight = self.char_weight_of_children();
                    let existing_remainder = grapheme_slicer_until_end(self.text.as_str(), position);
                    // let existing_remainder = self.text[position..].to_string();

                    let mut new_node = Node {
                        text: existing_remainder.clone(),
                        terminal: self.terminal, // if I was terminal, then suffice to say my splitted up self is also terminal
                        children: Default::default(),
                        value: None,
                        visit_count: std::sync::atomic::AtomicU64::new(self.visit_count()),
                        #[cfg(feature = "std")]
                        node_id: gen_id(),
                        weight: current_child_weight + existing_remainder.len()
                    };
                    // exhange my children for the new node (I am empty and will add a new node back)
                    std::mem::swap(&mut new_node.children, &mut self.children);
                    std::mem::swap(&mut new_node.value, &mut self.value);
                    let first_char_of_existing_remainder = existing_remainder.chars().next().unwrap();
                    // new node was created but same num chars which was split between 2 nodes
                    node_count.fetch_add(1, Relaxed);
                    self.children.insert(first_char_of_existing_remainder, new_node);


                    let common = grapheme_slicer_until_point(text, position);

                    let remainder = grapheme_slicer_until_end(text, position);
                    // let common = &text[0..position];
                    // let remainder = &text[position..];

                    let input_new_node = Node {
                        text: remainder.to_string(),
                        terminal: true,
                        children: Default::default(),
                        value,
                        visit_count: std::sync::atomic::AtomicU64::new(self.visit_count()),
                        #[cfg(feature = "tracing")]
                        node_id: gen_id(),
                        weight: remainder.len()
                    };

                    let c = remainder.chars().next().unwrap();
                    // new node added
                    node_count.fetch_add(1, Relaxed);
                    // remainder chars added to tree
                    char_count.fetch_add(remainder.len() as u32, Relaxed);
                    self.children.insert(c, input_new_node);
                    // let delta_weight = self.text.len() - common.len();
                    self.text = common.to_string();
                    self.terminal = false;
                    self.weight = self.text.len() + self.char_weight_of_children();
                    // self.weight += (delta_weight + remainder.len());
                    return;



                }
                (None, Some(c)) => {
                    // let prefix = self.text[0..position].to_string();
                    // let remainder = self.text[position..].to_string();

                    let prefix = grapheme_slicer_until_point(self.text.as_str(), position);

                    let remainder = grapheme_slicer_until_end(self.text.as_str(), position);
                    self.text = prefix.to_string();
                    self.terminal = true;
                    // self.nested_chars += remainder.len();
                    let current_child_weight = self.char_weight_of_children();
                    if let Some(child) = self.children.get_mut(&c) {
                        let taken = std::mem::take(&mut self.value);
                        self.value = value;

                        child.insert(remainder.as_str(), taken, node_count, char_count);
                        let new_char_weight = self.char_weight_of_children();
                        let delta = new_char_weight - current_child_weight;
                        self.weight += delta;
                        return
                    }
                    let new_node = Node {
                        text: remainder.to_string(),
                        terminal: true,
                        children: Default::default(),
                        value,
                        visit_count: std::sync::atomic::AtomicU64::new(self.visit_count()),
                        #[cfg(feature = "tracing")]
                        node_id: gen_id(),
                        weight: remainder.len()
                    };
                    node_count.fetch_add(1, Relaxed);
                    char_count.fetch_add(remainder.len() as u32, Relaxed);
                    self.children.insert(c, new_node);
                    self.terminal = true;
                    return;
                },
                (Some(text_next), None) => {
                    let remainder = grapheme_slicer_until_end(text, position);
                    // let remainder = &text[position..];

                    // self.nested_chars += remainder.len();
                    let current_child_weight = self.char_weight_of_children();
                    if let Some(next) = self.children.get_mut(&text_next) {

                        next.insert(remainder.as_str(), value, node_count, char_count);
                        let new_weight_of_children = self.char_weight_of_children();
                        let delta = new_weight_of_children - current_child_weight;
                        self.weight += delta;
                        return
                    }
                    // make new child
                    let new_node = Node {
                        text: remainder.to_string(),
                        terminal: true,
                        children: Default::default(),
                        value,
                        visit_count: std::sync::atomic::AtomicU64::new(self.visit_count()),
                        #[cfg(feature = "tracing")]
                        node_id: gen_id(),
                        weight: remainder.len()
                    };
                    self.weight += remainder.len();
                    node_count.fetch_add(1, Relaxed);
                    char_count.fetch_add(remainder.len() as u32, Relaxed);
                    self.children.insert(text_next, new_node);
                    return;

                }
                (None, None) => {
                    self.terminal = true;
                    if self.value.is_none() {
                        self.value = value;
                    }
                    return;
                }
                _ => {panic!("Should never be here {} {}", self.text, text )} // compiler yells that it wants this case but I don't see how it could occur
            }
            position += 1;



        }
    }

    fn match_on_treated_suffix_trees(&self, prefix: &[(Tagged, Offset)], options: &MatchingOptions) -> Vec<&Node<V>> {
        if prefix.len() == 0 {
            return vec![];
        }
        // i don't think this is necessarily correct - example if you have multiple whitespaces
        let tagged_char = &prefix.first().unwrap().0.char();

        let mut v = self.children.iter().filter(|(key, _node)| {
            key == tagged_char ||
            options.treatments.contains_key(key)
        } ).map(|(_, n)| {
            n.suffix_tree_with_options(prefix, options)
        }).flatten().collect::<Vec<_>>();
        v.sort_by(|x,y| y.weight.cmp(&x.weight));
        v
    }

    fn suffix_tree_with_options(&self, prefix: &[(Tagged, Offset)], options: &MatchingOptions) -> Option<&Node<V>> {
        // update visit count


        self.visit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let self_tagged = options.tag(self.text.as_str());
        #[cfg(feature = "tracing")]
        if true {
            trace!("this: {:?} {:?}", self.text, self.node_id);
            trace!("self_tagged: {:?}", self_tagged);
            trace!("prefix: {:?}", prefix);
            trace!("children {:?}", self.children.keys());
        }

        let this_iter = self_tagged.chars.iter();
        let taken = prefix.iter().zip(this_iter).take_while(|((x, _),(y, _))| {x == y}).collect::<Vec<_>>().len();
        // trace!("taken: {}", taken);
        if taken < prefix.len() {
            // trace!("prefix token after: {:?}", &prefix[taken]);
        }
        if taken < self_tagged.chars.len() {
            // trace!("this token after {:?}", &self_tagged.chars[taken]);
        }
        match taken {
            x if x < prefix.len() && x < self_tagged.chars.len() => {
                // we have a mismatch at some position and need to attempt to branch
                let (_,offset) = self_tagged.chars.get(x).unwrap();
                let continuation = grapheme_slicer_until_end(self.text.as_str(), *offset);
                // let continuation = &self.text.as_str()[*offset..];
                let child_ = continuation.chars().next().unwrap();
                let new_prefix = &prefix[x..];
                let best_attempt = self.match_on_treated_suffix_trees(new_prefix, options).into_iter().next();
                if let Some(child) = self.children.get(&child_) {
                    let c = child.suffix_tree_with_options(new_prefix, options);
                    match (best_attempt, c) {
                        (Some(best_attempt), Some(c)) if best_attempt.weight > c.weight => return Some(best_attempt),
                        (Some(best_attempt), _) => return Some(best_attempt),
                        (_, Some(c)) => return Some(c),
                        _ => {}
                    }
                    // if let Some(res) = child.suffix_tree_with_options(new_prefix, options) {
                    //     return Some(res)
                    // }
                    // return child.suffix_tree_with_options(new_prefix, options)
                }
                return best_attempt;
                // return None
            }
            x if x == prefix.len() && x <= self_tagged.chars.len() => {
                return Some(self) // this is the terminating node
            }
            x if x == self_tagged.chars.len() && x < prefix.len() => {
                let (last,_o) = prefix.get(x).unwrap();
                match last {
                    Tagged::Char(c) | Tagged::Sentinel(_, c) => {
                        // all this since choosing a single path
                        // perhaps the better solution is to return both options
                        // TODO: define the proper strategy here
                        let new_prefix = &prefix[x..];
                        let mut treated = self.match_on_treated_suffix_trees(new_prefix, options);
                        treated.sort_by(|x,y| {
                            y.weight.partial_cmp(&x.weight).unwrap()
                        });
                        let best = treated.into_iter().next();

                        if let Some(child) = self.children.get(c) {
                            let c = child.suffix_tree_with_options(new_prefix, options);
                            match (best,c) {
                                (Some(best), Some(c)) if best.weight > c.weight => {
                                    trace!("choose fuzzy over exact match");
                                    return Some(best)}
                                (Some(best), _) => {
                                    trace!("choose fuzzy (no exact match)");
                                    return Some(best)
                                },
                                (_, Some(c)) => return Some(c),
                                _ => {}
                            }

                        }
                        return best
                    }

                }

            }
            _ => {
                // should not get here
                return None
            }
        }
    }

    /// returns the suffix tree for the given prefix
    /// if you want to include partial results in the case that the node text contains the prefix text but possibly longer, then include_partial should be set to true
    /// example: Node: abcdef, prefix: abc
    /// include_partial: false => None
    /// include_partial: true => Some(self) // overlaps on abc
    pub fn suffix_root(&self, prefix: &str) -> Option<&Node<V>> {
        // update visit count
        self.visit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut position = 0;
        let mut this_iter = self.text.chars();
        let mut other_iter = prefix.chars();
        loop {
            let this_next_char = this_iter.next();
            let other_next_char = other_iter.next();
            match (this_next_char, other_next_char) {
                (Some(x), Some(y)) => {
                    if x != y {
                        // return nothing
                        if !x.is_whitespace() && !y.is_whitespace() {
                            return None
                        }

                    }
                },
                (None, Some(x)) => {
                    if let Some(next_child) = self.children.get(&x) {
                        return next_child.suffix_root(grapheme_slicer_until_end(prefix, position).as_str())
                    }
                    return None
                }
                (Some(x), None) => {
                    if let Some(child) = self.children.get(&x) {
                        return child.suffix_root(prefix)
                    }
                    return Some(self)
                }
                (None, None) => {
                    return Some(self);
                }
            }
            position += 1;
        }
    }

    /// returns all the suffixes below this node
    /// notice that we pass the prefix as an input parameter because there may be some overlap between the node contents and the prefix which needs to be stripped away
    /// for example a tree like "ab" -> "cde"
    /// for a given input "abc" we would be lead to this node
    /// so we want to return results like "abcde" and not "abccde" (notice the "c" appears twice if blindly appending)
    /// notice that there is an edge case here which is not yet handled where the match might have been fuzzy with options and that overlap is not handled correctly
    /// for instance if this node text ends with white space
    fn get_suffixes<'a>(&'a self, is_root: bool, prefix: &str, collector: &mut Vec<String>) -> Vec<Entry<'a, V>> {
        // update visit count
        self.visit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        //Entry
        if !is_root {
            collector.push(self.text.clone());
        } else {
            if let Some(pos) = find_common_overlap_of_prefix_with_node(prefix, self.text.as_str()) {
                if pos < self.text.len() {
                    collector.push(grapheme_slicer_until_end(self.text.as_str(), pos));
                }
            } else {
                collector.push(self.text.clone());
            }
        }

        let mut v: Vec<Entry<'a ,V>> = Vec::<Entry<V>>::new();
        if self.terminal {
            let jo = collector.join("");
            let entry: Entry<'a, V> = Entry {
                key: jo,
                val: &self.value
            };
            v.push(entry);
        }
        let mut children = self.children.values();
        while let Some(child) = children.next() {
            let mut add = child.get_suffixes(false, prefix, collector);
            v.append(&mut add);
        }
        collector.pop();
        v
    }

    pub fn get_string_suffixes(&self, is_root: bool, prefix: &str, collector: &mut Vec<String>, emit: &mut HashSet<String>) {
        // update visit count
        self.visit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if !is_root {
            collector.push(self.text.clone());
        } else {
            if let Some(pos) = find_common_overlap_of_prefix_with_node(prefix, self.text.as_str()) {
                if pos < self.text.len() {
                    collector.push(grapheme_slicer_until_end(self.text.as_str(), pos));
                }
            }

        }

        if self.terminal {
            let jo = collector.join("");
            emit.insert(jo);
        }
        let mut children = self.children.values();
        while let Some(child) = children.next() {
            child.get_string_suffixes(false, prefix, collector, emit);
        }
        collector.pop();
    }

    fn visit_count(&self) -> u64 {
        self.visit_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct Entry<'a, V> {
    pub key: String,
    pub val: &'a Option<V>
}

pub enum CharacterSet {
    /// spaces and tabs
    WhiteSpaces,
    /// just \n
    NewLines,
    /// spaces and tabs and \b
    WhiteSpacesAndNewLines,
    /// Capitalized are treated like lower cased
    CapitalizedLetters,

    Char(HashSet<char>)

}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum NormalizedChar {
    // this char can be squashed since it i
    Squash,
    Char(char),
    // sentinal hash value for this equivalnce set
    Sentinal(u64,char),
}

#[derive(Clone,Debug)]
enum Tagged {
    Char(char),
    /// for use when replacing sets of characters with a common code
    /// i.e consider a,b,c,d,e,f as identical characters - will be a Sentinel(hash, real char)
    Sentinel(u64, char)
}

impl Tagged {
    fn char(&self) -> &char {
        match self {
            Tagged::Char(x) => {x}
            Tagged::Sentinel(_, x) => {x}
        }
    }
}

impl PartialEq for Tagged {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (Tagged::Char(c1), Tagged::Char(c2)) => c1 == c2,
            (Tagged::Sentinel(h1, _),Tagged::Sentinel(h2, _) ) => h1 == h2,
            _ => false
        }
    }
}


/// the originating offset in that string
type Offset = usize;
/// the idea is that for a given string you map to the offset for a trimmed character set
#[derive(Clone,Debug)]
struct TaggedString {
    chars: Vec<(Tagged, Offset)>
}

impl TaggedString {
    /// tags this string
    /// example: "abc" => < (a,0), (b,1), (b,2) >
    #[allow(dead_code)]
    fn new(str: &str) -> Self {
        Self {
            chars: str.chars().enumerate().map(|(offset, char)| {
                (Tagged::Char(char), offset)
            }).collect()
        }
    }
}


/// tags the strings with the offsets
impl From<Vec<NormalizedChar>> for TaggedString {
    fn from(chars: Vec<NormalizedChar>) -> Self {
        let tagged  = chars.into_iter().enumerate().flat_map(|(offset,y)| {
            match y {
                NormalizedChar::Squash => {None}
                NormalizedChar::Char(x) => {Some((Tagged::Char(x), offset))}
                NormalizedChar::Sentinal(hash, char) => {Some((Tagged::Sentinel(hash,char), offset))}
            }
        }).collect();
        Self {
            chars: tagged
        }
    }
}

impl CharacterSet {
    pub fn normalized_char(&self, char: char) -> NormalizedChar {
        match self {
            CharacterSet::WhiteSpaces if char.is_whitespace() => {
                NormalizedChar::Squash
            }
            CharacterSet::NewLines if char == '\n' => { NormalizedChar::Squash}
            CharacterSet::WhiteSpacesAndNewLines if char == '\n' || char == ' '=> { NormalizedChar::Squash}
            CharacterSet::CapitalizedLetters => { NormalizedChar::Char(char.to_uppercase().next().unwrap_or(char))}
            CharacterSet::Char(x) if x.contains(&char)=> {
                let mut v = x.iter().collect::<Vec<_>>();
                v.sort(); // not sure if hash is dependant on ordering
                let mut s = DefaultHasher::new();

                v.hash(&mut s);
                NormalizedChar::Sentinal(s.finish(), char)
            }
            _ => NormalizedChar::Char(char)
        }
    }
}

fn encode(str: &str, treatments: &HashMap<char, CharacterSet>) -> Vec<NormalizedChar> {
    str.chars().map(|c| {
            treatments.get(&c).map(|t| t.normalized_char(c)).unwrap_or_else(|| NormalizedChar::Char(c))
        }
    ).collect::<Vec<_>>()
}

/// since slicing a string by bytes will not work when you have long unicode characters (wide grapheme cluster?)
/// take chars and allocate a new string (unless you have a way to slice it by cluster???).
/// TODO: figure out how to return a slice instead of new string
fn grapheme_slicer_until_end(str: &str, from:usize) -> String {
    // figure how to slice char boundary and not allocate a new string
    let temp = str.chars().collect::<Vec<_>>();
    let x = &temp.as_slice()[from..temp.len()];
    x.iter().collect()
}

fn grapheme_slicer_until_point(str: &str, until: usize) -> String {

    let temp = str.chars().collect::<Vec<_>>();
    let x = &temp.as_slice()[0..until];
    x.iter().collect()
}

#[test]
fn test_shitty_slicer() {
    let text = "🤡abcded🤡";
    let pref = grapheme_slicer_until_point(text, 4);
    assert_eq!(pref, "🤡abc");
}
/// describes matching options
/// you supply a mapping of characters to the character set to match against
/// for example * matches against all characters
pub struct MatchingOptions {
    treatments: HashMap<char, CharacterSet>, // TODO: need to check if "char" supports emoji and other wide characters
}

impl MatchingOptions {
    /// exact match only
    pub fn exact() -> Self {
        Self {
            treatments: Default::default()
        }
    }
    /// match but accept white space differences
    pub fn ignoring_white_space() -> Self {
        let mut treatments = HashMap::new();
        treatments.insert(' ', CharacterSet::WhiteSpaces);
        treatments.insert('\t', CharacterSet::WhiteSpaces);
        Self {
            treatments
        }
    }

    /// match but accept diff in new lines
    pub fn ignoring_new_lines() -> Self {
        let mut treatments = HashMap::new();
        treatments.insert('\n', CharacterSet::WhiteSpaces);
        Self {
            treatments
        }
    }
    /// match but accept new lines and whitespace differences
    pub fn ignoring_white_space_and_new_lines() -> Self {
        let mut treatments = HashMap::new();
        treatments.insert(' ', CharacterSet::WhiteSpaces);
        treatments.insert('\t', CharacterSet::WhiteSpaces);
        treatments.insert('\n', CharacterSet::WhiteSpaces);
        Self {
            treatments
        }
    }

    fn tag(&self, str: &str) -> TaggedString {
        let encoded = encode(str, &self.treatments);
        TaggedString::from(encoded)
    }
}

fn find_common_overlap_of_prefix_with_node(prefix: &str, node: &str) -> Option<usize>{
    // node - omeabcde
    // prefix rome
    // result o
    for offset in (0..prefix.len()).rev() {
        let str = grapheme_slicer_until_end(&prefix, offset);
        if node.starts_with(str.as_str()) {
            return Some(str.len())
        }
    }
    None

}


#[test]
fn test_squashing() {
    let node_text = "   abc    def       iop \t\t\t qwe   ";
    let input = "abc def iop     qwe";

    let internal = MatchingOptions::ignoring_white_space();
    let nt1 = internal.tag(node_text);
    let nt2 = internal.tag(input);

    println!("{:?}", nt1);
    println!("{:?}", nt2);
}

#[test]
fn test_empty() {
    let mut trie: Trie<String> = Trie::new();
    trie.insert("", None);
    let results = trie.get_suffixes_values("");
    assert!(results.is_none());
    trie.insert("romanus", None);
    let results = trie.get_suffixes_values("");
    assert!(results.is_none());
    trie.insert("romulus", None);
    let results = trie.get_suffixes_values("");
    assert!(results.is_none());
    trie.insert("rubens", None);
    let results = trie.get_suffixes_values("");
    assert!(results.is_none());
    trie.insert("ruber", None);
    let results = trie.get_suffixes_values("");
    assert!(results.is_none());
    trie.insert("rubicon", None);
    let results = trie.get_suffixes_values("");
    assert!(results.is_none());
    trie.insert("rubicundus", None);
    let results = trie.get_suffixes_values("");
    assert!(results.is_none());
}
