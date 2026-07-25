# Radix Primitives 全コンポーネント一次調査記録

**本文書のステータス**: 調査記録（イシュー #935、親 #925、トラッキング #924）。
`docs/design/component-coverage-map.md`（ark-ui / chakra-ui の 2 軸）に対する
第 3 の参照軸として Radix UI を組み込む準備の一部であり、本書は
**Radix Primitives（unstyled・anatomy・`data-*`・WAI-ARIA 層、
`fandhe-frontend-headless-ui` に対応）側の一次調査記録**を担う。
Radix Themes 側の調査記録は別イシュー（#936）が別文書として作る。

**本書は調査・記録のみであり、`crates/` 配下は一切変更しない。区分判定
（実装済み / 実装対象 / 保留 / 意図的非採用 / 参照対象外 / 対象外）は行わない。
判定は #937（`component-coverage-map.md` への Radix 列追加）の責務とする。**

## 1. 本文書の位置付けと出典の pin

一次ソースは 2 系統ある。

- (a) `https://www.radix-ui.com/primitives/docs/components` 配下の各ページ
  （出典 URL の正。人間・AI が実際に参照するのはこちら）
- (b) `radix-ui/website` リポジトリの `data/primitives/docs/**.mdx`
  （docs ページの原稿。列挙の完全性と anatomy・`data-*` 属性名の文字列を
  裏取りするために用いる。パーツ名・属性名は API Reference セクションの
  `DataAttributesTable` の `attribute: "[data-xxx]"` 記載を機械抽出した）

**pin**: `radix-ui/website` commit `bb424082fd33fadc244a6dd276d3ced55caa6234`
（短縮 `bb42408`）/ 取得日 2026-07-25。

再 pin（commit を進める）は意図的な改訂行為として扱う。既存
`component-coverage-map.md` §1 の commit `ab53c6b` pin と同じ規律に従う。

## 2. 件数サマリ（スカラの検証用アサーション）

| ディレクトリ | 件数 |
|---|---|
| components | 30 |
| utilities | 5 |
| overview | 4 |
| guides | 4 |
| **合計** | **43** |

`gh api 'repos/radix-ui/website/contents/data/primitives/docs/components?ref=bb42408'`
（他ディレクトリも同様）で取得した各ディレクトリのファイル名一覧は本書 §4〜§6
の行数と一致する。加えて `https://www.radix-ui.com/primitives/docs/components`
の一覧ページ（WebFetch 取得）に列挙される 30 件のコンポーネント名が、
`components/` 配下の mdx 30 件（拡張子を除いたファイル名）と一致することを
確認済み。

## 3. 記録の規約

- 1 mdx = 1 行（既存 `component-coverage-map.md` §5 と同じ粒度。#937 が
  機械的にマージできる形）
- anatomy パーツは Anatomy コードサンプルに現れる `<Xxx.Part>` の `Part`
  部分を出現順に列挙したもの（`>` 区切り）
- 公開 `data-*` はパートごとに `Part: [data-attr] [data-attr] ...` の形で
  1 セルへフラット化し、パート間は改行（`<br>`）で区切る
- **anatomy・`data-*` が存在しない文書は `—` を記録し、推測で補わない**
  （`aspect-ratio` / `avatar` / `label` のように data 属性を持たない部品、
  utilities 5 件のように anatomy はあるが公開 `data-*` を持たない文書がある）
- 「参考: fandhe headless-ui 候補 mod」列は**非確定の手掛かり**であり、
  区分判定は #937 が行う。名称が明白に対応するもののみ機械的に記入し、
  判断が割れるものは `?（#937 で判定）` と記す

## 4. Part A: Components（30 件）

