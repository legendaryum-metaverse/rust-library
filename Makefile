
all: prettier format lint-fix test
.PHONY: all

format:
	@cargo fmt --all
.PHONY: format-fix

format-check:
	@cargo fmt --all -- --check
.PHONY: format

lint:
	@cargo clippy --all -- -D warnings
.PHONY: lint

lint-fix:
	@cargo clippy --all --fix --allow-dirty --allow-staged
.PHONY: lint-fix

prettier:
	@bun i && bun run format
.PHONY: prettier

test:
	@bash scripts/test.sh
.PHONY: test

test-compile:
	@cargo test -p legend-saga --no-run --locked
.PHONY: test-compile
