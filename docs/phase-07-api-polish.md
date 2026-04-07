# Phase 7: API Polish & Developer Experience

## Overview

Polish the API, improve documentation, and enhance developer experience.

## Features

### 7.1 Documentation
- [ ] Comprehensive module documentation
- [ ] Usage examples for all features
- [ ] API reference documentation
- [ ] Migration guides

### 7.2 Procedural Macros
- [ ] #[require_role] attribute macro
- [ ] #[require_any_role] attribute macro
- [ ] Derive macros for user providers

### 7.3 Builder Pattern
- [ ] AuthConfig builder
- [ ] Fluent configuration API
- [ ] Validation at build time

### 7.4 Utilities
- [ ] Login/Register macros
- [ ] Token refresh macros
- [ ] Error handling helpers

## File Structure

```
src/
├── macros.rs           (< 200 lines)
├── builders.rs         (< 100 lines)
└── utils.rs            (< 100 lines)

rok-auth-macros/
└── src/
    ├── lib.rs          (< 300 lines)
    └── derive/
        ├── mod.rs      (< 50 lines)
        └── provider.rs (< 150 lines)
```

## Acceptance Criteria

1. All public APIs are documented
2. Macros work correctly with clear error messages
3. Builder patterns are ergonomic
4. Examples compile and work
5. All tests pass

## Dependencies

- quote
- syn
- proc-macro2

## Status

- [ ] Not Started
- [ ] In Progress
- [ ] Completed
