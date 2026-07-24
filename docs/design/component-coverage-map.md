# ark-ui / chakra-ui 全コンポーネント対応表

**本文書のステータス**: 確定（イシュー #734、親 #733/#726）。正はコミット
`ab53c6b` 時点の `.agents/skills/ark-ui|chakra-ui/references/` 一覧
（ark-ui 90 件 / chakra-ui 269 件 / 計 359 件）。以後この文書を
Phase 3〜6（#736/#748/#757/#766 配下、#737〜#747・#749〜#756・#758〜#765・
#767〜#776 の 37 issue 相当）実装の正とする。保留・意図的非採用の評価軸・
再評価トリガーはイシュー #735 で `docs/policy/intentional-non-adoption.md`
§3.22〜§3.24（新規非採用確定）・§7（保留項目の記録）に確定記録済み。

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

`crates/headless-ui/src/lib.rs` の基盤 mod（anatomy / aria / data_attrs /
positioning / state）を除く 35 mod、`crates/pre-styled-ui/src/lib.rs` の
基盤 mod（css / recipe / stylesheet / theme）を除く 43 mod（styled ラッパー
33 + 静的部品 10）が、本書の「実装済み」区分と一致することを次のコマンドで
確認できる。

```bash
grep -E '^pub mod ' crates/headless-ui/src/lib.rs \
  | grep -vE 'anatomy|aria|data_attrs|positioning|state'
grep -E '^pub mod ' crates/pre-styled-ui/src/lib.rs \
  | grep -vE 'css|recipe|stylesheet|theme'
```

2026-07-24 時点の実測（#828 マージにより download_trigger を追加反映。
これ以前は 2026-07-23 時点の実測（#750 マージにより listbox、#745 マージ
により editable を追加反映。#765 マージによる status / empty_state を含め
再実測。本節はこれ以前の複数 PR（#754〜#765 等）を経て蓄積した mod 数の
乖離を合わせて是正した）だった）:

- headless-ui 38（#836 で timer を追加反映。ただし本節は clipboard /
  qr_code / scroll_area / password_input / action_bar / steps / toast /
  skip_nav / visually_hidden 等、#836 以前に既にマージ済みの複数 mod を
  未反映のまま蓄積した既知のドリフトを抱えており、本 PR ではそれらの再実測
  までは行わない。全件再実測は別途の docs 整備イシューとして切り出しを提案
  する、`.claude/rules/out-of-scope-tracking.md` 対応）:
  accordion / avatar / breadcrumb / carousel / checkbox /
  collapsible / combobox / dialog / download_trigger / drawer / editable /
  field / fieldset / hover_card / link / link_overlay / listbox / menu /
  nav_list / number_input / pagination / pin_input / popover / progress /
  radio_group / rating_group / segment_group / select / slider / switch /
  tabs / tags_input / timer / toggle / toggle_group / toggle_tip / tooltip /
  tree_view
- pre-styled-ui 46（styled ラッパー 36 + 静的部品 10。#836 で timer を追加
  反映。上記と同じ既知のドリフトを抱える）:
  accordion / avatar / breadcrumb / carousel / checkbox / checkbox_card /
  combobox / dialog / download_trigger / drawer / editable / hover_card /
  link / link_overlay / listbox / menu / nav_list / number_input /
  pagination / pin_input / popover / radio_card / radio_group /
  rating_group / segment_group / select / slider / switch / tabs /
  tags_input / timer / toggle / toggle_group / toggle_tip / tooltip / tree_view
  （styled ラッパー、`checkbox_card`/`radio_card` は headless 状態機械
  （`checkbox`/`radio_group`）を再利用するカード型選択 UI として、
  `download_trigger` は headless 自由関数（`download_trigger::root`）を
  `crate::button::recipe_with_scope` で流用する styled 版として、本区分へ
  計上、36 件）+ alert / badge / button / card / spinner / input /
  textarea / native_select / status / empty_state（静的部品、10 件）

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
  （詳細はイシュー #735 で確定した `docs/policy/intentional-non-adoption.md`
  §3.22〜§3.24・§7、または既存記録 #716/#724）

