# Adaptive Cards — Complete Element Reference (v1.6+)

> **Purpose**: LLM knowledge base for generating Adaptive Cards. Every element, input, and chart type is documented with schema, properties, and copy-ready JSON examples.

---

## Card Envelope

Every card starts with this wrapper:

```json
{
  "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
  "type": "AdaptiveCard",
  "version": "1.5",
  "body": [ /* elements */ ],
  "actions": [ /* top-level actions */ ]
}
```

**Key properties**: `body` (array of elements), `actions` (array of actions), `selectAction` (action on card tap), `fallbackText` (plain text fallback), `lang` (BCP-47 language tag), `speak` (SSML), `verticalContentAlignment` ("top"|"center"|"bottom"), `minHeight` (e.g. "200px"), `refresh` (auto-refresh config), `authentication` (auth config).

---

## 1. ELEMENTS

### 1.1 TextBlock

Displays text with formatting control.

```json
{
  "type": "TextBlock",
  "text": "Hello **bold** and _italic_ text",
  "size": "Medium",
  "weight": "Bolder",
  "color": "Accent",
  "isSubtle": true,
  "wrap": true,
  "maxLines": 3,
  "horizontalAlignment": "Center",
  "fontType": "Monospace",
  "style": "heading",
  "spacing": "Medium"
}
```

| Property | Values | Default |
|----------|--------|---------|
| `text` | string (supports markdown subset: `**bold**`, `_italic_`, `[link](url)`, `\n` newline) | required |
| `size` | `"Small"`, `"Default"`, `"Medium"`, `"Large"`, `"ExtraLarge"` | `"Default"` |
| `weight` | `"Lighter"`, `"Default"`, `"Bolder"` | `"Default"` |
| `color` | `"Default"`, `"Dark"`, `"Light"`, `"Accent"`, `"Good"`, `"Warning"`, `"Attention"` | `"Default"` |
| `fontType` | `"Default"`, `"Monospace"` | `"Default"` |
| `isSubtle` | boolean | false |
| `wrap` | boolean | false |
| `maxLines` | integer | unlimited |
| `horizontalAlignment` | `"Left"`, `"Center"`, `"Right"` | `"Left"` |
| `style` | `"default"`, `"heading"`, `"columnHeader"` | `"default"` |
| `spacing` | `"None"`, `"Small"`, `"Default"`, `"Medium"`, `"Large"`, `"ExtraLarge"`, `"Padding"` | `"Default"` |
| `separator` | boolean — draws line above element | false |

---

### 1.2 RichTextBlock

Inline-level rich text with multiple runs. Supports mixed formatting within a single paragraph.

```json
{
  "type": "RichTextBlock",
  "inlines": [
    "This is normal text. ",
    {
      "type": "TextRun",
      "text": "This is bold red text.",
      "weight": "Bolder",
      "color": "Attention"
    },
    " And this is ",
    {
      "type": "TextRun",
      "text": "a clickable link",
      "selectAction": {
        "type": "Action.OpenUrl",
        "url": "https://example.com"
      },
      "color": "Accent",
      "italic": true
    },
    " within the same paragraph."
  ]
}
```

**TextRun properties**: `text`, `color`, `size`, `weight`, `fontType`, `highlight` (boolean), `italic` (boolean), `strikethrough` (boolean), `underline` (boolean), `selectAction` (action on click).

---

### 1.3 Image

Displays an image from a URL.

```json
{
  "type": "Image",
  "url": "https://example.com/photo.png",
  "altText": "Description of image",
  "size": "Medium",
  "style": "Person",
  "horizontalAlignment": "Center",
  "selectAction": {
    "type": "Action.OpenUrl",
    "url": "https://example.com"
  },
  "width": "80px",
  "height": "80px",
  "backgroundColor": "#EEEEEE"
}
```

| Property | Values | Notes |
|----------|--------|-------|
| `url` | string (URL) | required |
| `size` | `"Auto"`, `"Stretch"`, `"Small"`, `"Medium"`, `"Large"` | `"Auto"` |
| `style` | `"Default"`, `"Person"` | `"Person"` renders as circle |
| `width` | pixel string e.g. `"50px"` | overrides size |
| `height` | pixel string | overrides size |
| `backgroundColor` | hex color | visible behind transparent images |

---

### 1.4 ImageSet

Displays a collection of images in a gallery layout.

```json
{
  "type": "ImageSet",
  "imageSize": "Medium",
  "images": [
    { "type": "Image", "url": "https://example.com/img1.png", "altText": "Photo 1" },
    { "type": "Image", "url": "https://example.com/img2.png", "altText": "Photo 2" },
    { "type": "Image", "url": "https://example.com/img3.png", "altText": "Photo 3" },
    { "type": "Image", "url": "https://example.com/img4.png", "altText": "Photo 4" }
  ]
}
```

**Properties**: `images` (array of Image), `imageSize` (same values as Image.size).

---

### 1.5 Media

Embeds audio or video content.

