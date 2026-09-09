# slider の複数 thumb（range slider）対応の評価と設計案

## 1. 背景・目的

- Themes 層の主基準 3 者（chakra-ui / Radix Themes / shadcn-ui、
  `docs/design/shadcn-reference-adoption-policy.md` §8）はいずれも複数
  thumb の range slider を標準機能として持つ。shadcn/ui の参照スクショ
  （`docs/design/reference-screenshots/shadcn-slider-1.png` /
  `-2.png`）も両方とも range 表現である。
- 本リポジトリの headless-ui `slider`（`crates/headless-ui/src/slider.rs`、
  #741 で実装、#1621 で参照突合）は thumb 1 個固定の anatomy であり、
  `data-index` / 複数値の状態機械 / 複数 hidden input といった機構を
  持たない。pre-styled-ui `slider` は `--fandhe-slider-percent` の 1 点
  伝搬設計のため Themes 層単独では range を表現できない（PR #2167 の
  対象外節の指摘）。
- 現状の記録には整合上の問題があった。`docs/design/component-coverage-map.md`
  の slider 行（Part A / Part B）は「意図的非採用のまま据え置き」と
  記していたが、`docs/policy/intentional-non-adoption.md` に slider の
  非採用節は存在せず、#741 本文は「range 表現は将来拡張」（＝スコープ
  先送り）と述べているだけであった。`crates/headless-ui/src/slider.rs`
  の rustdoc（「参照突合」節・「スコープ外」節の 2 箇所）も「#741 以来の
  スコープを維持」としか書いていなかった。つまり **range slider は
  一度も評価されておらず、非採用が確定した事実もない**。
- 本イシュー（#2188）の目的は、(a) 複数 thumb の anatomy 案・キーボード
  操作・wasm-full の値更新経路を設計文書として固定し、(b) #741 の見送り
  根拠との整合を明記し、(c) 採否のユーザー判断を「そのまま実行できる」
  状態にすることである。

本 PR は評価文書の新設と、上記の整合性是正（rustdoc / coverage-map /
intentional-non-adoption.md の文言訂正）のみを成果物とし、
`crates/headless-ui/src/slider.rs` のコード・公開 API・テストは一切
変更しない。

## 2. 一次ソース突合

- **ark-ui / zag.js `slider` machine**（headless 層の一次参照軸、
  `docs/design/shadcn-reference-adoption-policy.md` §8 改訂 4 で不変）:
  `value: number[]`、`Thumb` は `index` を取り `data-index` を出力する。
  `aria-valuemin` / `aria-valuemax` は隣接 thumb の値を基準にした
  per-index 範囲（前隣 thumb の値 + `step * minStepsBetweenThumbs` を
  下限、次隣 thumb の値 − 同ギャップを上限、端は `min` / `max`）を出力し、
  `aria-valuenow` は `value[index]`、`aria-valuetext` は index ごとに
  導出する。`aria-label` / `aria-labelledby` は配列で渡されたとき
  `[index]` を採る。`data-focus` / `data-dragging` は
  `focusedIndex === index` のときのみ付与される。hidden input は
  thumb ごとに 1 個で、複数値のとき `name` へ `[]` を付加する実装例が
  多い。キーボード操作は ArrowUp/Right → increment、ArrowDown/Left →
  decrement、Home / End、PageUp/PageDown、Shift+Arrow で largeStep。
  thumb 衝突時の挙動（`thumbCollisionBehavior`）は `none`（隣接境界で
  clamp）/ `push` / `swap` の 3 種類が選べる実装が一般的である。
- **Radix Primitives Slider**（`docs/design/radix-primitives-inventory.md`
  行 89）: Root > Track > Range > Thumb、`value: number[]`、
  `minStepsBetweenThumbs` を持つ。shadcn/ui Slider は Radix Slider の
  薄いラッパーで、`defaultValue={[25, 75]}` のような複数値が range の
  典型例として示されている。
- **WAI-ARIA APG「Multi-Thumb Slider」パターン**: 各 thumb が独立した
  `role="slider"` を持ち、隣接 thumb の値を自身の `aria-valuemin` /
  `aria-valuemax` として反映する（zag の per-index 範囲と同方式）。
- **chakra-ui Slider**: ark-ui の再エクスポートであり同一機構を持つ。

採用時の実装 issue では、上記の要約ではなく zag.js
`packages/machines/slider/src/slider.connect.ts` /
`slider.utils.ts` の該当行、および APG 該当ページの記述を直接引用し、
行番号付きで参照突合表へ記録すること。

## 3. 現状の構造的制約