### Part A: ark-ui（`.agents/skills/ark-ui/references/`、90 件）

#### `.agents/skills/ark-ui/references/collections/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/collections/async-list.md` | AsyncListCollection | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
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
| `.agents/skills/ark-ui/references/components/collections/listbox.md` | Listbox | Listbox | listbox | listbox | 実装済み | headless+styled 実装済み（#750） |
| `.agents/skills/ark-ui/references/components/collections/pagination.md` | Pagination | Pagination | — | — | 実装対象 | #751（#716 保留の解除） |
| `.agents/skills/ark-ui/references/components/collections/steps.md` | Steps | Steps | steps | steps | 実装済み | headless+styled 実装済み（#752、#716 保留の解除） |
| `.agents/skills/ark-ui/references/components/collections/tree-view.md` | TreeView | TreeView | tree_view | tree_view | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/collections/carousel.md` | Carousel | Carousel | carousel | carousel | 実装済み | headless+styled 実装済み（#754）。autoplay（play/pause/aria-live 切替/delay）は初期実装スコープ外（`crates/headless-ui/src/carousel.rs` module doc 参照） |
| `.agents/skills/ark-ui/references/components/collections/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/date-time/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/date-time/date-input.md` | DateInput | — | — | — | 保留 | （date-time 系）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/ark-ui/references/components/date-time/date-picker.md` | DatePicker | DatePicker | — | — | 保留 | （date-time 系）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/ark-ui/references/components/date-time/timer.md` | Timer | Timer | timer | timer | 実装済み | headless+styled+wasm 配線実装済み（#836）。tick を外部から明示的に注入する決定的状態機械（時計 API 非依存）として実装し、`docs/policy/intentional-non-adoption.md` §7 の保留を解除した |
| `.agents/skills/ark-ui/references/components/date-time/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/disclosure/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/disclosure/accordion.md` | Accordion | Accordion | accordion | accordion | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/disclosure/collapsible.md` | Collapsible | Collapsible | collapsible | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/ark-ui/references/components/disclosure/tabs.md` | Tabs | Tabs | tabs | tabs | 実装済み | headless+styled 実装済み |
| `.agents/skills/ark-ui/references/components/disclosure/toggle.md` | Toggle | — | — | — | 実装対象 | #746 |
| `.agents/skills/ark-ui/references/components/disclosure/toggle-group.md` | ToggleGroup | — | — | — | 実装対象 | #746 |
| `.agents/skills/ark-ui/references/components/disclosure/scroll-area.md` | ScrollArea | ScrollArea | scroll_area | scroll_area | 実装済み | headless+styled 実装済み（#825、保留解除。JS によるスクロール位置追従・thumb drag は本イシューのスコープ外） |
| `.agents/skills/ark-ui/references/components/disclosure/splitter.md` | Splitter | Splitter | splitter | splitter | 実装済み | headless+styled 実装済み（#826、#735 保留の解除） |
| `.agents/skills/ark-ui/references/components/disclosure/README.md` | README | — | — | — | 対象外 | 対象外（非コンポーネント文書） |

#### `.agents/skills/ark-ui/references/components/display/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/ark-ui/references/components/display/avatar.md` | Avatar | Avatar | avatar | avatar | 実装済み | headless+styled 実装済み（#731 MutationObserver 対応込み） |
| `.agents/skills/ark-ui/references/components/display/progress-linear.md` | Progress (linear) | Progress | progress | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装（circular 対応の #763 とはスコープを分離、follow-up イシュー起票を検討） |
| `.agents/skills/ark-ui/references/components/display/progress-circular.md` | Progress (circular) | ProgressCircle | progress | progress | 実装済み | #763（既存 progress mod を circular 対応へ拡張。headless は #600 で実装済み、pre-styled ラッパーを #763 で追加） |
| `.agents/skills/ark-ui/references/components/display/clipboard.md` | Clipboard | Clipboard | — | — | 実装対象 | #773（wasm 配線込み） |
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
| `.agents/skills/ark-ui/references/components/form/number-input.md` | NumberInput | NumberInput | — | — | 実装対象 | #738 |
| `.agents/skills/ark-ui/references/components/form/pin-input.md` | PinInput | PinInput | — | — | 実装対象 | #739 |
| `.agents/skills/ark-ui/references/components/form/password-input.md` | PasswordInput | PasswordInput | password_input | password_input | 実装済み | headless+styled 実装済み（#740） |
| `.agents/skills/ark-ui/references/components/form/slider.md` | Slider | Slider | slider | slider | 実装済み | headless+styled 実装済み（#741） |
| `.agents/skills/ark-ui/references/components/form/rating-group.md` | RatingGroup | Rating | rating_group | rating_group | 実装済み | headless+styled 実装済み（#742） |
| `.agents/skills/ark-ui/references/components/form/segment-group.md` | SegmentGroup | SegmentedControl | segment_group | segment_group | 実装済み | headless+styled 実装済み（#743） |
| `.agents/skills/ark-ui/references/components/form/tags-input.md` | TagsInput | TagsInput | tags_input | tags_input | 実装済み | headless+styled 実装済み（#744） |
| `.agents/skills/ark-ui/references/components/form/editable.md` | Editable | Editable | editable | editable | 実装済み | headless+styled 実装済み（#745） |
| `.agents/skills/ark-ui/references/components/form/angle-slider.md` | AngleSlider | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.22（#735）で非採用確定（高度入力系、canvas/ポインタ座標/contenteditable 依存） |
| `.agents/skills/ark-ui/references/components/form/color-picker.md` | ColorPicker | ColorPicker | — | — | 保留 | （高度入力系、フォーム部品）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/ark-ui/references/components/form/file-upload.md` | FileUpload | FileUpload | file_upload | file_upload | 実装済み | headless+styled+wasm 実装済み（#840、`docs/policy/intentional-non-adoption.md` §7 保留解除。ItemPreview/ItemPreviewImage はスコープ外） |
| `.agents/skills/ark-ui/references/components/form/image-cropper.md` | ImageCropper | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.22（#735）で非採用確定（高度入力系、canvas/ポインタ座標/contenteditable 依存） |
| `.agents/skills/ark-ui/references/components/form/signature-pad.md` | SignaturePad | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.22（#735）で非採用確定（高度入力系、canvas/ポインタ座標/contenteditable 依存） |
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
| `.agents/skills/ark-ui/references/components/overlays/tour.md` | Tour | — | — | — | 保留 | （装飾系）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
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
| `.agents/skills/ark-ui/references/utilities/client-only.md` | ClientOnly | ClientOnly | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/download-trigger.md` | DownloadTrigger | DownloadTrigger | `download_trigger` | `download_trigger` | 実装済み | #828。保留（#735 §7「JS ランタイム固有 utilities のうち静的実装可能なもの」）を利用要望 issue（#828）の起票により解除。`a[download]` 属性による静的部品として実装（`Blob`/`data`/`mimeType` は JS 前提のため対応しない） |
| `.agents/skills/ark-ui/references/utilities/environment.md` | Environment | EnvironmentProvider | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/focus-trap.md` | FocusTrap | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/format-byte.md` | FormatByte | FormatByte | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/format-number.md` | FormatNumber | FormatNumber | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/format-relative-time.md` | FormatRelativeTime | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/format-time.md` | FormatTime | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/frame.md` | Frame | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/json-tree-view.md` | JsonTreeView | — | json_tree_view | json_tree_view | 実装済み | **保留解除**（イシュー #829、`tree_view`（#753）の派生として実装。headless `crates/headless-ui/src/json_tree_view.rs` + styled `crates/pre-styled-ui/src/json_tree_view.rs`。`docs/policy/intentional-non-adoption.md` §7 の解除記録参照） |
| `.agents/skills/ark-ui/references/utilities/locale.md` | Locale | LocaleProvider | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/presence.md` | Presence | Presence | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/ark-ui/references/utilities/swap.md` | Swap | — | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
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
| `.agents/skills/chakra-ui/references/charts/area-chart.md` | — | AreaChart | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/axes.md` | — | Axes | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/bar-chart.md` | — | BarChart | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/bar-list.md` | — | BarList | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/bar-segment.md` | — | BarSegment | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/cartesian-grid.md` | — | CartesianGrid | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/donut-chart.md` | — | DonutChart | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/installation.md` | — | Installation | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/legend.md` | — | Legend | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/line-chart.md` | — | LineChart | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/pie-chart.md` | — | PieChart | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/radar-chart.md` | — | RadarChart | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/scatter-chart.md` | — | ScatterChart | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/sparkline.md` | — | Sparkline | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/tooltip.md` | — | Tooltip | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/charts/use-chart.md` | — | UseChart | — | — | 保留 | （charts 全般）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |

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
| `.agents/skills/chakra-ui/references/components/collections/combobox.md` | Combobox | Combobox | — | — | 実装対象 | #749 |
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
| `.agents/skills/chakra-ui/references/components/data-display/data-list.md` | — | DataList | — | data-list | 実装済み | pre-styled 静的部品 実装済み（#767。`variant`（subtle/bold）/`size` variant はスコープ外） |
| `.agents/skills/chakra-ui/references/components/data-display/tag.md` | — | Tag | — | tag | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/data-display/stat.md` | — | Stat | — | `stat` | 実装済み | pre-styled 静的部品 実装済み（#769。headless-ui は変更なし） |
| `.agents/skills/chakra-ui/references/components/data-display/timeline.md` | — | Timeline | — | `timeline` | 実装済み | pre-styled 静的部品 実装済み（#769。headless-ui は変更なし） |
| `.agents/skills/chakra-ui/references/components/data-display/image.md` | — | Image | — | image | 実装済み | #770。状態機械なし静的部品、pre-styled 層のみに実装（headless-ui は変更なし）。fit（object-fit）/aspect-ratio の 2 軸 variant、alt 必須引数 |
| `.agents/skills/chakra-ui/references/components/data-display/icon.md` | — | Icon | — | icon | 実装済み | #770。状態機械なし静的部品、pre-styled 層のみに実装（headless-ui は変更なし）。size variant のみ、SVG 本体は呼び出し側がノード木 API で構築 |
| `.agents/skills/chakra-ui/references/components/data-display/clipboard.md` | Clipboard | Clipboard | — | — | 実装対象 | #773（wasm 配線込み） |
| `.agents/skills/chakra-ui/references/components/data-display/qr-code.md` | QrCode | QrCode | qr_code | qr_code | 実装済み | headless+styled 実装済み（#774） |
| `.agents/skills/chakra-ui/references/components/data-display/marquee.md` | Marquee | Marquee | — | marquee | 実装済み（再導入） | #831 で `docs/policy/intentional-non-adoption.md` §3.24 の再評価トリガー 1（CSS のみ・`prefers-reduced-motion` 対応の決定的設計案）を充足し再導入（CSS のみ・JS ゼロ）。headless-ui は変更なし、pre-styled-ui 層のみで新規 anatomy を定義 |

