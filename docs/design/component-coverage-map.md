# ark-ui / chakra-ui 全コンポーネント対応表

**本文書のステータス**: 確定（イシュー #734、親 #733/#726）。正はコミット
`ab53c6b` 時点の `.agents/skills/ark-ui|chakra-ui/references/` 一覧
（ark-ui 90 件 / chakra-ui 269 件 / 計 359 件）。以後この文書を
Phase 3〜6（#736/#748/#757/#766 配下、#737〜#747・#749〜#756・#758〜#765・
#767〜#776 の 37 issue 相当）実装の正とする。

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
| 保留 | 実装するか否かを本書時点では確定せず、`#735`（保留・意図的非採用の評価軸と再評価トリガー記録）の評価対象とする |
| 意図的非採用 | 既に非採用と確定済み（layout プリミティブ = #716/#724 で確定済み等）。再導入提案には `docs/policy/intentional-non-adoption.md` の評価軸充足確認が必須 |
| 対象外 | README・guides・overview・get-started・concepts 等、UI コンポーネントを指さない非コンポーネント文書 |

「保留」区分の評価軸・再評価トリガーの詳細記録は後続イシュー #735 の担当
であり、本書からは前方参照のみを行う（本書は区分の確定と根拠の要約に留め
る）。

## 3. 実装済み部品と lib.rs の突合手順

`crates/headless-ui/src/lib.rs` の基盤 mod（anatomy / aria / data_attrs /
positioning / state）を除く 34 mod、`crates/pre-styled-ui/src/lib.rs` の
基盤 mod（css / recipe / stylesheet / theme）を除く 42 mod（styled ラッパー
32 + 静的部品 10）が、本書の「実装済み」区分と一致することを次のコマンドで
確認できる。

```bash
grep -E '^pub mod ' crates/headless-ui/src/lib.rs \
  | grep -vE 'anatomy|aria|data_attrs|positioning|state'
grep -E '^pub mod ' crates/pre-styled-ui/src/lib.rs \
  | grep -vE 'css|recipe|stylesheet|theme'
```

2026-07-23 時点の実測（#765 マージにより status / empty_state を追加
反映。本節はこれ以前の複数 PR（#754〜#761 等）を経て蓄積した mod 数の
乖離を本イシューで合わせて是正した）:

- headless-ui 34: accordion / avatar / breadcrumb / carousel / checkbox /
  collapsible / combobox / dialog / drawer / field / fieldset / hover_card /
  link / link_overlay / menu / nav_list / number_input / pagination /
  pin_input / popover / progress / radio_group / rating_group /
  segment_group / select / slider / switch / tabs / tags_input / toggle /
  toggle_group / toggle_tip / tooltip / tree_view
- pre-styled-ui 42（styled ラッパー 32 + 静的部品 10）:
  accordion / avatar / breadcrumb / carousel / checkbox / checkbox_card /
  combobox / dialog / drawer / hover_card / link / link_overlay / menu /
  nav_list / number_input / pagination / pin_input / popover / radio_card /
  radio_group / rating_group / segment_group / select / slider / switch /
  tabs / tags_input / toggle / toggle_group / toggle_tip / tooltip /
  tree_view（styled ラッパー、`checkbox_card`/`radio_card` は headless 状態
  機械（`checkbox`/`radio_group`）を再利用するカード型選択 UI として本区分へ
  計上、32 件）+ alert / badge / button / card / spinner / input / textarea /
  native_select / status / empty_state（静的部品、10 件）

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

## 5. 表本体

参照ファイルパス単位で 1 md = 1 行。Part A（ark-ui）/ Part B（chakra-ui）
をディレクトリ節ごとに分割して掲載する。

- **参照ファイル**: `.agents/skills/.../xxx.md` のフルパス（機械突合のキー）
- **ark-ui 名 / chakra-ui 名**: 対応する相手側コンポーネント名。片側のみ・
  相手側に対応がない場合は `—`
