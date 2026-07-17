# rws-wasm-client hydrate() API 設計確定（TASK-6.2a）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-6（`docs/spec/04-requirements.md` REQ-6 節）の受け入れ基準
「ハイドレーション（`hydrate()`）が、サーバー出力済み DOM を再構築せず、既存 DOM へ
イベントリスナーを後付けする方式で成立すること」を満たすため、WASM/CSR クレート
`rws-wasm-client` の公開 API 表面・クレート構成・セキュリティ不変条件・CI 統合方針を
**設計として確定**するための成果物です。PoC-3
（`docs/spec/03-poc/rendering-web-standards/wasm-client/src/lib.rs`）・PoC-5
（`docs/spec/03-poc/wasm-runtime-split/wasm-full/src/lib.rs`）で実証済みの
「既存 DOM を再構築せずイベントリスナーを後付けする」最小ハイドレーションを
標準アーキテクチャとして採用します。

`docs/spec/05-tasks.md` の親タスク TASK-6.2（#46）は 4h 粒度で a〜c に分割されています。

- **TASK-6.2a（本ドキュメント・#47）**: `hydrate()` API の**設計確定**
- **TASK-6.2b（#48）**: 本書に従った `rws-wasm-client` クレートの実装
- **TASK-6.2c（#49）**: 関連テスト（ネイティブ単体テスト・wasm ビルド確認）の整備

**本文書のステータス**: TASK-6.2a 確定版。TASK-6.2b/c は本書の設計に従って実装し、
実装と本書の記述に乖離が生じた場合は本書を正として PR レビューで指摘する。

本書は `docs/app-api.md`（TASK-6.1a）・`docs/component-api.md`（TASK-5.1a）と同じ
書式（ステータス・トレーサビリティ・凍結表・設計判断表・スコープ外表・
セキュリティ不変条件・受け入れ基準対応表）に揃え、`docs/` 直下のフラット配置とする。

**本タスクのスコープ**: 設計確定書の作成のみ（docs-only 変更）。`wasm-client/` クレート
新設・依存クレート（`wasm-bindgen` / `web-sys`）の実追加・`.github/workflows/ci.yml` の
変更はいずれも TASK-6.2b（#48）のスコープであり、本タスクでは行わない。テスト実装は
TASK-6.2c（#49）のスコープ。`docs/spec/` はサブモジュールのため編集禁止（変更が必要な
場合は frontend-framework-spec リポジトリで行う）。

**先行依存関係**: `hydrate()` の設計は `rws-core`（マージ済み）・`rws-app`（TASK-6.1b
#43 でマージ済み）の公開 API のみに依存する。TASK-6.1c（#44 rws-server）・TASK-6.1d
（#45）は本書執筆時点で並行して進行中のため未完了だが、本書は `rws-app` API の
凍結表（`docs/app-api.md` 第 3 節）のみを前提とし、`server/` 側の未マージ実装詳細には
依存しない。万一 #44 側で `rws-app` の公開シグネチャに変更が入った場合は、本書の
凍結表を正として TASK-6.2b 実装時に調整する（`docs/app-api.md` の運用に倣う）。

## 2. クレート構成の確定

- **パッケージ名**: `rws-wasm-client`
- **配置**: `wasm-client/`
- **edition**: 2021
- **`crate-type`**: `["cdylib", "rlib"]`（`cdylib` は `wasm32-unknown-unknown` ターゲットの
  成果物として必須。`rlib` はネイティブ単体テスト — TASK-6.2c の `find_attr_values` 等の
  テスト — から通常の Rust クレートとして参照するために付与する）
- **依存**: `rws-core`・`rws-app`（いずれも path 依存）＋ `wasm-bindgen` / `web-sys`。
  `web-sys` の feature は必要最小限に列挙する（`Document`・`Element`・`Window`・
  `Event`・`EventTarget`・`DomTokenList`・`console` 等。実装時に実際に使用する API
  から逆算し、未使用 feature を含めない）。
  **依存の実追加は TASK-6.2b（#48）で行い、追加時に `cargo metadata` 実測値
  （パッケージ数・依存グラフ深さ）を本書へ追記すること**。REQ-3 の上限
  （60 件以内・深さ 6 以内、`docs/dependency-graph-policy.md`）は「標準サーバー構成」
  （SSR/SSG エントリを含む構成）を対象としており、`wasm-client` はブラウザ側に
  配布される別系統のビルド成果物であるため、同一の上限を機械的に適用するのではなく、
  `xtask` の deps-check 計測対象に `wasm-client` 系統を独立枠として含めるか否かを
  TASK-6.2b で判断し、判断結果を本書に追記する。
