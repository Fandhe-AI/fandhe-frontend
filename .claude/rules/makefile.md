---
paths:
  - "Makefile"
  - "scripts/**"
---

# Makefile / scripts 編集ルール

詳細は make skill（`.claude/skills/make`）を参照する。本ルールはこのリポジトリ固有の要点のみ記す。
`tools/hooks/**`（lefthook から呼ばれるフックスクリプト）は `git-hooks.md`、`tools/ci/**` は `ci.md` の担当であり、本ルールの対象外。

## 基本方針

- Makefile は**薄い入口**とする。レシピは `@scripts/<name>.sh` か、言語ネイティブの 1 行コマンド（`cargo build --workspace --locked`・`docker compose ...` 等）の呼び出しだけにする
- 複数行・`if` / `for`・環境変数の検証・複数コマンドの連結は `scripts/*.sh` に移す。Makefile にシェルの継続行（`\`）や `$$VAR` を増やさない
- チェック内容やバージョン pin を Makefile に二重定義しない（正は `tools/ci/ensure-gate-tools.sh`・`fw gate`・`ci.md`）
- 中身の無いターゲット（`@true` 等）は置かない。GNU Make 3.81（macOS 標準）互換を保ち、`.ONESHELL`・`.RECIPEPREFIX` 等 3.82+ 専用機能は使わない

## ターゲットの追加・改名・削除の手順

1. `.PHONY` に追加（改名・削除なら更新）する
2. ターゲット行に `## 説明` コメントを付ける（`make help` はこのコメントから生成される。無いと表示されない）
3. `scripts/help.sh` を同じ変更で更新する（Make 非依存の一覧。自動同期の仕組みは無い）
4. 呼び出し元を洗い出して更新する: `grep -rn "make " .github .claude docs README.md CLAUDE.md AGENTS.md lefthook.yml scripts tools`。既存ターゲット名・引数（`PORT` 等）・終了コードは CI・lefthook・README が前提にしているため原則変更しない
5. `make help` と `make -n <target>` で確認し、README.md / CLAUDE.md のターゲット一覧を同期する

## scripts/*.sh の規約

- 先頭は `#!/usr/bin/env bash` + 用途コメント + `set -euo pipefail`。`scripts/lib.sh` を `source` し、引数を取らないスクリプトは `reject_args` で誤用を終了コード 2 で止める
- 実行権限（`chmod +x`）を付け、`shellcheck scripts/*.sh` と `editorconfig-checker` を通す
- cwd に依存しない（`ROOT_DIR` へ `cd` する）。ユーザー向けのエラーメッセージ・ログは英語（`japanese-style.md`）、コメントは日本語または英語のどちらでもよいが 1 ファイル内で統一する
- 環境変数（`PORT` 等）は使用前に検証する。シェルコードとして再解釈させない（`security.md`）

## ターゲットの契約

| ターゲット | 契約 |
| --- | --- |
| `lint` | ソースを変更しない（`cargo fmt --all --check` + clippy `-D warnings`）。整形は `fmt` が行う |
| `fmt` | ソースを変更する唯一の整形ターゲット |
| `setup` | 再実行しても安全（冪等）。lefthook 未導入でも失敗させず案内のみ出す |
| `gate` | `fw` 未導入時は `cargo run` へフォールバックし、終了コードは `fw gate` のものをそのまま返す |
| `docs-preview` | 配信は `127.0.0.1` のみ。`PORT` は数字のみ・1〜65535 |
| `bench-cross` | ローカル専用（CI 非常設）。`bench/PROTOCOL.md` を正とする |

`clean` ターゲットは現在無い。追加する場合は `target/`・`dist/`・`_/site-preview/` など本リポジトリが生成したものだけを消し、共有ディレクトリ（`CARGO_TARGET_DIR` 等）や `_/` 配下のその他を消さない。
