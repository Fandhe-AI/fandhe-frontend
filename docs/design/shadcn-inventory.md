# shadcn/ui 全コンポーネント一次調査記録

**本文書のステータス**: 調査記録（イシュー #2004、親 #2002、トラッキング
#2001）。`docs/design/component-coverage-map.md`（ark-ui / chakra-ui /
Radix UI の 3 参照軸）に対する第 4 の参照軸として shadcn/ui を組み込む
準備の一部であり、本書は **shadcn/ui コンポーネントドキュメント一覧**の
一次調査記録を担う。shadcn/ui の位置づけは「補完参照」（主基準化はしない、
`docs/design/shadcn-reference-adoption-policy.md` 参照）である。

**本書は調査・記録のみであり、`crates/` 配下は一切変更しない。区分判定
（実装済み / 実装対象 / 保留 / 意図的非採用 / 参照対象外 / 対象外）は
`component-coverage-map.md` §5・§12 の責務とする。**

## 1. 本文書の位置付けと出典の pin

一次ソースは `shadcn-ui/ui` リポジトリの
`apps/v4/content/docs/components/` 配下（shadcn/ui v4、Base UI / Radix UI /
React Aria の 3 実装バリアントを切り替えられる現行ドキュメント構成）。

- 出典 URL の正: `https://ui.shadcn.com/docs/components/<slug>`
- 裏取り元: `apps/v4/content/docs/components/{base,radix,aria}/*.mdx`
  （3 サブディレクトリは同一コンポーネントの実装バリアント別ページであり、
  別コンポーネントではない。列挙は 3 者の**和集合**を正とする）

**pin**: `shadcn-ui/ui` commit `5c7072da672b0048bc6771e3204063a2537df91a`
（短縮 `5c7072d`）/ 取得日 2026-09-07。

再 pin（commit を進める）は意図的な改訂行為として扱う。既存
`component-coverage-map.md` §1 の commit `ab53c6b` pin・
`radix-primitives-inventory.md` の commit `bb42408` pin と同じ規律に従う。

## 2. スコープ

本書が扱うのは `apps/v4/content/docs/components/{base,radix,aria}/*.mdx`
（コンポーネントページ）のみとする。以下は対象外（別イシュー・別軸の責務）:

- **Blocks**（`apps/v4/content/docs/registry` 配下のダッシュボード・
  サイドバー・ログイン・サインアップ等の複合ページテンプレート）:
  `component-coverage-map.md` の行モデル（部品 1 件 = mod 1 件）に
  そぐわないため対象外。置き場所の判断は #2007 が担当する
- **Charts の個別バリアント**（`chart.mdx` 内の area/bar/line/pie/radar
  デモ）: v4 では単一ページ `chart.mdx` に統合され、旧 v3 のような
  「Area Chart」「Bar Chart」等の個別見出し（アンカー）を持たない構成へ
  変更されている（2026-09-07 時点で実測確認済み。`## Tooltip` /
  `## Legend` 等の見出しはあるが個別チャート種別の見出しはない）。
  このため本書では `chart` を単一の slug として扱い、`chart#area` のような
  アンカー付き表記は用いない（起票時点の計画が前提としていた v3 時代の
  ページ構成とは実装時点で乖離していたための判断。詳細は
  `component-coverage-map.md` §5 冒頭の表記規約・§12.1 の Radial Chart 行
  の注記を参照）
- **Theming ページ**（`chart.mdx` 内 `## Theming` 節、独立ページなし）:
  対象外
- **Forms 統合ガイド**（`apps/v4/content/docs/forms/react-hook-form.mdx` /
  `forms/tanstack-form.mdx`）: コンポーネントではなく外部フォームライブラリ
  との統合ガイド文書のため対象外。**shadcn/ui の v4 には旧 v3 に存在した
  独立コンポーネント `Form` が存在しない**（`apps/v4/content/docs/components`
  配下に `form.mdx` なし。`gh api search/code` でも
  `apps/v4/content/docs/components/**/form.mdx` はヒットせず、
  `forms/react-hook-form.mdx` へ置き換えられている）。起票時点の計画は
  旧 v3 の `Form` コンポーネント存在を前提としていたが、実装時点の一次
  ソース確認によりこれを訂正する。Radix Primitives 側の `Form`
  （`component-coverage-map.md` Part D、意図的非採用確定済み）には
  この訂正後も shadcn 値を記入しない（対応する shadcn ページが実在しない
  ため）