- **fandhe headless-ui / fandhe pre-styled-ui**: 対応する mod 名。未実装は `—`
- **区分**: §2 の 5 区分
- **根拠・対応 issue**: 実装対象は issue 番号、保留・意図的非採用は根拠概要
  （詳細は #735 または既存記録 #716/#724・`docs/policy/intentional-non-adoption.md`）

### Part A: ark-ui（`.agents/skills/ark-ui/references/`、90 件）

#### `.agents/skills/ark-ui/references/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/collections/async-list.md` | AsyncListCollection | — | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities、非コンポーネント API） |
| `.agents/skills/ark-ui/references/collections/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/list-collection.md` | ListCollection | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/list-selection.md` | ListSelection | — | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/ark-ui/references/collections/tree-collection.md` | TreeCollection | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/collections/menu.md` | Menu | Menu | menu | menu | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/select.md` | Select | Select | select | select | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/combobox.md` | Combobox | Combobox | — | — | 実装対象 | #749 |
| `.agents/skills/ark-ui/references/components/collections/listbox.md` | Listbox | Listbox | — | — | 実装対象 | #750 |
| `.agents/skills/ark-ui/references/components/collections/pagination.md` | Pagination | Pagination | — | — | 実装対象 | #751（#716 保留の解除） |
| `.agents/skills/ark-ui/references/components/collections/steps.md` | Steps | Steps | — | — | 実装対象 | #752（#716 保留の解除） |
| `.agents/skills/ark-ui/references/components/collections/tree-view.md` | TreeView | TreeView | tree_view | tree_view | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/carousel.md` | Carousel | Carousel | carousel | carousel | 実装済み | headless+styled 実装済み（#754）。autoplay（play/pause/aria-live 切替/delay）は初期実装スコープ外（`crates/headless-ui/src/carousel.rs` module doc 参照） |
| `.agents/skills/ark-ui/references/components/collections/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/date-time/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/date-time/date-input.md` | DateInput | — | — | — | 保留 | #735 で記録（date-time 系） |
| `.agents/skills/ark-ui/references/components/date-time/date-picker.md` | DatePicker | DatePicker | — | — | 保留 | #735 で記録（date-time 系） |
| `.agents/skills/ark-ui/references/components/date-time/timer.md` | Timer | — | — | — | 保留 | #735 で記録（date-time 系） |
| `.agents/skills/ark-ui/references/components/date-time/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/disclosure/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/disclosure/accordion.md` | Accordion | Accordion | accordion | accordion | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/disclosure/collapsible.md` | Collapsible | Collapsible | collapsible | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/disclosure/tabs.md` | Tabs | Tabs | tabs | tabs | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/disclosure/toggle.md` | Toggle | — | — | — | 実装対象 | #746 |
| `.agents/skills/ark-ui/references/components/disclosure/toggle-group.md` | ToggleGroup | — | — | — | 実装対象 | #746 |
| `.agents/skills/ark-ui/references/components/disclosure/scroll-area.md` | ScrollArea | ScrollArea | — | — | 保留 | #735 で記録（装飾系） |
| `.agents/skills/ark-ui/references/components/disclosure/splitter.md` | Splitter | Splitter | — | — | 保留 | #735 で記録（装飾系） |
| `.agents/skills/ark-ui/references/components/disclosure/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/display/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/display/avatar.md` | Avatar | Avatar | avatar | avatar | 実装済み | headless+styled 実装済み（#731 MutationObserver 対応込み） |
| `.agents/skills/ark-ui/references/components/display/progress-linear.md` | Progress (linear) | Progress | progress | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装（circular 対応の #763 とはスコープを分離、follow-up イシュー起票を検討） |
| `.agents/skills/ark-ui/references/components/display/progress-circular.md` | Progress (circular) | ProgressCircle | progress | progress | 実装済み | #763（既存 progress mod を circular 対応へ拡張。headless は #600 で実装済み、pre-styled ラッパーを #763 で追加） |
| `.agents/skills/ark-ui/references/components/display/clipboard.md` | Clipboard | Clipboard | — | — | 実装対象 | #773（wasm 配線込み） |
| `.agents/skills/ark-ui/references/components/display/qr-code.md` | QrCode | QrCode | — | — | 実装対象 | #774 |
| `.agents/skills/ark-ui/references/components/display/marquee.md` | Marquee | Marquee | — | — | 保留 | #735 で記録（装飾系） |
| `.agents/skills/ark-ui/references/components/display/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/form/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/form/checkbox.md` | Checkbox | Checkbox | checkbox | checkbox | 実装済み | headless+styled 実装済み（#730） |
| `.agents/skills/ark-ui/references/components/form/field.md` | Field | Field | field | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/form/fieldset.md` | Fieldset | Fieldset | fieldset | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/form/radio-group.md` | RadioGroup | Radio | radio_group | radio_group | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/form/switch.md` | Switch | Switch | switch | switch | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/form/number-input.md` | NumberInput | NumberInput | — | — | 実装対象 | #738 |
| `.agents/skills/ark-ui/references/components/form/pin-input.md` | PinInput | PinInput | — | — | 実装対象 | #739 |
| `.agents/skills/ark-ui/references/components/form/password-input.md` | PasswordInput | PasswordInput | — | — | 実装対象 | #740 |
| `.agents/skills/ark-ui/references/components/form/slider.md` | Slider | Slider | slider | slider | 実装済み | headless+styled 実装済み（#741） |
| `.agents/skills/ark-ui/references/components/form/rating-group.md` | RatingGroup | Rating | rating_group | rating_group | 実装済み | headless+styled 実装済み（#742） |
| `.agents/skills/ark-ui/references/components/form/segment-group.md` | SegmentGroup | SegmentedControl | segment_group | segment_group | 実装済み | headless+styled 実装済み（#743） |
| `.agents/skills/ark-ui/references/components/form/tags-input.md` | TagsInput | TagsInput | tags_input | tags_input | 実装済み | headless+styled 実装済み（#744） |
| `.agents/skills/ark-ui/references/components/form/editable.md` | Editable | Editable | — | — | 実装対象 | #745 |
| `.agents/skills/ark-ui/references/components/form/angle-slider.md` | AngleSlider | — | — | — | 保留 | #735 で記録（高度入力系） |
| `.agents/skills/ark-ui/references/components/form/color-picker.md` | ColorPicker | ColorPicker | — | — | 保留 | #735 で記録（高度入力系） |
| `.agents/skills/ark-ui/references/components/form/file-upload.md` | FileUpload | FileUpload | — | — | 保留 | #735 で記録（高度入力系） |
| `.agents/skills/ark-ui/references/components/form/image-cropper.md` | ImageCropper | — | — | — | 保留 | #735 で記録（高度入力系） |
| `.agents/skills/ark-ui/references/components/form/signature-pad.md` | SignaturePad | — | — | — | 保留 | #735 で記録（高度入力系） |
| `.agents/skills/ark-ui/references/components/form/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/overlays/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/overlays/dialog.md` | Dialog | Dialog | dialog | dialog | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/popover.md` | Popover | Popover | popover | popover | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/tooltip.md` | Tooltip | Tooltip | tooltip | tooltip | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/drawer.md` | Drawer | Drawer | drawer | drawer | 実装済み | headless+styled 実装済み（#758、dialog の状態機械を再利用） |
| `.agents/skills/ark-ui/references/components/overlays/hover-card.md` | HoverCard | HoverCard | hover_card | hover_card | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/overlays/toast.md` | Toast | Toast | — | — | 実装対象 | #760 |
| `.agents/skills/ark-ui/references/components/overlays/floating-panel.md` | FloatingPanel | FloatingPanel | — | — | 保留 | #735 で記録（装飾系） |
| `.agents/skills/ark-ui/references/components/overlays/tour.md` | Tour | — | — | — | 保留 | #735 で記録（装飾系） |
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
| `.agents/skills/ark-ui/references/utilities/client-only.md` | ClientOnly | ClientOnly | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/download-trigger.md` | DownloadTrigger | DownloadTrigger | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/environment.md` | Environment | EnvironmentProvider | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/focus-trap.md` | FocusTrap | — | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/format-byte.md` | FormatByte | FormatByte | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/format-number.md` | FormatNumber | FormatNumber | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/format-relative-time.md` | FormatRelativeTime | — | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/format-time.md` | FormatTime | — | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/frame.md` | Frame | — | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/json-tree-view.md` | JsonTreeView | — | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/locale.md` | Locale | LocaleProvider | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/presence.md` | Presence | Presence | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/swap.md` | Swap | — | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/ark-ui/references/utilities/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

### Part B: chakra-ui（`.agents/skills/chakra-ui/references/`、269 件）

#### `.agents/skills/chakra-ui/references/blocks/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/blocks/README.md` | — | README | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/blocks/ai.md` | — | Ai | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/app-headers.md` | — | AppHeaders | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/app-integrations.md` | — | AppIntegrations | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/app-navbars.md` | — | AppNavbars | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/authentication.md` | — | Authentication | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/banners.md` | — | Banners | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/blogs.md` | — | Blogs | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/cards.md` | — | Cards | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/careers.md` | — | Careers | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/charts.md` | — | Charts | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/contacts.md` | — | Contacts | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/cta.md` | — | Cta | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/dividers.md` | — | Dividers | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-changelog.md` | — | DocsChangelog | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-code-block.md` | — | DocsCodeBlock | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-example-preview.md` | — | DocsExamplePreview | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-header.md` | — | DocsHeader | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-navbar.md` | — | DocsNavbar | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-pagination.md` | — | DocsPagination | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-parameter-field.md` | — | DocsParameterField | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-sidebar.md` | — | DocsSidebar | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-step.md` | — | DocsStep | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/docs-toc.md` | — | DocsToc | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/faqs.md` | — | Faqs | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/features.md` | — | Features | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/feeds.md` | — | Feeds | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/footers.md` | — | Footers | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/help-center.md` | — | HelpCenter | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/heroes.md` | — | Heroes | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/layouts.md` | — | Layouts | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/logos.md` | — | Logos | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/marketing-headers.md` | — | MarketingHeaders | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/marketing-navbars.md` | — | MarketingNavbars | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/notifications.md` | — | Notifications | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/onboarding.md` | — | Onboarding | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/org-switcher.md` | — | OrgSwitcher | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/pricing.md` | — | Pricing | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/product-categories.md` | — | ProductCategories | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/product-grid.md` | — | ProductGrid | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/product-reviews.md` | — | ProductReviews | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/profiles.md` | — | Profiles | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/property-panels.md` | — | PropertyPanels | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/settings.md` | — | Settings | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/sharing.md` | — | Sharing | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/shopping-carts.md` | — | ShoppingCarts | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/sidebars.md` | — | Sidebars | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/stats.md` | — | Stats | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/store-signup-offers.md` | — | StoreSignupOffers | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/teams.md` | — | Teams | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/testimonials.md` | — | Testimonials | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |
| `.agents/skills/chakra-ui/references/blocks/webhooks.md` | — | Webhooks | — | — | 保留 | #735 で記録（blocks 全般、chakra-ui Pro） |

#### `.agents/skills/chakra-ui/references/charts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/charts/README.md` | — | README | — | — | 対象外 | 対象外（非コンポーネント文書） |
| `.agents/skills/chakra-ui/references/charts/area-chart.md` | — | AreaChart | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/axes.md` | — | Axes | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/bar-chart.md` | — | BarChart | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/bar-list.md` | — | BarList | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/bar-segment.md` | — | BarSegment | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/cartesian-grid.md` | — | CartesianGrid | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/donut-chart.md` | — | DonutChart | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/installation.md` | — | Installation | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/legend.md` | — | Legend | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/line-chart.md` | — | LineChart | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/pie-chart.md` | — | PieChart | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/radar-chart.md` | — | RadarChart | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/scatter-chart.md` | — | ScatterChart | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/sparkline.md` | — | Sparkline | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/tooltip.md` | — | Tooltip | — | — | 保留 | #735 で記録（charts 全般） |
| `.agents/skills/chakra-ui/references/charts/use-chart.md` | — | UseChart | — | — | 保留 | #735 で記録（charts 全般） |

#### `.agents/skills/chakra-ui/references/components/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/README.md` | — | README | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/chakra-ui/references/components/buttons/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/buttons/button.md` | — | Button | — | button | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/buttons/download-trigger.md` | DownloadTrigger | DownloadTrigger | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/buttons/close-button.md` | — | CloseButton | — | — | 保留 | #735 で評価予定（未計画、Button 実装済みのバリエーション） |
| `.agents/skills/chakra-ui/references/components/buttons/icon-button.md` | — | IconButton | — | — | 保留 | #735 で評価予定（未計画、Button 実装済みのバリエーション） |

