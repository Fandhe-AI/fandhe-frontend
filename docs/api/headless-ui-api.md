# fandhe-frontend-headless-ui API

## 1. 目的とトレーサビリティ

本ドキュメントは `fandhe-frontend-headless-ui`（ark-ui / chakra-ui 参考の
2 層 UI コンポーネント構成、親トラッキング #520）が提供する headless
（unstyled）UI コンポーネント層の公開 API 表面をまとめる。上層の
`fandhe-frontend-pre-styled-ui`（chakra-ui 相当、#520/#546）は本層の
anatomy・`data-*`・WAI-ARIA 出力を前提にスタイルを重ねる。

**spec 未反映の注記**: 本クレートに対応する REQ / TASK は
`docs/spec/04-requirements.md` / `05-tasks.md` に存在しない（要件提案は
fandhe-frontend-spec リポジトリの Issue #20 として起票済み、#520 参照）。
本書は実装の現状を記録する位置づけであり、`docs/api/component-api.md`
のような「凍結表」ではない。

`docs/api/component-api.md` との整合: 本クレートのコンポーネントはすべて
「`fandhe_frontend_core::Node` を返す通常の Rust 関数」（REQ-5 の凍結 API
前提）として実装され、`fandhe_frontend_core::render` の既定エスケープ
（REQ-1）を必ず経由する。`raw_html()` は使用しない。

## 2. 位置づけ

- **親トラッキング**: #520（ark-ui / chakra-ui 参考の 2 層 UI コンポーネント構成）
- **本クレートの担当領域**: Phase 1（#521、共通基盤）・Phase 2（#526〜#544、
  個別コンポーネント）
- **crates.io 公開状況**: v0.1.0 で公開済み（イシュー #608）。`fandhe-frontend-core` /
  `fandhe-frontend-interactive`（いずれも crates.io バージョン依存）のみへ依存する

## 3. 共通基盤 API（Phase 1、#523/#524）

| モジュール/型 | 役割 |
|---|---|
| `anatomy::Anatomy` / `anatomy::anatomy` | `data-scope`/`data-part` を付与してパーツノード（`div`/`button`/`span`/`input` 等）を組み立てる。全コンポーネント共通の anatomy 基盤 |
| `data_attrs` | `data-state`/`data-disabled`/`data-invalid`/`data-orientation`/`data-readonly`/`data-required` 等の状態属性ヘルパ。`Orientation` enum（`Horizontal`/`Vertical`） |
| `aria` | `role`/`aria-*`（`aria_checked`/`aria_controls`/`aria_describedby`/`aria_disabled`/`aria_expanded`/`aria_haspopup`/`aria_hidden`/`aria_invalid`/`aria_label`/`aria_labelledby`/`aria_modal`/`aria_orientation`/`aria_selected`）の WAI-ARIA 属性ヘルパ |
| `state::OpenState` | `Open`/`Closed` の 2 値状態（`Default` は `Closed`。SSR の状態なし初期描画に対応）。`as_data_state()`/`is_open()`/`toggled()` |
| `state::Disclosure` / `state::DisclosureAction` | 単一の開閉状態機械。`fandhe_frontend_interactive::Component`/`Hydrate` を実装し、dispatch アクション名 `"open"`/`"close"`/`"toggle"` を受理する |
| `state::SingleSelect` / `state::SingleSelectAction` | 「高々 1 項目が選択される」状態機械（Accordion の single モード等が使用）。dispatch アクション名 `"select"`/`"deselect"`/`"toggle"` |

これらは Dialog / Accordion / Tabs / Collapsible / Popover / Tooltip
（Phase 2、#526〜#533）が共通で使う「open/closed・selected」の dispatch 契約・
`data-state` 整合・SSR/hydration 契約を一度だけ実装したものであり、各
コンポーネントはフィールドとして埋め込んで再利用する。

## 4. コンポーネント一覧（実装済み、Phase 2）

