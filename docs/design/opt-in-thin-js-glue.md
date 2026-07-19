# 薄い JS グルー方式（`rws-wasm-thin`）のオプトイン提供（TASK-11.3b）

## 1. 目的とトレーサビリティ

本ドキュメントは REQ-11（`docs/spec/04-requirements.md` REQ-11 節「WASM 完全方式に
よるクライアントインタラクション（既定）と薄い JS グルー（オプトイン）」）のうち、
「薄い JS グルー」方式（`rws-wasm-thin`）をオプトインとして選択した場合に生じる
制約事項（(c) XSS の保証一貫性・(d) AI 生成検証の到達範囲が Rust 側の設計に
収まらなくなる旨）を明記した警告ドキュメントです。

`docs/spec/05-tasks.md` の親タスク TASK-11.3（#78）は 2 つの成果物に分かれています。

- **TASK-11.3a（#79）**: `wasm-thin/src/lib.rs`（製品版クレート実装）
- **TASK-11.3b（本ドキュメント・#80）**: `docs/design/opt-in-thin-js-glue.md`（オプトイン
  提供ドキュメント・制約事項の明記）

**本文書のステータス**: TASK-11.3a（#79）はマージ済みです。本書は製品版
`wasm-thin/src/lib.rs`（`demo` モジュールの `#[wasm_bindgen]` エクスポート）を
正として記述します。本書は元々 PoC-5
（`docs/spec/03-poc/wasm-runtime-split/wasm-thin/src/lib.rs`）で実証済みの
公開 API を土台に執筆されましたが、#79 マージ後に実装との乖離が判明したため
（イシュー #397）、実装を正として本書を追随更新しました
（`docs/design/wasm-full-architecture.md` が TASK-11.1b 未マージ時に採った運用と同一
の方針）。今後もエクスポート関数のシグネチャ変更は実装 PR と同一 PR で本書
（特に第 4.2 節）を更新することとし、単独の乖離を発生させません（第 4.2 節参照）。

**本タスクのスコープ**: 本ドキュメントの作成のみ（docs-only 変更）。`wasm-thin/`
クレートの実装・`Cargo.toml`（workspace）・CI の変更はいずれも TASK-11.3a（#79）の
スコープであり、本書では行いません。`docs/spec/` はサブモジュールのため編集禁止
（変更が必要な場合は frontend-framework-spec リポジトリで行います）。

本書は `docs/design/wasm-full-architecture.md`（TASK-11.2a）・`docs/guides/embedding-guide.md`
（TASK-7.1b）・`docs/api/component-api.md`（TASK-5.1a）と同じ書式（目的とトレーサビ
リティ・設計/手順・セキュリティ不変条件・受け入れ基準対応表・スコープ外表）に
揃え、`docs/` 直下のフラット配置とします。

参照元:

- REQ-11（`docs/spec/04-requirements.md` REQ-11 節）
- TASK-11.3（`docs/spec/05-tasks.md:288-293`）
- PoC-2（`docs/spec/03-poc/security-threat-model/README.md`「薄い JS グルーを
  許した場合の境界」節）— (c)(d) の境界定義の一次情報源
- PoC-5（`docs/spec/03-poc/wasm-runtime-split/README.md`「実施内容 5・7」
  「発見事項 2・3」「要件への示唆」節）— 実測・トレードオフ・警告文の実証根拠

## 2. 位置づけ — 既定とオプトイン

フレームワークの既定のクライアントインタラクション方式は `rws-wasm-full`
（WASM 完全方式、`docs/design/wasm-full-architecture.md` 参照）です。`rws-wasm-thin`
（薄い JS グルー方式）は既定ではなく、**次のいずれかに該当する場合に限り**
選択するオプトイン方式として位置づけます。

- 既存の DOM ヘルパーライブラリ（jQuery 系・軽量 DOM 操作ライブラリ等）を
  段階的に併用する必要がある
