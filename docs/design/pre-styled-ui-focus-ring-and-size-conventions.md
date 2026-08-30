# pre-styled-ui フォーカスリング・size バリアント共通規約（イシュー #1424）

## 1. 位置づけ・目的

親イシュー #1421（Phase 0: スタイル調整の共通基盤）配下の規約固定イシュー。
Phase 1 以降で `crates/pre-styled-ui` 107 部品を chakra-ui / Radix Themes
基準へ個別調整する前に、以下 2 点の「書き方」を 1 つに固定し、部品ごとの
差分の発散を防ぐことが目的である。

1. **フォーカスリング**: `:focus-visible` 等の状態時のリング表現を、
   実装手段（`outline` / `box-shadow`）・色・太さ・オフセットのすべてで
   トークン経由の単一形へ統一する。
2. **size バリアント**: 名称セット・既定値・「size を持つ部品 / 持たない
   部品」の判定基準を全部品共通で定める。

兄弟イシュー #1422（色トークン）・#1423（radius/shadow/spacing/z-index、
`docs/design/pre-styled-ui-scale-tokens.md`）と同じ `theme.rs` を変更対象と
するが、本イシューは focus-ring 専用のトークングループ（寸法）と `colors`
グループへの `focus-ring` エントリ追加のみを行い、他グループの定数・API
には触れない（コンフリクト面の最小化、#1423 冒頭の方針を踏襲）。

## 2. 実測した現状（2026-08-31、`crates/pre-styled-ui` v0.41.0、#1423 マージ後）

`git grep` による棚卸し結果（再現コマンドは末尾参照）:

| 観点 | 件数 | 内訳・所見 |
|------|------|-----------|
| `:focus`（`-visible`/`-within` を伴わない直書き） | 0 件 | 既に `:focus-visible`/`:focus-within`/`data-focus-visible` 属性の 3 経路のみ |
| `outline: 2px solid var(--fandhe-color-accent)` | 42 ファイル | 最多パターン。palette 非対応部品の既定形 |
| `solid var(--fandhe-palette, var(--fandhe-color-accent))` | 12 ファイル | `ColorPalette` 対応部品（checkbox/checkbox-card/checkbox-group/radio-group/radio-card/slider/password-input/blockquote/callout/timeline/pagination/steps 等） |
| `outline: none`（単独） | 5 ファイル | combobox / date-input / password-input / tags-input / skip-nav |
| `outline-offset` が `2px` 以外 | 4 ファイル | splitter/listbox/scroll-area は `-2px`（inset）、image-cropper は `1px` |
| `forced-colors` への言及 | 0 件（本イシューの新規コメント除く） | `@media (forced-colors: active)` の明示ルールは存在しない |
| `--fandhe-color-focus-ring` への事前参照 | 1 ファイル（`date_input.rs`） | `box-shadow: 0 0 0 2px var(--fandhe-color-focus-ring, var(--fandhe-color-accent))` という形で**未定義のトークンへフォールバック付きで先行参照**していた（下記 §5 参照） |

`box-shadow` によるリング表現は `outline: none` の代替として date-input の
`segment` slot 1 箇所のみで使われており（インライン `<input>` セグメントの
レイアウトを崩さないための意図的選択）、それ以外の 106 ファイルは
`outline` に事実上収斂している。

`recipe::Size` は `Sm`/`Md`/`Lg` の 3 値固定（`default_variant(Size::Md)` が
56 箇所、既定は `md` で事実上統一済み）。Typography は独立軸
`TextSize`/`HeadingSize`（`text.rs`/`heading.rs`）を持つ。

**段階数の拡張（xs/xl 追加・ColorPalette 拡張）は #1426（判断）と #1678
（enum・トークン定義）のスコープ**であり、本イシューは名称規約・既定・
保有判定基準・共通生成手段までに留め、段階数の変更は行わない
（#1423 と同じ理由でスコープの重複を避ける）。

## 3. フォーカスリング規約

