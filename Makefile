# fandhe-frontend 開発用 Makefile。
#
# 各ターゲットは既存の CI・ツール資産（`tools/ci/ensure-gate-tools.sh`・
# `fw gate`）を薄くラップするだけで、バージョン pin やチェック内容の
# 二重管理はしない（正は各スクリプト・`.claude/rules/ci.md` 側）。
#
# 使い方: `make help`（既定ターゲット）でターゲット一覧を表示する。
.DEFAULT_GOAL := help
.PHONY: help setup build test fmt lint gate bench bench-cross docs docs-preview docker-dev-build docker-dev

help: ## このヘルプを表示する
	@grep -E '^[a-zA-Z0-9_-]+:.*## ' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# lefthook はチェックサム pin なしで導入する開発支援ツール（cargo-deny とは
# 非対称）: hooks の実体はリポジトリ内 lefthook.yml / tools/hooks/ にあり、
# lefthook 自体はそれらを呼び出す薄いランナーに過ぎないため、cargo-deny と
# 同水準のバージョン + SHA256 pin は要求しない。推奨バージョンは目安として
# 案内する（`lefthook version` で実バージョンを確認できる）。
setup: ## 開発ツールを導入する（clippy/cargo-deny/wasm32 target + lefthook install）
	@bash tools/ci/ensure-gate-tools.sh
	@if command -v lefthook >/dev/null 2>&1; then \
		lefthook install; \
	else \
		echo "lefthook is not installed; recommended: v1.11.x (latest stable). see https://lefthook.dev/installation/ to install it (e.g. \`snap install lefthook --classic\` or \`cargo install lefthook\`)"; \
	fi

build: ## workspace をビルドする
	cargo build --workspace --locked

test: ## workspace のテストを実行する
	cargo test --workspace --locked

fmt: ## rustfmt で整形する
	cargo fmt --all

lint: ## fmt チェック + clippy（-D warnings）を実行する
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --locked -- -D warnings

gate: ## fw gate --project . を実行する（fw 未導入時は cargo run にフォールバック）
	@if command -v fw >/dev/null 2>&1; then \
		fw gate --project .; \
	else \
		cargo run -p fandhe-frontend-cli --locked -- gate --project .; \
	fi

# 出力は stdout の 1 行サマリのみ（JSON またはプレーンテキスト行）。レポート
# （docs/reports/perf-improvement-before-after-report.md）へは手動転記する運用で、
# 各ベンチの実体・ワークロード定義は crates/xtask/src/bench_*.rs を正とする。
bench: ## 常設ベンチマーク 3 種（bench-ssr / bench-state-update / bench-binding-update）を実行する
	cargo run -p xtask --release --locked -- bench-ssr
	cargo run -p xtask --release --locked -- bench-state-update
	cargo run -p xtask --release --locked -- bench-binding-update

# フレームワーク横断比較（SSR / CSR / payload）。手順・対象リストの正は
# bench/PROTOCOL.md。npm 依存導入（npm ci --ignore-scripts）・システム
# chromium・wasm-bindgen-cli を前提とするローカル専用ターゲット（CI 非常設、
# 理由は bench/PROTOCOL.md §5）。結果（stdout の JSON 行）は docs/reports/
# へ手動転記する。
bench-cross: ## フレームワーク横断ベンチ（bench/PROTOCOL.md 参照。要 npm ci / chromium）
	cargo run -p xtask --release --locked -- bench-ssr
	node bench/ssr/run_ssr.mjs
	bash bench/csr/fandhe/build.sh
	node bench/csr/build.mjs
	node bench/csr/run_csr.mjs
	node bench/payload/measure.mjs

docs: ## docs サイトを dist/ へビルドする
	cargo run -p fandhe-frontend-docs-site --locked -- --out dist/

# docs サイトの生成 HTML は `base_path = "/fandhe-frontend"` 付き絶対パスで
# CSS・JS・検索インデックスを参照するため file:// では解決しない。ビルド出力を
# `_/site-preview/fandhe-frontend/` へ置き、親を HTTP 配信して base_path を保つ
# （Playwright MCP のデザインレビュー用。手順は docs/guides/browser-testing.md §9a）。
# 配信は 127.0.0.1 のみ。PORT は環境変数経由（"$$PORT"）で参照し、make の
# 展開値をシェルコードとして再解釈させない（コマンド置換等によるシェル注入防止）。
# 検証は数字のみ・1〜65535 の範囲に限る。
PORT ?= 8765
export PORT
docs-preview: ## docs サイトをビルドし 127.0.0.1 でローカル配信する（PORT 既定 8765、Playwright レビュー用）
	@command -v python3 >/dev/null 2>&1 || { echo "error: python3 is required for docs-preview (http.server)" >&2; exit 1; }
	@case "$$PORT" in ''|*[!0-9]*) echo "error: PORT must be numeric" >&2; exit 1;; esac; \
	if [ "$$PORT" -lt 1 ] || [ "$$PORT" -gt 65535 ]; then echo "error: PORT must be in 1-65535" >&2; exit 1; fi
	rm -rf _/site-preview/fandhe-frontend
	cargo run -p fandhe-frontend-docs-site --locked -- --out _/site-preview/fandhe-frontend/
	@echo "serving http://127.0.0.1:$$PORT/fandhe-frontend/ (Ctrl-C to stop)"
	python3 -m http.server "$$PORT" --bind 127.0.0.1 --directory _/site-preview

docker-dev-build: ## 開発用 Docker イメージをビルドする
	docker compose -f docker/dev/compose.yml build

docker-dev: ## 開発用 Docker コンテナへ入る（bind mount + named volume）
	docker compose -f docker/dev/compose.yml run --rm dev