```json
{
  "type": "Media",
  "poster": "https://example.com/thumbnail.png",
  "altText": "Training video on data pipelines",
  "sources": [
    {
      "mimeType": "video/mp4",
      "url": "https://example.com/video.mp4"
    },
    {
      "mimeType": "video/webm",
      "url": "https://example.com/video.webm"
    }
  ]
}
```

**Properties**: `sources` (array of `{mimeType, url}`), `poster` (thumbnail URL), `altText`.

---

### 1.6 FactSet

Displays key-value pairs in a compact two-column layout.

```json
{
  "type": "FactSet",
  "facts": [
    { "title": "Status", "value": "Active" },
    { "title": "Department", "value": "Engineering" },
    { "title": "Start Date", "value": "January 15, 2024" },
    { "title": "Manager", "value": "Ahmad Rizki" }
  ]
}
```

**Fact properties**: `title` (string, left column), `value` (string, right column). Both support simple markdown.

---

### 1.7 Container

Groups elements together. Supports styling, visibility toggling, and nesting.

```json
{
  "type": "Container",
  "id": "detailsSection",
  "isVisible": true,
  "style": "emphasis",
  "bleed": true,
  "minHeight": "100px",
  "verticalContentAlignment": "Center",
  "selectAction": {
    "type": "Action.ToggleVisibility",
    "targetElements": ["detailsSection"]
  },
  "items": [
    { "type": "TextBlock", "text": "Container content here" }
  ]
}
```

| Property | Values | Notes |
|----------|--------|-------|
| `style` | `"default"`, `"emphasis"`, `"good"`, `"attention"`, `"warning"`, `"accent"` | Background styling |
| `bleed` | boolean | Extends background to card edges |
| `minHeight` | pixel string | Minimum height |
| `verticalContentAlignment` | `"Top"`, `"Center"`, `"Bottom"` | Vertical alignment of children |
| `isVisible` | boolean | For toggle-based navigation patterns |
| `id` | string | Required for ToggleVisibility targeting |
| `selectAction` | Action | Makes entire container tappable |
| `backgroundImage` | URL string or BackgroundImage object | Background image |

---

### 1.8 ColumnSet & Column

Creates horizontal multi-column layouts. Most versatile layout element.

```json
{
  "type": "ColumnSet",
  "columns": [
    {
      "type": "Column",
      "width": "auto",
      "items": [
        {
          "type": "Image",
          "url": "https://example.com/avatar.png",
          "size": "Small",
          "style": "Person"
        }
      ],
      "verticalContentAlignment": "Center"
    },
    {
      "type": "Column",
      "width": "stretch",
      "items": [
        { "type": "TextBlock", "text": "Jane Smith", "weight": "Bolder" },
        { "type": "TextBlock", "text": "Senior Engineer", "isSubtle": true, "spacing": "None" }
      ]
    },
    {
      "type": "Column",
      "width": "auto",
      "items": [
        { "type": "TextBlock", "text": "Online", "color": "Good", "weight": "Bolder" }
      ],
      "verticalContentAlignment": "Center"
    }
  ]
}
```

**Column.width values**: `"auto"` (shrink to content), `"stretch"` (fill remaining), `"1"` / `"2"` (weighted ratio), `"50px"` (fixed pixel).

**ColumnSet properties**: `columns`, `selectAction`, `style`, `bleed`, `minHeight`, `horizontalAlignment`.

**Column properties**: `width`, `items` (array of elements), `verticalContentAlignment`, `style`, `bleed`, `minHeight`, `backgroundImage`, `selectAction`, `id`, `isVisible`.

---

### 1.9 ActionSet

Renders a group of action buttons inline within the body.

```json
{
  "type": "ActionSet",
  "actions": [
    {
      "type": "Action.Submit",
      "title": "Approve",
      "style": "positive",
      "data": { "action": "approve", "id": "12345" }
    },
    {
      "type": "Action.Submit",
      "title": "Reject",
      "style": "destructive",
      "data": { "action": "reject", "id": "12345" }
    },
    {
      "type": "Action.OpenUrl",
      "title": "View Details",
      "url": "https://example.com/item/12345"
    }
  ]
}
```

---

### 1.10 Table

Displays tabular data with rows and cells. More structured than FactSet for multi-column data.

```json
{
  "type": "Table",
  "gridStyle": "accent",
  "firstRowAsHeader": true,
  "showGridLines": true,
  "columns": [
    { "width": 2 },
    { "width": 1 },
    { "width": 1 },
    { "width": 1 }
  ],
  "rows": [
    {
      "type": "TableRow",
      "style": "accent",
      "cells": [
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Project", "weight": "Bolder" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Status", "weight": "Bolder" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Budget", "weight": "Bolder" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Owner", "weight": "Bolder" }] }
      ]
    },
    {
      "type": "TableRow",
      "cells": [
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "MAP FAS Migration" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "On Track", "color": "Good" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Rp 1.2B" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Bima P." }] }
      ]
    },
    {
      "type": "TableRow",
      "cells": [
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "ANTAM Chatbot" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "At Risk", "color": "Warning" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Rp 680M" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Dewi S." }] }
      ]
    },
    {
      "type": "TableRow",
      "cells": [
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Telkom AI Ops" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Delayed", "color": "Attention" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Rp 450M" }] },
        { "type": "TableCell", "items": [{ "type": "TextBlock", "text": "Reza P." }] }
      ]
    }
  ]
}
```