| 項目 | 決定 | 根拠 |
|------|------|------|
| 実装手段 | `outline` + `outline-offset` に統一。新規に `box-shadow` によるリングを追加しない | 既存 106/107 ファイルが `outline` に収斂済み。`overflow: hidden` で切れない。`forced-colors: active` では `outline` の色がシステム色（`Highlight` 等）へ強制置換されて**必ず描画される**のに対し `box-shadow` は除去されるため、高コントラスト要件を構造的に満たす。Radix Themes・chakra `focusRing="outside"` も outline ベース |
| トークン | `--fandhe-focus-ring-width: 2px` / `--fandhe-focus-ring-offset: 2px`（モード非依存、`theme.rs` の `focus_ring` グループ）/ `--fandhe-color-focus-ring`（light `#3182ce` / dark `#63b3ed`、`colors` グループ、ダークモード追従） | 太さ・オフセットを 1 箇所で変更可能にする。色は `colors` グループが担うことで既存の `write_dark_declarations` を再利用し、ダークモード追従を二重実装しない |
| 既定宣言（canonical） | `outline: var(--fandhe-focus-ring-width) solid var(--fandhe-color-focus-ring); outline-offset: var(--fandhe-focus-ring-offset);` | リテラル値をハードコードせず、太さ・オフセット・色をテーマ側 1 箇所で変更できる |
| ColorPalette 対応部品 | `outline-color`（＝ `outline` の色部分）を `var(--fandhe-palette, var(--fandhe-color-focus-ring))` にする variant を許容 | 既存 12 ファイルの palette 連動を維持しつつ、フォールバック先を新トークンへ揃える |
| inset（内側）リング | `outline-offset: calc(-1 * var(--fandhe-focus-ring-offset))` | splitter/listbox/scroll-area の 3 件。独立トークンを増やさず符号反転で表現する |
| 適用セレクタ | `:focus-visible`（`StateCondition::FocusVisible`）を既定。hidden-input パターン（実フォーカスが visually-hidden な `<input>` にある構成）は `Attr("data-focus-visible")`、visually-hidden 子孫を持つ `<label>` 等祖先は `FocusWithin`。`:focus` 直書きは禁止（現状 0 件を維持） | 既存 `recipe.rs` の `StateCondition` 契約を規約として明文化するのみで、新しい概念は導入しない |
| `outline: none` | 「同一部品内の祖先 slot に canonical リングが `:focus-within` で存在する」場合のみ許容。単独使用は禁止（`forced-colors` でリングが消える唯一の経路のため） | combobox/date-input/password-input/tags-input の 4 件はこのパターン。ただし **skip-nav の `content` slot は例外**: `tabindex="-1"` で視覚的内容を持たないプログラム的フォーカスターゲットであり、そもそも視認可能なリングを表示する意味がないため祖先リングの有無を問わず許容する（§6 参照） |
| virtual focus（Menu/Select/Listbox/Combobox の `item`） | `[data-highlighted]` の背景表現を維持し、`item` へリングは付けない | 既存 `recipe.rs` `StateCondition::FocusVisible` doc の契約をそのまま踏襲 |

### 3.1 `date_input.rs` の `box-shadow` 先行参照について

`date_input.rs` の `segment` slot は本イシュー以前から
`box-shadow: 0 0 0 2px var(--fandhe-color-focus-ring, var(--fandhe-color-accent))`
という、当時未定義だった `--fandhe-color-focus-ring` へのフォールバック
付き参照を持っていた（インライン `<input>` セグメントは `outline` だと
`box-sizing: border-box` の枠内でクリップされるため意図的に `box-shadow`
を採用したと推測される）。本イシューで `--fandhe-color-focus-ring` を
実トークンとして定義したことで、この宣言は「未定義トークンへのフォール
バック」から「実際に定義済みトークンを参照する」形へ意味が変わる
（値自体は light では accent と同値のため見た目は変わらず、dark では
`#4299e1`→`#63b3ed` へわずかに変化する）。`segment` は `outline: none` を
別途持つため祖先リングとの整合は保たれるが、**手段が `box-shadow` である
点は §3 の canonical 形と異なる**。date-input 自体の `outline`/`box-shadow`
移行は Phase 1 以降の個別部品 issue のスコープとし、本書 §7 のスコープ外
一覧へ記載する。

## 4. size 規約

| 項目 | 決定 |
|------|------|
| 名称セット | `xs` < `sm` < `md` < `lg` < `xl` の順序語彙を規約として予約する。**現行の `recipe::Size` enum は `Sm`/`Md`/`Lg` のまま**（拡張は #1426/#1678 のスコープ） |
| 既定 | 全部品 `md`（`default_variant(Size::Md)`）。`sm`/`lg` は必ず `md` を基準に相対定義する |
| 保有判定基準 | (a) 利用者が操作する control・入力（Button/Input/Checkbox 等）: **持つ**。(b) 内部密度（padding/font-size/高さ）が可変な container・data-display（Card/Alert/Table 等）: **持つ**（未実装分は Phase 1 で追加）。(c) 型階層を持つ Typography（Text/Heading、および blockquote/quote/em/strong/mark/highlight/list 等の周辺部品）: `size` 軸は持たず専用軸（`TextSize`/`HeadingSize` 等）を使う。(d) 構造・可視性のみの Utilities（VisuallyHidden/LinkOverlay/SkipNav/Separator/ScrollArea/Marquee/Portal 相当）と、子の寸法に従属するレイアウト部品（Toolbar/ActionBar/NavList/Menubar/NavigationMenu 等の root）: **持たない** |
| 共通生成手段 | [`recipe::SlotRecipe::size_variants`]（本イシューで新設）: `(Size, Vec<Declaration>)` の組を渡すと各 size を `variant` として登録したうえで `default_variant(Size::Md)` を必ず設定する。既存 `variant`/`default_variant` API はそのまま残る（本メソッドは追加の共通手段であり、既存 API を置き換えない） |