| 参照 mdx パス | Radix 名 | 出典 URL | anatomy パーツ | 公開 data-*（パート: 属性） | 参考: fandhe headless-ui 候補 mod（非確定） |
|---|---|---|---|---|---|
| `data/primitives/docs/components/accordion.mdx` | Accordion | https://www.radix-ui.com/primitives/docs/components/accordion | Root > Item > Header > Trigger > Content | Root: [data-orientation]<br>Item: [data-state] [data-disabled] [data-orientation]<br>Header: [data-state] [data-disabled] [data-orientation]<br>Trigger: [data-state] [data-disabled] [data-orientation]<br>Content: [data-state] [data-disabled] [data-orientation] | accordion |
| `data/primitives/docs/components/alert-dialog.mdx` | Alert Dialog | https://www.radix-ui.com/primitives/docs/components/alert-dialog | Root > Trigger > Portal > Overlay > Content > Title > Description > Cancel > Action | Trigger: [data-state]<br>Overlay: [data-state]<br>Content: [data-state] | ?（#937 で判定） |
| `data/primitives/docs/components/aspect-ratio.mdx` | Aspect Ratio | https://www.radix-ui.com/primitives/docs/components/aspect-ratio | Root | — | ?（#937 で判定） |
| `data/primitives/docs/components/avatar.mdx` | Avatar | https://www.radix-ui.com/primitives/docs/components/avatar | Root > Image > Fallback | — | avatar |
| `data/primitives/docs/components/checkbox.mdx` | Checkbox | https://www.radix-ui.com/primitives/docs/components/checkbox | Root > Indicator | Root: [data-state] [data-disabled]<br>Indicator: [data-state] [data-disabled] | checkbox |
| `data/primitives/docs/components/collapsible.mdx` | Collapsible | https://www.radix-ui.com/primitives/docs/components/collapsible | Root > Trigger > Content | Root: [data-state] [data-disabled]<br>Trigger: [data-state] [data-disabled]<br>Content: [data-state] [data-disabled] | collapsible |
| `data/primitives/docs/components/context-menu.mdx` | Context Menu | https://www.radix-ui.com/primitives/docs/components/context-menu | Root > Trigger > Portal > Content > Label > Item > Group > CheckboxItem > ItemIndicator > RadioGroup > RadioItem > Sub > SubTrigger > SubContent > Separator | Trigger: [data-state]<br>Content: [data-state] [data-side] [data-align]<br>Item: [data-highlighted] [data-disabled]<br>CheckboxItem: [data-state] [data-highlighted] [data-disabled]<br>ItemIndicator: [data-state]<br>RadioItem: [data-state] [data-highlighted] [data-disabled]<br>SubTrigger: [data-state] [data-highlighted] [data-disabled]<br>SubContent: [data-state] [data-side] [data-align] | ?（#937 で判定） |
| `data/primitives/docs/components/dialog.mdx` | Dialog | https://www.radix-ui.com/primitives/docs/components/dialog | Root > Trigger > Portal > Overlay > Content > Title > Description > Close | Trigger: [data-state]<br>Overlay: [data-state]<br>Content: [data-state] | dialog |
| `data/primitives/docs/components/dropdown-menu.mdx` | Dropdown Menu | https://www.radix-ui.com/primitives/docs/components/dropdown-menu | Root > Trigger > Portal > Content > Label > Item > Group > CheckboxItem > ItemIndicator > RadioGroup > RadioItem > Sub > SubTrigger > SubContent > Separator > Arrow | Trigger: [data-state] [data-disabled]<br>Content: [data-state] [data-side] [data-align] [data-orientation]<br>Item: [data-orientation] [data-highlighted] [data-disabled]<br>CheckboxItem: [data-state] [data-highlighted] [data-disabled]<br>ItemIndicator: [data-state]<br>RadioItem: [data-state] [data-highlighted] [data-disabled]<br>SubTrigger: [data-state] [data-highlighted] [data-disabled]<br>SubContent: [data-state] [data-side] [data-align] [data-orientation] | menu（Dropdown Menu 相当） |
| `data/primitives/docs/components/form.mdx` | Form | https://www.radix-ui.com/primitives/docs/components/form | Root > Field > Label > Control > Message > ValidityState > Submit | Field: [data-invalid] [data-valid]<br>Label: [data-invalid] [data-valid]<br>Control: [data-invalid] [data-valid] | ?（#937 で判定） |
| `data/primitives/docs/components/hover-card.mdx` | Hover Card | https://www.radix-ui.com/primitives/docs/components/hover-card | Root > Trigger > Portal > Content > Arrow | Trigger: [data-state]<br>Content: [data-state] [data-side] [data-align] | hover_card |
| `data/primitives/docs/components/label.mdx` | Label | https://www.radix-ui.com/primitives/docs/components/label | Root | — | ?（#937 で判定） |
| `data/primitives/docs/components/menubar.mdx` | Menubar | https://www.radix-ui.com/primitives/docs/components/menubar | Root > Menu > Trigger > Portal > Content > Label > Item > Group > CheckboxItem > ItemIndicator > RadioGroup > RadioItem > Sub > SubTrigger > SubContent > Separator > Arrow | Trigger: [data-state] [data-highlighted] [data-disabled]<br>Content: [data-state] [data-side] [data-align]<br>Item: [data-highlighted] [data-disabled]<br>CheckboxItem: [data-state] [data-highlighted] [data-disabled]<br>ItemIndicator: [data-state]<br>RadioItem: [data-state] [data-highlighted] [data-disabled]<br>SubTrigger: [data-state] [data-highlighted] [data-disabled]<br>SubContent: [data-state] [data-side] [data-align] [data-orientation] | ?（#937 で判定） |
| `data/primitives/docs/components/navigation-menu.mdx` | Navigation Menu | https://www.radix-ui.com/primitives/docs/components/navigation-menu | Root > List > Item > Trigger > Content > Link > Sub > Viewport > Indicator | Root: [data-orientation]<br>List: [data-orientation]<br>Trigger: [data-state] [data-disabled]<br>Content: [data-state] [data-motion] [data-orientation]<br>Link: [data-active]<br>Sub: [data-orientation]<br>Viewport: [data-state] [data-orientation]<br>Indicator: [data-state] [data-orientation] | ?（#937 で判定） |
| `data/primitives/docs/components/one-time-password-field.mdx` | One-Time Password Field | https://www.radix-ui.com/primitives/docs/components/one-time-password-field | Root > Input > HiddenInput | Root: [data-orientation]<br>Input: [data-index] | pin_input（One-Time Password Field 相当） |
| `data/primitives/docs/components/password-toggle-field.mdx` | Password Toggle Field | https://www.radix-ui.com/primitives/docs/components/password-toggle-field | Root > Input > Toggle > Icon | — | password_input（Password Toggle Field 相当） |
| `data/primitives/docs/components/popover.mdx` | Popover | https://www.radix-ui.com/primitives/docs/components/popover | Root > Trigger > Anchor > Portal > Content > Close > Arrow | Trigger: [data-state]<br>Content: [data-state] [data-side] [data-align] | popover |
| `data/primitives/docs/components/progress.mdx` | Progress | https://www.radix-ui.com/primitives/docs/components/progress | Root > Indicator | Root: [data-state] [data-value] [data-max]<br>Indicator: [data-state] [data-value] [data-max] | progress |
| `data/primitives/docs/components/radio-group.mdx` | Radio Group | https://www.radix-ui.com/primitives/docs/components/radio-group | Root > Item > Indicator | Root: [data-disabled]<br>Item: [data-state] [data-disabled]<br>Indicator: [data-state] [data-disabled] | radio_group |
| `data/primitives/docs/components/scroll-area.mdx` | Scroll Area | https://www.radix-ui.com/primitives/docs/components/scroll-area | Root > Viewport > Scrollbar > Thumb > Corner | Scrollbar: [data-state] [data-orientation]<br>Thumb: [data-state] | scroll_area |
| `data/primitives/docs/components/select.mdx` | Select | https://www.radix-ui.com/primitives/docs/components/select | Root > Trigger > Value > Icon > Portal > Content > ScrollUpButton > Viewport > Item > ItemText > ItemIndicator > Group > Label > Separator > ScrollDownButton > Arrow | Trigger: [data-state] [data-disabled] [data-placeholder]<br>Content: [data-state] [data-side] [data-align]<br>Item: [data-state] [data-highlighted] [data-disabled] | select |
| `data/primitives/docs/components/separator.mdx` | Separator | https://www.radix-ui.com/primitives/docs/components/separator | Root | Root: [data-orientation] | ?（#937 で判定） |
| `data/primitives/docs/components/slider.mdx` | Slider | https://www.radix-ui.com/primitives/docs/components/slider | Root > Track > Range > Thumb | Root: [data-disabled] [data-orientation]<br>Track: [data-disabled] [data-orientation]<br>Range: [data-disabled] [data-orientation]<br>Thumb: [data-disabled] [data-orientation] | slider |
| `data/primitives/docs/components/switch.mdx` | Switch | https://www.radix-ui.com/primitives/docs/components/switch | Root > Thumb | Root: [data-state] [data-disabled]<br>Thumb: [data-state] [data-disabled] | switch |
| `data/primitives/docs/components/tabs.mdx` | Tabs | https://www.radix-ui.com/primitives/docs/components/tabs | Root > List > Trigger > Content | Root: [data-orientation]<br>List: [data-orientation]<br>Trigger: [data-state] [data-disabled] [data-orientation]<br>Content: [data-state] [data-orientation] | tabs |
| `data/primitives/docs/components/toast.mdx` | Toast | https://www.radix-ui.com/primitives/docs/components/toast | Provider > Root > Title > Description > Action > Close > Viewport | Root: [data-state] [data-swipe] [data-swipe-direction] | toast |
| `data/primitives/docs/components/toggle-group.mdx` | Toggle Group | https://www.radix-ui.com/primitives/docs/components/toggle-group | Root > Item | Root: [data-orientation]<br>Item: [data-state] [data-disabled] [data-orientation] | toggle_group |
| `data/primitives/docs/components/toggle.mdx` | Toggle | https://www.radix-ui.com/primitives/docs/components/toggle | Root | Root: [data-state] [data-disabled] | toggle |
| `data/primitives/docs/components/toolbar.mdx` | Toolbar | https://www.radix-ui.com/primitives/docs/components/toolbar | Root > Button > Separator > Link > ToggleGroup > ToggleItem | Root: [data-orientation]<br>Button: [data-orientation]<br>Separator: [data-orientation]<br>ToggleGroup: [data-orientation]<br>ToggleItem: [data-state] [data-disabled] [data-orientation] | ?（#937 で判定） |
| `data/primitives/docs/components/tooltip.mdx` | Tooltip | https://www.radix-ui.com/primitives/docs/components/tooltip | Provider > Root > Trigger > Portal > Content > Arrow | Trigger: [data-state]<br>Content: [data-state] [data-side] [data-align] | tooltip |

