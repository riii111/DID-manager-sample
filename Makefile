.PHONY: dev dev-build dev-clean dev-re dev-re-n logs db-setup shell

# 開発環境の制御
dev-build:
	docker compose -f docker-compose.dev.yml build

dev-build-n:
	docker compose -f docker-compose.dev.yml build --no-cache

dev-up:
	docker compose -f docker-compose.dev.yml up -d

dev-clean:
	docker compose -f docker-compose.dev.yml down -v

# 通常の再起動（キャッシュあり）
dev-re:
	docker compose -f docker-compose.dev.yml down -v
	docker compose -f docker-compose.dev.yml build
	docker compose -f docker-compose.dev.yml up -d

# キャッシュなしでの再起動
dev-re-n:
	docker compose -f docker-compose.dev.yml down -v
	docker compose -f docker-compose.dev.yml build --no-cache
	docker compose -f docker-compose.dev.yml up -d

# ログ表示
logs:
	docker compose -f docker-compose.dev.yml logs -f

# データベース関連
db-setup:
	docker compose -f docker-compose.dev.yml exec -T db psql -U miax -d miax_dev -f ./db/setup.sql

# 開発用シェル
shell:
	docker compose -f docker-compose.dev.yml exec miax-dev bash
