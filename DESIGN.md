# flowl — Design Guide & UI Mockups

## Design Philosophy

Warm, organic, calm. The UI should feel like a plant journal — not a corporate dashboard. Rounded shapes, soft colors, generous whitespace. Focus on the plants, not the chrome.

## Color Palette

### Light Mode (default)

| Role        | Name         | Hex       | Usage                              |
|-------------|--------------|-----------|-------------------------------------|
| Background  | Linen        | `#FAF6F1` | Page background                     |
| Surface     | White        | `#FFFFFF` | Cards, modals, inputs               |
| Primary     | Sage         | `#6B8F71` | Buttons, active states, links       |
| Primary Dark| Forest       | `#4A6B4F` | Hover, pressed states               |
| Secondary   | Terracotta   | `#C4775B` | Accents, highlights, overdue badges |
| Water       | Stream       | `#5B9BC4` | Watering indicators, water actions  |
| Text        | Bark         | `#2C2418` | Primary text                        |
| Text Muted  | Driftwood    | `#8C7E6E` | Secondary text, captions            |
| Border      | Sand         | `#E5DDD3` | Dividers, card borders              |
| Success     | Sprout       | `#7AB87A` | Healthy, watered, ok states         |
| Warning     | Amber        | `#D4A843` | Due soon                            |
| Danger      | Dry          | `#C45B5B` | Overdue, errors                     |

### Dark Mode

| Role        | Name         | Hex       | Usage                              |
|-------------|--------------|-----------|-------------------------------------|
| Background  | Soil         | `#1A1612` | Page background                     |
| Surface     | Loam         | `#252019` | Cards, modals, inputs               |
| Primary     | Sage         | `#8BB592` | Buttons, active states, links       |
| Primary Dark| Mint         | `#A3CDA9` | Hover, pressed states               |
| Secondary   | Clay         | `#D49478` | Accents, highlights                 |
| Water       | Sky          | `#78B4D8` | Watering indicators                 |
| Text        | Parchment    | `#EDE6DB` | Primary text                        |
| Text Muted  | Sandstone    | `#9C8E7E` | Secondary text                      |
| Border      | Root         | `#3A3228` | Dividers, card borders              |
| Success     | Leaf         | `#8BC48B` | Healthy states                      |
| Warning     | Honey        | `#D4B054` | Due soon                            |
| Danger      | Wilt         | `#D47878` | Overdue, errors                     |

## Typography

| Element     | Font           | Size   | Weight  |
|-------------|----------------|--------|---------|
| H1          | System sans    | 28px   | 700     |
| H2          | System sans    | 22px   | 600     |
| H3          | System sans    | 18px   | 600     |
| Body        | System sans    | 15px   | 400     |
| Caption     | System sans    | 13px   | 400     |
| Button      | System sans    | 15px   | 500     |
| Badge       | System sans    | 12px   | 600     |

Use the system font stack for fast rendering and native feel:

```css
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
```

## Spacing & Layout

- Base unit: `4px`
- Standard spacing: `8px`, `12px`, `16px`, `24px`, `32px`
- Card padding: `16px`
- Card border-radius: `12px`
- Button border-radius: `8px`
- Max content width: `1200px`
- Card grid gap: `16px`
- Mobile breakpoint: `768px`
- Widescreen breakpoint: `1280px`

## Components

### Plant Card

The primary UI element. Displayed in a responsive grid on the dashboard.

```
┌──────────────────────────────┐
│ ┌──────────┐                 │
│ │          │  Monstera       │
│ │  photo   │  Living Room    │
│ │          │                 │
│ │          │  ● Ok      3d   │
│ └──────────┘                 │
│                              │
│  💧 Every 7 days             │
│  Last: 3 days ago            │
│                              │
│  ┌──────────┐ ┌───────────┐  │
│  │ 💧 Water │ │  Details  │  │
│  └──────────┘ └───────────┘  │
└──────────────────────────────┘
```

- Photo: square, rounded corners (`8px`), placeholder icon if none
- Status dot: `Sprout` (ok), `Amber` (due soon), `Dry` (overdue)
- Days indicator: days until next watering or days overdue
- Quick "Water" action button in `Water/Stream` color
- Card background: `Surface`, border: `1px solid Border`

### Status Badge

Small pill-shaped badge showing watering status.

