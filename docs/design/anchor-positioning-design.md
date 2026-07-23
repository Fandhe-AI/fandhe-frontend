# anchor positioning の設計確定: 外部依存ゼロでの位置計算方針（イシュー #589）

## 1. 目的とトレーサビリティ

- トラッキング: #520（headless-ui / pre-styled-ui 全体トラッキング）
- 親: #579 → #588「位置決めロジック（anchor positioning）を共通化する」
- 本イシュー #589 は #588 の先頭タスクであり、後続実装タスク #590 の**正の規範文書**を作る
  docs-only タスクである。

| 後続タスク | 内容 |
|-----------|------|
| #590 | 位置計算モジュール（純粋関数）の実装・CSS 変数出力・4 コンポーネントへの接続 |

**本文書のステータス**: イシュー #589 の設計確定書。`crates/headless-ui/src/positioning.rs`
（モジュール名は #590 で確定）・`crates/headless-ui/src/popover.rs`・`tooltip.rs`・
`menu.rs`・`select.rs` の実装が本書の記述と乖離した場合は本書を正とし、#590 の PR
レビューで指摘する。

本書は `docs/design/loader-trait-design.md`・`docs/design/hydration-nested-state.md` と
同じ書式（目的とトレーサビリティ / スコープ / 制約の確認 / 設計判断と根拠 / 後続タスクへの
引き継ぎ / スコープ外の明記 / セキュリティ不変条件 / 受け入れ基準対応表 / 関連文書との
整合確認）に揃える。

### 1.1 現状（解決する課題）

Popover（#532）・Tooltip（#533）・Menu（#566）・Select（#568）はいずれも
`positioner`（コンテナ）・`arrow`/`arrow_tip`（矢印、Select は不要のため未実装）
パーツを「CSS フック（`data-*` セレクタ）のみ」で実装しており、Floating UI 相当の
placement 計算（実際の座標決定）はスコープ外として繰り延べられている。

| コンポーネント | ファイル | rustdoc の該当箇所（スコープ外記載） |
|---------------|---------|--------------------------------------|
| Popover | `crates/headless-ui/src/popover.rs:36`〜`:41`（モジュール doc §スコープ外）、`:107`〜`:109`（`positioner` 関数 doc） | 「位置決めロジック（Floating UI 相当の placement / `sameWidth` / CSS 変数出力）: `positioner`/`arrow`/`arrow_tip` は CSS フック（data-* セレクタ）のみを提供する」「Tooltip とも共通するため、overlays 親（#530）配下での共通化検討をユーザー承認のうえ別イシューへ切り出す想定」 |
| Tooltip | `crates/headless-ui/src/tooltip.rs:28`〜`:34`（モジュール doc §スコープ外）、`:103`〜`:104`（`positioner` 関数 doc） | 「フローティング位置計算はスコープ外」 |
| Menu | `crates/headless-ui/src/menu.rs:40`（モジュール doc §スコープ外）、`:119`〜`:121`（`positioner` 関数 doc） | 「`data-*` へ反映するのみで、Floating UI 相当の placement 計算はスコープ外」 |
| Select | `crates/headless-ui/src/select.rs:47`〜`:59`（モジュール doc §out-of-scope）、`:189`〜`:190`（`positioner` 関数 doc） | 「位置決めロジック（Floating UI 相当）: `positioner` は CSS フックのみ」 |

本書は、この共通化の**正の規範文書**として、外部依存ゼロ制約下での placement 計算方式・
対応する placement 範囲（flip / shift / sameWidth の採否）・CSS Anchor Positioning
（Web 標準）採用可否の評価を確定する。

**headless-ui の spec 未反映について**: `docs/spec/`（fandhe-frontend-spec サブモジュール）
には headless-ui / pre-styled-ui 層がまだ反映されていない（fandhe-frontend-spec 側
イシュー #20 で起票済み）。本書は `docs/spec/` の改訂ではなく、本リポジトリ側の
実装規範として `docs/design/` に置く。

## 2. スコープの確認

