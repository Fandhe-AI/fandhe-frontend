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
| `interactive`（rws-interactive） | `unsafe` を全面禁止 | `#![forbid(unsafe_code)]` を `interactive/src/lib.rs` に設定済み（TASK-11.1a 設計・TASK-11.1b で実装）。REQ-2 受け入れ基準を `core` と同様に満たす |
| `app` / `server`（rws-app / rws-server） | 原則 `unsafe` 禁止（safe Rust で実装） | 未作成クレート。SSR/SSG/ルーティングはアプリケーション層であり、FFI 境界を持たない前提。作成時に `forbid(unsafe_code)` の要否を判断し本表へ追記する |
| `wasm-client`（rws-wasm-client。TASK-6.2b/#48 で作成済み） | フレームワーク自作コードは safe Rust（`wasm-client/src/` に自作 `unsafe` ブロック 0 件）。`#![deny(unsafe_code)]` を `wasm-client/src/lib.rs` に設定済み（`#[wasm_bindgen]` 展開コードが内部で `unsafe` を含むため `forbid` は不採用）。`unsafe` は `wasm-bindgen`/`web-sys` の FFI 依存クレート内部・自動生成グルーコードに限定して許容 | `hydrate()`（`wiring` モジュール）はクロージャの寿命管理に `closure.forget()` ではなく `thread_local!` レジストリ（`wasm-client/src/registry.rs`）を用いる方式を採り、`unsafe` ブロックを要しない。`docs/api/hydration-api.md` 第 4 節・判断 6 の設計どおり |
| `wasm-full`（rws-wasm-full。TASK-11.2b/#75 で作成済み、`Runtime`/`mount()`/`hydrate()` は TASK-11.2d/#77 で実装済み） | フレームワーク自作コードは safe Rust（`wasm-full/src/` に自作 `unsafe` ブロック 0 件）。`#![deny(unsafe_code)]` を `wasm-full/src/lib.rs` に設定済み（`#[wasm_bindgen]` 展開コードが内部で `unsafe` を含むため `forbid` は不採用、`wasm-client` と同方針）。`unsafe` は `wasm-bindgen`/`web-sys` の FFI 依存クレート内部・自動生成グルーコードに限定して許容。**CI 強制済み（#155、REQ-11 受け入れ基準 2）**: `core/tests/unsafe_boundary.rs` の `DENY_UNSAFE_FFI_MEMBERS` に登録され、`.github/workflows/ci.yml` の `forbid-unsafe` ジョブが PR・main への push のたびに (a) `#![deny(unsafe_code)]` 属性の実在、(b) `wasm-full/src/` 配下の自作 `unsafe` トークン 0 件、(c) `allow(unsafe_code)` による deny 上書きが 0 件、の 3 点を機械検証する（forbid(unsafe_code) 相当の強制） | イベント委譲配線（`wasm-full/src/events.rs`）は `wasm_bindgen::closure::Closure::forget`（safe API）でリスナーを保持する方式を採り、`unsafe` ブロックを要しない。`Runtime::mount`/`Runtime::hydrate`（`wasm-full/src/lib.rs`）・アプリ側エントリポイント参照実装（`wasm-full/src/entry.rs`、`thread_local!` + `RefCell` で状態保持）も同様に自作 `unsafe` ブロックを要しない |
| `wasm-thin`（rws-wasm-thin。TASK-11.3a/#79 で作成済み） | フレームワーク自作コードは safe Rust（`wasm-thin/src/` に自作 `unsafe` ブロック 0 件）。`unsafe` は `wasm-bindgen` の FFI 依存クレート内部・自動生成グルーコードに限定して許容。`web-sys` には依存しない（DOM 操作・イベント配線は JS グルー側の責務であり、WASM 側は文字列 in・文字列 out の純粋計算に限定する設計、REQ-11） | 汎用層 `ThinRuntime<C: Component>`（`wasm-thin/src/lib.rs`）は `wasm-bindgen`/`web-sys` 非依存の純粋 Rust。境界層 `demo` モジュールの `#[wasm_bindgen]` エクスポート（`thread_local!` + `RefCell` で状態保持）も自作 `unsafe` ブロックを要しない。`RUSTFLAGS='-F unsafe_code' cargo check --workspace`／`cargo check -p rws-wasm-thin --target wasm32-unknown-unknown` のいずれも自作コード側の `#![forbid(unsafe_code)]` 相当の制約下で通過することを確認済み（`#![forbid(unsafe_code)]` の明示設定自体は wasm-bindgen 生成コードとの整合を将来 CI 強制時に判断、`wasm-full` と同方針・#155 参照） |