- `Slider { min, max, step, value: f64, orientation }` は単一値の状態
  機械である。`thumb()` は index を取らず、`hidden_input()` も 1 個
  しか出力しない。hydration 属性は `data-hydrate-{min,max,step,value,
  orientation}` の 5 件。dispatch は `"set"`（payload = 値）/
  `"increment"` / `"decrement"` / `"increment_large"` /
  `"decrement_large"` / `"home"` / `"end"`（payload 不使用）で、対象
  thumb を指定する経路が存在しない。
- `Slider::percent()` は 1 値のみを返す。pre-styled-ui は
  `--fandhe-slider-percent` の 1 点伝搬（range 表現は 0..percent）
  で設計されており、複数 thumb を表現できない。
- `fandhe-frontend-wasm-full` には linear slider の DOM 配線が
  **存在しない**（`crates/wasm-full/src/events.rs` の
  `ARIA_INTERACTIVE_ROLES` に `"slider"` ロール名が並ぶのみ。
  pointer / keydown 配線は #1621 で REQ-11 逼迫を理由に見送り済み。
  `angle_slider.rs` の配線とは別物）。したがって現状の値更新経路は
  「アプリ側が `fandhe_frontend_interactive::dispatch` を呼び、
  `DirtyTracked::dirty_fields` → 束縛点更新」のみである。

## 4. #741 の見送り根拠との整合

- #741 は「単一値スライダー（… range 表現は将来拡張）」と定義しており、
  複数 thumb を評価して落としたのではなく初期スコープから外しただけで
  ある。`docs/policy/intentional-non-adoption.md` に slider の非採用節
  は存在しない（§3.22 の angle-slider とは別件）。よって
  `component-coverage-map.md` の「意図的非採用」表記は誤りであり、
  本 PR で「スコープ先送り・未評価 → #2188 で評価済み」へ訂正した。
- `.claude/rules/coding-rust.md` の「意図的非採用機能の再導入提案には
  評価軸の充足確認が必須」は本件には適用されない（非採用確定項目では
  ないため §4 の再導入手続きは不要。
  `docs/policy/intentional-non-adoption.md` §7 の「保留 → 通常の
  feature issue」経路に乗る）。

## 5. anatomy・状態機械の設計案

2 案を比較し、案 B を推奨する（確定ではなくユーザー判断の材料）。

### 案 A: `Slider` を `values: Vec<f64>` へ一般化（破壊的変更）

- `value()` → `values()`、`thumb()` / `hidden_input()` へ `index`
  引数追加、hydration `data-hydrate-value` → `-values`（カンマ区切り）。
- 長所: ark-ui と同型の単一 API。
- 短所: pre-styled-ui `slider.rs` / docs-site `showcase.rs` /
  `primitive_showcase/forms_b.rs` / `primitive_specs/forms_b.rs` /
  `component_specs/forms.rs` / examples の全呼び出し側が壊れ、
  headless-ui minor バンプ → pre-styled-ui / wasm-full / xtask の
  追随・golden 再生成・`examples/headless-pre-styled-ui` の crates.io
  追随（#2219 と競合）が連鎖する。
  `docs/design/shadcn-reference-adoption-policy.md` §8 改訂 3 の
  「純追加原則」に反する。

### 案 B（推奨）: 同一モジュール内に `RangeSlider` を純追加

- `crates/headless-ui/src/slider.rs` に `RangeSlider { min, max, step,
  values: Vec<f64>, min_steps_between_thumbs: u32, orientation }` を
  追加し、既存 `Slider` と `normalize` / `snap_to_step_and_clamp` /
  `fmt_num` / `drop_reserved` を共有する。`values.len() >= 2` を要求
  し（1 値は `Slider` を使う）、上限 `MAX_THUMBS`（例: 16）を定数で
  固定して hydration の巨大リストを fail-closed に拒否する
  （`crates/headless-ui/src/splitter.rs` の `DEFAULT_PANEL_COUNT`
  下限検査と同型）。
- 不変条件: `values` は昇順ソート済み・各値は `min` 起点 step 整列
  済み・隣接差は `step * min_steps_between_thumbs` 以上
  （`normalize_values` 純粋関数で fail-closed に正規化。実現不能な
  組合せ〔例: `min_steps_between_thumbs` が大きすぎる〕は既定値へ
  フォールバックし panic しない）。
