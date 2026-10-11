# fandhe-frontend 開発用 Makefile。
#
# Make は薄い入口であり、各レシピは scripts/*.sh または言語ネイティブの 1 行コマンド
# （cargo / docker compose）の呼び出しに限定する。複数行・条件分岐を要する処理の定義は
# scripts/ 側にのみ置き、ここでチェック内容を二重定義しない（バージョン pin やチェック
# 内容の正は tools/ci/ensure-gate-tools.sh・`fw gate`・`.claude/rules/ci.md` 側）。
# Make を使わない場合の同等コマンドは scripts/help.sh を参照。
# 変更時は .claude/rules/makefile.md に従う（make skill: .claude/skills/make）。
#
# GNU Make 3.81（macOS 標準）互換。3.82+ 専用機能（.ONESHELL・.RECIPEPREFIX 等）は使わない。
#
# 使い方: `make help`（既定ターゲット）でターゲット一覧を表示する。
# 一覧は下記の `## ` コメントから生成されるため、`## ` の無いターゲットは表示されない。
# ターゲットを追加・改名・削除したら scripts/help.sh も同じ変更で更新すること。
# 中身の無いターゲット（`@true` 等）は呼び出し元が終了コードを信用してしまうため置かない。

.DEFAULT_GOAL := help

.PHONY: help setup build test fmt lint gate bench bench-cross docs docs-preview docker-dev-build docker-dev

help: ## このヘルプを表示する（Make 非依存の一覧: scripts/help.sh）
	@grep -E '^[a-zA-Z0-9_-]+:.*## ' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

setup: ## 開発ツールを導入する（clippy/cargo-deny/wasm32 target + lefthook install）。直接: scripts/setup.sh
	@scripts/setup.sh

build: ## workspace をビルドする
	cargo build --workspace --locked

test: ## workspace のテストを実行する
	cargo test --workspace --locked

fmt: ## rustfmt で整形する（ソースを変更する）
	cargo fmt --all

lint: ## fmt チェック + clippy（-D warnings）を実行する（ソースは変更しない）。直接: scripts/lint.sh
	@scripts/lint.sh

gate: ## fw gate --project . を実行する（fw 未導入時は cargo run にフォールバック）。直接: scripts/gate.sh
	@scripts/gate.sh

bench: ## 常設ベンチマーク 3 種（bench-ssr / bench-state-update / bench-binding-update）を実行する。直接: scripts/bench.sh
	@scripts/bench.sh

bench-cross: ## フレームワーク横断ベンチ（bench/PROTOCOL.md 参照。要 npm ci / chromium）。直接: scripts/bench-cross.sh
	@scripts/bench-cross.sh

docs: ## docs サイトを dist/ へビルドする
	cargo run -p fandhe-frontend-docs-site --locked -- --out dist/

# PORT は環境変数として scripts/docs-preview.sh へ渡す（`make docs-preview PORT=9000`）。
# 検証（数字のみ・1〜65535）はスクリプト側で行い、make の展開値をシェルコードとして再解釈させない。
PORT ?= 8765
export PORT
docs-preview: ## docs サイトをビルドし 127.0.0.1 でローカル配信する（PORT 既定 8765）。直接: scripts/docs-preview.sh
	@scripts/docs-preview.sh

docker-dev-build: ## 開発用 Docker イメージをビルドする
	docker compose -f docker/dev/compose.yml build

docker-dev: ## 開発用 Docker コンテナへ入る（bind mount + named volume）
	docker compose -f docker/dev/compose.yml run --rm dev