- 既存の JS 資産（イベント配線コード等）からの移行期間中で、Rust 側への
  完全移行が即座には行えない
- バンドルサイズを極限まで切り詰めたい特殊事情があり、かつ第 3 節の
  制約事項を受け入れられる

**選定フローチャート**: 上記のいずれにも該当しない限り、既定の `rws-wasm-full`
を選択してください。`rws-wasm-thin` を選ぶ判断は、第 3 節の制約事項を読んだ
うえで、それでも上記の事情が既定方式の採用を上回ると判断した場合に限ります。

```
既定方式（rws-wasm-full）で要件を満たせるか？
  │
  ├─ Yes → rws-wasm-full を使う（推奨・既定経路）
  │
  └─ No（DOM ヘルパー併用／既存 JS 資産からの移行／バンドルサイズ極限化が必要）
        │
        └─ 第 3 節の制約事項（(c)(d)・LOC 分類・サプライチェーン誘因）を
           受け入れられるか？
             │
             ├─ Yes → rws-wasm-thin をオプトインで採用する（第 4〜5 節に従う）
             │
             └─ No  → rws-wasm-full に留まる、または要件自体を見直す
```

## 3. 制約事項・警告（オプトイン選択時に必ず理解すべき事項）

PoC-2 は「薄い JS グルーを実行時に持ち込んだ時点で、Rust 基盤が確保した相対的な
安全性の緩和効果はほぼ即座に減衰する」という境界を明文化しています。PoC-5 は
この境界を `rws-wasm-thin` の実装で実証しました。`rws-wasm-thin` を選択する場合、
以下の 4 点を必ず理解したうえで採用してください。

### 3.1 (c) XSS の保証一貫性の減衰

`rws-wasm-thin` の公開関数（`initial_html()` / `apply()`）は、いずれも内部で
`ThinRuntime::html()`（`rws_core::render()` の既定エスケープを経由）を
呼び出しており、**関数が返す文字列自体は既定エスケープ済み**です
（第 4.2 節・`demo_boundary_layer_smoke` テストの XSS 回帰検証で確認済み）。

しかし、この文字列を実際に DOM へ適用する処理（`root.innerHTML = apply(...)`）
は JS グルー側の 1 行に委ねられています。この境界より先は Rust 側の型チェック・
エスケープ機構が及びません。したがって:

- JS グルー側で `apply()` の戻り値と他の文字列を連結してから `innerHTML` に
  代入する、別の描画パス（`document.write()` 等）を追加する、といった変更を
  行った場合、**その追加分には Rust 側のエスケープ保証が及びません**。
- これは「`rws-wasm-thin` の実装に脆弱性がある」という意味ではなく、
  「JS グルーというレイヤーが Rust の保証範囲外にある」という構造的な
  リスクです（PoC-5 実施内容 7・PoC-2 との突き合わせで確認済み）。

`rws-wasm-full`（WASM 完全方式）では DOM 適用（`set_inner_html`）まで Rust 側が
行うため、この一貫性の減衰は発生しません。これが REQ-11 で `rws-wasm-full` を
既定とする根拠の一つです。

### 3.2 (d) AI 生成検証の到達範囲

JS グルー（`static/glue-thin.js` 相当）は `cargo check` / `cargo clippy` /
`cargo test` およびフレームワークの REQ-13 系 AI 自己保守検証ゲートの対象外です。
AI がこの JS グルーを変更・生成した場合、Rust 側の型チェック・lint・テストは
一切働きません。JS グルーの変更は人手または JS 向けの別の検証手段でレビューする
必要があります。

### 3.3 LOC ルーブリック上の分類 —「薄い」という呼称と実測のズレ

PoC-3 が定めた LOC ベースの操作的ルーブリック（0〜10 行 = 薄い、11〜40 行 = 中）
に照らすと、PoC-5 実測の `glue-thin.js` は **16 行**であり、「薄い」ではなく
**「中」に分類**されます（PoC-5 発見事項 2）。一方、`rws-wasm-full` 側の
`glue-full.js` は **3 行**で、「薄い」の中でも最小の実例です。