- **docs-only**。実装（位置計算モジュール・CSS 変数出力・4 コンポーネントへの接続・
  wasm 層での計測値注入）は #590 のスコープであり、本書では変更しない。
- 依存クレート追加は**ゼロ**（REQ-3 依存上限 60 件/深さ 6。`fandhe-frontend-headless-ui`
  は `fandhe-frontend-core`/`fandhe-frontend-interactive` への内部（workspace 内）依存のみを
  維持し、外部クレートは追加しない）。

## 3. 制約の確認

- **外部依存ゼロ**: Floating UI・zag.js 等の外部クレートを移植・vendor 同梱せず、
  自前の純粋関数として実装する。
- **`#![forbid(unsafe_code)]`（REQ-2）**: `crates/headless-ui/` は `core`/`interactive` と
  同じ `forbid(unsafe_code)` 域であり、位置計算モジュールも `unsafe` を使わない。
- **既定エスケープ（REQ-1）**: 計算結果を HTML へ反映する経路は既存の
  `attrs: Vec<(&'a str, &'a str)>` → `fandhe_frontend_core::render` の既定エスケープ経由のみ
  とし、`raw_html()` は使用しない。`format!` による HTML 文字列直接組み立ても禁止
  （`coding-rust.md`）。
- **signal/store 非採用の継承**（`docs/policy/intentional-non-adoption.md` §3.4）: 位置
  計算の再実行トリガー（スクロール・リサイズ等）はイベント駆動の明示的呼び出し
  （wasm 層の既存 dispatch モデル）で行い、新規のリアクティブ状態管理機構を導入しない。
- **AI 開発前提の評価軸**（`docs/policy/intentional-non-adoption.md` §2: 明示性・決定性・
  機械検証可能性・コンテキスト消費）を、本書の設計判断（第 4 節）の根拠軸として使う。

## 4. 設計判断

### 4.1 計算方式: headless-ui 内の純粋関数 + wasm 層での計測値注入

位置計算は `crates/headless-ui/src/positioning.rs`（モジュール名は #590 で確定）に
**純粋関数・決定的**な実装として置く。

| 項目 | 内容 |
|------|------|
| 入力 | anchor（トリガー等）矩形（x/y/width/height）・floating（positioner）要素の寸法（width/height）・viewport 寸法（width/height）・希望 placement（第 4.2 節の語彙）・offset（主軸方向のギャップ）・有効フラグ（flip/shift/sameWidth の on/off） |
| 出力 | 確定座標（x/y）・確定 placement（flip 適用後の実際の side/align）・arrow 座標（arrow を持つコンポーネントのみ） |
| 実行環境 | `headless-ui` は `web-sys` 非依存のまま維持する。`getBoundingClientRect` 等の実 DOM 計測は行わない |
| 計測値の注入元 | `fandhe-frontend-wasm-full`（Popover/Tooltip/Menu）・`fandhe-frontend-wasm-client` 相当の CSR 層が実 DOM 計測値を取得し、純粋関数へ渡す。SSR/SSG（DOM 非存在）では計測値がないため、計算自体をスキップし CSS フォールバック（`positioner` の `data-side`/`data-align` 属性 + pre-styled-ui 側の静的 CSS）で初期表示を劣化なく描画する |
| 異常系 | 計測不能（viewport 外・寸法 0 等）の場合は fail-closed とし、既定 placement のまま座標を返す（`panic!`/`unwrap()` を使わない、`coding-rust.md` のライブラリコード規約に従う） |

この分離（純粋関数 = headless-ui、計測 = wasm 層）を選んだ根拠:

| 評価軸 | 根拠 |
|--------|------|
| 明示性 | 「入力（矩形・寸法）→ 出力（座標）」の関数シグネチャが計算の全体像を一目で示す。DOM 計測タイミング（いつ再計算するか）は呼び出し側（wasm 層）が明示的に呼ぶことで決まり、暗黙のオブザーバ機構を持たない |
| 決定性 | 同一入力（矩形・寸法・placement・フラグ）に対し同一出力を返す純粋関数のため、決定的なユニットテストが可能（表駆動テスト、第 7 節） |
| 機械検証可能性 | `web-sys` 非依存を保つことで、`headless-ui` の既存回帰テスト（`crates/core/tests/no_branching_across_modes.rs` 等）と同様、native `cargo test` のみで検証できる |
| コンテキスト消費 | `headless-ui` に DOM 計測 API を持ち込むと、SSR/CSR 両対応という既存の責務境界（`docs/api/headless-ui-api.md` §2「位置づけ」）が曖昧になる。計測とロジックの分離により、AI エージェントが変更時に把握すべき範囲を「純粋関数の入出力契約」に限定できる |

### 4.2 placement 範囲

ark-ui / Floating UI 準拠の 12 placement を語彙として凍結する:

`top` / `top-start` / `top-end` / `bottom` / `bottom-start` / `bottom-end` /
`left` / `left-start` / `left-end` / `right` / `right-start` / `right-end`

- `data-side`（`top`/`bottom`/`left`/`right` のいずれか、主軸方向）・`data-align`
  （`start`/`center`/`end`、交差軸方向）へ確定後の placement を分解して反映する。
  既存の `data_state`（`crate::data_attrs`）と同じ「値語彙を型・定数へ一元化し、
  本モジュールで独自の値を作らない」規約を継承する。
- Select の `positioner` は arrow を持たないため、`data-side`/`data-align` のみを
  出力する（arrow 座標計算は Popover/Tooltip/Menu のみ対象）。

### 4.3 flip / shift / sameWidth の採否

| middleware | 採否 | 内容 |
|-----------|------|------|
| **flip** | 採用（主軸の単純反転のみ） | 希望 placement で viewport からはみ出す場合、主軸（`top`⇄`bottom` または `left`⇄`right`）を反転した 1 候補のみを試す。Floating UI の `flip()` が持つ複数フォールバック候補列・`autoPlacement` 相当の全方位探索は非採用（下記参照）。反転後も収まらない場合は反転後の座標をそのまま採用する（viewport 内に収める保証は shift が担う） |
| **shift** | 採用（限定版、viewport 内クランプのみ） | 交差軸方向で floating 要素が viewport をはみ出す場合、viewport 内に収まる座標へクランプする。Floating UI の `shift()` が持つ `limiter`（`limitShift()`）等の高度な制限指定は非採用 |
| **sameWidth** | 採用 | anchor（トリガー等）の幅を CSS 変数として出力し、Select/Menu のドロップダウン幅を anchor 幅に一致させる用途に使う |

**意図的非対応の明記**（Floating UI の高度 middleware）:

| 項目 | 非採用理由（評価軸） | 再評価トリガー |
|------|----------------------|----------------|
| `autoPlacement`（全方位から最適解を探索） | 決定性: 探索順序・評価関数の実装差でエッジケースの結果が変わりやすく、本書の flip（1 候補のみの単純反転）と比べ挙動の予測可能性が下がる。コンテキスト消費: 探索アルゴリズムの理解コストが増える | 単純な主軸反転（flip）では実運用上 viewport 内に収まらないケースが実測で確認された場合 |
| `inline`（インライン折り返しテキストの矩形分割対応） | コンテキスト消費: 折り返しテキストを参照要素とするユースケースが本フレームワークの 4 コンポーネント（Popover/Tooltip/Menu/Select、いずれもボタン・トリガー要素が anchor）に存在しない | インライン要素（`<a>` 内テキスト範囲等）を anchor とするコンポーネントの需要が確定した場合 |
| `hide`（anchor が viewport 外に出た際の非表示制御） | 機械検証可能性: 「anchor の可視性」の判定はスクロール位置の連続監視を要し、本書が定める「呼び出し側が明示的に呼ぶ」という単純な再計算モデル（signal/store 非採用の継承）と相性が悪い | スクロール連動の連続監視機構（IntersectionObserver 相当）の導入がユーザー承認を得て確定した場合 |
| `size`（sameWidth 以外、floating 要素自体の高さ等を viewport に合わせて動的に縮小） | コンテキスト消費: 高さの動的リサイズは CSS（`max-height` + `overflow`）側で静的に対応可能な範囲が大きく、JS 計算側に持ち込む必要性が低い | 動的な高さ調整（viewport 残り高さに応じた `max-height` の実行時計算）が pre-styled-ui 側の CSS だけでは表現できないケースが確定した場合 |
| `VirtualElement`（実 DOM 要素を持たない仮想参照要素、例: マウスカーソル追従） | 明示性・コンテキスト消費: 本フレームワークの anchor は常に実 DOM 要素（トリガー・アンカーパーツ）であり、仮想参照要素の抽象を導入すると入力契約（第 4.1 節）に分岐が生じる | コンテキストメニュー等、マウス座標を anchor とするコンポーネントの実装が確定した場合 |
| ポインタ追従・アニメーションフレーム連動の連続再計算（`autoUpdate` 相当） | 決定性・コンテキスト消費: `requestAnimationFrame` 連動の連続監視はテストの決定性を弱め、signal/store 非採用の評価軸（`intentional-non-adoption.md` §3.4）と同じ理由で非採用とする | スクロール・リサイズの都度呼び出し（イベント駆動の離散再計算）では実用上不十分と判明した場合 |