#### `.agents/skills/chakra-ui/references/components/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/collections/select.md` | Select | Select | select | select | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/collections/combobox.md` | Combobox | Combobox | — | — | 実装対象 | #749 |
| `.agents/skills/chakra-ui/references/components/collections/listbox.md` | Listbox | Listbox | — | — | 実装対象 | #750 |
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
| `.agents/skills/chakra-ui/references/components/data-display/table.md` | — | Table | — | — | 実装対象 | #767 |
| `.agents/skills/chakra-ui/references/components/data-display/data-list.md` | — | DataList | — | — | 実装対象 | #767 |
| `.agents/skills/chakra-ui/references/components/data-display/tag.md` | — | Tag | — | — | 実装対象 | #768 |
| `.agents/skills/chakra-ui/references/components/data-display/stat.md` | — | Stat | — | — | 実装対象 | #769 |
| `.agents/skills/chakra-ui/references/components/data-display/timeline.md` | — | Timeline | — | — | 実装対象 | #769 |
| `.agents/skills/chakra-ui/references/components/data-display/image.md` | — | Image | — | image | 実装済み | #770。状態機械なし静的部品、pre-styled 層のみに実装（headless-ui は変更なし）。fit（object-fit）/aspect-ratio の 2 軸 variant、alt 必須引数 |
| `.agents/skills/chakra-ui/references/components/data-display/icon.md` | — | Icon | — | icon | 実装済み | #770。状態機械なし静的部品、pre-styled 層のみに実装（headless-ui は変更なし）。size variant のみ、SVG 本体は呼び出し側がノード木 API で構築 |
| `.agents/skills/chakra-ui/references/components/data-display/clipboard.md` | Clipboard | Clipboard | — | — | 実装対象 | #773（wasm 配線込み） |
| `.agents/skills/chakra-ui/references/components/data-display/qr-code.md` | QrCode | QrCode | — | — | 実装対象 | #774 |
| `.agents/skills/chakra-ui/references/components/data-display/marquee.md` | Marquee | Marquee | — | — | 保留 | #735 で記録（装飾系） |

