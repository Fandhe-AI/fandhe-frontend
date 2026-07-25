# ark-ui / chakra-ui 全コンポーネント対応表

**本文書のステータス**: 確定（イシュー #734、親 #733/#726）。正はコミット
`ab53c6b` 時点の `.agents/skills/ark-ui|chakra-ui/references/` 一覧
（ark-ui 90 件 / chakra-ui 269 件 / 計 359 件）。以後この文書を
Phase 3〜6（#736/#748/#757/#766 配下、#737〜#747・#749〜#756・#758〜#765・
#767〜#776 の 37 issue 相当）実装の正とする。保留・意図的非採用の評価軸・
再評価トリガーはイシュー #735 で `docs/policy/intentional-non-adoption.md`
§3.22〜§3.24（新規非採用確定）・§7（保留項目の記録）に確定記録済み。
§3 の実装済み mod 件数・§5 の区分は、2026-07-25 時点でイシュー #934 が
全件再実測し、それ以前の PR ごとの差分追記による蓄積ドリフト（headless-ui
38/pre-styled-ui 46 という過小計上）を是正済み。

## 1. 背景

親ツリー #726（Phase 2 親: #733）は、ark-ui / chakra-ui の全コンポーネント
網羅を「コミット `ab53c6b` の `.agents/skills/ark-ui|chakra-ui/references/`
一覧」を正として進める。本書新設以前は、実装済み・実装対象・保留・意図的
非採用の別を機械確認可能な形で一覧化した恒久文書がなく、Phase 3〜6 の実装
が参照すべき正が issue 本文（#726 のサマリー）にしか存在しなかった。

本書は references 配下 md **全件**（README・guides 等の非コンポーネント
文書を含む）を 1 行ずつ列挙した対応表であり、以後の Phase 3〜6 実装・
将来の追加検討はこの文書を参照する。

## 2. 区分の定義

| 区分 | 意味 |
|------|------|
| 実装済み | `fandhe-frontend-headless-ui` / `fandhe-frontend-pre-styled-ui` に mod として実装済み |
| 実装対象 | Phase 3〜6（#736/#748/#757/#766 配下）のいずれかの issue で実装予定。根拠・対応 issue 列に issue 番号を記載 |
| 保留 | 実装するか否かを本書時点では確定しない。根拠・再評価トリガーの詳細は `docs/policy/intentional-non-adoption.md` §7（イシュー #735）に記録済み |
| 意図的非採用 | 既に非採用と確定済み（layout プリミティブ = #716/#724、高度入力系・JS ランタイム固有 utilities・装飾系の一部・chakra `Theme` = #735（同書 §3.22〜§3.24）で確定済み等）。再導入提案には `docs/policy/intentional-non-adoption.md` の評価軸充足確認が必須 |
| 対象外 | README・guides・overview・get-started・concepts 等、UI コンポーネントを指さない非コンポーネント文書。加えてイシュー #735 で商用テンプレート集（chakra-ui Pro blocks）・styling / theming 概念文書を本区分へ追加確定した |

「保留」区分の評価軸・再評価トリガーの詳細記録はイシュー #735 で
`docs/policy/intentional-non-adoption.md` §7 に確定済みであり、本書の
「根拠・対応 issue」列からは同節への参照を行う。

## 3. 実装済み部品と lib.rs の突合手順

本節の数値は下記コマンドの出力そのものであり、PR ごとの差分追記による
更新を禁止する（イシュー #934。差分追記の蓄積が headless-ui 38 /
pre-styled-ui 46 という大幅ドリフトを生んだ）。数値を更新する場合は
必ず下記コマンドを再実行し、その出力をそのまま転記すること。

**基準日**: 2026-07-25 時点の実測（イシュー #934 で全件再実測）。

**除外集合の定義と根拠**: 除外基準は「ノード（`fandhe_frontend_core::Node`）
を返さない・anatomy を持たない非描画の基盤 mod」。

- headless-ui: `anatomy` / `aria` / `data_attrs` / `positioning` / `state` /
  `format` / `color` / `date`（`date` は `crates/headless-ui/src/date.rs`
  の module doc が「HTML を一切組み立てない」と明記する非描画の純計算
  モジュールであり、`format` / `color` と同一カテゴリのため同基準で除外）
- pre-styled-ui: `css` / `recipe` / `stylesheet` / `theme`
- 注記: `qr_encode`（headless-ui）・`class_attr`（pre-styled-ui）は
  `mod`（非 `pub`）宣言のため `^pub mod` 突合の対象外であり、除外パターンへ
  書く必要がない
- 注記: `charts` は名前空間 mod だが §5 に `charts/installation.md` /
  `charts/use-chart.md` の 2 行が割り当て済みのため計上に含める（この方針の
  恒久化は §7 参照）

**突合コマンド**（`^pub mod (...);$` の完全一致でアンカーする。部分一致の
`grep -vE 'css|recipe|stylesheet|theme'` は `color` / `date` を alternation
へ足した瞬間に `color_picker` / `date_input` / `date_picker` を巻き込んで
消してしまうため使わない）:

```bash
grep -E '^pub mod ' crates/headless-ui/src/lib.rs \
  | grep -vE '^pub mod (anatomy|aria|data_attrs|positioning|state|format|color|date);$' | wc -l
# => 59
grep -E '^pub mod ' crates/pre-styled-ui/src/lib.rs \
  | grep -vE '^pub mod (css|recipe|stylesheet|theme);$' | wc -l
# => 94
grep -cE '^pub mod ' crates/pre-styled-ui/src/charts/mod.rs
# => 13
```

- headless-ui **59**:
  accordion / action_bar / angle_slider / avatar / breadcrumb / calendar /
  carousel / checkbox / clipboard / collapsible / color_picker / combobox /
  date_input / date_picker / dialog / download_trigger / drawer / editable /
  field / fieldset / file_upload / floating_panel / hover_card /
  image_cropper / json_tree_view / link / link_overlay / listbox / menu /
  nav_list / number_input / pagination / password_input / pin_input /
  popover / progress / qr_code / radio_group / rating_group / scroll_area /
  segment_group / select / signature_pad / skip_nav / slider / splitter /
  steps / switch / tabs / tags_input / timer / toast / toggle /
  toggle_group / toggle_tip / tooltip / tour / tree_view / visually_hidden
- pre-styled-ui **94**:
  accordion / action_bar / alert / angle_slider / area_chart / avatar /
  badge / blockquote / breadcrumb / button / calendar / card / carousel /
  charts / checkbox / checkbox_card / clipboard / code / color_picker /
  color_swatch / combobox / data_list / date_input / date_picker / dialog /
  donut_chart / download_trigger / drawer / editable / em / empty_state /
  file_upload / floating_panel / heading / highlight / hover_card / icon /
  image / image_cropper / input / json_tree_view / kbd / line_chart / link /
  link_overlay / list / listbox / mark / marquee / menu / native_select /
  nav_list / number_input / pagination / password_input / pie_chart /
  pin_input / popover / progress / qr_code / radio_card / radio_group /
  rating_group / scroll_area / segment_group / select / separator /
  signature_pad / skeleton / skip_nav / slider / sparkline / spinner /
  splitter / stat / status / steps / switch / table / tabs / tag /
  tags_input / text / textarea / timeline / timer / toast / toggle /
  toggle_group / toggle_tip / tooltip / tour / tree_view / visually_hidden
- `charts/` 配下 **13**:
  axis / bar_chart / bar_list / bar_segment / data / grid / legend / pie /
  radar_chart / scale / scatter_chart / svg / tooltip

## 4. 抜けの機械確認手順

references 側の md 全件が本書に列挙されていることは、以下の diff が空
であることで確認する（完全性の判定条件。件数は 359 件）。

