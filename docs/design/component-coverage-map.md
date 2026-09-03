# ark-ui / chakra-ui / Radix 全コンポーネント対応表

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
イシュー #937（親 #925、トラッキング #924）で ark-ui / chakra-ui の 2 参照軸へ
**Radix UI を第 3 の参照軸**として追加した。Radix 側の一次記録は
`docs/design/radix-primitives-inventory.md`（Radix Primitives、`radix-ui/website`
commit `bb42408` pin・取得日 2026-07-25）と `docs/design/radix-themes-survey.md`
（Radix Themes、取得日 2026-07-25）であり、区分判定（実装済み/実装対象/保留/
意図的非採用/参照対象外/対象外）は両書がいずれも #937 へ委ねた事実に基づき
本書 §5 で確定した。

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
| 実装対象 | Phase 3〜6（#736/#748/#757/#766 配下）または Phase 8（#932/#959 配下、イシュー #937 で新規判定した Radix 差分）のいずれかの issue で実装予定。根拠・対応 issue 列に issue 番号を記載 |
| 保留 | 実装するか否かを本書時点では確定しない。既存（ark-ui/chakra-ui 由来）の保留は `docs/policy/intentional-non-adoption.md` §7（イシュー #735）に評価軸・再評価トリガーが記録済み。イシュー #937 で新規に判定した Radix 由来の保留は本書 §9 に再評価トリガーを記す（`intentional-non-adoption.md` §7 への転記は #959 の判断に委ねる） |
| 意図的非採用 | 既に非採用と確定済み（layout プリミティブ = #716/#724、高度入力系・JS ランタイム固有 utilities・装飾系の一部・chakra `Theme` = #735（同書 §3.22〜§3.24）で確定済み、**アプリケーションロジックを内包する UI 部品（Radix `Form`）= 2026-07-25 のユーザー判断（同書 §3.25 規則 1）で確定済み**等）。再導入提案には `docs/policy/intentional-non-adoption.md` の評価軸充足確認が必須 |
| 参照対象外 | イシュー #937 で新設。Radix 側に存在するが本リポジトリの参照軸に含めない部品。対象は Radix Themes の layout プリミティブ（Box/Flex/Grid/Container/Section）と Theme provider コンポーネントの計 6 件のみ。根拠: #716/#724/#735、`docs/policy/intentional-non-adoption.md` §3.24、`docs/design/radix-themes-survey.md` §6。既存の意図的非採用決定（同 issue）を Radix 軸の文脈で再掲するものであり、新規の非採用判定ではない |
| 対象外 | README・guides・overview・get-started・concepts 等、UI コンポーネントを指さない非コンポーネント文書。加えてイシュー #735 で商用テンプレート集（chakra-ui Pro blocks）・styling / theming 概念文書を本区分へ追加確定した |

### 2.1 UI 部品の責務境界（ユーザー判断 2026-07-25、`intentional-non-adoption.md` §3.25）

区分判定にあたっては、上表の各区分に加えて次の 2 規則を適用する。詳細・
評価軸・再評価トリガーは `docs/policy/intentional-non-adoption.md` §3.25、
実装時の厳守事項は `.claude/rules/coding-rust.md` を参照する。

- **規則 1（非採用）**: UI コンポーネント層が担うのは anatomy（構造）・
  アクセシビリティ（WAI-ARIA・キーボード操作）・表示状態（`data-*`）まで。
  バリデーション・送信処理・データ整形・永続化といったアプリケーション
  ロジックを内包する部品は、参照軸に存在しても実装しない。確定対象は
  Radix Primitives `Form` の 1 件（§5 Part D・§9 の該当行を参照）。
- **規則 2（層の割り当て）**: 参照元が primitives 層へ持ち込んでいる装飾・
  アニメーション・レイアウト計測の関心（Radix の `data-motion`、viewport
  測定等）は `fandhe-frontend-headless-ui` へ持ち込まず、必要なら上層の
  `fandhe-frontend-pre-styled-ui` の責務として設計する。部品を実装対象から
  外す規則ではなく層の割り当てを定める規則であり、適用対象は Radix
  Primitives `Navigation Menu`（§5 Part D・§9 の該当行を参照）。

「保留」区分のうち ark-ui/chakra-ui 由来の評価軸・再評価トリガーの詳細記録は
イシュー #735 で `docs/policy/intentional-non-adoption.md` §7 に確定済みで
あり、本書の「根拠・対応 issue」列からは同節への参照を行う。Radix 由来の
新規保留は本書 §9 を参照する。

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

§3 で実測した全 mod が §5 の表（Part A / Part B / Part C / Part D）の fandhe
列に行として存在することは、以下の出力が空であることで確認する（§4 が
references → 本書の方向の完全性を検査するのに対し、本手順は
lib.rs → 本書の逆方向を検査する）。

```bash
tmp=$(mktemp -d)
awk '/^## 5\./,/^## 6\./' docs/design/component-coverage-map.md \
  | awk -F'|' 'NF>=11 { print $7; print $8 }' \
  | tr -c 'a-z0-9_' '\n' | sort -u > "$tmp/cols"
{ grep -E '^pub mod ' crates/headless-ui/src/lib.rs \
    | grep -vE '^pub mod (anatomy|aria|data_attrs|positioning|state|format|color|date);$'
  grep -E '^pub mod ' crates/pre-styled-ui/src/lib.rs \
    | grep -vE '^pub mod (css|recipe|stylesheet|theme);$'
  grep -E '^pub mod ' crates/pre-styled-ui/src/charts/mod.rs
} | sed 's/^pub mod //; s/;$//' | sort -u | comm -23 - "$tmp/cols"
rm -rf "$tmp"
```

- `NF>=11` は先頭・末尾の空フィールドを含む 9 列テーブル行（イシュー #937
  で Radix Primitives 名 / Radix Themes 名の 2 列を追加した後の構成）を選ぶ
  条件。`$7`/`$8` が fandhe headless-ui / pre-styled-ui 列（旧 `$5`/`$6`）に
  対応する。Part D（§5 末尾）も同じ 9 列構成にすることで、この 1 本の
  コマンドで Part A/B/C/D を一括検査できる
- `tr -c 'a-z0-9_'` により `` `charts::axis` `` のような表記は `charts` と
  `axis` に分解され、両方が計上済みとして拾われる
- 未検証・未信頼の値を一切補間しない: `$tmp` は `mktemp -d` の結果のみを
  参照しダブルクォートで囲む。`eval`・バッククォート実行・ネットワーク
  アクセスは含まない。`rm -rf "$tmp"` は `mktemp -d` が返す一時ディレクトリ
  に限定され、リポジトリ内パスを削除しうる形にはしない

### 4.3 Radix Primitives 側の完全性（イシュー #937）

`docs/design/radix-primitives-inventory.md` に pin 済みの mdx 一覧（43 件）
と、本書 §5 の Radix Primitives 列に列挙した slug 集合の差分が空であること
で確認する。ネットワーク照会を一切行わない（pin 済みローカル文書との突合
であり、Radix 側の更新検知は同書 §8/§9 の責務）。

```bash
diff <(awk '/^## 5\./,/^## 6\./' docs/design/component-coverage-map.md \
         | awk -F'|' 'NF>=11 { print $5 }' | grep -oE '`[a-z0-9-]+`' | tr -d '`' | sort -u) \
     <(grep -oE 'data/primitives/docs/[a-z-]+/[a-z0-9-]+\.mdx' \
         docs/design/radix-primitives-inventory.md \
         | sed 's|.*/||; s|\.mdx$||' | sort -u)
```

期待: 空（43 slug が過不足なく一致）。

### 4.4 Radix Themes 側の完全性（イシュー #937）

`docs/design/radix-themes-survey.md` §3 に列挙したコンポーネント slug
（56 件、`/themes/docs/components/*`）と、本書 §5 の Radix Themes 列に
列挙した slug 集合の差分が空であることで確認する。ネットワーク照会は
行わない（同上）。`/themes/docs/theme/*` の 9 ページ（トークン解説ページ）
はコンポーネントではないため本突合の対象外とする（トークン体系の対比は
`radix-themes-survey.md` §5 が担う）。

```bash
diff <(awk '/^## 5\./,/^## 6\./' docs/design/component-coverage-map.md \
         | awk -F'|' 'NF>=11 { print $6 }' | grep -oE '`[a-z0-9-]+`' | tr -d '`' | sort -u) \
     <(grep -oE '/themes/docs/components/[a-z0-9-]+' docs/design/radix-themes-survey.md \
         | sed 's|.*/||' | sort -u)
```

期待: 空（56 slug が過不足なく一致）。

### 4.5 表記規約の自己検査（イシュー #937）

§2.2 の表記規約（非 `—` セルは「表示名 + バッククォート付き slug」を
必ず 1 個だけ含む）が守られていることを、Radix Primitives 列・Radix Themes
列それぞれについて検査する。規約違反（バッククォートなしの記入・複数
slug の記入漏れ）があると §4.3/§4.4 の完全性 diff がセルを拾えず偽 PASS
になるため、以下の 2 条件を独立に確認する。

```bash
# (1) 非 — セル数とバッククォート slug 数が列ごとに一致すること
for col in 5 6; do
  awk '/^## 5\./,/^## 6\./' docs/design/component-coverage-map.md \
    | awk -F'|' -v c="$col" 'NF>=11 { v=$c; gsub(/^ +| +$/,"",v); if (v!="—" && v!~/^-+$/ && v!~/^Radix /) print v }' \
    > "/tmp/radix-col-$col-nonblank.$$"
  non_blank=$(wc -l < "/tmp/radix-col-$col-nonblank.$$")
  slug_count=$(grep -oE '`[a-z0-9-]+`' "/tmp/radix-col-$col-nonblank.$$" | wc -l)
  rm -f "/tmp/radix-col-$col-nonblank.$$"
  echo "col=$col non_blank=$non_blank slug_count=$slug_count"
done

# (2) slug の重複がないこと（列内で 1 slug = 1 行。Primitives と Themes は
#     別名前空間のため列をまたぐ同一 slug（例: 両軸に存在する `dialog`）は
#     重複とみなさない。§2.2 の「抽出は必ず列スコープで行う」規約どおり、
#     重複検査も列ごとに独立させる）
for col in 5 6; do
  awk '/^## 5\./,/^## 6\./' docs/design/component-coverage-map.md \
    | awk -F'|' -v c="$col" 'NF>=11 { print $c }' | grep -oE '`[a-z0-9-]+`' | tr -d '`' \
    | sort | uniq -d
done
```

期待: (1) 列ごとに `non_blank == slug_count`（1 セル 1 slug）、(2) 列ごとの
出力が空（列内重複なし）。`awk` の範囲指定は §4.2〜§4.4 と同じ
`/^## 5\./,/^## 6\./` に固定し、§9 の Phase 8 引き渡し表の slug が完全性
diff へ混入しないようにする（§9 は `## 5.` 〜 `## 6.` の範囲外）。

## 5. 表本体

参照ファイルパス単位で 1 md = 1 行。Part A（ark-ui）/ Part B（chakra-ui）
をディレクトリ節ごとに分割して掲載する。イシュー #937 で Part D
（Radix にのみ存在する部品）を新設し、全 Part を 9 列で統一した。

- **参照ファイル**: `.agents/skills/.../xxx.md` のフルパス（機械突合のキー）。
  Part D は対応 md を持たないため `—（対応 md なし）` を記載する
- **ark-ui 名 / chakra-ui 名**: 対応する相手側コンポーネント名。片側のみ・
  相手側に対応がない場合は `—`
- **Radix Primitives 名 / Radix Themes 名**: イシュー #937 で追加。対応する
  Radix 側コンポーネント名を「表示名 + バッククォート付き slug」の形式
  （例: `` Alert Dialog (`alert-dialog`) ``）で記載する。1 slug は列内で
  1 行にのみ出現し（重複禁止）、Primitives と Themes は別名前空間のため
  同名 slug（`dialog` 等）が両列に独立して現れうる。抽出は必ず列スコープ
  （$5/$6）で行う（§4.3〜§4.5 参照）。対応がない場合は `—`
- **fandhe headless-ui / fandhe pre-styled-ui**: 対応する mod 名。未実装は
  `—`。イシュー #937 でバッククォート表記を全行で統一した（#934 の
  積み残し）