#### `.agents/skills/chakra-ui/references/components/date-time/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/date-time/date-picker.md` | DatePicker | DatePicker | — | — | 保留 | #735 で記録（date-time 系） |
| `.agents/skills/chakra-ui/references/components/date-time/calendar.md` | — | Calendar | — | — | 保留 | #735 で記録（date-time 系） |

#### `.agents/skills/chakra-ui/references/components/disclosure/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/disclosure/accordion.md` | Accordion | Accordion | accordion | accordion | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/disclosure/collapsible.md` | Collapsible | Collapsible | collapsible | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/chakra-ui/references/components/disclosure/tabs.md` | Tabs | Tabs | tabs | tabs | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/disclosure/pagination.md` | Pagination | Pagination | — | — | 実装対象 | #751（#716 保留の解除） |
| `.agents/skills/chakra-ui/references/components/disclosure/steps.md` | Steps | Steps | — | — | 実装対象 | #752（#716 保留の解除） |
| `.agents/skills/chakra-ui/references/components/disclosure/carousel.md` | Carousel | Carousel | carousel | carousel | 実装済み | headless+styled 実装済み（#754）。autoplay（play/pause/aria-live 切替/delay）は初期実装スコープ外（`crates/headless-ui/src/carousel.rs` module doc 参照） |
| `.agents/skills/chakra-ui/references/components/disclosure/breadcrumb.md` | — | Breadcrumb | breadcrumb | breadcrumb | 実装済み | #755（#716 追加候補の消化）。headless+styled 実装済み |

