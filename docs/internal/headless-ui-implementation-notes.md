# fandhe-frontend-headless-ui 実装記録（内部）

**本文書の位置づけ**: docs サイト非掲載（`site/nav.toml` 未登録）の内部設計記録。
`docs/api/headless-ui-api.md` から移設した実装経緯・進行管理記述・ロードマップ・
意図的非対応・スコープ外事項を保存する。分離基準は
`../design/docs-site-api-reference-split.md` を正とする。
**注意**: 本リポジトリは public であり「サイト非掲載」は「非公開」を意味しない。

## 0. 旧 → 新 マッピング表

| 旧（`docs/api/headless-ui-api.md`） | 新（本書） |
|---|---|
| §1 spec 未反映の注記 | §1 |
| §2 親トラッキング・Phase 担当・公開来歴 | §2 |
| §4a.6 意図的非対応 | §4a.6 |
| §4b ロードマップ（§4b.1〜§4b.5） | §4b（節番号温存） |
| §4d.3 スコープ外（#835 時点） | §4d.3 |
| §4e.2 スコープ外 | §4e.2 |

**S4（受容記録）**: 以下 8 ファイル・11 箇所の rustdoc は本移設前の
`headless-ui-api.md §4b`/`§4b.3` を指しており、本移設によりパスとして陳腐化する。
公開済みクレートの `src/` を変更すると `.claude/rules/coding-rust.md` の semver
バンプ運用と競合するため、本 PR（イシュー #953）では修正しない（`docs/design/
docs-site-api-reference-split.md` §3-5 S4 が受容を確定している）。

- `crates/headless-ui/src/lib.rs`
- `crates/headless-ui/src/link.rs`
- `crates/headless-ui/src/link_overlay.rs`
- `crates/headless-ui/src/breadcrumb.rs`
- `crates/headless-ui/src/pagination.rs`
- `crates/headless-ui/src/steps.rs`
- `crates/pre-styled-ui/src/lib.rs`
- `crates/pre-styled-ui/tests/pagination_css.rs`

## 1. spec 未反映の注記（旧 §1）

**spec 未反映の注記**: 本クレートに対応する REQ / TASK は
`docs/spec/04-requirements.md` / `05-tasks.md` に存在しない（要件提案は
fandhe-frontend-spec リポジトリの Issue #20 として起票済み、#520 参照）。
本書は実装の現状を記録する位置づけであり、`docs/api/component-api.md`
のような「凍結表」ではない。

## 2. 親トラッキング・担当領域・公開来歴（旧 §2）

- **親トラッキング**: #520（ark-ui / chakra-ui 参考の 2 層 UI コンポーネント構成）
- **本クレートの担当領域**: Phase 1（#521、共通基盤）・Phase 2（#526〜#544、
  個別コンポーネント）
- **crates.io 公開状況**: v0.1.0 で公開済み（イシュー #608）

### 2a. その他の来歴イシュー番号（API ページから除去した個別参照）

API ページ本文の来歴カッコ書きから除去した issue 番号のうち、上記以外の
ものを保存する（`docs/api/headless-ui-api.md` §7.4 の情報消失機械検査対応）。

| Issue | 文脈 |
|---|---|
| #523/#524 | 共通基盤 API（§3）の実装イシュー（Phase 1） |
| #542 | Progress circular の親イシュー |
| #546 | pre-styled-ui 側の親トラッキング（chakra-ui 相当層） |
| #550 | `fandhe_frontend_core` 再エクスポート追加のイシュー |
| #552 | `examples/headless-pre-styled-ui` 追加のイシュー |
| #588 | anchor positioning の親トラッキング |
| #589 | `docs/design/anchor-positioning-design.md`（ADR）確定のイシュー |
| #590 | anchor positioning（Floating UI 相当の placement 計算）実装のイシュー |
| #622 | `css_vars_style` の `same_width` 分岐修正レビュー指摘のイシュー |
| #663 | `data-positioned` マーカー契約（ADR §4.4b）実装のイシュー |
| #712 | `fandhe_frontend_interactive` 再エクスポート追加のイシュー |
| #832 | date-time 系コンポーネント（Calendar/DatePicker/DateInput/Timer）の親トラッキング |
| #837 | color_picker の親イシュー（#838/#839 の親） |
| #838 | `color` モジュール（色変換コア）実装のイシュー |
| #852 | Format ユーティリティの親（Phase 5） |
| #853 | `format` モジュール実装のイシュー |
| #854 | `Locale` 実装のイシュー |

### 4a.6 意図的非対応

Floating UI 高度 middleware（`autoPlacement`/`inline`/`hide`/`size`
（sameWidth 以外）/`VirtualElement`/`autoUpdate` 相当の連続監視）の非採用
判断は `docs/policy/intentional-non-adoption.md` §3.20（正、イシュー #639
で転記済み）を参照する。CSS Anchor Positioning（Web 標準）の非採用は
同書 §3.21 を参照し、一次記録・progressive enhancement の検討経緯は ADR
第 4.5 節・第 4.5a 節を参照する（評価軸・再評価トリガーの表は本書へ
複製しない）。

## 4b. ロードマップ（レイアウト・ナビゲーション系部品、イシュー #716）

### 4b.1 検討の背景

`docs/design/docs-site-styled-ui-adoption.md`（イシュー #694）は、docs
サイト骨格（`crates/docs-site/src/nav.rs`）への pre-styled-ui 適用を
以下 2 点の意味論不整合を理由に見送った。

- §3.1: `nav.rs::sidebar`（文書ナビ・リンク一覧）に対応する部品が
  headless-ui に存在せず、最も近い `menu` は WAI-ARIA `menu` ロール
  （操作可能なコマンドリスト向け）であり転用するとアクセシビリティを
  毀損する
- §3.2: `nav.rs::prev_next_nav`（前後ページャ、アンカー要素全体をカード化
  するリンク）に対応する部品がなく、`card` はアンカー全体のカード化に
  非対応

同書 §5 再評価トリガー 1 は「pre-styled-ui にレイアウト・ナビゲーション系
部品（Breadcrumb / Pagination / 文書ナビ向け Link リスト / Container 等）
が追加されたとき」を明示しており、本イシュー #716 はこのトリガーに先立ち
候補群の追加要否を検討し、恒久文書として記録するものである。

**本節の位置づけ**: 本節は検討結果の記録であり、実装（コード追加）を含ま
ない。追加候補と判断した部品も、実装着手には別途イシュー起票とユーザー
承認を要する（`.claude/rules/out-of-scope-tracking.md`）。

### 4b.2 候補の分類軸

ark-ui / chakra-ui のレイアウト・ナビゲーション系コンポーネントを、
本クレートの設計方針（anatomy + `data-*` + WAI-ARIA、状態機械は
`fandhe_frontend_interactive::Component`/`Hydrate` 経由）に照らして
3 分類する。

| 分類 | 特徴 | 本クレートでの実装形態の見立て |
|---|---|---|
| (a) 状態機械を持つナビ | ページ番号・現在位置等のクライアント状態を持つ | `select`/`menu` と同型（`state::Disclosure`/`SingleSelect` 相当の新規状態機械 + anatomy）。工数大 |
| (b) SSR 静的な意味論ナビ | 「現在位置のハイライト」のみで状態機械不要 | `tabs`/`field` と同型（自由関数のみ、SSR 静的 props で `aria-current`/`data-current` を出力）。工数小〜中 |
| (c) 純粋レイアウトプリミティブ | CSS ボックスモデルのみで ARIA 意味論を持たない | 「プレーンな HTML / CSS を尊重する」という本フレームワークの中核価値（CLAUDE.md Overview）と `docs/policy/intentional-non-adoption.md` の評価軸（明示性・コンテキスト消費）に照らし、headless 層としての意味がない |

### 4b.3 候補ごとの評価と判断

| 候補 | 分類 | ark-ui / chakra-ui の実装状況 | docs-site 利用見込み | 工数参考 | 判断 |
|---|---|---|---|---|---|
| 文書ナビ向け Link リスト（`nav` + リンク一覧 + `aria-current="page"`） | (b) | ark-ui に専用コンポーネントはなく、chakra-ui も汎用 `Link`/`List` の組み合わせで表現する軽量パターン | `nav.rs::sidebar` の意味論不整合（§3.1）を直接解消しうる第一候補 | `field.rs`（740 行）程度。状態機械なし・anatomy と `aria-current`/`data-current` 出力のみ | **追加候補**（最優先）→ **イシュー #756 で実装済み**（headless `crates/headless-ui/src/nav_list.rs` + styled `crates/pre-styled-ui/src/nav_list.rs`。`nav.rs::sidebar` 自体を本部品へ移行済み、§3.1 解消） |
| Link / LinkOverlay（アンカー要素全体のカード化） | (b) | chakra-ui に `Link`/`LinkOverlay`（`LinkBox` パターン、`position: absolute` でアンカーを親要素全面へ拡張する構成）あり。ark-ui に専用コンポーネントはなし | `nav.rs::prev_next_nav` の `card` 非対応（§3.2）を直接解消しうる | `avatar.rs` 相当（独自状態なしの小規模 anatomy）と同程度。工数小 | **追加候補**→ **イシュー #756 で実装済み**（headless `crates/headless-ui/src/link.rs`（Link）・`crates/headless-ui/src/link_overlay.rs`（LinkOverlay）+ styled 対）。`nav.rs::prev_next_nav` を LinkOverlay へ移行済み、§3.2 解消 |
| Breadcrumb | (b) | ark-ui に headless 実体はなく、chakra-ui も styled 合成のみ（状態機械を持たない） | 現時点で docs-site に階層パンくずの利用箇所はない（サイドバー1階層構成のため）。ユーザープロジェクトでの利用見込みはある | `tabs.rs`（790 行）程度。状態機械なし・`aria-current="page"` 出力のみ | **追加候補**（優先度中）→ **イシュー #755 で実装済み**（headless `crates/headless-ui/src/breadcrumb.rs` + styled `crates/pre-styled-ui/src/breadcrumb.rs`。docs-site showcase へは掲示済みだが `nav.rs::sidebar` 自体の置き換えは行っていない、下記 §4b.5 参照） |
| Pagination | (a) | ark-ui に headless 実体あり（ページ番号・件数・現在ページの状態機械を持つ） | 当初は docs-site に該当箇所なし・利用見込み未確認だったが、イシュー #751（親トラッキング #520 の全コンポーネント網羅方針）により保留を解除して実装した（§4 参照、`crates/docs-site/src/showcase.rs::pagination_section` に静的掲示あり） | `select.rs`（1481 行）/`menu.rs`（1818 行）相当だったが、実装は `page_range`（決定的・境界/sibling レンジのマージ）+ 値状態機械の直接実装で完結し想定より小規模だった | **実装済み**（#751。headless: `pagination` モジュール／pre-styled: `pagination` モジュール、golden CSS・`push_recipe` 登録・Size variant あり） |
| Steps | (a) | ark-ui に headless 実体あり（進行状態を持つウィザード的ナビ） | docs-site・examples のいずれにも利用見込みなし | Pagination 同様に工数大 | **実装済み**（イシュー #752 で保留解除。§4 コンポーネント一覧表参照。工数はかかったが状態機械が `count`/`step` の 2 値のみで `progress`/`pin_input` と同型の独自 `Component`/`Hydrate` 直接実装で収まったため、着手障壁は当初見積もりより小さかった） |
| Container / Stack / Flex / Grid / Center 等の純粋レイアウトプリミティブ | (c) | chakra-ui に styled プリミティブとして存在するが、ark-ui に headless 実体はない（ARIA 意味論を持たないため） | 適用対象なし。プレーンな `div` + CSS で代替可能 | — | **意図的非採用**（`docs/policy/intentional-non-adoption.md` の運用に準拠。headless-ui は anatomy・ARIA・状態機械の提供が責務であり、ARIA 意味論を持たない純粋レイアウトは本層の対象外。CSS プリミティブが必要な場合はユーザー側の素の CSS で足り、フレームワーク側の抽象化はコンテキスト消費を増やすだけで利得がない） |

### 4b.4 追加候補の実装方針（将来実装時の不変条件、参考）

追加候補（文書ナビ向け Link リスト・Link/LinkOverlay・Breadcrumb）を
将来実装する場合、以下を満たすこと。

- 既存 (b) 群（`tabs`/`field`）と同様、自由関数のみで SSR 静的マークアップ
  を組み立てられること（状態機械を必須にしない）
- `href` 等のリンク属性値はすべて `fandhe_frontend_core::render` の既定
  エスケープ（REQ-1）を経由し、`raw_html()` を使用しないこと
- 外部依存はゼロのまま（`fandhe-frontend-core`/`-interactive` のみ）を
  維持すること
- 現在位置の表現は `aria-current`（値は `"page"` 等の APG 準拠語彙）と
  `data-current` の併用とし、既存の `data-state` 値語彙一元化方針
  （§6 不変条件 3）を踏襲すること

### 4b.5 再評価条件

- 追加候補（文書ナビ向け Link リスト・Link/LinkOverlay・Breadcrumb）は
  すべて実装済み（Breadcrumb はイシュー #755、文書ナビ向け Link リスト・
  Link/LinkOverlay はイシュー #756）。`docs/design/docs-site-styled-ui-adoption.md`
  §5 再評価トリガー 1 の発火条件を満たしたため、同書 §3.1/§3.2 は
  イシュー #756 で再評価し「解消済み」へ更新した（詳細は同書参照）。
- 保留（Pagination・Steps）はいずれも実装済み（Pagination はイシュー #751、
  Steps はイシュー #752 で保留解除・実装済み、§4b.3 参照）
- 意図的非採用（純粋レイアウトプリミティブ）の再評価は
  `docs/policy/intentional-non-adoption.md` §4 の運用（評価軸の充足確認を
  Issue・PR に明記）に従う

### 4d.3 スコープ外（#835 時点）

- キーボードナビゲーション（矢印キーでの gridcell フォーカス移動・roving tabindex）の実 DOM 配線
- 範囲選択（range mode）・複数月表示（multi-month）・年/月ビュー切替
- DateInput（#834）との配線・wasm-full ハイドレーション配線

### 4e.2 スコープ外

- en/ja 以外のロケール対応（将来イシュー、`Locale` enum への variant 追加として行う）
- `format_time` へのロケール依存表記（和暦・時制表記等）の導入
- `LocaleProvider`・`AsyncListCollection` の Rust 等価概念対応表
  （`docs/design/component-coverage-map.md` §8、イシュー #855 で追加済み）
- `fandhe-frontend-pre-styled-ui`・examples ショーケースへの実演追加
  （format はノードを返さない純関数であり既存ショーケース枠に該当しない）

## T. トレーサビリティ（`mod → 対応イシュー/PR`）

| コンポーネント | モジュール | 対応イシュー |
|---|---|---|
| Collapsible | `collapsible` | #529 |
| Accordion（single モード） | `accordion` | #527 |
| Tabs | `tabs` | #528 |
| Tooltip | `tooltip` | #533 |
| Dialog | `dialog` | #531 |
| Popover | `popover` | #532 |
| RadioGroup | `radio_group` | #536 |
| Switch | `switch` | #537 |
| Field | `field` | #538 |
| Menu | `menu` | #540 |
| Select | `select` | #541 |
| Avatar | `avatar` | #543 |
| NumberInput | `number_input` | #738 |
| PasswordInput | `password_input` | #740 |
| Slider | `slider` | #741 |
| PinInput | `pin_input` | #739 |
| TagsInput | `tags_input` | #744 |
| RatingGroup | `rating_group` | #742 |
| Editable | `editable` | #745 |
| Toggle | `toggle` | #746 |
| ToggleGroup（single モード） | `toggle_group` | #746 |
| MultiToggleGroup（multiple モード） | `toggle_group` | #746 |
| SegmentGroup | `segment_group` | #743 |
| Listbox / MultiListbox | `listbox` | #750 |
| Combobox | `combobox` | #749 |
| Steps | `steps` | #752（§4b.3 の保留解除） |
| TreeView | `tree_view` | #753 |
| Pagination | `pagination` | #751 |
| Breadcrumb | `breadcrumb` | #755 |
| HoverCard | `hover_card` | #759 |
| Carousel | `carousel` | #754 |
| Drawer | `drawer` | #758 |
| Link | `link` | #756 |
| LinkOverlay | `link_overlay` | #756 |
| NavList | `nav_list` | #756 |
| ActionBar | `action_bar` | #762 |
| Toast | `toast` | #760 |
| Checkbox | `checkbox` | #535 |
| Progress（linear + circular） | `progress` | #544（linear）/#600（circular） |
| ToggleTip | `toggle_tip` | #761 |
| VisuallyHidden | `visually_hidden` | #776 |
| SkipNav | `skip_nav` | #776 |
| Clipboard | `clipboard` | #773 |
| QrCode | `qr_code` | #774 |
| FloatingPanel | `floating_panel` | #827 |
| ScrollArea | `scroll_area` | #825 |
| DownloadTrigger | `download_trigger` | #828 |
| Splitter | `splitter` | #826（`docs/policy/intentional-non-adoption.md` §7・`docs/design/component-coverage-map.md` の保留を解除） |
| JsonTreeView | `json_tree_view` | #829（`tree_view` #753 の派生、`docs/policy/intentional-non-adoption.md` §7 の保留解除） |
| ColorPicker | `color_picker` | #839（親 #837、`docs/policy/intentional-non-adoption.md` §7 の保留解除） |
| FileUpload | `file_upload` | #840 |
| DateInput | `date_input` | #834（`date` #833 を先行前提として利用、`docs/policy/intentional-non-adoption.md` §7・`docs/design/component-coverage-map.md` の date-time 系「保留」を DateInput 分のみ解除） |
| Timer | `timer` | #836 |
| Tour | `tour` | #841（#735 保留の解除） |
| AngleSlider | `angle_slider` | #842（`docs/policy/intentional-non-adoption.md` §3.22 の再導入） |
| SignaturePad | `signature_pad` | #843（`docs/policy/intentional-non-adoption.md` §3.22 の再導入） |
| ImageCropper | `image_cropper` | #844（`docs/policy/intentional-non-adoption.md` §3.22 の再導入、先例は AngleSlider #842） |
| Fieldset | `fieldset` | #602 |

## R. 関連文書

- `../api/headless-ui-api.md`
- `../design/docs-site-api-reference-split.md`
- `../design/docs-site-styled-ui-adoption.md`: docs サイト骨格への pre-styled-ui
  適用可否の評価記録。§5 再評価トリガー 1 は本書 §4b のレイアウト・ナビゲーション
  系部品ロードマップと相互参照の関係にある
- `../policy/intentional-non-adoption.md`
- `../design/component-coverage-map.md`