Phase 1 で size 軸追加の候補となる (b) 該当部品の例: alert / card /
hover-card / popover / tooltip / toast / tour / toggle-tip / floating-panel
/ data-list / json-tree-view / tree-view / timer / tab-nav / kbd / code /
clipboard / color-picker / signature-pad / skeleton / image 等。分類の
最終確定は各部品の rustdoc・anatomy を確認したうえで Phase 1 の個別 issue
で行う（本書は判定基準の提示までがスコープ）。

## 5. `forced-colors` 方針

現時点で `@media (forced-colors: active)` の明示ルールは追加しない。
`outline` を実装手段として選定したこと自体が、Windows High Contrast Mode
等の `forced-colors: active` でブラウザが `outline` の色をシステム色
（`Highlight`/`GrayText` 等）へ強制置換し**必ず描画する**という UA 既定動作
に乗る設計であり、追加の `@media` ルールなしでもフォーカス可視性が失われ
ない。将来、実機検証（#1428 のスクリーンショット手順等）で `outline-color:
Highlight` の明示指定が必要と判明した場合は、本書 §7 の再評価トリガーと
して別 issue を提案する。

## 6. 移行手順（Phase 1 以降の各部品 issue が参照するチェックリスト）

1. 対象部品の `.state(slot, StateCondition::FocusVisible | FocusWithin |
   Attr("data-focus-visible"), vec![decl("outline", ...), decl("outline-offset", ...)])`
   を特定する。
2. `ColorPalette` 軸を持つ部品か判定する（`variant(palette, ...)` の有無）。
   - 持つ: `focus_ring_declarations(FocusRingColor::Palette,
     FocusRingOffset::Outside)` へ置換。
   - 持たない: `focus_ring_declarations(FocusRingColor::Token,
     FocusRingOffset::Outside)` へ置換。
3. `outline-offset` が `-2px` 相当（inset）の場合は `FocusRingOffset::Inset`
   を使う。
4. `outline: none` 単独の部品（§3 表の 4 件 + skip-nav 例外）は据え置く
   （祖先側の canonical 化のみ行う）。`date_input.rs` の `box-shadow` 形は
   本イシューのスコープ外（§3.1 参照、Phase 1 の date-input 個別 issue で
   `outline` への移行可否を判断する）。
5. 対応する golden テスト（`tests/<component>_css.rs`、および
   `src/<component>.rs` 内 `#[cfg(test)]` の inline golden アサーション）の
   `outline`/`outline-offset` 期待値を canonical 形（トークン参照）へ更新
   する。
6. 部品モジュールの rustdoc（フォーカスリング関連の記述がある場合）に
   本書への参照を追記する。
7. `cargo test -p fandhe-frontend-pre-styled-ui` が green であることを確認
   する。

パイロット実装として `crates/pre-styled-ui/src/radio_group.rs`
（`item` slot: `FocusRingColor::Palette`、`item-control` slot:
`FocusRingColor::Token`）を本イシューで移行済み。他 106 部品への展開は
Phase 1 以降の個別 issue へ委ねる。

## 7. #1426・#1678・#1425 との境界

- **size 段階数の拡張（xs/xl・ColorPalette 拡張）**: #1426（判断）→
  #1678〜#1681（enum・トークン定義）。本書は名称語彙・既定・判定基準の
  提示までで、`recipe::Size` enum 自体の拡張は行わない。
- **hover/disabled/transition 規約**: #1425。本書はフォーカス状態と
  size バリアントに限定する。
- **`forced-colors` 向けの追加ルール**（`@media (forced-colors: active)`
  での `outline-color: Highlight` 明示等）が実機検証で必要と判明した場合は、
  本書 §5 の再評価トリガーとして別 issue を提案する。
- **107 部品の `focus_ring_declarations` への一括移行と golden 更新**:
  Phase 1 以降の各部品 issue（本書 §6 の移行手順を適用）。
- **`date_input.rs` の `box-shadow` → `outline` 移行可否**: Phase 1 の
  date-input 個別 issue（§3.1 参照）。

## 8. 棚卸し再現コマンド

```bash
cd crates/pre-styled-ui
grep -rn ':focus\b[^-]' src | grep -v 'focus-visible\|focus-within'   # 直書き（期待 0 件）
grep -rln 'decl("outline", "2px solid var(--fandhe-color-accent)")' src
grep -rln 'solid var(--fandhe-palette' src
grep -rln '"outline", "none"' src
grep -rn 'outline-offset", "-2px"\|outline-offset", "1px"' src
grep -rn 'forced-colors' src                                          # 期待 0 件（本書由来のコメントを除く）
grep -rln 'color-focus-ring' src
```