## 5. Part B: Utilities（5 件）

| 参照 mdx パス | Radix 名 | 出典 URL | anatomy パーツ | 公開 data-*（パート: 属性） | 参考: fandhe headless-ui 候補 mod（非確定） |
|---|---|---|---|---|---|
| `data/primitives/docs/utilities/accessible-icon.mdx` | Accessible Icon | https://www.radix-ui.com/primitives/docs/utilities/accessible-icon | Root | — | ?（#937 で判定） |
| `data/primitives/docs/utilities/direction-provider.mdx` | Direction Provider | https://www.radix-ui.com/primitives/docs/utilities/direction-provider | Provider | — | ?（#937 で判定） |
| `data/primitives/docs/utilities/portal.mdx` | Portal | https://www.radix-ui.com/primitives/docs/utilities/portal | Root | — | ?（#937 で判定） |
| `data/primitives/docs/utilities/slot.mdx` | Slot | https://www.radix-ui.com/primitives/docs/utilities/slot | Root | — | ?（#937 で判定） |
| `data/primitives/docs/utilities/visually-hidden.mdx` | Visually Hidden | https://www.radix-ui.com/primitives/docs/utilities/visually-hidden | Root | — | visually_hidden |

Slot と Portal は `docs/policy/intentional-non-adoption.md` の layout・
utilities 系の意図的非採用の判断軸に照らして「参照対象外」候補になり得るが、
**その判定自体は本書では行わない**（#937 のスコープ）。

