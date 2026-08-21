# Apple II Memory Map

![unit tests](https://github.com/dfgordon/a2-memory-map/actions/workflows/node.js.yml/badge.svg)

This is the TypeScript binding to a database of Apple II special addresses intended to be used with language servers.  The central element is the file `map.json` which maps addresses to a set of descriptive strings.

## Map Records

The map records correspond to the interface

```ts
interface AddressInfo {
    brief: string | undefined,
    ctx: string | undefined,
    desc: string,
    label: string | undefined,
    note: string | undefined,
    subctx: string | undefined,
    type: string
}
```

The whole database is retrieved as a `Map` using

```ts
get_all() : Map<string,AddressInfo>
```

To get a single record use

```ts
get_one(addr: number): AddressInfo | undefined
```

The argument `addr` can range from -32767 to 65535.  As an example `get_one(0xfded)` will give

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

The `map.json` file uses logic expressions to delineate information by context, e.g., `ctx` might have the value `"Applesoft | Integer BASIC"`.  The TypeScript interface provides a function to parse the logic and produce an array of records, with each element corresponding to a given context:

```ts
get_one_and_split(addr: number): Array<AddressInfo> | undefined
```

If there is no splitting of the `ctx` field, no other fields will be split.  It is not required to use this function.  If `get_one` is used instead, downstream will receive the logic expressions, which can be parsed in any manner desired.