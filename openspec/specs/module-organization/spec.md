# module-organization

## Purpose

Enforce Rust 2018 edition modern module style across the TrendAITool codebase. Prohibits `mod.rs` files and mandates the `src/module.rs` + `src/module/` pattern for all multi-file modules.

## Requirements

### Requirement: Modern module style

All multi-file Rust modules SHALL use the 2018 edition "non-mod.rs" convention. For any module `X` with submodules, the project SHALL have `src/X.rs` as the module entry point and `src/X/` as the submodule directory. Files named `mod.rs` SHALL NOT exist anywhere in `src/`.

#### Scenario: Models module with submodules

- **WHEN** the `models` module contains `token`, `source`, and `keyword` submodules
- **THEN** the file `src/models.rs` SHALL exist and contain `pub mod token; pub mod source; pub mod keyword;`
- **THEN** the files `src/models/token.rs`, `src/models/source.rs`, and `src/models/keyword.rs` SHALL exist
- **THEN** no file at `src/models/mod.rs` SHALL exist

#### Scenario: Handler module with submodules

- **WHEN** the `handlers` module is created
- **THEN** the file `src/handlers.rs` SHALL exist as the module entry point
- **THEN** the directory `src/handlers/` SHALL exist for submodule files
- **THEN** no file at `src/handlers/mod.rs` SHALL exist

#### Scenario: Middleware module with submodules

- **WHEN** the `middleware` module is created
- **THEN** the file `src/middleware.rs` SHALL exist as the module entry point
- **THEN** the directory `src/middleware/` SHALL exist for submodule files
- **THEN** no file at `src/middleware/mod.rs` SHALL exist

#### Scenario: Services module with submodules

- **WHEN** the `services` module is created
- **THEN** the file `src/services.rs` SHALL exist as the module entry point
- **THEN** the directory `src/services/` SHALL exist for submodule files
- **THEN** no file at `src/services/mod.rs` SHALL exist
- **THEN** `src/services.rs` SHALL contain `pub mod parser;`, `pub mod filter;`, and `pub mod pusher;`

### Requirement: CLAUDE.md documents the convention

The project's CLAUDE.md SHALL document the module organization convention, clearly prohibiting `mod.rs` usage and showing correct examples.

#### Scenario: Developer reads CLAUDE.md

- **WHEN** a developer opens CLAUDE.md
- **THEN** the module organization rules SHALL be clearly stated
- **THEN** correct and incorrect examples SHALL be shown
