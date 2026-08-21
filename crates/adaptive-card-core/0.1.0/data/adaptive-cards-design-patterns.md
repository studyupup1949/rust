# Greentic Adaptive Cards — Annotated Knowledge Base Entries

## Purpose
This document contains annotated KB entries for 23 modern adaptive card examples.
Each entry includes metadata, design notes explaining WHY the design works, and
visual patterns used. The agent should use these as reference examples (palette,
not copy) when generating new card flows.

## How to Use
1. Each entry has an `id`, `tags`, `title`, `use_case`, and `description`
2. `design_notes` explain the creative tricks — this is the key learning
3. `visual_patterns_used` list reusable composition patterns
4. Card JSON follows each annotation block

## KB Entry Format for Embedding
When embedding in the system prompt, use this JSON wrapper per entry:
```json
{
  "id": "...",
  "tags": ["..."],
  "title": "...",
  "use_case": "...",
  "description": "...",
  "design_notes": ["..."],
  "visual_patterns_used": ["..."],
  "card": { /* full card JSON */ }
}
```

---

## Entry 1: Events Timeline

**ID:** `events-timeline`
**Tags:** `events, timeline, schedule, calendar, agenda, chronological`
**Title:** Upcoming Events Timeline
**Use Case:** Display a chronological list of events with a continuous visual timeline connector
**Description:** Vertical timeline using SVG background trick, event thumbnails as Container backgrounds, data-bound repeating event blocks with CTAs

### Design Notes
1. **TIMELINE TRICK:** Left Column (width `40px`) uses `backgroundImage` with an SVG and `fillMode: "RepeatVertically"` — this creates a continuous vertical line connecting all events without any custom rendering
2. **THUMBNAIL:** Container with `backgroundImage` (minHeight `180px`) instead of Image element — gives consistent sizing regardless of image aspect ratio
3. **RIGHT PADDING:** Empty Column (width `16px`) — creates breathing room on the right edge, prevents content feeling cramped against card border
4. **DATA BINDING:** `$data` on ColumnSet repeats the entire event block per item — the whole ColumnSet (timeline + content + padding) repeats as a unit
5. **VISUAL HIERARCHY:** title (Large Bold) → thumbnail → date (Accent Bold) → description (Small Subtle) → CTA button. Each level is visually distinct
6. **SPACER:** Empty Container (minHeight `4px`) at top creates subtle spacing before first event

### Visual Patterns Used
`timeline_connector`, `thumbnail_container`, `data_repeat`, `padding_column`, `visual_hierarchy`

---

## Entry 2: Social News Feed with Interactions

**ID:** `social-news-feed`
**Tags:** `news, social, article, feed, like, comment, blog, intranet`
**Title:** Social News Feed with Like & Comment
**Use Case:** Display a news article with social interaction buttons (like toggle, comment toggle)
**Description:** Article card with author avatar, title, excerpt, like counter with toggle animation, and expandable comment section

### Design Notes
1. **ARTICLE LAYOUT:** 4:3 ColumnSet ratio — text-heavy left column (width `4`) with content, right column (width `3`) as full backgroundImage. This creates a magazine-style layout
2. **AUTHOR ROW:** Nested ColumnSet with Person-style avatar (auto width) + author name (stretch width). verticalContentAlignment Center keeps them aligned
3. **LIKE TOGGLE TRICK:** Two Image elements (normal + clicked state) with one `isVisible: false`. ToggleVisibility swaps both images AND both like count TextBlocks simultaneously — creates instant visual feedback without server call
4. **INLINE SVG:** Like button uses base64-encoded SVG as Image source — no external image dependency, renders consistently everywhere
5. **COMMENT SECTION:** Hidden Container (style `emphasis`) with Input.Text + Submit button, toggled by Comment button. emphasis style creates visual separation from main content
6. **SELECTABLE ROW:** `selectAction` on entire ColumnSet makes the whole article area clickable to open full article

### Visual Patterns Used
`magazine_layout`, `author_row`, `toggle_interaction`, `inline_svg`, `expandable_section`, `selectable_row`

---

## Entry 3: Approval Request (Split Layout)

**ID:** `approval-split-layout`
**Tags:** `approval, leave, PTO, request, HR, two-column`
**Title:** Approval Request — Split Panel Layout
**Use Case:** Display a leave/PTO approval request with rich two-panel design
**Description:** 35/65 split layout with branded left panel (accent + emphasis containers) and content right panel with requester info, date range, and approve/decline actions