| Property | Values | Notes |
|----------|--------|-------|
| `gridStyle` | `"default"`, `"accent"`, `"good"`, `"attention"`, `"warning"` | Background style for grid |
| `firstRowAsHeader` | boolean | Styles first row as header |
| `showGridLines` | boolean | Shows border lines |
| `columns` | array of `{ width: number }` | Column width ratios |
| `rows` | array of TableRow | Each row contains cells |
| `horizontalCellContentAlignment` | `"Left"`, `"Center"`, `"Right"` | Default cell alignment |
| `verticalCellContentAlignment` | `"Top"`, `"Center"`, `"Bottom"` | Default cell vertical alignment |

**TableRow**: `type`, `style` (same as Container), `cells` (array of TableCell).
**TableCell**: `type`, `items` (array of elements), `style`, `selectAction`, `minHeight`, `verticalContentAlignment`, `bleed`.

---

### 1.11 Badge

Compact status indicator — ideal for tags, labels, and status chips.

```json
{
  "type": "Badge",
  "text": "Approved",
  "appearance": "filled",
  "style": "good",
  "shape": "rounded",
  "size": "medium",
  "icon": "checkmark"
}
```

| Property | Values | Default |
|----------|--------|---------|
| `text` | string | required |
| `appearance` | `"filled"`, `"tint"` | `"filled"` |
| `style` | `"default"`, `"accent"`, `"good"`, `"attention"`, `"warning"`, `"informative"`, `"subtle"` | `"default"` |
| `shape` | `"square"`, `"rounded"`, `"circular"` | `"rounded"` |
| `size` | `"small"`, `"medium"`, `"large"`, `"extraLarge"` | `"medium"` |
| `icon` | icon name string (Fluent icon) | optional |
| `tooltip` | string | hover text |

**Common Badge patterns**:
```json
{ "type": "Badge", "text": "Active", "style": "good", "appearance": "filled" }
{ "type": "Badge", "text": "Pending", "style": "warning", "appearance": "tint" }
{ "type": "Badge", "text": "Critical", "style": "attention", "appearance": "filled" }
{ "type": "Badge", "text": "New", "style": "informative", "appearance": "tint" }
{ "type": "Badge", "text": "Draft", "style": "subtle", "appearance": "tint" }
```

---

### 1.12 Icon

Renders a Fluent UI icon inline.

```json
{
  "type": "Icon",
  "name": "AlertUrgent",
  "size": "Medium",
  "color": "Attention",
  "style": "Filled"
}
```

| Property | Values |
|----------|--------|
| `name` | Fluent icon name: `"AlertUrgent"`, `"Checkmark"`, `"CheckmarkCircle"`, `"DismissCircle"`, `"Info"`, `"Warning"`, `"Person"`, `"People"`, `"Calendar"`, `"Clock"`, `"Mail"`, `"Chat"`, `"Document"`, `"Folder"`, `"Settings"`, `"Search"`, `"Add"`, `"Edit"`, `"Delete"`, `"Share"`, `"Link"`, `"Attach"`, `"Star"`, `"Heart"`, `"Flag"`, `"Pin"`, `"Location"`, `"Phone"`, `"Video"`, `"Camera"`, `"Mic"`, `"Send"`, `"Save"`, `"Copy"`, `"Clipboard"`, `"Filter"`, `"Sort"`, `"ArrowUp"`, `"ArrowDown"`, `"ArrowLeft"`, `"ArrowRight"`, `"ChevronRight"`, `"MoreHorizontal"`, `"Home"`, `"Globe"`, etc. |
| `size` | `"xxSmall"`, `"xSmall"`, `"Small"`, `"Standard"`, `"Medium"`, `"Large"`, `"xLarge"`, `"xxLarge"` |
| `color` | `"Default"`, `"Dark"`, `"Light"`, `"Accent"`, `"Good"`, `"Warning"`, `"Attention"` |
| `style` | `"Regular"`, `"Filled"` |

---

### 1.13 CompoundButton

A rich button with title, description, and optional icon. Better than plain actions for feature selection or navigation.

```json
{
  "type": "CompoundButton",
  "title": "Submit Expense Report",
  "description": "Upload receipts and submit for reimbursement",
  "icon": {
    "type": "Icon",
    "name": "Receipt",
    "size": "Small",
    "style": "Filled"
  },
  "badge": {
    "type": "Badge",
    "text": "3 pending",
    "style": "warning",
    "appearance": "tint"
  },
  "selectAction": {
    "type": "Action.Submit",
    "data": { "action": "open_expenses" }
  }
}
```

| Property | Values |
|----------|--------|
| `title` | string — primary label |
| `description` | string — secondary description text |
| `icon` | Icon element |
| `badge` | Badge element — displays in corner |
| `selectAction` | Action — triggers on tap |

