# Makefile for installing/uninstalling the MultiversX dev environment

# ============ Variables ============
TOOLCHAIN            = nightly-2023-12-11-aarch64-apple-darwin
MXPY_VERSION         = v9.5.5
SC_META_VERSION      = 0.40.1

# Mark these targets as "phony" (they don't produce actual files)
.PHONY: install uninstall

## install: Download, install toolchain, install mxpy, install multiversx-sc-meta
install:
	@echo "1. Downloading mxpy-up.py from GitHub..."
	curl -L "https://raw.githubusercontent.com/multiversx/mx-sdk-py-cli/${MXPY_VERSION}/mxpy-up.py" -o mxpy-up.py

	@echo "2. Installing Rust toolchain: $(TOOLCHAIN)"
	rustup toolchain install $(TOOLCHAIN)

	@echo "3. Setting default toolchain to $(TOOLCHAIN)"
	rustup default $(TOOLCHAIN)

	@echo "Adding WebAssembly target for Rust..."
	rustup target add wasm32-unknown-unknown

	@echo "4. Installing mxpy (MultiversX Python CLI) version $(MXPY_VERSION)..."
	python3 mxpy-up.py --exact-version=$(MXPY_VERSION) --not-interactive

	@echo "5. Installing multiversx-sc-meta version $(SC_META_VERSION)..."
	cargo install multiversx-sc-meta --version=$(SC_META_VERSION)

	@echo "6. Removing mxpy-up.py..."
	rm -f mxpy-up.py

	@echo "Setup complete!"

## uninstall: Remove/uninstall items previously installed
uninstall:
	@echo "1. Uninstalling multiversx-sc-meta..."
	cargo uninstall multiversx-sc-meta || true

	@echo "2. Uninstalling the Rust toolchain: $(TOOLCHAIN)"
	rustup toolchain uninstall $(TOOLCHAIN) || true

	@echo "Removing WebAssembly target from Rust..."
	rustup target remove wasm32-unknown-unknown

	@echo "3. Removing the ~/multiversx-sdk/ folder (mxpy installation directory)..."
	rm -rf ~/multiversx-sdk/

	@echo "Uninstall process complete!"
