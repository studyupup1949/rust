mod block;
mod quota;

use crate::syntax::*;
use block::{AclBlock, AclHeader, AclRule};
use quota::*;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::{Entry, Keys, Values};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str;
use std::str::FromStr;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_DATA_GROUPS: usize = 256;
const MAX_DATA_BLOCKS: usize = 256;

/// Acl is the data holder and the controller for CaitSith's policy.
/// It holds policy parameters in `data` field within AclData type.
///
///

pub struct Acl {
    pattern: Matcher,
    pub data: AclData,
    debug: bool,
}

impl FromStr for Acl {
    type Err = String;
    fn from_str(feed: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut acl = Acl::new();
        let mut lineno: u16 = 1;
        // referencial instance for HashMap key (on acl.data.table)
        let mut header_ref = AclHeader::new();

        for line in String::from(feed).split('\n') {
            match acl.pattern.parse_category(&line) {
                Category::Version => acl.data.version = acl.pattern.parse_version(&line),
                Category::Quota => {}
                Category::Stat => {}
                Category::NumberGroupDef => acl.number_group_add(&line),
                Category::IpGroupDef => acl.ip_group_add(&line),
                Category::StringGroupDef => acl.string_group_add(line),
                Category::Header => header_ref = acl.header_add(line),
                Category::Audit => acl.audit_add(&header_ref, line),
                Category::Rule => acl.rule_add(&header_ref, line),
                Category::Blank | Category::Comment => {}
                _ => {
                    eprintln!("unknown syntax: line {}: other {}", lineno, line);
                    return Err(format!("unknown syntax: line {}: other {}", lineno, line));
                }
            }
            lineno += 1;
        }
        if acl.debug {
            println!("ip_group {:?}", acl.data.ip_group);
            println!("num_group {:?}", acl.data.number_group);
            println!("string_group {:?}", acl.data.string_group);
            println!("acl list {:?}", acl.dump_table());
        }
        Ok(acl)
    }
}

impl Clone for Acl {
    fn clone(&self) -> Acl {
        Acl {
            pattern: Matcher::new(),
            data: self.data.clone(),
            debug: false,
        }
    }
}

impl Drop for Acl {
    fn drop(&mut self) {
        self.clear();
    }
}