- **`typeset`**: 起票時点の計画が言及していたが、実装時点の一次ソースには
  存在しない（`apps/v4/content/docs/components` 配下に該当ページなし）。
  上記 `Form` と同様、計画段階の誤情報として訂正する

## 3. 件数サマリ（スカラの検証用アサーション）

| ディレクトリ | 件数（`meta.json` を除く） |
|---|---|
| `components/base` | 63 |
| `components/radix` | 64 |
| `components/aria` | 62 |
| **和集合（正）** | **65** |

3 バリアントは同一コンポーネントの実装差分（例: `radix` のみ `sonner.mdx`
を持つ、`aria`/`radix` は `navigation-menu.mdx` を持たない）であり、本書
§4 の一覧は和集合 65 件を正とする。取得コマンド:

```bash
gh api "repos/shadcn-ui/ui/contents/apps/v4/content/docs/components/base?ref=5c7072da672b0048bc6771e3204063a2537df91a" --jq '.[].name'
gh api "repos/shadcn-ui/ui/contents/apps/v4/content/docs/components/radix?ref=5c7072da672b0048bc6771e3204063a2537df91a" --jq '.[].name'
gh api "repos/shadcn-ui/ui/contents/apps/v4/content/docs/components/aria?ref=5c7072da672b0048bc6771e3204063a2537df91a" --jq '.[].name'
```
（各出力から `meta.json` を除外し、拡張子 `.mdx` を除いた和集合を取る）

## 4. コンポーネント一覧（65 件）

参考: 「区分ヒント」欄は `component-coverage-map.md` §5・§12 での区分判定
作業を補助する非確定の手掛かりであり、確定した判定ではない
（`radix-primitives-inventory.md` §3 と同じ規約）。

