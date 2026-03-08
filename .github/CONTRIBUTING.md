# Contributing to Time2Change

Thank you for your interest in contributing to Time2Change! This guide will help you get started and ensure a smooth contribution process.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Architecture Overview](#architecture-overview)
- [Development Workflow](#development-workflow)
- [Coding Conventions](#coding-conventions)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [CI/CD Pipeline](#cicd-pipeline)
- [Getting Help](#getting-help)

## Code of Conduct

By participating in this project, you agree to be respectful, inclusive, and constructive in all interactions. Harassment, discrimination, and toxic behavior will not be tolerated.

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork locally:
   ```sh
   git clone https://github.com/<your-username>/Time2Change.git
   cd Time2Change
   ```
3. **Add the upstream remote:**
   ```sh
   git remote add upstream https://github.com/DocteurPing/Time2Change.git
   ```
4. **Create a branch** for your work:
   ```sh
   git checkout -b feat/your-feature-name
   ```

## Development Setup

### Prerequisites

- **Rust** (edition 2024) — install via [rustup](https://rustup.rs/):
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Nightly toolchain** (required for `rustfmt` and `cargo-udeps`):
  ```sh
  rustup toolchain install nightly
  rustup component add rustfmt --toolchain nightly
  ```
- **Clippy:**
  ```sh
  rustup component add clippy
  ```

### Recommended Tools

Install these tools for the best development experience:

```sh
# Fast test runner with better output
cargo install cargo-nextest

# Code coverage
cargo install cargo-llvm-cov

# Dependency license & vulnerability checking
cargo install cargo-deny

# Security audit
cargo install cargo-audit

# Detect unused dependencies (requires nightly)
cargo install cargo-udeps
```

### Building the Project

```sh
# Check that everything compiles
cargo check --workspace

# Build all crates in debug mode
cargo build --workspace

# Build in release mode
cargo build --workspace --release
```

### Verifying Your Setup

Run the full validation suite locally before pushing:

```sh
# Format check
cargo +nightly fmt --all -- --check

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Tests
cargo nextest run --workspace --all-features

# Doc tests
cargo test --workspace --doc
```

## Architecture Overview

Time2Change follows **Clean Architecture** principles with the following crate structure:

```
crates/
├── domain/           # Core business logic, entities, and value objects
│                     # No external dependencies — pure Rust types and logic
├── application/      # Use cases, ports (trait interfaces), and DTOs
│                     # Depends on: domain, shared
├── infrastructure/   # Adapter implementations (databases, external APIs, etc.)
│                     # Depends on: application, domain, shared
├── api/              # HTTP API layer (binary crate)
│                     # Depends on: application, shared
├── ingestion/        # Data ingestion pipeline (binary crate)
│                     # Depends on: application, infrastructure
└── shared/           # Cross-cutting utilities shared across crates
```

### Dependency Rules

These rules are critical and must be followed:

- **`domain`** must NOT depend on any other workspace crate.
- **`application`** depends only on `domain` and `shared`.
- **`infrastructure`** implements traits defined in `application`.
- **`api`** and **`ingestion`** are entry points (binary crates) and can depend on other crates as needed.
- **`shared`** must NOT depend on any other workspace crate.

## Development Workflow

### Branch Naming

Use descriptive, prefixed branch names:

| Prefix       | Purpose                              | Example                        |
| ------------ | ------------------------------------ | ------------------------------ |
| `feat/`      | New feature                          | `feat/add-rate-history-api`    |
| `fix/`       | Bug fix                              | `fix/decimal-precision-error`  |
| `refactor/`  | Code restructuring                   | `refactor/simplify-use-cases`  |
| `docs/`      | Documentation changes                | `docs/add-api-examples`        |
| `test/`      | Adding or improving tests            | `test/coverage-domain-types`   |
| `ci/`        | CI/CD changes                        | `ci/add-docker-build`          |
| `chore/`     | Maintenance, deps, tooling           | `chore/update-dependencies`    |

### Keeping Your Branch Up to Date

```sh
git fetch upstream
git rebase upstream/main
```

Prefer **rebase** over merge to keep a clean linear history.

## Coding Conventions

### Rust Style

- **Format all code** with `cargo +nightly fmt` before committing.
- **No warnings allowed** — CI runs with `RUSTFLAGS="-Dwarnings"`.
- **Clippy pedantic** is enforced. Fix all Clippy warnings before submitting.
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

### General Principles

- **Prefer explicitness over cleverness.** Code is read far more than it is written.
- **Handle errors properly.** Use `thiserror` for library error types. Never use `.unwrap()` in production code — only in tests.
- **Write documentation.** All public items (`pub fn`, `pub struct`, `pub enum`, `pub trait`) must have doc comments (`///`).
- **Avoid `unsafe`.** If you absolutely must use it, include a `// SAFETY:` comment explaining why it is sound.
- **Keep functions small and focused.** If a function exceeds ~40 lines, consider decomposing it.
- **Use meaningful names.** Variable and function names should clearly convey intent.

### Module Organization

- One type per file when the type is non-trivial.
- Group related types together with a `mod.rs` that re-exports them.
- Keep `mod.rs` files minimal — declarations and re-exports only.

### Error Handling

```rust
// Good: Use thiserror for structured error types
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("currency pair {0} is not supported")]
    UnsupportedPair(String),

    #[error("insufficient data points: need {needed}, got {got}")]
    InsufficientData { needed: usize, got: usize },
}

// Bad: String errors or panics
fn bad_example() -> Result<(), String> { ... }  // Don't do this
fn worse_example() { panic!("something went wrong"); }  // Definitely don't do this
```

## Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <short description>

[optional body]

[optional footer(s)]
```

### Types

| Type         | Description                                            |
| ------------ | ------------------------------------------------------ |
| `feat`       | A new feature                                          |
| `fix`        | A bug fix                                              |
| `docs`       | Documentation only changes                             |
| `style`      | Formatting, missing semicolons, etc. (no logic change) |
| `refactor`   | Code change that neither fixes a bug nor adds a feature|
| `perf`       | Performance improvement                                |
| `test`       | Adding or correcting tests                             |
| `ci`         | Changes to CI configuration                            |
| `chore`      | Maintenance tasks (deps, tooling)                      |
| `revert`     | Reverts a previous commit                              |

### Scopes

Use the crate name as the scope: `domain`, `application`, `infrastructure`, `api`, `ingestion`, `shared`.

### Examples

```
feat(domain): add volatility indicator calculation

Implement Bollinger Bands and ATR indicators for the TimeSeries type.
These are needed for the upcoming rate quality scoring feature.

Closes #42
```

```
fix(application): handle empty rate history in analysis

Return an appropriate error instead of panicking when the rate
provider returns an empty history for a currency pair.
```

```
ci: add code coverage reporting to CI pipeline
```

### Rules

- Use the **imperative mood** in the subject line ("add feature" not "added feature").
- Keep the subject line to **72 characters or fewer**.
- Do not end the subject line with a period.
- Separate the subject from the body with a blank line.
- Reference relevant issues in the footer.

## Pull Request Process

1. **Ensure CI passes** — run the full validation suite locally before pushing.
2. **Fill out the PR template** completely. Skip nothing.
3. **Keep PRs focused** — one logical change per PR. If a PR exceeds ~500 lines, consider splitting it.
4. **Write descriptive titles** — use the same Conventional Commits format as commit messages.
5. **Link related issues** — use `Closes #123` or `Fixes #123` in the PR description.
6. **Respond to review feedback** promptly and constructively.

### Review Expectations

- All PRs require at least **one approving review** before merging.
- Maintainers may request changes. This is normal and collaborative — not adversarial.
- Mark conversations as "resolved" once addressed, but let the reviewer verify.

### Merge Strategy

- PRs are merged via **squash merge** to keep the main branch history clean.
- The squashed commit message should follow Conventional Commits.

## Testing

### Writing Tests

- **Unit tests** live alongside the code in `#[cfg(test)]` modules or in dedicated `tests/` directories within each crate.
- **Test names** should describe the scenario and expected outcome:
  ```rust
  #[test]
  fn analyze_pair_returns_error_when_history_is_empty() { ... }

  #[test]
  fn time_series_calculates_correct_moving_average() { ... }
  ```
- Use the existing test helpers and mocks in `crates/application/src/tests/` as patterns for new tests.

### Running Tests

```sh
# Run all tests with nextest
cargo nextest run --workspace --all-features

# Run tests for a specific crate
cargo nextest run -p domain

# Run a specific test by name
cargo nextest run -p domain -- time_series

# Run doc tests (nextest doesn't support these)
cargo test --workspace --doc

# Generate a coverage report
cargo llvm-cov --workspace --all-features --html
# Open target/llvm-cov/html/index.html in your browser
```

### Test Guidelines

- Every bug fix should include a test that reproduces the bug.
- Every new feature should include tests covering the happy path and important edge cases.
- Tests should be **deterministic** — no reliance on external services, network, or wall-clock time.
- Use `rust_decimal_macros::dec!` for precise decimal values in tests.
- Aim for meaningful coverage, not 100% line coverage. Test behavior, not implementation details.

## CI/CD Pipeline

The CI pipeline runs automatically on every push to `main`/`develop` and on all pull requests. Here's what it checks:

| Job              | What It Does                                              |
| ---------------- | --------------------------------------------------------- |
| **Rustfmt**      | Verifies all code is formatted with `cargo fmt`           |
| **Clippy**       | Runs pedantic linting with zero warnings allowed          |
| **Check**        | Validates that the workspace compiles                     |
| **Tests**        | Runs the full test suite with `cargo-nextest`             |
| **Security Audit** | Checks dependencies for known vulnerabilities           |
| **Cargo Deny**   | Validates licenses, bans, and sources                     |
| **Build**        | Produces release binaries for Linux and macOS             |

All jobs must pass before a PR can be merged. The **CI Success** job acts as a single gate for branch protection rules.

### Running CI Checks Locally

You can replicate the full CI pipeline locally:

```sh
# 1. Format
cargo +nightly fmt --all -- --check

# 2. Clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::nursery -A clippy::module_name_repetitions -A clippy::future_not_send

# 3. Check
cargo check --workspace --all-targets --all-features

# 4. Test
cargo nextest run --workspace --all-features

# 5. Doc tests
cargo test --workspace --doc

# 6. Security
cargo audit
cargo deny check advisories bans licenses sources
```

## Getting Help

- **Questions:** Open a [Discussion](https://github.com/DocteurPing/Time2Change/discussions) on GitHub.
- **Bugs:** File an issue using the [Bug Report](https://github.com/DocteurPing/Time2Change/issues/new?template=bug_report.yml) template.
- **Features:** File an issue using the [Feature Request](https://github.com/DocteurPing/Time2Change/issues/new?template=feature_request.yml) template.

---

Thank you for contributing to Time2Change! Every contribution — whether it's code, documentation, bug reports, or ideas — makes this project better. 🚀
