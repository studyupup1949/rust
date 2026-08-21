use serde_json;
use std::collections::HashMap;

#[cfg(windows)]
const JSON_MAP: &str = include_str!("..\\..\\src\\map.json");
#[cfg(not(windows))]
const JSON_MAP: &str = include_str!("../../src/map.json");

#[derive(Clone)]
pub struct AddressInfo {
    pub brief: Option<String>,
    pub ctx: Option<String>,
    pub desc: String,
    pub label: Option<String>,
    pub note: Option<String>,
    pub subctx: Option<String>,
    pub typ: String
}

/// All interactions are through this object
pub struct MemoryMap {
    memmap: HashMap<u16,AddressInfo>
}

fn get_val(key: &str,val: &serde_json::Value) -> Option<String> {
    match &val[key] {
        serde_json::Value::String(s) => Some(s.to_string()),
        _ => None
    }
}
fn get_contextual_str(str: &String, count: usize) -> String {
    let contexts: Vec<_> = str.split('|').collect();
    if count < contexts.len() {
        return contexts[count].trim().to_string();
    }
    return str.to_string();
}
fn get_contextual_str_maybe(maybe_str: &Option<String>, count: usize) -> Option<String> {
    if let Some(str) = maybe_str {
        let contexts: Vec<_> = str.split('|').collect();
        if count < contexts.len() {
            return Some(contexts[count].trim().to_string());
        }
    }
    return None;
}
fn get_contextual_info(info: &AddressInfo, count: usize) -> AddressInfo {
    AddressInfo {
        brief: get_contextual_str_maybe(&info.brief,count),
        ctx: get_contextual_str_maybe(&info.ctx,count),
        desc: get_contextual_str(&info.desc,count),
        subctx: get_contextual_str_maybe(&info.subctx,count),
        label: get_contextual_str_maybe(&info.label,count),
        note: get_contextual_str_maybe(&info.note,count),
        typ: get_contextual_str(&info.typ,count)
    }
}

impl MemoryMap {
    pub fn new() -> Self {
        let mut m = Self {
            memmap: HashMap::new()
        };
        let temp = serde_json::from_str::<serde_json::Value>(JSON_MAP).unwrap();
        if let Some(obj) = temp.as_object() {
            for (key,val) in obj {
                let addr  = u16::from_str_radix(&key[2..],16).expect("bad address in memory map");
                let addr_info = AddressInfo {
                    brief: get_val("brief",val),
                    ctx: get_val("ctx",val),
                    desc: get_val("desc",val).unwrap(),
                    subctx: get_val("subctx",val),
                    label: get_val("label",val),
                    note: get_val("note",val),
                    typ: get_val("type",val).unwrap(),
                };
                m.memmap.insert(addr,addr_info);
            }
        }
        assert!(m.memmap.len()>0);
        m
    }
    /// Borrow the underlying HashMap
    pub fn get_all(&self) -> &HashMap<u16,AddressInfo> {
        return &self.memmap;
    }
    /// Get data for one address.
    /// @param addr address in the range -32767 to 65535
    pub fn get_one(&self,addr: i64) -> Option<AddressInfo> {
        let pos_addr = match addr < 0 { true => addr + 1 + u16::MAX as i64, false => addr };
        if pos_addr >=0 && pos_addr < u16::MAX as i64 {
            self.memmap.get(&(pos_addr as u16)).cloned()
        } else {
            None
        }
    }
    /// Get data for one address and split by context
    /// @param addr Array of AddressInfo objects, one for each context, or undefined
    pub fn get_one_and_split(&self,addr: i64) -> Option<Vec<AddressInfo>> {
        if let Some(obj) = self.get_one(addr) {
            let mut ans = Vec::new();
            let num = match &obj.ctx {
                Some(ctx) => ctx.split('|').collect::<Vec<_>>().len(),
                None => 1
            };
            // if one context do not split anything
            if num == 1 {
                ans.push(obj);
                return Some(ans);
            }
            // if multi-context, split all fields on `|`
            for count in 0..num {
                ans.push(get_contextual_info(&obj, count));
            }
            return Some(ans);
        }
        None
    }
}

#[test]
fn get_cout() {
    let mem_map = MemoryMap::new();
    let cout = mem_map.get_one(0xfded);
    assert!(cout.unwrap().label==Some("COUT".to_string()));
}

#[test]
fn applesoft_integer() {
    let mem_map = MemoryMap::new();
    let addr = 0xf8;
    match mem_map.get_one_and_split(addr) {
        Some(info_list) => {
            assert_eq!(info_list.len(),2);
            assert_eq!(info_list[0].ctx,Some("Applesoft".to_string()));
            assert_eq!(info_list[1].ctx,Some("Integer BASIC".to_string()));
            assert_eq!(info_list[0].typ,"byte value".to_string());
            assert_eq!(info_list[1].typ,"float32".to_string());
            assert_eq!(info_list[0].label,Some("REMSTK".to_string()));
            assert_eq!(info_list[1].label,Some("FP1".to_string()));
        },
        None => panic!("address not found")
    }
    let addr = 0xdc;
    match mem_map.get_one_and_split(addr) {
        Some(info_list) => {
            assert_eq!(info_list.len(),2);
            assert_eq!(info_list[0].ctx,Some("Applesoft".to_string()));
            assert_eq!(info_list[1].ctx,Some("Integer BASIC".to_string()));
            assert_eq!(info_list[0].typ,"word".to_string());
            assert_eq!(info_list[1].typ,"word".to_string());
            assert_eq!(info_list[0].label,Some("ERRPOS".to_string()));
            assert_eq!(info_list[1].label,Some("CURLIN".to_string()));
        },
        None => panic!("address not found")
    }
}

#[test]
fn no_split() {
    let mem_map = MemoryMap::new();
    let addr = 0xff70;
    match mem_map.get_one_and_split(addr) {
        Some(info_list) => {
            assert_eq!(info_list.len(),1);
            assert_eq!(info_list[0].typ,"ROM routine".to_string());
        },
        None => panic!("address not found")
    }
    let addr = 0xe597;
    match mem_map.get_one_and_split(addr) {
        Some(info_list) => {
            assert_eq!(info_list.len(),1);
            assert_eq!(info_list[0].typ,"ROM routine".to_string());
            assert_eq!(info_list[0].subctx,Some("parsing & string".to_string()));
        },
        None => panic!("address not found")
    }
}

#[test]
fn splits_but_one_ctx() {
    let mem_map = MemoryMap::new();
    let addr = 0xc080;
    match mem_map.get_one_and_split(addr) {
        Some(info_list) => {
            assert_eq!(info_list.len(),1);
            assert_eq!(info_list[0].brief,Some("RDRAM | stepper 0".to_string()));
        },
        None => panic!("address not found")
    }
}