### Design Notes
1. **SPLIT PANEL:** ColumnSet with width `35` and `65` ratio — creates an asymmetric two-panel layout. Left panel is decorative/branding, right panel is content
2. **LEFT PANEL DEPTH:** Two stacked containers in left column — top Container (style `accent` + bleed + backgroundImage) holds approval type icon, bottom Container (style `emphasis` + bleed) holds title and app info. Different styles create visual layering
3. **ICON + APP ROW:** Nested ColumnSet in left panel with small app icon (width `29px`) + "From ${app}" text — shows source system context
4. **DATE RANGE VISUAL:** Three columns with from-date → arrow image → to-date, using Image element for the arrow. More visual than text-only date display
5. **FACTSET FOR DETAILS:** FactSet used for supplementary info (allowance, staff on leave) — appropriate use for key-value metadata, not as primary content
6. **POSITIVE/DESTRUCTIVE ACTIONS:** Approve button with `style: "positive"` and Decline with `style: "destructive"` — semantic action styling communicates intent
7. **FULL HEIGHT:** `height: "stretch"` on Columns and Containers — ensures left panel extends full height regardless of right panel content length

### Visual Patterns Used
`split_panel`, `layered_containers`, `icon_label_row`, `date_range_visual`, `semantic_actions`, `full_height_columns`

---

## Entry 4: Employee Praise & Anniversaries

**ID:** `employee-praise`
**Tags:** `employee, praise, anniversary, HR, recognition, celebration, team`
**Title:** Employee Praise & Anniversaries Dashboard
**Use Case:** Show upcoming work anniversaries and received praise from colleagues
**Description:** Three-section card: hero celebration image, anniversary list with Person avatars, and praise cards with award details in emphasis containers

### Design Notes
1. **HERO IMAGE:** Large base64-encoded celebration image (container with accent style) centered as visual anchor — draws attention immediately
2. **CTA IN HERO:** ActionSet directly below hero with encouraging action ("Send praise to a colleague 🥳") — one of few cases where emoji in CTA is appropriate (celebration context)
3. **DYNAMIC COUNT:** `${string(count(anniversaries))}` in heading — shows list length dynamically, sets user expectation
4. **PERSON LIST:** `$data` on ColumnSet repeats employee rows — each row has Person-style avatar (50px column), name+years info (width 20), date (width 20), forward arrow (20px column)
5. **FORWARD ARROW:** Small arrow image in last column signals "tappable" / "more details" — a subtle affordance without explicit buttons
6. **SECTION SPACING:** Empty Container (minHeight `16px`, spacing None) between sections — creates consistent visual breaks without relying on spacing property alone
7. **PRAISE CONTAINER:** Each praise item wrapped in Container with style `emphasis` and spacing `Large` — visually separates praise cards from each other
8. **NESTED AVATAR ROW:** Within praise section, ColumnSet with small Person avatar (25px) + sender name — consistent author attribution pattern

### Visual Patterns Used
`hero_image`, `dynamic_count`, `person_list`, `forward_arrow_affordance`, `spacer_container`, `emphasis_card`, `author_row`

---

## Entry 5: Expense Report Approval

**ID:** `expense-report-approval`
**Tags:** `expense, approval, finance, report, table, expandable, workflow`
**Title:** Expense Report Approval with Line Item Details
**Use Case:** Display expense report for approval with expandable line items, totals, history, and approve/reject workflow
**Description:** Complex multi-section card: status header, expense summary with FactSet, data-bound line items with expand/collapse, financial totals, toggleable history, and approve/reject actions with inline reject comment