「薄い JS グルー」という設計思想上の呼称と、客観的な LOC 実測にはズレがあります。
`rws-wasm-thin` を採用しても、JS の記述量が `rws-wasm-full` より少なくなる
とは限らない点に注意してください（第 4.3 節「JS 実効行数の操作的定義」も参照）。

### 3.4 サプライチェーン誘因

`rws-wasm-thin` 自体は `web-sys` にすら依存せず、依存面では最小です。しかし
「薄い JS グルー」という設計方針そのものが、将来的に DOM ヘルパーライブラリ・
フォーマッタ等の npm パッケージを JS 側（実行時）へ持ち込む誘因になり得ます。
これは PoC-2 が懸念する「NPM パッケージの実行時混入」と同一線上のリスクです。

- クライアント実行時への NPM パッケージ・Node ランタイムの持ち込みは REQ-12 の
  スコープ外であり、**禁止**です。
- `rws-wasm-thin` の JS グルーはブラウザ標準 API（`addEventListener` /
  `innerHTML` 等）のみを使用し、npm 由来の実行時コードを一切含めないでください
  （第 5 節「JS グルーの規範」参照）。

## 4. オプトイン手順

### 4.1 ビルド

```bash
# wasm32 ターゲットへのビルド
cargo build -p rws-wasm-thin --target wasm32-unknown-unknown --release

# wasm-bindgen でブラウザ配布用の JS グルーを生成
wasm-bindgen --target web \
  --out-dir pkg/thin --out-name rws_wasm_thin \
  target/wasm32-unknown-unknown/release/rws_wasm_thin.wasm
```

### 4.2 公開 API 凍結表

**凍結の基準点**: 本表の正は製品版 `wasm-thin/src/lib.rs` の `demo` モジュール
（`#[wasm_bindgen]` エクスポート）であり、表はそのシグネチャを転記したもの
です（実装が正、文書が従。第 1 節参照）。**変更手続き**: エクスポート関数の
シグネチャ変更は破壊的変更として扱い、実装 PR と同一 PR で本表を更新してく
ださい。文書のみ・実装のみの単独変更で乖離させないでください。**検証手段**:
`wasm-thin/tests/thin_runtime.rs` の `demo_boundary_layer_smoke` が 3 関数の
呼び出し形（`demo::hydrate_from_attrs(vec![...], Vec::new()) -> bool` 等）を
コンパイル時に固定しており、本表の読者はこのテストで実シグネチャを裏取り
できます。

`rws-wasm-thin` は `web-sys` に一切依存しません。`initial_html` / `apply` は
「文字列 in・文字列 out」の純粋な状態計算を行い、`hydrate_from_attrs` は
「文字列配列 2 本 in・真偽値 out」の純粋な状態計算を行います（`wasm_bindgen`
がタプルの `Vec` を直接エクスポートできないため 2 配列表現になっています）。
いずれも DOM 操作・イベント配線は行いません。

| API | シグネチャ | 役割 |
|-----|-----------|------|
| `initial_html` | `pub fn initial_html() -> String` | 初期状態の HTML を返す。CSR モードで JS グルーがこれを `root.innerHTML` に設定する |
| `hydrate_from_attrs` | `pub fn hydrate_from_attrs(names: Vec<String>, values: Vec<String>) -> bool` | SSR が出力した `data-hydrate-*` 属性の「プレフィックス付き属性名」と値を、同一添字が対応する 2 本の配列（`names`/`values`）で渡し、WASM 内部状態を復元する。`names` の長さが `values` と一致しない場合、または復元に失敗した場合は状態を変更せず `false` を返す（初期状態のまま CSR を継続する安全側フォールバック）。JS 側は `root.innerHTML` を書き換えない（SSR 済み DOM をそのまま尊重する） |
| `apply` | `pub fn apply(action: &str, payload: &str) -> String` | アクションを適用し、更新後の HTML 全体（`#interactive-root` を含む rooted tree 全体・既定エスケープ済み）を返す。JS グルーはイベントから `action`/`payload` を読み取ってこの関数を呼び、戻り値のみを、`#interactive-root` 自身ではなくその親要素（mount）の `innerHTML` に設定する（`#interactive-root` 自身に設定すると戻り値に含まれる同名要素がネストし id が重複するため。DOM 差分計算は行わない最小実装） |