///
///
impl Acl {
    pub fn new() -> Self {
        Acl {
            pattern: Matcher::new(),
            data: AclData::new(),
            debug: false,
        }
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn debug(&mut self) {
        self.debug = true;
    }

    pub fn from(v: Vec<&AclBlock>) -> Acl {
        let mut acl = Acl::new();
        for i in 0..v.len() {
            acl.raw_header_add(&v[i].header);
            for r in v[i].rule.clone() {
                acl.raw_rule_add(&v[i].header, r);
            }
        }
        acl
    }

    /// parse parses policy from a file reader and assign the result into the instance.
    ///
    pub fn parse(&mut self, reader: &mut BufReader<File>) -> Result<&mut Self, String> {
        let mut sbuf = vec![];
        match reader.read_until(0, &mut sbuf) {
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }

        let result = Self::from_str(str::from_utf8(&sbuf).unwrap());
        match result {
            Ok(acl) => {
                *self = acl.clone();
                Ok(self)
            }
            Err(e) => Err(String::from(e)),
        }
    }

    pub fn number_group_add(&mut self, line: &str) {
        let t = self.pattern.parse_number_group(&line);
        self.data
            .number_group
            .entry(t.0)
            .and_modify(|e| e.push(t.1.clone()))
            .or_insert(vec![t.1]);
    }

    pub fn ip_group_add(&mut self, line: &str) {
        let t = self.pattern.parse_ip_group(&line);
        self.data
            .ip_group
            .entry(t.0)
            .and_modify(|e| e.push(t.1.clone()))
            .or_insert(vec![t.1]);
    }

    pub fn string_group_add(&mut self, line: &str) {
        let t = self.pattern.parse_string_group(&line);
        self.data
            .string_group
            .entry(t.0)
            .and_modify(|e| e.push(t.1.clone()))
            .or_insert(vec![t.1]);
    }

    /// header_add appends header into Acl.table.
    /// If a semantically identical AclHeader is exists in Acl.table,
    /// it does not make changes.
    ///
    pub fn header_add(&mut self, line: &str) -> AclHeader {
        let a = self.pattern.parse_acl_header(line);
        let header = AclHeader {
            priority: a.0,
            op: a.1,
            attr: a.2.clone(),
        };
        //println!("attr-recog: {:?}", a.2);
        let mut hasher = DefaultHasher::new();
        header.hash(&mut hasher);

        self.data.entry(hasher.finish()).or_insert(AclBlock {
            header: header.clone(),
            rule: vec![],
        });
        header
    }
    pub fn raw_header_add(&mut self, header: &AclHeader) -> &AclHeader {
        let mut hasher = DefaultHasher::new();
        header.hash(&mut hasher);
        let header = header.clone();
        self.data.entry(hasher.finish()).or_insert(AclBlock {
            header,
            rule: vec![],
        });
        &self.data.get(hasher.finish()).unwrap().header
    }

    /// audit_add appends audit line into an ACL block, specified with
    /// AclHeader as a key.
    ///
    pub fn audit_add(&mut self, header: &AclHeader, line: &str) {
        let seq = self.pattern.parse_acl_audit_sequence(&line.to_string());
        let audit = AclRule {
            priority: seq,
            verb: Verb::Audit,
            attr: vec![],
        };
        self.raw_rule_add(&header, audit);
    }

    /// rule_add appends rule line into an ACL block, specified with
    /// AclHeader as a key.
    ///
    pub fn rule_add(&mut self, header: &AclHeader, line: &str) {
        let r = self.pattern.parse_acl_rule(&line);
        if self.debug {
            println!("prio={} verb={:?} attr={:?}", r.0, r.1, r.2.clone());
        }
        let rule = AclRule {
            priority: r.0,
            verb: r.1,
            attr: r.2,
        };
        self.raw_rule_add(&header, rule);
    }
    pub fn raw_rule_add(&mut self, header: &AclHeader, rule: AclRule) {
        let mut hasher = DefaultHasher::new();
        header.hash(&mut hasher);
        self.data
            .entry(hasher.finish())
            .and_modify(|e| e.rule.push(rule));
    }

    /// count counts headers with supplied matcher function
    ///
    pub fn count(&self, f: impl Fn(AclHeader) -> bool) -> usize {
        let mut count = 0;
        for v in self.data.values() {
            if f(v.header.clone()) {
                count += 1;
            }
        }
        count
    }

    /// count_op counts headers which has given operation
    ///
    pub fn count_op(&self, op: Op) -> usize {
        let mut count = 0;
        for v in self.data.values() {
            if op == v.header.op {
                count += 1;
            }
        }
        count
    }

    pub fn set_op(&mut self, op: &str) -> Result<(), String> {
        let mut new_table = HashMap::new();
        let op_list: Vec<Op> = Op::detect(op)?;
        for op in op_list {
            for k in self.data.clone().keys() {
                let mut v = self.data.get(*k).unwrap().clone();
                let mut hasher = DefaultHasher::new();
                v.header.op = op.clone();
                v.header.hash(&mut hasher);
                new_table.insert(hasher.finish(), v);
            }
        }
        self.data.set_table(new_table);
        Ok(())
    }

    /// rule_count_verb counts rules which has given verb
    /// for each ACL blocks
    ///
    pub fn rule_count_verb(&self, verb: Verb) -> Vec<usize> {
        let mut res = vec![0; self.data.len()];
        for v in self.data.values() {
            for i in 0..v.rule.len() {
                if self.debug {
                    eprintln!(
                        "\x1B[33m[debug]\x1B[0m verb: {}, cmp {}",
                        &v.rule[i].verb, &verb
                    )
                }
                if v.rule[i].verb == verb {
                    res[i] += 1;
                }
            }
        }
        res
    }

    /// rule_count_resource
    ///
    pub fn rule_count_resource(&self, resource: Resource) -> Vec<usize> {
        let mut res = vec![0; self.data.len()];
        let mut i = 0;
        for v in self.data.values() {
            res[i] = 0;
            for i in 0..v.rule.len() {
                for attr in &v.rule[i].attr {
                    if attr.0 == resource {
                        res[i] += 1;
                        break;
                    }
                }
            }
            i += 1;
        }
        res
    }

    /// parse_acl_blocks_by_header returns the reference to an AclBlock.
    /// It makes a match for given line and find a line which has completely
    /// equal semantics in the policy.
    ///
    pub fn parse_acl_block_by_header(&self, line: &str) -> Option<&AclBlock> {
        if !self.pattern.is_acl_header(line) {
            eprintln!("invalid syntax for acl header");
            return None;
        }
        let a = self.pattern.parse_acl_header(&line);
        let mut hasher = DefaultHasher::new();
        AclHeader {
            priority: a.0,
            op: a.1,
            attr: a.2,
        }
        .hash(&mut hasher);
        match self.data.get(hasher.finish().clone()) {
            Some(n) => Some(n),
            None => None,
        }
    }

    // parse_acl_blocks_by_header_hash returns a reference to AclBlock, that is pointed from
    // the given hash.
    pub fn parse_acl_blocks_by_header_hash(&self, hash: u64) -> Option<&AclBlock> {
        self.data.get(hash)
    }

    /// parse_acl_blocks_by_header_priority returns sets of header and rules for which the header priority
    /// equals to given value.
    ///
    pub fn parse_acl_blocks_by_header_priority(&self, priority: u16) -> Vec<&AclBlock> {
        let mut res = vec![];
        for b in self.data.values() {
            if b.header.priority == priority {
                res.push(b)
            }
        }

        res.sort_by(|v, w| match v.header.priority == w.header.priority {
            false => v.header.priority.cmp(&w.header.priority),
            true => v.header.op.cmp(&w.header.op),
        });
        res
    }

    /// parse_acl_blocks_by_header_operation returns sets of header and rules for which the header operaion
    /// equals to given operation.
    ///
    pub fn parse_acl_blocks_by_header_operation(&self, op: Op) -> Vec<&AclBlock> {
        let mut res = vec![];
        for b in self.data.values() {
            if b.header.op == op {
                res.push(b)
            }
        }
        res.sort_by(|v, w| match v.header.priority == w.header.priority {
            false => v.header.priority.cmp(&w.header.priority),
            true => v.header.op.cmp(&w.header.op),
        });
        res
    }

    /// parse_acl_blocks_by_header_pattern returns sets of header and rules within Results type.
    /// The given string is assumed to be an regex seed, unless it returns error.
    /// Any headers which matches given regex pattern are returned with coorespondiing
    /// Rule sets.
    ///
    pub fn parse_acl_blocks_by_header_pattern(
        &self,
        pattern: &str,
    ) -> Result<Vec<&AclBlock>, String> {
        match Regex::new(pattern) {
            Ok(pat) => {
                let mut res = vec![];
                for b in self.data.values() {
                    let mut k = format!("{} acl {}", b.header.priority, b.header.op.as_str());
                    for i in 0..b.header.attr.len() {
                        let a = &b.header.attr[i];
                        k += format!(" {}{}{}", a.0.as_str(), a.1.as_str(), a.2).as_str();
                    }
                    if pat.is_match(&k) {
                        res.push(b);
                    }
                }

                res.sort_by(|v, w| match v.header.priority == w.header.priority {
                    false => v.header.priority.cmp(&w.header.priority),
                    true => v.header.op.cmp(&w.header.op),
                });
                Ok(res)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// parse_acl_blocks_by_rule returns a slice of sets of AclHeader and AclRule(s).
    /// It makes a match for given line as a rule, and find rule(s) which has
    /// completely equal semantics. Finally, it returns ACL block(s) within a
    /// slice of tuple, which contains matched rules.
    ///
    pub fn parse_acl_blocks_by_rule(&self, line: &str) -> Vec<&AclBlock> {
        let mut res = vec![];
        let comp_rule = self.pattern.parse_acl_rule(line);
        let comp_rule = AclRule {
            priority: comp_rule.0,
            verb: comp_rule.1,
            attr: comp_rule.2,
        };

        for b in self.data.values() {
            for i in 0..b.rule.len() {
                if comp_rule == b.rule[i] {
                    res.push(b)
                }
            }
        }
        res.sort_by(|v, w| match v.header.priority == w.header.priority {
            false => v.header.priority.cmp(&w.header.priority),
            true => v.header.op.cmp(&w.header.op),
        });
        res
    }

    /// parse_acl_blocks_by_rule_pattern returns sets of header and rules within Results type.
    /// The given string is assumed to be an regex seed, unless it returns error.
    /// This is a rule version of `parse_acl_blocks_by_header_pattern` and returns any ACL
    /// blocks which has matched pattern in its rules.
    ///
    pub fn parse_acl_blocks_by_rule_pattern(
        &self,
        pattern: &str,
    ) -> Result<Vec<&AclBlock>, String> {
        match Regex::new(pattern) {
            Ok(pat) => {
                let mut res = vec![];
                for b in self.data.values() {
                    let rule = &b.rule;
                    for i in 0..rule.len() {
                        let mut attr = String::new();
                        for j in 0..rule[i].attr.len() {
                            let a = &rule[i].attr[j];
                            attr += &format!(" {}{}{}", a.0.as_str(), a.1.as_str(), a.2);
                        }
                        if pat.is_match(&format!(
                            "  {} {}{}",
                            rule[i].priority,
                            rule[i].verb.as_str(),
                            attr
                        )) {
                            res.push(b);
                            break;
                        }
                    }
                }
                res.sort_by(|v, w| match v.header.priority == w.header.priority {
                    false => v.header.priority.cmp(&w.header.priority),
                    true => v.header.op.cmp(&w.header.op),
                });
                Ok(res)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// parse_acl_headers returns simple slice for all header in the Acl.
    ///
    pub fn parse_acl_headers(&self) -> Vec<AclHeader> {
        let mut res = vec![];
        for b in self.data.values() {
            res.push(b.header.clone());
        }
        res.sort_by(|v, w| match v.priority == w.priority {
            false => v.priority.cmp(&w.priority),
            true => v.op.cmp(&w.op),
        });
        res
    }

    pub fn parse_acl_headers_by_pattern(&self, pat: &str) -> Vec<AclHeader> {
        if let Ok(a) = self.parse_acl_by_header(pat) {
            a.parse_acl_headers()
        } else {
            vec![]
        }
    }

    /// list_acl_headers
    ///
    pub fn list_acl_headers(&self) {
        for h in self.parse_acl_headers() {
            println!("{}", h);
        }
    }

    /// has_header detects the Acl has a header or not
    ///
    pub fn has_header(&self, h: &AclHeader) -> bool {
        let mut hasher = DefaultHasher::new();
        h.hash(&mut hasher);
        self.data.contains_key(hasher.finish())
    }

    /// parse_acl_by_header returns a new Acl with header query.
    /// A query may be digit (for priority),  operation string (for Op::from),
    ///
    pub fn parse_acl_by_header(&self, query: &str) -> Result<Acl, String> {
        let blocks = match query.parse::<u16>() {
            Ok(n) => self.parse_acl_blocks_by_header_priority(n),
            Err(_) => match Op::from(query) {
                Op::Error => {
                    return Err(format!("invalid query pattern: {}\ngive number (priority) or a valid operation (e.g. chmod).", query));
                }
                _ => self.parse_acl_blocks_by_header_operation(Op::from(query)),
            },
        };
        Ok(Acl::from(blocks))
    }

    /// parse_acl_by_header_with_regex returns a new Acl with header query in regex pattern.
    /// Headers are parsed as plain strings and filtered with given regex.
    pub fn parse_acl_by_header_with_regex(&self, query: &str) -> Result<Acl, String> {
        match self.parse_acl_blocks_by_header_pattern(query) {
            Ok(set) => Ok(Acl::from(set)),
            Err(e) => Err(e.to_string()),
        }
    }

    /// parse_acl_by_rule results a new Acl with rule query.
    /// Now a query should be a regex pattern with the prefix `regex:`.
    ///
    pub fn parse_acl_by_rule(&self, query: &str) -> Result<Acl, String> {
        let set = self.parse_acl_blocks_by_rule(query);
        if set.len() != 0 {
            Ok(Acl::from(set))
        } else {
            Err(format!("no such rules: {}", query))
        }
    }

    /// parse_acl_by_rule returns a new Acl with header query in regex pattern.
    /// Headers are parsed as plain strings and filtered with given regex.
    pub fn parse_acl_by_rule_with_regex(&self, query: &str) -> Result<Acl, String> {
        let set = self.parse_acl_blocks_by_rule_pattern(query)?;
        if set.len() != 0 {
            return Ok(Acl::from(set));
        }
        return Err("no matches".to_string());
    }

    pub fn list_header_by_header_pattern(&self, query: &str) -> Result<Acl, String> {
        let blocks = match query.parse::<u16>() {
            Ok(n) => self.parse_acl_blocks_by_header_priority(n),
            Err(_) => match Op::from(query) {
                Op::Error => {
                    let pat = Regex::new(r"^regex:(?P<pat>.+$)").unwrap();
                    match pat.is_match(query) {
                        true => {
                            let cap = pat.captures(query).unwrap();
                            match self.parse_acl_blocks_by_header_pattern(&cap["pat"]) {
                                Ok(set) => set,
                                Err(e) => {
                                    eprintln!("{}", e);
                                    vec![]
                                }
                            }
                        }
                        false => {
                            return Err(format!("no such pattern: {}", query));
                        }
                    }
                }
                _ => self.parse_acl_blocks_by_header_operation(Op::from(query)),
            },
        };
        Ok(Acl::from(blocks))
    }

    /// dumps raw object of ACL table
    ///
    pub fn dump_table(&self) {
        println!("{:?}", self.data.as_table_ref());
    }

    /// An atomic Acl is a policy composed of just one AclHeader
    /// and related rules.
    ///
    pub fn is_atomic(&self) -> bool {
        self.data.len() == 1
    }

    /// len returns the number of acl blocks
    ///
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// rule_len gives slices of rule length for each acl blocks
    ///
    pub fn rule_len(&self) -> Vec<usize> {
        let mut res = vec![0; self.data.len()];
        let mut i = 0;
        for v in self.data.values() {
            res[i] = v.rule.len();
            i += 1;
        }
        res
    }
}

impl fmt::Display for Acl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut resp = String::new();
        for b in self.data.values().clone() {
            resp += format!("{}\n", &b.header).as_str();
            for i in 0..b.rule.len() {
                resp += format!("{}\n", &b.rule[i]).as_str();
            }
        }
        write!(f, "{}", resp)
    }
}

impl fmt::Debug for Acl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Acl")
            .field(&self.data)
            .field(&self.debug)
            .finish()
    }
}

/// AclData represents a policy entity. ACL blocks are held in the `list`
/// field within HashMap, keyed with hash value of AclHeader instances.
///
/// Optionlly, ACL instance
/// can hold number_group, string_group and ip_group for its resource groups.
///
/// Acl instance can represent full-featured, active policy on real host or valid
/// ACL blocks written in plain text.
///
#[derive(Clone, Serialize, Deserialize)]
pub struct AclData {
    pub version: u32,
    pub stat: u32,
    pub quota: AclQuota,
    pub number_group: HashMap<String, Vec<String>>,
    pub string_group: HashMap<String, Vec<String>>,
    pub ip_group: HashMap<String, Vec<String>>,
    //     table: Vec<AclBlock>
    table: HashMap<u64, AclBlock>,
}

impl fmt::Debug for AclData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Acl")
            .field(&self.version)
            .field(&self.stat)
            .field(&self.quota)
            .field(&self.number_group)
            .field(&self.string_group)
            .field(&self.ip_group)
            .field(&self.table)
            .finish()
    }
}

impl<'a> AclData {
    pub fn new() -> Self {
        AclData {
            version: 0,
            stat: 0,
            quota: AclQuota::new(),
            number_group: HashMap::with_capacity(MAX_DATA_GROUPS),
            ip_group: HashMap::with_capacity(MAX_DATA_GROUPS),
            string_group: HashMap::with_capacity(MAX_DATA_GROUPS),
            table: HashMap::with_capacity(MAX_DATA_BLOCKS),
        }
    }
    pub fn clear(&mut self) {
        self.version = 0;
        self.stat = 0;
        //         self.quota.clear();
        self.number_group.clear();
        self.ip_group.clear();
        self.string_group.clear();
        self.table.clear();
    }
    pub fn set_table(&mut self, hm: HashMap<u64, AclBlock>) {
        self.table.clear();
        self.table = hm;
    }
    pub fn as_table_ref(&self) -> &HashMap<u64, AclBlock> {
        &self.table
    }

