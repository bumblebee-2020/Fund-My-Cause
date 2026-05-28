.PHONY: help build-contract test-contract deploy-testnet dev-frontend test-frontend lint install-deps clean format build test docs docs-open docs-check

# Default target - show help
help:
	@echo "Fund-My-Cause - Common Developer Commands"
	@echo ""
	@echo "Available targets:"
	@echo "  make build              Build all (contracts + frontend)"
	@echo "  make test               Run all tests (contracts + frontend)"
	@echo "  make build-contract     Build Rust contracts to WebAssembly"
	@echo "  make test-contract      Run all Rust contract tests"
	@echo "  make deploy-testnet     Deploy contract to Stellar testnet"
	@echo "  make dev-frontend       Start frontend development server"
	@echo "  make test-frontend      Run frontend tests"
	@echo "  make lint               Run linters (Rust clippy + ESLint)"
	@echo "  make install-deps       Install all dependencies"
	@echo "  make clean              Clean build artifacts"
	@echo "  make format             Format code (Rust + JavaScript)"
	@echo "  make docs               Build rustdoc for all contracts"
	@echo "  make docs-open          Build rustdoc and open in browser"
	@echo "  make docs-check         Build rustdoc with warnings-as-errors (mirrors CI)"
	@echo ""
	@echo "Example workflow:"
	@echo "  make install-deps       # One-time setup"
	@echo "  make build"
	@echo "  make test"
	@echo "  make lint"
	@echo "  make dev-frontend"

# Build all (contracts + frontend)
build: build-contract
	@echo "All builds complete"

# Run all tests
test: test-contract test-frontend
	@echo "All tests complete"

# Build Rust contracts to WebAssembly
build-contract:
	@echo "Building Rust contracts..."
	cargo build --release --target wasm32-unknown-unknown

# Run all Rust contract tests
test-contract:
	@echo "Testing Rust contracts..."
	cargo test --workspace

# Deploy contract to Stellar testnet
# Usage: make deploy-testnet CREATOR=<account> TOKEN=<token_id> GOAL=<amount> DEADLINE=<unix_timestamp>
deploy-testnet:
	@if [ -z "$(CREATOR)" ] || [ -z "$(TOKEN)" ] || [ -z "$(GOAL)" ] || [ -z "$(DEADLINE)" ]; then \
		echo "Error: Required parameters missing"; \
		echo "Usage: make deploy-testnet CREATOR=<account> TOKEN=<token_id> GOAL=<amount> DEADLINE=<unix_timestamp>"; \
		echo ""; \
		echo "Optional parameters:"; \
		echo "  MIN_CONTRIBUTION=<amount>   (default: 1)"; \
		echo "  TITLE=<string>              (default: 'Default Title')"; \
		echo "  DESCRIPTION=<string>        (default: 'Default Description')"; \
		echo "  SOCIAL_LINKS=<json>         (default: null)"; \
		echo "  REGISTRY_ID=<contract_id>   (deploys new registry if not provided)"; \
		exit 1; \
	fi
	@./scripts/deploy.sh \
		"$(CREATOR)" \
		"$(TOKEN)" \
		"$(GOAL)" \
		"$(DEADLINE)" \
		"$(MIN_CONTRIBUTION)" \
		"$(TITLE)" \
		"$(DESCRIPTION)" \
		"$(SOCIAL_LINKS)" \
		"$(REGISTRY_ID)"

# Start frontend development server
dev-frontend:
	@echo "Starting frontend development server..."
	cd apps/interface && npm run dev

# Run frontend tests
test-frontend:
	@echo "Running frontend tests..."
	cd apps/interface && npm test

# Run linters (Rust clippy + ESLint)
lint:
	@echo "Running Rust clippy..."
	cargo clippy --workspace --all-targets
	@echo "Running ESLint on frontend..."
	cd apps/interface && npm run lint

# Install all dependencies
install-deps:
	@echo "Installing Rust dependencies..."
	cargo fetch
	@echo "Installing frontend dependencies..."
	cd apps/interface && npm install

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	cd apps/interface && rm -rf .next out

# Format code (Rust + JavaScript)
format:
	@echo "Formatting Rust code..."
	cargo fmt --all
	@echo "Formatting JavaScript/TypeScript code..."
	cd apps/interface && npm run format 2>/dev/null || npx prettier --write "src/**/*.{ts,tsx,js,jsx}" || true

# ── Documentation ─────────────────────────────────────────────────────────────

# Build rustdoc for all contracts (excludes benchmarks and third-party deps)
docs:
	@echo "Building contract documentation..."
	cargo doc --workspace --exclude benchmarks --no-deps --document-private-items
	@echo "Docs written to target/doc/"
	@echo "Entry point: target/doc/crowdfund/index.html"

# Build docs and open in the default browser
docs-open: docs
	@echo "Opening docs in browser..."
	@if command -v xdg-open > /dev/null 2>&1; then \
		xdg-open target/doc/crowdfund/index.html; \
	elif command -v open > /dev/null 2>&1; then \
		open target/doc/crowdfund/index.html; \
	else \
		start target/doc/crowdfund/index.html 2>/dev/null || \
		echo "Open target/doc/crowdfund/index.html in your browser"; \
	fi

# Build docs with warnings-as-errors — mirrors the CI docs workflow
docs-check:
	@echo "Checking contract documentation (warnings as errors)..."
	RUSTDOCFLAGS="--default-theme=ayu --cfg docsrs -D warnings" \
		cargo doc --workspace --exclude benchmarks --no-deps --document-private-items
	@echo "Documentation check passed."