3 関数はいずれも境界層（`wasm-thin/src/lib.rs` の `demo` モジュール、
`#[wasm_bindgen]` エクスポート）が汎用層 [`ThinRuntime<C>`]（`wasm-bindgen`
非依存の純粋 Rust）を `rws_interactive::AppState` に束縛して呼び出す薄い
ラッパーです。`ThinRuntime<C>` は内部で `rws_core::render()`・
`rws_interactive::dispatch`・`rws_interactive::Hydrate::from_hydration_attrs`
を呼び出します。状態機械そのものは `rws-wasm-full` と共通の
`rws-interactive` を使用します。

**補足（イシュー #376）**: `AppState::view()`（`render_with_root_attrs` 経由で
束縛点マーカーを付与する）が `rws-wasm-full` と共通のコードであるため、
`initial_html()` / `apply()` の出力 HTML には `rws-wasm-full`
と同様に `data-bind-*` / `data-key` / `data-hydrate-item-ids` マーカーが
含まれます。ただし `rws-wasm-thin` の更新経路は本節の `apply` の説明の
とおり戻り値の全置換 `innerHTML` 代入のみであり、これらのマーカーは
`wasm-thin` 経路では**不活性（inert）**です（束縛点更新・keyed list の
差分適用ロジック自体を `wasm-thin` / JS グルー側は持たないため）。属性値は
既定エスケープ済みであり、不活性であっても無害です。この方針を束縛点更新
一般化の対象に含めるかどうかの検討・結論は
`docs/policy/intentional-non-adoption.md` §3.10 に記録しています
（非採用確定）。

### 4.3 JS 実効行数の操作的定義

PoC-3 のルーブリック（0〜10 行 = 薄い、11〜40 行 = 中）を JS グルーの LOC 判定に
用います。カウント対象は `import` / 空行 / コメント行を除いた実効コード行です。
PoC-5 実測の `glue-thin.js`（CSR モード、下記 4.4 節例と同一構成）は 16 行で
「中」に分類されます（第 3.3 節）。

### 4.4 JS グルー最小例

#### CSR モード（PoC-5 `static/glue-thin.js` 準拠）

```javascript
/**
 * WASM＋薄い JS グルー方式・CSR モード。
 * イベント配線（addEventListener）・DOM 更新（innerHTML の書き換え）を
 * この JS ファイル側で行う。WASM 側（rws-wasm-thin）は
 * 「文字列 in・文字列 out」の純粋な状態計算のみを提供する。
 */
import init, { initial_html, apply } from "./pkg/thin/rws_wasm_thin.js";

await init();

const root = document.getElementById("interactive-root-mount");
root.innerHTML = initial_html();

root.addEventListener("click", (ev) => {
  const target = ev.target.closest("[data-action]");
  if (!target) return;
  const action = target.getAttribute("data-action");
  const idx = target.getAttribute("data-idx") || "";
  root.innerHTML = apply(action, idx);
});

root.addEventListener("input", (ev) => {
  const target = ev.target;
  if (target.id !== "draft-input") return;
  // 入力中は再描画しない（innerHTML の書き換えはフォーカス・キャレット
  // 位置を破棄するため）。状態のみ更新し、次の click アクション時に
  // まとめて反映する。
  apply("set_draft", target.value);
});
```

#### ハイドレーションモード（PoC-5 `static/glue-thin-hydrate.js` 準拠）