    pub fn get(&self, k: u64) -> Option<&AclBlock> {
        self.table.get(&k)
    }

    pub fn keys(&'a self) -> Keys<'a, u64, AclBlock> {
        self.table.keys()
    }
    pub fn entry(&'a mut self, k: u64) -> Entry<'a, u64, AclBlock> {
        self.table.entry(k)
    }
    pub fn values(&'a self) -> Values<'a, u64, AclBlock> {
        self.table.values()
    }
    pub fn contains_key(&self, k: u64) -> bool {
        self.table.contains_key(&k)
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }
}

#[cfg(test)]
mod bdd {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn get_acl_fixtures(p: &str, is_valid_fixtures: bool) -> Vec<String> {
        let mut files = Vec::new();
        let root = PathBuf::from(p);

        let contents = std::fs::read_dir(root).unwrap();

        for f in contents {
            let line = f.unwrap();
            if (line.file_name().to_string_lossy().starts_with("0")
                || line.file_name().to_string_lossy().starts_with("1"))
                ^ !is_valid_fixtures
            {
                files.push(format!("{}{}", p, line.file_name().to_string_lossy()));
            }
        }

        files.sort();
        files
    }

    #[test]
    fn test_valid_policies() {
        for f in get_acl_fixtures(
            format!("{}/{}", env::current_dir().unwrap().display(), "fixtures/").as_str(),
            true,
        ) {
            let data = fs::read_to_string(&f).unwrap();
            match Acl::from_str(&data) {
                Ok(_) => {
                    println!("\x1b[32mOK\x1b[00m: {}", f)
                }
                Err(e) => {
                    eprintln!("\x1b[31mERR\x1b[00m: {} ({})", f, e.to_string());
                    panic!("\x1b[31mERR\x1b[00m:\n {} ({})", f, e.to_string());
                }
            }
        }
    }

    #[test]
    fn test_invalid_policies() {
        for f in get_acl_fixtures(
            format!("{}/{}", env::current_dir().unwrap().display(), "fixtures/").as_str(),
            false,
        ) {
            let data = fs::read_to_string(&f).unwrap();
            match Acl::from_str(&data) {
                Ok(_) => {
                    eprintln!("\x1b[31mERR\x1b[00m: {} (should be error)", f);
                    panic!("\x1b[31mERR\x1b[00m:\n {} (should be error)", f);
                }
                Err(_) => {
                    println!("\x1b[32mOK\x1b[00m: {}", f)
                }
            }
        }
    }
}