- **lint 属性**: `#![forbid(unsafe_code)]` は `#[wasm_bindgen]` マクロが展開する
  グルーコードが内部で `unsafe` を含むため、`wasm-client` クレートには適用できない。
  代わりに **`#![deny(unsafe_code)]` を採用**し、「自作コードで `unsafe` を新規に書かない・
  `unsafe` は `wasm-bindgen` の生成コードに限定する」という運用ポリシーで担保する。
  `docs/unsafe-boundary.md` 第 2 節の `wasm-client` / `wasm-full` 行（現在「未作成」表記）
  は、TASK-6.2b でクレートを作成した時点で本書の `deny(unsafe_code)` 方針に沿って
  実態を更新する。

**モジュール構成案**:

| モジュール | 内容 |
|-----------|------|
| `lib.rs` | `#[wasm_bindgen]` 公開関数（`hydrate` / `mount_csr`）のエントリポイント |
| `registry`（内部） | クロージャハンドルの `thread_local!` レジストリ（第 4.2 節参照） |

`rws-core` 側には新たに `find_attr_values` / `find_nav_targets`（第 3 節）を追加する。
これらは `rws_core::Node`（`render()` が受け取るノード木自身の型。実 DOM 型
（`web-sys::Node` 等）ではない）を引数に取る**DOM 非依存の純粋関数**であり、
PoC-3（`docs/spec/03-poc/rendering-web-standards/core/src/lib.rs:131,152`）の
実装をそのまま踏襲する。実 DOM を経由しないため `core` の外部依存ゼロ契約
（`.claude/rules/coding-rust.md`）を侵さず、`wasm32` ターゲットを介さないネイティブ
環境でもテスト可能である（第 4.1 節・判断 3 参照）。

## 3. 公開 API 凍結表

| API | 配置 | シグネチャ | 役割 |
|-----|------|-----------|------|
| `hydrate` | `wasm-client` | `#[wasm_bindgen] pub fn hydrate(root_id: &str) -> Result<(), JsValue>` | 指定 ID のルート要素配下の既存 DOM を**再構築せず**（`set_inner_html` を呼ばず）、イベントリスナーを後付けする |
| `mount_csr` | `wasm-client` | `#[wasm_bindgen] pub fn mount_csr(root_id: &str) -> Result<(), JsValue>` | `rws_app` のページ関数 → `rws_core::render()` の既定エスケープ済み出力を、指定 ID の要素へ `set_inner_html` で反映する（CSR 経路） |
| `find_attr_values` | `rws-core` | `pub fn find_attr_values(node: &Node, attr_name: &str) -> Vec<String>`（`Node` は `rws_core::Node`） | 指定属性を持つ子孫要素の属性値を列挙する DOM 非依存の純粋関数。ハイドレーション対象特定に用いる（ネイティブテスト可能） |
| `find_nav_targets` | `rws-core` | `pub fn find_nav_targets(node: &Node) -> Vec<String>`（`Node` は `rws_core::Node`） | `data-nav` 属性専用のショートカット |

### 3.1 設計方針の要点

- **root 指定型を採用**: PoC-3 の `hydrate()`（引数なし・単一グローバルルート前提）
  ではなく、**PoC-5 の `hydrate(root_id: &str)`（root 指定型）を標準として採用**する。
  複数マウント・部分埋め込み構成（REQ-7）と整合させるため、単一ページ内に複数の
  ハイドレーション対象領域を持てる設計とする。
- **`mount_csr` の位置付け**: REQ-6 受け入れ基準「CSR が SSR/SSG と同一関数を呼び
  `innerHTML` へ反映すること」に対応する。`rws_app::list_page` / `detail_page` /
  `page_shell` 等、`docs/app-api.md` 第 3 節で凍結済みの関数を**分岐なく**呼び出し、
  その戻り値（常に `rws_core::render()` 経由でエスケープ済み）のみを DOM へ渡す。
- **ハイドレーション対象のマーキング規約**: v1 最小スコープでは `rws_app::LIKE_BUTTON_ID`
  定数契約（`docs/app-api.md` 第 3 節）による固定 ID 指定を標準機構とし、加えて
  `data-*` 属性ベースの汎用列挙（`find_attr_values`）を標準機構として定義する。
  将来的な対象追加は `data-hydrate` 属性の値による分岐を想定するが、実装は
  TASK-6.2b のスコープとする。

## 4. 設計判断と根拠

