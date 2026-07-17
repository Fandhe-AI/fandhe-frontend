# unsafe 境界ポリシーと使用箇所一覧

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-2（メモリ安全なコアランタイム）の受け入れ基準（`docs/spec/04-requirements.md`）が要求する
「`unsafe` を使用するコード（WASM バインディング層・FFI 依存クレート）が、コアクレートから分離された箇所に限定され、
ドキュメント上で明示されること」を満たすための成果物です。TASK-2.2（親 Issue: TASK-2.2 系列）の一部として、
TASK-2.2a（コード側の unsafe 使用箇所の洗い出し・WASM/FFI 境界への分離）と対をなし、
本ドキュメント（TASK-2.2b）が一覧・ポリシーの明文化を担当します。

PoC-2 の脅威モデルの結論は次のとおりです。コア（`rws-core` / `rws-interactive`）を safe Rust に収める限り、
ネイティブアドオン相当の攻撃面（任意メモリアクセス・バッファオーバーフロー等）はコア自体には持ち込まれません。
ただし、WASM バインディング層（`wasm-bindgen` 等）や FFI 依存クレートの内部実装に含まれる `unsafe` は、
本フレームワークの実装方針だけでは解消できない残存リスクとして扱います（第 4 節参照）。

## 2. unsafe 許容ポリシー（クレート別マトリクス）

| クレート | 方針 | 根拠 |
|---------|------|------|
| `core`（rws-core） | `unsafe` を全面禁止 | `#![forbid(unsafe_code)]` を `core/src/lib.rs` に設定済み。REQ-2 受け入れ基準の中核 |
| `interactive`（rws-interactive） | `unsafe` を全面禁止 | `#![forbid(unsafe_code)]` を `interactive/src/lib.rs` に設定済み（TASK-11.1a）。REQ-2 受け入れ基準を `core` と同様に満たす |
| `app` / `server`（rws-app / rws-server） | 原則 `unsafe` 禁止（safe Rust で実装） | 未作成クレート。SSR/SSG/ルーティングはアプリケーション層であり、FFI 境界を持たない前提。作成時に `forbid(unsafe_code)` の要否を判断し本表へ追記する |
| `wasm-client` / `wasm-full`（未作成） | フレームワーク自作コードは safe Rust。`unsafe` は wasm-bindgen 等の FFI 依存クレート内部・自動生成グルーコードに限定して許容 | ブラウザ DOM とのバインディングは `wasm-bindgen` の生成コードに委譲し、自作コード側で `unsafe` を新規に書かない方針とする |

未作成のクレートについては、作成時にこの表へ実際の `forbid` 設定・依存クレートの実態を追記すること
（本ドキュメントを「計画中」のまま放置しない）。

## 3. unsafe 使用箇所一覧（インベントリ）

**現時点（2026-07-16 時点の main）: ワークスペース内の `unsafe` 使用箇所は 0 件。**

`core` / `interactive` に `#![forbid(unsafe_code)]` が設定されているため、両クレート内での `unsafe` 使用は
コンパイルエラーとして機械的に禁止されています。`wasm-client` / `wasm-full` は本ドキュメント時点で未作成の
ため、インベントリは空です。

### 一覧テーブル雛形

クレートが増え `unsafe` ブロックが導入された際は、以下の形式で追記します。

| クレート | ファイル:行 | SAFETY 根拠概要 | 監査日 | 監査者 |
|---------|------------|-----------------|--------|--------|
| （例）wasm-client | `wasm-client/src/dom.rs:42` | `// SAFETY:` コメントの要約を記載 | YYYY-MM-DD | reviewer/security-auditor |

### 機械確認手順

以下のコマンドで、コード実態と本ドキュメントの記載が乖離していないか確認できます。

```bash
# 実際の unsafe コードブロック（unsafe fn / unsafe impl / unsafe trait / unsafe { ... }）の網羅的検索
# （core・interactive 等、既存クレートを対象。素朴な `grep -rn "unsafe" core/src/` では
# `#![forbid(unsafe_code)]` 属性行やドキュメンテーションコメント中の "unsafe" という語まで
# ヒットしてしまい、本節の「0 件」という記述と字義通りには一致しないため、
# 実コードとしての unsafe 使用箇所に絞り込んだパターンを使用する）
grep -rnE '\bunsafe\s*(fn|impl|trait|\{)' core/src/

# forbid(unsafe_code) 属性の存在確認
grep -n "forbid(unsafe_code)" core/src/lib.rs
```

TASK-2.1（`forbid` の CI 強制）により、`.github/workflows/ci.yml` の `forbid-unsafe` ジョブが
PR・main への push のたびに上記と同等の検証を自動実行します（`cargo test -p rws-core --test
unsafe_boundary` の実行に加え、`RUSTFLAGS='-F unsafe_code' cargo check --workspace` による
ビルド時 lint 強制、`cargo test --workspace` による XSS 回帰テストの実行を含みます）。
本ドキュメントの手動一覧は、CI による機械検証を補完するものであり、置き換えるものではありません。

## 4. FFI 依存クレートの残存リスク

将来導入予定の `wasm-bindgen` / `js-sys` / `web-sys` などの FFI 依存クレートは、内部実装に `unsafe` を含みます。
これらのクレート自体、および `build.rs`・手続きマクロに由来する任意コード実行リスクは、
本フレームワークの実装方針（コアを safe Rust に収める・`forbid(unsafe_code)` を設定する）だけでは解消されません。

`docs/spec/04-requirements.md` の制約に従い、本ドキュメントは「Rust だから完全にメモリ安全である」という
一般化した安全性主張を行いません。メモリ安全性の保証範囲は `core` / `interactive`（`forbid(unsafe_code)` を
設定したクレート）に限定され、WASM バインディング層・FFI 依存クレートは残存リスクとして利用者に開示されるべき
対象です。

依存クレートの追加時は `.claude/rules/coding-rust.md` および `.claude/rules/security.md` に従い、
`cargo metadata` による影響確認・依存グラフ上限（60 件以内・深さ 6 以内）の遵守・ユーザー承認を必須とします。

## 5. 更新・運用ルール

`unsafe` を新規に書く必要が生じた場合は、以下のフローに従います。

1. **境界の限定**: `unsafe` は WASM バインディング層・FFI 境界に該当するクレート（`wasm-client` / `wasm-full` 等）
   に限定する。`core` / `interactive` への追加は `#![forbid(unsafe_code)]` によりビルド自体が失敗するため、
   構造的に不可能である。
2. **SAFETY コメント必須**: `.claude/rules/code-comment-style.md` に従い、`unsafe` ブロックには安全性の根拠を
   `// SAFETY:` コメントとして必ず記載する。
3. **本ドキュメントへの追記**: 第 3 節の一覧テーブルに、ファイル・行・SAFETY 根拠概要・監査日・監査者を追記する。
4. **レビュー必須**: security-auditor によるレビューを経ること（`.claude/rules/security.md` の PR 前必須チェック）。
5. **CI 強制との関係**: TASK-2.1 で導入した `.github/workflows/ci.yml` の `forbid-unsafe` ジョブが、
   `core` / `interactive` への `unsafe` 混入を PR・main への push のたびに自動的に検出する。
   本ドキュメントの一覧は、CI が対象としない `wasm-client` 等の許容領域における人手の追跡台帳として機能する。