| コンポーネント | モジュール | anatomy パーツ | 埋め込む状態機械 | 対応イシュー |
|---|---|---|---|---|
| Collapsible | `collapsible` | Root/Trigger/Indicator/Content | `state::Disclosure` | #529 |
| Accordion（single モード） | `accordion` | Root/Item/ItemTrigger/ItemIndicator/ItemContent | `state::SingleSelect` | #527 |
| Tabs | `tabs` | Root/List/Trigger/Content（自由関数 `tabs()`、SSR 静的選択状態のみ） | なし（クリック/dispatch は wasm 層のスコープ） | #528 |
| Tooltip | `tooltip` | Root/Trigger/Positioner/Content/Arrow/ArrowTip | `state::Disclosure` | #533 |
| Dialog | `dialog` | Root/Trigger/Backdrop/Positioner/Content/Title/Description/CloseTrigger | `state::Disclosure` | #531 |
| Popover | `popover` | Root/Trigger/Anchor/Positioner/Arrow/ArrowTip/Content/Title/Description/CloseTrigger/Indicator | `state::Disclosure` | #532 |
| RadioGroup | `radio_group` | Root/Label/Item/ItemControl/ItemText/ItemHiddenInput | `state::SingleSelect` | #536 |
| Switch | `switch` | Root/Control/Thumb/Label/HiddenInput | 独自実装（`"checked"`/`"unchecked"` 語彙が `Disclosure` と異なるため `Component`/`Hydrate` を直接実装） | #537 |
| Field | `field` | Root/Label/Input/Textarea/Select/HelperText/ErrorText/RequiredIndicator | なし（`invalid`/`disabled`/`required`/`readonly` は SSR 静的な props） | #538 |
| Menu | `menu` | Root/Trigger/Indicator/Positioner/Content/Arrow/ArrowTip/Item/ItemGroup/ItemGroupLabel/Separator | `state::Disclosure` | #540 |
| Select | `select` | Root/Label/Control/Trigger/ValueText/ClearTrigger/Indicator/Positioner/Content/ItemGroup/ItemGroupLabel/Item/ItemText/ItemIndicator/HiddenSelect | `state::Disclosure` + `state::SingleSelect`（開閉 + 選択値の合成） | #541 |
| Avatar | `avatar` | Root/Image/Fallback | 独自実装（`"loading"`/`"loaded"`/`"error"` の 3 値ステータス、`ImageStatus`） | #543 |
| NumberInput | `number_input` | Root/Label/Control/Input/IncrementTrigger/DecrementTrigger | 独自実装（連続量の値のため `data-state` を持たず `Component`/`Hydrate` を直接実装。数値整形・パースはロケール非依存で決定的、`step` 演算は小数桁への丸めで浮動小数点ドリフトを防ぐ） | #738 |
| Slider | `slider` | Root/Label/Control/Track/Range/Thumb/HiddenInput/ValueText | 独自実装（連続量の値のため `data-state` を持たず `Component`/`Hydrate` を直接実装。`value` は常に `min` 起点で `step` 単位へスナップしてから `[min, max]` へ clamp する。`thumb` が `role="slider"` + `aria-valuemin/max/now`/`aria-orientation` を担う） | #741 |
| PinInput | `pin_input` | Root/Label/Control/Input/HiddenInput | 独自実装（固定桁数の文字配列 + フォーカス位置、`Disclosure`/`SingleSelect` の語彙に収まらないため `Component`/`Hydrate` を直接実装） | #739 |
| SegmentGroup | `segment_group` | Root/Indicator/Item/ItemText/ItemControl/ItemHiddenInput | `radio_group::RadioGroup`（`state::SingleSelect`）へ全委譲（独自の状態機械を新設せず、既存 RadioGroup の dispatch/hydration をそのまま再利用する） | #743 |

**未実装（open イシュー、後続で追補）**: Checkbox（#535）・Progress（#544）。
本表はこれらの実装完了時に更新する。

## 4a. 位置決め（anchor positioning、イシュー #590、親 #588）

Popover/Tooltip/Menu/Select の `positioner`/`arrow`/`arrow_tip` は「CSS フック
（`data-*` セレクタ）のみ」だったが、イシュー #590（正の規範文書は
`docs/design/anchor-positioning-design.md`。以下 ADR）で Floating UI 相当の
placement 計算が実装済みとなった。

### 4a.1 対象コンポーネントと anatomy