| # | 判断 | 根拠 |
|---|------|------|
| 1 | `hydrate()` は `root_id: &str` を受け取る root 指定型とする（PoC-3 の引数なし版は不採用） | 複数マウント・部分埋め込み構成（REQ-7）と整合させるため。PoC-5 で実証済みの方式を標準とする |
| 2 | `hydrate()` は対象 DOM に対し `set_inner_html` 等の再構築系 API を一切呼ばない | REQ-6 受け入れ基準「サーバー出力済み DOM を再構築しない」を字義通り満たすための不変条件。イベントリスナーの後付け（`add_event_listener_with_callback` 等）のみを行う |
| 3 | `find_attr_values` / `find_nav_targets` は `rws-core` に配置し、`rws_core::Node`（core 自身のノード木型。実 DOM 型ではない）を引数に取る純粋関数として実装する | PoC-3（`docs/spec/03-poc/rendering-web-standards/core/src/lib.rs:127-154`）実績をそのまま踏襲。`rws_core::Node` を辿るだけの DOM 非依存ロジックのため、`cargo test -p rws-core` によるネイティブテストが可能で、wasm ビルドを介さない高速な回帰検証ができる。`core` は実 DOM 型（`web-sys::Node` 等）を一切参照しないため、外部依存ゼロ契約（`.claude/rules/coding-rust.md`）は当然に維持される。`wasm-client` 側は、サーバー/CSR 双方で描画に用いた `rws_core::Node` 値（またはその同値な再構築）に対してこれらの関数を呼び、結果として得た ID/属性値をもとに実 DOM 要素を `web-sys` 経由で取得してイベントリスナーを結びつける |
| 4 | クロージャの寿命管理は `closure.forget()`（意図的リーク）ではなく、`thread_local!` レジストリでハンドルを保持する方式を第一候補とする | PoC の `forget()` は `hydrate()` を複数回呼び出すシナリオ（SPA 内遷移・再ハイドレーション）でリークが蓄積する。`thread_local!` にクロージャの `Closure<dyn FnMut(..)>` を保持することで、将来的な明示的破棄（`dehydrate` 相当）への拡張余地を残す。代替案（`forget()` を v1 に限り容認）は「単一ページロード・再ハイドレーションなし」という制約下でのみ許容しうるが、REQ-7（部分埋め込み構成）が複数回の `hydrate()` 呼び出しを要求しうるため不採用とする |
| 5 | エラーは `Result<(), JsValue>` で返し、`unwrap()` / `panic!` を用いない。`JsValue` エラー文字列は英語・内部パス等を含まない | `.claude/rules/coding-rust.md` のエラーハンドリング規約・`.claude/rules/security.md` の機微情報露出防止（A09）を wasm 境界にも適用する |
| 6 | `#![forbid(unsafe_code)]` ではなく `#![deny(unsafe_code)]` を採用する | `#[wasm_bindgen]` 展開コードが内部で `unsafe` を含むため `forbid` はコンパイルを妨げる。`deny` により自作コードでの新規 `unsafe` 追加は防止しつつ、マクロ展開コードとは両立させる。`docs/unsafe-boundary.md` 第 2 節のポリシー行は TASK-6.2b でこの実態に合わせて更新する |

## 5. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| `wasm-client/` クレート新設・`Cargo.toml` 依存追加（`wasm-bindgen` / `web-sys`） | TASK-6.2b（#48） |
| `.github/workflows/ci.yml` の `forbid-unsafe` ジョブ対象を safe 域クレートへ `-p` 絞り込みする変更 | TASK-6.2b（#48） |
| `core/tests/unsafe_boundary.rs` の `UNSAFE_ALLOWED_MEMBERS` への `rws-wasm-client` 追加 | TASK-6.2b（#48） |
| ネイティブ単体テスト（`find_attr_values` 等）・`cargo build --target wasm32-unknown-unknown -p rws-wasm-client` のコンパイル確認 | TASK-6.2c（#49） |
| 実ブラウザでの実証（Playwright 等による E2E 検証） | TASK-6.3 系（#64〜、Conditional Go 条件 1） |
| 状態注入・`rws-interactive` との統合 | TASK-11.4（#81） |
| `hydrate()` の明示的破棄（`dehydrate` 相当）API | 本書では第 4 節・判断 4 で拡張余地として記録するのみ。API 設計は将来タスクで再検討 |

## 6. セキュリティ不変条件（REQ-1 の引き継ぎ）

`core/src/lib.rs` 冒頭の不変条件（REQ-1・REQ-2）を、`rws-wasm-client` への制約として
そのまま再掲・固定する。加えて `wasm-client` 固有の不変条件を以下に定義する。

