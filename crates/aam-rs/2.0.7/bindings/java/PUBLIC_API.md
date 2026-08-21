# Public API (Java/Kotlin JVM)

Source of truth: `java/src/main/kotlin/AamDocument.kt`, `java/src/main/kotlin/AamBuilder.kt`.

## Main Type

- `com.rustgames.aam.AamDocument : AutoCloseable`
- `com.rustgames.aam.AamBuilder`

## Construction

- `AamDocument.parse(String content)`
- `AamDocument.load(String path)`

## Instance Methods

- `void reload(String content)`
- `String? get(String key)`
- `Map<String, String> deepSearch(String pattern)`
- `List<String> reverseSearch(String value)`
- `List<String> schemaNames()`
- `List<String> typeNames()`
- `void close()`

## Runtime Notes

- Native library is loaded from packaged resources at runtime.
- `IllegalStateException` is thrown on parse/load failures and closed-handle usage.

## Builder API

- `AamBuilder(capacity: Int = 0)`
- `AamBuilder.SchemaField.required(name: String, typeName: String)`
- `AamBuilder.SchemaField.optional(name: String, typeName: String)`
- `addLine(key: String, value: String): AamBuilder`
- `comment(text: String): AamBuilder`
- `schema(name: String, fields: Iterable<AamBuilder.SchemaField>): AamBuilder`
- `derive(path: String, schemas: Iterable<String> = emptyList()): AamBuilder`
- `import(path: String): AamBuilder`
- `typeAlias(alias: String, typeName: String): AamBuilder`
- `build(): String`
- `toFile(path: String)`

