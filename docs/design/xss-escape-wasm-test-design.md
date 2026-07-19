# WASM 経路 XSS テスト設計（TASK-1.3a）

## 1. 目的とトレーサビリティ

- `docs/spec/05-tasks.md` の TASK-1.3「WASM 経由イベント処理・DOM 更新のエスケープ
  一貫性検証」（REQ-1・REQ-11 連携、親イシュー #90）は、PoC-5 の WASM 完全方式実績を
  踏まえ、クライアント WASM のイベント処理・DOM 更新を経由した出力にも SSR/SSG/CSR と
  同一の既定エスケープ保証が及ぶことを検証する回帰テストを追加するタスクである。
  前提タスクは TASK-1.1（完了・#7 系）・TASK-11.1（完了・#69）。
- TASK-1.3 は 4h 粒度の 2 段階に分割されている。

  - **TASK-1.3a（本ドキュメント・#91）**: テスト観点の設計確定
  - **TASK-1.3b（#92）**: 本書に従ったテスト実装・CI 統合

- **本文書のステータス**: TASK-1.3a 確定版。実装（#92）と本書の記述に乖離が生じた
  場合は本書を正として PR レビューで指摘する。

- **本タスクのスコープ**: テスト観点の設計確定書の作成のみ（docs-only 変更）。
  テストファイルの新規作成・`Cargo.toml` の依存追加・CI ワークフローの変更は
  TASK-1.3b（#92）のスコープであり、本タスクでは行わない。`docs/spec/` は
  サブモジュールのため編集禁止。

本書は `docs/design/plain-html-output-test-design.md`（TASK-5.2a）・
`docs/design/wasm-full-architecture.md`（TASK-11.2a）・`docs/api/hydration-state-format.md`
（TASK-11.4a）と同じ `docs/` 直下フラット配置とし、「本文書のステータス」
「トレーサビリティ」「凍結表」「スコープ外表」「セキュリティ不変条件」
「受け入れ基準対応表」を備える形式に揃える。

## 2. 成果物配置のマッピング判断（最重要）

仕様（`docs/spec/05-tasks.md` TASK-1.3）が示す成果物パス
`crates/wasm-client/tests/xss_escape_wasm.rs` は計画時点のクレート名であり、現行
workspace で「クライアント WASM のイベント処理・DOM 更新」を実装しているのは
`fandhe-frontend-wasm-full`（`crates/wasm-full/`）である。以下のマッピングを本書で確定し、
TASK-1.3b（#92）の実装対象として固定する。

- **一次成果物（#92 実装対象）**: `crates/wasm-full/tests/xss_escape_wasm.rs`
- **根拠**:
  1. TASK-1.3 が検証対象とする「イベント処理・DOM 更新」の実装
     （`crates/wasm-full/src/events.rs`・`crates/wasm-full/src/dom.rs`・
     `crates/wasm-full/src/hydration.rs`）は現行 workspace では `wasm-full` にのみ
     存在する。
  2. `crates/wasm-client/`（`fandhe-frontend-wasm-client`）クレートの新設は TASK-6.2b（#48）の
     スコープである。`docs/guides/browser-testing.md` 第 2 節が同様の判断
     （`crates/wasm-client/` 未作成を前提にクレート新設を自スコープに含めない）を
     既に確立しており、本書はその判断を踏襲する。TASK-1.3a で `crates/wasm-client/`
     を重複新設すると #48 との責務混線・コンフリクトを招く。
  3. `crates/wasm-full/Cargo.toml` は `crate-type = ["cdylib", "rlib"]` のため、
     native（rlib、`cargo test`）と wasm32 実機（`wasm-pack test`）の両方の
     テストが同一クレートに同居できる。

`crates/wasm-client/` 新設（#48）後にハイドレーション経路の実ブラウザ検証
（TASK-6.3b・#66）と本書の設計内容がどう接続するかは第 9 節（スコープ外と
引き継ぎ）に記録する。