**Navigation menu pattern using CompoundButton**:
```json
{
  "type": "Container",
  "items": [
    {
      "type": "CompoundButton",
      "title": "Leave Management",
      "description": "Request time off and check your balance",
      "icon": { "type": "Icon", "name": "CalendarLtr", "style": "Filled" },
      "badge": { "type": "Badge", "text": "8 days left", "style": "informative", "appearance": "tint" },
      "selectAction": { "type": "Action.ToggleVisibility", "targetElements": [
        { "elementId": "menu", "isVisible": false },
        { "elementId": "leaveSection", "isVisible": true }
      ]}
    },
    {
      "type": "CompoundButton",
      "title": "IT Support",
      "description": "Report issues or request new equipment",
      "icon": { "type": "Icon", "name": "DesktopToolbox", "style": "Filled" },
      "badge": { "type": "Badge", "text": "1 open ticket", "style": "warning", "appearance": "tint" },
      "selectAction": { "type": "Action.ToggleVisibility", "targetElements": [
        { "elementId": "menu", "isVisible": false },
        { "elementId": "itSection", "isVisible": true }
      ]}
    }
  ]
}
```

---

### 1.14 CodeBlock

Displays formatted code with syntax highlighting.

```json
{
  "type": "CodeBlock",
  "language": "python",
  "startLineNumber": 1,
  "codeSnippet": "import pandas as pd\n\ndf = pd.read_csv('data.csv')\ndf_clean = df.dropna()\nprint(f'Rows: {len(df_clean)}')"
}
```

| Property | Values |
|----------|--------|
| `codeSnippet` | string — the code content (use `\n` for newlines) |
| `language` | `"python"`, `"javascript"`, `"typescript"`, `"json"`, `"yaml"`, `"xml"`, `"html"`, `"css"`, `"sql"`, `"bash"`, `"powershell"`, `"csharp"`, `"java"`, `"go"`, `"rust"`, `"cpp"`, `"ruby"`, `"php"`, `"swift"`, `"kotlin"`, `"dockerfile"`, `"markdown"` |
| `startLineNumber` | integer — first line number displayed |

---

### 1.15 ProgressBar

Horizontal progress indicator with label and value.

```json
{
  "type": "ProgressBar",
  "value": 0.72,
  "label": "Sprint Progress",
  "color": "accent"
}
```

| Property | Values | Notes |
|----------|--------|-------|
| `value` | number 0.0–1.0 | required — percentage as decimal |
| `label` | string | text label above bar |
| `color` | `"accent"`, `"good"`, `"warning"`, `"attention"` | bar color |

**Multiple progress bars pattern**:
```json
{
  "type": "Container",
  "items": [
    { "type": "ProgressBar", "label": "Bronze Layer", "value": 1.0, "color": "good" },
    { "type": "ProgressBar", "label": "Silver Layer", "value": 0.85, "color": "good" },
    { "type": "ProgressBar", "label": "Gold Layer", "value": 0.45, "color": "warning" },
    { "type": "ProgressBar", "label": "Gold Mart", "value": 0.1, "color": "attention" }
  ]
}
```

---

### 1.16 ProgressRing

Circular/ring-shaped progress indicator. Better for compact status displays.

```json
{
  "type": "ProgressRing",
  "value": 0.87,
  "label": "Utilization Rate"
}
```

| Property | Values |
|----------|--------|
| `value` | number 0.0–1.0 (omit for indeterminate/spinner) |
| `label` | string |

**Indeterminate (loading spinner)**:
```json
{ "type": "ProgressRing", "label": "Loading..." }
```

---

### 1.17 Rating

Displays a star/icon rating (read-only display, not input).

```json
{
  "type": "Rating",
  "value": 4.5,
  "max": 5,
  "count": 218,
  "size": "Medium",
  "color": "Marigold"
}
```

| Property | Values | Default |
|----------|--------|---------|
| `value` | number | required |
| `max` | number | 5 |
| `count` | number | optional — shows "(count)" label |
| `size` | `"Small"`, `"Medium"`, `"Large"` | `"Medium"` |
| `color` | `"Neutral"`, `"Marigold"` | `"Marigold"` |

---

## 2. INPUTS

### 2.1 Input.Text

Single-line or multi-line text input.

```json
{
  "type": "Input.Text",
  "id": "user_comment",
  "label": "Your comment",
  "placeholder": "Enter your feedback here...",
  "isMultiline": true,
  "maxLength": 500,
  "isRequired": true,
  "errorMessage": "Please enter a comment.",
  "value": "",
  "style": "Text",
  "regex": "^[a-zA-Z0-9 ]+$"
}
```

| Property | Values | Notes |
|----------|--------|-------|
| `id` | string | required — unique identifier for data binding |
| `label` | string | label text above input |
| `placeholder` | string | ghost text |
| `isMultiline` | boolean | renders textarea |
| `maxLength` | integer | character limit |
| `isRequired` | boolean | validation |
| `errorMessage` | string | shown on validation failure |
| `value` | string | default value |
| `style` | `"Text"`, `"Tel"`, `"Url"`, `"Email"`, `"Password"` | input type hint |
| `regex` | string | regex pattern for validation |
| `separator` | boolean | line above |
| `spacing` | spacing value | gap above |