### Design Notes
1. **STATUS HEADER:** Container style `emphasis` with bleed — ColumnSet shows approval type (left, isSubtle) and status (right, color Attention) — immediately communicates card purpose and urgency
2. **INLINE ACTION BUTTON:** "SEND TO MY INBOX" ActionSet in same ColumnSet as title — action at top for quick access without scrolling
3. **FACTSET FOR METADATA:** FactSet with bold markdown in values (`**${submitted.name}**`) — structured metadata display, appropriate for key-value pairs
4. **TABLE HEADER:** Container style `emphasis` + bleed with ColumnSet containing DATE/CATEGORY/AMOUNT headers in Bold — simulates table header row
5. **EXPAND/COLLAPSE PATTERN:** Each line item has a chevron column with selectAction ToggleVisibility — swaps chevron-down/chevron-up images and shows/hides detail container. Uses consistent IDs (`cardContent1`, `chevronDown1`, `chevronUp1`)
6. **INLINE COMMENT:** Hidden detail container has description text (isSubtle) + Input.Text for comments + Send ActionSet — allows per-line-item comments
7. **FINANCIAL SUMMARY:** Right-aligned ColumnSet for total/non-reimbursable/advance amounts — Attention color for deductions. Final reimbursable amount in emphasis Container with Bold weight
8. **HISTORY TOGGLE:** "Show history"/"Hide history" TextBlocks (color Accent) toggled via selectAction — history Container hidden by default. Accent color signals interactive text
9. **SHOWCARD FOR REJECT:** Action.ShowCard with `style: "destructive"` reveals inline reject comment form — keeps reject workflow in-card without navigation
10. **DATA BINDING:** `$data` on Container repeats line items — each with its own expand/collapse state

### Visual Patterns Used
`status_header`, `table_simulation`, `expand_collapse_chevron`, `inline_comment`, `financial_summary`, `history_toggle`, `showcard_form`, `data_repeat`

---

## Entry 6: Warehouse Inventory Dashboard

**ID:** `warehouse-inventory`
**Tags:** `inventory, warehouse, dashboard, KPI, metrics, products, stock`
**Title:** Warehouse Inventory Dashboard
**Use Case:** Display warehouse inventory KPIs (available/ready stock) and top-selling product list
**Description:** Location header, dual KPI boxes with change indicators, and product list with thumbnails

### Design Notes
1. **CONTEXTUAL HEADER:** Warehouse name (ExtraLarge Bold) + live timestamp using `formatDateTime(utcNow(), 'HH:mm')` + location — shows freshness of data
2. **DUAL KPI BOXES:** Two equal-width Columns (width `48` each) with style `emphasis` — each shows label (Small Bold isSubtle), big number (ExtraLarge Bold), and change indicator. Empty Column (width `4`) creates gap between boxes
3. **CHANGE INDICATOR:** Nested ColumnSet with style `good` — contains up arrow emoji (28px column) + percentage change text (stretch column). `good` style container creates green background for positive change
4. **PRODUCT LIST:** `$data` on ColumnSet repeats product rows — Image element (48x48, centered) | product name + SKU/change info | forward arrow. Separator between rows
5. **CHANGE COLOR:** Product change value uses `color: "Accent"` — draws attention to performance metrics
6. **FORWARD ARROW:** Small arrow image (20px) in rightmost column signals drilldown — consistent with person list pattern
7. **NUMBER FORMATTING:** `formatNumber(available.amount,2)` + "M" suffix — proper number formatting for large values

### Visual Patterns Used
`live_timestamp`, `dual_kpi_boxes`, `change_indicator`, `product_list`, `forward_arrow_affordance`, `number_formatting`

---

## Entry 7: Stock Price Widget

**ID:** `stock-price`
**Tags:** `stock, finance, price, market, ticker, widget, compact`
**Title:** Stock Price Widget
**Use Case:** Display current stock price with change indicator
**Description:** Compact stock ticker with conditional up/down/unchanged icon, price display, and color-coded change percentage

### Design Notes
1. **CONDITIONAL ICONS:** Three Image elements with `$when` expressions (`${Change > 0}`, `${Change < 0}`, `${Change == 0}`) — only one renders based on data. Different icon for each state
2. **PRICE DISPLAY:** Large ticker symbol (ExtraLarge, color Dark, isSubtle) + current price (ExtraLarge) — clear visual hierarchy between symbol and price
3. **COLOR-CODED CHANGE:** Two TextBlock elements with `$when` — positive change shows `color: "Good"`, negative shows `color: "Attention"`. Only one renders
4. **CENTERED LAYOUT:** All columns use `verticalContentAlignment: "Center"` — keeps icon, symbol, and price vertically aligned
5. **REFRESH TIMESTAMP:** "Last refreshed: X minutes ago" using `formatDateTime(utcNow(),'mm')` — builds trust in data freshness
6. **MINIMAL CONTAINER:** Container with minHeight `100px` and verticalContentAlignment Center — ensures minimum visual presence even with compact content

### Visual Patterns Used
`conditional_rendering`, `color_coded_status`, `centered_layout`, `live_timestamp`, `compact_widget`

---

## Entry 8: Payslip Viewer