1. DOM への HTML 挿入は `rws_core::render()` の出力（既定エスケープ済み）**のみ**を
   経由する。`format!` による HTML 断片の組み立てや、ユーザー入力を直接
   `set_inner_html` に渡すコードを書かない（`.claude/rules/coding-rust.md` の
   「HTML 文字列の直接組み立て禁止」を wasm 境界でも維持する）。
2. `hydrate()` が後付けするイベントハンドラ内で行う DOM 更新は、`set_text_content` /
   `class_list`（`DomTokenList`）等のテキスト・属性 API に限定する。新たな
   エスケープ迂回経路（`unsafe` な innerHTML 直書き等）を作らない。
3. `rws_core::raw_html()` を `wasm-client` から呼ばない。
4. `#![deny(unsafe_code)]` により自作コードでの `unsafe` 新規追加をビルド時に
   検出する。`unsafe` は `wasm-bindgen` 生成コードに限定され、自作コードには
   一切書かない（第 4 節・判断 6）。
5. エラー・ログ（`JsValue` / `web_sys::console`）に内部パス・状態値等の機微情報を
   含めない（第 4 節・判断 5）。
6. 依存クレート（`wasm-bindgen` / `web-sys`）の追加は事前に `cargo metadata` で
   影響を確認し、ユーザー承認を得る（`.claude/rules/coding-rust.md`・
   `.claude/rules/security.md`）。

これらは「設計制約」であり、TASK-6.2b の実装レビューではこの一覧との整合を確認する。

## 7. CI・テスト統合方針（TASK-6.2b・TASK-6.2c への指示）

- `.github/workflows/ci.yml` の `RUSTFLAGS='-F unsafe_code' cargo check --workspace` は
  `wasm-client` 追加により workspace 全体には適用できなくなるため、safe 域クレート
  （`core` / `interactive` / `app` / `server` / `xtask`）への `-p` 絞り込みに変更する
  （TASK-6.2b）。
- `core/tests/unsafe_boundary.rs` の `UNSAFE_ALLOWED_MEMBERS` に `rws-wasm-client` を
  追加し、CI 上の機械確認（`docs/unsafe-boundary.md` 第 3 節）と実態を同期させる
  （TASK-6.2b）。
- テスト階層は以下の 3 段階とする。
  1. `find_attr_values` / `find_nav_targets` のネイティブ単体テスト（`cargo test -p
     rws-core`）— TASK-6.2c
  2. `cargo build --target wasm32-unknown-unknown -p rws-wasm-client` によるコンパイル
     成立確認 — TASK-6.2c
  3. 実ブラウザでの動作検証（Playwright 等） — TASK-6.3 系（Conditional Go 条件 1）
     のスコープであり、TASK-6.2c では扱わない

## 8. REQ-6 受け入れ基準との対応表

| REQ-6 受け入れ基準 | 満たす API・設計要素 | 担当タスク |
|--------------------|----------------------|-----------|
| ハイドレーションが既存 DOM を再構築せず、イベントリスナーを後付けする方式で成立すること | `hydrate(root_id: &str)`（第 3 節）。第 4 節・判断 2 で「`set_inner_html` を呼ばない」ことを不変条件として固定 | TASK-6.2b（#48）・TASK-6.2c（#49） |
| CSR が SSR/SSG と同一関数を呼び `innerHTML` へ反映すること | `mount_csr(root_id: &str)` が `rws_app` のページ関数・`rws_core::render()` を分岐なく呼び出す（第 3.1 節） | TASK-6.2b（#48） |
| 実ブラウザでの実証 | 第 7 節でテスト階層 3 段階目として明記し、TASK-6.3 系へ引き継ぐ | TASK-6.3 系（#64〜） |

## 9. 関連文書との整合確認

- 第 3 節の `mount_csr` / `hydrate` は `docs/app-api.md` 第 3 節の凍結表
  （`list_page` / `detail_page` / `layout` / `page_shell` / `LIKE_BUTTON_ID`）と
  シグネチャ・契約点で矛盾しない。`docs/app-api.md` 第 5 節「ハイドレーション支援
  API は TASK-6.2 系へ引き継ぎ」は本書第 2〜3 節で確定した内容により解消される。
- 第 4 節・判断 6（`deny(unsafe_code)` 採用）は `docs/unsafe-boundary.md` 第 2 節の
  `wasm-client` / `wasm-full` 行（現状「未作成」）と矛盾せず、TASK-6.2b はこの表を
  本書の方針に沿って更新する。
- `docs/component-api.md` 第 2 節の「コンポーネントは通常の Rust 関数」規約は
  `rws_app` 側の契約であり、本書の `wasm-client` API 設計とは直接の依存関係を
  持たない。