## 6. Part C: Overview / Guides（8 件、非コンポーネント文書）

区分の想定は「対象外」（既存 `component-coverage-map.md` の README・guides と
同じ扱い）。本書では列挙のみ行い、区分の確定は #937 が行う。

| 参照 mdx パス | 出典 URL |
|---|---|
| `data/primitives/docs/overview/accessibility.mdx` | https://www.radix-ui.com/primitives/docs/overview/accessibility |
| `data/primitives/docs/overview/getting-started.mdx` | https://www.radix-ui.com/primitives/docs/overview/getting-started |
| `data/primitives/docs/overview/introduction.mdx` | https://www.radix-ui.com/primitives/docs/overview/introduction |
| `data/primitives/docs/overview/releases.mdx` | https://www.radix-ui.com/primitives/docs/overview/releases |
| `data/primitives/docs/guides/animation.mdx` | https://www.radix-ui.com/primitives/docs/guides/animation |
| `data/primitives/docs/guides/composition.mdx` | https://www.radix-ui.com/primitives/docs/guides/composition |
| `data/primitives/docs/guides/server-side-rendering.mdx` | https://www.radix-ui.com/primitives/docs/guides/server-side-rendering |
| `data/primitives/docs/guides/styling.mdx` | https://www.radix-ui.com/primitives/docs/guides/styling |

## 7. 合成パターン（asChild / Slot）の記録

