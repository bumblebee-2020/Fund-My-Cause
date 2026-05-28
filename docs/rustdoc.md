# Rustdoc Setup & Contract Documentation Guide

This document explains how rustdoc is configured for the Fund-My-Cause smart contracts,
how to build the docs locally, and how they are deployed to GitHub Pages.

---

## Overview

The project uses [rustdoc](https://doc.rust-lang.org/rustdoc/) — Rust's built-in documentation
generator — to produce API reference docs for both Soroban contracts:

| Crate | Description | Docs URL |
|-------|-------------|----------|
| `crowdfund` | Main crowdfunding campaign contract | `<pages-url>/crowdfund/` |
| `registry` | Campaign discovery registry contract | `<pages-url>/registry/` |

Docs are built automatically on every push to `main` via the
[`.github/workflows/docs.yml`](../.github/workflows/docs.yml) workflow and deployed to
GitHub Pages.

---

## Building Docs Locally

### Prerequisites

- Rust stable toolchain (`rustup toolchain install stable`)
- The `rust-docs` component (`rustup component add rust-docs`)

### Quick build

```bash
# From the repo root — builds all workspace crates, excludes benchmarks
cargo doc --workspace --exclude benchmarks --no-deps --document-private-items

# Open in browser (macOS / Linux)
open target/doc/crowdfund/index.html

# Open in browser (Windows)
start target/doc/crowdfund/index.html
```

### With warnings-as-errors (mirrors CI)

```bash
RUSTDOCFLAGS="--default-theme=ayu --cfg docsrs -D warnings" \
  cargo doc --workspace --exclude benchmarks --no-deps --document-private-items
```

### Using Make

```bash
# Build docs
make docs

# Build and open in browser
make docs-open

# Check docs compile without warnings
make docs-check
```

---

## Documentation Standards

All public items in the contracts **must** have doc comments. The CI workflow enforces
`-D warnings` so missing or malformed doc comments will fail the build.

### Required sections

| Item type | Required sections |
|-----------|------------------|
| Module (`mod`) | Summary line + `## Overview` |
| Public function | Summary line + `# Arguments` + `# Returns` + `# Example` |
| Public struct | Summary line + field-level `///` comments |
| Public enum | Summary line + variant-level `///` comments |
| Error enum | Summary line + variant-level `///` comments |

### Writing doc comments

```rust
/// One-line summary of what this function does.
///
/// Optional longer description. Explain the *why*, not just the *what*.
/// Use full sentences and end with a period.
///
/// # Arguments
///
/// * `env` - The Soroban environment (always first).
/// * `amount` - Contribution amount in stroops (must be > 0).
///
/// # Returns
///
/// * `Ok(())` on success.
/// * `Err(ContractError::BelowMinimum)` if `amount` is zero or negative.
///
/// # Errors
///
/// Returns [`ContractError::NotActive`] if the campaign is not in `Active` status.
///
/// # Example
///
/// ```ignore
/// // Contribute 10 XLM (100_000_000 stroops)
/// contract.contribute(env, contributor, 100_000_000, token)?;
/// ```
pub fn contribute(env: Env, contributor: Address, amount: i128, token: Address) -> Result<(), ContractError> {
    // ...
}
```

> **Note:** Use `ignore` on code examples that require a live Soroban environment.
> This prevents rustdoc from trying to compile them as standalone tests.

### Linking between items

Use intra-doc links to cross-reference types and functions:

```rust
/// See [`CrowdfundContract::contribute`] for the contribution flow.
/// Returns a [`ContractError::BelowMinimum`] if the amount is too small.
/// The full error list is in [`errors::ContractError`].
```

---

## Configuration

### `Cargo.toml` — per-crate settings

Each contract crate has a `[package.metadata.docs.rs]` section that controls how
docs.rs (and our CI) builds the documentation:

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = [
  "--cfg", "docsrs",          # enables #[cfg(docsrs)] feature-gated doc items
  "--document-private-items", # include private items (useful for internal audits)
  "--default-theme=ayu",      # dark theme by default
]
```

### `RUSTDOCFLAGS` environment variable

The CI workflow sets:

```
RUSTDOCFLAGS="--default-theme=ayu --cfg docsrs -D warnings"
```

`-D warnings` turns all rustdoc warnings into errors, ensuring the docs stay clean.

---

## CI/CD Deployment

The workflow at `.github/workflows/docs.yml`:

1. Triggers on pushes to `main` that touch `contracts/**` or the workflow file itself.
2. Installs the stable Rust toolchain with the `rust-docs` component.
3. Runs `cargo doc --workspace --exclude benchmarks --no-deps --document-private-items`.
4. Creates a root `index.html` redirect to `crowdfund/index.html`.
5. Uploads the `target/doc/` directory as a GitHub Pages artefact.
6. Deploys to GitHub Pages via the `actions/deploy-pages` action.

To enable GitHub Pages for the repository:
1. Go to **Settings → Pages**.
2. Set **Source** to **GitHub Actions**.
3. The next push to `main` will trigger a deployment.

---

## Viewing the Deployed Docs

Once deployed, the docs are available at:

```
https://<org>.github.io/<repo>/
```

The root redirects to the `crowdfund` crate. Navigate to `registry/` for the registry
contract docs.

---

## Troubleshooting

### `error[E0658]: use of unstable library feature`

Some Soroban SDK types use nightly-only features. If you see this error, ensure you are
on the **stable** toolchain:

```bash
rustup override set stable
cargo doc --workspace --exclude benchmarks --no-deps
```

### `warning: missing documentation for ...`

Add a `///` doc comment to the flagged item. With `-D warnings` in CI this becomes an
error. Run locally first to catch all missing docs before pushing.

### Docs not updating on GitHub Pages

Check the **Actions** tab for the `Deploy Contract Docs` workflow. Common causes:
- The push did not touch `contracts/**` (workflow only triggers on contract changes).
- GitHub Pages is not enabled — see the setup steps above.
- The `pages` write permission is missing from the workflow (already set in `docs.yml`).