### 4.4 CSS 変数出力

pre-styled-ui の既存トークン規約（`--fandhe-*` プレフィックス、`crates/pre-styled-ui/src/css.rs`
の `decl()`）に整合するプレフィックスで以下の CSS 変数名を凍結する。

| 変数名 | 意味 |
|--------|------|
| `--fandhe-x` | floating 要素の確定 x 座標（px） |
| `--fandhe-y` | floating 要素の確定 y 座標（px） |
| `--fandhe-reference-width` | anchor（参照要素）の幅（px、sameWidth 用） |
| `--fandhe-arrow-x` | arrow の x 座標（px、arrow を持つコンポーネントのみ） |
| `--fandhe-arrow-y` | arrow の y 座標（px、arrow を持つコンポーネントのみ） |

**出力経路と `pre-styled-ui` の静的 CSS（`decl()`）との違い（重要な区別）**:

`crates/pre-styled-ui/src/css.rs` の `Declaration`/`decl()` はプロパティ名・値ともに
`&'static str` に固定されており、動的文字列を受け付けない設計（同ファイル rustdoc
「実行時入力が CSS 規則へ直接混入する経路を型レベルで塞ぐ」）である。これは
**ビルド時に確定する静的スタイルシート**（`SlotRecipe::css` によるセレクタベースの
CSS 生成）のための制約であり、位置計算のように**実行時に anchor 矩形から動的に
決まる座標値**はこの静的経路には収まらない。

anchor positioning の CSS 変数は、既存の `positioner`/`arrow`/`arrow_tip` 関数が
既に受け付けている呼び出し側 `attrs: Vec<(&'a str, &'a str)>` 引数（`&'a str` であり
`&'static str` 固定ではない、既存の動的値契約）へ `style` 属性として渡す経路を使う。
すなわち:

1. wasm 層（#590 で実装）が計測値を純粋関数へ渡して座標を計算する。
2. wasm 層が `format!("--fandhe-x: {x}px; --fandhe-y: {y}px; ...")` の形で **内部生成の
   数値書式のみ**からなる `style` 値文字列を組み立てる（ユーザー入力を直接埋め込まない、
   第 6 節参照）。
3. `("style", &computed_style)` を `positioner`/`arrow`/`arrow_tip` の `attrs` へ渡す。
4. `fandhe_frontend_core::render` の既定エスケープ（属性値エスケープ）を経由して出力する。

この経路は新規 API を追加するものではなく、既存の `attrs` 引数契約をそのまま用いる。
`docs/policy/attribute-output-policy.md`・`docs/policy/intentional-non-adoption.md` §3.7
（`style` 属性の CSS サニタイザ非採用、代替として属性値エスケープの breakout 防止に
依拠する判断）とも整合する。

### 4.4a 実装補記（PR #622 レビュー指摘の反映）