**ID:** `payslip-viewer`
**Tags:** `payslip, salary, pay, HR, earnings, deductions, taxes, finance`
**Title:** Payslip Viewer with Period Navigation
**Use Case:** Display detailed payslip breakdown with earnings, deductions, taxes, and period navigation
**Description:** Centered date header, highlighted net pay section, data-bound earnings/deductions/taxes breakdowns, and previous/next period navigation buttons

### Design Notes
1. **CENTERED HEADER:** Pay date (ExtraLarge Bold, Center) + period covered (Small Bold, Center) — clean, formal document feel
2. **NET PAY HIGHLIGHT:** ColumnSet with style `emphasis` + bleed — "Net Pay" label left, amount right in `color: "Good"`. Emphasis container makes this the visual anchor of the entire card
3. **DATA-BOUND SECTIONS:** Three identical section patterns for earnings/deductions/taxes — each has section header ColumnSet (Bold label + Bold total) followed by `$data` ColumnSet for line items. Separator between sections
4. **RIGHT-ALIGNED AMOUNTS:** All amount TextBlocks use `horizontalAlignment: "Right"` — creates clean column alignment for financial data
5. **PERIOD NAVIGATION:** Two Columns with ActionSet containing Submit buttons — "⇦ ${previous_period}" left and "${next_period} ⇨" right. Arrow emoji in button text for direction indication
6. **SECTION SEPARATORS:** `separator: true` + `spacing: "ExtraLarge"` between deductions and taxes sections — clear visual breaks for different financial categories
7. **CONSISTENT SPACING:** Line items use `spacing: "Small"` — keeps items tight within sections while sections have larger spacing between them

### Visual Patterns Used
`centered_header`, `emphasis_highlight`, `data_bound_sections`, `right_aligned_amounts`, `period_navigation`, `section_separators`

---

## Entry 9: Public Holidays Calendar

**ID:** `public-holidays`
**Tags:** `holidays, calendar, dates, schedule, HR, time-off`
**Title:** Public Holidays Calendar
**Use Case:** Display upcoming public holidays with next holiday highlighted
**Description:** Highlighted next holiday in good-style container, followed by year-grouped holiday lists with data binding

### Design Notes
1. **NEXT HOLIDAY HIGHLIGHT:** Container with style `good` — three centered TextBlocks (intro text, date ExtraLarge Bold, holiday name Bold). Green background makes it the visual anchor, communicates positive/upcoming event
2. **YEAR HEADERS:** TextBlock with Bold Large weight + ExtraLarge spacing — clear section breaks between years
3. **DATA-BOUND LISTS:** `$data` on ColumnSet for each year — two columns (name stretch, date stretch right-aligned) with separator between rows. Simple, scannable format
4. **SPEAK ATTRIBUTE:** `speak` property for accessibility — "The next public holiday is ${next_holiday} on ${next_holiday_date}"
5. **MINIMAL DESIGN:** No images, no icons, no complex layouts — pure text hierarchy with Container styles for emphasis. Proves that good design doesn't require visual complexity

### Visual Patterns Used
`status_container`, `year_grouped_lists`, `data_repeat`, `accessibility_speak`, `minimal_text_hierarchy`

---

## Entry 10: Lead/Content Cards with Actions

**ID:** `lead-content-cards`
**Tags:** `leads, CRM, content, articles, approval, messaging, list`
**Title:** Lead/Content Cards with Inline Actions
**Use Case:** Display content items with thumbnail, author info, and multiple inline actions (view details, approve, send message)
**Description:** Repeating content cards with backgroundImage thumbnails, toggle-based detail expansion, inline message input, and FactSet for metadata

### Design Notes
1. **CONTENT LAYOUT:** ColumnSet with stretch text column + 120px backgroundImage column (minHeight 160px) — consistent magazine-style layout with reliable thumbnail sizing
2. **AUTHOR ROW:** Nested ColumnSet with Person avatar (25px, auto width) + "Published by" text (Small isSubtle) — compact attribution line
3. **SELECTABLE ROW:** `selectAction` on ColumnSet (Action.OpenUrl) — entire card area is clickable for full content view
4. **MULTI-ACTION ROW:** ActionSet with three horizontal actions — View details (ToggleVisibility), Quick Approve (Submit), Send Message (ToggleVisibility, style positive). Different action types for different workflows
5. **TOGGLEABLE FACTSET:** FactSet with `isVisible: false` shown via ToggleVisibility — progressive disclosure of detailed metadata
6. **INLINE MESSAGE INPUT:** Input.Text (isMultiline, isVisible false) with `inlineAction` Submit button — message composition without navigation
7. **DATA REPEAT:** `$data` on Container repeats entire card blocks — each with ExtraLarge spacing for clear separation
8. **INDEX-BASED IDS:** `{$index}` in toggle target IDs (`lead-details-toggle{$index}`, `lead-message-toggle{$index}`) — ensures each card's toggles are independent