```javascript
/**
 * WASM＋薄い JS グルー方式・ハイドレーションモード。
 * サーバー Rust（SSR）が出力した data-hydrate-* 属性を読み取り、
 * WASM 内部状態を復元する。SSR 済みの DOM は作り直さず、
 * イベント配線のみ行う。
 *
 * hydrate_from_attrs は names/values の 2 配列 in・真偽値 out。
 * data-hydrate- プレフィックス付きの属性名をそのまま names に渡す
 * （HTMLElement.dataset は camelCase 化してプレフィックスを失うため
 * 使用しない）。復元失敗時は false が返り、状態は初期状態のまま
 * （CSR フォールバック）。
 *
 * DOM 更新（innerHTML の書き換え）の対象は #interactive-root の
 * 親要素（mount）とし、#interactive-root 自身には設定しない。
 * apply() の戻り値は #interactive-root を含む rooted tree 全体
 * （initial_html() と同じ形）であるため、#interactive-root 自身へ
 * 代入すると、戻り値に含まれる #interactive-root がその内部に
 * ネストされて id が重複し、次回以降のクリックで DOM が壊れる
 * （CSR モードと同じく mount と rooted tree の id を分離する）。
 */
import init, { hydrate_from_attrs, apply } from "./pkg/thin/rws_wasm_thin.js";

await init();

const mount = document.getElementById("interactive-root-mount");
const root = mount.querySelector("#interactive-root");

const names = [];
const values = [];
for (const attr of root.attributes) {
  if (attr.name.startsWith("data-hydrate-")) {
    names.push(attr.name);
    values.push(attr.value);
  }
}
// 復元失敗時は false（状態は初期のまま = CSR フォールバック）。
hydrate_from_attrs(names, values);

mount.addEventListener("click", (ev) => {
  const target = ev.target.closest("[data-action]");
  if (!target) return;
  const action = target.getAttribute("data-action");
  const idx = target.getAttribute("data-idx") || "";
  mount.innerHTML = apply(action, idx);
});

mount.addEventListener("input", (ev) => {
  const target = ev.target;
  if (target.id !== "draft-input") return;
  apply("set_draft", target.value);
});
```

## 5. JS グルーの規範（セキュリティ不変条件）

`rws-wasm-thin` をオプトインで採用する場合、JS グルーは以下の不変条件を
**必ず**守ってください。違反すると第 3.1 節の (c) XSS 保証一貫性の減衰が
実際に脆弱性として顕在化します。

1. **`innerHTML` へ代入してよいのは `initial_html()` / `apply()` の戻り値
   のみ**とする。他の文字列との連結・テンプレートリテラルへのユーザー入力の
   直接埋め込みを行わない。
2. **別の描画パスを追加しない**。`document.write()` や `insertAdjacentHTML()`
   等、`apply()` の戻り値以外を経由する DOM 書き換え経路を新設しない。
3. **イベント委譲は `data-action` 属性ベース**とする（第 4.4 節の例に従う）。
   `onclick` 属性文字列の動的生成など、追加のインジェクション面を作らない。
4. **ブラウザ標準 API のみを使用する**。`addEventListener` / `innerHTML` /
   `getAttribute` 等の標準 DOM API に限定し、npm 依存（DOM ヘルパーライブラリ
   等）を実行時コードとして持ち込まない（第 3.4 節・REQ-12）。

### 禁止例（**このパターンは採用しないこと**）

```javascript
// 禁止例: apply() の戻り値と別の文字列を連結して innerHTML に代入している。
// 連結した部分（ここでは badge 変数）には Rust 側のエスケープ保証が及ばない。
const badge = `<span>${userSuppliedLabel}</span>`; // ユーザー入力を未エスケープで埋め込み
root.innerHTML = apply(action, idx) + badge; // 禁止: apply() の戻り値のみを代入すべき
```

```javascript
// 禁止例: onclick 属性を動的に生成している。
// data-action ベースのイベント委譲（第 4.4 節）を使うべきで、
// 属性値としての HTML 文字列組み立ては新たなインジェクション面になる。
el.setAttribute("onclick", `handleClick('${payload}')`); // 禁止
```

