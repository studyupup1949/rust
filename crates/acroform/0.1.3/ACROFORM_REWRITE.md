# AcroForm Rewrite Plan: Migrating from Forked PDF to lopdf@0.38.0

## Executive Summary

This document outlines the plan to rewrite the `acroform` crate to use the official `lopdf@0.38.0` from crates.io instead of the forked `pdf` crate (`acroform-pdf`). The goal is to achieve API parity with the existing implementation while leveraging a well-maintained, community-supported PDF library.

## Current Implementation Analysis

### Existing Architecture (acroform-rs-old)

The current implementation consists of:

1. **acroform-pdf** (forked from pdf-rs)
   - Custom fork of pdf-rs library
   - Located in `acroform-rs-old/pdf/`
   - Provides low-level PDF parsing and manipulation
   - Key types: `CachedFile`, `FieldDictionary`, `InteractiveFormDictionary`, `Resolve`, `RcRef`

2. **acroform** (form manipulation layer)
   - Located in `acroform-rs-old/acroform/`
   - High-level API for form filling
   - Three main modules:
     - `lib.rs`: Public exports and documentation
     - `api.rs`: `AcroFormDocument`, `FormField`, `FieldValue`
     - `field.rs`: Extension traits for field traversal

### Current API Surface

```rust
// Core API
pub struct AcroFormDocument;
impl AcroFormDocument {
    pub fn from_pdf(path: impl AsRef<Path>) -> Result<Self, PdfError>;
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, PdfError>;
    pub fn fields(&self) -> Result<Vec<FormField>, PdfError>;
    pub fn fill(&mut self, values: HashMap<String, FieldValue>) -> Result<Vec<u8>, PdfError>;
    pub fn fill_and_save(&mut self, values: HashMap<String, FieldValue>, output: impl AsRef<Path>) -> Result<(), PdfError>;
}

// High-level field representation
pub struct FormField {
    pub name: String,
    pub field_type: FieldType,
    pub current_value: Option<FieldValue>,
    pub default_value: Option<FieldValue>,
    pub flags: u32,
    pub tooltip: Option<String>,
}

// Typed field values
pub enum FieldValue {
    Text(String),
    Boolean(bool),
    Choice(String),
    Integer(i32),
}

// Extension traits (internal)
pub trait FieldDictionaryExt {
    fn get_full_name(&self, resolver: &impl Resolve) -> Result<String, PdfError>;
    fn traverse_field_refs(&self, resolver: &impl Resolve) -> Result<Vec<RcRef<FieldDictionary>>, PdfError>;
}

pub trait InteractiveFormDictionaryExt {
    fn all_fields(&self, resolver: &impl Resolve) -> Result<Vec<RcRef<FieldDictionary>>, PdfError>;
    fn find_field_by_name(&self, name: &str, resolver: &impl Resolve) -> Result<Option<RcRef<FieldDictionary>>, PdfError>;
}
```

## lopdf@0.38.0 Analysis

### Core Types and API