- **区分**: §2 の 6 区分
- **根拠・対応 issue**: 実装対象は issue 番号、保留・意図的非採用は根拠概要
  （詳細はイシュー #735 で確定した `docs/policy/intentional-non-adoption.md`
  §3.22〜§3.24・§7、または既存記録 #716/#724）。意図的非採用の等価概念・
  代替は本書 §8 の対応表（イシュー #855）を参照。イシュー #937 で新規判定
  した実装対象・保留の根拠・再評価トリガーは本書 §9 を参照

### Part A: ark-ui（`.agents/skills/ark-ui/references/`、90 件）

#### `.agents/skills/ark-ui/references/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/collections/async-list.md` | AsyncListCollection | — | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/collections/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/list-collection.md` | ListCollection | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/list-selection.md` | ListSelection | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/tree-collection.md` | TreeCollection | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/collections/menu.md` | Menu | Menu | Dropdown Menu (`dropdown-menu`) | Dropdown Menu (`dropdown-menu`) | `menu` | `menu` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/select.md` | Select | Select | Select (`select`) | Select (`select`) | `select` | `select` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/combobox.md` | Combobox | Combobox | — | — | `combobox` | `combobox` | 実装済み | headless+styled 実装済み（#749、PR #793） |
| `.agents/skills/ark-ui/references/components/collections/listbox.md` | Listbox | Listbox | — | — | `listbox` | `listbox` | 実装済み | headless+styled 実装済み（#750） |
| `.agents/skills/ark-ui/references/components/collections/pagination.md` | Pagination | Pagination | — | — | `pagination` | `pagination` | 実装済み | headless+styled 実装済み（#751、PR #796、#716 保留の解除） |
| `.agents/skills/ark-ui/references/components/collections/steps.md` | Steps | Steps | — | — | `steps` | `steps` | 実装済み | headless+styled 実装済み（#752、#716 保留の解除） |
| `.agents/skills/ark-ui/references/components/collections/tree-view.md` | TreeView | TreeView | — | — | `tree_view` | `tree_view` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/carousel.md` | Carousel | Carousel | — | — | `carousel` | `carousel` | 実装済み | headless+styled 実装済み（#754）。autoplay（play/pause/aria-live 切替/delay）は初期実装スコープ外（`crates/headless-ui/src/carousel.rs` module doc 参照） |
| `.agents/skills/ark-ui/references/components/collections/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/date-time/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/date-time/date-input.md` | DateInput | — | — | — | `date_input` | `date_input` | 実装済み | headless+styled 実装済み（#834、#735 保留のうち DateInput 分のみ解除。DatePicker（#835）・Timer（#836）も別途保留解除済み（下記行参照）。calendar は独立部品として実装済み） |
| `.agents/skills/ark-ui/references/components/date-time/date-picker.md` | DatePicker | DatePicker | — | — | `date_picker` | `date_picker` | 実装済み | headless+styled 実装済み（#835、親トラッキング #832。`docs/policy/intentional-non-adoption.md` §7（#735）の保留解除。DateInput（#834）との連携は行わず ISO 8601 値のネイティブ `<input>` のみで完結） |
| `.agents/skills/ark-ui/references/components/date-time/timer.md` | Timer | Timer | — | — | `timer` | `timer` | 実装済み | headless+styled+wasm 配線実装済み（#836）。tick を外部から明示的に注入する決定的状態機械（時計 API 非依存）として実装し、`docs/policy/intentional-non-adoption.md` §7 の保留を解除した |
| `.agents/skills/ark-ui/references/components/date-time/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/disclosure/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/disclosure/accordion.md` | Accordion | Accordion | Accordion (`accordion`) | — | `accordion` | `accordion` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/disclosure/collapsible.md` | Collapsible | Collapsible | Collapsible (`collapsible`) | — | `collapsible` | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/disclosure/tabs.md` | Tabs | Tabs | Tabs (`tabs`) | Tabs (`tabs`) | `tabs` | `tabs` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/disclosure/toggle.md` | Toggle | — | Toggle (`toggle`) | — | `toggle` | `toggle` | 実装済み | headless+styled 実装済み（#746、PR #791） |
| `.agents/skills/ark-ui/references/components/disclosure/toggle-group.md` | ToggleGroup | — | Toggle Group (`toggle-group`) | — | `toggle_group` | `toggle_group` | 実装済み | headless+styled 実装済み（#746、PR #791） |
| `.agents/skills/ark-ui/references/components/disclosure/scroll-area.md` | ScrollArea | ScrollArea | Scroll Area (`scroll-area`) | Scroll Area (`scroll-area`) | `scroll_area` | `scroll_area` | 実装済み | headless+styled 実装済み（#825、保留解除。JS によるスクロール位置追従・thumb drag は本イシューのスコープ外） |
| `.agents/skills/ark-ui/references/components/disclosure/splitter.md` | Splitter | Splitter | — | — | `splitter` | `splitter` | 実装済み | headless+styled 実装済み（#826、#735 保留の解除） |
| `.agents/skills/ark-ui/references/components/disclosure/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/display/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/display/avatar.md` | Avatar | Avatar | Avatar (`avatar`) | Avatar (`avatar`) | `avatar` | `avatar` | 実装済み | headless+styled 実装済み（#731 MutationObserver 対応込み） |
| `.agents/skills/ark-ui/references/components/display/progress-linear.md` | Progress (linear) | Progress | Progress (`progress`) | Progress (`progress`) | `progress` | `progress` | 実装済み | headless+styled（root/range）実装済み。#1564 で linear（Track/Range）styled CSS・`ProgressVariant`/`ColorPalette` 軸を新設し pre-styled ラッパー未実装状態を解消 |
| `.agents/skills/ark-ui/references/components/display/progress-circular.md` | Progress (circular) | ProgressCircle | — | — | `progress` | `progress` | 実装済み | #763（既存 progress mod を circular 対応へ拡張。headless は #600 で実装済み、pre-styled ラッパーを #763 で追加） |
| `.agents/skills/ark-ui/references/components/display/clipboard.md` | Clipboard | Clipboard | — | — | `clipboard` | `clipboard` | 実装済み | headless+styled+wasm 配線 実装済み（#773、PR #816） |
| `.agents/skills/ark-ui/references/components/display/qr-code.md` | QrCode | QrCode | — | — | `qr_code` | `qr_code` | 実装済み | headless+styled 実装済み（#774） |
| `.agents/skills/ark-ui/references/components/display/marquee.md` | Marquee | Marquee | — | — | — | `marquee` | 実装済み（再導入） | #831 で `docs/policy/intentional-non-adoption.md` §3.24 の再評価トリガー 1（CSS のみ・`prefers-reduced-motion` 対応の決定的設計案）を充足し再導入（CSS のみ・JS ゼロ）。headless-ui は変更なし、pre-styled-ui 層のみで新規 anatomy を定義 |
| `.agents/skills/ark-ui/references/components/display/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/form/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/form/checkbox.md` | Checkbox | Checkbox | Checkbox (`checkbox`) | Checkbox (`checkbox`) | `checkbox` | `checkbox` | 実装済み | headless+styled 実装済み（#730） |
| `.agents/skills/ark-ui/references/components/form/field.md` | Field | Field | Label (`label`) | — | `field` | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/form/fieldset.md` | Fieldset | Fieldset | — | — | `fieldset` | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/form/radio-group.md` | RadioGroup | Radio | Radio Group (`radio-group`) | Radio Group (`radio-group`) | `radio_group` | `radio_group` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/form/switch.md` | Switch | Switch | Switch (`switch`) | Switch (`switch`) | `switch` | `switch` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/form/number-input.md` | NumberInput | NumberInput | — | — | `number_input` | `number_input` | 実装済み | headless+styled 実装済み（#738、PR #785） |
| `.agents/skills/ark-ui/references/components/form/pin-input.md` | PinInput | PinInput | One-Time Password Field (`one-time-password-field`) | — | `pin_input` | `pin_input` | 実装済み | headless+styled 実装済み（#739、PR #784） |
| `.agents/skills/ark-ui/references/components/form/password-input.md` | PasswordInput | PasswordInput | Password Toggle Field (`password-toggle-field`) | — | `password_input` | `password_input` | 実装済み | headless+styled 実装済み（#740） |
| `.agents/skills/ark-ui/references/components/form/slider.md` | Slider | Slider | Slider (`slider`) | Slider (`slider`) | `slider` | `slider` | 実装済み | headless+styled 実装済み（#741） |
| `.agents/skills/ark-ui/references/components/form/rating-group.md` | RatingGroup | Rating | — | — | `rating_group` | `rating_group` | 実装済み | headless+styled 実装済み（#742） |
| `.agents/skills/ark-ui/references/components/form/segment-group.md` | SegmentGroup | SegmentedControl | — | Segmented Control (`segmented-control`) | `segment_group` | `segment_group` | 実装済み | headless+styled 実装済み（#743） |
| `.agents/skills/ark-ui/references/components/form/tags-input.md` | TagsInput | TagsInput | — | — | `tags_input` | `tags_input` | 実装済み | headless+styled 実装済み（#744） |
| `.agents/skills/ark-ui/references/components/form/editable.md` | Editable | Editable | — | — | `editable` | `editable` | 実装済み | headless+styled 実装済み（#745） |
| `.agents/skills/ark-ui/references/components/form/angle-slider.md` | AngleSlider | AngleSlider | — | — | `angle_slider` | `angle_slider` | 実装済み（再導入） | #842 で `docs/policy/intentional-non-adoption.md` §3.22 の再評価トリガー 1（決定的自動テスト基盤の確立・具体的ユースケースを伴う利用要望）を充足し再導入。座標→角度変換（`atan2`）を wasm-full 層の単一純粋関数へ隔離、headless 層は整数角度状態機械のみ |
| `.agents/skills/ark-ui/references/components/form/color-picker.md` | ColorPicker | ColorPicker | — | — | `color_picker` | `color_picker` | 実装済み | headless+styled 実装済み（#839、親 #837）。canvas 非依存（CSS グラデーション + 導出整数割合）で `docs/policy/intentional-non-adoption.md` §7 再評価トリガー充足、保留解除 |
| `.agents/skills/ark-ui/references/components/form/file-upload.md` | FileUpload | FileUpload | — | — | `file_upload` | `file_upload` | 実装済み | headless+styled+wasm 実装済み（#840、`docs/policy/intentional-non-adoption.md` §7 保留解除。ItemPreview/ItemPreviewImage はスコープ外） |
| `.agents/skills/ark-ui/references/components/form/image-cropper.md` | ImageCropper | — | — | — | `image_cropper` | `image_cropper` | 実装済み | headless+styled 実装済み（#844、再導入）。crop 矩形（整数）のみを扱う決定的状態機械として §4 手続きに従い再導入（`docs/policy/intentional-non-adoption.md` §3.22 参照）。canvas による実画像切り出し・pointer ドラッグ配線は対象外（後続 issue） |
| `.agents/skills/ark-ui/references/components/form/signature-pad.md` | SignaturePad | — | — | — | `signature_pad` | `signature_pad` | 実装済み | canvas を使わない決定的 SVG path 方式で再導入（#843）。headless+styled+wasm 配線済み。非採用の再導入手続きは `docs/policy/intentional-non-adoption.md` §3.22 追補（#735/#843）参照。canvas 方式・残り部品（AngleSlider/RichTextEditor）の非採用判断は不変 |
| `.agents/skills/ark-ui/references/components/form/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/overlays/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/overlays/dialog.md` | Dialog | Dialog | Dialog (`dialog`) | Dialog (`dialog`) | `dialog` | `dialog` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/popover.md` | Popover | Popover | Popover (`popover`) | Popover (`popover`) | `popover` | `popover` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/tooltip.md` | Tooltip | Tooltip | Tooltip (`tooltip`) | Tooltip (`tooltip`) | `tooltip` | `tooltip` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/drawer.md` | Drawer | Drawer | — | — | `drawer` | `drawer` | 実装済み | headless+styled 実装済み（#758、dialog の状態機械を再利用） |
| `.agents/skills/ark-ui/references/components/overlays/hover-card.md` | HoverCard | HoverCard | Hover Card (`hover-card`) | Hover Card (`hover-card`) | `hover_card` | `hover_card` | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/toast.md` | Toast | Toast | Toast (`toast`) | — | `toast` | `toast` | 実装済み | headless+styled 実装済み（#760、キュー状態機械は `Disclosure`/`SingleSelect` に収まらないため `Component`/`Hydrate` 直接実装） |
| `.agents/skills/ark-ui/references/components/overlays/floating-panel.md` | FloatingPanel | FloatingPanel | — | — | `floating_panel` | `floating_panel` | 実装済み | headless+styled 実装済み（イシュー #827、`docs/policy/intentional-non-adoption.md` §7 の保留区分から解除） |
| `.agents/skills/ark-ui/references/components/overlays/tour.md` | Tour | Tour | — | — | `tour` | `tour` | 実装済み | headless+styled 実装済み（#841、#735 保留の解除）。決定的な状態機械・SSR 出力のみが対象で、対象要素の実座標追従・スクロール/リサイズ再計算・target セレクタの実解決は `fandhe-frontend-wasm-full` の後続イシューのスコープ |
| `.agents/skills/ark-ui/references/components/overlays/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/guides/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/guides/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/animation.md` | Animation | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/component-state.md` | ComponentState | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/composition.md` | Composition | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/forms.md` | Forms | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/ref.md` | Ref | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/guides/styling.md` | Styling | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/overview/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/overview/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/about.md` | About | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/changelog.md` | Changelog | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/framework-differences.md` | FrameworkDifferences | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/getting-started.md` | GettingStarted | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/llms-txt.md` | LlmsTxt | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/overview/mcp-server.md` | McpServer | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/utilities/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/utilities/highlight.md` | Highlight | Highlight | — | — | — | `highlight` | 実装済み | #775。pre-styled 静的部品 実装済み |
| `.agents/skills/ark-ui/references/utilities/client-only.md` | ClientOnly | ClientOnly | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/download-trigger.md` | DownloadTrigger | DownloadTrigger | — | — | `download_trigger` | `download_trigger` | 実装済み | #828。保留（#735 §7「JS ランタイム固有 utilities のうち静的実装可能なもの」）を利用要望 issue（#828）の起票により解除。`a[download]` 属性による静的部品として実装（`Blob`/`data`/`mimeType` は JS 前提のため対応しない） |
| `.agents/skills/ark-ui/references/utilities/environment.md` | Environment | EnvironmentProvider | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/focus-trap.md` | FocusTrap | — | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/format-byte.md` | FormatByte | FormatByte | — | — | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_byte`（Intl 非依存の決定的純関数） |
| `.agents/skills/ark-ui/references/utilities/format-number.md` | FormatNumber | FormatNumber | — | — | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_number`（Intl 非依存の決定的純関数） |
| `.agents/skills/ark-ui/references/utilities/format-relative-time.md` | FormatRelativeTime | — | — | — | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_relative_time`（基準時刻は呼び出し側注入、現在時刻 API 非依存） |
| `.agents/skills/ark-ui/references/utilities/format-time.md` | FormatTime | — | — | — | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_time`（決定的純関数） |
| `.agents/skills/ark-ui/references/utilities/frame.md` | Frame | — | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/json-tree-view.md` | JsonTreeView | — | — | — | `json_tree_view` | `json_tree_view` | 実装済み | **保留解除**（イシュー #829、`tree_view`（#753）の派生として実装。headless `crates/headless-ui/src/json_tree_view.rs` + styled `crates/pre-styled-ui/src/json_tree_view.rs`。`docs/policy/intentional-non-adoption.md` §7 の解除記録参照） |
| `.agents/skills/ark-ui/references/utilities/locale.md` | Locale | LocaleProvider | — | — | — | — | 実装済み（Rust 最適化形） | イシュー #854。`Locale` 値型（`crates/headless-ui/src/format.rs` の `format` mod、en/ja）として実装。`LocaleProvider` の Context/Provider 機構は非採用のまま（`docs/policy/intentional-non-adoption.md` §3.23 参照） |
| `.agents/skills/ark-ui/references/utilities/presence.md` | Presence | Presence | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/swap.md` | Swap | — | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/ark-ui/references/utilities/README.md` | README | — | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