#### `.agents/skills/chakra-ui/references/components/feedback/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/feedback/progress.md` | Progress (linear) | Progress | progress | progress | 実装済み | headless+styled（root）実装済み。linear（Track/Range）用 styled ラッパーは #763 とスコープを分離した対応表側の未実装事項（follow-up イシュー起票を検討） |
| `.agents/skills/chakra-ui/references/components/feedback/alert.md` | — | Alert | — | alert | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/feedback/spinner.md` | — | Spinner | — | spinner | 実装済み | pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/feedback/toast.md` | Toast | Toast | — | — | 実装対象 | #760 |
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
| `.agents/skills/chakra-ui/references/components/forms/number-input.md` | NumberInput | NumberInput | — | — | 実装対象 | #738 |
| `.agents/skills/chakra-ui/references/components/forms/pin-input.md` | PinInput | PinInput | — | — | 実装対象 | #739 |
| `.agents/skills/chakra-ui/references/components/forms/password-input.md` | PasswordInput | PasswordInput | — | — | 実装対象 | #740 |
| `.agents/skills/chakra-ui/references/components/forms/slider.md` | Slider | Slider | slider | slider | 実装済み | headless+styled 実装済み（#741） |
| `.agents/skills/chakra-ui/references/components/forms/rating.md` | RatingGroup | Rating | rating_group | rating_group | 実装済み | headless+styled 実装済み（#742） |
| `.agents/skills/chakra-ui/references/components/forms/segmented-control.md` | SegmentGroup | SegmentedControl | segment_group | segment_group | 実装済み | headless+styled 実装済み（#743） |
| `.agents/skills/chakra-ui/references/components/forms/tags-input.md` | TagsInput | TagsInput | tags_input | tags_input | 実装済み | headless+styled 実装済み（#744） |
| `.agents/skills/chakra-ui/references/components/forms/editable.md` | Editable | Editable | — | — | 実装対象 | #745 |
| `.agents/skills/chakra-ui/references/components/forms/checkbox-card.md` | — | CheckboxCard | — | checkbox_card | 実装済み | pre-styled styled バリエーション実装済み（#747。headless-ui は変更なし、状態機械は headless Checkbox を再利用） |
| `.agents/skills/chakra-ui/references/components/forms/radio-card.md` | — | RadioCard | — | radio_card | 実装済み | pre-styled styled バリエーション実装済み（#747。headless-ui は変更なし、状態機械は headless RadioGroup を再利用） |
| `.agents/skills/chakra-ui/references/components/forms/color-picker.md` | ColorPicker | ColorPicker | — | — | 保留 | #735 で記録（高度入力系） |
| `.agents/skills/chakra-ui/references/components/forms/color-swatch.md` | — | ColorSwatch | — | — | 保留 | #735 で記録（高度入力系、ColorPicker 併設部品） |
| `.agents/skills/chakra-ui/references/components/forms/file-upload.md` | FileUpload | FileUpload | — | — | 保留 | #735 で記録（高度入力系） |