## 6. 受け入れ基準対応表

| REQ-11 / TASK-11.3 の要求 | 本書での対応箇所 |
|---------------------------|-----------------|
| 「薄い JS グルー」をオプトイン方式として整理する（TASK-11.3） | 第 2 節「位置づけ — 既定とオプトイン」 |
| 選択した場合に (c) XSS の保証一貫性が Rust 側の設計に収まらなくなる旨を警告する（TASK-11.3） | 第 3.1 節 |
| 選択した場合に (d) AI 生成検証の到達範囲が Rust 側の設計に収まらなくなる旨を警告する（TASK-11.3） | 第 3.2 節 |
| WASM 完全方式のアプリ側 JS グルーの実効行数が 10 行以内（「薄い」）であること。対比として薄い JS グルー方式の LOC 実態を明示する（REQ-11 受け入れ基準） | 第 3.3 節・第 4.3 節 |
| バンドルサイズ（gzip 後）が 200KB 以内であること（REQ-11 受け入れ基準） | 第 7 節「性能実測の参照」に PoC-5 実測値（約 20.1KB）を記載 |
| NPM 互換は実行時スコープ外（REQ-12） | 第 3.4 節・第 5 節・不変条件 4 |

## 7. 性能実測の参照

`rws-wasm-thin` の性能特性は PoC-5（`docs/spec/03-poc/wasm-runtime-split/README.md`
「実施内容 2〜4」）で以下のとおり実測済みです（実ブラウザでの正式実証は
TASK-11.5【Conditional Go 条件 1】の宿題であり、本書の実測値は Node.js 近似・
WASM 関数呼び出しスループット計測による代替値である点に注意してください）。

- バンドルサイズ（gzip 合計）: 約 20.1KB（目標 200KB 以内に対し約 10 倍の余裕）
- 初期ロード（Node.js 近似）: avg 0.237ms（目標 300ms 以内に対し極めて大きな余裕、
  ただしブラウザでの HTTP フェッチ・HTML パース・実ペイントのコストは含まない）
- DOM 操作性能（WASM 関数呼び出しスループット近似）: カウンター更新 4.08μs/回・
  リスト追加削除 31.72μs/回（目標 16ms/フレームに対し 500 倍以上の余裕、ただし
  実 DOM 差分適用・レイアウト・ペイントのコストは含まない）

## 8. スコープ外

| 項目 | 引き継ぎ先 |
|------|-----------|
| `wasm-thin/` クレート本体の実装・`Cargo.toml`（workspace）追加 | TASK-11.3a（#79） |
| `.github/workflows/ci.yml` への `wasm-thin` 存在ガードジョブ追加 | TASK-11.3a（#79）以降（`rws-wasm-full` の運用に倣う） |
| 複雑な状態（ネストしたオブジェクト等）のハイドレーション一般化 | PoC-5「要件への示唆」節に記載のとおり Phase 4 以降の設計課題 |
| 実ブラウザでの初期ロード・DOM 操作性能の正式計測 | TASK-11.5【Conditional Go 条件 1】（#85〜） |
| バンドルサイズ検証の自動化 | TASK-11.6（#85〜#89） |
| `docs/policy/unsafe-boundary.md` の `wasm-thin` 行の更新（`rws-wasm-thin` は `web-sys` 非依存のため `unsafe` を使用しない見込みだが、確定は実装時に行う） | TASK-11.3a（#79） |
| 束縛点更新・keyed list（イシュー #345 の一般化方針）を `wasm-thin` の JS グルー側更新経路にも適用するか | `docs/policy/intentional-non-adoption.md` §3.10（イシュー #376）で非採用確定 |
| 仕様（`docs/spec/`）自体の変更が必要な事項が生じた場合 | frontend-framework-spec リポジトリの Issue として起票を提案する（本書の対象外） |