| コンポーネント | 対応パーツ | `data-scope` | arrow の有無 |
|---|---|---|---|
| Popover | Positioner/Arrow/ArrowTip | `"popover"` | あり |
| Tooltip | Positioner/Arrow/ArrowTip | `"tooltip"` | あり |
| Menu | Positioner/Arrow/ArrowTip | `"menu"` | あり |
| Select | Positioner のみ | `"select"` | なし |

再計算対象の走査は開いている positioner のみに限定する
セレクタ `[data-part="positioner"][data-state="open"]`
（`crates/wasm-full/src/position.rs` の `OPEN_POSITIONER_SELECTOR`）を使う。

### 4a.2 placement API（`positioning` モジュール、クレートルート再エクスポート）

`crates/headless-ui/src/positioning.rs` が外部依存ゼロの純粋関数として
提供し、クレートルート（`lib.rs`）から次の型・関数を再エクスポートする:
`compute_position` / `css_vars_style` / `data_align` / `data_side` /
`placement_attrs` / `Align` / `ArrowPosition` / `Placement` /
`PositioningConfig` / `Rect` / `ResolvedPosition` / `Side` / `Size`。
CSS 変数名定数は `positioning::css_vars`（`X`/`Y`/`REFERENCE_WIDTH`/
`ARROW_X`/`ARROW_Y`）としてクレートルートとは別に公開される。

- [`Placement`] は `Side`（`top`/`bottom`/`left`/`right`）× `Align`
  （`start`/`center`/`end`）の組み合わせで、12 placement 語彙
  （`top`/`top-start`/`top-end`/`bottom`/`bottom-start`/`bottom-end`/
  `left`/`left-start`/`left-end`/`right`/`right-start`/`right-end`）を
  型として一元化する。`as_str()`/`from_str()` は相互に逆写像であり、
  `from_str()` は未知の値に対し `None` を返す（fail-closed）。
- `data-*` 契約:
  - `data-side`（`top`/`bottom`/`left`/`right`）・`data-align`
    （`start`/`center`/`end`）は **flip 適用後の確定値の出力専用**で
    あり、再計算のたびに上書きされる CSS セレクタ用の属性である。
  - 希望 placement（flip 適用前）は別の永続化領域である
    `data-requested-side`/`data-requested-align` 属性に保持する
    （wasm 層の `reposition_one` が初回のみ書き込む。`data-side`/
    `data-align` を希望値の保持先に流用すると flip 後に希望値が
    失われるため分離した、詳細は ADR §4.4a）。
  - SSR/SSG では位置計算そのものをスキップし、[`placement_attrs`] による
    `data-side`/`data-align` の静的出力と `pre-styled-ui` 側の静的 CSS
    フォールバックで初期表示を描画する。

### 4a.3 位置計算 API（純粋関数・外部依存ゼロ・`web-sys` 非依存）

入力型:

- [`Rect`]（`x`/`y`/`width`/`height`）: anchor（参照要素）の矩形。
- [`Size`]（`width`/`height`）: floating 要素・viewport の寸法。
- [`PositioningConfig`]（`placement`/`offset`/`flip`/`shift`/`same_width`）:
  `Default` は `bottom-center`・`offset: 0.0`・`flip`/`shift` 有効・
  `same_width: false`。

`compute_position(anchor: Rect, floating: Size, viewport: Size, config: &PositioningConfig, has_arrow: bool) -> ResolvedPosition`:

1. `config.placement` で主軸・交差軸座標を計算する。
2. `flip`（主軸の単純反転 1 候補のみ）が有効かつ主軸方向で viewport を
   はみ出す場合、反転後の座標で置き換える（反転後も収まらない場合は
   反転後の座標をそのまま採用する）。
3. `shift`（交差軸方向の viewport 内クランプ）を適用する。
4. `has_arrow` が `true` のときのみ arrow 座標（[`ArrowPosition`]、floating
   要素左上原点の相対座標）を計算する（Select は arrow を持たないため
   呼び出し側が `false` を渡す）。

異常入力（`NaN`/`Infinity`・負の幅高さ・viewport 寸法 0 等）は
fail-closed: `panic!`/`unwrap()` を使わず、`config.placement` のまま座標
`(0.0, 0.0)`・`arrow: None` を返す。

出力型 [`ResolvedPosition`]（`x`/`y`/確定 `placement`/`Option<ArrowPosition>`）。

