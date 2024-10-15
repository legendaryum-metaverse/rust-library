
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
	@docker compose -f ./scripts/prettier/compose.prettier.yml run --rm prettier
.PHONY: prettier

prettier-build:
	@docker compose -f ./scripts/prettier/compose.prettier.yml --progress=plain build prettier
.PHONY: prettier-build

test:
	@bash scripts/test.sh
.PHONY: test