#590 の実装（`crates/wasm-full/src/position.rs`・`crates/headless-ui/src/positioning.rs`）
に対する PR #622 のレビューで判明した、本書執筆時点の記述と実装の乖離 2 点を反映する:

- **`--fandhe-reference-width` は `sameWidth` が有効なときのみ出力する**:
  `css_vars_style(position, reference_width, same_width)` は `same_width` を追加引数
  として受け取り、`false` の場合は変数自体を出力しない。本書 §4.4 の表・§4.3 の
  「sameWidth: 採用」自体は変わらないが、`PositioningConfig::same_width`
  （`PositionedKind::same_width_default`: Menu/Select は `true`、Popover/Tooltip は
  `false`）が実際に出力へ反映されることを明記する。
- **`data-side`/`data-align` は「確定」side/align の出力専用であり、「希望
  placement」の入力としては使わない**: §4.2 が定める `data-side`/`data-align` の
  役割（flip 適用後の確定値を分解して反映する）自体は変わらないが、`wasm-full` の
  再計算（`reposition_all`/`reposition_one`）はこれらを**書き込み専用**として扱う。
  「希望 placement」（flip 適用前の入力）は独立した `data-requested-side`/
  `data-requested-align` 属性（`reposition_one` が初回のみ書き込み、以後は上書き
  しない永続化領域）に保持する。`data-side`/`data-align` 自体を希望として読み戻すと、
  flip 後の side が次回の希望として扱われてしまい、viewport のスペースが戻っても
  元の希望へ戻せない不具合があったため（PR #622 レビュー指摘）。

### 4.5 CSS Anchor Positioning（Web 標準）採用可否の評価

