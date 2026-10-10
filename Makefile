.PHONY: test gallery screenshot

test:
	cargo test
	cargo clippy --all-targets -- -D warnings

gallery:
	cargo run --example gallery

# The docs screenshots, light and dark, with the tree rooted at this repo.
screenshot:
	PATINA_ROOT=$(CURDIR) PATINA_THEME=light PATINA_SCREENSHOT=docs/gallery-light.png cargo run --example gallery
	PATINA_ROOT=$(CURDIR) PATINA_THEME=dark PATINA_SCREENSHOT=docs/gallery-dark.png cargo run --example gallery