### Part B: chakra-ui（`.agents/skills/chakra-ui/references/`、269 件）

#### `.agents/skills/chakra-ui/references/blocks/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/blocks/README.md` | — | README | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/blocks/ai.md` | — | Ai | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/app-headers.md` | — | AppHeaders | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/app-integrations.md` | — | AppIntegrations | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/app-navbars.md` | — | AppNavbars | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/authentication.md` | — | Authentication | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/banners.md` | — | Banners | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/blogs.md` | — | Blogs | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/cards.md` | — | Cards | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/careers.md` | — | Careers | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/charts.md` | — | Charts | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/contacts.md` | — | Contacts | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/cta.md` | — | Cta | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/dividers.md` | — | Dividers | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-changelog.md` | — | DocsChangelog | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-code-block.md` | — | DocsCodeBlock | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-example-preview.md` | — | DocsExamplePreview | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-header.md` | — | DocsHeader | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-navbar.md` | — | DocsNavbar | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-pagination.md` | — | DocsPagination | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-parameter-field.md` | — | DocsParameterField | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-sidebar.md` | — | DocsSidebar | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-step.md` | — | DocsStep | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/docs-toc.md` | — | DocsToc | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/faqs.md` | — | Faqs | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/features.md` | — | Features | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/feeds.md` | — | Feeds | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/footers.md` | — | Footers | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/help-center.md` | — | HelpCenter | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/heroes.md` | — | Heroes | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/layouts.md` | — | Layouts | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/logos.md` | — | Logos | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/marketing-headers.md` | — | MarketingHeaders | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/marketing-navbars.md` | — | MarketingNavbars | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/notifications.md` | — | Notifications | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/onboarding.md` | — | Onboarding | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/org-switcher.md` | — | OrgSwitcher | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/pricing.md` | — | Pricing | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/product-categories.md` | — | ProductCategories | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/product-grid.md` | — | ProductGrid | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/product-reviews.md` | — | ProductReviews | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/profiles.md` | — | Profiles | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/property-panels.md` | — | PropertyPanels | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/settings.md` | — | Settings | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/sharing.md` | — | Sharing | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/shopping-carts.md` | — | ShoppingCarts | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/sidebars.md` | — | Sidebars | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/stats.md` | — | Stats | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/store-signup-offers.md` | — | StoreSignupOffers | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/teams.md` | — | Teams | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/testimonials.md` | — | Testimonials | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |
| `.agents/skills/chakra-ui/references/blocks/webhooks.md` | — | Webhooks | — | — | — | — | 対象外 | （#735、chakra-ui Pro の商用テンプレート集。§2 の対象外定義拡張） |

#### `.agents/skills/chakra-ui/references/charts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/charts/README.md` | — | README | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/charts/area-chart.md` | — | AreaChart | — | — | — | `area_chart` | 実装済み | 保留解除。#848（軸/グリッド/凡例/ツールチップ/積み上げは #847 以降のスコープ） |
| `.agents/skills/chakra-ui/references/charts/axes.md` | — | Axes | — | — | — | `charts::axis`（`y_axis`/`x_axis_linear`/`x_axis_categories`） | 実装済み | #847、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/bar-chart.md` | — | BarChart | — | — | — | `charts::bar_chart` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy `bar-chart` を定義）。#849（親 Phase #845、charts 基盤 #846 の上に実装）。`docs/policy/intentional-non-adoption.md` §7 の保留を解除。軸線・グリッド・凡例・ツールチップは #847 のスコープ |
| `.agents/skills/chakra-ui/references/charts/bar-list.md` | — | BarList | — | — | — | `charts::bar_list` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy `bar-list` を定義）。#849（親 Phase #845）。`docs/policy/intentional-non-adoption.md` §7 の保留を解除 |
| `.agents/skills/chakra-ui/references/charts/bar-segment.md` | — | BarSegment | — | — | — | `charts::bar_segment` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy `bar-segment` を定義）。#849（親 Phase #845）。`docs/policy/intentional-non-adoption.md` §7 の保留を解除 |
| `.agents/skills/chakra-ui/references/charts/cartesian-grid.md` | — | CartesianGrid | — | — | — | `charts::grid`（`cartesian_grid`） | 実装済み | #847、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/donut-chart.md` | — | DonutChart | — | — | — | `donut_chart` | 実装済み | 保留解除。#850、charts 基盤（#846）を用いたドーナツグラフ、詳細は `crates/pre-styled-ui/src/donut_chart.rs` rustdoc |
| `.agents/skills/chakra-ui/references/charts/installation.md` | — | Installation | — | — | — | `charts`（外部依存追加なし、`fandhe-frontend-pre-styled-ui` のみで完結） | 実装済み | 保留解除（基盤のみ）。#846、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/legend.md` | — | Legend | — | — | — | `charts::legend`（`legend`） | 実装済み | #847、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/line-chart.md` | — | LineChart | — | — | — | `line_chart` | 実装済み | 保留解除。#848（軸/グリッド/凡例/ツールチップ/積み上げは #847 以降のスコープ） |
| `.agents/skills/chakra-ui/references/charts/pie-chart.md` | — | PieChart | — | — | — | `pie_chart` | 実装済み | 保留解除。#850、charts 基盤（#846）を用いた円グラフ、詳細は `crates/pre-styled-ui/src/pie_chart.rs` rustdoc |
| `.agents/skills/chakra-ui/references/charts/radar-chart.md` | — | RadarChart | — | — | — | `charts::radar_chart` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy）。#851（親 Phase #845、charts 基盤 #846 の上に実装）。保留解除 |
| `.agents/skills/chakra-ui/references/charts/scatter-chart.md` | — | ScatterChart | — | — | — | `charts::scatter_chart` | 実装済み | headless-ui 非経由（styled 直下で新規 anatomy）。#851（親 Phase #845、charts 基盤 #846 の上に実装）。保留解除 |
| `.agents/skills/chakra-ui/references/charts/sparkline.md` | — | Sparkline | — | — | — | `sparkline` | 実装済み | 保留解除。#848（単一系列専用。複数系列は LineChart/AreaChart を使用） |
| `.agents/skills/chakra-ui/references/charts/tooltip.md` | — | Tooltip | — | — | — | `charts::tooltip`（`datum`/`datum_label`。汎用 headless Tooltip（`tooltip` モジュール）とは別物） | 実装済み | #847、詳細は `docs/design/charts-foundation-design.md` |
| `.agents/skills/chakra-ui/references/charts/use-chart.md` | — | UseChart | — | — | — | `charts`（`ChartData`/`LinearScale`/SVG ヘルパー） | 実装済み | 保留解除（基盤のみ）。#846、詳細は `docs/design/charts-foundation-design.md` |

#### `.agents/skills/chakra-ui/references/components/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/README.md` | — | README | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/chakra-ui/references/components/buttons/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/buttons/button.md` | — | Button | — | Button (`button`) | — | `button` | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/buttons/download-trigger.md` | DownloadTrigger | DownloadTrigger | — | — | `download_trigger` | `download_trigger` | 実装済み | #828。保留（#735 §7「JS ランタイム固有 utilities のうち静的実装可能なもの」）を利用要望 issue（#828）の起票により解除。`a[download]` 属性による静的部品として実装（`Blob`/`data`/`mimeType` は JS 前提のため対応しない） |
| `.agents/skills/chakra-ui/references/components/buttons/close-button.md` | — | CloseButton | — | — | — | `button`（`close_button`） | 実装済み | #830。保留（Button バリエーション、#735 §7）を `Button` variant 拡張要望 issue（#830）の起票により解除。独立部品ではなく `button` recipe の icon-only 修飾 variant として実装（`data-scope="button"` を共有） |
| `.agents/skills/chakra-ui/references/components/buttons/icon-button.md` | — | IconButton | — | Icon Button (`icon-button`) | — | `button`（`icon_button`） | 実装済み | #830。close-button と同一の解除・実装判断（同上） |

#### `.agents/skills/chakra-ui/references/components/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/collections/select.md` | Select | Select | — | — | `select` | `select` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/collections/combobox.md` | Combobox | Combobox | — | — | `combobox` | `combobox` | 実装済み | headless+styled 実装済み（#749、PR #793） |
| `.agents/skills/chakra-ui/references/components/collections/listbox.md` | Listbox | Listbox | — | — | `listbox` | `listbox` | 実装済み | headless+styled 実装済み（#750） |
| `.agents/skills/chakra-ui/references/components/collections/tree-view.md` | TreeView | TreeView | — | — | `tree_view` | `tree_view` | 実装済み | headless+styled 実装済み |

