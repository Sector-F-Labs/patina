.DEFAULT_GOAL := help

.PHONY: help gallery dev build test check doc screenshot clean

help: ## Show this help
	@echo "Usage: make <target>"
	@echo
	@awk 'BEGIN { FS = ":.*## " } /^[a-z-]+:.*## / { printf "  \033[1m%-14s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

gallery: ## Show every component in the gallery example
	cargo run --example gallery

dev: ## Run the gallery with live reload (requires cargo-watch)
	@command -v cargo-watch >/dev/null 2>&1 || { echo "cargo-watch is required: cargo install cargo-watch"; exit 1; }
	cargo watch -x "run --example gallery"

build: ## Build the library and the gallery
	cargo build --all-targets

test: ## Run the tests
	cargo test

check: test ## Run the tests and clippy, as before a release
	cargo clippy --all-targets -- -D warnings

doc: ## Build the API docs and open them
	cargo doc --no-deps --open

# The docs screenshots, light and dark, with the tree rooted at this repo.
screenshot: ## Regenerate the docs/gallery-*.png screenshots
	PATINA_ROOT=$(CURDIR) PATINA_SKIN=light PATINA_SCREENSHOT=docs/gallery-light.png cargo run --example gallery
	PATINA_ROOT=$(CURDIR) PATINA_SKIN=dark PATINA_SCREENSHOT=docs/gallery-dark.png cargo run --example gallery
	PATINA_ROOT=$(CURDIR) PATINA_SKIN="Dark Orange" PATINA_SCREENSHOT=docs/gallery-dark-orange.png cargo run --example gallery

clean: ## Remove build output
	cargo clean