未作成のクレートについては、作成時にこの表へ実際の `forbid` 設定・依存クレートの実態を追記すること
（本ドキュメントを「計画中」のまま放置しない）。

### 許容 FFI 境界（wasm-full、#155）

`wasm-full` のアプリロジック層（自作コード、`wasm-full/src/`）に対して CI が強制する
forbid(unsafe_code) 相当の制約下で、なお解消されず許容される `unsafe` の境界は
以下の 3 点に限定される。これら以外に `wasm-full/src/` 自作コードで `unsafe` が
現れることは CI（`core/tests/unsafe_boundary.rs` の `DENY_UNSAFE_FFI_MEMBERS` チェック）
が拒否する。

1. **依存クレート内部の `unsafe`**: `wasm-bindgen`（0.2 系）・`web-sys`（0.3 系、
   feature: `Attr` / `Document` / `Element` / `Event` / `EventTarget` /
   `History` / `HtmlInputElement` / `Location` / `MouseEvent` /
   `NamedNodeMap` / `Window` / `console`）の各クレート自体の
   実装内部に含まれる `unsafe`。これらは `wasm-full/Cargo.toml` の依存であり、
   `wasm-full/src/` のソーステキストには現れないため、上記 CI 走査の対象外
   （= 解消不能な残存リスクとして第 4 節で開示する）
2. **`#[wasm_bindgen]` 属性マクロ展開の自動生成コード**: `#[wasm_bindgen]` を
   付与した関数・構造体からコンパイル時に生成される JS 境界のグルーコードは
   内部で `unsafe` を含むが、これは `wasm-full/src/` のソーステキスト（マクロ
   展開前）には `unsafe` トークンとして現れないため、`#![deny(unsafe_code)]` の
   lint 対象にもならず、CI 走査（ソーステキスト走査）の対象にもならない
3. **`nav.rs` のカスタム `#[wasm_bindgen] extern "C"` ブロック（イシュー #404、
   View Transitions 連携）**: `wasm-full/src/nav.rs` の `wiring` モジュールは
   `document.startViewTransition` を機能検出・呼び出しするため、`web-sys` の
   `Document::start_view_transition`（`#[cfg(web_sys_unstable_apis)]` ゲート付き、
   ワークスペース全体への `RUSTFLAGS` 汚染を招くため不採用）の代わりに独自の
   duck-typing `extern "C"` 型（`DocumentViewTransitions`）を定義し、
   `start_view_transition_prop`（getter、機能検出用）・`start_view_transition`
   （`catch` 属性付き呼び出し）の 2 メソッドを束縛する。この extern ブロックは
   上記 2 点目と同区分（`#[wasm_bindgen]` マクロ展開の自動生成コードのみが
   `unsafe` を含み、`nav.rs` のソーステキスト自体には `unsafe` トークンが
   現れない）であり、CI 走査（ソーステキスト走査）の対象にはならない

上記 3 点はいずれも `wasm-full` の自作コード（`Runtime`/`events`/`dom`/`entry`/
`hydration`/`nav` 各モジュール）が直接 `unsafe` ブロックを書くことを意味しない。
自作コード側の `unsafe` 使用箇所は本ドキュメント作成時点・CI 強制時点のいずれも
0 件である（第 3 節）。

## 3. unsafe 使用箇所一覧（インベントリ）