```rust
// Main document type
pub struct Document {
    pub version: String,
    pub trailer: Dictionary,
    pub reference_table: Xref,
    pub objects: BTreeMap<ObjectId, Object>,
    pub max_id: u32,
    // ... other fields
}

impl Document {
    pub fn new() -> Self;
    pub fn load(path: impl AsRef<Path>) -> Result<Self>;
    pub fn load_from(reader: impl Read) -> Result<Self>;
    pub fn save(path: impl AsRef<Path>) -> Result<()>;
    pub fn save_to<W: Write>(&mut self, writer: W) -> Result<()>;
    
    pub fn get_object(&self, id: ObjectId) -> Result<&Object>;
    pub fn get_object_mut(&mut self, id: ObjectId) -> Result<&mut Object>;
    pub fn get_dictionary(&self, id: ObjectId) -> Result<&Dictionary>;
    pub fn get_dictionary_mut(&mut self, id: ObjectId) -> Result<&mut Dictionary>;
    pub fn catalog(&self) -> Result<&Dictionary>;
    pub fn catalog_mut(&mut self) -> Result<&mut Dictionary>;
    pub fn get_pages(&self) -> BTreeMap<u32, ObjectId>;
    pub fn get_page_annotations(&self, page_id: ObjectId) -> Result<Vec<&Dictionary>>;
    // ... more methods
}

// Object types
pub type ObjectId = (u32, u16);

pub enum Object {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f32),
    Name(Vec<u8>),
    String(Vec<u8>, StringFormat),
    Array(Vec<Object>),
    Dictionary(Dictionary),
    Stream(Stream),
    Reference(ObjectId),
}

pub struct Dictionary(IndexMap<Vec<u8>, Object>);

impl Dictionary {
    pub fn new() -> Self;
    pub fn get(&self, key: &[u8]) -> Result<&Object>;
    pub fn set(&mut self, key: impl Into<Vec<u8>>, value: impl Into<Object>);
    pub fn has(&self, key: &[u8]) -> bool;
    pub fn remove(&mut self, key: &[u8]) -> Option<Object>;
    pub fn len(&self) -> usize;
    pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &Object)>;
    // ... more methods
}
```

### Key Differences

| Feature | Forked pdf crate | lopdf@0.38.0 |
|---------|------------------|--------------|
| **Document Type** | `CachedFile<Vec<u8>>` | `Document` |
| **Loading** | `FileOptions::cached().open(path)` | `Document::load(path)` |
| **Object Access** | `resolver.get(ref)` pattern | `doc.get_object(id)` |
| **Reference Handling** | `RcRef<T>` wrapper | Direct `ObjectId` |
| **Type System** | Strongly typed structs (`FieldDictionary`, `InteractiveFormDictionary`) | Generic `Dictionary` + manual key lookup |
| **Updates** | `file.update(ref, obj)` | Direct mutation via `get_object_mut()` |
| **Saving** | `file.save()` returns bytes | `doc.save_to()` writes to writer |
| **Error Type** | `PdfError` | `lopdf::Error` |
| **String Encoding** | `PdfString` with encoding helpers | `Vec<u8>` with manual UTF-16BE encoding |

## Implementation Strategy

### Phase 1: Project Setup ✅

**Goal**: Set up new crate structure and dependencies

**Tasks**:
- [x] Create new `acroform/` directory at repository root
- [x] Create `Cargo.toml` with `lopdf = "0.38.0"` dependency
- [x] Copy existing API structure (lib.rs, api.rs, field.rs skeleton)
- [x] Set up basic module structure and re-exports

**Deliverables**:
- ✅ Working Cargo project that compiles
- ✅ Empty trait and struct definitions matching existing API

### Phase 2: Core Types and Error Handling ✅

**Goal**: Implement type conversions and error handling

**Tasks**:
- [x] Create error type that wraps `lopdf::Error`
- [x] Implement `FieldValue` enum with conversion methods
- [x] Implement `FormField` struct
- [x] Create helper functions for string encoding (UTF-16BE with BOM)

**Key Implementation Details**:

```rust
// Error handling
#[derive(Debug)]
pub enum Error {
    Lopdf(lopdf::Error),
    MissingField { field: String },
    InvalidFieldType { expected: String, found: String },
    Io(std::io::Error),
}

impl From<lopdf::Error> for Error {
    fn from(err: lopdf::Error) -> Self {
        Error::Lopdf(err)
    }
}

// Field value conversions
impl FieldValue {
    pub fn from_object(obj: &Object) -> Option<Self> {
        match obj {
            Object::String(bytes, _) => {
                // Handle UTF-16BE decoding if BOM present
                if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
                    // UTF-16BE with BOM
                    let u16_chars: Vec<u16> = bytes[2..]
                        .chunks_exact(2)
                        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                        .collect();
                    String::from_utf16(&u16_chars).ok().map(FieldValue::Text)
                } else {
                    // Latin-1/PDFDocEncoding
                    String::from_utf8_lossy(bytes).to_string().into()
                }
            }
            Object::Integer(i) => Some(FieldValue::Integer(*i as i32)),
            Object::Name(n) => Some(FieldValue::Choice(String::from_utf8_lossy(n).to_string())),
            Object::Boolean(b) => Some(FieldValue::Boolean(*b)),
            _ => None,
        }
    }
    
    pub fn to_object(&self) -> Object {
        match self {
            FieldValue::Text(s) => {
                // Encode as UTF-16BE with BOM
                let mut bytes = vec![0xFE, 0xFF]; // BOM
                for c in s.encode_utf16() {
                    bytes.push((c >> 8) as u8);
                    bytes.push((c & 0xFF) as u8);
                }
                Object::String(bytes, StringFormat::Literal)
            }
            FieldValue::Integer(i) => Object::Integer(*i as i64),
            FieldValue::Choice(s) => Object::Name(s.as_bytes().to_vec()),
            FieldValue::Boolean(b) => Object::Boolean(*b),
        }
    }
}
```

