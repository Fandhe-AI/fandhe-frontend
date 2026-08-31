# hover / disabled / transition の共通ビジュアル言語 決定記録

- **イシュー**: [#1425](https://github.com/Fandhe-AI/fandhe-frontend/issues/1425)（親: #1421「フェーズ 0: 部品共通ビジュアル言語の確立」）
- **対象**: `crates/pre-styled-ui/src/theme.rs`・`crates/pre-styled-ui/src/recipe.rs`・`crates/pre-styled-ui/src/button.rs`（参照実装）
- **関連**: `docs/design/pre-styled-ui-data-attr-vocabulary.md`（`data-*` 語彙の決定記録、同型の文書体裁）・`docs/design/pre-styled-ui-scale-tokens.md`（兄弟イシュー #1423、radius/shadow/spacing/z-index トークン）

## 1. 背景

代表 20 部品の実測で、`:hover` / disabled 視覚化 / transition の 3 点が横断的に欠落・不統一であることが判明した。Phase 1 以降の部品別調整（各部品 issue）に先立ち、本イシューで共通規約とヘルパを固定する。

## 2. 現状調査結果（実装時点で再計測）

- **hover 欠落**（`cursor: pointer` を持つが `StateCondition::Hover` 未登録）: 計測は `button.rs` 対応前の時点で 47 モジュール。本 PR で `button.rs` を解消したため、Phase 1 以降の残数は 46
- **disabled 未消費**（disabled を扱うが `Attr("data-disabled")` が recipe に無い）: `tabs.rs` / `select.rs` / `menu.rs` / `color_picker.rs` / `signature_pad.rs` / `steps.rs`（`steps.rs` は `Attr("disabled")` というネイティブ属性語彙を使っており `data-disabled` への統一が別途必要）。`button.rs` は本 PR で解消済み
- **transition 欠落**（インタラクティブだが `transition` 宣言なし）: 計測は `button.rs` 対応前の時点で 36 モジュール。本 PR で `button.rs` を解消したため、Phase 1 以降の残数は 35。既存 18 モジュールはリテラル duration（`0.15s` 等）を個別に手書きしており、本イシューが定めたトークンへの置換対象

いずれも `grep` による機械的再計測であり、Plan フェーズ時点の見積もりと一致した。

## 3. 決定事項

| 論点 | 決定 | 根拠 |
|---|---|---|
| hover 付与の判定基準 | **インタラクティブ slot のみ**（`cursor: pointer` を持つ、または `<button>`/`<a>`/`role=option\|menuitem\|tab` 等を担う slot）。表示専用（badge/alert/card/stat 等）には付けない | 参照 3 サイト（ark-ui/chakra-ui/Radix）の慣行 |
| hover 色の作り方 | 新 `-hover` 段は足さない。solid 系は既存 `<palette>-emphasized`、面が無い系（ghost/outline/subtle/list item）は `bg-muted`。variant 別の色差は `--fandhe-hover-bg` custom property 間接参照で表現し、`Hover` state は `background: var(--fandhe-hover-bg)` の 1 本に集約する | `SlotRecipe` は「variant × state」の複合条件を持たない。`table.rs` の `--fandhe-table-stripe-bg` と同型パターン |
| disabled の統一形 | `opacity: 0.5` + `cursor: not-allowed`。彩度低下・`pointer-events: none` は採らない | 既存 40 箇所超の実装と同一。`cursor` 表示と tooltip 到達性を保つ |
| `:disabled` vs `[data-disabled]` | `[data-disabled]` を正とする | `<li>`/`<a>`/`<div>` ベースの item/trigger にも同じ 1 経路で適用できる。`:disabled` はネイティブフォーム要素にしか効かない |
| button の `data-disabled` 非消費 | 改める（`recipe_with_scope` の `root` に `[data-disabled]` 規則を追加）。`download_trigger` にも同時に波及するが disabled を持たない設計のため dead CSS のみ | 参照サイト標準に合わせる |
| transition 既定 | duration: `fast=150ms` / `normal=200ms` / `slow=300ms`、easing: `standard=cubic-bezier(0.4, 0, 0.2, 1)` / `emphasized=cubic-bezier(0.2, 0, 0, 1)`。既存リテラル `0.15s` は `fast`、`0.2s ease` は `normal` へ写像する | chakra-ui / Radix の既定値帯 |
| transition の宣言分解 | shorthand `transition:` ではなく longhand 3 プロパティ（`transition-property`/`transition-duration`/`transition-timing-function`）へ分解する | `Declaration::value` が `&'static str` のみを受け付ける制約下では、複数プロパティへ同一 duration を shorthand で割り当てるための実行時文字列連結を回避できない（`crate::css` モジュール冒頭の「`decl()` はソースコード中のリテラルからのみ構築される」不変条件を保つため） |
| reduced-motion の共通化 | `Theme::to_css` が motion トークンのうち `duration-*` のみを `@media (prefers-reduced-motion: reduce) { :root { --fandhe-motion-duration-*: 0ms; } }` で一括上書きする。部品側は `var(--fandhe-motion-duration-*)` を参照するだけで自動的に無効化される | 各 recipe への `@media` 追加を不要にする唯一の共通経路 |
| タッチ端末での hover 貼り付き | `StateCondition::Hover` を `@media (hover: hover)` で囲んで出力し、セレクタも `:hover:not([data-disabled])` とする | 消費者は `charts::tooltip` の 1 件のみで golden 更新が容易。「消費者が現れた時点で契約を固める」前例（#847）に従う |

## 4. 実装

### 4.1 `theme.rs`

- `Theme` に `motions: Vec<ScaleToken>` を追加。`DEFAULT_MOTIONS`（duration 3 段 + easing 2 種）を `Default` で登録する
- `push_motion` / `upsert_motion`（既存 `push_scale`/`upsert_scale` を再利用）、`motion_var(name) -> var(--fandhe-motion-<name>)` を追加
- `to_css` は `:root` ブロック末尾（z-indices の後）に `--fandhe-motion-<name>` を出力し、`:root[data-theme="dark"]` ブロックの後に `write_reduced_motion_block` が `@media (prefers-reduced-motion: reduce)` ブロックを追記する（`duration-` で始まる motion が 1 件以上あるときのみ）
- motion を一切 push しないテーマの `to_css()` 出力は本イシュー導入前とバイト同一（既存 radii/shadows/z-indices の純追加パターンを踏襲）

### 4.2 `recipe.rs`

- `StateCondition::Hover` の出力先を states ループの通常出力先から分離し、css() 末尾で `@media (hover: hover) { ... }` に 1 つだけまとめて出力する。セレクタは `:hover:not([data-disabled])`
- 共通ヘルパを追加: `disabled_declarations()` / `hover_surface_declarations()` / `hover_bg_solid()` / `hover_bg_muted()` / `MotionDuration`（`Fast`/`Normal`/`Slow`）/ `transition_declarations(properties, duration)`
- `transition_declarations` は `Declaration::value` の `&'static str` 制約により shorthand ではなく longhand 3 プロパティを返す設計とした（§3 参照）。呼び出し側は `properties` にカンマ区切りのプロパティ名リテラルを渡す

### 4.3 `button.rs`（参照実装）

- `recipe_with_scope` の `root` base へ `transition_declarations("background, border-color, color, box-shadow", MotionDuration::Fast)` を追加
- `Solid` variant に `hover_bg_solid()`、`Outline`/`Ghost` に `hover_bg_muted()` を追加（本節が定義する初期実装。以下はイシュー #1448 による更新）
- `root` へ `.state(_, StateCondition::Hover, hover_surface_declarations())` と `.state(_, StateCondition::Attr("data-disabled"), disabled_declarations())` を追加
- **イシュー #1448 での更新（`Surface`/`Plain` 追加・`Subtle` の hover 再設計を含む 6 variant 化）**:
  - `Subtle`/`Surface`（chakra-ui v3 準拠の `palette_scale_declarations` による tint 着色面）は `hover_bg_muted()` ではなく `decl("--fandhe-hover-bg", "var(--fandhe-palette-muted)")` を直接指定する。tint 面に中立色 `bg-muted` を当てると tint → gray の不自然な遷移になるため、`hover_bg_muted()` 自体は使わず、#1425 の趣旨（既存段の再利用・新段を作らない）に沿って `--fandhe-palette-muted` を採用した意図的差分である。`Outline`/`Ghost` の hover は #1425 の参照実装（`hover_bg_muted()`）のまま変更しない
  - `Plain`（背景・輪郭なしの最小装飾）は hover 背景変化を持たない chakra-ui v3 の `plain` に合わせ、`--fandhe-hover-bg` を `transparent` として明示定義する（未定義のまま共有 Hover state に任せると `background: var(--fandhe-hover-bg)` が computed-value time に無効化される非決定的挙動になるため、明示定義で回避する fail-closed 設計）

### 4.4 `charts::tooltip`

コード変更なし。`StateCondition::Hover` の出力形式変更（`@media` 包み + `:not([data-disabled])`）に追随する rustdoc 更新のみ。

## 5. Phase 1 以降の適用手順（チェックリスト）

各部品 issue は以下を確認する:

1. **インタラクティブ slot 判定**: `cursor: pointer` を持つ slot、または `<button>`/`<a>`/`role=option|menuitem|tab` 等を担う slot にのみ hover を適用する
2. **hover**: 各 variant で `--fandhe-hover-bg` を定義し、対象 slot へ `.state(_, StateCondition::Hover, hover_surface_declarations())` を 1 本登録する。定義方法は原則 `hover_bg_solid()`（solid 系）/ `hover_bg_muted()`（`Outline`/`Ghost` 等の面なし系）のいずれかだが、tint 着色面（`palette_scale_declarations` を使う variant。`Subtle`/`Surface` 参照）のように中立色 bg-muted を当てると tint → gray の不自然な遷移になる場合は `decl("--fandhe-hover-bg", "var(--fandhe-palette-muted)")` を直接指定してよい（イシュー #1448）。hover 背景変化を持たない variant（`Plain` 参照）は `transparent` を明示定義する（未定義のまま共有 Hover state に任せると computed-value time に無効化される非決定的挙動になるため）
3. **disabled**: `Attr("data-disabled")` + `disabled_declarations()` を登録する。`:disabled` ネイティブ擬似クラスや独自の `opacity`/`cursor` 値は使わない。`steps.rs` は `Attr("disabled")` → `Attr("data-disabled")` への語彙統一も行う
4. **transition**: 既存リテラル duration をトークン参照へ置換する。写像表: `0.15s` → `MotionDuration::Fast`、`0.2s`（`ease` 有無問わず）→ `MotionDuration::Normal`、`0.3s` → `MotionDuration::Slow`。未導入モジュールは `transition_declarations` を base へ追加する

対象一覧（実装時点の再計測、§2 参照）は本イシューのコメントに投稿する。

## 6. 意図的に採らなかった案

- **`-hover` 色トークンの新設**: `--fandhe-hover-bg` 間接参照で足りるため、テーマ層への新規段追加は行わない（#1422 の色トークン改称と疎結合を保つ）
- **`:disabled` ネイティブ擬似クラスへの統一**: `<li>`/`<a>`/`<div>` ベースの item/trigger に適用できないため不採用
- **`pointer-events: none` による disabled 表現**: `cursor` 表示・tooltip 到達性を損なうため不採用
- **per-recipe `@media (prefers-reduced-motion)` の個別手書き**: `Theme::to_css` の一括処理に統一する。ただし skeleton/marquee の既存個別 `@media`（`animation: none` 等、duration では表現できない副作用を持つ）は本イシューでは変更せず維持する
- **transition の shorthand 1 プロパティ化**: `Declaration::value` の `&'static str` 制約（`crate::css` の「`decl()` はソースコード中のリテラルからのみ構築される」不変条件）を保つため、longhand 3 プロパティに分解した

## 7. 兄弟イシューとの境界

- **#1422（色トークン）**: `-emphasized`/`bg-muted` の具体的なトークン名は #1422 側で改称され得る。本イシューはトークンの「役割」（solid 系 → emphasized、面なし系 → muted）でのみ規定し、名前の詳細を 1 箇所（`hover_bg_solid`/`hover_bg_muted`）に閉じ込めている
- **#1423（radius/shadow/spacing/z-index）**: `theme.rs` へ独立フィールド（`motions`）・独立 const（`DEFAULT_MOTIONS`）・独立関数（`push_motion`/`upsert_motion`/`motion_var`）として追加し、既存 const・既存関数本文を書き換えていないため、並列マージのコンフリクト面は最小化されている
- **#1424（フォーカスリング・size）**: 本イシューはフォーカスリング・size 軸には触れない

## 8. 再評価トリガー

- transition の複合プロパティ（例: `transform`）を扱う部品が増え、longhand 分解では表現しきれないケースが出てきた場合、`Declaration` の型（`&'static str` 制約）自体の見直しを再検討する
- easing の種類（`emphasized` 等）を実際に使う消費者が現れた場合、`transition_declarations` の easing 引数化を検討する（現状は `easing-standard` に固定する単純化）