```bash
diff <(grep -oE '\.agents/skills/(ark-ui|chakra-ui)/references/[A-Za-z0-9/._-]+\.md' \
         docs/design/component-coverage-map.md | sort -u) \
     <(find .agents/skills/ark-ui/references .agents/skills/chakra-ui/references \
         -name '*.md' | sort)
```

references 側が将来更新された場合（`.agents/skills/ark-ui` /
`.agents/skills/chakra-ui` の再取得によるコンポーネント追加・削除）は、
本表の改訂 issue を起票して追随する。本書は将来更新を CI で自動検知しない
（§7 参照）。

### 4.2 mod 側の計上漏れ確認手順（イシュー #934）

§3 で実測した全 mod が §5 の表（Part A / Part B / Part C）の fandhe 列に
行として存在することは、以下の出力が空であることで確認する（§4 が
references → 本書の方向の完全性を検査するのに対し、本手順は
lib.rs → 本書の逆方向を検査する）。

```bash
tmp=$(mktemp -d)
awk '/^## 5\./,/^## 6\./' docs/design/component-coverage-map.md \
  | awk -F'|' 'NF>=8 { print $5; print $6 }' \
  | tr -c 'a-z0-9_' '\n' | sort -u > "$tmp/cols"
{ grep -E '^pub mod ' crates/headless-ui/src/lib.rs \
    | grep -vE '^pub mod (anatomy|aria|data_attrs|positioning|state|format|color|date);$'
  grep -E '^pub mod ' crates/pre-styled-ui/src/lib.rs \
    | grep -vE '^pub mod (css|recipe|stylesheet|theme);$'
  grep -E '^pub mod ' crates/pre-styled-ui/src/charts/mod.rs
} | sed 's/^pub mod //; s/;$//' | sort -u | comm -23 - "$tmp/cols"
rm -rf "$tmp"
```

- `NF>=8` は先頭・末尾の空フィールドを含む 7 列テーブル行を選ぶ条件。
  Part C（§5 末尾）も同じ 7 列構成にすることで、この 1 本のコマンドで
  Part A/B/C を一括検査できる
- `tr -c 'a-z0-9_'` により `` `charts::axis` `` のような表記は `charts` と
  `axis` に分解され、両方が計上済みとして拾われる
- 未検証・未信頼の値を一切補間しない: `$tmp` は `mktemp -d` の結果のみを
  参照しダブルクォートで囲む。`eval`・バッククォート実行・ネットワーク
  アクセスは含まない。`rm -rf "$tmp"` は `mktemp -d` が返す一時ディレクトリ
  に限定され、リポジトリ内パスを削除しうる形にはしない

## 5. 表本体

参照ファイルパス単位で 1 md = 1 行。Part A（ark-ui）/ Part B（chakra-ui）
をディレクトリ節ごとに分割して掲載する。

- **参照ファイル**: `.agents/skills/.../xxx.md` のフルパス（機械突合のキー）
- **ark-ui 名 / chakra-ui 名**: 対応する相手側コンポーネント名。片側のみ・
  相手側に対応がない場合は `—`
- **fandhe headless-ui / fandhe pre-styled-ui**: 対応する mod 名。未実装は `—`
- **区分**: §2 の 5 区分
- **根拠・対応 issue**: 実装対象は issue 番号、保留・意図的非採用は根拠概要
  （詳細はイシュー #735 で確定した `docs/policy/intentional-non-adoption.md`
  §3.22〜§3.24・§7、または既存記録 #716/#724）。意図的非採用の等価概念・
  代替は本書 §8 の対応表（イシュー #855）を参照

### Part A: ark-ui（`.agents/skills/ark-ui/references/`、90 件）