## 3. テスト観点表（第 1 層: native / rlib）

`cargo test --workspace` で常時実行する層。`Runtime::mount()`/`hydrate()`
（TASK-11.2d・#77）が未実装の現時点でも実装可能な観点のみで構成する。

| 観点 ID | 経路 | 検証内容 |
|---------|------|---------|
| N-1 | `fandhe_frontend_interactive::dispatch` → `wasm_full::render_component_html` | 全ペイロード（第 5 節）× テキストノード文脈: 生 `<script>`・`<img onerror>` 等の実タグ構文が出力に現れず、対応するエスケープ済み表現（`&lt;script&gt;` 等）が出力に現れる（両方向 assert。空出力による偽陰性を防ぐ） |
| N-2 | 同上 | 全ペイロード × 属性値文脈: `"` `'` による属性境界破壊（例: `onmouseover="..."` の生成、属性値からの breakout）が発生しない |
| N-3 | `events::action_from_click` / `events::action_from_input`（純粋関数） → `dispatch` → `render_component_html` | イベント処理を経由して state に取り込まれたペイロードにもエスケープが貫通する（TASK-1.3 の要件文言「イベント処理を経由した出力」に直接対応する観点） |
| N-4 | `hydration::restore_state` → 状態復元後の `view()` → `render` | `data-hydrate-*` 属性値として注入されたペイロードが、状態復元・再描画を経てもエスケープされたまま出力される（DOM 更新経由の観点） |
| N-5 | エンティティ偽装ペイロード（`ENTITY_SPOOF_TAG`・`ENTITY_SPOOF_AMP`）を N-1〜N-4 の経路へ通す | 二重エスケープ・エスケープ漏れの双方を検知する（`crates/core/tests/xss_escape.rs` と同一観点の WASM 経路版） |

- N-1・N-2 は `crates/wasm-full/tests/dom_update.rs`（TASK-11.2c・#76 成果物）に
  既に存在する 2 ケース（`render_component_html_escapes_xss_payload_in_list_items`・
  `render_component_html_escapes_xss_payload_in_attribute_value`）を土台に、
  第 5 節のペイロードカタログ全種へ拡張する形で実装する。既存ケースの
  削除・置換は行わず、`xss_escape_wasm.rs` 側で網羅拡張する。
- N-3 は `events.rs` の `action_from_click`/`action_from_input` が web-sys に
  依存しない純粋関数であることを利用し、native テストのまま「イベント処理を
  経由した」という要件文言を字義通り満たす。
- N-4 は `hydration.rs` の `restore_state`/`filter_hydration_attrs` が
  DOM・`web-sys` に依存しない純粋ロジック層であることを利用する。

## 4. テスト観点表（第 2 層: WASM 実機 / headless Chrome）

`wasm-bindgen-test` + `wasm_bindgen_test_configure!(run_in_browser)` による
実 DOM 検証層。文字列 grep ではなく実ブラウザの DOM パース結果を検証すること
で、文字列一致では検出できない「ブラウザがどう解釈するか」を保証する。

| 観点 ID | 検証内容 |
|---------|---------|
| W-1 | ペイロードを含む `render_component_html` 出力を `Element::set_inner_html` した後、`root.query_selector("script")` が `None` であること（実 DOM 上に `<script>` 要素が生成されない） |
| W-2 | ペイロードを含むテキストノードの `text_content()` が、注入前の生ペイロード文字列と一致して読み戻せること（エスケープ → ブラウザパース → 復元のラウンドトリップ健全性。エスケープが「情報を破壊」していないことの確認） |
| W-3 | `onerror` / `onmouseover` 等のイベントハンドラ属性を持つ要素が DOM 上に存在しないこと（`root.query_selector("[onerror],[onmouseover]")` が `None`） |
| W-4 | `data-hydrate-*` 属性にペイロードを仕込んだ実 DOM 要素から `read_hydration_attrs` → `restore_state` → 再描画した後も W-1〜W-3 が成立すること |
| W-5（拡張予約） | `Runtime::mount()`/`hydrate()`（TASK-11.2d・#77）経由のマウント、および実イベント発火（`dispatchEvent`）を伴う再描画でも W-1〜W-3 と同等の保証が成立すること |