CSS Anchor Positioning（`anchor-name` / `position-anchor` / `position-try-fallbacks` 等）
のブラウザ実装状況を調査した（調査日 2026-07-22、[MDN CSS Anchor Positioning](
https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_anchor_positioning)・[Can I Use:
css-anchor-positioning](https://caniuse.com/css-anchor-positioning) を参照）。

| ブラウザ | 対応バージョン | 備考 |
|---------|---------------|------|
| Chrome / Edge | 125 以降 | v117〜124 はデフォルト無効（フラグ有効化が必要） |
| Firefox | 147 以降 | v145〜146 はデフォルト無効 |
| Safari | 26.0 以降 | 対応は 3 エンジン中もっとも遅い |
| グローバル使用率（caniuse 集計） | 約 81.67% | Firefox/Safari の対応が直近であるため、旧バージョン利用者の非対応割合が残る |

**判断: 現時点（2026-07）では非採用とする（安全側の既定）。**

- Baseline の「Widely available」判定は、主要 3 エンジンでの対応が揃ってから
  概ね 30 か月（2.5 年）経過した機能に付与される。Firefox（v147）・Safari（v26.0）
  の対応時期はいずれも直近であり、本書執筆時点で「Widely available」の基準（3 エンジン
  対応後 30 か月経過）を満たしていない（Firefox/Safari 対応版のリリースからの経過期間が
  短い）。安全側の既定に従い、Baseline 未達である限り非採用とする。
- (a) **正式方式**: 第 4.1〜4.4 節の JS（wasm 層）計測 + 純粋関数計算 + CSS カスタム
  プロパティ出力を正式方式として採用する。
- (b) **将来の progressive enhancement 候補**: CSS Anchor Positioning が Baseline
  「Widely available」に到達した場合、`@supports (anchor-name: --a)` 等の機能検出を
  用いた progressive enhancement（対応ブラウザは CSS のみで位置決めし、JS 計算を
  スキップする）を再評価候補とする。ただし SSR/no-JS 環境での初期表示・flip/shift の
  挙動（`position-try-fallbacks` が本書の flip/shift 相当をどこまでカバーするか）の
  再検証が必要であり、採用は本書の再改訂を伴う。
- (c) **再評価トリガー**: CSS Anchor Positioning が Baseline「Widely available」へ
  到達したとき、またはブラウザサポート状況の実測が本節の記載と乖離したことが判明した
  とき。

### 4.5a progressive enhancement 検討記録（イシュー #644）

第 4.5 節の非採用判断（`intentional-non-adoption.md` §3.21 へ転記済み）を
前提に、イシュー #644（親: #640）は「`@supports` による段階適用（対応
ブラウザはネイティブ位置決め・非対応は本書 4.1〜4.4 節の wasm 計測実装へ
フォールバック）」の設計自体を検討記録として残すことを受け入れ条件とする。
本節はその検討結果であり、**判断（非採用）自体は変更しない**。

**Baseline 再確認結果**: 本節執筆時点（2026-07-23、#644 着手時）でネットワーク
経由の再調査は行わず、第 4.5 節（調査日 2026-07-22）の記載（Chrome/Edge
125+・Firefox 147+・Safari 26.0+・グローバル使用率約 81.67%）を最新の一次
記録として引用する。1 日の経過では Baseline「Widely available」判定（3
エンジン対応後 30 か月経過）に影響する変化は生じ得ず、判断に影響しない。

**フォールバック設計案（採用した場合の構成、記録のみ・実装しない）**:

| 層 | 案 |
|----|----|
| CSS（`pre-styled-ui`） | `crates/pre-styled-ui/src/css.rs` の `SlotRecipe::css`（`&'static str` 固定の `decl()` 契約）内に `@supports (anchor-name: --a)` ブロックを追加し、trigger パーツ（`data-part` セレクタ）へ `anchor-name`、positioner パーツへ `position-anchor` / `position-area` / `position-try-fallbacks` を宣言する。sameWidth 相当は `anchor-size(width)` で表現する。宣言値はすべてビルド時に確定する `&'static str` であり、`decl()` の型制約と両立する（第 4.4 節の「動的 `style` 値」とは別経路） |
| wasm（`fandhe-frontend-wasm-full`） | `reposition_all` / `reposition_one`（`crates/wasm-full/src/position.rs`）の入口で `CSS.supports("anchor-name: --a")` による機能検出を行い、対応ブラウザでは JS 計測・`--fandhe-x`/`--fandhe-y` の `style` 出力（第 4.4 節）をスキップして CSS 側へ委譲する |
| SSR / no-JS | 変更不要。既存の `placement_attrs`（`data-side`/`data-align` の静的出力、第 4.2 節）+ pre-styled-ui の静的 CSS フォールバックのままとする |

**非採用の論点整理（第 4.5 節・`intentional-non-adoption.md` §3.21 からの
補足）**:

- Baseline「Widely available」未達（再評価トリガー未成立、第 4.5 節 (c)）。
- ネイティブ経路では `position-try-fallbacks` による flip 発生を
  `data-side`/`data-align`（第 4.4a 節が定める「確定値の書き込み専用」契約）
  へ反映する標準的な通知手段がなく、`data-requested-side`/
  `data-requested-align` との分離契約とも乖離する。
- 両経路（CSS ネイティブ / wasm 計測）のブラウザテスト行列が倍増し、
  決定性・機械検証可能性・コンテキスト消費（`intentional-non-adoption.md`
  §2）の評価軸で不利。
- `position-try-fallbacks` の候補列挙モデルは第 4.3 節が凍結した「主軸単純
  反転 1 候補のみの flip + viewport クランプ shift」より表現が広く、挙動の
  同値性検証自体が新規コストになる。

**結論**: 非採用（変更なし）。フォールバック設計案・論点整理は
`intentional-non-adoption.md` §3.21 の参照行へ追記済み（新規節番号は
追加しない。同文書の採番規則により §3.21 の内容自体は書き換えない）。

## 5. 後続タスク（#590）への引き継ぎ

| 項目 | 内容 |
|------|------|
| 純粋関数のシグネチャ案 | 第 4.1 節の入出力契約（anchor 矩形・floating 寸法・viewport 寸法・placement・offset・フラグ → 座標・確定 placement・arrow 座標）を Rust の具体的な構造体・関数シグネチャへ落とす。`headless-ui` 内モジュール名は `positioning.rs`（仮）とする |
| CSS 変数名一覧 | 第 4.4 節の表（`--fandhe-x`/`--fandhe-y`/`--fandhe-reference-width`/`--fandhe-arrow-x`/`--fandhe-arrow-y`）をそのまま実装で使う |
| wasm 層計測注入インターフェース案 | `fandhe-frontend-wasm-full`（Popover/Tooltip/Menu）・CSR 層が `web-sys` の `getBoundingClientRect` 相当で矩形を取得し、純粋関数へ渡すグルーコードの設計は #590 のスコープ |
| 再計算タイミング | スクロール・リサイズイベントを契機とした離散的な再計算呼び出し（`autoUpdate` 相当の連続監視は非採用、第 4.3 節）の具体的なイベント配線は #590 のスコープ |
| テスト観点 | 決定的ユニットテスト（native、`cargo test -p fandhe-frontend-headless-ui`）: 入力矩形の組み合わせ → 期待座標の表駆動テスト。flip の境界値（viewport 端ちょうど・はみ出し 1px 前後）・shift のクランプ境界・sameWidth の幅一致を個別ケースとして持つ |

## 6. スコープ外の明記

| 項目 | 引き継ぎ先 |
|------|-----------|
| 第 4.3 節で非対応と確定した Floating UI 高度 middleware（autoPlacement / inline / hide / size（sameWidth 以外）/ VirtualElement / ポインタ追従・連続再計算） | イシュー #639 で `docs/policy/intentional-non-adoption.md` §3.20 へ転記済み。同節が第 4.3 節・本節（第 6 節）への相互参照を持つ。CSS Anchor Positioning（第 4.5 節）の非採用判断は同書 §3.21 へ転記済み |
| `docs/api/headless-ui-api.md` への本設計の反映 | イシュー #666 で反映済み |
| CSS Anchor Positioning の progressive enhancement 実装（第 4.5 節 (b)） | イシュー #644 で検討記録（フォールバック設計案・非採用の論点整理）を第 4.5a 節へ追加済み。判断（非採用）は変更なし。実装は Baseline「Widely available」到達後の再評価まで着手しない |

新規の Issue 起票が必要な事項は、本書執筆時点では起票せず、PR 本文で提案に留め、
`out-of-scope-tracking.md` の手順（ユーザー承認を得てから起票）に従う。

## 7. セキュリティ不変条件

1. **既定エスケープの一貫性（REQ-1）**: CSS 変数値を含む `style` 属性値は必ず
   `attrs: Vec<(&'a str, &'a str)>` → `fandhe_frontend_core::render` の既定エスケープ
   （属性値エスケープ）経由で出力する。`raw_html()` は使用しない。`format!` による
   HTML 文字列直接組み立ては禁止（`coding-rust.md`）。CSS 文字列自体の直接組み立て
   （セレクタ・宣言の手動連結によるスタイルシート生成）も、本書が定める `style`
   属性の数値埋め込み以外の経路では行わない。
2. **内部生成の数値書式に限定**: `style` 属性値へ埋め込む値は、wasm 層が計算した
   座標（px 単位の数値）のみとし、ユーザー入力（フォーム値・URL パラメータ等）を
   直接 `style` 値へ流さない。これにより `intentional-non-adoption.md` §3.7
   （`style` 属性の CSS サニタイザ非採用、代替として属性値エスケープの breakout
   防止に依拠する判断）との整合を保つ。属性値エスケープが `"` による属性境界の
   脱出を防止する前提は変わらない。
3. **属性名は `&'static str` 固定**: `data-side`/`data-align`/`style` 等の属性名スロット
   自体は既存の `crate::anatomy`/`crate::data_attrs` の不変条件（属性名は
   `&'static str` リテラル固定、動的値が属性名スロットへ混入する経路なし）を継承し、
   本書の設計判断では変更しない。
4. **fail-closed（A04 相当）**: 計測不能・viewport 外などの異常系で `panic!`/`unwrap()`
   せず、既定 placement のまま座標を返す（第 4.1 節）。`headless-ui` の既存
   `Component`/`Hydrate` 実装（`HydrateError` を返す fail-closed 契約）と同じ思想を
   位置計算にも適用する。
5. **hydration 属性由来の値の扱い**: 位置計算の入力（矩形・寸法）は wasm 層が実 DOM
   から取得する計測値であり、クライアント側で改ざんされうる `data-hydrate-*` 属性値
   とは異なる（サーバー注入状態の改ざん耐性契約とは別系統）。ただし wasm 層が
   計測値を純粋関数へ渡す際、異常値（負の幅・`NaN`/`Infinity` 相当）を受け取っても
   `panic!` しないことを純粋関数側の契約として定める（#590 実装時のテスト観点、
   第 5 節）。
6. **サプライチェーン（REQ-3）**: 依存クレート追加はゼロ。`fandhe-frontend-headless-ui`
   の外部依存ゼロ・`#![forbid(unsafe_code)]` を維持する。

## 8. 受け入れ基準対応表

| #588 受け入れ条件・#589 本文の要求 | 満たす設計要素 |
|-------------------------------------|----------------|
| 外部依存ゼロでの実装方針 | 第 3 節（制約の確認）・第 4.1 節（純粋関数 + wasm 層計測注入の分離） |
| 対応する placement 範囲の明記 | 第 4.2 節（12 placement 語彙） |
| flip / shift / sameWidth の採否 | 第 4.3 節 |
| CSS Anchor Positioning（Web 標準）採用可否の評価 | 第 4.5 節 |
| 意図的非対応の明記 | 第 4.3 節（Floating UI 高度 middleware の非採用表）・第 6 節（`intentional-non-adoption.md` §3.20〜§3.21 への転記済み） |

## 9. 関連文書との整合確認

- `docs/api/headless-ui-api.md` §2（位置づけ）・§6（セキュリティ不変条件）と、本書
  第 4.1 節（`headless-ui` は `web-sys` 非依存を維持）・第 7 節（既定エスケープ・
  属性名固定の継承）は矛盾しない。本書は既存の責務境界を変更せず、位置計算という
  新しい純粋関数の置き場所を同じ境界内に確定するのみである。
- `docs/policy/attribute-output-policy.md`（属性出力の脅威マトリクス・URL 属性の
  許可スキーム検証）と、本書第 4.4 節・第 7 節（`style` 属性値を内部生成の数値のみに
  限定し既定エスケープを経由する契約）は整合する。`style` 属性は同文書が定める
  URL スキーム検証の対象外（値が URL を含まない）であり、新たな脅威面を追加しない。
- `docs/policy/intentional-non-adoption.md` §2（AI 開発前提の評価軸）・§3.4（signal/store
  非採用）・§3.7（`style` 属性の CSS サニタイザ非採用）と、本書第 3 節・第 4.3 節・
  第 4.4 節・第 7 節はそれぞれ整合する。本書第 4.3 節の非採用表は、同文書
  §3.20（イシュー #639 で転記済み、第 6 節参照）へ同じ評価軸・書式で転記
  されている。本書第 4.5 節（CSS Anchor Positioning 非採用）も同様に同文書
  §3.21 へ転記済みである。第 4.5a 節（イシュー #644、progressive
  enhancement のフォールバック設計案・非採用の論点整理）は判断を変更せず
  §3.21 の参照行へ追記されており、同節本文・節番号自体は変更しない
  （同文書の採番規則、§3.6〜§3.8 の重複発覚を受けた明記部分を参照）。
- `crates/pre-styled-ui/src/css.rs` の `Declaration`/`decl()`（静的 CSS 宣言、
  `&'static str` 固定）と、本書第 4.4 節（動的な `style` 属性値、既存 `attrs` 引数
  経由）は異なる経路であることを明記し、両者の整合を混同しないよう区別した
  （第 4.4 節「出力経路と `pre-styled-ui` の静的 CSS（`decl()`）との違い」参照）。
- PR #563（Popover 実装、イシュー #532）の out-of-scope 記載（「overlays 親（#530）
  配下での共通化検討をユーザー承認のうえ別イシューへ切り出す想定」）は、本書が
  #588/#589/#590 の系譜としてその共通化検討を正式に引き継いだものである。