#### `.agents/skills/ark-ui/references/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/collections/async-list.md` | AsyncListCollection | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/collections/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/list-collection.md` | ListCollection | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/list-selection.md` | ListSelection | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/tree-collection.md` | TreeCollection | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/collections/menu.md` | Menu | Menu | menu | menu | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/select.md` | Select | Select | select | select | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/combobox.md` | Combobox | Combobox | combobox | combobox | 実装済み | headless+styled 実装済み（#749、PR #793） |
| `.agents/skills/ark-ui/references/components/collections/listbox.md` | Listbox | Listbox | listbox | listbox | 実装済み | headless+styled 実装済み（#750） |
| `.agents/skills/ark-ui/references/components/collections/pagination.md` | Pagination | Pagination | pagination | pagination | 実装済み | headless+styled 実装済み（#751、PR #796、#716 保留の解除） |
| `.agents/skills/ark-ui/references/components/collections/steps.md` | Steps | Steps | steps | steps | 実装済み | headless+styled 実装済み（#752、#716 保留の解除） |
| `.agents/skills/ark-ui/references/components/collections/tree-view.md` | TreeView | TreeView | tree_view | tree_view | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/carousel.md` | Carousel | Carousel | carousel | carousel | 実装済み | headless+styled 実装済み（#754）。autoplay（play/pause/aria-live 切替/delay）は初期実装スコープ外（`crates/headless-ui/src/carousel.rs` module doc 参照） |
| `.agents/skills/ark-ui/references/components/collections/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/date-time/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/date-time/date-input.md` | DateInput | — | date_input | date_input | 実装済み | headless+styled 実装済み（#834、#735 保留のうち DateInput 分のみ解除。DatePicker（#835）・Timer（#836）も別途保留解除済み（下記行参照）。calendar は独立部品として実装済み） |
| `.agents/skills/ark-ui/references/components/date-time/date-picker.md` | DatePicker | DatePicker | date_picker | date_picker | 実装済み | headless+styled 実装済み（#835、親トラッキング #832。`docs/policy/intentional-non-adoption.md` §7（#735）の保留解除。DateInput（#834）との連携は行わず ISO 8601 値のネイティブ `<input>` のみで完結） |
| `.agents/skills/ark-ui/references/components/date-time/timer.md` | Timer | Timer | timer | timer | 実装済み | headless+styled+wasm 配線実装済み（#836）。tick を外部から明示的に注入する決定的状態機械（時計 API 非依存）として実装し、`docs/policy/intentional-non-adoption.md` §7 の保留を解除した |
| `.agents/skills/ark-ui/references/components/date-time/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/disclosure/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/disclosure/accordion.md` | Accordion | Accordion | accordion | accordion | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/disclosure/collapsible.md` | Collapsible | Collapsible | collapsible | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/disclosure/tabs.md` | Tabs | Tabs | tabs | tabs | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/disclosure/toggle.md` | Toggle | — | toggle | toggle | 実装済み | headless+styled 実装済み（#746、PR #791） |
| `.agents/skills/ark-ui/references/components/disclosure/toggle-group.md` | ToggleGroup | — | toggle_group | toggle_group | 実装済み | headless+styled 実装済み（#746、PR #791） |
| `.agents/skills/ark-ui/references/components/disclosure/scroll-area.md` | ScrollArea | ScrollArea | scroll_area | scroll_area | 実装済み | headless+styled 実装済み（#825、保留解除。JS によるスクロール位置追従・thumb drag は本イシューのスコープ外） |
| `.agents/skills/ark-ui/references/components/disclosure/splitter.md` | Splitter | Splitter | splitter | splitter | 実装済み | headless+styled 実装済み（#826、#735 保留の解除） |
| `.agents/skills/ark-ui/references/components/disclosure/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/display/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/display/avatar.md` | Avatar | Avatar | avatar | avatar | 実装済み | headless+styled 実装済み（#731 MutationObserver 対応込み） |
| `.agents/skills/ark-ui/references/components/display/progress-linear.md` | Progress (linear) | Progress | progress | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装（circular 対応の #763 とはスコープを分離、follow-up イシュー起票を検討） |
| `.agents/skills/ark-ui/references/components/display/progress-circular.md` | Progress (circular) | ProgressCircle | progress | progress | 実装済み | #763（既存 progress mod を circular 対応へ拡張。headless は #600 で実装済み、pre-styled ラッパーを #763 で追加） |
| `.agents/skills/ark-ui/references/components/display/clipboard.md` | Clipboard | Clipboard | clipboard | clipboard | 実装済み | headless+styled+wasm 配線 実装済み（#773、PR #816） |
| `.agents/skills/ark-ui/references/components/display/qr-code.md` | QrCode | QrCode | qr_code | qr_code | 実装済み | headless+styled 実装済み（#774） |
| `.agents/skills/ark-ui/references/components/display/marquee.md` | Marquee | Marquee | — | marquee | 実装済み（再導入） | #831 で `docs/policy/intentional-non-adoption.md` §3.24 の再評価トリガー 1（CSS のみ・`prefers-reduced-motion` 対応の決定的設計案）を充足し再導入（CSS のみ・JS ゼロ）。headless-ui は変更なし、pre-styled-ui 層のみで新規 anatomy を定義 |
| `.agents/skills/ark-ui/references/components/display/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/form/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/form/checkbox.md` | Checkbox | Checkbox | checkbox | checkbox | 実装済み | headless+styled 実装済み（#730） |
| `.agents/skills/ark-ui/references/components/form/field.md` | Field | Field | field | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/form/fieldset.md` | Fieldset | Fieldset | fieldset | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/form/radio-group.md` | RadioGroup | Radio | radio_group | radio_group | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/form/switch.md` | Switch | Switch | switch | switch | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/form/number-input.md` | NumberInput | NumberInput | number_input | number_input | 実装済み | headless+styled 実装済み（#738、PR #785） |
| `.agents/skills/ark-ui/references/components/form/pin-input.md` | PinInput | PinInput | pin_input | pin_input | 実装済み | headless+styled 実装済み（#739、PR #784） |
| `.agents/skills/ark-ui/references/components/form/password-input.md` | PasswordInput | PasswordInput | password_input | password_input | 実装済み | headless+styled 実装済み（#740） |
| `.agents/skills/ark-ui/references/components/form/slider.md` | Slider | Slider | slider | slider | 実装済み | headless+styled 実装済み（#741） |
| `.agents/skills/ark-ui/references/components/form/rating-group.md` | RatingGroup | Rating | rating_group | rating_group | 実装済み | headless+styled 実装済み（#742） |
| `.agents/skills/ark-ui/references/components/form/segment-group.md` | SegmentGroup | SegmentedControl | segment_group | segment_group | 実装済み | headless+styled 実装済み（#743） |
| `.agents/skills/ark-ui/references/components/form/tags-input.md` | TagsInput | TagsInput | tags_input | tags_input | 実装済み | headless+styled 実装済み（#744） |
| `.agents/skills/ark-ui/references/components/form/editable.md` | Editable | Editable | editable | editable | 実装済み | headless+styled 実装済み（#745） |
| `.agents/skills/ark-ui/references/components/form/angle-slider.md` | AngleSlider | AngleSlider | angle_slider | angle_slider | 実装済み（再導入） | #842 で `docs/policy/intentional-non-adoption.md` §3.22 の再評価トリガー 1（決定的自動テスト基盤の確立・具体的ユースケースを伴う利用要望）を充足し再導入。座標→角度変換（`atan2`）を wasm-full 層の単一純粋関数へ隔離、headless 層は整数角度状態機械のみ |
| `.agents/skills/ark-ui/references/components/form/color-picker.md` | ColorPicker | ColorPicker | color_picker | color_picker | 実装済み | headless+styled 実装済み（#839、親 #837）。canvas 非依存（CSS グラデーション + 導出整数割合）で `docs/policy/intentional-non-adoption.md` §7 再評価トリガー充足、保留解除 |
| `.agents/skills/ark-ui/references/components/form/file-upload.md` | FileUpload | FileUpload | file_upload | file_upload | 実装済み | headless+styled+wasm 実装済み（#840、`docs/policy/intentional-non-adoption.md` §7 保留解除。ItemPreview/ItemPreviewImage はスコープ外） |
| `.agents/skills/ark-ui/references/components/form/image-cropper.md` | ImageCropper | — | image_cropper | image_cropper | 実装済み | headless+styled 実装済み（#844、再導入）。crop 矩形（整数）のみを扱う決定的状態機械として §4 手続きに従い再導入（`docs/policy/intentional-non-adoption.md` §3.22 参照）。canvas による実画像切り出し・pointer ドラッグ配線は対象外（後続 issue） |
| `.agents/skills/ark-ui/references/components/form/signature-pad.md` | SignaturePad | — | signature_pad | signature_pad | 実装済み | canvas を使わない決定的 SVG path 方式で再導入（#843）。headless+styled+wasm 配線済み。非採用の再導入手続きは `docs/policy/intentional-non-adoption.md` §3.22 追補（#735/#843）参照。canvas 方式・残り部品（AngleSlider/RichTextEditor）の非採用判断は不変 |
| `.agents/skills/ark-ui/references/components/form/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/overlays/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/overlays/dialog.md` | Dialog | Dialog | dialog | dialog | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/popover.md` | Popover | Popover | popover | popover | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/tooltip.md` | Tooltip | Tooltip | tooltip | tooltip | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/drawer.md` | Drawer | Drawer | drawer | drawer | 実装済み | headless+styled 実装済み（#758、dialog の状態機械を再利用） |
| `.agents/skills/ark-ui/references/components/overlays/hover-card.md` | HoverCard | HoverCard | hover_card | hover_card | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/toast.md` | Toast | Toast | toast | toast | 実装済み | headless+styled 実装済み（#760、キュー状態機械は `Disclosure`/`SingleSelect` に収まらないため `Component`/`Hydrate` 直接実装） |
| `.agents/skills/ark-ui/references/components/overlays/floating-panel.md` | FloatingPanel | FloatingPanel | floating_panel | floating_panel | 実装済み | headless+styled 実装済み（イシュー #827、`docs/policy/intentional-non-adoption.md` §7 の保留区分から解除） |
| `.agents/skills/ark-ui/references/components/overlays/tour.md` | Tour | Tour | tour | tour | 実装済み | headless+styled 実装済み（#841、#735 保留の解除）。決定的な状態機械・SSR 出力のみが対象で、対象要素の実座標追従・スクロール/リサイズ再計算・target セレクタの実解決は `fandhe-frontend-wasm-full` の後続イシューのスコープ |
| `.agents/skills/ark-ui/references/components/overlays/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/guides/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/guides/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/animation.md` | Animation | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/component-state.md` | ComponentState | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/composition.md` | Composition | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/forms.md` | Forms | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/ref.md` | Ref | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/styling.md` | Styling | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/overview/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/overview/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/about.md` | About | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/changelog.md` | Changelog | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/framework-differences.md` | FrameworkDifferences | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/getting-started.md` | GettingStarted | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/llms-txt.md` | LlmsTxt | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/mcp-server.md` | McpServer | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/utilities/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/utilities/highlight.md` | Highlight | Highlight | — | `highlight` | 実装済み | #775。pre-styled 静的部品 実装済み |
| `.agents/skills/ark-ui/references/utilities/client-only.md` | ClientOnly | ClientOnly | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/download-trigger.md` | DownloadTrigger | DownloadTrigger | `download_trigger` | `download_trigger` | 実装済み | #828。保留（#735 §7「JS ランタイム固有 utilities のうち静的実装可能なもの」）を利用要望 issue（#828）の起票により解除。`a[download]` 属性による静的部品として実装（`Blob`/`data`/`mimeType` は JS 前提のため対応しない） |
| `.agents/skills/ark-ui/references/utilities/environment.md` | Environment | EnvironmentProvider | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/focus-trap.md` | FocusTrap | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/format-byte.md` | FormatByte | FormatByte | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_byte`（Intl 非依存の決定的純関数） |
| `.agents/skills/ark-ui/references/utilities/format-number.md` | FormatNumber | FormatNumber | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_number`（Intl 非依存の決定的純関数） |
| `.agents/skills/ark-ui/references/utilities/format-relative-time.md` | FormatRelativeTime | — | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_relative_time`（基準時刻は呼び出し側注入、現在時刻 API 非依存） |
| `.agents/skills/ark-ui/references/utilities/format-time.md` | FormatTime | — | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_time`（決定的純関数） |
| `.agents/skills/ark-ui/references/utilities/frame.md` | Frame | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/json-tree-view.md` | JsonTreeView | — | json_tree_view | json_tree_view | 実装済み | **保留解除**（イシュー #829、`tree_view`（#753）の派生として実装。headless `crates/headless-ui/src/json_tree_view.rs` + styled `crates/pre-styled-ui/src/json_tree_view.rs`。`docs/policy/intentional-non-adoption.md` §7 の解除記録参照） |
| `.agents/skills/ark-ui/references/utilities/locale.md` | Locale | LocaleProvider | — | — | 実装済み（Rust 最適化形） | イシュー #854。`Locale` 値型（`crates/headless-ui/src/format.rs` の `format` mod、en/ja）として実装。`LocaleProvider` の Context/Provider 機構は非採用のまま（`docs/policy/intentional-non-adoption.md` §3.23 参照） |
| `.agents/skills/ark-ui/references/utilities/presence.md` | Presence | Presence | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/swap.md` | Swap | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

### Part B: chakra-ui（`.agents/skills/chakra-ui/references/`、269 件）

#### `.agents/skills/chakra-ui/references/blocks/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/blocks/README.md` | — | README | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/blocks/ai.md` | — | Ai | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/app-headers.md` | — | AppHeaders | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/app-integrations.md` | — | AppIntegrations | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/app-navbars.md` | — | AppNavbars | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/authentication.md` | — | Authentication | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/banners.md` | — | Banners | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/blogs.md` | — | Blogs | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/cards.md` | — | Cards | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/careers.md` | — | Careers | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/charts.md` | — | Charts | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/contacts.md` | — | Contacts | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/cta.md` | — | Cta | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/dividers.md` | — | Dividers | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-changelog.md` | — | DocsChangelog | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-code-block.md` | — | DocsCodeBlock | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-example-preview.md` | — | DocsExamplePreview | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-header.md` | — | DocsHeader | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-navbar.md` | — | DocsNavbar | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-pagination.md` | — | DocsPagination | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-parameter-field.md` | — | DocsParameterField | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-sidebar.md` | — | DocsSidebar | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-step.md` | — | DocsStep | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-toc.md` | — | DocsToc | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/faqs.md` | — | Faqs | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/features.md` | — | Features | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/feeds.md` | — | Feeds | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/footers.md` | — | Footers | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/help-center.md` | — | HelpCenter | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/heroes.md` | — | Heroes | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/layouts.md` | — | Layouts | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/logos.md` | — | Logos | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/marketing-headers.md` | — | MarketingHeaders | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/marketing-navbars.md` | — | MarketingNavbars | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/notifications.md` | — | Notifications | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/onboarding.md` | — | Onboarding | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/org-switcher.md` | — | OrgSwitcher | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/pricing.md` | — | Pricing | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/product-categories.md` | — | ProductCategories | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/product-grid.md` | — | ProductGrid | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/product-reviews.md` | — | ProductReviews | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/profiles.md` | — | Profiles | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/property-panels.md` | — | PropertyPanels | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/settings.md` | — | Settings | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/sharing.md` | — | Sharing | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/shopping-carts.md` | — | ShoppingCarts | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/sidebars.md` | — | Sidebars | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/stats.md` | — | Stats | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/store-signup-offers.md` | — | StoreSignupOffers | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/teams.md` | — | Teams | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/testimonials.md` | — | Testimonials | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/webhooks.md` | — | Webhooks | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |

#### `.agents/skills/chakra-ui/references/charts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/charts/README.md` | — | README | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/charts/area-chart.md` | — | AreaChart | — | `area_chart` | 実装済み | 保留解除。#848（軸/グリッド/凡例/ツールチップ/積み上げは #847 以降のスコープ） |
| `.agents/skills/chakra-ui/references/charts/axes.md` | — | Axes | — | `charts::axis`（`y_axis`/`x_axis_linear`/`x_axis_categories`） | 実装済み | #847、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/bar-chart.md` | — | BarChart | — | `charts::bar_chart` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy `bar-chart` を定義）。#849（親 Phase #845、charts 基盤 #846 の上に実装）。`docs/policy/intentional-non-adoption.md` §7 の保留を解除。軸線・グリッド・凡例・ツールチップは #847 のスコープ |
| `.agents/skills/chakra-ui/references/charts/bar-list.md` | — | BarList | — | `charts::bar_list` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy `bar-list` を定義）。#849（親 Phase #845）。`docs/policy/intentional-non-adoption.md` §7 の保留を解除 |
| `.agents/skills/chakra-ui/references/charts/bar-segment.md` | — | BarSegment | — | `charts::bar_segment` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy `bar-segment` を定義）。#849（親 Phase #845）。`docs/policy/intentional-non-adoption.md` §7 の保留を解除 |
| `.agents/skills/chakra-ui/references/charts/cartesian-grid.md` | — | CartesianGrid | — | `charts::grid`（`cartesian_grid`） | 実装済み | #847、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/donut-chart.md` | — | DonutChart | — | `donut_chart` | 実装済み | 保留解除。#850、charts 基盤（#846）を用いたドーナツグラフ、詳細は `crates/pre-styled-ui/src/donut_chart.rs` rustdoc |
| `.agents/skills/chakra-ui/references/charts/installation.md` | — | Installation | — | `charts`（外部依存追加なし、`fandhe-frontend-pre-styled-ui` のみで完結） | 実装済み | 保留解除（基盤のみ）。#846、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/legend.md` | — | Legend | — | `charts::legend`（`legend`） | 実装済み | #847、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/line-chart.md` | — | LineChart | — | `line_chart` | 実装済み | 保留解除。#848（軸/グリッド/凡例/ツールチップ/積み上げは #847 以降のスコープ） |
| `.agents/skills/chakra-ui/references/charts/pie-chart.md` | — | PieChart | — | `pie_chart` | 実装済み | 保留解除。#850、charts 基盤（#846）を用いた円グラフ、詳細は `crates/pre-styled-ui/src/pie_chart.rs` rustdoc |
| `.agents/skills/chakra-ui/references/charts/radar-chart.md` | — | RadarChart | — | `charts::radar_chart` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy）。#851（親 Phase #845、charts 基盤 #846 の上に実装）。保留解除 |
| `.agents/skills/chakra-ui/references/charts/scatter-chart.md` | — | ScatterChart | — | `charts::scatter_chart` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy）。#851（親 Phase #845、charts 基盤 #846 の上に実装）。保留解除 |
| `.agents/skills/chakra-ui/references/charts/sparkline.md` | — | Sparkline | — | `sparkline` | 実装済み | 保留解除。#848（単一系列専用。複数系列は LineChart/AreaChart を使用） |
| `.agents/skills/chakra-ui/references/charts/tooltip.md` | — | Tooltip | — | `charts::tooltip`（`datum`/`datum_label`。汎用 headless Tooltip（`tooltip` モジュール）とは別物） | 実装済み | #847、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/use-chart.md` | — | UseChart | — | `charts`（`ChartData`/`LinearScale`/SVG ヘルパー） | 実装済み | 保留解除（基盤のみ）。#846、詳細は `docs/design/charts-foundation-design.md` |