```
  ┌─────────┐   ┌──────────┐   ┌───────────┐
  │ ● Ok    │   │ ● Due    │   │ ● Overdue │
  └─────────┘   └──────────┘   └───────────┘
   (Sprout)      (Amber)         (Dry)
```

- Border-radius: `999px` (full pill)
- Padding: `4px 10px`
- Font: Badge size, uppercase
- Dot: `6px` circle, color matches status

### Water Button

Primary action button, always accessible.

```
  ┌────────────────┐
  │   💧 Water     │
  └────────────────┘
```

- Background: `Stream`
- Text: `White`
- Border-radius: `8px`
- Hover: darken 10%
- Active: scale(0.97) for press feedback
- After click: brief success animation (ripple or checkmark)

### Navigation

Sidebar on desktop, bottom tabs on mobile. Three sections: Plants, Log, Settings.

```
Desktop:                    Mobile:
┌──────┬───────────────┐    ┌───────────────────┐
│      │               │    │                   │
│  🌱  │               │    │     content       │
│      │               │    │                   │
│  🪴  │               │    ├─────┬─────┬─────┤
│Plants│   content     │    │ 🪴  │ 📓  │ ⚙️  │
│      │               │    │Plant│ Log │Conf │
│  📓  │               │    └─────┴─────┴─────┘
│ Log  │               │
│      │               │
│  ⚙️  │               │
│ Conf │               │
└──────┴───────────────┘
```

- Sidebar width: `64px` (icon-only, 769px–1279px) or `200px` (expanded with labels, >= 1280px)
- Active item: `Primary` background with rounded corners
- Mobile bottom bar: `56px` height, `Surface` background, top border

## Screens

### 1. Plants (Dashboard)

The landing page. Overview of all plants with focus on what needs attention.

```
┌──────────────────────────────────────────────────┐
│  flowl                                    ⚙️     │
├──────────────────────────────────────────────────┤
│                                                  │
│  Good morning! 🌱                                │
│  2 plants need water today                       │
│                                                  │
│  ┌─ Needs Attention ──────────────────────────┐  │
│  │                                            │  │
│  │  ┌─────────┐  ┌─────────┐                 │  │
│  │  │Monstera │  │ Ficus   │                 │  │
│  │  │● Overdue│  │● Due    │                 │  │
│  │  │ 2d late │  │ today   │                 │  │
│  │  └─────────┘  └─────────┘                 │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  All Plants (12)                    + Add Plant   │
│                                                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ Plant 1 │  │ Plant 2 │  │ Plant 3 │         │
│  │ ● Ok    │  │ ● Ok    │  │ ● Ok    │         │
│  └─────────┘  └─────────┘  └─────────┘         │
│                                                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ Plant 4 │  │ Plant 5 │  │ Plant 6 │         │
│  │ ● Ok    │  │ ● Ok    │  │ ● Ok    │         │
│  └─────────┘  └─────────┘  └─────────┘         │
│                                                  │
└──────────────────────────────────────────────────┘
```

- Greeting changes by time of day (morning/afternoon/evening)
- "Needs Attention" section only visible when plants are due/overdue
- Cards sorted: overdue first, then due, then by next watering date
- Grid: 4 columns widescreen (overlay cards), 3 columns desktop, 2 tablet, 1 mobile
- Widescreen cards: full-bleed image (240px tall), name and location float over a bottom gradient overlay

### 2. Plant Detail

Full view of a single plant with all its information.

```
┌──────────────────────────────────────────────────┐
│  ← Back                              Edit  🗑️   │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────────────┐                            │
│  │                  │  Monstera Deliciosa         │
│  │                  │  Living Room · Window       │
│  │      photo       │                            │
│  │                  │  ● Ok — next in 4 days      │
│  │                  │                            │
│  └──────────────────┘  ┌────────────────────┐    │
│                        │    💧 Water now     │    │
│                        └────────────────────┘    │
│                                                  │
│  ┌─ Watering ─────────────────────────────────┐  │
│  │  Every 7 days                              │  │
│  │  Last watered: Feb 10, 2026                │  │
│  │  Next due: Feb 17, 2026                    │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Notes ────────────────────────────────────┐  │
│  │  Likes indirect light. Wipe leaves monthly │  │
│  │  with damp cloth. Sensitive to overwatering│  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Care Log ─────────────────────────────────┐  │
│  │                                            │  │
│  │  Feb 10 · 💧 Watered                       │  │
│  │  Feb 3  · 💧 Watered                       │  │
│  │  Jan 28 · 🌱 Repotted — moved to bigger   │  │
│  │           pot, added fresh soil             │  │
│  │  Jan 27 · 💧 Watered                       │  │
│  │  Jan 15 · 📝 Yellowing leaf on lower       │  │
│  │           branch, removed it               │  │
│  │                                            │  │
│  │  + Add log entry                           │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
└──────────────────────────────────────────────────┘
```