**現時点（2026-07-17 時点）: ワークスペース内の自作 `unsafe` 使用箇所は 0 件。**

`core` / `interactive` に `#![forbid(unsafe_code)]` が設定されているため、両クレート内での `unsafe` 使用は
コンパイルエラーとして機械的に禁止されています。`app` / `server` は本ドキュメント更新時点で
未作成のため、インベントリは空です。`wasm-full`（rws-wasm-full）は TASK-11.2b（#75）で作成済みですが、
`grep -rnE '\bunsafe\s*(fn|impl|trait|\{)' wasm-full/src/` の結果は 0 件であり、自作コード側の `unsafe`
ブロックはありません（`wasm-bindgen`/`web-sys` の FFI 依存クレート内部・自動生成コードのみが対象、第 4 節参照）。
`wasm-thin`（rws-wasm-thin）は TASK-11.3a（#79）で作成済みですが、
`grep -rnE '\bunsafe\s*(fn|impl|trait|\{)' wasm-thin/src/` の結果も同様に 0 件です
（`wasm-bindgen` の FFI 依存クレート内部・自動生成コードのみが対象。`web-sys` には依存しない）。
`wasm-client`（rws-wasm-client）は TASK-6.2b（#48）で作成済みですが、
`grep -rnE '\bunsafe\s*(fn|impl|trait|\{)' wasm-client/src/` の結果も同様に 0 件です
（`wasm-bindgen`/`web-sys` の FFI 依存クレート内部・自動生成コードのみが対象。`core/tests/unsafe_boundary.rs`
の `UNSAFE_ALLOWED_MEMBERS` に `wasm-client` が既に登録済みで、CI の `forbid-unsafe` ジョブは
本クレート追加後も `RUSTFLAGS='-F unsafe_code' cargo check --workspace` の通過を維持している）。

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

# wasm-full（deny 域、#155）: 自作コード側の unsafe コードブロックの網羅的検索
# （0 件であること。ヒットした場合は第 2 節「許容 FFI 境界」に該当しない持ち込み）
grep -rnE '\bunsafe\s*(fn|impl|trait|\{)' wasm-full/src/

# wasm-full: deny(unsafe_code) 属性の存在確認・allow による上書きが無いことの確認
grep -n "deny(unsafe_code)" wasm-full/src/lib.rs
grep -rn "allow(unsafe_code)" wasm-full/src/ || echo "allow(unsafe_code) による上書きなし"
```

TASK-2.1（`forbid` の CI 強制）により、`.github/workflows/ci.yml` の `forbid-unsafe` ジョブが
PR・main への push のたびに上記と同等の検証を自動実行します（`cargo test -p rws-core --test
unsafe_boundary` の実行に加え、`RUSTFLAGS='-F unsafe_code' cargo check --workspace` による
ビルド時 lint 強制、`cargo test --workspace` による XSS 回帰テストの実行を含みます）。
`cargo test -p rws-core --test unsafe_boundary` には TASK-2.1 時点の safe 域チェックに加え、
#155 で追加した `wasm-full`（deny 域）向けの `ffi_deny_crates_*` テストも含まれ、上記
wasm-full 用コマンドと同等の検証を PR・main への push のたびに自動実行します。
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
   `core` / `interactive`（forbid 域）への `unsafe` 混入を PR・main への push のたびに自動的に検出する。
   #155 でこれを拡張し、`wasm-full`（deny 域）のアプリロジック層（自作コード）への `unsafe` 追加・
   `allow(unsafe_code)` による deny 上書き・`deny` 属性の削除も同じジョブが自動検出する
   （`core/tests/unsafe_boundary.rs` の `DENY_UNSAFE_FFI_MEMBERS`）。
   本ドキュメントの一覧は、CI が対象としない `wasm-client` / `wasm-thin`（`UNSAFE_ALLOWED_MEMBERS`、
   スコープ外・#155 参照）等の許容領域における人手の追跡台帳として機能する。
