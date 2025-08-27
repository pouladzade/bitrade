# CI/CD Setup

This project uses GitHub Actions for continuous integration and deployment.

## Workflows

### 1. CI (`ci.yml`)

Main continuous integration workflow that runs on every push and pull request.

**Jobs:**

- **Build and Test**: Compiles the project, runs tests, and performs code quality checks
- **Security Audit**: Runs `cargo audit` to check for known vulnerabilities
- **Dependency Updates**: Checks for outdated dependencies using `cargo outdated`

**Features:**

- PostgreSQL service container for database tests
- Rust toolchain caching for faster builds
- Code formatting and linting checks
- Release build verification

### 2. Database (`database.yml`)

Handles database migrations and schema validation.

**Triggers:**

- Changes to database migrations
- Changes to database source code
- Changes to Cargo.toml or Cargo.lock

**Jobs:**

- **Database Migrations**: Runs migrations, validates schema, and tests rollback

### 3. Code Quality (`code-quality.yml`)

Performs comprehensive code quality checks.

**Checks:**

- Code formatting with `rustfmt`
- Linting with `clippy`
- Unused dependency detection
- Dead code detection
- Workspace structure validation
- TODO/FIXME comment detection

## Local Development

To run the same checks locally:

```bash
# Install required tools
cargo install cargo-audit cargo-outdated cargo-udeps

# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Check for security vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated

# Check for unused dependencies
cargo udeps --all-targets
```

## Environment Variables

The CI workflows use the following environment variables:

- `DATABASE_URL`: PostgreSQL connection string for tests
- `CARGO_TERM_COLOR`: Enables colored output for Cargo

## Dependencies

The CI system automatically installs:

- Rust toolchain (stable)
- `protobuf-compiler`
- `libpq-dev`
- `pkg-config`
- PostgreSQL 16 (as service container)

## Caching

The CI uses GitHub Actions caching to speed up builds:

- Rust dependencies (`~/.cargo/registry`, `~/.cargo/git`)
- Build artifacts (`target/`)

## Branch Protection

Consider setting up branch protection rules for:

- `main` branch
- `develop` branch

Recommended settings:

- Require status checks to pass before merging
- Require branches to be up to date before merging
- Require pull request reviews
- Restrict pushes to matching branches

## Troubleshooting

### Common Issues

1. **Build failures due to missing dependencies**

   - Ensure all system dependencies are installed
   - Check that PostgreSQL service is running

2. **Test failures**

   - Verify database connection string
   - Check that migrations have been run

3. **Formatting issues**

   - Run `cargo fmt --all` locally
   - Check for trailing whitespace

4. **Clippy warnings**
   - Address all warnings before pushing
   - Use `cargo clippy --fix` for auto-fixable issues

### Debugging

To debug CI issues:

1. Check the workflow logs in GitHub Actions
2. Reproduce the issue locally
3. Test with the same environment (Ubuntu latest)
4. Verify all dependencies are correctly specified
