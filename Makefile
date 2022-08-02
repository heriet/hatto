bash:
	docker compose exec dev bash

build-dev:
	docker compose exec dev cargo build

clippy:
	docker compose exec dev cargo clippy