### Phase 3: Field Traversal ✅

**Goal**: Implement field discovery and hierarchy traversal

**Tasks**:
- [x] Locate AcroForm dictionary in document catalog
- [x] Traverse Fields array in AcroForm
- [x] Implement recursive field tree traversal
- [x] Build fully qualified field names (parent.child.field)
- [x] Extract field metadata (type, flags, tooltip, values)

**Key Implementation Details**:

```rust
// Field traversal helpers
fn get_acroform_dict<'a>(doc: &'a Document) -> Result<&'a Dictionary> {
    let catalog = doc.catalog()?;
    let acroform_ref = catalog.get(b"AcroForm")?.as_reference()?;
    doc.get_dictionary(acroform_ref)
}

fn get_field_type(field_dict: &Dictionary) -> Option<String> {
    field_dict.get(b"FT")
        .ok()
        .and_then(|obj| obj.as_name().ok())
        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
}

fn get_field_name(field_dict: &Dictionary) -> Option<String> {
    field_dict.get(b"T")
        .ok()
        .and_then(|obj| obj.as_str().ok())
        .map(|bytes| decode_text_string(bytes))
}

fn get_field_value(field_dict: &Dictionary) -> Option<Object> {
    field_dict.get(b"V").ok().cloned()
}

fn traverse_field_tree(
    doc: &Document,
    field_ref: ObjectId,
    parent_name: Option<String>,
    results: &mut Vec<FormField>,
) -> Result<()> {
    let field_dict = doc.get_dictionary(field_ref)?;
    
    // Get field name
    let field_name = get_field_name(field_dict);
    let full_name = match (parent_name, field_name) {
        (Some(p), Some(n)) => format!("{}.{}", p, n),
        (None, Some(n)) => n,
        (Some(p), None) => p,
        (None, None) => String::new(),
    };
    
    // Check if this is a terminal field (has FT key)
    if let Some(field_type) = get_field_type(field_dict) {
        // Extract field information
        let current_value = get_field_value(field_dict)
            .and_then(|obj| FieldValue::from_object(&obj));
        
        let default_value = field_dict.get(b"DV")
            .ok()
            .and_then(|obj| FieldValue::from_object(obj));
        
        let flags = field_dict.get(b"Ff")
            .ok()
            .and_then(|obj| obj.as_i64().ok())
            .unwrap_or(0) as u32;
        
        let tooltip = field_dict.get(b"TU")
            .ok()
            .and_then(|obj| obj.as_str().ok())
            .map(|bytes| decode_text_string(bytes));
        
        results.push(FormField {
            name: full_name.clone(),
            field_type,
            current_value,
            default_value,
            flags,
            tooltip,
        });
    }
    
    // Recursively process children (Kids array)
    if let Ok(kids) = field_dict.get(b"Kids").and_then(|obj| obj.as_array()) {
        for kid_obj in kids {
            if let Ok(kid_ref) = kid_obj.as_reference() {
                traverse_field_tree(doc, kid_ref, Some(full_name.clone()), results)?;
            }
        }
    }
    
    Ok(())
}

impl AcroFormDocument {
    pub fn fields(&self) -> Result<Vec<FormField>> {
        let mut results = Vec::new();
        
        let acroform = get_acroform_dict(&self.doc)?;
        if let Ok(fields_array) = acroform.get(b"Fields").and_then(|obj| obj.as_array()) {
            for field_obj in fields_array {
                if let Ok(field_ref) = field_obj.as_reference() {
                    traverse_field_tree(&self.doc, field_ref, None, &mut results)?;
                }
            }
        }
        
        Ok(results)
    }
}
```