#### `.agents/skills/chakra-ui/references/components/concepts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/concepts/animation.md` | — | Animation | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/color-mode.md` | — | ColorMode | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/composition.md` | — | Composition | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/overview.md` | — | Overview | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/server-components.md` | — | ServerComponents | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/components/concepts/testing.md` | — | Testing | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/chakra-ui/references/components/data-display/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/data-display/avatar.md` | Avatar | Avatar | — | — | `avatar` | `avatar` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/data-display/badge.md` | — | Badge | — | Badge (`badge`) | — | `badge` | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/data-display/card.md` | — | Card | — | Card (`card`) | — | `card` | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/data-display/table.md` | — | Table | — | Table (`table`) | — | `table` | 実装済み | pre-styled 静的部品 実装済み（#767。`stickyHeader` は #1571 で実装済み。`interactive`/`showColumnBorder`/`ScrollArea`/`ColumnGroup` はスコープ外） |
| `.agents/skills/chakra-ui/references/components/data-display/data-list.md` | — | DataList | — | Data List (`data-list`) | — | `data_list` | 実装済み | pre-styled 静的部品 実装済み（#767。`variant`（subtle/bold）/`size` variant は #1559 で追加済み） |
| `.agents/skills/chakra-ui/references/components/data-display/tag.md` | — | Tag | — | — | — | `tag` | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/data-display/stat.md` | — | Stat | — | — | — | `stat` | 実装済み | pre-styled 静的部品 実装済み（#769。headless-ui は変更なし） |
| `.agents/skills/chakra-ui/references/components/data-display/timeline.md` | — | Timeline | — | — | — | `timeline` | 実装済み | pre-styled 静的部品 実装済み（#769。headless-ui は変更なし） |
| `.agents/skills/chakra-ui/references/components/data-display/image.md` | — | Image | — | — | — | `image` | 実装済み | #770。状態機械なし静的部品、pre-styled 層のみに実装（headless-ui は変更なし）。fit（object-fit）/aspect-ratio の 2 軸 variant、alt 必須引数 |
| `.agents/skills/chakra-ui/references/components/data-display/icon.md` | — | Icon | — | — | — | `icon` | 実装済み | #770。状態機械なし静的部品、pre-styled 層のみに実装（headless-ui は変更なし）。size variant のみ、SVG 本体は呼び出し側がノード木 API で構築 |
| `.agents/skills/chakra-ui/references/components/data-display/clipboard.md` | Clipboard | Clipboard | — | — | `clipboard` | `clipboard` | 実装済み | headless+styled+wasm 配線 実装済み（#773、PR #816） |
| `.agents/skills/chakra-ui/references/components/data-display/qr-code.md` | QrCode | QrCode | — | — | `qr_code` | `qr_code` | 実装済み | headless+styled 実装済み（#774） |
| `.agents/skills/chakra-ui/references/components/data-display/marquee.md` | Marquee | Marquee | — | — | — | `marquee` | 実装済み（再導入） | #831 で `docs/policy/intentional-non-adoption.md` §3.24 の再評価トリガー 1（CSS のみ・`prefers-reduced-motion` 対応の決定的設計案）を充足し再導入（CSS のみ・JS ゼロ）。headless-ui は変更なし、pre-styled-ui 層のみで新規 anatomy を定義 |

#### `.agents/skills/chakra-ui/references/components/date-time/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/date-time/date-picker.md` | DatePicker | DatePicker | — | — | `date_picker` | `date_picker` | 実装済み | headless+styled 実装済み（#835、親トラッキング #832。`docs/policy/intentional-non-adoption.md` §7（#735）の保留解除） |
| `.agents/skills/chakra-ui/references/components/date-time/calendar.md` | — | Calendar | — | — | `calendar` | `calendar` | 実装済み | headless+styled 実装済み（#835、親トラッキング #832。`docs/policy/intentional-non-adoption.md` §7（#735）の保留解除） |

#### `.agents/skills/chakra-ui/references/components/disclosure/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/disclosure/accordion.md` | Accordion | Accordion | — | — | `accordion` | `accordion` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/disclosure/collapsible.md` | Collapsible | Collapsible | — | — | `collapsible` | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/chakra-ui/references/components/disclosure/tabs.md` | Tabs | Tabs | — | — | `tabs` | `tabs` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/disclosure/pagination.md` | Pagination | Pagination | — | — | `pagination` | `pagination` | 実装済み | headless+styled 実装済み（#751、PR #796、#716 保留の解除） |
| `.agents/skills/chakra-ui/references/components/disclosure/steps.md` | Steps | Steps | — | — | `steps` | `steps` | 実装済み | headless+styled 実装済み（#752、#716 保留の解除） |
| `.agents/skills/chakra-ui/references/components/disclosure/carousel.md` | Carousel | Carousel | — | — | `carousel` | `carousel` | 実装済み | headless+styled 実装済み（#754）。autoplay（play/pause/aria-live 切替/delay）は初期実装スコープ外（`crates/headless-ui/src/carousel.rs` module doc 参照） |
| `.agents/skills/chakra-ui/references/components/disclosure/breadcrumb.md` | — | Breadcrumb | — | — | `breadcrumb` | `breadcrumb` | 実装済み | #755（#716 追加候補の消化）。headless+styled 実装済み |

#### `.agents/skills/chakra-ui/references/components/feedback/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/feedback/progress.md` | Progress (linear) | Progress | — | — | `progress` | `progress` | 実装済み | headless+styled（root/range）実装済み。#1564 で linear（Track/Range）styled CSS・`ProgressVariant`/`ColorPalette` 軸を新設し pre-styled ラッパー未実装状態を解消 |
| `.agents/skills/chakra-ui/references/components/feedback/alert.md` | — | Alert | — | — | — | `alert` | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/feedback/spinner.md` | — | Spinner | — | Spinner (`spinner`) | — | `spinner` | 実装済み | pre-styled 静的部品 実装済み。#1567 でスタイルを参考サイト基準へ調整（半円弧・トラック透明既定・size 5 段を chakra 一致・reduced-motion 停止） |
| `.agents/skills/chakra-ui/references/components/feedback/toast.md` | Toast | Toast | — | — | `toast` | `toast` | 実装済み | headless+styled 実装済み（#760） |
| `.agents/skills/chakra-ui/references/components/feedback/progress-circle.md` | Progress (circular) | ProgressCircle | — | — | `progress` | `progress` | 実装済み | #763（既存 progress mod を circular 対応へ拡張。headless は #600 で実装済み、pre-styled ラッパーを #763 で追加） |
| `.agents/skills/chakra-ui/references/components/feedback/skeleton.md` | — | Skeleton | — | Skeleton (`skeleton`) | — | `skeleton` | 実装済み | #764。pre-styled 静的部品 実装済み。#1566 でスタイルを参考サイト基準へ調整（`bg-emphasized` 背景・`animation` 軸追加） |
| `.agents/skills/chakra-ui/references/components/feedback/status.md` | — | Status | — | — | — | `status` | 実装済み | pre-styled 静的部品 実装済み（#765） |
| `.agents/skills/chakra-ui/references/components/feedback/empty-state.md` | — | EmptyState | — | — | — | `empty_state` | 実装済み | pre-styled 静的部品 実装済み（#765） |

#### `.agents/skills/chakra-ui/references/components/forms/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/forms/checkbox.md` | Checkbox | Checkbox | — | — | `checkbox` | `checkbox` | 実装済み | headless+styled 実装済み（#730） |
| `.agents/skills/chakra-ui/references/components/forms/field.md` | Field | Field | — | — | `field` | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/chakra-ui/references/components/forms/fieldset.md` | Fieldset | Fieldset | — | — | `fieldset` | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/chakra-ui/references/components/forms/radio.md` | RadioGroup | Radio | — | — | `radio_group` | `radio_group` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/forms/switch.md` | Switch | Switch | — | — | `switch` | `switch` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/forms/input.md` | — | Input | — | Text Field (`text-field`) | — | `input` | 実装済み | pre-styled 静的部品として実装済み（#737） |
| `.agents/skills/chakra-ui/references/components/forms/textarea.md` | — | Textarea | — | Text Area (`text-area`) | — | `textarea` | 実装済み | pre-styled 静的部品として実装済み（#737） |
| `.agents/skills/chakra-ui/references/components/forms/native-select.md` | — | NativeSelect | — | — | — | `native_select` | 実装済み | pre-styled 静的部品として実装済み（#737） |
| `.agents/skills/chakra-ui/references/components/forms/number-input.md` | NumberInput | NumberInput | — | — | `number_input` | `number_input` | 実装済み | headless+styled 実装済み（#738、PR #785） |
| `.agents/skills/chakra-ui/references/components/forms/pin-input.md` | PinInput | PinInput | — | — | `pin_input` | `pin_input` | 実装済み | headless+styled 実装済み（#739、PR #784） |
| `.agents/skills/chakra-ui/references/components/forms/password-input.md` | PasswordInput | PasswordInput | — | — | `password_input` | `password_input` | 実装済み | headless+styled 実装済み（#740） |
| `.agents/skills/chakra-ui/references/components/forms/slider.md` | Slider | Slider | — | — | `slider` | `slider` | 実装済み | headless+styled 実装済み（#741） |
| `.agents/skills/chakra-ui/references/components/forms/rating.md` | RatingGroup | Rating | — | — | `rating_group` | `rating_group` | 実装済み | headless+styled 実装済み（#742） |
| `.agents/skills/chakra-ui/references/components/forms/segmented-control.md` | SegmentGroup | SegmentedControl | — | — | `segment_group` | `segment_group` | 実装済み | headless+styled 実装済み（#743） |
| `.agents/skills/chakra-ui/references/components/forms/tags-input.md` | TagsInput | TagsInput | — | — | `tags_input` | `tags_input` | 実装済み | headless+styled 実装済み（#744） |
| `.agents/skills/chakra-ui/references/components/forms/editable.md` | Editable | Editable | — | — | `editable` | `editable` | 実装済み | headless+styled 実装済み（#745） |
| `.agents/skills/chakra-ui/references/components/forms/checkbox-card.md` | — | CheckboxCard | — | Checkbox Cards (`checkbox-cards`) | — | `checkbox_card` | 実装済み | pre-styled styled バリエーション実装済み（#747。headless-ui は変更なし、状態機械は headless Checkbox を再利用） |
| `.agents/skills/chakra-ui/references/components/forms/radio-card.md` | — | RadioCard | — | Radio Cards (`radio-cards`) | — | `radio_card` | 実装済み | pre-styled styled バリエーション実装済み（#747。headless-ui は変更なし、状態機械は headless RadioGroup を再利用） |
| `.agents/skills/chakra-ui/references/components/forms/color-picker.md` | ColorPicker | ColorPicker | — | — | `color_picker` | `color_picker` | 実装済み | headless+styled 実装済み（#839、親 #837）。canvas 非依存（CSS グラデーション + 導出整数割合）で `docs/policy/intentional-non-adoption.md` §7 再評価トリガー充足、保留解除 |
| `.agents/skills/chakra-ui/references/components/forms/color-swatch.md` | — | ColorSwatch | — | — | — | `color_swatch` | 実装済み | pre-styled 静的部品として実装済み（#838。headless-ui には対応する anatomy を新設しない。色変換コアは `fandhe-frontend-headless-ui::color`、親 #837） |
| `.agents/skills/chakra-ui/references/components/forms/file-upload.md` | FileUpload | FileUpload | — | — | `file_upload` | `file_upload` | 実装済み | headless+styled+wasm 実装済み（#840、`docs/policy/intentional-non-adoption.md` §7 保留解除。ItemPreview/ItemPreviewImage はスコープ外） |

#### `.agents/skills/chakra-ui/references/components/i18n/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/i18n/format-byte.md` | FormatByte | FormatByte | — | — | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_byte`（Intl 非依存の決定的純関数） |
| `.agents/skills/chakra-ui/references/components/i18n/format-number.md` | FormatNumber | FormatNumber | — | — | `format` | — | 実装済み | #853。`docs/policy/intentional-non-adoption.md` §3.23 の非採用から区分変更。`fandhe-frontend-headless-ui::format::format_number`（Intl 非依存の決定的純関数） |
| `.agents/skills/chakra-ui/references/components/i18n/locale-provider.md` | Locale | LocaleProvider | — | — | — | — | 実装済み（Rust 最適化形） | イシュー #854。`Locale` 値型（`format` mod、en/ja）として実装。`LocaleProvider` の Context/Provider 機構・グローバル既定ロケールは意図的に非採用のまま（`docs/policy/intentional-non-adoption.md` §3.23 参照） |