- Large photo at top, tap to view fullscreen
- Quick water action prominently placed
- Sections as collapsible cards
- Care log as a timeline with icons per event type

### 3. Add / Edit Plant

Structured form with grouped sections and visual selectors instead of plain inputs.

```
┌──────────────────────────────────────────────────┐
│  Cancel              Add Plant             Save   │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┐  │
│  │         📷  Add a photo                    │  │
│  │      Click to select or drag & drop        │  │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┘  │
│                                                  │
│  ┌─ Identity ─────────────────────────────────┐  │
│  │  Name *      [Monstera Deliciosa        ]  │  │
│  │  Species     [Monstera                  ]  │  │
│  │  Icon  [🪴] [🌿] [🌵] [🌸] [🪻] [🌱] ...│  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Location ─────────────────────────────────┐  │
│  │  (Living Room) (Bedroom) (Kitchen)         │  │
│  │  (Balcony) (Office) (+ New)                │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Watering ─────────────────────────────────┐  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌──────┐│  │
│  │  │ 3 days │ │ 7 days │ │14 days │ │30 day││  │
│  │  │Thirsty │ │ Weekly │ │Biweekly│ │Monthl││  │
│  │  └────────┘ └────────┘ └────────┘ └──────┘│  │
│  │                                            │  │
│  │  Or set custom:  [−] [ 7 ] [+]  days      │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Light Needs ──────────────────────────────┐  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐   │  │
│  │  │ ☀️ Direct│ │🌤️Indirect│ │ 🌥️ Low   │   │  │
│  │  │ Full sun │ │ Filtered │ │  Shade   │   │  │
│  │  └──────────┘ └──────────┘ └──────────┘   │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Notes ────────────────────────────────────┐  │
│  │  Care tips, observations...                │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
└──────────────────────────────────────────────────┘
```

- **Photo upload**: centered area with subtle tinted background, icon in circle
- **Identity section**: name (required), species (optional), emoji icon picker for plant card display
- **Location**: selectable chips from existing locations + "New" chip to add a custom one
- **Watering**: preset cards (3d/7d/14d/30d) with descriptive labels, plus a stepper for custom intervals
- **Light needs**: three visual options (Direct/Indirect/Low) with icons and descriptions
- **Notes**: free-text area for care tips and observations
- Validation: name required, one location selected, watering interval > 0

### 4. Care Log

Global timeline of all care events across all plants, with filtering.

```
┌──────────────────────────────────────────────────┐
│  Care Log                                        │
├──────────────────────────────────────────────────┤
│                                                  │
│  [All] [💧 Watered] [🧪 Fertilized] [🌱 Repot] │
│  [✂️ Pruned] [📝 Notes]                          │
│                                                  │
│  TODAY — Feb 14, 2026                            │
│  ┌────────────────────────────────────────────┐  │
│  │ 💧  Monstera Deliciosa        10:30 AM     │  │
│  │     Watered                                │  │
│  ├────────────────────────────────────────────┤  │
│  │ 💧  Ficus Lyrata              10:28 AM     │  │
│  │     Watered                                │  │
│  ├────────────────────────────────────────────┤  │
│  │ 📝  Orchid                     9:15 AM     │  │
│  │     Note added                             │  │
│  │     New flower spike emerging on south stem│  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  YESTERDAY — Feb 13, 2026                        │
│  ┌────────────────────────────────────────────┐  │
│  │ 🧪  Snake Plant                6:00 PM     │  │
│  │     Fertilized                             │  │
│  │     Half-strength liquid fertilizer        │  │
│  ├────────────────────────────────────────────┤  │
│  │ 💧  Pothos                     8:30 AM     │  │
│  │     Watered                                │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
└──────────────────────────────────────────────────┘
```

