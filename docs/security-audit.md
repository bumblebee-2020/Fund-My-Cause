# Security Audit Documentation

## Overview

Automated security scanning runs on every PR and push to main/develop branches, plus weekly scheduled scans.

## Tools Used

### 1. cargo-audit
Checks Rust dependencies against the RustSec Advisory Database for known vulnerabilities.

**What it checks:**
- Known CVEs in dependencies
- Unmaintained crates
- Yanked crates
- Security advisories

**Failure criteria:**
- Any critical severity vulnerabilities

### 2. cargo-geiger
Analyzes usage of `unsafe` code in the project and dependencies.

**What it checks:**
- Unsafe blocks in contracts
- Unsafe code in dependencies
- Percentage of unsafe code

**Purpose:**
- Identify potential memory safety issues
- Track unsafe code usage over time

### 3. Custom Security Checks
Project-specific security patterns and anti-patterns.

**Checks performed:**
- ❌ `unwrap()` calls (should use proper error handling)
- ❌ `panic!` macros (should use Result types)
- ⚠️ TODO/FIXME in production code
- ❌ Hardcoded Stellar addresses

**Failure criteria:**
- Any unwrap() or panic! in non-test code
- Hardcoded addresses in contracts

### 4. Clippy Security Lints
Rust's official linter with security-focused rules enabled.

**Lints enabled:**
- `clippy::unwrap_used` - Warns on unwrap usage
- `clippy::expect_used` - Warns on expect usage
- `clippy::panic` - Warns on panic usage
- `clippy::integer_arithmetic` - Warns on unchecked arithmetic
- `clippy::indexing_slicing` - Warns on unchecked indexing

## Running Locally

### Full audit suite
```bash
# Install tools
cargo install cargo-audit cargo-geiger

# Run audits
cargo audit
cargo geiger

# Run custom checks
./scripts/security-check.sh  # (if created)

# Run clippy with security lints
cargo clippy --all-targets -- \
  -W clippy::unwrap_used \
  -W clippy::expect_used \
  -W clippy::panic
```

### Individual tools
```bash
# Check for vulnerabilities
cargo audit

# Analyze unsafe code
cargo geiger

# Security-focused clippy
cargo clippy -- -W clippy::unwrap_used
```

## Interpreting Results

### cargo-audit Output
```
Crate:     some-crate
Version:   1.0.0
Warning:   vulnerability
Title:     Buffer overflow in some-crate
Date:      2024-01-01
ID:        RUSTSEC-2024-0001
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0001
Severity:  high
```

**Actions:**
- **Critical/High**: Update immediately or find alternative
- **Medium**: Plan update in next sprint
- **Low**: Monitor and update when convenient

### cargo-geiger Output
```
Metric output format: x/y
    x = unsafe code used by the build
    y = total unsafe code found in the crate

Functions  Expressions  Impls  Traits  Methods  Dependency
0/0        0/0          0/0    0/0     0/0      crowdfund
```

**Ideal state**: All zeros (no unsafe code)

### Custom Checks
- **Unwrap/Panic**: Refactor to use `?` operator or proper error handling
- **Hardcoded addresses**: Move to configuration or parameters
- **TODO/FIXME**: Complete or create tracking issues

## Fixing Vulnerabilities

### Update dependencies
```bash
# Update to latest compatible versions
cargo update

# Update to specific version
cargo update -p vulnerable-crate --precise 1.2.3

# Check if fixed
cargo audit
```

### Replace vulnerable crates
```toml
# In Cargo.toml, replace:
# vulnerable-crate = "1.0"
# with:
alternative-crate = "2.0"
```

### Apply patches
```toml
# In Cargo.toml
[patch.crates-io]
vulnerable-crate = { git = "https://github.com/maintainer/vulnerable-crate", branch = "security-fix" }
```

## CI Integration

### Workflow triggers
- Every PR to main/develop
- Every push to main/develop
- Weekly on Sunday at midnight UTC

### Blocking vs Non-blocking
- **Blocks merge**: Critical vulnerabilities, unwrap/panic in production code
- **Warns only**: TODO comments, medium/low vulnerabilities

### Viewing reports
1. Go to Actions tab
2. Select "Security Audit" workflow
3. Click on latest run
4. Download artifacts for detailed JSON reports

## Best Practices

### For contract development
1. ✅ Use `?` operator instead of `unwrap()`
2. ✅ Return `Result` types from functions
3. ✅ Use `panic_with_error!` from soroban-sdk for contract errors
4. ✅ Validate all inputs
5. ✅ Use checked arithmetic operations
6. ❌ Avoid `unsafe` code unless absolutely necessary
7. ❌ Never hardcode addresses or secrets

### For dependency management
1. ✅ Pin exact versions in Cargo.lock (already committed)
2. ✅ Review dependency updates before merging
3. ✅ Prefer well-maintained crates with security audits
4. ✅ Minimize dependency count
5. ❌ Don't ignore audit warnings

## Reporting Security Issues

If you discover a security vulnerability:

1. **DO NOT** open a public issue
2. Email security contact (add email here)
3. Include:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Resources

- [RustSec Advisory Database](https://rustsec.org/)
- [Soroban Security Best Practices](https://soroban.stellar.org/docs/learn/security)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [cargo-audit Documentation](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-geiger Documentation](https://github.com/geiger-rs/cargo-geiger)