### 4a.4 CSS 変数契約（`--fandhe-*`）

| 変数 | 内容 |
|---|---|
| `--fandhe-x` | floating 要素の確定 x 座標（px） |
| `--fandhe-y` | floating 要素の確定 y 座標（px） |
| `--fandhe-reference-width` | anchor 幅（sameWidth 用、`same_width` 有効時のみ出力） |
| `--fandhe-arrow-x` | arrow の x 座標（px、arrow を持つ場合のみ出力） |
| `--fandhe-arrow-y` | arrow の y 座標（px、arrow を持つ場合のみ出力） |

`css_vars_style(position: &ResolvedPosition, reference_width: f64, same_width: bool) -> String`:

- `same_width == false` のときは `--fandhe-reference-width` 自体を
  出力しない（`PositioningConfig::same_width` をそのまま渡す契約。
  イシュー #622 レビュー指摘: 従来は `same_width` の値によらず常に
  出力しており、コンポーネント種別ごとの sameWidth 既定値が実行時挙動に
  影響しない不具合があった）。
- `position.arrow` が `Some` のときのみ arrow 2 変数を出力する。
- 出力は内部生成の数値書式（px）のみからなり、非有限値は最終防御線として
  `0.0` へ丸める。
- 戻り値は `("style", &value)` として既存の `attrs: Vec<(&'a str, &'a str)>`
  引数へ渡し、[`fandhe_frontend_core::render`] の既定エスケープ経由で
  出力する契約とする（§6 不変条件 7 と同一）。

コンポーネント別の sameWidth 既定（`fandhe-frontend-wasm-full` の
`PositionedKind::same_width_default`）: Menu/Select は `true`、
Popover/Tooltip は `false`。

### 4a.5 計測注入・再計算（`fandhe-frontend-wasm-full` の `position` モジュール）

`headless-ui` は `web-sys` 非依存のまま維持し、実 DOM 計測
（`getBoundingClientRect`・viewport 寸法）とスクロール/リサイズ契機の
再計算は `fandhe-frontend-wasm-full`（`position` モジュール）が担う。
再計算はスクロール・リサイズイベントを契機とした**離散的**な呼び出しであり、
`autoUpdate` 相当の連続監視は非採用。

- 純粋ロジック層（native `cargo test` 可）: `PositionedKind`
  （`from_scope`: 未知の `data-scope` 値は `None` の fail-closed /
  `has_arrow`: Select のみ `false` / `same_width_default`: 上記表）・
  `parse_side_attr`/`parse_align_attr`（属性欠落・未知値は
  `bottom`/`center` へ fail-closed）・`resolve_requested_placement`・
  `Measurement`・`resolve_position(kind, measurement, requested) -> RepositionResult`
  （flip/shift 常時有効・offset `0.0` 固定）。
- 配線層（`#[cfg(target_arch = "wasm32")]`）: `reposition_all`（開いている
  positioner を `OPEN_POSITIONER_SELECTOR` で走査）・`PositionController`
  （scroll/resize リスナー）。
- DOM 属性値（`data-side`/`data-requested-side` 等）は改ざんされうる
  クライアント入力として扱い、fail-closed でパースする。

### 4a.6 意図的非対応

Floating UI 高度 middleware（`autoPlacement`/`inline`/`hide`/`size`
（sameWidth 以外）/`VirtualElement`/`autoUpdate` 相当の連続監視）の非採用
判断は `docs/policy/intentional-non-adoption.md` §3.20（正、イシュー #639
で転記済み）を参照する。CSS Anchor Positioning（Web 標準）の非採用は
同書 §3.21 を参照し、一次記録・progressive enhancement の検討経緯は ADR
第 4.5 節・第 4.5a 節を参照する（評価軸・再評価トリガーの表は本書へ
複製しない）。

### 4a.7 `data-positioned` マーカー契約（イシュー #663、ADR §4.4b）