- パーツ: Root / Label / Control / Track / Range / MarkerGroup /
  Marker / ValueText は既存自由関数を再利用する（`RangeSlider` の
  利便メソッドから委譲）。Thumb は新規自由関数 `thumb_at(orientation,
  index, min, max, now, aria_valuetext, props, attrs, children)` で
  `data-index="<index>"` を追加出力する（`splitter.rs` の
  `PANEL_RESERVED` / `pagination.rs` と同じく `data-index` を
  RESERVED に登録し呼び出し側 `attrs` の偽装を除去する）。
  `aria-valuemin` / `aria-valuemax` は zag / APG と同じ per-index
  範囲（隣接 thumb ± ギャップ、端は min/max）を
  `RangeSlider::bounds_at(index) -> (f64, f64)` で算出して渡す。
  `aria-valuetext` は `Option<&str>` を index ごとに呼び出し側が渡す
  （整形は `docs/policy/intentional-non-adoption.md` §3.23 どおり
  利用者責務）。HiddenInput は `RangeSlider::hidden_input_at(index,
  name, disabled, attrs)` で thumb ごとに出力し、`name` は呼び出し側
  が渡した文字列をそのまま使う（zag の `[]` 自動付加はフォーム規約＝
  アプリ側関心のため headless では行わず rustdoc で案内する）。
  `data-focus` / `data-dragging` は既存 `Slider` と同じ理由（配線
  スコープ外）で出力しない。
- 状態機械: `RangeSliderAction::{SetValue { index, value },
  Increment(index), Decrement(index), IncrementLarge(index),
  DecrementLarge(index), SetToMin(index), SetToMax(index)}`。
  `update()` は対象 index の値を snap → 隣接境界（`bounds_at`）へ
  clamp する（= zag `thumbCollisionBehavior: "none"` 相当。`push` /
  `swap` は再評価トリガーへ送り初期実装では採らない: 決定性軸で
  「1 操作が 1 thumb のみを変える」ことが最も検証しやすいため）。
  範囲外 index は no-op。`DirtyTracked` を実装する場合のフィールド名
  は `"values"` 1 件（`&'static str` 固定）とする。
- dispatch 契約: `"set"` payload = `"<index>:<value>"`（`color_picker`
  の `"set_channel"` payload `"<channel>:<value>"` と同型）、
  `"increment"` / `"decrement"` / `"increment_large"` /
  `"decrement_large"` / `"home"` / `"end"` payload = `"<index>"`。
  index は `usize` 厳密パース + `< values.len()` 検査、値は `f64`
  厳密パース + 有限性検査、いずれも失敗は `None`（fail-closed
  no-op）。
- hydration 契約: `data-hydrate-{min,max,step,values,min-steps,
  orientation}`。`values` はカンマ区切り（`splitter.rs::
  from_hydration_attrs` の `parse_list` と同型: 各要素の有限性・件数
  下限 2・上限 `MAX_THUMBS`・昇順・`[min, max]` 内を検証し、受理後も
  `normalize_values` を再適用する多層防御）。
- キーボード操作: 対象 thumb は「フォーカス中の thumb」＝ `data-index`
  を DOM から読む配線側の責務。headless 側は `action_for_key(key,
  modifiers, index) -> Option<RangeSliderAction>` 純粋関数
  （`image_cropper.rs::action_for_key` と同型）で「ArrowUp/Right →
  Increment、ArrowDown/Left → Decrement、Shift+Arrow / PageUp /
  PageDown → *Large、Home / End」の対応表を固定する（DOM keydown 配線
  自体は wasm-full の後続責務のまま）。
- wasm-full の値更新経路: 本設計では配線を追加しない（linear slider
  は単一値版も未配線であり、#1621 と同じ REQ-11 理由）。値更新は
  `dispatch` → `update` → `dirty_fields` → 束縛点更新
  （`aria-valuenow` / `aria-valuemin` / `aria-valuemax` / hidden
  input `value` / `--fandhe-slider-range-start|end`）で、headless
  側が `bounds_at` を公開していれば配線層に独自計算は不要、という
  契約とする。wasm-full が `RangeSlider` をリンクしない限りバンドル
  サイズ（REQ-11）への影響はゼロである。
- pre-styled-ui 側の設計（採用時の別 sub-issue）:
  `--fandhe-slider-range-start` / `--fandhe-slider-range-end`
  （zag の `--slider-range-start/end` に相当。#1621 で「装飾用 CSS
  変数は headless へ持ち込まず pre-styled が担う」と決めた
  `intentional-non-adoption.md` §3.25 規則 2 の適用）を
  `RangeSlider::percent_at(index)` から styled `range_multi()` が
  組み立て、thumb は `thumb_styled_at(index)` で
  `--fandhe-slider-percent` を index ごとに付与する。golden は純追加
  （既存 `slider_css.rs` の出力バイト同一）。

## 6. `docs/policy/intentional-non-adoption.md` §2 の 4 軸評価と §3.25 照合

- **明示性**: `values` と `data-index` が状態・DOM の双方に明示され、
  per-index 境界は `bounds_at` の純粋関数で読める。
- **決定性**: snap / clamp / 隣接境界がすべて純粋関数、collision は
  `none` 固定で 1 操作 1 thumb。
- **機械検証可能性**: 単体テスト（正規化の冪等性・境界到達・index
  範囲外 no-op・hydration fail-closed・XSS）で固定できる。