### Visual Patterns Used
`magazine_layout`, `thumbnail_container`, `author_row`, `selectable_row`, `multi_action_bar`, `progressive_disclosure`, `inline_input`, `data_repeat`

---

## Entry 11: Support Ticket List

**ID:** `support-tickets`
**Tags:** `support, helpdesk, ticket, IT, tracking, comment, list`
**Title:** Support Ticket List with Comments
**Use Case:** Display support tickets with status, assignee, description, and inline comment functionality
**Description:** Repeating ticket cards with emphasis header bar, assignee avatar, description, author info, action buttons, and toggleable comment section

### Design Notes
1. **TICKET HEADER BAR:** ColumnSet with style `emphasis` + bleed — ticket ID + status (left, Bold) and assignee info + Person avatar (right, 100px + 30px columns). Bleed creates full-width colored bar
2. **STATUS EMOJI:** `📣 ${status}` with emoji prefix — quick visual status indicator
3. **DESCRIPTION WITH LIMIT:** TextBlock with `maxLines: 2` + isSubtle — prevents long descriptions from overwhelming the card
4. **AUTHOR TIMESTAMP:** Person avatar (24px) + "opened this ${created_time}" text — compact attribution with temporal context
5. **COMMENT TOGGLE:** Action.ToggleVisibility targets `comment-${id}` — dynamic ID per ticket ensures independent toggle state
6. **INLINE COMMENT FORM:** Hidden Container (style emphasis) with ColumnSet: Input.Text (stretch) + Submit ActionSet (auto width) side by side — compact comment entry
7. **DATA REPEAT ON CONTAINER:** `$data` on outermost Container with spacing Large — each ticket is a distinct visual block

### Visual Patterns Used
`emphasis_header_bar`, `assignee_display`, `maxlines_truncation`, `author_timestamp`, `toggleable_comment`, `inline_form`, `data_repeat`

---

## Entry 12: Meeting Room Booking

**ID:** `meeting-room-booking`
**Tags:** `meeting, room, booking, reservation, calendar, form, facilities`
**Title:** Meeting Room Booking Form
**Use Case:** Book a meeting room with date/time, title, attendees, and facility selection
**Description:** Hero backgroundImage with text overlay, two-column date/time inputs, form fields, multi-select ChoiceSet for facilities, and submit action

### Design Notes
1. **HERO BANNER:** Container with backgroundImage (minHeight 200px, bleed) — verticalContentAlignment Bottom pushes content to bottom of image
2. **TEXT OVERLAY:** ColumnSet with empty stretch Column + 225px Column (style default) containing title — default style on Column creates opaque background behind text for readability
3. **TWO-COLUMN DATE/TIME:** ColumnSet with two stretch columns — each has label (Small) + Input.Date + Input.Time. Logical grouping of from/to
4. **MULTI-SELECT:** Input.ChoiceSet with `isMultiSelect: true` and placeholder text — for facility selection
5. **SINGLE CTA:** One ActionSet with clear action text — singular action after form completion
6. **LABEL PATTERN:** Each input preceded by TextBlock (size Small) with field label — consistent label-above-input pattern

### Visual Patterns Used
`hero_banner`, `text_overlay`, `two_column_form`, `multi_select`, `label_above_input`, `single_cta`

---

## Entry 13: Notification with Quick Actions

**ID:** `notification-quick-actions`
**Tags:** `notification, alert, CRM, reminder, quick-action, dropdown`
**Title:** Notification with Quick Actions Dropdown
**Use Case:** Display a notification/reminder with details and quick action dropdown for immediate response

### Design Notes
1. **HEADER ROW:** ColumnSet with title (stretch, Large Bold) + app name (Accent Bold Small) on left, icon Image (auto, Small) on right — clear source attribution
2. **NOTIFICATION TEXT:** TextBlock with isSubtle + maxLines 3 — prevents verbose notifications from dominating
3. **CLICKABLE DETAILS:** Container with `$data` repeater — each detail item has `selectAction` (Action.OpenUrl)
4. **QUICK ACTION DROPDOWN:** ColumnSet with ChoiceSet (stretch) + Submit ActionSet (auto) — inline action pattern