#### `.agents/skills/chakra-ui/references/components/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/README.md` | — | README | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/chakra-ui/references/components/buttons/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/buttons/button.md` | — | Button | — | button | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/buttons/download-trigger.md` | DownloadTrigger | DownloadTrigger | `download_trigger` | `download_trigger` | 実装済み | #828。保留（#735 §7「JS ランタイム固有 utilities のうち静的実装可能なもの」）を利用要望 issue（#828）の起票により解除。`a[download]` 属性による静的部品として実装（`Blob`/`data`/`mimeType` は JS 前提のため対応しない） |
| `.agents/skills/chakra-ui/references/components/buttons/close-button.md` | — | CloseButton | — | `button`（`close_button`） | 実装済み | #830。保留（Button バリエーション、#735 §7）を `Button` variant 拡張要望 issue（#830）の起票により解除。独立部品ではなく `button` recipe の icon-only 修飾 variant として実装（`data-scope="button"` を共有） |
| `.agents/skills/chakra-ui/references/components/buttons/icon-button.md` | — | IconButton | — | `button`（`icon_button`） | 実装済み | #830。close-button と同一の解除・実装判断（同上） |

#### `.agents/skills/chakra-ui/references/components/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/collections/select.md` | Select | Select | select | select | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/collections/combobox.md` | Combobox | Combobox | combobox | combobox | 実装済み | headless+styled 実装済み（#749、PR #793） |
| `.agents/skills/chakra-ui/references/components/collections/listbox.md` | Listbox | Listbox | listbox | listbox | 実装済み | headless+styled 実装済み（#750） |
| `.agents/skills/chakra-ui/references/components/collections/tree-view.md` | TreeView | TreeView | tree_view | tree_view | 実装済み | headless+styled 実装済み |

#### `.agents/skills/chakra-ui/references/components/concepts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/concepts/animation.md` | — | Animation | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/color-mode.md` | — | ColorMode | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/composition.md` | — | Composition | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/overview.md` | — | Overview | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/server-components.md` | — | ServerComponents | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/testing.md` | — | Testing | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/chakra-ui/references/components/data-display/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/data-display/avatar.md` | Avatar | Avatar | avatar | avatar | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/data-display/badge.md` | — | Badge | — | badge | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/data-display/card.md` | — | Card | — | card | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/data-display/table.md` | — | Table | — | table | 実装済み | pre-styled 静的部品 実装済み（#767。`interactive`/`stickyHeader`/`showColumnBorder`/`ScrollArea`/`ColumnGroup` はスコープ外） |
| `.agents/skills/chakra-ui/references/components/data-display/data-list.md` | — | DataList | — | `data_list` | 実装済み | pre-styled 静的部品 実装済み（#767。`variant`（subtle/bold）/`size` variant はスコープ外） |
| `.agents/skills/chakra-ui/references/components/data-display/tag.md` | — | Tag | — | tag | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/data-display/stat.md` | — | Stat | — | `stat` | 実装済み | pre-styled 静的部品 実装済み（#769。headless-ui は変更なし） |
| `.agents/skills/chakra-ui/references/components/data-display/timeline.md` | — | Timeline | — | `timeline` | 実装済み | pre-styled 静的部品 実装済み（#769。headless-ui は変更なし） |
| `.agents/skills/chakra-ui/references/components/data-display/image.md` | — | Image | — | image | 実装済み | #770。状態機械なし静的部品、pre-styled 層のみに実装（headless-ui は変更なし）。fit（object-fit）/aspect-ratio の 2 軸 variant、alt 必須引数 |
| `.agents/skills/chakra-ui/references/components/data-display/icon.md` | — | Icon | — | icon | 実装済み | #770。状態機械なし静的部品、pre-styled 層のみに実装（headless-ui は変更なし）。size variant のみ、SVG 本体は呼び出し側がノード木 API で構築 |
| `.agents/skills/chakra-ui/references/components/data-display/clipboard.md` | Clipboard | Clipboard | clipboard | clipboard | 実装済み | headless+styled+wasm 配線 実装済み（#773、PR #816） |
| `.agents/skills/chakra-ui/references/components/data-display/qr-code.md` | QrCode | QrCode | qr_code | qr_code | 実装済み | headless+styled 実装済み（#774） |
| `.agents/skills/chakra-ui/references/components/data-display/marquee.md` | Marquee | Marquee | — | marquee | 実装済み（再導入） | #831 で `docs/policy/intentional-non-adoption.md` §3.24 の再評価トリガー 1（CSS のみ・`prefers-reduced-motion` 対応の決定的設計案）を充足し再導入（CSS のみ・JS ゼロ）。headless-ui は変更なし、pre-styled-ui 層のみで新規 anatomy を定義 |

#### `.agents/skills/chakra-ui/references/components/date-time/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/date-time/date-picker.md` | DatePicker | DatePicker | date_picker | date_picker | 実装済み | headless+styled 実装済み（#835、親トラッキング #832。`docs/policy/intentional-non-adoption.md` §7（#735）の保留解除） |
| `.agents/skills/chakra-ui/references/components/date-time/calendar.md` | — | Calendar | calendar | calendar | 実装済み | headless+styled 実装済み（#835、親トラッキング #832。`docs/policy/intentional-non-adoption.md` §7（#735）の保留解除） |

#### `.agents/skills/chakra-ui/references/components/disclosure/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/disclosure/accordion.md` | Accordion | Accordion | accordion | accordion | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/disclosure/collapsible.md` | Collapsible | Collapsible | collapsible | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/chakra-ui/references/components/disclosure/tabs.md` | Tabs | Tabs | tabs | tabs | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/disclosure/pagination.md` | Pagination | Pagination | pagination | pagination | 実装済み | headless+styled 実装済み（#751、PR #796、#716 保留の解除） |
| `.agents/skills/chakra-ui/references/components/disclosure/steps.md` | Steps | Steps | steps | steps | 実装済み | headless+styled 実装済み（#752、#716 保留の解除） |
| `.agents/skills/chakra-ui/references/components/disclosure/carousel.md` | Carousel | Carousel | carousel | carousel | 実装済み | headless+styled 実装済み（#754）。autoplay（play/pause/aria-live 切替/delay）は初期実装スコープ外（`crates/headless-ui/src/carousel.rs` module doc 参照） |
| `.agents/skills/chakra-ui/references/components/disclosure/breadcrumb.md` | — | Breadcrumb | breadcrumb | breadcrumb | 実装済み | #755（#716 追加候補の消化）。headless+styled 実装済み |

#### `.agents/skills/chakra-ui/references/components/feedback/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/feedback/progress.md` | Progress (linear) | Progress | progress | progress | 実装済み | headless+styled（root）実装済み。linear（Track/Range）用 styled ラッパーは #763 とスコープを分離した対応表側の未実装事項（follow-up イシュー起票を検討） |
| `.agents/skills/chakra-ui/references/components/feedback/alert.md` | — | Alert | — | alert | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/feedback/spinner.md` | — | Spinner | — | spinner | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/feedback/toast.md` | Toast | Toast | toast | toast | 実装済み | headless+styled 実装済み（#760） |
| `.agents/skills/chakra-ui/references/components/feedback/progress-circle.md` | Progress (circular) | ProgressCircle | progress | progress | 実装済み | #763（既存 progress mod を circular 対応へ拡張。headless は #600 で実装済み、pre-styled ラッパーを #763 で追加） |
| `.agents/skills/chakra-ui/references/components/feedback/skeleton.md` | — | Skeleton | — | skeleton | 実装済み | #764。pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/feedback/status.md` | — | Status | — | status | 実装済み | pre-styled 静的部品 実装済み（#765） |
| `.agents/skills/chakra-ui/references/components/feedback/empty-state.md` | — | EmptyState | — | empty_state | 実装済み | pre-styled 静的部品 実装済み（#765） |

