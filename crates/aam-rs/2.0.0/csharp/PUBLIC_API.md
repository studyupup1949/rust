# Public API (C#)

Source of truth: `csharp/src/AamDocument.cs`, `csharp/src/AamBuilder.cs`.

## Main Type

- `AamDocument : IDisposable`

## Construction

- `new AamDocument()`
- `AamDocument.Parse(string content)`
- `AamDocument.Load(string path)`

## Core Methods

- `string Format(string content)`
- `string? Get(string key)`
- `Dictionary<string, string> Find(string query)`
- `Dictionary<string, string> DeepSearch(string pattern)`
- `string[] ReverseSearch(string value)`
- `string[] SchemaNames()`
- `string[] TypeNames()`

## Builder API

- `AamBuilder`
- `SchemaField`
- `new AamBuilder()`
- `new AamBuilder(int capacity)`
- `SchemaField.Required(string name, string typeName)`
- `SchemaField.OptionalField(string name, string typeName)`
- `AamBuilder.AddLine(string key, string value)`
- `AamBuilder.Comment(string text)`
- `AamBuilder.Schema(string name, IEnumerable<SchemaField> fields)`
- `AamBuilder.SchemaMultiline(string name, IEnumerable<SchemaField> fields)`
- `AamBuilder.Derive(string path, IEnumerable<string> schemas)`
- `AamBuilder.Import(string path)`
- `AamBuilder.TypeAlias(string alias, string typeName)`
- `AamBuilder.Build()`
- `AamBuilder.ToFile(string path)`

## Lifecycle

- `bool IsClosed`
- `void Dispose()`

## Error Model

- Throws `AamException` for native parse/load/format/query failures.
- Throws `ObjectDisposedException` when using a closed instance.