- W-5 は `Runtime<C>` が未実装（#77 open）のため TASK-1.3b（#92）の実装
  対象外とし、#77 マージ後の拡張として第 9 節に引き継ぐ。#92 のブロッカーには
  しない。
- W-1〜W-4 は `crates/wasm-full/Cargo.toml` の `crate-type = ["cdylib", "rlib"]` を
  前提とする。`set_inner_html`/`query_selector`（`Element`）・属性走査
  （`NamedNodeMap`・`Attr`）は既存 `web-sys` features で充足するが、
  W-2 が使う `text_content()` は `web_sys::Node` のメソッドであり、現行
  `crates/wasm-full/Cargo.toml` の features 一覧（`Attr`・`Document`・`Element`・
  `Event`・`EventTarget`・`HtmlInputElement`・`NamedNodeMap`・`Window`・
  `console`）に `Node` は含まれていない。#92 実装時に `Node` feature の
  追加が必要になる見込みが高い。これは既存 `web-sys` 依存への **feature
  追加**であり新規クレート追加ではないため、REQ-3（依存グラフ上限）・
  事前承認の対象外（既存 `hydration.rs` 冒頭コメントが `NamedNodeMap`/`Attr`
  feature 追加時に採った扱いと同じ）だが、#92 実装時に `cargo metadata`
  で実際に必要な feature 一覧を確定し、Cargo.toml のコメントへ追記すること。

## 5. ペイロードカタログ

`crates/core/tests/xss_escape.rs` の `mod payloads`（OWASP XSS Prevention Cheat Sheet
Rule #1 準拠、CSR 経路テストが使用する共有集合）と観点を揃えて
`xss_escape_wasm.rs` 内に**再定義**する。`crates/interactive/tests/xss_escape.rs` が
既に確立している「テストコードはクレート境界をまたいで共有せず再定義する」
規約（クレート間のテストコード依存を作らない）に従う。

| `core::payloads` 定数 | 攻撃パターン | `xss_escape_wasm.rs` での用途 |
|------------------------|-------------|-------------------------------|
| `SCRIPT_TAG` | タグ注入 | N-1・N-3・N-4・W-1 |
| `IMG_ONERROR` | イベントハンドラ属性つきタグ注入 | N-1・W-1・W-3 |
| `DOUBLE_QUOTE_BREAKOUT` | 二重引用符属性値からの breakout | N-2・W-3 |
| `SINGLE_QUOTE_BREAKOUT` | 単一引用符属性値からの breakout（イベントハンドラ注入込み） | N-2・W-3 |
| `ENTITY_SPOOF_TAG` | エンティティ偽装（`&` 先頭処理） | N-5 |
| `ENTITY_SPOOF_AMP` | エンティティ偽装（`&amp;` 再エスケープ挙動） | N-5 |
| `CONTEXT_BREAKOUT` | コンテキスト脱出（閉じタグによる親コンテキスト離脱） | N-1・W-1 |
| `NON_ASCII_MIXED` | 非 ASCII 混在（マルチバイト透過確認） | W-2（ラウンドトリップの文字境界確認に最適） |

再定義規約:

- 定数名・値は `core::payloads` と同一の文字列を用いる（値の乖離を防ぐため、
  実装時に `crates/core/tests/xss_escape.rs` から値をコピーしたことをコメントで
  明記する）。
- `mod payloads`（またはトップレベル `const` 集合 + `all()` 関数）の形で
  `xss_escape_wasm.rs` 冒頭に定義し、全観点がこのカタログを共有する。