#### `.agents/skills/chakra-ui/references/components/forms/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/forms/checkbox.md` | Checkbox | Checkbox | checkbox | checkbox | 実装済み | headless+styled 実装済み（#730） |
| `.agents/skills/chakra-ui/references/components/forms/field.md` | Field | Field | field | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/chakra-ui/references/components/forms/fieldset.md` | Fieldset | Fieldset | fieldset | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/chakra-ui/references/components/forms/radio.md` | RadioGroup | Radio | radio_group | radio_group | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/forms/switch.md` | Switch | Switch | switch | switch | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/forms/input.md` | — | Input | — | input | 実装済み | pre-styled 静的部品として実装済み（#737） |
| `.agents/skills/chakra-ui/references/components/forms/textarea.md` | — | Textarea | — | textarea | 実装済み | pre-styled 静的部品として実装済み（#737） |
| `.agents/skills/chakra-ui/references/components/forms/native-select.md` | — | NativeSelect | — | native_select | 実装済み | pre-styled 静的部品として実装済み（#737） |
| `.agents/skills/chakra-ui/references/components/forms/number-input.md` | NumberInput | NumberInput | number_input | number_input | 実装済み | headless+styled 実装済み（#738、PR #785） |
| `.agents/skills/chakra-ui/references/components/forms/pin-input.md` | PinInput | PinInput | pin_input | pin_input | 実装済み | headless+styled 実装済み（#739、PR #784） |
| `.agents/skills/chakra-ui/references/components/forms/password-input.md` | PasswordInput | PasswordInput | password_input | password_input | 実装済み | headless+styled 実装済み（#740） |
| `.agents/skills/chakra-ui/references/components/forms/slider.md` | Slider | Slider | slider | slider | 実装済み | headless+styled 実装済み（#741） |
| `.agents/skills/chakra-ui/references/components/forms/rating.md` | RatingGroup | Rating | rating_group | rating_group | 実装済み | headless+styled 実装済み（#742） |
| `.agents/skills/chakra-ui/references/components/forms/segmented-control.md` | SegmentGroup | SegmentedControl | segment_group | segment_group | 実装済み | headless+styled 実装済み（#743） |
| `.agents/skills/chakra-ui/references/components/forms/tags-input.md` | TagsInput | TagsInput | tags_input | tags_input | 実装済み | headless+styled 実装済み（#744） |
| `.agents/skills/chakra-ui/references/components/forms/editable.md` | Editable | Editable | editable | editable | 実装済み | headless+styled 実装済み（#745） |
| `.agents/skills/chakra-ui/references/components/forms/checkbox-card.md` | — | CheckboxCard | — | checkbox_card | 実装済み | pre-styled styled バリエーション実装済み（#747。headless-ui は変更なし、状態機械は headless Checkbox を再利用） |
| `.agents/skills/chakra-ui/references/components/forms/radio-card.md` | — | RadioCard | — | radio_card | 実装済み | pre-styled styled バリエーション実装済み（#747。headless-ui は変更なし、状態機械は headless RadioGroup を再利用） |
| `.agents/skills/chakra-ui/references/components/forms/color-picker.md` | ColorPicker | ColorPicker | color_picker | color_picker | 実装済み | headless+styled 実装済み（#839、親 #837）。canvas 非依存（CSS グラデーション + 導出整数割合）で `docs/policy/intentional-non-adoption.md` §7 再評価トリガー充足、保留解除 |
| `.agents/skills/chakra-ui/references/components/forms/color-swatch.md` | — | ColorSwatch | — | color_swatch | 実装済み | pre-styled 静的部品として実装済み（#838。headless-ui には対応する anatomy を新設しない。色変換コアは `fandhe-frontend-headless-ui::color`、親 #837） |
| `.agents/skills/chakra-ui/references/components/forms/file-upload.md` | FileUpload | FileUpload | file_upload | file_upload | 実装済み | headless+styled+wasm 実装済み（#840、`docs/policy/intentional-non-adoption.md` §7 保留解除。ItemPreview/ItemPreviewImage はスコープ外） |

#### `.agents/skills/chakra-ui/references/components/i18n/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/i18n/format-byte.md` | FormatByte | FormatByte | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_byte`（Intl 非依存の決定的純関数） |
| `.agents/skills/chakra-ui/references/components/i18n/format-number.md` | FormatNumber | FormatNumber | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_number`（Intl 非依存の決定的純関数） |
| `.agents/skills/chakra-ui/references/components/i18n/locale-provider.md` | Locale | LocaleProvider | — | — | 実装済み（Rust 最適化形） | イシュー #854。`Locale` 値型（`format` mod、en/ja）として実装。`LocaleProvider` の Context/Provider 機構・グローバル既定ロケールは意図的に非採用のまま（`docs/policy/intentional-non-adoption.md` §3.23 参照） |

#### `.agents/skills/chakra-ui/references/components/layout/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/layout/separator.md` | — | Separator | — | separator | 実装済み | #772。pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/layout/scroll-area.md` | ScrollArea | ScrollArea | scroll_area | scroll_area | 実装済み | headless+styled 実装済み（#825、保留解除。JS によるスクロール位置追従・thumb drag は本イシューのスコープ外） |
| `.agents/skills/chakra-ui/references/components/layout/splitter.md` | Splitter | Splitter | splitter | splitter | 実装済み | headless+styled 実装済み（#826、#735 保留の解除） |
| `.agents/skills/chakra-ui/references/components/layout/absolute-center.md` | — | AbsoluteCenter | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/aspect-ratio.md` | — | AspectRatio | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/bleed.md` | — | Bleed | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/box.md` | — | Box | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/center.md` | — | Center | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/container.md` | — | Container | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/flex.md` | — | Flex | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/float.md` | — | Float | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/grid.md` | — | Grid | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/group.md` | — | Group | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/simple-grid.md` | — | SimpleGrid | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/stack.md` | — | Stack | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/wrap.md` | — | Wrap | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |

#### `.agents/skills/chakra-ui/references/components/overlays/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/overlays/dialog.md` | Dialog | Dialog | dialog | dialog | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/menu.md` | Menu | Menu | menu | menu | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/popover.md` | Popover | Popover | popover | popover | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/tooltip.md` | Tooltip | Tooltip | tooltip | tooltip | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/drawer.md` | Drawer | Drawer | drawer | drawer | 実装済み | headless+styled 実装済み（#758、dialog の状態機械を再利用） |
| `.agents/skills/chakra-ui/references/components/overlays/hover-card.md` | HoverCard | HoverCard | hover_card | hover_card | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/toggle-tip.md` | — | ToggleTip | toggle_tip | toggle_tip | 実装済み | headless+styled 実装済み（#761、PR #804） |
| `.agents/skills/chakra-ui/references/components/overlays/action-bar.md` | — | ActionBar | action_bar | action_bar | 実装済み | headless+styled 実装済み（#762） |
| `.agents/skills/chakra-ui/references/components/overlays/overlay-manager.md` | — | OverlayManager | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/overlays/floating-panel.md` | FloatingPanel | FloatingPanel | floating_panel | floating_panel | 実装済み | headless+styled 実装済み（イシュー #827、`docs/policy/intentional-non-adoption.md` §7 の保留区分から解除） |