### Phase 4: Document Loading ✅

**Goal**: Implement document loading from file and bytes

**Tasks**:
- [x] Implement `from_pdf()` using `Document::load()`
- [x] Implement `from_bytes()` using `Document::load_from()`
- [x] Handle encryption/password-protected PDFs (if needed)

**Key Implementation Details**:

```rust
pub struct AcroFormDocument {
    doc: Document,
}

impl AcroFormDocument {
    pub fn from_pdf(path: impl AsRef<Path>) -> Result<Self> {
        let doc = Document::load(path)?;
        Ok(AcroFormDocument { doc })
    }
    
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        use std::io::Cursor;
        let doc = Document::load_from(Cursor::new(data))?;
        Ok(AcroFormDocument { doc })
    }
}
```

### Phase 5: Field Updates and Saving ✅

**Goal**: Implement field value updates and PDF saving

**Tasks**:
- [x] Implement field update by name lookup
- [x] Update both field dictionary and widget annotations
- [x] Handle appearance stream updates (NeedAppearances flag)
- [x] Implement `fill()` method returning bytes
- [x] Implement `fill_and_save()` convenience method

**Key Implementation Details**:

```rust
impl AcroFormDocument {
    pub fn fill(&mut self, values: HashMap<String, FieldValue>) -> Result<Vec<u8>> {
        // 1. Find all fields that need updating
        let acroform_ref = self.doc.catalog()?
            .get(b"AcroForm")?
            .as_reference()?;
        
        // 2. For each value to update:
        for (field_name, new_value) in values {
            // Find field by traversing tree
            if let Some(field_ref) = self.find_field_by_name(&field_name)? {
                // Update field value
                let field_dict = self.doc.get_dictionary_mut(field_ref)?;
                field_dict.set("V", new_value.to_object());
                
                // Also update appearance state for buttons/choices
                if matches!(new_value, FieldValue::Choice(_) | FieldValue::Boolean(_)) {
                    field_dict.set("AS", new_value.to_object());
                }
            }
        }
        
        // 3. Set NeedAppearances flag
        let acroform = self.doc.get_dictionary_mut(acroform_ref)?;
        acroform.set("NeedAppearances", true);
        
        // 4. Save to bytes
        let mut buffer = Vec::new();
        self.doc.save_to(&mut buffer)?;
        Ok(buffer)
    }
    
    pub fn fill_and_save(
        &mut self,
        values: HashMap<String, FieldValue>,
        output: impl AsRef<Path>,
    ) -> Result<()> {
        let bytes = self.fill(values)?;
        std::fs::write(output, bytes)?;
        Ok(())
    }
    
    fn find_field_by_name(&self, name: &str) -> Result<Option<ObjectId>> {
        let acroform = get_acroform_dict(&self.doc)?;
        if let Ok(fields_array) = acroform.get(b"Fields").and_then(|obj| obj.as_array()) {
            for field_obj in fields_array {
                if let Ok(field_ref) = field_obj.as_reference() {
                    if let Some(found) = self.search_field_tree(field_ref, name, None)? {
                        return Ok(Some(found));
                    }
                }
            }
        }
        Ok(None)
    }
    
    fn search_field_tree(
        &self,
        field_ref: ObjectId,
        target_name: &str,
        parent_name: Option<String>,
    ) -> Result<Option<ObjectId>> {
        let field_dict = self.doc.get_dictionary(field_ref)?;
        
        let field_name = get_field_name(field_dict);
        let full_name = match (parent_name, field_name) {
            (Some(p), Some(n)) => format!("{}.{}", p, n),
            (None, Some(n)) => n,
            (Some(p), None) => p,
            (None, None) => String::new(),
        };
        
        if full_name == target_name {
            return Ok(Some(field_ref));
        }
        
        // Search children
        if let Ok(kids) = field_dict.get(b"Kids").and_then(|obj| obj.as_array()) {
            for kid_obj in kids {
                if let Ok(kid_ref) = kid_obj.as_reference() {
                    if let Some(found) = self.search_field_tree(kid_ref, target_name, Some(full_name.clone()))? {
                        return Ok(Some(found));
                    }
                }
            }
        }
        
        Ok(None)
    }
}
```

