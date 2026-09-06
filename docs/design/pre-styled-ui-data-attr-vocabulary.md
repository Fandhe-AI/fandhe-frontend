# pre-styled-only 部品の `data-*` 語彙 決定記録

- **イシュー**: [#1063](https://github.com/Fandhe-AI/fandhe-frontend/issues/1063)（親: #1057「headless / pre-styled の責務分離整備」、祖父: #1056）
- **対象**: `crates/pre-styled-ui/`
- **関連**: `docs/policy/intentional-non-adoption.md` §3.25（UI 部品の責務境界）

## 1. 背景

`crates/headless-ui/src/data_attrs.rs` は 16 個の共有 `data-*` ヘルパを提供するが、pre-styled-only 部品（headless-ui に対応部品が存在しない部品）は生タプルで `data-*` を出力しており、語彙の定義元・使い分け基準がどこにも明文化されていなかった。本文書はイシュー #1063 の成果物として、全数洗い出し・方針決定・適用結果を記録する。

### 1.1 イシュー本文の前提の是正

着手前調査の結果、イシュー本文が挙げる 4 例のうち 3 例は誤認である。

| イシュー本文の記述 | 調査結果 | 実際の出力元 |
|---|---|---|
| `floating_panel.rs` の `data-stage` | pre-styled の **recipe 内 `StateCondition::AttrEq`（＝ CSS セレクタ側の参照）**。属性を出力していない | `crates/headless-ui/src/floating_panel.rs` |
| `json_tree_view.rs` の `data-kind` | 同上（recipe の CSS セレクタ） | `crates/headless-ui/src/json_tree_view.rs` |
| `calendar.rs` の `data-today` / `data-outside-month` | 同上（recipe の CSS セレクタ） | `crates/headless-ui/src/calendar.rs` |
| `button.rs` の `data-loading` | **真に pre-styled 側の出力** | `crates/pre-styled-ui/src/button.rs` |

すなわち「pre-styled-only 部品が生タプルで独自語彙をばらばらに定義している」という前提は成立せず、pre-styled が実際に `data-*` を**出力**している非 anatomy 箇所は **5 種 6 行**のみである（§2.1）。この事実確認自体が受け入れ条件「全数洗い出し」の中核であり、方針判断（集約 vs 明文化）の根拠でもある。

## 2. 現状調査結果（全数洗い出し）

`data-*` 名は本リポジトリで **3 つの異なる役割**を持つ。役割を混ぜて数えると誤った結論になるため、レジストリは 3 列で記録する。

### 2.1 役割 A: pre-styled-ui が「出力」する非 anatomy `data-*`（本イシューの真の対象）

| 属性 | 出力箇所 | 値域 | 同名属性の他層での出力 | recipe（CSS）からの参照 | 判定 |
|---|---|---|---|---|---|
| `data-current` | `crates/pre-styled-ui/src/tab_nav.rs::link` | 存在属性（`""`） | headless に `data_attrs::data_current` ヘルパが既に存在 | なし | **是正済み**: 生タプル → ヘルパ経由へ変更 |
| `data-loading` | `crates/pre-styled-ui/src/button.rs::button_internal`（`loading` 分岐） | 存在属性（`""`） | なし（headless-ui に 0 件） | なし（CSS 消費者なし・利用者 CSS/JS 用フック） | pre-styled-only 語彙として維持 + rustdoc 明文化 |
| `data-action` | `crates/pre-styled-ui/src/tag.rs::close_trigger` | 動的文字列（dispatch action 識別子） | `crates/headless-ui/src/timer.rs::action_trigger`（start/pause/resume/reset） | なし | **同一意味論の共有語彙**（「クリック時に発火する action 識別子」）。改名せず、値域差を rustdoc に明記 |
| `data-value` | `crates/pre-styled-ui/src/radio_card.rs::item` | 動的文字列（選択肢の値） | headless の `radio_group` / `checkbox_group` / `toggle_group` / `tree_view` / `rating_group`（いずれも生タプル、ヘルパなし） | なし | 同一意味論の共有語彙。両層ともヘルパ未整備だが本イシューでは新設しない（§3.3） |
| `data-series` | `crates/pre-styled-ui/src/charts/radar_chart.rs`、`crates/pre-styled-ui/src/charts/scatter_chart.rs` | 動的文字列（系列名） | なし | なし | charts は pre-styled-only。pre-styled-only 語彙として維持 + rustdoc 明文化 |

### 2.2 役割 B: pre-styled-ui が「参照のみ」する `data-*`（recipe の `StateCondition`）

出力元は他層。pre-styled のアドホック語彙ではない。

| 属性 | pre-styled 側の参照箇所 | 出力元 |
|---|---|---|
| `data-stage` | `floating_panel.rs` | headless `floating_panel.rs` |
| `data-kind` | `json_tree_view.rs` | headless `json_tree_view.rs` |
| `data-today` / `data-outside-month` | `calendar.rs` | headless `calendar.rs` |
| `data-selected` | `pagination.rs` / `tree_view.rs` / `calendar.rs` | headless `pagination.rs` / `tree_view.rs` ほか |
| `data-placement` | `drawer.rs` | headless `drawer.rs` / `toast.rs` |
| `data-position` | `image_cropper.rs` | headless `image_cropper.rs`（イシュー #1610 で `data-handle-position` から改名。参照実装〔ark-ui/zag.js〕の語彙と一致させるため） |
| `data-side` / `data-align` | `tour.rs` | headless `positioning.rs` |
| `data-placeholder` | `date_input.rs` | headless `date_input.rs` |
| `data-placeholder-shown` | `editable.rs` | headless `editable.rs` / `select.rs` |
| `data-autoresize` | `textarea.rs` | headless `field.rs` |
| `data-empty` | （テストのみ、`signature_pad.rs`） | headless `signature_pad.rs` |
| `data-positioned` | `select.rs` / `menu.rs` / `combobox.rs` | `crates/wasm-full/src/position.rs`（実行時に wasm 層のみが付与、UI 2 層はいずれも出力しない。イシュー #663 の設計） |
| `data-disabled` | `field.rs`（イシュー #1684、`label`/`helper-text` slot への state 規則）、`fieldset.rs`（イシュー #1686、`legend`/`helper-text` slot への state 規則） | headless `field.rs`（`FieldProps::disabled` から `state_data_attrs` が生成）、headless `fieldset.rs`（`FieldsetProps::disabled` から `state_data_attrs` が生成） |

### 2.3 役割 C: 「予約名として防御的に列挙」される `data-*`

呼び出し側 `attrs` からの偽装を `drop_reserved` で除去するための名前リスト。出力でも参照でもない。

- `radio_card.rs`（`ROOT_RESERVED` / `STATE_RESERVED` / `ITEM_RESERVED` / `HIDDEN_INPUT_RESERVED`）
- `tab_nav.rs`（`LINK_RESERVED = ["href", "aria-current", "data-current"]`）
- `checkbox_card.rs` / `charts/tooltip.rs`（`DATUM_RESERVED`）

### 2.4 スコープ外（語彙ではないもの・誤検知注意点）

洗い出し時の grep が拾うが語彙ではないもの。後続の読み手が調査を繰り返さないよう記録する。

- `data-hydrate-*` プレフィックス: `fandhe-frontend-interactive` の `Hydrate` 由来。pre-styled `src/` ではテストアサーションにのみ出現
- **`"data-list"` は scope 名であって属性名ではない**（`data_list.rs` の `anatomy("data-list")`、`SlotRecipe::new("data-list", SLOTS)` ほか）
- テスト用ペイロードの属性名: `data-x`（XSS 回帰の共通ペイロード）/ `data-testid` / `data-foo`（`class_attr.rs`）/ `data-note`（`tab_nav.rs`）
- anatomy 由来の `data-scope` / `data-part`（`Anatomy::part` が固定出力）

## 3. 決定方針と判断根拠

**結論: 「限定的な是正 ＋ 明文化」のハイブリッドを採用し、pre-styled 側に第 2 の `data_attrs` レジストリは新設しない。**

### 3.1 規約 A（層帰属規則）

`data-*` 語彙の定義元は「その属性を**出力**する層」に唯一存在する。headless-ui にラップ対象部品が存在する場合、`data-*` の出力は headless 層の責務であり、pre-styled 層は recipe の `StateCondition` から**参照するだけ**とする（`docs/policy/intentional-non-adoption.md` §3.25 の層責務分離の系）。§2.2 の属性群はこの規約に既に適合している。

### 3.2 規約 B（pre-styled-only 部品の独自語彙）

headless-ui に対応部品が存在しない pre-styled-only 部品（Button / Tag / RadioCard / TabNav / charts 各種）は独自の `data-*` 語彙を持ってよい。ただし次の 3 条件を必須とする。

1. **B-1**: headless の `data_attrs` に**同名ヘルパが既に存在する語彙は必ずそのヘルパを経由する**（生タプルでの再定義を禁止）。→ `tab_nav.rs` の `data-current` が唯一の違反であり是正済み。
2. **B-2**: **同名属性は同一意味論でのみ再利用し、値域を部品モジュール rustdoc に明記する。意味論が異なる場合は別名を使う。**
   - 評価済みケース: `data-action`（pre-styled `tag` の dispatch action 識別子 / headless `timer` の control kind）は、いずれも「この要素をクリックしたときに発火する action の識別子」で意味論が同一であるため共有語彙と判定し、改名しない。値域が部品ごとに異なる点のみ rustdoc に明記する。
   - 評価済みケース: `data-value`（pre-styled `radio_card` / headless の 5 部品）も「項目の値」で意味論同一。共有語彙として維持する。
3. **B-3**: 語彙（属性名・値域・意味・付与条件・CSS 消費者の有無）を部品モジュール rustdoc に「`data-*` 語彙」節として明記し、本文書のレジストリから相互参照できるようにする。

### 3.3 規約 C（集約はしない）— 判断根拠

pre-styled 側に `data_attrs` モジュールを新設して `data_loading()` / `data_series()` / `data_value()` 等を提供する案は**採らない**。

1. **重複削減効果がない**: 対象語彙は 5 種であり、`data-loading` 1 箇所・`data-action` 1 箇所・`data-value` 1 箇所・`data-series` 2 箇所。単一利用のヘルパ化は行数を増やすだけで呼び出し側の重複を減らさない。
2. **語彙レジストリが 2 系統に分岐する**: 「headless `data_attrs` / pre-styled `data_attrs` のどちらを見ればよいか」という判断が全部品の実装・レビューに恒常的に発生する。AI 開発・保守前提の評価軸（明示性・機械検証可能性・コンテキスト消費、`docs/policy/intentional-non-adoption.md` の評価軸）に照らして負に働く。
3. **層帰属規則（規約 A）と衝突する**: 共有語彙（`data-action` / `data-value`）を pre-styled 側ヘルパに置くと「headless も出力する語彙が pre-styled に定義される」逆転が生じる。かといって headless 側へ追加すると `fandhe-frontend-headless-ui` の公開 API 拡張（semver バンプ + 依存元 3 クレートの `version` 追随、`xtask check-dep-versions`）が本イシューの目的（語彙の整理）に対して過大なコストになる。
4. **エスケープ迂回経路を増やさない**: `data-value` / `data-action` / `data-series` は動的値を運ぶ。ヘルパ層を挟まず `Anatomy::part` → `core::render` の既定エスケープ経路に直結させたままにするほうが、セキュリティ不変条件（REQ-1）の追跡経路が短い。

### 3.4 機械検証の方針（ソース走査スキャナは作らない）

`crates/pre-styled-ui/src/**` を正規表現で走査して `data-*` リテラルを許可リストと突合する fail-closed テストは**採用しない**。§2.4 の誤検知（scope 名 `"data-list"`・`*_RESERVED` の名前リスト・doc コメント内の言及・CSS 期待値文字列リテラル・テストペイロード）をすべて特例化する必要があり、`#[cfg(test)]` 位置の慣習にも依存するため、契約テストとして脆い。

代替として:

- **本イシュー**: §2.1 の 5 出力箇所を**レンダリング結果で固定する**契約テスト（`crates/pre-styled-ui/tests/data_attr_vocabulary.rs`）を追加した。属性名・値・付与条件・非付与条件を出力 HTML で直接アサートする決定的なテストであり、誤検知が原理的に起きない。
- **将来のアドホック語彙追加の抑止**: レビュー（reviewer / 本文書のレジストリ）で担保する。
- **両層横断の `data-*` 機械センサス**が必要なら、それは兄弟イシュー **#1064（headless 63 部品 / pre-styled 107 部品のラップ状態を機械可視化する契約テスト）** の責務である。#1064 は両クレートを走査する機構を必然的に持つため、そちらへ引き継ぐ。

## 4. 適用結果（変更ファイル）

- `crates/pre-styled-ui/src/tab_nav.rs`: `data-current` を `fandhe_frontend_headless_ui::data_attrs::data_current` 経由へ変更（規約 B-1）。出力 HTML は不変（既存テストで固定済み）。
- `crates/pre-styled-ui/src/button.rs` / `tag.rs` / `radio_card.rs` / `charts/radar_chart.rs` / `charts/scatter_chart.rs`: モジュール rustdoc に「`data-*` 語彙」節を追加（規約 B-3）。
- `crates/pre-styled-ui/src/floating_panel.rs` / `json_tree_view.rs` / `calendar.rs`: recipe の `StateCondition` 近傍に出力元（headless-ui）を明示する 1 行コメントを追加（規約 A、イシュー本文の誤認の再発防止）。
- `crates/pre-styled-ui/tests/data_attr_vocabulary.rs`: §2.1 の 5 語彙 6 箇所の出力を固定する契約テストを新設。

## 5. スコープ外（`.claude/rules/out-of-scope-tracking.md`）

1. **両層横断の `data-*` 機械センサス** → **#1064** の責務（同イシューが両クレート走査機構を持つため）。
2. **headless-ui `data_attrs` への `data_value` ヘルパ追加**: `data-value` は headless 5 部品 + pre-styled 1 部品が生タプルで出力しており、headless 側ヘルパ化の余地がある。ただし headless-ui の公開 API 拡張 = semver バンプ + 依存元 3 クレート（pre-styled-ui / wasm-full / xtask）の `version` 追随が必要で、本イシューの範囲を超える。未実施の改善候補として記録し、Issue 化はユーザー承認を得てから提案する。
3. **`data-loading` / `data-series` に CSS 消費者が存在しない件**: 出力されているが recipe の `StateCondition` から参照されていない。意図的な利用者向けフックか、スタイル未実装かの判断は各部品の視覚仕様の問題であり、本イシュー（語彙の帰属）とは別軸。事実として記録するに留める。
4. **`docs/design/component-coverage-map.md` との突合**: pre-styled-only 部品の定義（headless に対応部品が存在しない部品）は同マップが正だが、本イシューでは §2.1 の 5 部品について個別確認したのみで全 107 部品の再突合は行わない（#1064 の範囲）。