#### `.agents/skills/chakra-ui/references/components/date-time/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/date-time/date-picker.md` | DatePicker | DatePicker | — | — | 保留 | （date-time 系）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/components/date-time/calendar.md` | — | Calendar | — | — | 保留 | （date-time 系）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |

#### `.agents/skills/chakra-ui/references/components/disclosure/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/disclosure/accordion.md` | Accordion | Accordion | accordion | accordion | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/disclosure/collapsible.md` | Collapsible | Collapsible | collapsible | — | 実装済み | headless 実装済み。pre-styled ラッパー未実装 |
| `.agents/skills/chakra-ui/references/components/disclosure/tabs.md` | Tabs | Tabs | tabs | tabs | 実装済み | headless+styled 実装済み |
| `.agents/skills/chakra-ui/references/components/disclosure/pagination.md` | Pagination | Pagination | — | — | 実装対象 | #751（#716 保留の解除） |
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
| `.agents/skills/chakra-ui/references/components/forms/number-input.md` | NumberInput | NumberInput | — | — | 実装対象 | #738 |
| `.agents/skills/chakra-ui/references/components/forms/pin-input.md` | PinInput | PinInput | — | — | 実装対象 | #739 |
| `.agents/skills/chakra-ui/references/components/forms/password-input.md` | PasswordInput | PasswordInput | password_input | password_input | 実装済み | headless+styled 実装済み（#740） |
| `.agents/skills/chakra-ui/references/components/forms/slider.md` | Slider | Slider | slider | slider | 実装済み | headless+styled 実装済み（#741） |
| `.agents/skills/chakra-ui/references/components/forms/rating.md` | RatingGroup | Rating | rating_group | rating_group | 実装済み | headless+styled 実装済み（#742） |
| `.agents/skills/chakra-ui/references/components/forms/segmented-control.md` | SegmentGroup | SegmentedControl | segment_group | segment_group | 実装済み | headless+styled 実装済み（#743） |
| `.agents/skills/chakra-ui/references/components/forms/tags-input.md` | TagsInput | TagsInput | tags_input | tags_input | 実装済み | headless+styled 実装済み（#744） |
| `.agents/skills/chakra-ui/references/components/forms/editable.md` | Editable | Editable | editable | editable | 実装済み | headless+styled 実装済み（#745） |
| `.agents/skills/chakra-ui/references/components/forms/checkbox-card.md` | — | CheckboxCard | — | checkbox_card | 実装済み | pre-styled styled バリエーション実装済み（#747。headless-ui は変更なし、状態機械は headless Checkbox を再利用） |
| `.agents/skills/chakra-ui/references/components/forms/radio-card.md` | — | RadioCard | — | radio_card | 実装済み | pre-styled styled バリエーション実装済み（#747。headless-ui は変更なし、状態機械は headless RadioGroup を再利用） |
| `.agents/skills/chakra-ui/references/components/forms/color-picker.md` | ColorPicker | ColorPicker | — | — | 保留 | （高度入力系、フォーム部品）。根拠・再評価トリガーは `docs/policy/intentional-non-adoption.md` §7（#735） |
| `.agents/skills/chakra-ui/references/components/forms/color-swatch.md` | — | ColorSwatch | — | color_swatch | 実装済み | pre-styled 静的部品として実装済み（#838。headless-ui には対応する anatomy を新設しない。色変換コアは `fandhe-frontend-headless-ui::color`、親 #837） |
| `.agents/skills/chakra-ui/references/components/forms/file-upload.md` | FileUpload | FileUpload | file_upload | file_upload | 実装済み | headless+styled+wasm 実装済み（#840、`docs/policy/intentional-non-adoption.md` §7 保留解除。ItemPreview/ItemPreviewImage はスコープ外） |

