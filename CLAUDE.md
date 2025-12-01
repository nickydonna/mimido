# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Mimido is a desktop calendar and task management application built with Tauri (Rust backend), SvelteKit (frontend), and TypeScript. The app connects to CalDAV servers to sync calendar events and todos, offering a native desktop experience with a glass-morphism UI design.

## Key Technologies

- **Backend**: Tauri 2.x, Rust, Diesel ORM with SQLite
- **Frontend**: SvelteKit 2.x, Svelte 5 (with runes), TypeScript, Tailwind CSS 4.x
- **CalDAV**: Custom CalDAV client using `libdav` for iCalendar sync
- **Type Safety**: TypeScript bindings auto-generated from Rust using `tauri-specta`

## Development Commands

### Frontend Development
```bash
pnpm dev                 # Start Vite dev server
pnpm build              # Build frontend for production
pnpm check              # Run svelte-check for type checking
pnpm check:watch        # Run svelte-check in watch mode
```

### Tauri Commands
```bash
pnpm tauri dev          # Run Tauri app in development mode
pnpm tauri build        # Build Tauri app for production
pnpm tauri ios build    # Build for iOS (if configured)
```

### Database
Migrations are in `src-tauri/migrations/` and run automatically on app startup via Diesel. SQLite database is stored in the app config directory (`~/Library/Application Support/com.tauri.mimido/mimido.db` on macOS).

## Architecture

### Backend (Rust)

**Entry Point**: `src-tauri/src/lib.rs`
- Initializes Tauri app with tray icon
- Sets up SQLite connection and runs migrations
- Exports TypeScript bindings in debug mode to `src/bindings.ts`

**Core Modules**:
- `commands/`: Tauri commands exposed to frontend
  - `create_server`, `list_servers`: Server management
  - `calendar.rs`: Calendar operations (fetch, sync, set default)
  - `components.rs`: Event/todo CRUD operations (list, create, update, delete)
- `caldav.rs`: CalDAV client implementation using `libdav`
- `calendar_items/`: Event parsing and data models
  - `event_upsert.rs`: Natural language parser for creating events (e.g., "Fly like an eagle tomorrow at 9 @block %done")
  - `event_date/`: Date parsing logic
  - `event_status.rs`, `event_type.rs`, `event_tags.rs`: Event metadata enums
  - `component_props.rs`: Props extraction from iCalendar data
- `models/`: Diesel ORM models (Server, Calendar, VEvent, VTodo)
- `schema.rs`: Database schema definitions

**Natural Language Event Creation**:
The app supports natural language input for creating events. Users can type strings like:
- `"Meeting tomorrow at 2pm @block %done"`
- `"Weekly standup every Monday at 10am @reminder"`

Syntax elements (parsed in `calendar_items/event_upsert.rs`):
- `@block`, `@event`, `@reminder`, `@task`: Event types
- `%backlog`, `%todo`, `%done`, `%inprogress`: Status
- `#tag`: Tags
- Dates/times: Parsed using natural language (tomorrow, next week, at 2pm, etc.)

### Frontend (SvelteKit + Svelte 5)

**Routing**:
- `/` → Redirects to `/day`
- `/day`: Main calendar view showing events/todos for selected day
- `/servers`: Server and calendar management

**Key Components** (`src/lib/components/`):
- `event-creation-modal/`: Modal for creating/editing events with natural language input
- `event-card/`: Displays individual event/todo items
- `glass-*`: Custom glass-morphism styled UI components (buttons, inputs, checkboxes)
- `navigation/`: Navigation sidebar
- `task-list/`: Task list display

**State Management** (`src/stores/`):
- `eventUpserter.svelte.ts`: ADT-based state for create/update/none event modal states
- `times.svelte.ts`: Global time state for calendar navigation

**Type Definitions**:
- `src/bindings.ts`: Auto-generated TypeScript types and command wrappers from Rust
- All Tauri commands return `Result<T, string>` type (success/error union)

**Svelte 5 Runes**:
This project uses Svelte 5's runes syntax:
- `$state`: For reactive state (replaces `let` for reactive variables)
- `$props()`: For component props
- `$derived`: For computed values
- Example: `let { defaultCalendar }: { defaultCalendar: Calendar | undefined } = $props();`

### CalDAV Sync Flow

1. User adds server with credentials (`create_server` command)
2. Fetch calendars from server (`fetch_calendars` command)
3. Sync calendar events/todos (`sync_calendar` or `sync_all_calendars`)
4. Local SQLite cache stores VEvents and VTodos
5. Frontend queries local cache for display (`list_events_for_day`, `list_todos_for_day`)

### Database Schema

Key tables (see `src-tauri/src/schema.rs`):
- `servers`: CalDAV server credentials
- `calendars`: Calendar metadata (name, URL, etag, sync_token)
- `vevents`: Calendar events (with recurrence rules)
- `vtodos`: Task/todo items

## Important Patterns

### Error Handling
Rust commands use `Result<T, CommandError>` which gets serialized to `Result<T, string>` in TypeScript. Frontend uses `unwrap()` helper from `src/lib/result.ts` to extract values.

### Type Generation
**CRITICAL**: When modifying Rust commands or types, TypeScript bindings are auto-generated in **debug mode only**. Run `pnpm tauri dev` (not `pnpm dev`) to regenerate `src/bindings.ts`.

### ADT Pattern
The frontend uses `@korkje/adt` for algebraic data types (sum types). See `eventUpserter.svelte.ts` for example:
```typescript
export const EventUpsert = adt({
  None: null,
  Creating: (type?: EventType, startDate?: Date) => ({ type, startDate }),
  Updating: (event: ScheduledTask | UnscheduledTask) => ({ event }),
});
```

### Static Site Generation
SvelteKit uses `adapter-static` to prerender the app as static HTML since Tauri doesn't run a Node.js server. All routes must be prerenderable or explicitly set to `ssr: false`.

## Dependencies

### External Rust Dependencies
- `libdav`: Located at `../../libdav/` (relative path dependency for CalDAV operations)

### Build Configuration
Uses `pnpm` as package manager (version 10.14.0+). Node modules are managed via pnpm workspaces.

## Platform Support
- Desktop: macOS, Windows, Linux (via Tauri)
- iOS: Build support included (`pnpm tauri ios build`)