## 6. CI 統合方針（TASK-1.3b・#92 への指示）

### 6.1 `test` ジョブ（第 1 層・native）

- `cargo test --workspace --locked` に自動的に含まれる。
- 加えて、既存の「XSS regression tests (REQ-1)」ステップ
  （`.github/workflows/ci.yml` の `test` ジョブ、`cargo test -p fandhe-frontend-core --test
  xss_escape --locked`）に**独立したステップ**として
  `cargo test -p fandhe-frontend-wasm-full --test xss_escape_wasm --locked` を追加し、
  WASM 経路の回帰を CI 上で可視化する。

### 6.2 `browser-test` ジョブ（第 2 層・実機）

- 既存の `browser-test` ジョブ（`.github/workflows/ci.yml`）は
  `crates/wasm-client/Cargo.toml` の存在ガードで現在は空実行になっている
  （TASK-6.2b・#48 未マージのため）。本タスクの第 2 層はこのガードとは
  **独立**したステップとして追加する。`wasm-full` は既存クレートであり
  存在ガードは不要（fail-closed。ガード漏れによる誤スキップを避ける）。
- 追加ステップ: `CHROMEDRIVER="${CHROMEWEBDRIVER}/chromedriver" wasm-pack
  test --headless --chrome wasm-full`
- 既存のサプライチェーン対策（wasm-pack のバージョン固定 + SHA256
  チェックサム検証・chromedriver のランナー内蔵バイナリ明示指定によるダウンロード
  防止）をそのまま利用し、新規の第三者 action・自動ダウンロード経路を追加しない。

### 6.3 dev 依存の追加（`wasm-bindgen-test`）

- `crates/wasm-full/Cargo.toml` に `wasm-bindgen-test`（`[dev-dependencies]`）の追加が
  必要になる。
- **承認根拠**: PoC-5（`docs/spec/03-poc/`）が実機テストで実績のある構成であり、
  `docs/guides/browser-testing.md` 第 3 節が `wasm-pack test --headless --chrome`
  （`wasm-bindgen-test` ベース）を標準ランナーとして既に指名している。
  dev 依存であるため標準サーバー構成の依存グラフ（REQ-3: 60 件/深さ 6）には
  算入されない。
- #92 実装時に `cargo metadata` の実測結果（追加前後のパッケージ数・依存深さ）
  を PR 本文へ記録することを実装条件とする。

## 7. セキュリティ不変条件

- **両方向 assert の徹底**: 「エスケープ済み表現が出力に含まれる」ことと
  「対応する生の攻撃構文が出力に含まれない」ことの両方を assert する。
  一方向のみの assert は、出力が空文字列になる等の偽陰性を見逃す
  （`crates/core/tests/xss_escape.rs` 冒頭コメントと同一原則）。
- **DOM 構造検証による補完（第 2 層）**: 文字列 grep（`contains`/`!contains`）
  に加え、実 DOM パース後の `query_selector` によって「ブラウザが実際にどう
  解釈したか」を検証する。文字列一致だけでは検出できないブラウザ側の
  正規化・パース差異による回避を防ぐ。
- **既定エスケープの迂回禁止**: 本書が設計するテスト・実装対象コードのいずれも
  `raw_html()` の新規使用・HTML 文字列直接組み立て（`format!("<div>{}</div>",
  ...)` 相当）を行わない。テスト対象は既存の `render_component_html` /
  `dispatch` / `restore_state` の既定経路のみとする。
- **削除・弱体化の禁止**: `xss_escape_wasm.rs` は `.claude/rules/coding-rust.md`
  の規約により、以後の削除・弱体化・`#[ignore]` 化を禁止する。この規約を
  ファイル冒頭 `//!` に明記することを TASK-1.3b の実装条件とする
  （`crates/core/tests/xss_escape.rs` と同様の書式）。