- **コンテキスト消費**: 案 B は既存 `Slider` を変えず追加分のみ読めば
  よい。案 A は全呼び出し側の再読が必要。
- **§3.25 規則 1**: バリデーション・整形・永続化を含まない
  （`aria-valuetext` の整形・hidden input の `name` 規約は利用者
  責務）。
- **§3.25 規則 2**: 位置計算 CSS 変数・drag 状態は pre-styled /
  wasm-full 側に置く。

両規則に抵触しない。

## 7. 採否判定・再評価トリガー・実装分割案

- **推奨: 条件付き採用（案 B）**。条件は (1) 純追加で既存 `Slider`
  API と golden 出力を不変に保つこと、(2) wasm-full 配線・`push` /
  `swap` collision は初期スコープ外とすること、(3) headless-ui は
  minor バンプ（純追加でも 0.x では API 追加を minor とする本
  リポジトリ慣例）とし pre-styled-ui が追随すること。**最終判断は
  ユーザーが行う**（本文書は推奨のみを記し、確定はしない）。
- **再評価トリガー**（非採用となった場合に再提案できる条件）:
  利用要望 issue の起票 / 参照 3 者（chakra-ui / Radix Themes /
  shadcn-ui）のいずれかが range を既定 UI から外す / wasm-full の
  REQ-11 予算が配線追加を許容する水準へ回復すること。
- **採用時の sub-issue 分割案**:
  1. headless-ui `RangeSlider`（anatomy・状態機械・hydration・
     `action_for_key`・テスト・`docs/api/headless-ui-api.md`）
  2. pre-styled-ui `range_multi` / `thumb_styled_at` と CSS 変数・
     golden 純追加・`data_attr_vocabulary.rs` /
     `xss_escape_styled.rs` 登録
  3. docs-site showcase / primitives ページの range デモと
     coverage-map 更新（#2216 の vertical marker デモと同一ファイル
     を触るため後着とする）
  4. 任意: wasm-full keydown / pointer 配線（REQ-11 実測付き）
- **非採用時の記録草案**: `docs/policy/intentional-non-adoption.md`
  §3.27 として本文書 §6 の評価と本節の再評価トリガーを転記する。

## 8. 初期設計スコープ外（再評価トリガーへ記録）

- `thumbCollisionBehavior` の `push` / `swap`
- per-thumb `aria-label` 配列
- `data-focus` / `data-dragging`
- wasm-full の pointer ドラッグ・DOM keydown 配線

## 9. セキュリティ考慮（OWASP Top 10 観点）

- **A03 インジェクション / XSS**: 本設計案では `data-index` /
  `aria-valuemin` / `aria-valuemax` / `aria-valuenow` / hidden input
  `value` はすべてサーバー側で正規化した `usize` / `f64` の
  `fmt_num` 文字列のみをスロットへ通し、属性名は `&'static str`
  固定とする。動的値は `fandhe_frontend_core::render` の既定
  エスケープを経由し（REQ-1 を弱めない）、`raw_html()` は使用しない。
- **A04 安全でない設計**: dispatch payload（`"<index>:<value>"`）と
  hydration 属性（カンマ区切り `values`）はクライアント改ざん入力
  として厳密パース + 有限性 + 件数上限 `MAX_THUMBS` + 昇順・範囲検証
  で fail-closed とする（不正は no-op / `HydrateError`、panic しない）。
  範囲外 index は no-op。
- **A05 設定ミス**: 採用時の実装 PR で headless-ui の semver バンプと
  依存元（pre-styled-ui / wasm-full / xtask）の `version = "..."`
  追随を `xtask check-dep-versions` で確認する。
- **A08 データ整合性**: `#![forbid(unsafe_code)]` を維持する
  （headless-ui / pre-styled-ui / interactive）。外部依存の追加なし
  （REQ-3 に影響なし）。
- **秘密情報**: 本 PR の変更ファイルは docs / rustdoc のみで、認証
  情報・環境変数を含まない。

## 10. 参照

- `docs/design/shadcn-reference-adoption-policy.md`
- `docs/design/component-coverage-map.md`（§5 Part A `form/slider.md`
  行、Part B `forms/slider.md` 行）
- `docs/design/radix-primitives-inventory.md`（Slider 行）
- `docs/policy/intentional-non-adoption.md`（§7、§3.25）
- `crates/headless-ui/src/slider.rs`（現行 `Slider` 実装）
- `crates/headless-ui/src/splitter.rs`（`PANEL_RESERVED` /
  `parse_list` の参考実装）
- `crates/headless-ui/src/color_picker.rs`（`"<channel>:<value>"`
  payload の参考実装）
- `crates/headless-ui/src/image_cropper.rs`（`action_for_key` の
  参考実装）