`fandhe-frontend-wasm-full` の `position::wiring::reposition_one` は座標
反映のたびに `positioner` へ `data-positioned=""`（値なしの存在マーカー）
を書き込む。`headless-ui` 層（本モジュール）は SSR/SSG のいずれの出力
経路でもこの属性を一切出力しない（[`placement_attrs`] は `data-side`/
`data-align` の 2 属性のみを返す）。`fandhe-frontend-pre-styled-ui`
（`crates/pre-styled-ui/src/menu.rs`/`select.rs` の `recipe()`）はこの
非対称性を利用し、マーカーの有無で「SSR 静的フォールバック（`position:
absolute` + ローカル座標系）」と「wasm 確定座標（`position: fixed` +
viewport 座標系、`--fandhe-x`/`--fandhe-y` を `transform: translate3d`
で消費）」を切り替える。マーカー不在（wasm 未稼働）では常に静的表示へ
fail-closed に留まる。arrow（Menu のみ、`has_arrow()` が Select を対象外
とする、§4a.2）は `--fandhe-arrow-x`/`--fandhe-arrow-y` を変数フォール
バックのみで消費し、マーカー切り替えを必要としない。

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
| 文書ナビ向け Link リスト（`nav` + リンク一覧 + `aria-current="page"`） | (b) | ark-ui に専用コンポーネントはなく、chakra-ui も汎用 `Link`/`List` の組み合わせで表現する軽量パターン | `nav.rs::sidebar` の意味論不整合（§3.1）を直接解消しうる第一候補 | `field.rs`（740 行）程度。状態機械なし・anatomy と `aria-current`/`data-current` 出力のみ | **追加候補**（最優先） |
| Link / LinkOverlay（アンカー要素全体のカード化） | (b) | chakra-ui に `Link`/`LinkOverlay`（`LinkBox` パターン、`position: absolute` でアンカーを親要素全面へ拡張する構成）あり。ark-ui に専用コンポーネントはなし | `nav.rs::prev_next_nav` の `card` 非対応（§3.2）を直接解消しうる | `avatar.rs` 相当（独自状態なしの小規模 anatomy）と同程度。工数小 | **追加候補** |
| Breadcrumb | (b) | ark-ui に headless 実体はなく、chakra-ui も styled 合成のみ（状態機械を持たない） | 現時点で docs-site に階層パンくずの利用箇所はない（サイドバー1階層構成のため）。ユーザープロジェクトでの利用見込みはある | `tabs.rs`（790 行）程度。状態機械なし・`aria-current="page"` 出力のみ | **追加候補**（優先度中。工数小さく他 (b) 群と設計を共有できるが docs-site 側の直接解消対象ではない） |
| Pagination | (a) | ark-ui に headless 実体あり（ページ番号・件数・現在ページの状態機械を持つ） | docs-site に該当箇所なし（現状ページ分割一覧を持たない）。現時点で利用見込みが確認できない | `select.rs`（1481 行）/`menu.rs`（1818 行）相当。状態機械の新規設計を要し工数大 | **保留**（利用見込みが確認できてから再評価。状態機械設計コストが (b) 群より大きく優先度を下げる） |
| Steps | (a) | ark-ui に headless 実体あり（進行状態を持つウィザード的ナビ） | docs-site・examples のいずれにも利用見込みなし | Pagination 同様に工数大 | **保留** |
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

- 追加候補（文書ナビ向け Link リスト・Link/LinkOverlay・Breadcrumb）が
  実際に実装された場合、`docs/design/docs-site-styled-ui-adoption.md`
  §5 再評価トリガー 1 の発火条件を満たすため、同書 §3.1/§3.2 の再評価を
  行う
- 保留（Pagination・Steps）は、docs-site またはユーザープロジェクトで
  ページ分割一覧・ウィザード的ナビの利用見込みが具体化した時点で再評価
  する
- 意図的非採用（純粋レイアウトプリミティブ）の再評価は
  `docs/policy/intentional-non-adoption.md` §4 の運用（評価軸の充足確認を
  Issue・PR に明記）に従う

## 5. 呼び出し規約（SSR / CSR 共通の前提）

- 各コンポーネントの anatomy パーツ（`root`/`trigger`/`content` 等）は
  **状態を引数で受け取る純粋関数**として実装されており、SSR は自由関数を
  直接呼ぶだけで静的マークアップを組み立てられる（状態機械（`Accordion`/
  `Dialog`/`Switch` 等の型）を経由する必要はない）。
