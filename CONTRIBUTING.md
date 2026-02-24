# Contributing to MERKLITH

Thank you for your interest in contributing to MERKLITH! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Review Process](#review-process)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our commitment to:

- **Be respectful**: Treat everyone with respect. Healthy debate is encouraged, but harassment is not tolerated.
- **Be constructive**: Provide constructive feedback and be open to receiving it.
- **Be collaborative**: Work together towards common goals.
- **Be professional**: Maintain professionalism in all interactions.

## Getting Started

### Prerequisites

- **Rust**: Version 1.75 or higher
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Git**: For version control
  ```bash
  git clone https://github.com/merklith/merklith.git
  cd merklith
  ```

- **System dependencies**:
  - Linux: `build-essential`, `pkg-config`, `libssl-dev`
  - macOS: Xcode command line tools
  - Windows: Visual Studio Build Tools

### Build the Project

```bash
# Build all crates
cargo build --release

# Run tests
cargo test --lib

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets -- -D warnings
```

## Development Workflow

### 1. Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/merklith.git
cd merklith

# Add upstream remote
git remote add upstream https://github.com/original/merklith.git
```

### 2. Create a Branch

```bash
# Create a feature branch
git checkout -b feature/my-feature

# Or for bug fixes
git checkout -b fix/bug-description
```

**Branch naming conventions**:
- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code refactoring
- `test/description` - Test additions

### 3. Make Changes

- Write clean, well-documented code
- Follow the coding standards (see below)
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run all tests
cargo test --lib

# Run specific crate tests
cargo test -p merklith-types
cargo test -p merklith-core

# Run with output
cargo test --lib -- --nocapture

# Run benchmarks
cargo bench
```

### 5. Commit Your Changes

```bash
# Stage changes
git add .

# Commit with descriptive message
git commit -m "feat: add new consensus mechanism

- Implement Proof of Contribution scoring
- Add contribution tracking for validators
- Update tests for new logic

Closes #123"
```

**Commit message format** (Conventional Commits):
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `chore`: Build process, dependencies

**Examples**:
```
feat(consensus): implement PoC validator selection

fix(storage): resolve deadlock in state persistence

docs(api): add examples for RPC methods

test(crypto): add benchmarks for signature verification
```

### 6. Push and Create PR

```bash
# Push to your fork
git push origin feature/my-feature

# Create pull request on GitHub
```

## Coding Standards

### Rust Style Guide

**Formatting**:
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --all -- --check
```

**Key formatting rules**:
- 4 spaces for indentation (no tabs)
- 100 character line limit
- Trailing commas in multi-line structures
- Spaces around operators

**Example**:
```rust
// Good
pub fn calculate_score(
    contributions: &[Contribution],
    block_height: u64,
) -> Result<Score, Error> {
    let total = contributions
        .iter()
        .map(|c| c.points)
        .sum();
    
    Ok(Score::new(total, block_height))
}

// Bad
pub fn calculate_score(contributions:&[Contribution],block_height:u64)->Result<Score,Error>{
    let total=contributions.iter().map(|c|c.points).sum();
    Ok(Score::new(total,block_height))
}
```

### Code Quality

**Linting**:
```bash
cargo clippy --all-targets -- -D warnings
```

**Common clippy rules**:
- No unused imports
- No unwrap() in production code (use ? or expect())
- Proper error handling
- Documentation for public APIs

### Error Handling

Use `Result` and `Option` appropriately:

```rust
// Good - use ? operator
pub fn process_transaction(tx: &Transaction) -> Result<Receipt, Error> {
    validate_transaction(tx)?;
    let receipt = execute_transaction(tx)?;
    Ok(receipt)
}

// Good - handle error explicitly
match validate_transaction(tx) {
    Ok(()) => {},
    Err(e) => {
        log::error!("Invalid transaction: {}", e);
        return Err(e);
    }
}

// Bad - unwrap in production
let result = validate_transaction(tx).unwrap(); // Don't do this
```

### Documentation

Document all public APIs:

```rust
/// Validates a transaction before execution.
///
/// # Arguments
///
/// * `tx` - The transaction to validate
/// * `state` - Current blockchain state
///
/// # Returns
///
/// Returns `Ok(())` if valid, or an error describing why invalid.
///
/// # Examples
///
/// ```
/// let tx = Transaction::new(...);
/// let result = validate_transaction(&tx, &state);
/// assert!(result.is_ok());
/// ```
pub fn validate_transaction(
    tx: &Transaction,
    state: &State,
) -> Result<(), ValidationError> {
    // Implementation
}
```

### Testing

Write tests for all new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transaction() {
        let tx = create_test_transaction();
        let state = create_test_state();
        
        assert!(validate_transaction(&tx, &state).is_ok());
    }

    #[test]
    fn test_insufficient_balance() {
        let tx = create_test_transaction_with_high_value();
        let state = create_test_state_with_low_balance();
        
        let result = validate_transaction(&tx, &state);
        assert!(matches!(result, Err(ValidationError::InsufficientBalance)));
    }

    #[test]
    fn test_zero_amount() {
        let tx = create_test_transaction_with_zero_amount();
        let state = create_test_state();
        
        assert!(validate_transaction(&tx, &state).is_err());
    }
}
```

**Test coverage goals**:
- Minimum 80% coverage for new code
- 100% coverage for critical paths (consensus, crypto)
- Integration tests for API endpoints

## Testing

### Running Tests

```bash
# All tests
cargo test --lib

# Specific crate
cargo test -p merklith-types

# With output
cargo test --lib -- --nocapture

# Specific test
cargo test test_valid_transaction

# Integration tests
cargo test --test integration
```

### Test Organization

```
crate/
├── src/
│   └── lib.rs
└── tests/
    ├── unit_tests.rs       # Unit tests
    ├── integration_tests.rs # Integration tests
    └── fixtures/           # Test data
        └── genesis.json
```

### Writing Good Tests

1. **One concept per test**
2. **Clear names**: `test_what_is_being_tested`
3. **Arrange-Act-Assert** structure
4. **Use helpers** for common setup

```rust
#[test]
fn test_contribution_score_calculation_with_multiple_contributions() {
    // Arrange
    let contributions = vec![
        Contribution::new(ContributionType::BlockProduction, 100),
        Contribution::new(ContributionType::Attestation, 10),
        Contribution::new(ContributionType::Attestation, 10),
    ];
    let calculator = ScoreCalculator::new();
    
    // Act
    let score = calculator.calculate(&contributions);
    
    // Assert
    assert_eq!(score.total, 120);
    assert_eq!(score.block_production, 100);
    assert_eq!(score.attestations, 20);
}
```

## Documentation

### Documentation Structure

```
merklith/
├── docs/
│   ├── CLI_GUIDE.md      # CLI usage
│   ├── API.md            # RPC API reference
│   ├── EXPLORER.md       # TUI explorer guide
│   ├── ARCHITECTURE.md   # System design
│   └── examples/         # Code examples
├── README.md             # Main readme
├── CHANGELOG.md          # Version history
└── CONTRIBUTING.md       # This file
```

### Documentation Standards

- Use clear, concise language
- Include code examples
- Keep information up-to-date
- Use proper Markdown formatting

## Submitting Changes

### Pull Request Process

1. **Update documentation**: If your change affects user-facing features
2. **Add tests**: Ensure all tests pass
3. **Update CHANGELOG.md**: Add entry under Unreleased section
4. **Create PR**: Use PR template

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation

## Testing
- [ ] All tests pass
- [ ] Added new tests
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] CHANGELOG.md updated