- **秘密情報の混入防止**: ペイロードはすべて OWASP 公開カタログ由来の
  ダミー攻撃文字列であり、実クレデンシャル・実在の内部情報を含まない。

## 8. TASK-1.3 受け入れ基準との対応表

| TASK-1.3 受け入れ基準（要件文言） | 満たすテスト観点 | 対応節 |
|-----------------------------------|-------------------|--------|
| クライアント WASM のイベント処理を経由した出力に既定エスケープ保証が及ぶこと | N-3・W-1〜W-3 | 第 3・4 節 |
| クライアント WASM の DOM 更新を経由した出力に既定エスケープ保証が及ぶこと | N-1・N-2・N-4・W-1〜W-4 | 第 3・4 節 |
| SSR/SSG/CSR（TASK-1.2）と同一のエスケープ保証水準であること | 第 5 節のペイロードカタログを `crates/core/tests/xss_escape.rs` と揃えることで担保 | 第 5 節 |
| 進捗チェック | #90（親）・#91（本書・設計）・#92（実装・CI 統合） | — |

## 9. スコープ外と引き継ぎ

| 事項 | 状態・引き継ぎ先 |
|------|-------------------|
| `xss_escape_wasm.rs` の実装本体・CI ワークフロー変更・`wasm-bindgen-test` の実追加 | TASK-1.3b（#92）。本書は設計のみで実装しない |
| `Runtime::mount()`/`hydrate()` + 実イベント発火（`dispatchEvent`）経由の検証（W-5） | TASK-11.2d（#77）マージ後の拡張として追補する。#92 のブロッカーにしない |
| `crates/wasm-client/` クレート経由のハイドレーション実ブラウザ検証 | TASK-6.2b（#48）・TASK-6.3b（#66）。`docs/guides/browser-testing.md` 第 7 節の引き継ぎ表に「WASM 経路 XSS テストの本環境への統合」として既に記載済み |
| `wasm-thin`（オプトイン薄い JS グルー方式）経路の XSS 検証 | v1 スコープ外。`docs/design/opt-in-thin-js-glue.md` が既に制約を明記済み。本タスクでは新規 Issue 化を提案しない（既存文書での担保で十分と判断） |

本書が新たに Issue 化を要すると判断した事項は現時点でない
（いずれも既存タスク番号・既存文書への引き継ぎ）。

## 10. セキュリティ考慮事項（OWASP Top 10 観点）

- **A03 インジェクション / XSS（本タスクの主題）**: 第 7 節に不変条件として
  明記した通り、両方向 assert + DOM 構造検証（`query_selector`）により
  偽陰性を排除する設計とした。ペイロードカタログは OWASP XSS Prevention
  Cheat Sheet Rule #1 準拠のまま既存 `crates/core/tests/xss_escape.rs` と整合させた。
- **A04 安全でない設計**: 既定エスケープの迂回経路（`raw_html` の新規使用・
  HTML 文字列直接組み立て）をテスト対象コードへ導入しないことを第 7 節に
  明記。テスト自体の弱体化（`#[ignore]`・削除）禁止も規約として埋め込んだ。
- **A05 セキュリティ設定ミス**: CI 統合設計（第 6 節）は既存 `browser-test`
  ジョブ・`loc-check` 等と同じ fail-closed 方針（緩和用 input・
  `continue-on-error` を設けない）に揃えた。
- **A06/A08 脆弱な依存・サプライチェーン**: 第 2 層に必要な
  `wasm-bindgen-test` は dev 依存に限定し、承認根拠と `cargo metadata`
  実測記録の義務を第 6.3 節に明記した。wasm-pack のバージョン固定 +
  SHA256 検証・chromedriver 自動ダウンロード禁止という既存対策は変更しない
  （第 6.2 節）。
- **機微情報の露出**: 本書は docs-only 変更であり、掲載するペイロードは
  すべて公開済みのダミー攻撃文字列である。実クレデンシャル・実在の内部情報を
  含まない。