---

### 2.2 Input.Number

Numeric input with min/max bounds.

```json
{
  "type": "Input.Number",
  "id": "quantity",
  "label": "Quantity",
  "placeholder": "Enter amount",
  "min": 1,
  "max": 100,
  "value": 1,
  "isRequired": true,
  "errorMessage": "Enter a number between 1 and 100."
}
```

---

### 2.3 Input.Date

Date picker.

```json
{
  "type": "Input.Date",
  "id": "start_date",
  "label": "Start Date",
  "value": "2026-04-12",
  "min": "2026-01-01",
  "max": "2026-12-31",
  "isRequired": true,
  "errorMessage": "Please select a date."
}
```

**Value format**: `YYYY-MM-DD` string.

---

### 2.4 Input.Time

Time picker.

```json
{
  "type": "Input.Time",
  "id": "meeting_time",
  "label": "Meeting Time",
  "value": "09:30",
  "min": "08:00",
  "max": "18:00",
  "isRequired": true
}
```

**Value format**: `HH:MM` (24-hour).

---

### 2.5 Input.Toggle

Boolean on/off switch.

```json
{
  "type": "Input.Toggle",
  "id": "agree_terms",
  "title": "I agree to the terms and conditions",
  "value": "false",
  "valueOn": "true",
  "valueOff": "false",
  "isRequired": true,
  "errorMessage": "You must accept the terms."
}
```

| Property | Values | Notes |
|----------|--------|-------|
| `title` | string | label text beside toggle |
| `value` | string | default: matches valueOn or valueOff |
| `valueOn` | string | value when toggled on |
| `valueOff` | string | value when toggled off |
| `wrap` | boolean | wraps title text |

---

### 2.6 Input.ChoiceSet

Dropdown, radio buttons, or checkbox group.

**Dropdown (default)**:
```json
{
  "type": "Input.ChoiceSet",
  "id": "department",
  "label": "Department",
  "value": "engineering",
  "isRequired": true,
  "choices": [
    { "title": "Engineering", "value": "engineering" },
    { "title": "Product", "value": "product" },
    { "title": "Design", "value": "design" },
    { "title": "Operations", "value": "operations" }
  ]
}
```

**Expanded radio buttons**:
```json
{
  "type": "Input.ChoiceSet",
  "id": "priority",
  "label": "Priority",
  "style": "expanded",
  "value": "medium",
  "choices": [
    { "title": "High", "value": "high" },
    { "title": "Medium", "value": "medium" },
    { "title": "Low", "value": "low" }
  ]
}
```

**Multi-select checkboxes**:
```json
{
  "type": "Input.ChoiceSet",
  "id": "skills",
  "label": "Select skills",
  "isMultiSelect": true,
  "style": "expanded",
  "value": "python,sql",
  "choices": [
    { "title": "Python", "value": "python" },
    { "title": "SQL", "value": "sql" },
    { "title": "Rust", "value": "rust" },
    { "title": "JavaScript", "value": "javascript" }
  ]
}
```

**Filtered (type-ahead / searchable)**:
```json
{
  "type": "Input.ChoiceSet",
  "id": "country",
  "label": "Country",
  "style": "filtered",
  "choices": [
    { "title": "Indonesia", "value": "ID" },
    { "title": "Japan", "value": "JP" },
    { "title": "United Kingdom", "value": "GB" },
    { "title": "United States", "value": "US" }
  ]
}
```

| Property | Values | Notes |
|----------|--------|-------|
| `style` | `"compact"` (dropdown), `"expanded"` (radio/checkbox), `"filtered"` (searchable) | `"compact"` |
| `isMultiSelect` | boolean | enables multi-select; value is comma-separated |
| `value` | string | default selected value(s); comma-separated for multi |
| `placeholder` | string | ghost text for compact style |
| `wrap` | boolean | wraps choice titles |

**Dynamic data source (for API-populated choices)**:
```json
{
  "type": "Input.ChoiceSet",
  "id": "user_search",
  "label": "Search users",
  "style": "filtered",
  "choices.data": {
    "type": "Data.Query",
    "dataset": "users",
    "count": 25
  }
}
```

---

### 2.7 Input.Rating

Interactive star rating input (unlike Rating element which is display-only).

```json
{
  "type": "Input.Rating",
  "id": "satisfaction_score",
  "label": "How satisfied are you?",
  "max": 5,
  "value": 0,
  "size": "Medium",
  "color": "Marigold",
  "isRequired": true,
  "errorMessage": "Please provide a rating."
}
```

| Property | Values | Default |
|----------|--------|---------|
| `id` | string | required |
| `max` | number (1–10) | 5 |
| `value` | number | 0 (no selection) |
| `size` | `"Small"`, `"Medium"`, `"Large"` | `"Medium"` |
| `color` | `"Neutral"`, `"Marigold"` | `"Marigold"` |

