STATIC_DIST = ./deploy-static
WEBSITE = website
WEBSITE_DIST = $(WEBSITE)/public

default: ## Debug build
	cargo build

clean: ## Clean all build artifacts and dependencies
	@-/bin/rm -rf target/
	@-/bin/rm -rf database/migrator/target/
	@-/bin/rm -rf database/*/target/
	@-/bin/rm -rf database/*/tmp/
	@-/bin/rm -rf $(WEBSITE)
	@-/bin/rm -rf $(STATIC_DIST)
	@cargo clean

coverage: migrate ## Generate coverage report in HTML format
	cargo tarpaulin -t 1200 --out Html --skip-clean  --all-features --no-fail-fast --workspace=database/db-sqlx-postgres,database/db-sqlx-sqlite,.

dev-env: ## Download development dependencies
	cargo fetch

doc: ## Prepare documentation
	@-/bin/rm -rf $(STATIC_DIST) || true
	@cargo doc --no-deps --workspace --all-features
	@-mkdir -p $(WEBSITE)/static/doc || true
	cp -r target/doc $(WEBSITE)/static/doc
	@./scripts/ci.sh build
	mkdir -p $(STATIC_DIST)
	cp -r  $(WEBSITE_DIST)/* $(STATIC_DIST)

docker: ## Build docker images
	docker build -t realaravinth/gitpad:master -t realaravinth/gitpad:latest .

docker-publish: docker ## Build and publish docker images
	docker push realaravinth/gitpad:master 
	docker push realaravinth/gitpad:latest

lint: ## Lint codebase
	cargo fmt -v --all -- --emit files
	cargo clippy --workspace --tests --all-features

release: ## Release build
	cargo build --release

run: default ## Run debug build
	cargo run

migrate: ## run migrations
	@-rm -rf database/db-sqlx-sqlite/tmp && mkdir database/db-sqlx-sqlite/tmp
	cd database/migrator && cargo run

sqlx-offline-data: ## prepare sqlx offline data
	cargo sqlx prepare  --database-url=${POSTGRES_DATABASE_URL} -- --bin gitpad \
		--all-features

test: migrate ## Run tests
	cd database/db-sqlx-postgres &&\
		DATABASE_URL=${POSTGRES_DATABASE_URL}\
		cargo test --no-fail-fast
	cd database/db-sqlx-sqlite &&\
		DATABASE_URL=${SQLITE_DATABASE_URL}\
		cargo test --no-fail-fast
	cargo test

xml-test-coverage: migrate ## Generate cobertura.xml test coverage
	cargo tarpaulin -t 1200 --out Xml --skip-clean --all-features --no-fail-fast --workspace=database/db-sqlx-postgres,database/db-sqlx-sqlite,.

help: ## Prints help for targets with comments
	@cat $(MAKEFILE_LIST) | grep -E '^[a-zA-Z_-]+:.*?## .*$$' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
