.PHONY: dev dev-build dev-clean logs db-setup shell

# 開発環境の制御
dev-build:
	docker compose -f docker-compose.dev.yml build

dev-build-n:
	docker compose -f docker-compose.dev.yml build --no-cache

dev-up:
	docker compose -f docker-compose.dev.yml up -d

dev-clean:
	docker compose -f docker-compose.dev.yml down -v

# ログ表示
logs:
	docker compose -f docker-compose.dev.yml logs -f

# データベース関連
db-setup:
	docker compose -f docker-compose.dev.yml exec -T db psql -U miax -d miax_dev -f ./db/setup.sql

# 開発用シェル
shell:
	docker compose -f docker-compose.dev.yml exec miax-dev bash