#### `.agents/skills/chakra-ui/references/components/layout/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/layout/separator.md` | — | Separator | Separator (`separator`) | Separator (`separator`) | — | `separator` | 実装済み | #772。pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/layout/scroll-area.md` | ScrollArea | ScrollArea | — | — | `scroll_area` | `scroll_area` | 実装済み | headless+styled 実装済み（#825、保留解除。JS によるスクロール位置追従・thumb drag は本イシューのスコープ外） |
| `.agents/skills/chakra-ui/references/components/layout/splitter.md` | Splitter | Splitter | — | — | `splitter` | `splitter` | 実装済み | headless+styled 実装済み（#826、#735 保留の解除） |
| `.agents/skills/chakra-ui/references/components/layout/absolute-center.md` | — | AbsoluteCenter | — | — | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/aspect-ratio.md` | — | AspectRatio | Aspect Ratio (`aspect-ratio`) | Aspect Ratio (`aspect-ratio`) | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/bleed.md` | — | Bleed | — | — | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/box.md` | — | Box | — | Box (`box`) | — | — | 参照対象外 | #716/#724 で非採用確定済み（layout プリミティブ）。（#937）Radix Themes 側にも同一概念が存在するが、layout プリミティブ / Theme provider は本リポジトリの参照軸に含めない「参照対象外」（根拠: #716/#724/#735、`docs/policy/intentional-non-adoption.md` §3.24、`docs/design/radix-themes-survey.md` §6）。既存の非採用決定（同 issue）を Radix 軸の文脈で再掲するものであり、新規の非採用判定ではない。 |
| `.agents/skills/chakra-ui/references/components/layout/center.md` | — | Center | — | — | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/container.md` | — | Container | — | Container (`container`) | — | — | 参照対象外 | #716/#724 で非採用確定済み（layout プリミティブ）。（#937）Radix Themes 側にも同一概念が存在するが、layout プリミティブ / Theme provider は本リポジトリの参照軸に含めない「参照対象外」（根拠: #716/#724/#735、`docs/policy/intentional-non-adoption.md` §3.24、`docs/design/radix-themes-survey.md` §6）。既存の非採用決定（同 issue）を Radix 軸の文脈で再掲するものであり、新規の非採用判定ではない。 |
| `.agents/skills/chakra-ui/references/components/layout/flex.md` | — | Flex | — | Flex (`flex`) | — | — | 参照対象外 | #716/#724 で非採用確定済み（layout プリミティブ）。（#937）Radix Themes 側にも同一概念が存在するが、layout プリミティブ / Theme provider は本リポジトリの参照軸に含めない「参照対象外」（根拠: #716/#724/#735、`docs/policy/intentional-non-adoption.md` §3.24、`docs/design/radix-themes-survey.md` §6）。既存の非採用決定（同 issue）を Radix 軸の文脈で再掲するものであり、新規の非採用判定ではない。 |
| `.agents/skills/chakra-ui/references/components/layout/float.md` | — | Float | — | — | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/grid.md` | — | Grid | — | Grid (`grid`) | — | — | 参照対象外 | #716/#724 で非採用確定済み（layout プリミティブ）。（#937）Radix Themes 側にも同一概念が存在するが、layout プリミティブ / Theme provider は本リポジトリの参照軸に含めない「参照対象外」（根拠: #716/#724/#735、`docs/policy/intentional-non-adoption.md` §3.24、`docs/design/radix-themes-survey.md` §6）。既存の非採用決定（同 issue）を Radix 軸の文脈で再掲するものであり、新規の非採用判定ではない。 |
| `.agents/skills/chakra-ui/references/components/layout/group.md` | — | Group | — | — | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/simple-grid.md` | — | SimpleGrid | — | — | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/stack.md` | — | Stack | — | — | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |
| `.agents/skills/chakra-ui/references/components/layout/wrap.md` | — | Wrap | — | — | — | — | 意図的非採用 | #716/#724 で非採用確定済み（layout プリミティブ） |

#### `.agents/skills/chakra-ui/references/components/overlays/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/overlays/dialog.md` | Dialog | Dialog | Alert Dialog (`alert-dialog`) | Alert Dialog (`alert-dialog`) | `dialog` | `dialog` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/menu.md` | Menu | Menu | Context Menu (`context-menu`) | Context Menu (`context-menu`) | `menu` | `menu` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/popover.md` | Popover | Popover | — | — | `popover` | `popover` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/tooltip.md` | Tooltip | Tooltip | — | — | `tooltip` | `tooltip` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/drawer.md` | Drawer | Drawer | — | — | `drawer` | `drawer` | 実装済み | headless+styled 実装済み（#758、dialog の状態機械を再利用） |
| `.agents/skills/chakra-ui/references/components/overlays/hover-card.md` | HoverCard | HoverCard | — | — | `hover_card` | `hover_card` | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/overlays/toggle-tip.md` | — | ToggleTip | — | — | `toggle_tip` | `toggle_tip` | 実装済み | headless+styled 実装済み（#761、PR #804） |
| `.agents/skills/chakra-ui/references/components/overlays/action-bar.md` | — | ActionBar | — | — | `action_bar` | `action_bar` | 実装済み | headless+styled 実装済み（#762） |
| `.agents/skills/chakra-ui/references/components/overlays/overlay-manager.md` | — | OverlayManager | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/overlays/floating-panel.md` | FloatingPanel | FloatingPanel | — | — | `floating_panel` | `floating_panel` | 実装済み | headless+styled 実装済み（イシュー #827、`docs/policy/intentional-non-adoption.md` §7 の保留区分から解除） |

#### `.agents/skills/chakra-ui/references/components/typography/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/typography/link.md` | — | Link | — | Link (`link`) | `link` | `link` | 実装済み | headless+styled 実装済み（#756、PR #801、#716 最優先候補の消化） |
| `.agents/skills/chakra-ui/references/components/typography/link-overlay.md` | — | LinkOverlay | — | — | `link_overlay` | `link_overlay` | 実装済み | headless+styled 実装済み（#756、PR #801、#716 最優先候補の消化） |
| `.agents/skills/chakra-ui/references/components/typography/kbd.md` | — | Kbd | — | Kbd (`kbd`) | — | `kbd` | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/typography/code.md` | — | Code | — | Code (`code`) | — | `code` | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/typography/heading.md` | — | Heading | — | Heading (`heading`) | — | `heading` | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/text.md` | — | Text | — | Text (`text`) | — | `text` | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/em.md` | — | Em | — | Em (`em`) | — | `em` | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/mark.md` | — | Mark | — | — | — | `mark` | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/blockquote.md` | — | Blockquote | — | Blockquote (`blockquote`) | — | `blockquote` | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/list.md` | — | List | — | — | — | `list` | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/highlight.md` | Highlight | Highlight | — | — | — | `highlight` | 実装済み | #775。pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/typography/rich-text-editor.md` | — | RichTextEditor | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.22（#735）で非採用確定（高度入力系、canvas/ポインタ座標/contenteditable 依存）。等価概念は本書 §8 参照 |
| `.agents/skills/chakra-ui/references/components/typography/code-block.md` | — | CodeBlock | — | — | — | — | 対象外 | 対象外（docs-site 既存機構と役割分担） |
| `.agents/skills/chakra-ui/references/components/typography/prose.md` | — | Prose | — | — | — | — | 対象外 | 対象外（docs-site 既存機構と役割分担。役割分担の詳細は #771 の `crates/pre-styled-ui/src/text.rs` rustdoc・`docs/api/pre-styled-ui-api.md` 参照） |