### Visual Patterns Used
`icon_title_header`, `truncated_notification`, `clickable_detail_links`, `inline_dropdown_action`, `data_repeat`

---

## Entry 14: PTO Balance & Request

**ID:** `pto-balance-request`
**Tags:** `PTO, leave, time-off, balance, request, HR, form, hero`
**Title:** PTO Balance with Inline Request Form
**Use Case:** Show PTO balances and allow requesting PTO without leaving the card

### Design Notes
1. **HERO BANNER:** Container with backgroundImage + verticalContentAlignment Bottom — text overlay via ColumnSet with stretch spacer + styled Column
2. **CONDITIONAL CONTENT:** `$when` expression — only shows if data exists
3. **BALANCE LIST:** `$data` on ColumnSet within emphasis Container — icon + name + balance hours
4. **VIEW/FORM TOGGLE:** Action.ToggleVisibility targets THREE elements — swaps entire card view from dashboard to form
5. **EXPANDED CHOICESET:** Input.ChoiceSet with `style: "expanded"` — renders as radio buttons for important choices

### Visual Patterns Used
`hero_banner`, `text_overlay`, `conditional_content`, `balance_dashboard`, `view_swap_toggle`, `expanded_choiceset`

---

## Entry 15: Safety Alert

**ID:** `safety-alert`
**Tags:** `safety, alert, warning, incident, severity, conditional`
**Title:** Safety Alert with Severity-Based Styling
**Use Case:** Display safety alerts with visual severity indication using conditional container styles

### Design Notes
1. **CONDITIONAL CONTAINERS:** Three Container elements with `$when` — low uses `accent`, medium uses `warning`, high uses `attention`. Only one renders
2. **SEVERITY ICONS:** Different icons per severity — visual differentiation beyond color alone (accessibility)
3. **CONSISTENT LAYOUT:** All conditional containers share identical internal structure
4. **CARD-LEVEL ACTION:** `actions` at card level, not ActionSet in body — renders at bottom with full width

### Visual Patterns Used
`conditional_container_style`, `severity_icons`, `consistent_conditional_layout`, `card_level_actions`

---

## Entry 16: Calendar with Out of Office

**ID:** `calendar-ooo`
**Tags:** `calendar, out-of-office, team, schedule, HR, monthly`
**Title:** Monthly Calendar with Team OOO Display

### Design Notes
1. **MONTH NAVIGATION:** ColumnSet — "Previous" (Accent left), month name (Large Bold center), "Next" (Accent right)
2. **CALENDAR GRID:** Seven ColumnSets with 7 stretch columns — simulates table without Table element
3. **CURRENT WEEK HIGHLIGHT:** One ColumnSet has style `accent` + bleed — colored background for current week
4. **OOO SECTION:** Container style `emphasis` — `$data` repeats employee rows with date range + Person avatar + name
5. **DYNAMIC HEADING:** `string(count(time_off))` — dynamic count in heading

### Visual Patterns Used
`month_navigation`, `column_grid`, `accent_highlight_row`, `ooo_list`, `dynamic_count`, `grid_simulation`

---

## Entry 17: Employee Onboarding

**ID:** `employee-onboarding`
**Tags:** `onboarding, HR, welcome, steps, checklist, new-hire, links`
**Title:** Employee Onboarding Welcome Card

### Design Notes
1. **WELCOME HEADER:** Container style `accent` — personalized welcome with markdown link
2. **STEP CARDS:** `$data` on Container (style emphasis) — step number + title + description + CTA
3. **USEFUL LINKS:** Container style `accent` with `$data` ColumnSet — icon + label with selectAction
4. **ALTERNATING STYLES:** accent header → emphasis steps → accent links — creates visual rhythm

### Visual Patterns Used
`accent_header`, `markdown_links`, `step_cards`, `data_repeat`, `icon_link_list`, `alternating_styles`, `personalization`

---

## Entry 18: Product Showcase with Image Carousel

**ID:** `product-showcase`
**Tags:** `product, showcase, features, specs, carousel, images, e-commerce`
**Title:** Product Showcase with Image Carousel