---

## 3. CHARTS

Charts render data visualizations inline in adaptive cards.

### 3.1 Chart.Donut

```json
{
  "type": "Chart.Donut",
  "title": "Budget Allocation",
  "data": [
    { "legend": "Cloud Infrastructure", "value": 45, "color": "accent" },
    { "legend": "Personnel", "value": 30, "color": "good" },
    { "legend": "Software Licenses", "value": 15, "color": "warning" },
    { "legend": "Training", "value": 10, "color": "attention" }
  ]
}
```

---

### 3.2 Chart.Pie

```json
{
  "type": "Chart.Pie",
  "title": "Issue Distribution",
  "data": [
    { "legend": "Bugs", "value": 42, "color": "attention" },
    { "legend": "Features", "value": 35, "color": "accent" },
    { "legend": "Improvements", "value": 18, "color": "good" },
    { "legend": "Documentation", "value": 5, "color": "warning" }
  ]
}
```

---

### 3.3 Chart.Gauge

Single-value gauge/speedometer.

```json
{
  "type": "Chart.Gauge",
  "title": "System Health",
  "value": 87,
  "max": 100,
  "color": "good",
  "segments": [
    { "min": 0, "max": 40, "color": "attention" },
    { "min": 40, "max": 70, "color": "warning" },
    { "min": 70, "max": 100, "color": "good" }
  ]
}
```

**Properties**: `value` (current number), `max` (maximum), `valueLabel` (text under value), `segments` (color ranges).

---

### 3.4 Chart.Line

Line chart with one or more data series.

```json
{
  "type": "Chart.Line",
  "title": "Monthly Revenue (Rp M)",
  "xAxis": {
    "title": "Month",
    "dataType": "string"
  },
  "yAxis": {
    "title": "Revenue (M)"
  },
  "data": [
    {
      "legend": "2025",
      "color": "neutral",
      "values": [
        { "x": "Jan", "y": 120 },
        { "x": "Feb", "y": 135 },
        { "x": "Mar", "y": 148 },
        { "x": "Apr", "y": 142 },
        { "x": "May", "y": 165 },
        { "x": "Jun", "y": 172 }
      ]
    },
    {
      "legend": "2026",
      "color": "accent",
      "values": [
        { "x": "Jan", "y": 155 },
        { "x": "Feb", "y": 168 },
        { "x": "Mar", "y": 182 }
      ]
    }
  ]
}
```

---

### 3.5 Chart.HorizontalBar

Horizontal bar chart.

```json
{
  "type": "Chart.HorizontalBar",
  "title": "Skills Assessment",
  "data": [
    { "legend": "Rust", "value": 92, "color": "accent" },
    { "legend": "Python", "value": 88, "color": "accent" },
    { "legend": "Azure", "value": 85, "color": "good" },
    { "legend": "Databricks", "value": 80, "color": "good" },
    { "legend": "React", "value": 65, "color": "warning" }
  ]
}
```

---

### 3.6 Chart.HorizontalBar.Stacked

Stacked horizontal bar — shows composition per category.

```json
{
  "type": "Chart.HorizontalBar.Stacked",
  "title": "Sprint Story Points by Status",
  "data": [
    {
      "category": "Sprint 12",
      "values": [
        { "legend": "Done", "value": 34, "color": "good" },
        { "legend": "In Progress", "value": 8, "color": "warning" },
        { "legend": "To Do", "value": 0, "color": "neutral" }
      ]
    },
    {
      "category": "Sprint 13",
      "values": [
        { "legend": "Done", "value": 28, "color": "good" },
        { "legend": "In Progress", "value": 12, "color": "warning" },
        { "legend": "To Do", "value": 5, "color": "neutral" }
      ]
    },
    {
      "category": "Sprint 14",
      "values": [
        { "legend": "Done", "value": 15, "color": "good" },
        { "legend": "In Progress", "value": 18, "color": "warning" },
        { "legend": "To Do", "value": 12, "color": "neutral" }
      ]
    }
  ]
}
```

---

### 3.7 Chart.VerticalBar

Vertical bar chart.

```json
{
  "type": "Chart.VerticalBar",
  "title": "Tickets Resolved per Week",
  "xAxis": { "title": "Week" },
  "yAxis": { "title": "Tickets" },
  "data": [
    {
      "legend": "Resolved",
      "color": "good",
      "values": [
        { "x": "W10", "y": 23 },
        { "x": "W11", "y": 31 },
        { "x": "W12", "y": 28 },
        { "x": "W13", "y": 35 },
        { "x": "W14", "y": 42 }
      ]
    }
  ]
}
```

---

### 3.8 Chart.VerticalBar.Grouped

Grouped vertical bar — compare multiple series side by side.