#### `.agents/skills/chakra-ui/references/components/utilities/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/utilities/visually-hidden.md` | — | VisuallyHidden | Visually Hidden (`visually-hidden`) | Visually Hidden (`visually-hidden`) | `visually_hidden` | `visually_hidden` | 実装済み | #776 |
| `.agents/skills/chakra-ui/references/components/utilities/skip-nav.md` | — | SkipNav | — | — | `skip_nav` | `skip_nav` | 実装済み | #776 |
| `.agents/skills/chakra-ui/references/components/utilities/client-only.md` | ClientOnly | ClientOnly | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/environment-provider.md` | Environment | EnvironmentProvider | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/presence.md` | Presence | Presence | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/checkmark.md` | — | Checkmark | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/radiomark.md` | — | Radiomark | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/for.md` | — | For | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/portal.md` | — | Portal | Portal (`portal`) | Portal (`portal`) | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/show.md` | — | Show | — | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし。等価概念は本書 §8 参照） |
| `.agents/skills/chakra-ui/references/components/utilities/theme.md` | — | Theme | — | Theme (`theme`) | — | — | 参照対象外 | `docs/policy/intentional-non-adoption.md` §3.24（#735）で非採用確定（既存 theme mod と役割重複）。等価概念は本書 §8 参照。（#937）Radix Themes 側にも同一概念が存在するが、layout プリミティブ / Theme provider は本リポジトリの参照軸に含めない「参照対象外」（根拠: #716/#724/#735、`docs/policy/intentional-non-adoption.md` §3.24、`docs/design/radix-themes-survey.md` §6）。既存の非採用決定（同 issue）を Radix 軸の文脈で再掲するものであり、新規の非採用判定ではない。 |

#### `.agents/skills/chakra-ui/references/get-started/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/get-started/README.md` | — | README | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/ai-llms.md` | — | AiLlms | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/ai-mcp-server.md` | — | AiMcpServer | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/ai-rules.md` | — | AiRules | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/ai-skills.md` | — | AiSkills | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/changelog.md` | — | Changelog | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/cli.md` | — | Cli | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/contributing.md` | — | Contributing | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/env-iframe.md` | — | EnvIframe | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/env-shadow-dom.md` | — | EnvShadowDom | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/figma.md` | — | Figma | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-next-app.md` | — | FrameworkNextApp | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-next-pages.md` | — | FrameworkNextPages | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-remix.md` | — | FrameworkRemix | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-storybook.md` | — | FrameworkStorybook | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-tanstack-router.md` | — | FrameworkTanstackRouter | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/framework-vite.md` | — | FrameworkVite | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/installation.md` | — | Installation | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/migration.md` | — | Migration | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/get-started/playground.md` | — | Playground | — | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/chakra-ui/references/styling/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/README.md` | — | Readme | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/styling/compositions/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/compositions/animation-styles.md` | — | AnimationStyles | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/focus-ring.md` | — | FocusRing | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/layer-styles.md` | — | LayerStyles | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/text-styles.md` | — | TextStyles | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/styling/concepts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/concepts/cascade-layers.md` | — | CascadeLayers | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/chakra-factory.md` | — | ChakraFactory | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/color-opacity-modifier.md` | — | ColorOpacityModifier | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/conditional-styles.md` | — | ConditionalStyles | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/css-variables.md` | — | CssVariables | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/dark-mode.md` | — | DarkMode | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/overview.md` | — | Overview | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/responsive-design.md` | — | ResponsiveDesign | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/virtual-color.md` | — | VirtualColor | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/styling/style-props/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/style-props/background.md` | — | Background | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/border.md` | — | Border | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/display.md` | — | Display | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/effects.md` | — | Effects | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/filters.md` | — | Filters | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/flex-and-grid.md` | — | FlexAndGrid | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/interactivity.md` | — | Interactivity | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/layout.md` | — | Layout | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/list.md` | — | List | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/sizing.md` | — | Sizing | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/spacing.md` | — | Spacing | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/svg.md` | — | Svg | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/tables.md` | — | Tables | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/transforms.md` | — | Transforms | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/transitions.md` | — | Transitions | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/typography.md` | — | Typography | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/README.md` | — | Readme | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/compositions/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/compositions/layer-styles.md` | — | LayerStyles | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/compositions/text-styles.md` | — | TextStyles | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/concepts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/concepts/overview.md` | — | Overview | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/recipes.md` | — | Recipes | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/semantic-tokens.md` | — | SemanticTokens | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/slot-recipes.md` | — | SlotRecipes | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/tokens.md` | — | Tokens | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/customization/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/customization/animations.md` | — | Animations | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/breakpoints.md` | — | Breakpoints | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/colors.md` | — | Colors | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/conditions.md` | — | Conditions | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/css-variables.md` | — | CssVariables | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/global-css.md` | — | GlobalCss | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/overview.md` | — | Overview | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/recipes.md` | — | Recipes | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/sizes.md` | — | Sizes | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/spacing.md` | — | Spacing | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/utilities.md` | — | Utilities | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

#### `.agents/skills/chakra-ui/references/theming/design-tokens/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/design-tokens/animations.md` | — | Animations | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/aspect-ratios.md` | — | AspectRatios | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/breakpoints.md` | — | Breakpoints | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/colors.md` | — | Colors | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/cursors.md` | — | Cursors | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/radii.md` | — | Radii | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/shadows.md` | — | Shadows | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/sizes.md` | — | Sizes | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/spacing.md` | — | Spacing | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/typography.md` | — | Typography | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/z-index.md` | — | ZIndex | — | — | — | — | 対象外 | （#735、概念文書。fandhe は theme/recipe/StyleSheet（`crates/pre-styled-ui`）で対応） |

### Part C: fandhe 独自 mod・charts 基盤（ark-ui / chakra-ui に対応 md なし）

§4 の完全性 diff（references 側 359 件の突合）の対象外である。本 Part の
セルには `.agents/skills/…` のパスを一切書かない（書くと §4 の抽出正規表現
（`\.agents/skills/(ark-ui|chakra-ui)/references/[A-Za-z0-9/._-]+\.md`）が
実在しないパス文字列を拾ってしまい、diff が非空になって §4 が壊れる）。

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| —（対応 md なし） | — | — | — | — | `nav_list` | `nav_list` | 実装済み | fandhe 独自。#756 → PR #801 で Link / LinkOverlay と同時に実装した文書ナビ用リスト（Root / Heading / List / Item / Link の 5 anatomy） |
| —（対応 md なし） | — | — | — | — | — | `charts::data` | 実装済み | charts 基盤。#846。`ChartData` / `Series` モデルと集計 API（ノードを生成しない純計算） |
| —（対応 md なし） | — | — | — | — | — | `charts::scale` | 実装済み | charts 基盤。#846。線形スケール・1-2-5 nice tick 算出（ノードを生成しない純計算） |
| —（対応 md なし） | — | — | — | — | — | `charts::svg` | 実装済み | charts 基盤。#846。SVG ノード木生成ヘルパー |
| —（対応 md なし） | — | — | — | — | — | `charts::pie` | 実装済み | charts 基盤。#850 → PR #881。`pie_chart` / `donut_chart` が使う円弧ジオメトリ（`d` 属性文字列を返す純関数） |

※ `charts::data` / `scale` / `svg` は既に `charts/use-chart.md` 行（Part B
「charts」節）の本文中で散文的に触れられているが、mod 名としては計上されて
いなかったため本 Part で行として明示する（重複計上ではなく、mod 側キーでの
計上）。

### Part D: Radix にのみ存在する部品（ark-ui / chakra-ui に対応 md なし）

イシュー #937 で新設。§4 の完全性 diff（references 側 359 件の突合）の対象外
である。本 Part のセルには `.agents/skills/…` のパスを一切書かない（Part C
と同じ理由。書くと §4 の抽出正規表現が実在しないパス文字列を拾ってしまい、
diff が非空になって §4 が壊れる）。「実装対象」区分の根拠・対応 issue 列は
`#959 で確定、仮 ID 8-x（Step B で採番、§9.1 参照）`（Phase 8、本書 §9/§9.1
参照）を、「保留」区分は `#959 で判定継続。intentional-non-adoption.md §7 へ
転記済み（#959）`（`docs/policy/intentional-non-adoption.md` §7 参照）を指す。

| 参照ファイル | ark-ui 名 | chakra-ui 名 | Radix Primitives 名 | Radix Themes 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|---|---|
| —（対応 md なし） | — | — | Form (`form`) | — | `field`/`fieldset`（構造部分は充足） | — | 意図的非採用 | **2026-07-25 のユーザー判断により保留を解除し意図的非採用へ確定**。根拠: `docs/policy/intentional-non-adoption.md` §3.25 規則 1（アプリケーションロジックを内包する UI 部品は採用しない）。Radix Form の本体はブラウザ Constraint Validation API を用いたバリデーション実行・カスタム検証・エラーメッセージ対応付け・送信ハンドリングであり、UI 層の責務（anatomy・アクセシビリティ・表示状態）を超える。UI 構造に相当する部分（フィールドとラベル・説明・エラー表示の結び付け、`data-invalid` による無効状態の表現）は `crates/headless-ui/src/field.rs` / `fieldset.rs` が既に担っており、利用者はバリデーションを通常の Rust コードで書いてその結果を `field` の状態として渡す。再評価トリガーは §3.25 規則 1 を参照。`intentional-non-adoption.md` §7 へ転記済み（#959） |
| —（対応 md なし） | — | — | Menubar (`menubar`) | — | `menubar` | `menubar` | 実装済み | 複数 Menu を水平（または垂直）に並べる専用 anatomy（Root/Menu/Trigger/Positioner/Content/Item/ItemGroup/ItemGroupLabel/Separator/SubTrigger/SubContent）と、roving tabindex + 開いている Menu を跨いだ左右移動の状態機械 `Menubar` を実装。`menu` の anatomy はそのまま再利用せず、状態機械・値語彙（`OpenState`/`aria`/`data_attrs`）のみを再利用する。#959 で確定、仮 ID 8-2、実装は #992 |
| —（対応 md なし） | — | — | Navigation Menu (`navigation-menu`) | — | `navigation_menu` | `navigation_menu` | 実装済み | **2026-07-25 のユーザー判断により `docs/policy/intentional-non-adoption.md` §3.25 規則 2 を適用**: viewport 測定・`data-motion` は装飾・アニメーション関心のため headless-ui へ持ち込まず、必要なら pre-styled-ui 側の責務として設計する（本イシュー #993 では pre-styled-ui 側にも未実装。必要になった時点で別途 Issue 化を検討する）。headless-ui は Root/List/Item/Trigger/Content/Link の anatomy とアクティブリンクの `aria-current` までを実装した（[`crate::state::SingleSelect`] を埋め込んだ「高々 1 個の Trigger だけが開く」状態機械 `NavigationMenu`）。`role` は一切付与しない（`root` は素の `nav` の暗黙 role に依拠）。`nav_list`（イシュー #756）は role を持たない文書ナビ専用部品であり、Navigation Menu のディスクロージャ（Trigger/Content の開閉）・アクティブリンク追跡・`data-motion` とは意味論・機能ともに別物（`crates/headless-ui/src/nav_list.rs` module doc 参照）。#959 で確定、仮 ID 8-3、実装は #993 |
| —（対応 md なし） | — | — | Toolbar (`toolbar`) | — | `toolbar` | `toolbar` | 実装済み | ボタン・セパレータ・ToggleGroup を横方向グループ化する専用 anatomy（Root/Button/Link/Separator/ToggleGroup/ToggleItem）と roving tabindex 状態機械 `Toolbar` を実装。#959 で確定、仮 ID 8-1、実装は #991 |
| —（対応 md なし） | — | — | Direction Provider (`direction-provider`) | — | — | — | 保留 | RTL/LTR を動的注入する provider 機構は `docs/policy/intentional-non-adoption.md` §3.23 の JS ランタイム固有 utilities に類するが、同節に個別記録がない。再評価トリガー: provider 機構全般の非採用可否が §3.23/§3.24 へ確定記録された場合、または `dir` 属性の明示的引数渡しで代替可能と判断された場合。#959 で判定継続。`intentional-non-adoption.md` §7 へ転記済み（#959） |
| —（対応 md なし） | — | — | Accessible Icon (`accessible-icon`) | Accessible Icon (`accessible-icon`) | `role`/`aria_label`（`crates/headless-ui/src/aria.rs`、いずれも `pub`）+ `visually_hidden`（代替経路、検証済み） | `icon`（`IconProps::label`、代替済み、検証済み）/ `visually_hidden`（代替経路、検証済み） | 保留 | **イシュー #1066 で検証完了（2026-07-26）**。層別の結論: (1) Themes/pre-styled 層は `icon` 単体で完全代替する。`crates/pre-styled-ui/src/icon.rs` の `icon()` は `IconProps.label` の `Some`/`None` で `role="img"`+`aria-label` と `aria-hidden="true"` を分岐済みで、`label_some_switches_to_role_img_and_aria_label`/`default_props_render_md_size_decorative`（`crates/pre-styled-ui/src/icon.rs` 内テスト）で固定されている。利用者向け原稿 `site/themes/icon.md` にも既に明記済み。(2) Primitives/headless 層は `icon` パート自体を持たないが、`crates/headless-ui/src/aria.rs` の `pub fn role`/`pub fn aria_label`（`icon.rs` 自身が同じヘルパを利用）を SVG へ組み合わせる、または `crates/headless-ui/src/visually_hidden.rs`（`root` は `aria-hidden` を一切出力しない不変条件、`root_does_not_emit_aria_hidden_by_default` で固定）でテキストノードを併記すれば代替でき、専用部品の新設は不要。(3) 機構差: Radix はビジュアリーヒドゥンな**テキストコンテンツ**でアクセシブルネームを与えるが、fandhe の `icon` は `aria-label` で与える。アクセシブルネーム計算上は等価だが同一手段ではなく、実テキストノードが必要な場合は `visually_hidden::root` が該当する。結論: 代替可能・実装不要。再評価トリガー: 充足済み（検証完了 2026-07-26、イシュー #1066）。意図的非採用への区分移行はユーザー判断待ち（区分列は本 PR では変更しない） |
| —（対応 md なし） | — | — | Slot (`slot`) | Slot (`slot`) | — | — | 保留 | asChild/Slot 相当の要素種別差し替え・子要素への props マージという仕組み自体が現時点の `fandhe-frontend-headless-ui` には存在しない（`docs/design/radix-primitives-inventory.md` §7.2）。再導入の提案はここでは書かない（`.claude/rules/coding-rust.md`）。再評価トリガー: 要素差し替え機構の要否が別途の設計検討で明示的に再評価された場合。#959 で判定継続。`intentional-non-adoption.md` §7 へ転記済み（#959） |
| —（対応 md なし） | — | — | Accessibility (`accessibility`) | — | — | — | 対象外 | 対象外（非コンポーネント文書、overview） |
| —（対応 md なし） | — | — | Getting Started (`getting-started`) | — | — | — | 対象外 | 対象外（非コンポーネント文書、overview） |
| —（対応 md なし） | — | — | Introduction (`introduction`) | — | — | — | 対象外 | 対象外（非コンポーネント文書、overview） |
| —（対応 md なし） | — | — | Releases (`releases`) | — | — | — | 対象外 | 対象外（非コンポーネント文書、overview） |
| —（対応 md なし） | — | — | Animation (`animation`) | — | — | — | 対象外 | 対象外（非コンポーネント文書、guides） |
| —（対応 md なし） | — | — | Composition (`composition`) | — | — | — | 対象外 | 対象外（非コンポーネント文書、guides。合成パターン（asChild/Slot）の事実記録は本書 §5 Part D の Slot 行・`docs/design/radix-primitives-inventory.md` §7 参照） |
| —（対応 md なし） | — | — | Server-Side Rendering (`server-side-rendering`) | — | — | — | 対象外 | 対象外（非コンポーネント文書、guides） |
| —（対応 md なし） | — | — | Styling (`styling`) | — | — | — | 対象外 | 対象外（非コンポーネント文書、guides） |
| —（対応 md なし） | — | — | — | Callout (`callout`) | fandhe pre-styled-ui = `callout` | — | 実装済み | 既存 `alert` の anatomy を参考に root/icon/text の 3 パーツで新設（イシュー #994）。`alert` と異なり `role="alert"` を付与しない静的部品。仮 ID 8-4 |
| [checkbox-group](../../site/themes/checkbox-group.md) | — | — | — | Checkbox Group (`checkbox-group`) | — | — | 実装済み | イシュー #997 で実装済み。headless-ui（`checkbox_group` mod、Root/Label/Item/ItemControl/ItemIndicator/ItemText の 6 パーツ + `state::MultiSelect` を埋め込んだ複数選択状態機械）と pre-styled-ui（`checkbox_group` mod、`size`/`color-palette` variant）の両層で新設。単一選択版 `radio_group` と対称の構造。ネイティブ `<input type="checkbox">` は自前パーツを持たず既存 `checkbox::hidden_input` の入れ子再利用で賄う（`checkbox`/`checkbox_card` の重複実装を回避）。#959 で確定、仮 ID 8-7 |
| —（対応 md なし） | — | — | — | Inset (`inset`) | — | — | 保留 | layout 系ユーティリティに近いが、#716/#724 の layout プリミティブ 5 件（Box/Flex/Grid/Container/Section）には含まれない（`docs/design/radix-themes-survey.md` §3.1 注記・§6 が明記）。参照対象外リストに含めない一次記録に従い「保留」とする。再評価トリガー: layout 系ユーティリティ全般の参照方針が別途確定した場合。#959 で判定継続。`intentional-non-adoption.md` §7 へ転記済み（#959） |
| [quote](../../site/themes/quote.md) | — | — | — | Quote (`quote`) | — | — | 実装済み | イシュー #995 で実装済み。既存 `blockquote` と役割が近い静的テキスト部品として新設（#959 で確定、仮 ID 8-5） |
| —（対応 md なし） | — | — | — | Radio (`radio`) | — | — | 保留 | 既存 `radio_group` はグループ前提の anatomy。グループ化しない単独 Radio ボタンの anatomy 差分は未検証。再評価トリガー: 単独 Radio の要否・anatomy 差分の検証完了時。#959 で判定継続。`intentional-non-adoption.md` §7 へ転記済み（#959） |
| —（対応 md なし） | — | — | — | Reset (`reset`) | — | — | 保留 | ブラウザ既定スタイルのリセット専用コンポーネント。**イシュー #1066 で検証完了（2026-07-26）**。実測結果: `crates/pre-styled-ui/src/theme.rs::Theme::to_css` の出力は `:root { … }`/`:root[data-theme="light"]`/`@media (prefers-color-scheme: dark)`/`:root[data-theme="dark"]` の 4 ブロックのみで要素セレクタへのリセット宣言を一切出力しない。`crates/pre-styled-ui/src/stylesheet.rs` の `StyleSheet` は検証済み CSS のみを保持する配布ヘルパ（`push_css`/`push_recipe`/`push_theme`）でありリセット内容自体は持たない。`crates/pre-styled-ui/src` 全体で全称セレクタ（`"*"`）の使用は 0 件、`box-sizing`/`margin: 0` は各 recipe の `base` として `[data-scope="…"][data-part="…"]` にスコープされて分散している。グローバルリセットの実在箇所は `crates/docs-site/src/site_theme.rs:202-213`（docs サイト骨格側）であり UI コンポーネント層ではない。結論: 既存 `stylesheet`/`theme` mod との**重複はない**。ただし重複がないことは採用理由にならず、Radix `Reset` は `<Reset>{child}</Reset>` 形式の asChild（Slot 依存）型ラッパーであり、実装可否は Slot 行（本書 §5 Part D / §9 の Slot 行）の保留に従属するため**保留を維持**する。再評価トリガー: 充足済み（検証完了 2026-07-26、イシュー #1066）。結論: 保留維持・実装 issue 起票なし。Slot 行の再評価時に同時再判定する |
| [strong](../../site/themes/strong.md) | — | — | — | Strong (`strong`) | — | — | 実装済み | イシュー #995 で実装済み。既存 `em`（強調）と役割が対称な静的テキスト部品として新設（#959 で確定、仮 ID 8-5） |
| —（対応 md なし） | — | — | — | Tab Nav (`tab-nav`) | — | `tab_nav` | 実装済み | `tabs` の見た目を持つナビゲーションリンク集合として新規 anatomy（Root/Link）を pre-styled-ui 単独で定義（headless-ui は変更なし、`checkbox_card`/`radio_card` と同型の判断）。`role="tablist"`/`role="tab"` を一切出力せず `aria-current="page"` で現在地を示す。#959 で確定、仮 ID 8-6、実装は #996 |
| —（対応 md なし） | — | — | — | Section (`section`) | — | — | 参照対象外 | layout プリミティブ（根拠: #716/#724/#735、`docs/policy/intentional-non-adoption.md` §3.24、`docs/design/radix-themes-survey.md` §6）。Box/Flex/Grid/Container と同方針。既存の非採用決定を Radix 軸の文脈で再掲するものであり新規判定ではない |

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
  計上漏れ確認コマンド（イシュー #934 で追加）・§4.3〜§4.5 の Radix 完全性
  diff・表記規約自己検査（イシュー #937 で追加）をいずれも CI ジョブ /
  テストとして自動実行する仕組みは本イシューのスコープ外。`crates/xtask`
  への `check-coverage-map` サブコマンド追加等でこれらを機械検知したい
  場合は別 issue の起票を提案する（`.claude/rules/out-of-scope-tracking.md`
  に従い、ユーザー承認の上で #726 配下へ）