#### `.agents/skills/chakra-ui/references/components/typography/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/typography/link.md` | — | Link | link | link | 実装済み | headless+styled 実装済み（#756、PR #801、#716 最優先候補の消化） |
| `.agents/skills/chakra-ui/references/components/typography/link-overlay.md` | — | LinkOverlay | link_overlay | link_overlay | 実装済み | headless+styled 実装済み（#756、PR #801、#716 最優先候補の消化） |
| `.agents/skills/chakra-ui/references/components/typography/kbd.md` | — | Kbd | — | kbd | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/typography/code.md` | — | Code | — | code | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/typography/heading.md` | — | Heading | — | heading | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/text.md` | — | Text | — | text | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/em.md` | — | Em | — | em | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/mark.md` | — | Mark | — | mark | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/blockquote.md` | — | Blockquote | — | blockquote | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/list.md` | — | List | — | list | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/highlight.md` | Highlight | Highlight | — | highlight | 実装済み | #775。pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/typography/rich-text-editor.md` | — | RichTextEditor | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.22（#735）で非採用確定（高度入力系、canvas/ポインタ座標/contenteditable 依存）。等価概念は本書 §8 参照 |
| `.agents/skills/chakra-ui/references/components/typography/code-block.md` | — | CodeBlock | — | — | 対象外 | 対象外（docs-site 既存機構と役割分担） |
| `.agents/skills/chakra-ui/references/components/typography/prose.md` | — | Prose | — | — | 対象外 | 対象外（docs-site 既存機構と役割分担。役割分担の詳細は #771 の `crates/pre-styled-ui/src/text.rs` rustdoc・`docs/api/pre-styled-ui-api.md` 参照） |

#### `.agents/skills/chakra-ui/references/components/utilities/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/utilities/visually-hidden.md` | — | VisuallyHidden | `visually_hidden` | `visually_hidden` | 実装済み | #776 |
| `.agents/skills/chakra-ui/references/components/utilities/skip-nav.md` | — | SkipNav | `skip_nav` | `skip_nav` | 実装済み | #776 |
| `.agents/skills/chakra-ui/references/components/utilities/client-only.md` | ClientOnly | ClientOnly | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/environment-provider.md` | Environment | EnvironmentProvider | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/presence.md` | Presence | Presence | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/checkmark.md` | — | Checkmark | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/radiomark.md` | — | Radiomark | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/for.md` | — | For | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/portal.md` | — | Portal | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/show.md` | — | Show | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/theme.md` | — | Theme | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.24（#735）で非採用確定（既存 theme mod と役割重複）。等価概念は本書 §8 参照 |

#### `.agents/skills/chakra-ui/references/get-started/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/get-started/README.md` | — | README | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/ai-llms.md` | — | AiLlms | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/ai-mcp-server.md` | — | AiMcpServer | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/ai-rules.md` | — | AiRules | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/ai-skills.md` | — | AiSkills | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/changelog.md` | — | Changelog | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/cli.md` | — | Cli | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/contributing.md` | — | Contributing | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/env-iframe.md` | — | EnvIframe | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/env-shadow-dom.md` | — | EnvShadowDom | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/figma.md` | — | Figma | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-next-app.md` | — | FrameworkNextApp | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-next-pages.md` | — | FrameworkNextPages | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-remix.md` | — | FrameworkRemix | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-storybook.md` | — | FrameworkStorybook | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-tanstack-router.md` | — | FrameworkTanstackRouter | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-vite.md` | — | FrameworkVite | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/installation.md` | — | Installation | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/migration.md` | — | Migration | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/playground.md` | — | Playground | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/chakra-ui/references/styling/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/README.md` | — | Readme | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/styling/compositions/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/compositions/animation-styles.md` | — | AnimationStyles | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/focus-ring.md` | — | FocusRing | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/layer-styles.md` | — | LayerStyles | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/text-styles.md` | — | TextStyles | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/styling/concepts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/concepts/cascade-layers.md` | — | CascadeLayers | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/chakra-factory.md` | — | ChakraFactory | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/color-opacity-modifier.md` | — | ColorOpacityModifier | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/conditional-styles.md` | — | ConditionalStyles | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/css-variables.md` | — | CssVariables | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/dark-mode.md` | — | DarkMode | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/overview.md` | — | Overview | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/responsive-design.md` | — | ResponsiveDesign | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/virtual-color.md` | — | VirtualColor | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/styling/style-props/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/style-props/background.md` | — | Background | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/border.md` | — | Border | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/display.md` | — | Display | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/effects.md` | — | Effects | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/filters.md` | — | Filters | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/flex-and-grid.md` | — | FlexAndGrid | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/interactivity.md` | — | Interactivity | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/layout.md` | — | Layout | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/list.md` | — | List | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/sizing.md` | — | Sizing | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/spacing.md` | — | Spacing | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/svg.md` | — | Svg | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/tables.md` | — | Tables | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/transforms.md` | — | Transforms | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/transitions.md` | — | Transitions | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/typography.md` | — | Typography | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/README.md` | — | Readme | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/compositions/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/compositions/layer-styles.md` | — | LayerStyles | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/compositions/text-styles.md` | — | TextStyles | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/concepts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/concepts/overview.md` | — | Overview | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/recipes.md` | — | Recipes | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/semantic-tokens.md` | — | SemanticTokens | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/slot-recipes.md` | — | SlotRecipes | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/tokens.md` | — | Tokens | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/customization/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/customization/animations.md` | — | Animations | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/breakpoints.md` | — | Breakpoints | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/colors.md` | — | Colors | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/conditions.md` | — | Conditions | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/css-variables.md` | — | CssVariables | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/global-css.md` | — | GlobalCss | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/overview.md` | — | Overview | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/recipes.md` | — | Recipes | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/sizes.md` | — | Sizes | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/spacing.md` | — | Spacing | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/utilities.md` | — | Utilities | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/design-tokens/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/design-tokens/animations.md` | — | Animations | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/aspect-ratios.md` | — | AspectRatios | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/breakpoints.md` | — | Breakpoints | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/colors.md` | — | Colors | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/cursors.md` | — | Cursors | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/radii.md` | — | Radii | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/shadows.md` | — | Shadows | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/sizes.md` | — | Sizes | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/spacing.md` | — | Spacing | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/typography.md` | — | Typography | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/z-index.md` | — | ZIndex | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

### Part C: fandhe 独自 mod・charts 基盤（ark-ui / chakra-ui に対応 md なし）

§4 の完全性 diff（references 側 359 件の突合）の対象外である。本 Part の
セルには `.agents/skills/…` のパスを一切書かない（書くと §4 の抽出正規表現
（`\.agents/skills/(ark-ui|chakra-ui)/references/[A-Za-z0-9/._-]+\.md`）が
実在しないパス文字列を拾ってしまい、diff が非空になって §4 が壊れる）。

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| —（対応 md なし） | — | — | `nav_list` | `nav_list` | 実装済み | fandhe 独自。#756 → PR #801 で Link / LinkOverlay と同時に実装した文書ナビ用リスト（Root / Heading / List / Item / Link の 5 anatomy） |
| —（対応 md なし） | — | — | — | `charts::data` | 実装済み | charts 基盤。#846。`ChartData` / `Series` モデルと集計 API（ノードを生成しない純計算） |
| —（対応 md なし） | — | — | — | `charts::scale` | 実装済み | charts 基盤。#846。線形スケール・1-2-5 nice tick 算出（ノードを生成しない純計算） |
| —（対応 md なし） | — | — | — | `charts::svg` | 実装済み | charts 基盤。#846。SVG ノード木生成ヘルパー |
| —（対応 md なし） | — | — | — | `charts::pie` | 実装済み | charts 基盤。#850 → PR #881。`pie_chart` / `donut_chart` が使う円弧ジオメトリ（`d` 属性文字列を返す純関数） |

※ `charts::data` / `scale` / `svg` は既に `charts/use-chart.md` 行（Part B
「charts」節）の本文中で散文的に触れられているが、mod 名としては計上されて
いなかったため本 Part で行として明示する（重複計上ではなく、mod 側キーでの
計上）。

## 6. `site/nav.toml` 掲載要否の判断

**掲載しない**（安全側の判断）。