### Phase 6: Widget Annotations ✅

**Goal**: Update widget annotations to ensure proper rendering

**Tasks**:
- [x] Find widget annotations linked to fields
- [x] Update annotation values in sync with field values
- [x] Handle appearance dictionaries
- [x] Test with multiple widget annotations per field

**Implementation Notes**:
- Widget annotations can be merged with field dictionaries (field IS the widget)
- Widget annotations can be in the field's Kids array
- Widget annotations can be referenced from page annotations
- The implementation updates all three cases: field dictionary, Kids array widgets, and page annotation widgets
- Both `/V` (value) and `/AS` (appearance state) entries are updated for buttons and choices

### Phase 7: Testing ✅

**Goal**: Ensure full API compatibility and correctness

**Tasks**:
- [x] Port all existing integration tests
- [x] Test with af8.pdf test file
- [x] Verify round-trip field updates
- [x] Test in-memory operations
- [x] Test with various field types (text, boolean, choice, integer)
- [x] Validate PDFs open correctly in viewers

### Phase 8: Documentation and Examples ✅

**Goal**: Complete documentation for the new implementation

**Tasks**:
- [x] Update README with lopdf-based implementation
- [x] Port examples (simple_fill.rs, in_memory_fill.rs)
- [x] Add migration guide from old implementation
- [x] Document known limitations and differences

## API Parity Checklist

### Public API (Must Match)

- [x] `AcroFormDocument::from_pdf(path)` - Load from file path
- [x] `AcroFormDocument::from_bytes(data)` - Load from byte vector
- [x] `AcroFormDocument::fields()` - List all form fields
- [x] `AcroFormDocument::fill(values)` - Fill and return bytes
- [x] `AcroFormDocument::fill_and_save(values, output)` - Fill and save to file
- [x] `FormField` struct with all public fields
- [x] `FieldValue` enum with all variants
- [x] `FieldValue::from_object()` - Convert from PDF primitive (renamed from from_primitive)
- [x] `FieldValue::to_object()` - Convert to PDF primitive (renamed from to_primitive)

### Behavior (Must Match)

- [x] UTF-16BE encoding for text fields with BOM
- [x] Hierarchical field name resolution (parent.child.field)
- [x] Setting NeedAppearances flag
- [x] Non-incremental PDF writing
- [x] In-memory operations (no temp files)
- [x] Widget annotation updates
- [x] Default value extraction
- [x] Tooltip extraction

### Extensions (Nice to Have)

- [ ] Better error messages
- [ ] Support for additional field types
- [ ] Field validation helpers
- [ ] Appearance stream generation (optional)

## Known Challenges and Solutions

### Challenge 1: No Typed Field Dictionaries

**Problem**: lopdf uses generic `Dictionary` instead of typed `FieldDictionary`

**Solution**: Create helper functions for field dictionary access:
```rust
fn get_field_type(dict: &Dictionary) -> Option<String>;
fn get_field_value(dict: &Dictionary) -> Option<Object>;
fn get_field_name(dict: &Dictionary) -> Option<String>;
```

### Challenge 2: Reference Resolution

**Problem**: No built-in `Resolve` trait; must manually resolve references

**Solution**: Use `Document::get_object()` and `Object::as_reference()`:
```rust
let obj_ref = some_obj.as_reference()?;
let resolved = doc.get_object(obj_ref)?;
```

### Challenge 3: String Encoding