#### `.agents/skills/chakra-ui/references/components/i18n/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/i18n/format-byte.md` | FormatByte | FormatByte | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/i18n/format-number.md` | FormatNumber | FormatNumber | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/i18n/locale-provider.md` | Locale | LocaleProvider | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |

#### `.agents/skills/chakra-ui/references/components/layout/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/layout/separator.md` | — | Separator | — | separator | 実装済み | #772。pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/layout/scroll-area.md` | ScrollArea | ScrollArea | — | — | 保留 | #735 で記録（装飾系） |
| `.agents/skills/chakra-ui/references/components/layout/splitter.md` | Splitter | Splitter | — | — | 保留 | #735 で記録（装飾系） |
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
| `.agents/skills/chakra-ui/references/components/overlays/toggle-tip.md` | — | ToggleTip | — | — | 実装対象 | #761 |
| `.agents/skills/chakra-ui/references/components/overlays/action-bar.md` | — | ActionBar | — | — | 実装対象 | #762 |
| `.agents/skills/chakra-ui/references/components/overlays/overlay-manager.md` | — | OverlayManager | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/overlays/floating-panel.md` | FloatingPanel | FloatingPanel | — | — | 保留 | #735 で記録（装飾系） |

#### `.agents/skills/chakra-ui/references/components/typography/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/typography/link.md` | — | Link | — | — | 実装対象 | #756（#716 最優先候補の消化） |
| `.agents/skills/chakra-ui/references/components/typography/link-overlay.md` | — | LinkOverlay | — | — | 実装対象 | #756（#716 最優先候補の消化） |
| `.agents/skills/chakra-ui/references/components/typography/kbd.md` | — | Kbd | — | — | 実装対象 | #768 |
| `.agents/skills/chakra-ui/references/components/typography/code.md` | — | Code | — | — | 実装対象 | #768 |
| `.agents/skills/chakra-ui/references/components/typography/heading.md` | — | Heading | — | — | 実装対象 | #771 |
| `.agents/skills/chakra-ui/references/components/typography/text.md` | — | Text | — | — | 実装対象 | #771 |
| `.agents/skills/chakra-ui/references/components/typography/em.md` | — | Em | — | — | 実装対象 | #771 |
| `.agents/skills/chakra-ui/references/components/typography/mark.md` | — | Mark | — | — | 実装対象 | #771 |
| `.agents/skills/chakra-ui/references/components/typography/blockquote.md` | — | Blockquote | — | — | 実装対象 | #771 |
| `.agents/skills/chakra-ui/references/components/typography/list.md` | — | List | — | — | 実装対象 | #771 |
| `.agents/skills/chakra-ui/references/components/typography/highlight.md` | Highlight | Highlight | — | `highlight` | 実装済み | #775。pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/typography/rich-text-editor.md` | — | RichTextEditor | — | — | 保留 | #735 で記録（高度入力系） |
| `.agents/skills/chakra-ui/references/components/typography/code-block.md` | — | CodeBlock | — | — | 対象外 | 対象外（docs-site 既存機構と役割分担） |
| `.agents/skills/chakra-ui/references/components/typography/prose.md` | — | Prose | — | — | 対象外 | 対象外（docs-site 既存機構と役割分担） |