- `site/nav.toml` の現行掲載対象は利用者向け文書（`docs/guides/` /
  `docs/api/` / `examples/*/README.md` / `site/`）のみであり、
  `docs/design/` 配下の設計文書は既存 23 件すべて非掲載。本書は開発内部の
  実装トラッキング文書（issue 番号・実装状況を含む）であり、既存方針に
  整合する
- 非掲載のため docs-site の linkcheck（`crates/docs-site/src/linkcheck.rs`
  内蔵、`build_site()` から呼ばれる）の対象にならない

## 7. スコープ外事項（放置しない）

- **機械確認の CI 常時実行化**: 本書 §4 の diff コマンド・§4.2 の mod 側
  計上漏れ確認コマンド（イシュー #934 で追加）をいずれも CI ジョブ /
  テストとして自動実行する仕組みは本イシューのスコープ外。`crates/xtask`
  への `check-coverage-map` サブコマンド追加等でこれらを機械検知したい
  場合は別 issue の起票を提案する（`.claude/rules/out-of-scope-tracking.md`
  に従い、ユーザー承認の上で #726 配下へ）
- **§5 全 359 行のバッククォート表記の統一**: 本イシュー（#934）では
  `data-list` 表記の 1 件のみを是正し、他の表記ゆれには手を入れていない。
  全面統一は #937（Radix 列追加）と同時実施のほうが差分が小さいため
  スコープ外とした
- **`mod qr_encode` / `mod class_attr` の可視性見直し**: 両 mod は
  `mod`（非 `pub`）宣言のため §3 の突合コマンドの対象外だが、`crates/**`
  の変更を伴うため本書のみを対象とする本イシュー（doc-only）のスコープ外
- **保留・意図的非採用の評価軸・再評価トリガーの詳細記録**: イシュー #735 で
  `docs/policy/intentional-non-adoption.md` §3.22〜§3.24（新規非採用確定）・
  §7（保留項目の記録）に確定記録済み（完了）

## 8. JS ランタイム固有 utilities の Rust 等価概念対応表（イシュー #855）

§3.23 の 24 行（JS ランタイム固有 utilities）+ RichTextEditor（§3.22）+
chakra `Theme`（§3.24）について、利用者が「chakra-ui / ark-ui の X は
fandhe では何か」を引くための対応表。非採用判断そのものの一次記録は
引き続き `docs/policy/intentional-non-adoption.md` §3.22〜§3.24 であり、
本節はそこから導かれる等価概念・代替実装のみを利用者向けに集約する。

| JS 側コンポーネント | 対応する参照ファイル（本書 §5） | fandhe / Rust での等価概念・代替 | 参照先 | 備考（設計上の理由） |
|---|---|---|---|---|
| Portal | chakra `components/utilities/portal.md` | ノード木上の明示的配置（overlay 基盤が担当） | `crates/wasm-full/src/overlay.rs` | オーバーレイのスタック管理・配置はノード木 API 上で明示的に組み立てる。第 2 の描画経路（ランタイム的な要素の移設）は導入しない |
| Show | chakra `components/utilities/show.md` | `Option` + Rust の `if` によるノード木条件構築 | `docs/api/component-api.md` | 条件付きレンダリングは通常の Rust 制御構文でノード木を組み立てる。専用コンポーネント API は設けない |
| For | chakra `components/utilities/for.md` | Rust イテレータ + keyed_list | `crates/core/src/keyed.rs` | リストレンダリングは `keyed_list` 束縛点（`fandhe-frontend-core`）で差分更新する。JS 側の宣言的ヘルパーに相当する専用 API は設けない |
| Presence | ark `utilities/presence.md`, chakra `components/utilities/presence.md` | `data-state` 属性 + CSS transition（View Transitions 連携） | `docs/guides/view-transitions.md` | マウント/アンマウントのアニメーションは `data-state` 属性と CSS/View Transitions で表現し、ランタイム機構は持ち込まない |
| ClientOnly | ark `utilities/client-only.md`, chakra `components/utilities/client-only.md` | 等価概念なし | `docs/policy/intentional-non-adoption.md` §3.23 | SSR/CSR 単一描画モデル（ノード木を唯一の描画経路とする設計）のため、実行環境で分岐するランタイム機構自体が存在しない |
| Environment(Provider) | ark `utilities/environment.md`, chakra `components/utilities/environment-provider.md` | 等価概念なし | `docs/policy/intentional-non-adoption.md` §3.23 | 同上（実行環境分岐の機構がない） |
| Frame | ark `utilities/frame.md` | 等価概念なし | `docs/policy/intentional-non-adoption.md` §3.23 | iframe 内レンダリングの専用機構は持たない |
| Swap | ark `utilities/swap.md` | 等価概念なし | `docs/policy/intentional-non-adoption.md` §3.23 | 要素差し替えのランタイム機構は持たない |
| AsyncListCollection | ark `collections/async-list.md` | 等価概念なし | `docs/policy/intentional-non-adoption.md` §3.23 | 非同期コレクション処理は UI コンポーネント層の責務外。専用ランタイムは持たない |
| FocusTrap | ark `utilities/focus-trap.md` | 実装済み | `crates/wasm-full/src/focus_trap.rs` | フォーカストラップは既存実装で代替済み。汎用 utilities API としての新設は不要 |
| OverlayManager | chakra `components/overlays/overlay-manager.md` | 実装済み | `crates/wasm-full/src/overlay.rs` | オーバーレイのスタック管理は既存実装で代替済み |
| FormatByte / FormatNumber / FormatRelativeTime / FormatTime | ark `utilities/format-*.md`, chakra `components/i18n/format-byte.md` / `format-number.md` | 利用者側の通常の Rust 関数で整形 | `docs/policy/intentional-non-adoption.md` §3.23 | 数値・日時整形は UI コンポーネント層の責務外。国際化ライブラリは持ち込まない |
| Locale(Provider) | ark `utilities/locale.md`, chakra `components/i18n/locale-provider.md` | `Locale` 値型（`headless-ui::format::Locale`）を引数渡し | `crates/headless-ui/src/format.rs` | イシュー #854。en/ja の 2 種を各 `Format*Options::locale` フィールド経由で明示的に渡す値型として実装。`LocaleProvider` の Context/Provider 機構・グローバル既定ロケールは非採用のまま（`docs/policy/intentional-non-adoption.md` §3.23） |
| Checkmark | chakra `components/utilities/checkmark.md` | `checkbox` の状態機械に吸収済み | `crates/headless-ui/src/checkbox.rs` | チェック表示は `checkbox` mod の一部として実装済み。装飾専用の独立 API は設けない |
| Radiomark | chakra `components/utilities/radiomark.md` | `radio_group` の状態機械に吸収済み | `crates/headless-ui/src/radio_group.rs` | ラジオ表示は `radio_group` mod の一部として実装済み |
| Theme（chakra） | chakra `components/utilities/theme.md` | `crates/pre-styled-ui` の `theme` / `recipe` / `stylesheet` mod | `crates/pre-styled-ui/src/theme.rs` / `recipe.rs` / `stylesheet.rs` | テーマ管理は既存 3 mod を唯一の入口として維持する（§3.24） |
| RichTextEditor | chakra `components/typography/rich-text-editor.md` | 非採用維持（等価概念なし） | `docs/policy/intentional-non-adoption.md` §3.22 | REQ-1（既定エスケープ）と本質衝突（`contenteditable` 由来 HTML がエスケープ経路外から持ち込まれる）。EditContext API 等の構造化編集 API 成熟時に再評価（§3.22 の再評価トリガー 2） |

上記表は §3.23 の 24 行（L124, L245, L247〜L253, L255〜L257, L456〜L458,
L493, L521〜L528）+ RichTextEditor（L511, §3.22）+ chakra `Theme`（L529,
§3.24）をすべてカバーする。行番号は本書の現行版時点のものであり、将来の
行挿入でずれうる（一次キーは参照ファイルパス）。