**Problem**: Must manually handle UTF-16BE encoding

**Solution**: Implement helper functions:
```rust
fn encode_text_utf16be(text: &str) -> Vec<u8>;
fn decode_text_string(bytes: &[u8]) -> String;
```

### Challenge 4: Mutable Updates

**Problem**: Different update mechanism (direct mutation vs update queue)

**Solution**: Use `get_dictionary_mut()` for direct updates:
```rust
let dict = doc.get_dictionary_mut(field_ref)?;
dict.set("V", new_value);
```

## Testing Strategy

### Unit Tests
- String encoding/decoding
- Field value conversions
- Field name resolution
- Error handling

### Integration Tests
- Load PDF with forms
- List all fields
- Update field values
- Save modified PDF
- Round-trip verification
- In-memory operations

### Test Files
- Reuse existing `acroform_files/af8.pdf`
- Add PDFs with various field types
- Add PDFs with nested field hierarchies

## Migration Path

### For Library Users

The API remains identical, so migration is just changing the dependency:

```toml
# Old
[dependencies]
acroform = { path = "acroform-rs-old/acroform" }

# New
[dependencies]
acroform = "0.2.0"  # or appropriate version
```

### For Library Maintainers

1. Keep old implementation in `acroform-rs-old/` for reference
2. Develop new implementation in `acroform/` or root `acroform/`
3. Run parallel testing with both implementations
4. When stable, replace old implementation

## Success Criteria

1. ✅ All public API functions implemented
2. ✅ All integration tests pass
3. ✅ Example programs work identically
4. ✅ Generated PDFs render correctly in viewers
5. ✅ Performance is comparable or better
6. ✅ Code is well-documented
7. ✅ Error messages are helpful

## Timeline Estimate

- **Phase 1-2** (Setup & Types): 1-2 days
- **Phase 3** (Field Traversal): 2-3 days
- **Phase 4** (Loading): 1 day
- **Phase 5** (Updates): 2-3 days
- **Phase 6** (Annotations): 1-2 days
- **Phase 7** (Testing): 2-3 days
- **Phase 8** (Documentation): 1-2 days

**Total**: 10-16 days of development time

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| lopdf API insufficient for forms | High | Early prototype of field reading |
| Performance degradation | Medium | Benchmark against old implementation |
| Encoding issues with special chars | Medium | Comprehensive test suite with Unicode |
| Missing lopdf features | Medium | Contribute to lopdf or implement workarounds |
| Breaking API changes | Low | Keep API identical via adapter pattern |

## Dependencies

```toml
[dependencies]
lopdf = "0.38.0"

[dev-dependencies]
# For testing
```

## References

- [lopdf documentation](https://docs.rs/lopdf/0.38.0/lopdf/)
- [lopdf GitHub](https://github.com/J-F-Liu/lopdf)
- [PDF Reference 1.7](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)
- Current implementation: `acroform-rs-old/acroform/`

## Appendix: Key PDF Structures

### AcroForm Dictionary
```
/AcroForm <<
    /Fields [ (array of field references) ]
    /NeedAppearances true  (we set this)
    /SigFlags (integer)    (optional)
    /CO [ ... ]            (optional)
    /DR << ... >>          (optional default resources)
    /DA (string)           (optional default appearance)
    /Q (integer)           (optional quadding)
>>
```

### Field Dictionary
```
/Field <<
    /FT /Tx                (field type: /Tx, /Btn, /Ch, /Sig)
    /T (string)            (partial field name)
    /TU (string)           (tooltip/alternate name)
    /V (any)               (field value)
    /DV (any)              (default value)
    /Ff (integer)          (field flags)
    /Kids [ ... ]          (child fields)
    /Parent (ref)          (parent field)
>>
```

### Widget Annotation
```
/Annot <<
    /Type /Annot
    /Subtype /Widget
    /Rect [ ... ]
    /FT /Tx                (field type - if terminal field)
    /T (string)            (field name)
    /V (any)               (field value)
    /AS /name              (appearance state)
    /AP << ... >>          (appearance dictionary)
>>
```