#### `.agents/skills/chakra-ui/references/components/utilities/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/utilities/visually-hidden.md` | — | VisuallyHidden | — | — | 実装対象 | #776 |
| `.agents/skills/chakra-ui/references/components/utilities/skip-nav.md` | — | SkipNav | — | — | 実装対象 | #776 |
| `.agents/skills/chakra-ui/references/components/utilities/client-only.md` | ClientOnly | ClientOnly | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/utilities/environment-provider.md` | Environment | EnvironmentProvider | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/utilities/presence.md` | Presence | Presence | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/utilities/checkmark.md` | — | Checkmark | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/utilities/radiomark.md` | — | Radiomark | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/utilities/for.md` | — | For | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/utilities/portal.md` | — | Portal | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/utilities/show.md` | — | Show | — | — | 保留 | #735 で記録（JS ランタイム固有 utilities） |
| `.agents/skills/chakra-ui/references/components/utilities/theme.md` | — | Theme | — | — | 保留 | #735 で記録（既存 theme mod との役割重複を含め要精査） |

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
| `.agents/skills/chakra-ui/references/styling/README.md` | — | Readme | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

#### `.agents/skills/chakra-ui/references/styling/compositions/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/compositions/animation-styles.md` | — | AnimationStyles | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/focus-ring.md` | — | FocusRing | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/layer-styles.md` | — | LayerStyles | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/compositions/text-styles.md` | — | TextStyles | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