#### `.agents/skills/chakra-ui/references/components/i18n/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/i18n/format-byte.md` | FormatByte | FormatByte | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/i18n/format-number.md` | FormatNumber | FormatNumber | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/i18n/locale-provider.md` | Locale | LocaleProvider | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |

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
| `.agents/skills/chakra-ui/references/components/overlays/toggle-tip.md` | — | ToggleTip | — | — | 実装対象 | #761 |
| `.agents/skills/chakra-ui/references/components/overlays/action-bar.md` | — | ActionBar | action_bar | action_bar | 実装済み | headless+styled 実装済み（#762） |
| `.agents/skills/chakra-ui/references/components/overlays/overlay-manager.md` | — | OverlayManager | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/overlays/floating-panel.md` | FloatingPanel | FloatingPanel | floating_panel | floating_panel | 実装済み | headless+styled 実装済み（イシュー #827、`docs/policy/intentional-non-adoption.md` §7 の保留区分から解除） |

#### `.agents/skills/chakra-ui/references/components/typography/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/typography/link.md` | — | Link | — | — | 実装対象 | #756（#716 最優先候補の消化） |
| `.agents/skills/chakra-ui/references/components/typography/link-overlay.md` | — | LinkOverlay | — | — | 実装対象 | #756（#716 最優先候補の消化） |
| `.agents/skills/chakra-ui/references/components/typography/kbd.md` | — | Kbd | — | kbd | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/typography/code.md` | — | Code | — | code | 実装済み | pre-styled 静的部品 実装済み（#768） |
| `.agents/skills/chakra-ui/references/components/typography/heading.md` | — | Heading | — | heading | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/text.md` | — | Text | — | text | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/em.md` | — | Em | — | em | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/mark.md` | — | Mark | — | mark | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/blockquote.md` | — | Blockquote | — | blockquote | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/list.md` | — | List | — | list | 実装済み | #771 |
| `.agents/skills/chakra-ui/references/components/typography/highlight.md` | Highlight | Highlight | — | highlight | 実装済み | #775。pre-styled 静的部品 実装済み |
| `.agents/skills/chakra-ui/references/components/typography/rich-text-editor.md` | — | RichTextEditor | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.22（#735）で非採用確定（高度入力系、canvas/ポインタ座標/contenteditable 依存） |
| `.agents/skills/chakra-ui/references/components/typography/code-block.md` | — | CodeBlock | — | — | 対象外 | 対象外（docs-site 既存機構と役割分担） |
| `.agents/skills/chakra-ui/references/components/typography/prose.md` | — | Prose | — | — | 対象外 | 対象外（docs-site 既存機構と役割分担。役割分担の詳細は #771 の `crates/pre-styled-ui/src/text.rs` rustdoc・`docs/api/pre-styled-ui-api.md` 参照） |

#### `.agents/skills/chakra-ui/references/components/utilities/`

| 参照ファイル | ark-ui 名 | chakra-ui 名 | fandhe headless-ui | fandhe pre-styled-ui | 区分 | 根拠・対応 issue |
|---|---|---|---|---|---|---|
| `.agents/skills/chakra-ui/references/components/utilities/visually-hidden.md` | — | VisuallyHidden | `visually_hidden` | `visually_hidden` | 実装済み | #776 |
| `.agents/skills/chakra-ui/references/components/utilities/skip-nav.md` | — | SkipNav | `skip_nav` | `skip_nav` | 実装済み | #776 |
| `.agents/skills/chakra-ui/references/components/utilities/client-only.md` | ClientOnly | ClientOnly | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/utilities/environment-provider.md` | Environment | EnvironmentProvider | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/utilities/presence.md` | Presence | Presence | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/utilities/checkmark.md` | — | Checkmark | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/utilities/radiomark.md` | — | Radiomark | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/utilities/for.md` | — | For | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/utilities/portal.md` | — | Portal | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/utilities/show.md` | — | Show | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.23（#735）で非採用確定（JS ランタイム固有 utilities、該当概念なし） |
| `.agents/skills/chakra-ui/references/components/utilities/theme.md` | — | Theme | — | — | 意図的非採用 | `docs/policy/intentional-non-adoption.md` §3.24（#735）で非採用確定（既存 theme mod と役割重複） |

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
- **保留・意図的非採用の評価軸・再評価トリガーの詳細記録**: イシュー #735 で
  `docs/policy/intentional-non-adoption.md` §3.22〜§3.24（新規非採用確定）・
  §7（保留項目の記録）に確定記録済み（完了）