```json
{
  "type": "Chart.VerticalBar.Grouped",
  "title": "Billable vs Non-Billable Hours",
  "xAxis": { "title": "Month" },
  "yAxis": { "title": "Hours" },
  "data": [
    {
      "legend": "Billable",
      "color": "good",
      "values": [
        { "x": "Jan", "y": 142 },
        { "x": "Feb", "y": 135 },
        { "x": "Mar", "y": 158 }
      ]
    },
    {
      "legend": "Non-Billable",
      "color": "warning",
      "values": [
        { "x": "Jan", "y": 28 },
        { "x": "Feb", "y": 35 },
        { "x": "Mar", "y": 22 }
      ]
    }
  ]
}
```

---

## 4. ACTIONS

### 4.1 Action.Submit

Sends input data to the bot/backend.

```json
{
  "type": "Action.Submit",
  "title": "Submit",
  "style": "positive",
  "tooltip": "Submit the form",
  "isEnabled": true,
  "data": {
    "action": "submit_form",
    "formType": "leave_request",
    "demo_mode": true
  }
}
```

| Property | Values |
|----------|--------|
| `style` | `"default"`, `"positive"`, `"destructive"` |
| `data` | object — merged with all input values on submit |
| `associatedInputs` | `"auto"`, `"none"` — whether to include input values |
| `isEnabled` | boolean |
| `tooltip` | string |

---

### 4.2 Action.OpenUrl

Opens a URL in the browser.

```json
{
  "type": "Action.OpenUrl",
  "title": "View Documentation",
  "url": "https://docs.example.com",
  "tooltip": "Opens in browser"
}
```

---

### 4.3 Action.ToggleVisibility

Shows/hides elements by ID. Core pattern for multi-step forms and navigation.

```json
{
  "type": "Action.ToggleVisibility",
  "title": "Next Step",
  "targetElements": [
    { "elementId": "step1", "isVisible": false },
    { "elementId": "step2", "isVisible": true }
  ]
}
```

**Simple toggle** (toggles between visible/hidden):
```json
{
  "type": "Action.ToggleVisibility",
  "title": "Show Details",
  "targetElements": ["detailSection"]
}
```

---

### 4.4 Action.ShowCard

Reveals an inline sub-card below the button.

```json
{
  "type": "Action.ShowCard",
  "title": "Add Comment",
  "card": {
    "type": "AdaptiveCard",
    "body": [
      {
        "type": "Input.Text",
        "id": "inline_comment",
        "label": "Comment",
        "isMultiline": true,
        "placeholder": "Write your comment..."
      }
    ],
    "actions": [
      {
        "type": "Action.Submit",
        "title": "Post",
        "style": "positive",
        "data": { "action": "add_comment" }
      }
    ]
  }
}
```

---

### 4.5 Action.Execute

Universal action for bots — supports refresh, authentication, and server-side processing.

```json
{
  "type": "Action.Execute",
  "title": "Refresh Data",
  "verb": "refreshDashboard",
  "data": {
    "dashboardId": "kpi_001"
  },
  "associatedInputs": "auto"
}
```

| Property | Values |
|----------|--------|
| `verb` | string — action identifier for the bot |
| `data` | object — payload |
| `associatedInputs` | `"auto"`, `"none"` |

**IMPORTANT for multi-card flows**: When using Action.Execute for navigation between cards in a flow, ALWAYS include `"nextCardId"` in the `data` object so the frontend can extract edges for the graph view:

```json
{
  "type": "Action.Execute",
  "title": "Book a Trip",
  "verb": "navigate",
  "data": { "nextCardId": "book_trip" }
}
```

Both `Action.Submit` and `Action.Execute` work for navigation — just ensure `data.nextCardId` is present.

---

## 5. COMMON PROPERTIES (all elements)

These properties are available on most elements:

| Property | Type | Notes |
|----------|------|-------|
| `id` | string | Unique identifier (required for ToggleVisibility targets and inputs) |
| `isVisible` | boolean | Show/hide element |
| `separator` | boolean | Draws horizontal line above |
| `spacing` | string | Gap above: `"None"`, `"Small"`, `"Default"`, `"Medium"`, `"Large"`, `"ExtraLarge"`, `"Padding"` |
| `height` | string | `"auto"`, `"stretch"` |
| `fallback` | element or `"drop"` | Fallback for unsupported elements |
| `requires` | object | Feature requirements check |

---

## 6. DESIGN PATTERNS

### 6.1 Multi-Step Wizard

Use Container + ToggleVisibility for step-by-step flows:

