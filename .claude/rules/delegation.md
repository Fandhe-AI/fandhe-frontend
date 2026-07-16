# 委譲原則（調査・設計フェーズ）

## 目的

main セッションのコンテキスト消費を抑え、指揮・統合・ユーザー対話に専念する。
ファイルの読み込みを伴う調査は原則 sub-agent へ委譲し、main は**要約のみ**を受け取る。

## 基本原則

1. **2 ファイル以上の読み込みが見込まれる調査は委譲する**（explorer）
2. **外部仕様（Rust / WASM / Web 標準 / 依存クレート）の調査は委譲する**（reference-researcher）
3. main が直接 Read してよいのは、パスが確定した 1〜2 ファイルの特定箇所を確認する場合のみ
4. 委譲時は「目的・対象パス・返してほしい形式（要約・file:line 付き）」をプロンプトに明記する
5. 独立した調査は**並列に**委譲する

## パスベース切り替え（調査）

| 対象 | 委譲先 |
|------|--------|
| リポ内コード・`docs/spec/` の横断調査 | explorer（sonnet） |
| Rust / wasm-bindgen / web-sys / axum / cargo-deny 等の外部仕様 | reference-researcher（sonnet） |
| アーキテクチャ設計・クレート境界の横断判断 | opus / fable（Plan Agent または main で慎重に） |

## 設計フェーズ

- 実装方針の設計は create-plan スキル（`_/local-plans/`）または implement-issue の計画立案を使用する
- 大規模設計・複数クレートにまたがる横断判断のみ opus / fable を使用する
- 設計前の現状把握は必ず explorer に委譲してから行う
