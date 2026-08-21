# Public API (Go)

Source of truth: `go/aam/aam.go`.

## Main Type

- `type AAM struct`
- `type AAMBuilder struct`
- `type SchemaField struct`

## Constructors

- `New() (*AAM, error)`
- `Parse(content string) (*AAM, error)`
- `Load(path string) (*AAM, error)`
- `SplitAam(content string) map[string]*AAMBuilder`
- `NewBuilder() *AAMBuilder`
- `RequiredField(name, typeName string) SchemaField`
- `OptionalField(name, typeName string) SchemaField`

## Instance Methods

- `Format(content string) (string, error)`
- `Get(key string) (string, bool)`
- `Find(query string) map[string]string`
- `DeepSearch(pattern string) map[string]string`
- `ReverseSearch(value string) []string`
- `SchemaNames() []string`
- `TypeNames() []string`
- `LastError() string`
- `Close()`

## Builder Methods

- `AddLine(key, value string) *AAMBuilder`
- `Comment(text string) *AAMBuilder`
- `Schema(name string, fields []SchemaField) *AAMBuilder`
- `Derive(path string, schemas []string) *AAMBuilder`
- `Import(path string) *AAMBuilder`
- `TypeAlias(alias, typeName string) *AAMBuilder`
- `Build() string`
- `ToFile(path string) error`

## Notes

- `Close()` is idempotent and should be called when done.
- Methods on a closed handle return empty values or errors, depending on method.