- CSR/hydration は各コンポーネントの状態機械型（`Accordion`/`Dialog` 等）を
  経由し、`fandhe_frontend_interactive::Component`/`Hydrate` の dispatch で
  状態遷移する。クリック/キーボード操作の実挙動は wasm 層
  （`fandhe-frontend-wasm-client`/`-wasm-full`）の責務であり、本クレートの
  スコープ外。
- `examples/headless-pre-styled-ui`（#552）は自由関数のみを使う SSR
  静的ショーケースの実例。

## 6. セキュリティ不変条件

1. 属性名（`data-*`/`aria-*`/`type`/`role`/`hidden`/`disabled`/`id` 等）は
   すべて `&'static str` リテラルで固定されており、動的値が属性名スロットへ
   混入する経路はない。
2. 動的値（`value`/`id`/`controls`/`labelled_by`/呼び出し側 `attrs`/
   `children` テキスト）は `fandhe_frontend_core::render` の既定エスケープ
   （REQ-1）を必ず経由する。本クレート内で `raw_html()` は使用しない。
3. `data-state` 値語彙（`"open"`/`"closed"`/`"checked"`/`"unchecked"` 等）は
   各状態モジュール（`state`/`switch`/`avatar` 等）に一元化し、パーツ関数
   側で独自の値を作らない。
4. hydration 属性（`data-hydrate-*`）はクライアント側で改ざんされうる入力
   として扱う。各状態機械の `Hydrate` 実装は既存の状態機械
   （`Disclosure`/`SingleSelect`）へ委譲することで、panic せず
   `HydrateError` を返す保証を継承する。
5. `#![forbid(unsafe_code)]`（REQ-2）。`unsafe` はクレート全体で使用しない。
6. 外部依存は `fandhe-frontend-core` / `fandhe-frontend-interactive`
   （いずれも path）のみ（`.claude/rules/coding-rust.md`）。加えて本クレートは
   `fandhe_frontend_core`（#550）・`fandhe_frontend_interactive`（イシュー
   #712）の両方をクレートそのものとして再エクスポートし、本クレート単独
   依存の利用者が `Component`/`Hydrate`/`dispatch`/`HydrateError`/
   `render_for_hydration` を含む hydration API まで到達できるようにしている
   （`docs/api/pre-styled-ui-api.md` §3b・`crates/headless-ui/tests/
   interactive_reexport.rs` 参照）。
7. `positioning::css_vars_style(position, reference_width, same_width)` が
   返す `style` 属性値は内部生成の数値書式（px）のみからなり、呼び出し側は
   必ず既存の `attrs` 引数 → 上記 2 の既定エスケープを経由して出力する
   （`same_width == false` のとき `--fandhe-reference-width` は出力しない、
   イシュー #590、`docs/design/anchor-positioning-design.md` §7）。

## 7. 関連ドキュメント

- [`docs/api/component-api.md`](./component-api.md): `Node`/`el`/`text`/
  `raw_html`/`render` の凍結 API 表面（本クレートが薄く委譲する下層）
- [`docs/api/pre-styled-ui-api.md`](./pre-styled-ui-api.md): 本クレートの
  上層（chakra-ui 相当）
- [`examples/headless-pre-styled-ui/README.md`](../../examples/headless-pre-styled-ui/README.md):
  本クレートのショーケース正本サンプル
- `docs/design/anchor-positioning-design.md`: anchor positioning の設計確定書
  （イシュー #589、正の規範文書。docs サイト nav.toml 未登録の内部設計文書
  のためリンク化しない）
- `docs/policy/intentional-non-adoption.md` §3.20/§3.21: anchor positioning
  関連（Floating UI 高度 middleware・CSS Anchor Positioning）の非採用判断の
  正（同様に nav.toml 未登録のためリンク化しない）
- `docs/design/docs-site-styled-ui-adoption.md`: docs サイト骨格への
  pre-styled-ui 適用可否の評価記録。§5 再評価トリガー 1 は本書 §4b の
  レイアウト・ナビゲーション系部品ロードマップと相互参照の関係にある
  （同様に nav.toml 未登録のためリンク化しない）
- `.claude/skills/ark-ui/`: 設計時の参考にした ark-ui リファレンススキル