| slug | 表示名（見立て） | 区分ヒント |
|---|---|---|
| `accordion` | Accordion | 既存 `accordion` と対応 |
| `alert` | Alert | 既存 `alert` と対応 |
| `alert-dialog` | Alert Dialog | 既存 `dialog`（Alert Dialog variant）と対応 |
| `aspect-ratio` | Aspect Ratio | 既存の意図的非採用（layout プリミティブ、#716/#724） |
| `attachment` | Attachment | 会話系部品。#2110 で実装対象確定 |
| `avatar` | Avatar | 既存 `avatar` と対応 |
| `badge` | Badge | 既存 `badge` と対応 |
| `breadcrumb` | Breadcrumb | 既存 `breadcrumb` と対応 |
| `bubble` | Bubble | 会話系部品。#2107 で実装対象確定 |
| `button` | Button | 既存 `button` と対応 |
| `button-group` | Button Group | 実装対象。#2058 |
| `calendar` | Calendar | 既存 `calendar` と対応 |
| `card` | Card | 既存 `card` と対応 |
| `carousel` | Carousel | 既存 `carousel` と対応 |
| `chart` | Chart | 既存 `charts::*`（area/bar/donut/line/pie チャート）と対応。個別種別のアンカーなし（§2 参照） |
| `checkbox` | Checkbox | 既存 `checkbox` と対応 |
| `collapsible` | Collapsible | 既存 `collapsible` と対応 |
| `combobox` | Combobox | 既存 `combobox` と対応 |
| `command` | Command | 実装対象。#2067 |
| `context-menu` | Context Menu | 既存 `menu` で充足（Phase 1/2） |
| `data-table` | Data Table | 実装対象（構造・DOM 配線のみ）。#2124 |
| `date-picker` | Date Picker | 既存 `date_picker` と対応 |
| `dialog` | Dialog | 既存 `dialog` と対応 |
| `direction` | Direction | Radix Primitives `Direction Provider` と対応（既存保留行） |
| `drawer` | Drawer | 既存 `drawer` で充足。#2031 |
| `dropdown-menu` | Dropdown Menu | 既存 `menu` で充足（Phase 1/2） |
| `empty` | Empty | 既存 `empty_state` で充足。#2047 |
| `field` | Field | 既存 `field` と対応 |
| `hover-card` | Hover Card | 既存 `hover_card` と対応 |
| `input` | Input | 既存 `input` と対応 |
| `input-group` | Input Group | 実装対象。#2061 |
| `input-otp` | Input OTP | 既存 `pin_input` で充足。#2016 |
| `item` | Item | 実装対象。#2064 |
| `kbd` | Kbd | 既存 `kbd` と対応 |
| `label` | Label | 既存 `field` で充足（Radix Themes 名も Label）。#2014 |
| `marker` | Marker | 会話系部品。#2113 で実装対象確定 |
| `menubar` | Menubar | 既存 `menubar` と対応 |
| `message` | Message | 会話系部品。#2104 で実装対象確定 |
| `message-scroller` | Message Scroller | 会話系部品。#2120 で実装対象確定 |
| `native-select` | Native Select | 既存 `native_select` と対応 |
| `navigation-menu` | Navigation Menu | 既存 `navigation_menu` と対応 |
| `pagination` | Pagination | 既存 `pagination` と対応 |
| `popover` | Popover | 既存 `popover` と対応 |
| `progress` | Progress | 既存 `progress` と対応 |
| `questionnaire` | Questionnaire | 実装対象。#2116 |
| `radio-group` | Radio Group | 既存 `radio_group` と対応 |
| `resizable` | Resizable | 既存 `splitter` で充足。#2038 |
| `scroll-area` | Scroll Area | 既存 `scroll_area` と対応（`scroll-fade` utility 相当は pre-styled-ui `scroll_area` の注記として記載） |
| `select` | Select | 既存 `select` と対応 |
| `separator` | Separator | 既存 `separator` と対応 |
| `sheet` | Sheet | 既存 `drawer` で充足。#2031 |
| `sidebar` | Sidebar | 実装対象。#2071 |
| `skeleton` | Skeleton | 既存 `skeleton` と対応（`shimmer` utility 相当は注記として記載） |
| `slider` | Slider | 既存 `slider` と対応 |
| `sonner` | Sonner | 既存 `toast` で充足。#2040 |
| `spinner` | Spinner | 既存 `spinner` と対応 |
| `switch` | Switch | 既存 `switch` と対応 |
| `table` | Table | 既存 `table` と対応 |
| `tabs` | Tabs | 既存 `tabs` と対応 |
| `textarea` | Textarea | 既存 `textarea` と対応 |
| `toast` | Toast | 既存 `toast` と対応 |
| `toggle` | Toggle | 既存 `toggle` と対応 |
| `toggle-group` | Toggle Group | 既存 `toggle_group` と対応 |
| `tooltip` | Tooltip | 既存 `tooltip` と対応 |
| `typography` | Typography | 既存 `heading`/`text`/`blockquote`/`list`/`code` 等の複数 mod に分散対応。個別バリアント（Lead/Large/Small/Muted）は未実装 |

## 5. 除外・訂正事項（起票時点の計画との差分）

- `form`: v4 に独立コンポーネントとして存在しない（§2 参照）。計画段階の
  誤情報として訂正し、Radix Primitives `Form` 行（既存の意図的非採用）へ
  shadcn 値を追記しない
- `typeset`: v4 に存在しない。計画段階の誤情報として訂正する
- Charts の個別バリアント（`chart#area` 等）: v4 では単一ページに統合され
  個別アンカーを持たない（§2 参照）。`component-coverage-map.md` の
  表記規約はアンカー付き slug 表記（`` `chart#anchor` `` 形式）を許容する
  拡張を持つが、typography の一部見出し（h1/blockquote/list/inline code 等）
  にのみ適用し、chart には適用しない（見出し自体が存在しないため）
