# Apple II Memory Map

![unit tests](https://github.com/dfgordon/a2-memory-map/actions/workflows/rust.yml/badge.svg)

This is the rust binding to a database of Apple II special addresses intended to be used with language servers.  The central element is the file `map.json` which maps addresses to a set of descriptive strings.

## Map Records

The map records correspond to the structure

```rs
pub struct AddressInfo {
    pub brief: Option<String>,
    pub ctx: Option<String>,
    pub desc: String,
    pub label: Option<String>,
    pub note: Option<String>,
    pub subctx: Option<String>,
    pub typ: String
}
```

Information is accessed through the opaque `struct MemoryMap`.  The whole database can be borrowed as a `HashMap` as follows:

```rs
let a2map = MemoryMap::new();
let full_map: &HashMap<u16,AddressInfo> = a2map.get_all();
```

To get a single record use

```rs
let maybe_info: Option<AddressInfo> = a2map.get_one(addr);
```

The argument `addr` can range from -32767 to 65535.  As an example `get_one(0xfded)` will give the rust equivalent of

```js
{
  brief: 'Print character in A',
  desc: 'Invoke output routine whose address is in (56).  Usually prints character in A.',
  label: 'COUT',
  type: 'ROM routine'
}
```

## AddressInfo Fields

* `brief`: short description suitable for display in a selection box
* `ctx`: notes on limiting context, such as hardware requirements, applicability to given language, etc.
* `desc`: long description suitable for hovers
* `label`: suggested assembler label
* `note`: any other notes
* `subctx`: notes on sub context, e.g., specific aspect of a language
* `type`: data type (e.g. `word`) or operation type (e.g. `ROM routine`, `soft switch`)

## Multi-context Records

The `map.json` file uses logic expressions to delineate information by context, e.g., `ctx` might have the value `"Applesoft | Integer BASIC"`.  The rust binding provides a function to parse the logic and produce an array of records, with each element corresponding to a given context:

```rs
let maybe_records: Option<Vec<AddressInfo>> = a2map.get_one_and_split(addr);
```

If there is no splitting of the `ctx` field, no other fields will be split.  It is not required to use this function.  If `get_one` is used instead, downstream will receive the logic expressions, which can be parsed in any manner desired.