出典: `guides/composition.mdx`（`https://www.radix-ui.com/primitives/docs/guides/composition`）・
`utilities/slot.mdx`（`https://www.radix-ui.com/primitives/docs/utilities/slot`）。

### 7.1 事実（Radix 側）

- DOM 要素を描画する Radix primitive の全パートは `asChild` prop を受け付ける。
  `asChild={true}` のとき、Radix は既定の DOM 要素を描画せず、代わりにその
  パートの子要素をクローンし、機能に必要な props・振る舞いをそこへ渡す
- `asChild` の用途は 2 つ: (1) 要素の種類を変える（例:
  `Tooltip.Trigger` を既定の `button` から `a` へ）、(2) 自作 React
  コンポーネントへ Radix の機能を合成する
- 自作コンポーネントへ合成する場合、コンポーネント側は「全 props を
  下位 DOM ノードへスプレッドすること」「`React.forwardRef` で `ref` を
  転送すること」の 2 条件を満たす必要がある（満たさないと壊れる）
- `asChild` は多重にネストできる（例: `Tooltip.Trigger asChild` の子に
  `Dialog.Trigger asChild` を置き、さらにその子に自作ボタンを置く）
- `Slot`（`radix-ui` パッケージの `Slot.Root`）は `asChild` パターンを
  自作コンポーネント側で実装するための下位ユーティリティであり、
  `const Comp = asChild ? Slot.Root : "button";` のように分岐して使う。
  `Slot.Slottable` は複数の子要素のうちどれをマージ対象にするかを
  明示するための補助コンポーネント

### 7.2 fandhe-frontend-headless-ui 側の対応構造（事実の併記のみ）

- `crates/headless-ui/src/anatomy.rs` の `Anatomy::part` は
  `data-scope` / `data-part` を固定属性として付与する薄い委譲層であり、
  `scope` / `part` はいずれも `&'static str` に固定されている（動的な
  セレクタ差し替えを型レベルで塞ぐ設計）
- `crates/headless-ui/src/data_attrs.rs` は `data-state` / `data-disabled`
  等の状態属性を組み立てる関数群を提供し、`Anatomy::part` の `attrs`
  引数へ合成する形で使う
- 上記はいずれも Rust のノード木 API（`fandhe_frontend_core::el`）を
  1 回呼ぶ薄い委譲であり、Radix の `asChild`／`Slot` に相当する
  「要素種別の差し替え・子要素への props マージ」という仕組み自体は
  現時点の `fandhe-frontend-headless-ui` には存在しない

**再導入・採用の提案はここでは書かない**（`.claude/rules/coding-rust.md`
「意図的非採用機能の再導入提案には評価軸の充足確認が必須」、および #936
の受入条件「実装候補として扱っていない」に合わせる）。asChild／Slot 相当の
仕組みの要否は #937 以降の判定・別途の設計検討に委ねる。

## 8. 完全性の機械確認手順（#937 が参照する検証コマンド）

```bash
# 文書に列挙された mdx パスと、pin した commit の実ファイル一覧が一致すること（diff が空）
diff <(grep -oE 'data/primitives/docs/[a-z-]+/[a-z0-9-]+\.mdx' \
         docs/design/radix-primitives-inventory.md | sort -u) \
     <(for d in components utilities overview guides; do \
         gh api "repos/radix-ui/website/contents/data/primitives/docs/$d?ref=bb42408" \
           --jq ".[] | \"data/primitives/docs/$d/\(.name)\""; \
       done | sort)
```

- `ref=bb42408` を必ず付ける（`main` を見ると Radix 側の追加で赤くなり、
  fandhe 側の記載漏れと誤認される）
- `api.github.com` へ到達できない場合は**環境エラー**であり FAIL ではない
  （`.claude/rules/ci.md` の crates.io 到達性と同じ分類）

出典 URL の解決性確認（Part A / Part B 全 35 行）:

```bash
grep -oE 'https://www\.radix-ui\.com/primitives/docs/[a-z-]+/[a-z0-9-]+' \
  docs/design/radix-primitives-inventory.md | sort -u \
  | while read -r u; do printf '%s %s\n' "$(curl -s -o /dev/null -w '%{http_code}' "$u")" "$u"; done \
  | grep -v '^200 ' || echo "all 200"
```

## 9. 更新方針

Radix Primitives 側の更新（新規コンポーネント追加・anatomy 変更等）は、
本文書の改訂を目的とした issue を起票して追随する。CI による自動検知は
行わない（既存 `component-coverage-map.md` §7 と同方針）。
