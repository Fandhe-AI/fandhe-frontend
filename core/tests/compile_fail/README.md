# compile_fail フィクスチャ

このディレクトリのファイルは **意図的にコンパイル不能** な `fandhe-frontend-core` 利用例です。
TASK-5.3（`docs/design/compile-error-review.md`）のためのレビュー材料であり、
`docs/spec/04-requirements.md` REQ-5 受け入れ基準 3「コンパイルエラーが
マクロ展開後のコードを指す読みにくいメッセージではなく、通常の Rust の
型エラーとして表示されること」を検証するために作成しました。

## ビルド・テストへの影響

`cargo` はデフォルトで `tests/*.rs`（直下のファイルのみ）を個別のテスト
バイナリとしてコンパイルします。本ディレクトリは `tests/compile_fail/`
というサブディレクトリであり、直下の `.rs` ファイルではないため
**cargo のテストターゲットに含まれず、`cargo test` / `cargo build` に
一切影響しません**。この性質は `cargo test --workspace --locked` が
通過することで確認済みです（`docs/design/compile-error-review.md` 内の検証結果参照）。

## 再現手順

各ファイルは単独では `core` クレートの一部としてコンパイルされないため、
実際にエラーを再現するには一時的な検証クレートを作成し、対象ファイルの
内容を `src/lib.rs` に配置して `cargo check` を実行します。

```bash
mkdir -p /tmp/compile-error-check/src
cat > /tmp/compile-error-check/Cargo.toml <<'EOF'
[package]
name = "compile-error-check"
version = "0.1.0"
edition = "2021"

[dependencies]
fandhe-frontend-core = { path = "<このリポジトリの絶対パス>/core" }
EOF
cp core/tests/compile_fail/case01_child_type_mismatch.rs /tmp/compile-error-check/src/lib.rs
cd /tmp/compile-error-check && cargo check
```

各ケースの実測出力・評価は `docs/design/compile-error-review.md` を参照してください。

## ケース一覧

| ファイル | 想定エラー | 観点 |
|---------|-----------|------|
| `case01_child_type_mismatch.rs` | E0308 | `Vec<Node>` 期待箇所に `Node` を渡す |
| `case02_into_string_not_implemented.rs` | E0277 | `Into<String>` 未実装型を渡す |
| `case03_dynamic_tag_name.rs` | E0597 | タグ名 `&'static str` 制約と動的 `String` |
| `case04_attr_tuple_type_mismatch.rs` | E0308 | 属性値 `(&str, &str)` に数値を渡す |
| `case05_children_vec_type_mismatch.rs` | E0308 | 子 `Vec<Node>` に `&str` を混在 |
| `case06_nonexistent_variant.rs` | E0599 | 存在しない enum バリアント参照 |
| `case07_missing_reference.rs` | E0308 | `&Node` 引数への参照渡し忘れ |