## Related Issues
Fixes #123
```

### Review Process

1. **Automated checks**: CI must pass
2. **Code review**: At least 2 approvals required
3. **Testing**: Verified by reviewer
4. **Merge**: Squash and merge by maintainer

**Review criteria**:
- Code quality and style
- Test coverage
- Documentation completeness
- Performance impact
- Security considerations

## Community

### Communication Channels

- **Website**: https://merklith.com
- **GitHub**: https://github.com/merklith/merklith - Issues, discussions, and contributions
- **GitHub Issues**: Bug reports and feature requests

### Getting Help

1. Check [documentation](docs/)
2. Search [existing issues](https://github.com/merklith/merklith/issues)
3. Create new issue with template

### Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in commit history

## Development Tips

### IDE Setup

**VS Code extensions**:
- rust-analyzer
- CodeLLDB (debugging)
- Better TOML
- Markdown All in One

**IntelliJ/RustRover**:
- Built-in Rust support
- Excellent debugging

### Debugging

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-gdb ./target/debug/merklith-node

# Or use VS Code debugger
# See .vscode/launch.json
```

### Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
flamegraph -- ./target/release/merklith-node

# View results
open flamegraph.svg
```

### Common Issues

**Compilation errors**:
```bash
# Clean and rebuild
cargo clean
cargo build --release

# Update dependencies
cargo update
```

**Test failures**:
```bash
# Run single test with output
cargo test test_name -- --nocapture

# Check for race conditions
cargo test --lib -- --test-threads=1
```

## Resources

### Learning Rust

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

### Blockchain Development

- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf)
- [Mastering Ethereum](https://github.com/ethereumbook/ethereumbook)
- [Rust in Blockchain](https://rustinblockchain.org/)

### MERKLITH Resources

- [Architecture](docs/ARCHITECTURE.md)
- [API Reference](docs/API.md)
- [CLI Guide](docs/CLI_GUIDE.md)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (Apache 2.0 OR MIT).

---

Thank you for contributing to MERKLITH! Together we're building the future of blockchain infrastructure.