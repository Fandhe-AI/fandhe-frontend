---
paths:
  - "lefthook.yml"
  - "tools/hooks/**"
  - ".claude/settings.json"
---

# Git hooks（lefthook）編集ルール

## 目的

フックはコミットのたびに実行されるため、遅いフックはそのまま開発速度の低下になる。
フックはローカルでの早期検知専用であり、CI の代替ではない。**開発速度を落とさないことを最優先**し、
重い検査・網羅的な検査は CI（`.github/workflows/ci.yml`）に任せる。

## 時間予算

| フック | 予算 | 判定方法 |
| --- | --- | --- |
| pre-commit | 通常のコミット（staged 数ファイル）で合計 1 秒未満 | `lefthook run pre-commit --file <path> ...` の `summary: (done in N seconds)` |
| commit-msg | 即時（外部コマンド・ネットワーク不可） | 実コミット時に表示される commit-msg の summary（メッセージファイルを引数に取るため `--all-files` では計測しない） |
| pre-push・post-checkout・post-merge 等 | 原則追加しない | 追加する場合は理由と実測値を PR 本文に記載する |

`lefthook run pre-commit --all-files` は追跡中の全 `*.rs`（約 1,600 ファイル）を対象にする最悪ケースで、
rustfmt が `lib.rs` 等から子モジュールを辿って同じファイルを重複検査するため数秒かかる。
回帰の比較には使えるが、予算の判定は実際のコミットに近い `--file` 指定で行う。

## フックに置かないもの（→ `ci.yml` へ）

- テストスイート（`cargo test`・browser-test 等）・ビルド・`cargo check`
- clippy・cargo-deny・`fw gate` などクレート全体をコンパイル/解析する検査
- workspace 全体を走査する fmt / lint（`cargo fmt --all --check`・`make lint` の呼び出しを含む）。`cargo fmt --check -- <files>` もファイル指定に関係なく workspace 全体を検査するため、フックでは使わない
- 依存のインストール・ネットワークアクセス
- `ci.yml` のジョブと同じコマンドを同じ範囲で実行するもの
- フックスクリプト（`tools/hooks/*.sh`）自体のテスト — 追加する場合も CI のジョブとして実行する

## フックに置いてよいもの

- `{staged_files}` に限定し、`glob` / `exclude` で対象を絞った 1 秒未満の検査
- CI では検査していない、または push 後では遅いもの
    - staged diff secret scan（`lefthook.yml` 内のインラインスクリプト）: 秘密情報は push 前に止める必要がある。CI に同等の検査はない
    - commit-msg（`tools/hooks/commit-msg-check.sh`）: コミットメッセージの Conventional Commits 形式は CI では検証していない

### 明示的な例外

- pre-commit の `rustfmt --check --edition 2021 {staged_files}` は `ci.yml` の `fmt` ジョブ（`cargo fmt --all --check`）と重複するが、staged ファイル限定で 1 ファイル 0.1 秒・大きなクレートの `lib.rs` を含む 5 ファイルでも 0.94 秒のため維持する（2026-10-11 計測・決定）
    - 以前の `cargo fmt --check -- {staged_files}` は workspace 全体（約 550 ターゲット）を毎回検査し、staged 1 ファイルでも約 3.1 秒かかっていた
    - rustfmt 単体は `Cargo.toml` を読まず既定 edition が 2015 になるため `--edition` を明示している。全クレート・`templates/`・`examples/`・`bench/` の `Cargo.toml` は `edition = "2021"` で統一されている。edition を変えるクレートが出たら、下記「rustfmt の edition 指定」の箇所をすべて見直す（混在するならクレートごとに分ける）

## rustfmt の edition 指定

rustfmt を直接呼ぶ箇所は `Cargo.toml` の edition を読まないため、`--edition` を明示し、値を揃える。
edition を変更するときは以下をすべて同じ値へ更新する（`cargo fmt` は `Cargo.toml` から edition を読むため対象外）。

| 箇所 | 用途 | 現在値 |
| --- | --- | --- |
| `lefthook.yml` の pre-commit `rust: rustfmt --check (staged files)` | staged `*.rs` の整形チェック | `--edition 2021` |
| `.claude/settings.json` の PostToolUse（`Edit\|Write`） | Claude が編集した `*.rs` の自動整形 | `--edition 2021` |

PostToolUse の rustfmt は編集をブロックしない方針とする。jq・rustfmt が無い環境では何もせず exit 0、
rustfmt が失敗した（構文エラー等で整形できなかった）ときは stderr へ警告を出して exit 1（非ブロッキングエラー）で通知する。
exit 2 は Claude へのブロッキングフィードバック扱いになるため使わない。

## 手元で CI 相当を確認する

フックで強制しないだけで、ローカルでの確認の規約自体は変わらない。PR 前には必要に応じて以下を実行する。

```bash
make lint   # cargo fmt --all --check + clippy（-D warnings）
make test   # workspace のテスト
make gate   # fw gate --project .
```

## 書き方の規約

- `pre-commit` の `parallel: true` を維持する（直列化しない）
- `{staged_files}` と `glob` / `exclude` で対象を絞る
- 実体が長くなる場合は `tools/hooks/` へ切り出し、`lefthook.yml` は呼び出しのみにする
- 条件付き実行が必要な場合は lefthook の `skip` / `only` を使ってよい（`.claude/skills/lefthook/references/examples/skip.md`）
- npm 依存のツール（commitlint 等）は導入しない（REQ-12。commit-msg はシェル正規表現で検証する）

## 検査を追加するときの判断手順

1. その検査は `ci.yml` で既に実施されているか
    - 実施済みで、staged 限定にしても 1 秒以上かかる → フックに入れない
2. staged ファイルに限定できるか
    - できない → CI へ置く
3. CI へ置く場合は `ci.yml` にジョブを追加し、集約ジョブ `ci-complete` の `needs` にも必ず追加する
    - Runner・ツール導入の方針は `.claude/rules/ci.md` に従う
4. 編集後に以下を実行し、summary の合計時間をユーザーへ報告する（通常のコミット相当で 1 秒以上なら構成を見直す）

```bash
lefthook run pre-commit --file crates/core/src/lib.rs
time lefthook run pre-commit --all-files
```

## バイパス禁止

遅いからといって `--no-verify` や `LEFTHOOK=0` で回避しない（`.claude/rules/conventional-commits.md`）。
遅いフックはフック側を直す（検査を CI へ移す・staged 限定にする・対象 glob を絞る）。