- **`mod qr_encode` / `mod class_attr` の可視性見直し**: 両 mod は
  `mod`（非 `pub`）宣言のため §3 の突合コマンドの対象外だが、`crates/**`
  の変更を伴うため本書のみを対象とする本イシュー（doc-only）のスコープ外
- **保留・意図的非採用の評価軸・再評価トリガーの詳細記録**: イシュー #735 で
  `docs/policy/intentional-non-adoption.md` §3.22〜§3.24（新規非採用確定）・
  §7（保留項目の記録）に確定記録済み（完了）。イシュー #937 で新規判定した
  Radix 由来の保留 7 件（Form / Direction Provider / Accessible Icon / Slot /
  Inset / Radio / Reset）は #959 の Step A で同節へ転記済み（完了）
- **Radix 側の更新追随**: `radix-primitives-inventory.md` / 本書の pin
  （`radix-ui/website` commit `bb42408`）以降の Radix 側の新規コンポーネント
  追加・anatomy 変更の追随は、本書の改訂 issue を起票して行う。CI による
  自動検知は行わない（既存 §4 と同方針）
- **Phase 8（#932/#959）の子 issue 起票・番号採番**: 本書 §9/§9.1 は
  実行可能な完全な roster（タイトル・依存順・対象クレート・非目標）を
  提供するのみであり、実際の `gh issue create`・sub_issues 紐付け・本書
  §5 Part D/§9.1 への実採番値の反映は #959 の Step B（Step A の PR マージ後）
  のスコープ
- **Phase 8（#932/#959）の子 issue 分割**: 本書 §9 は引き渡し表を提供する
  のみであり、実際の子 issue 起票・実装は #959 のスコープ

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

## 9. Phase 8（Radix 差分の部品実装）への引き渡し（イシュー #937 → #959）

本節は #959（Phase 8 子 issue 分割、親 #932、トラッキング #924）の唯一の
入力である。§5 Part D のうち区分「実装対象」「保留」の行のみを列挙する
（「対象外」「実装済み」「参照対象外」は Phase 8 の対象外のため載せない）。
保留行には再評価トリガーを必ず記す（§2 の保留定義の改訂に対応）。

| Radix 名 (slug) | 層 | §5 の該当行 | 区分 | 判定根拠 / 保留の再評価トリガー | Phase 8 issue |
|---|---|---|---|---|---|
| Menubar (`menubar`) | Primitives → headless-ui | Part D | 実装済み | 複数 Menu の水平配置・roving tabindex・開いている Menu を跨いだ左右移動の専用 anatomy を実装。`menu` の状態機械・値語彙のみ再利用 | #992 |
| Navigation Menu (`navigation-menu`) | Primitives → headless-ui | Part D | 実装済み | `nav_list` は role なし・ディスクロージャ非対応（対称項目参照）。**2026-07-25 のユーザー判断により `intentional-non-adoption.md` §3.25 規則 2 を適用**: viewport 測定・`data-motion` は装飾関心のため headless-ui へ置かず、実装しない（pre-styled-ui 側も本イシュー #993 では未実装）。headless-ui は Root/List/Item/Trigger/Content/Link の anatomy とアクティブリンクの `aria-current` までを実装した | #993 |
| Toolbar (`toolbar`) | Primitives → headless-ui | Part D | 実装済み | ボタン・セパレータ・ToggleGroup の横方向グループ化 anatomy を実装 | #991 |
| Callout (`callout`) | Themes → pre-styled-ui | Part D | 実装済み | root/icon/text の 3 パーツで新設。`alert` と異なり `role="alert"` を付与しない静的部品 | #994 |
| Checkbox Group (`checkbox-group`) | Themes → pre-styled-ui | Part D | 実装済み | headless-ui（複数選択状態機械 `state::MultiSelect` を埋め込んだ `checkbox_group::CheckboxGroup`）+ pre-styled-ui（`size`/`color-palette` variant）の両層で実装。ネイティブ input は既存 `checkbox::hidden_input` を再利用（重複実装なし） | #997 |
| Quote (`quote`) | Themes → pre-styled-ui | Part D | 実装済み | 既存 `blockquote` と役割が近い静的部品として実装（インライン `<q>`、`blockquote` はブロック引用） | #995 |
| Strong (`strong`) | Themes → pre-styled-ui | Part D | 実装済み | 既存 `em`（強調）と役割対称の静的部品として実装（`font-weight: bold` で `em` と区別） | #995 |
| Tab Nav (`tab-nav`) | Themes → pre-styled-ui | Part D | 実装済み | `tabs` の見た目を持つナビゲーションリンク集合として pre-styled-ui 単独で新規 anatomy（Root/Link）を実装。`role` は一切出力しない | #996 |
| Form (`form`) | Primitives → headless-ui | Part D | 意図的非採用 | 2026-07-25 のユーザー判断で保留解除・非採用確定。バリデーション・送信はアプリケーションロジックであり UI 層の責務外（`intentional-non-adoption.md` §3.25 規則 1）。構造部分は `field`/`fieldset` が充足 | —（実装しない） |
| Direction Provider (`direction-provider`) | Primitives（utility） | Part D | 保留 | JS ランタイム固有 utilities に類するが個別記録なし。再評価トリガー: provider 機構全般の非採用可否が §3.23/§3.24 へ確定記録された場合、または `dir` 属性引数渡しで代替可能と判断された場合 | —（#959 で判定継続） |
| Accessible Icon (`accessible-icon`) | Primitives（utility）+ Themes（utility） | Part D | 保留 | イシュー #1066 で検証完了（2026-07-26）。Themes 層は `icon`（`IconProps::label`）単体で完全代替、Primitives 層は `role`/`aria_label`（`crates/headless-ui/src/aria.rs`）+ `visually_hidden` の組み合わせで代替可能（詳細は §5 Part D 該当行参照）。結論: 代替可能・実装不要 | —（実装しない: 既存部品で代替可能。区分確定はユーザー判断待ち） |
| Slot (`slot`) | Primitives（utility）+ Themes（utility） | Part D | 保留 | asChild/Slot 相当の要素差し替え・props マージ機構が現時点で存在しない（再導入提案はしない）。再評価トリガー: 要素差し替え機構の要否が別途の設計検討で明示的に再評価された場合 | —（#959 で判定継続） |
| Inset (`inset`) | Themes → pre-styled-ui | Part D | 保留 | layout 系ユーティリティに近いが layout プリミティブ 5 件には含まれない（`radix-themes-survey.md` §3.1/§6）。再評価トリガー: layout 系ユーティリティ全般の参照方針が別途確定した場合 | —（#959 で判定継続） |
| Radio (`radio`) | Themes → pre-styled-ui | Part D | 保留 | 既存 `radio_group` はグループ前提。単独 Radio の anatomy 差分未検証。再評価トリガー: 単独 Radio の要否・anatomy 差分の検証完了時 | —（#959 で判定継続） |
| Reset (`reset`) | Themes → pre-styled-ui | Part D | 保留 | イシュー #1066 で検証完了（2026-07-26）。`stylesheet`/`theme` mod のリセット範囲は実測でゼロ（要素セレクタへのリセット出力なし）で重複はないが、Radix `Reset` は Slot 依存の asChild 型のため実装可否は Slot 行の保留に従属（詳細は §5 Part D 該当行参照）。結論: 保留維持 | —（Slot 行の保留に従属。実装 issue 起票なし） |