### Design Notes
1. **IMAGE CAROUSEL:** Four Container blocks with ToggleVisibility — peek-next column shows next image as backgroundImage with chevron overlay
2. **FEATURE GRID:** Container emphasis — two rows of 2-column ColumnSets with icon + name + description
3. **SPECS TABLE:** `$data` ColumnSet — two columns: spec name (Bold) + stat
4. **PRICING FOOTER:** Container emphasis — availability badge + price (ExtraLarge Bold)

### Visual Patterns Used
`emphasis_header`, `image_carousel`, `peek_next`, `feature_grid`, `specs_table`, `pricing_footer`

---

## Entry 19-23: Additional Patterns

**Entry 19 (News Feed):** `conditional_thumbnail`, `article_layout`, `rich_timestamp`, `empty_state`, `separator_pattern`
**Entry 20 (Work Anniversary):** `hero_illustration`, `personalized_text`, `avatar_row`, `center_aligned_card`
**Entry 21 (App Directory):** `minimal_list_item`, `selectable_row`, `icon_right`
**Entry 22 (KPI Counter):** `big_number`, `conditional_delta`, `dynamic_color`, `pure_data_binding`
**Entry 23 (Notification Toast):** `icon_message`, `card_level_action`, `compact_design`

---

## Visual Patterns Summary (Quick Reference)

| Pattern | Description | Key Technique |
|---------|-------------|---------------|
| `hero_banner` | Full-bleed background image with text overlay | Container backgroundImage + bleed + minHeight + spacer |
| `text_overlay` | Text on semi-transparent background over image | Column style "default" inside backgroundImage Container |
| `thumbnail_container` | Fixed-size image area | Container backgroundImage with minHeight |
| `timeline_connector` | Vertical line connecting items | Column backgroundImage SVG + fillMode RepeatVertically |
| `tab_navigation` | Switch between views in same card | ColumnSet selectAction + ToggleVisibility on content containers |
| `expand_collapse_chevron` | Show/hide details per item | ToggleVisibility swapping chevron images + detail container |
| `view_swap_toggle` | Replace entire card view | ToggleVisibility hiding multiple elements, showing others |
| `image_carousel` | Browse multiple images | Multiple Containers with ToggleVisibility, peek-next column |
| `data_repeat` | Repeat layout per data item | $data property on ColumnSet or Container |
| `conditional_rendering` | Show/hide based on data | $when expressions on elements |
| `split_panel` | Asymmetric two-column layout | ColumnSet with unequal width ratios (e.g., 35/65) |
| `emphasis_card` | Visual card-in-card effect | Container style "emphasis" |
| `status_container` | Color-coded status section | Container style good/warning/attention |
| `dual_kpi_boxes` | Side-by-side metric displays | Two emphasis Columns with big numbers + deltas |
| `person_list` | Avatar + name + detail list | $data ColumnSet with Person Image + text columns |
| `author_row` | Compact author attribution | ColumnSet with small Person avatar + author text |
| `selectable_row` | Entire row as clickable area | selectAction on ColumnSet or Container |
| `spacer_container` | Precise spacing control | Empty Container with minHeight |
| `padding_column` | Right/left breathing room | Empty Column with fixed width |
| `inline_form` | Input + submit in same row | ColumnSet with Input (stretch) + ActionSet (auto) |
| `forward_arrow_affordance` | Visual drill-down hint | Small arrow image in last column |
| `dynamic_count` | Show list length in heading | string(count(array)) in TextBlock |
| `compact_widget` | Minimal footprint component | Container with bleed, minimal elements |

---

## Anti-Patterns to Avoid

1. **WALL OF FACTSET:** Don't use FactSet with 8+ items as primary content. Use Table or split across cards
2. **EMOJI IN LABELS:** Don't prefix every label with emoji. Use Icon element or semantic colors instead
3. **FLAT MENU:** Don't use ActionSet with 4+ plain buttons for navigation. Use CompoundButton with title + description + icon
4. **NESTED EMPHASIS:** Don't put Container style "emphasis" inside another "emphasis". Creates confusing visual nesting
5. **ORPHAN INPUTS:** Don't render a single Input alone. Always pair with context (heading, description, existing data)
6. **RAINBOW COLORS:** Don't use more than 2 semantic colors per card. Too many colors = visual noise
7. **IDENTICAL CARDS:** In multi-card flows, vary the opening element: hero on card 1, dashboard on card 2, list on card 3, form on card 4
