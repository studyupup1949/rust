# Knowledge Base

Curated advanced sample library for LLM inspiration.

## Structure

- `_manifest.json` — list of all entries (`{version, entries: [{id, path}...]}`)
- `example-schema.json` — JSON Schema validating each entry
- `examples/` — entry files, one JSON per sample

## Adding an Entry

1. Create `examples/<category>/<name>.json` following `example-schema.json`
2. Add `{"id": "<category>/<name>", "path": "examples/<category>/<name>.json"}` to `_manifest.json`
3. Run `cargo build -p adaptive-card-core` — build script validates and embeds

## Entry Format

```json
{
  "id": "finance/expense-approval",
  "title": "Expense Approval with Line Items",
  "description": "Multi-line expense with approver action and comment field",
  "category": "finance",
  "tags": ["expense", "approval", "finance", "indonesian:persetujuan"],
  "use_cases": [
    "Employee submits expense for manager approval",
    "Multi-line item breakdown with total"
  ],
  "host_targets": ["teams", "outlook"],
  "complexity": "advanced",
  "card": { "type": "AdaptiveCard", "version": "1.6", "body": [], "actions": [] },
  "notes": "Uses ColumnSet for line items, Action.Execute for submit"
}
```