「Phase 8 issue」列の初期値はすべて `—（#959 で採番）`／`—（#959 で判定継続）`
であり、#959 が子 issue を採番した時点で本節を更新して埋める。うち
Accessible Icon / Reset はイシュー #1066 で検証完了し判定継続を終了した
（前者は代替可能・実装不要、後者は保留維持。詳細は上表・§5 Part D 参照）。

### 9.1 Phase 8 子 issue roster（#959 確定）

実装対象 8 行のうち Quote/Strong は対称の静的テキスト部品として 1 issue に
統合し、**7 issue** へ分割する（roster 確定は Step A、実採番・sub_issues
紐付けは Step B）。層は §9 の「層」列（`Primitives → headless-ui` /
`Themes → pre-styled-ui`）を額面どおり採用し、headless-ui のみ・
pre-styled-ui のみで完結させる（両層セット実装にはしない）。実行順は
`parallel: 1` の直列実行を前提に dependsOn で固定する。

| 実行順 | 仮 ID | タイトル案 | 対象クレート | 層 | dependsOn | version バンプ対象（イシュー #638/#657） | 明示的非目標 |
|---|---|---|---|---|---|---|---|
| 1 | 8-1 | `feat(headless-ui): Toolbar — ボタン/セパレータ/ToggleGroup の横方向グループ化 anatomy を新設する` | headless-ui | Primitives → headless-ui | — | headless-ui + pre-styled-ui + wasm-full（`check-dep-versions --fix` で req 追随） | roving tabindex 等 `crates/wasm-full/` 側の DOM 配線 |
| 2 | 8-2 | `feat(headless-ui): Menubar — menu anatomy 再利用による水平メニューバーを新設する` | headless-ui | Primitives → headless-ui | 8-1 | headless-ui + pre-styled-ui + wasm-full | 同上 |
| 3 | 8-3 | `feat(headless-ui): Navigation Menu — viewport/アクティブリンク追跡付きナビゲーションメニューを新設する`（実装イシュー #993 のタイトルは「構造とアクティブリンク（装飾関心は pre-styled 層へ）」。「viewport」は本注記のとおり§3.25 規則 2 により非目標へ確定したため実装イシューのタイトルからは落とした） | headless-ui | Primitives → headless-ui | 8-2 | headless-ui + pre-styled-ui + wasm-full（実績: headless-ui のみ。pre-styled-ui は寸法/強調色 variant なしの薄いラッパー、wasm-full はバージョン追随のみで実 DOM 配線は未着手） | viewport 実配置・アクティブリンク動的同期・`data-motion` 遷移駆動（`intentional-non-adoption.md` §3.25 規則 2 により headless-ui 層から恒久的に除外。pre-styled-ui 側で必要になった場合も本表の非目標指定はそのまま有効） |
| 4 | 8-4 | `feat(pre-styled-ui): Callout — alert anatomy を参考にした注意喚起部品を新設する`（実装イシュー #994 のタイトルは「Callout — 補足情報の強調表示部品」） | pre-styled-ui | Themes → pre-styled-ui | #943, 8-3 | pre-styled-ui のみ | — |
| 5 | 8-5 | `feat(pre-styled-ui): Quote / Strong — em 対称の静的テキスト部品 2 件を新設する`（実装イシュー #995 で実装済み） | pre-styled-ui | Themes → pre-styled-ui | #943, 8-4 | pre-styled-ui のみ | — |
| 6 | 8-6 | `feat(pre-styled-ui): Tab Nav — tabs の見た目を持つナビゲーションリンク集合を新設する` | pre-styled-ui | Themes → pre-styled-ui | #943, 8-5 | pre-styled-ui のみ | — |
| 7 | 8-7 | `feat(headless-ui): Checkbox Group — 複数選択グループの anatomy と状態機械`（実装イシュー #997。実績: 単体 `checkbox`/`checkbox_card` の複数選択グループ anatomy が既存クレートに存在しなかったため headless-ui + pre-styled-ui の両層で実装） | headless-ui + pre-styled-ui | Themes → pre-styled-ui | #943, 8-6 | headless-ui + pre-styled-ui（`check-dep-versions --fix` で req 追随。wasm-full は Cargo.toml の依存 version 追随のみでバンプ対象、実 DOM 配線は未着手） | 矢印キー/Space の実 DOM 配線・全選択/一部選択の集約 API・Field 連携（`crates/headless-ui/src/checkbox_group.rs` rustdoc「out-of-scope」節参照） |

**Phase 3（#943/#944）との依存関係（重要な発見）**: #932/#924 は「Phase 8
は Phase 2〜7 と完全に独立」と記すが、pre-styled-ui へ新規 mod を追加する
8-4〜8-7（計 5 mod: Callout/Checkbox Group/Quote/Strong/Tab Nav）は
`docs/design/docs-site-component-pages.md` §3 の部品ページ台帳（99 行）
の対象であり独立ではない。各 issue は成果物へ「台帳 1 行追加 + §3 総ページ数の
更新（99 → 104）+ `site/themes/<kebab>.md`（イシュー #1017 で
`site/components/<kebab>.md` から移行） + `site/nav.toml` 登録 +
`site/themes.md`（イシュー #1018 で `site/components-pre-styled-ui.md` から
改称・移設）索引への 1 行追加」を含める。`nav::validate_sources`
が `page.source` の実在を検査するため、`nav.toml` 登録と原稿投入は同一 PR で
行う。#944 がページ数の機械アサーションを導入済みの場合は同アサーションも
更新する。`crates/docs-site/tests/site_css_contract.rs` は対象外（同ファイルの
module doc によりサイト骨格 `docs-*` class のみが契約対象、pre-styled-ui
部品の class は管轄外）。

**Phase 6（#953/#954）との関係**: 新規 mod は `docs/api/pre-styled-ui-api.md`
§2「実装状況」表（headless-ui は `docs/api/headless-ui-api.md`）へ追記する。
#953/#954 との衝突は**加法マージ（両方のエントリを残す）**で解決する。

**headless-ui 3 件（8-1〜8-3）の DOM 配線はスコープ外**: SSR anatomy +
決定的状態機械のみへ閉じる（先例: `scroll_area` #825、`carousel` #754、
`image_cropper` #844）。roving tabindex のキーボードハンドリング・viewport
実配置・アクティブリンク動的同期・`data-motion` 遷移駆動は `crates/wasm-full/`
側の後続対応とし、`.claude/rules/out-of-scope-tracking.md` に従い follow-up
issue の起票をユーザーへ提案する旨を各 issue 本文へ記載する。

**触らないもの**: `examples/headless-pre-styled-ui` と
`crates/cli/embedded-examples/` の crates.io version pin
（`example_publish_copy_drift` がバイト一致を強制するため、`.claude/rules/ci.md`
#609 節参照）。

各実装 issue（8-1〜8-7）に必ず含める完了条件テンプレート（#959 Step B の
issue 本文へそのまま転記する）:

```
- [ ] `crates/<layer>/src/<mod>.rs` を新設し `crates/<layer>/src/lib.rs` へ `pub mod` 追加
- [ ] `#![forbid(unsafe_code)]` 維持・外部依存ゼロ維持（headless-ui は core/interactive、
      pre-styled-ui は headless-ui のみ）・HTML 文字列の直接組み立て禁止・`raw_html()` 不使用
- [ ] anatomy / a11y テスト（headless-ui: `crates/headless-ui/tests/<mod>.rs`）
      または CSS 契約テスト（pre-styled-ui: `crates/pre-styled-ui/tests/<mod>_css.rs`）を追加
- [ ] XSS 回帰: テキスト補間経路のアサーションを既存
      `crates/pre-styled-ui/tests/xss_escape.rs` / `xss_escape_styled.rs`
      （headless-ui は `crates/headless-ui/tests/xss_escape.rs`）へ追加する。
      既存アサーションの削除・弱体化・`#[ignore]` 化は禁止
- [ ] version バンプ: 上表「version バンプ対象」列のクレート名を列挙 ＋
      `cargo run -p xtask -- check-dep-versions --fix`
- [ ] docs: `docs/design/component-coverage-map.md` 該当行を「実装済み」へ更新
- [ ] docs（pre-styled-ui のみ）: `docs/design/docs-site-component-pages.md` §3 台帳へ 1 行 +
      総ページ数更新 / `site/themes/<kebab>.md`（イシュー #1017 で
      `site/components/<kebab>.md` から移行） / `site/nav.toml` /
      `site/themes.md`（イシュー #1018 で `site/components-pre-styled-ui.md`
      から改称・移設）索引 / `docs/api/pre-styled-ui-api.md` §2
- [ ] docs（headless-ui のみ）: `docs/api/headless-ui-api.md` へ項目追加 +
      **改訂（2026-07-26、イシュー #1021、適用 #1031）**: headless-ui 部品も
      イシュー #1021 以降は `/primitives/<kebab>/` + `site/primitives/<kebab>.md`
      + `site/nav.toml`（Primitives セクション）登録・原稿レジストリ
      （`crates/docs-site/src/primitive_specs/`）登録が必要（旧「docs ページ
      台帳は pre-styled-ui mod をキーとするため `/components/<kebab>/` は
      作らない」は #1021 で実態が変わったため撤回する。Primitives 台帳の正は
      `crates/docs-site/src/primitives_catalog.rs`、登録の正は `site/nav.toml`）
- [ ] 非目標（wasm-full の DOM 配線 / examples の crates.io pin 変更）に手を出していない
- [ ] `#943`/`#944`/`#953`/`#954` と衝突した場合は加法マージ（両方のエントリを残す）
```

Step B（issue 起票、#959 の実装計画の Step A/Step B 分割参照）の PR 本文には
本節（§9.1）を全文転記し、「マージ = 起票承認」の旨を明記する。

## 10. docs サイト掲載先の対応（イシュー #1031）

**導出規約**:

| 実装列 | docs サイト掲載先 | 件数 |
|---|---|---|
| headless-ui mod（本書 §5 の「headless-ui」列） | `/primitives/<kebab>/`（原稿 `site/primitives/<kebab>.md`） | 63 |
| pre-styled-ui mod（本書 §5 の「pre-styled-ui」列） | `/themes/<kebab>/`（原稿 `site/themes/<kebab>.md`） | 107 |

`<kebab>` は mod 名の `_` → `-` 置換で機械導出する。

**本書は対応表の正であり、掲載先 URL の台帳は持たない**。掲載先の一次情報は
`site/nav.toml`（登録の正）と `docs/design/docs-site-primitives-themes-split.md`
§2/§3/§6（2 層分割・URL 体系・Primitives 台帳判別規約の正）であり、本節は
両者への導線のみを提供する。

**再実測コマンド**（§3/`docs-site-primitives-themes-split.md` §6 の慣例に
倣い、コマンドをそのまま埋める）:

```bash
ls site/primitives/*.md | wc -l   # => 63
ls site/themes/*.md    | wc -l    # => 107
# Primitives 台帳の判別規則（docs-site-primitives-themes-split.md §6）
grep -l 'anatomy(' crates/headless-ui/src/*.rs | grep -v '/anatomy.rs' | wc -l   # => 63
```

掲載先の網羅性の機械検証は `crates/docs-site/tests/primitives_catalog.rs`
（台帳と基盤リストの排他網羅）と `crates/docs-site/tests/site_nav.rs`
（ページ数期待値）が担い、**本書は件数を二重管理しない**。

**注意**: headless-ui のみ実装済みで pre-styled-ui ラッパー未実装の部品
（`collapsible` / `field` / `fieldset` / `progress`(linear) 等）は
`/primitives/` にのみ掲載され `/themes/` には現れない。逆に pre-styled-ui
独自部品（`marquee` 等）は `/themes/` にのみ現れる。掲載先は層ごとに
独立であり、本書の「実装済み」区分と 1:1 ではない。