```json
{
  "type": "AdaptiveCard",
  "version": "1.5",
  "body": [
    {
      "type": "Container",
      "id": "step1",
      "isVisible": true,
      "items": [
        { "type": "TextBlock", "text": "Step 1 of 3", "weight": "Bolder" },
        { "type": "Input.Text", "id": "name", "label": "Name" },
        {
          "type": "ActionSet",
          "actions": [{
            "type": "Action.ToggleVisibility",
            "title": "Next",
            "style": "positive",
            "targetElements": [
              { "elementId": "step1", "isVisible": false },
              { "elementId": "step2", "isVisible": true }
            ]
          }]
        }
      ]
    },
    {
      "type": "Container",
      "id": "step2",
      "isVisible": false,
      "items": [
        { "type": "TextBlock", "text": "Step 2 of 3", "weight": "Bolder" },
        { "type": "Input.Date", "id": "date", "label": "Date" },
        {
          "type": "ActionSet",
          "actions": [
            {
              "type": "Action.ToggleVisibility",
              "title": "Back",
              "targetElements": [
                { "elementId": "step2", "isVisible": false },
                { "elementId": "step1", "isVisible": true }
              ]
            },
            {
              "type": "Action.ToggleVisibility",
              "title": "Next",
              "style": "positive",
              "targetElements": [
                { "elementId": "step2", "isVisible": false },
                { "elementId": "step3", "isVisible": true }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### 6.2 Status List Item (reusable row)

```json
{
  "type": "ColumnSet",
  "columns": [
    {
      "type": "Column",
      "width": "stretch",
      "items": [
        { "type": "TextBlock", "text": "Item Title", "weight": "Bolder" },
        { "type": "TextBlock", "text": "Subtitle or metadata", "size": "Small", "isSubtle": true, "spacing": "None", "wrap": true }
      ]
    },
    {
      "type": "Column",
      "width": "auto",
      "items": [
        { "type": "Badge", "text": "Active", "style": "good", "appearance": "tint" }
      ],
      "verticalContentAlignment": "Center"
    }
  ]
}
```

### 6.3 KPI Metric Card

```json
{
  "type": "Column",
  "width": "1",
  "items": [
    { "type": "TextBlock", "text": "Label", "size": "Small", "isSubtle": true, "horizontalAlignment": "Center" },
    { "type": "TextBlock", "text": "87%", "size": "ExtraLarge", "weight": "Bolder", "color": "Good", "horizontalAlignment": "Center", "spacing": "None" },
    { "type": "TextBlock", "text": "Target: 80%", "size": "Small", "isSubtle": true, "horizontalAlignment": "Center", "spacing": "None" }
  ]
}
```

### 6.4 User Profile Row

```json
{
  "type": "ColumnSet",
  "columns": [
    {
      "type": "Column",
      "width": "auto",
      "items": [
        { "type": "Image", "url": "https://example.com/avatar.png", "size": "Small", "style": "Person" }
      ],
      "verticalContentAlignment": "Center"
    },
    {
      "type": "Column",
      "width": "stretch",
      "items": [
        { "type": "TextBlock", "text": "Bima Pangestu", "weight": "Bolder" },
        { "type": "TextBlock", "text": "Head of Delivery", "size": "Small", "isSubtle": true, "spacing": "None" }
      ]
    }
  ]
}
```

### 6.5 Chart Color Reference

All chart elements support these color values:
`"accent"`, `"good"`, `"warning"`, `"attention"`, `"neutral"`, `"categoricalRed"`, `"categoricalPurple"`, `"categoricalLavender"`, `"categoricalBlue"`, `"categoricalNavy"`, `"categoricalTeal"`, `"categoricalForest"`, `"categoricalBerry"`, `"categoricalMarigold"`, `"categoricalLilac"`, `"categoricalMink"`.

---

## 7. TEMPLATING & DATA BINDING

Adaptive Cards support data binding with `${}` syntax:

```json
{
  "type": "TextBlock",
  "text": "Hello ${employee.name}, your balance is ${leave.remaining} days."
}
```

**Conditional visibility**:
```json
{
  "type": "Container",
  "$when": "${status == 'urgent'}",
  "style": "attention",
  "items": [
    { "type": "TextBlock", "text": "This item requires immediate attention!" }
  ]
}
```

**Repeating data with $data**:
```json
{
  "type": "Container",
  "$data": "${tickets}",
  "items": [
    { "type": "TextBlock", "text": "${title}", "weight": "Bolder" },
    { "type": "TextBlock", "text": "${status} · ${priority}", "size": "Small", "isSubtle": true }
  ]
}
```

---

## 8. REFRESH & AUTHENTICATION

**Auto-refresh** (card refreshes periodically):
```json
{
  "type": "AdaptiveCard",
  "version": "1.5",
  "refresh": {
    "action": {
      "type": "Action.Execute",
      "verb": "refreshCard"
    },
    "userIds": ["user123"],
    "expires": "2026-04-12T23:59:59Z"
  },
  "body": []
}
```

**Authentication config**:
```json
{
  "type": "AdaptiveCard",
  "version": "1.5",
  "authentication": {
    "text": "Please sign in to continue",
    "connectionName": "myOAuthConnection",
    "tokenExchangeResource": {
      "id": "token-exchange-id",
      "uri": "api://botid-xxx/token"
    },
    "buttons": [
      { "type": "signin", "title": "Sign In", "value": "https://auth.example.com" }
    ]
  },
  "body": []
}
```

---

*Generated for Greentic adaptive cards knowledge base. Covers Adaptive Cards schema v1.5+ with extended v1.6 elements (Badge, Icon, CompoundButton, CodeBlock, ProgressBar, ProgressRing, Rating, Input.Rating, Charts, Table).*