- Filter chips at top: All (default), Watered, Fertilized, Repotted, Pruned, Notes
- Entries grouped by day with date headers
- Each entry shows: icon, plant name, timestamp, action type, optional note
- Mobile: icon-only filter chips to save space, shorter timestamps

### 5. Settings

Configuration page.

```
┌──────────────────────────────────────────────────┐
│  Settings                                        │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌─ Appearance ───────────────────────────────┐  │
│  │  Theme          Light / Dark / System       │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ MQTT ─────────────────────────────────────┐  │
│  │  Status         ● Connected                │  │
│  │  Broker         192.168.1.10:1883          │  │
│  │  Topic prefix   flowl                      │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Data ─────────────────────────────────────┐  │
│  │  Export         [Download JSON]             │  │
│  │  Import         [Upload JSON]              │  │
│  │  Plants         12 plants, 84 log entries  │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌─ About ────────────────────────────────────┐  │
│  │  Version        0.1.0                      │  │
│  │  Source         github.com/simplyRoba/flowl│  │
│  └────────────────────────────────────────────┘  │
│                                                  │
└──────────────────────────────────────────────────┘
```

- MQTT settings are read-only (configured via env vars), but show connection status
- Theme toggle: tri-state (light / dark / follow system)
- Export/import for backup and migration

## Iconography

Hybrid icon system using two libraries:

- **Lucide** (`lucide-svelte`) — monochrome outline icons for UI chrome (navigation, buttons, actions, status indicators). Icons inherit the current text color and adapt to light/dark themes.
- **Noto Color Emoji** (Google, Apache 2.0) — colored SVG emoji for plant identity icons (plant card photos, emoji picker). Downloaded locally to `ui/static/emoji/` to avoid CDN calls at runtime.

### UI Chrome Icons (Lucide)

| Concept    | Lucide name      | Context                        |
|------------|------------------|--------------------------------|
| Water      | `droplet`        | Watering actions, schedule     |
| Plant      | `sprout`         | New plant, repotting, growth   |
| Fertilize  | `flask-conical`  | Fertilizing log entries        |
| Note       | `file-text`      | General notes, observations    |
| Prune      | `scissors`       | Pruning log entries            |
| Location   | `map-pin`        | Room/location labels           |
| Settings   | `settings`       | Configuration                  |
| Warning    | `alert-triangle` | Overdue, attention needed      |
| Plants nav | `leaf`           | Sidebar/bottom nav             |
| Log nav    | `book-open`      | Sidebar/bottom nav             |
| Logo       | `sprout`         | App logo in sidebar            |
| Back       | `arrow-left`     | Navigation back                |
| Edit       | `pencil`         | Edit plant                     |
| Delete     | `trash-2`        | Delete plant                   |
| Camera     | `camera`         | Photo upload                   |
| Sun        | `sun`            | Direct light                   |
| Partial    | `cloud-sun`      | Indirect light                 |
| Shade      | `cloud`          | Low light                      |

### Plant Identity Icons (Noto Color Emoji)

| File                   | Emoji | Usage                  |
|------------------------|-------|------------------------|
| `emoji_u1fab4.svg`     | 🪴    | Potted plant (default) |
| `emoji_u1f33f.svg`     | 🌿    | Herb                   |
| `emoji_u1f335.svg`     | 🌵    | Cactus                 |
| `emoji_u1f338.svg`     | 🌸    | Cherry blossom         |
| `emoji_u1fabb.svg`     | 🪻    | Hyacinth               |
| `emoji_u1f331.svg`     | 🌱    | Seedling               |
| `emoji_u1f337.svg`     | 🌷    | Tulip                  |
| `emoji_u1f33b.svg`     | 🌻    | Sunflower              |
| `emoji_u1f340.svg`     | 🍀    | Four leaf clover       |

Served from `/emoji/` as static assets. Source: https://github.com/googlefonts/noto-emoji (Apache 2.0).

## Animations & Interactions

- **Page transitions**: subtle fade (150ms)
- **Card hover**: slight lift (`translateY(-2px)`, shadow increase)
- **Water action**: ripple effect on button, card status transitions smoothly to "Ok"
- **Loading states**: skeleton placeholders matching card layout
- **Toast notifications**: slide in from top, auto-dismiss after 3s ("Monstera watered!")
- **Pull to refresh** (mobile): custom animation with water drop