#### `.agents/skills/chakra-ui/references/styling/concepts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/concepts/cascade-layers.md` | — | CascadeLayers | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/chakra-factory.md` | — | ChakraFactory | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/color-opacity-modifier.md` | — | ColorOpacityModifier | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/conditional-styles.md` | — | ConditionalStyles | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/css-variables.md` | — | CssVariables | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/dark-mode.md` | — | DarkMode | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/overview.md` | — | Overview | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/responsive-design.md` | — | ResponsiveDesign | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/concepts/virtual-color.md` | — | VirtualColor | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

#### `.agents/skills/chakra-ui/references/styling/style-props/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/styling/style-props/background.md` | — | Background | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/border.md` | — | Border | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/display.md` | — | Display | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/effects.md` | — | Effects | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/filters.md` | — | Filters | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/flex-and-grid.md` | — | FlexAndGrid | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/interactivity.md` | — | Interactivity | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/layout.md` | — | Layout | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/list.md` | — | List | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/sizing.md` | — | Sizing | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/spacing.md` | — | Spacing | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/svg.md` | — | Svg | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/tables.md` | — | Tables | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/transforms.md` | — | Transforms | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/transitions.md` | — | Transitions | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/styling/style-props/typography.md` | — | Typography | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

#### `.agents/skills/chakra-ui/references/theming/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/README.md` | — | Readme | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

#### `.agents/skills/chakra-ui/references/theming/compositions/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/compositions/layer-styles.md` | — | LayerStyles | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/compositions/text-styles.md` | — | TextStyles | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

#### `.agents/skills/chakra-ui/references/theming/concepts/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/concepts/overview.md` | — | Overview | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/recipes.md` | — | Recipes | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/semantic-tokens.md` | — | SemanticTokens | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/slot-recipes.md` | — | SlotRecipes | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/concepts/tokens.md` | — | Tokens | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

#### `.agents/skills/chakra-ui/references/theming/customization/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/customization/animations.md` | — | Animations | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/breakpoints.md` | — | Breakpoints | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/colors.md` | — | Colors | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/conditions.md` | — | Conditions | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/css-variables.md` | — | CssVariables | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/global-css.md` | — | GlobalCss | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/overview.md` | — | Overview | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/recipes.md` | — | Recipes | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/sizes.md` | — | Sizes | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/spacing.md` | — | Spacing | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/customization/utilities.md` | — | Utilities | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

#### `.agents/skills/chakra-ui/references/theming/design-tokens/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/theming/design-tokens/animations.md` | — | Animations | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/aspect-ratios.md` | — | AspectRatios | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/breakpoints.md` | — | Breakpoints | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/colors.md` | — | Colors | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/cursors.md` | — | Cursors | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/radii.md` | — | Radii | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/shadows.md` | — | Shadows | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/sizes.md` | — | Sizes | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/spacing.md` | — | Spacing | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/typography.md` | — | Typography | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |
| `.agents/skills/chakra-ui/references/theming/design-tokens/z-index.md` | — | ZIndex | — | — | 保留 | #735 で記録（styling/theming 概念、既存 theme/recipe/StyleSheet で対応） |

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

- **機械確認の CI 常時実行化**: 本書 §4 の diff コマンドを CI ジョブ /
  テストとして自動実行する仕組みは本イシューのスコープ外。references
  更新時のドリフト検知を自動化したい場合は別 issue の起票を提案する
  （`.claude/rules/out-of-scope-tracking.md` に従い、ユーザー承認の上で
  #726 配下へ）
- **保留・意図的非採用の評価軸・再評価トリガーの詳細記録**: 後続 #735 の
  担当。本書からは前方参照のみ行う
