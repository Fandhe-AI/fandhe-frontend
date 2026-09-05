# fandhe-frontend-pre-styled-ui 実装ノート（docs サイト非掲載）

> 本書は docs サイトへ出力しない内部設計記録である（`site/nav.toml` へ登録
> しない）。リポジトリは public であり「非掲載＝非公開」ではない。
> 分離方針は `docs/design/docs-site-api-reference-split.md`（イシュー #952）。
> 凍結された公開 API 契約自体の正は
> [`docs/api/pre-styled-ui-api.md`](../api/pre-styled-ui-api.md)。
> Phase 7 の全文検索インデックス（#956〜#958）にも本書は含めない
> （既定、`docs-site-api-reference-split.md` §6 再評価トリガー 4）。

## 0. 旧 → 新 マッピング表

| 旧（`docs/api/pre-styled-ui-api.md`） | 新（本書） |
|---|---|
| §1 目的とトレーサビリティ（来歴・spec 未反映の注記） | 本書 §1 |
| §2 実装状況（記載方針・イシュー列挙 prose・由来イシュー列・examples 統合来歴） | 本書 §2 |
| §3a（#685 当時の経緯・PR #679・#729 切り替え・#693 消化） | 本書 §3 |
| §3b（背景・採用方針案 A の根拠・棄却案 B） | 本書 §4 |
| §4 設計方針（各項目の詳細な設計判断） | 本書 §5 |
| §4b avatar（PR #695 Bugbot 指摘・#684 是正の来歴） | 本書 §6 |
| §4c radio_group（イシュー番号来歴） | 本書 §7 |
| §4d 複合部品 variant 表（元「状態」列全文・`tabs_with_root_attrs` 新設経緯） | 本書 §8 |
| §4d data-focus-visible（実装時系列・wasm-full 配線経緯） | 本書 §9 |
| §4e checkbox（イシュー番号来歴） | 本書 §10 |
| §4f input/textarea/native_select（スコープ外節） | 本書 §11 |
| §4g checkbox_card/radio_card（スコープ外節・chakra-ui/ark-ui 由来判断） | 本書 §12 |
| §4h status/empty_state（テストファイル列挙・スコープ外節） | 本書 §13 |
| §4i タイポグラフィ（chakra-ui からの縮約・prose との役割分担） | 本書 §14 |
| §4j charts 軸ほか（chakra-ui からの縮約） | 本書 §15 |
| §4k line/area/sparkline（chakra-ui からの縮約） | 本書 §16 |

## 1. 目的とトレーサビリティ（来歴）

`fandhe-frontend-pre-styled-ui` は親トラッキング #520・骨格新設 #546 で
新設した。`fandhe-frontend-headless-ui` と同様、本クレートに対応する
REQ / TASK は `docs/spec/` に存在しない（要件提案は fandhe-frontend-spec
リポジトリの Issue #20 として起票済み、#520 参照）。

## 2. 実装状況（v0.31.0 時点、2026-07-24 更新・イシュー来歴）

**記載方針**: 実装済み API の正は `crates/pre-styled-ui/src/lib.rs` 冒頭の
rustdoc および各モジュール冒頭の rustdoc とする。API ページの § 2 相当節は
モジュール一覧の概要のみを保持し、イシューごとの進行状態（未着手・実装中・
マージ待ち等）は記載しない。マージ済みイシューを都度更新する運用は
陳腐化しやすく、実際に骨格新設（#546）時点の記述が長期間放置されていた
（イシュー #714）。

本クレートは第 5 弾ツリー（#680）完了・crates.io v0.4.0 公開（#686）・
checkbox styled ラッパー追加（#730）・静的フォーム部品 3 種追加（#737）・
NumberInput styled ラッパー追加（#738）・PinInput styled ラッパー追加
（#739）・PasswordInput styled ラッパー追加（#740）・Slider styled ラッパー
追加（#741）・RatingGroup styled ラッパー追加（#742）・SegmentGroup styled
ラッパー追加（#743）・TagsInput styled ラッパー追加（#744）・Editable
styled ラッパー追加（#745）・Toggle/ToggleGroup styled ラッパー追加
（#746）・CheckboxCard/RadioCard styled バリエーション追加（#747）・
Combobox styled ラッパー追加（#749）・Pagination styled ラッパー追加
（#751）・Steps styled ラッパー追加（#752）・Breadcrumb styled ラッパー
追加（#755）・Carousel styled ラッパー追加（#754）・Drawer styled ラッパー
追加（#758）・Link/LinkOverlay/NavList styled ラッパー追加（#756）・
HoverCard styled ラッパー追加（#759）・ToggleTip styled ラッパー追加
（#761）・Progress circular 対応追加（#763）・Skeleton 静的部品追加
（#764）・Tag/Kbd/Code styled 静的部品追加（#768）・Image/Icon 静的部品
追加（#770）・Status/EmptyState 静的部品追加（#765）・タイポグラフィ静的
部品 6 種追加（#771）・Separator 静的部品追加（#772）・Highlight 静的部品
追加（#775）・Clipboard headless ラッパー追加（#773）・QrCode styled
ラッパー追加（#774）・VisuallyHidden/SkipNav 静的部品追加（#776）・
ActionBar styled ラッパー追加（#762）・Toast styled ラッパー追加
（#760）・Stat/Timeline styled 静的部品追加（#769）・Table/DataList
静的部品追加（#767）・FloatingPanel styled ラッパー追加（#827）・
ScrollArea headless ラッパー追加（#825）・DownloadTrigger headless
ラッパー追加（#828）・Splitter styled ラッパー追加（#826、
`docs/policy/intentional-non-adoption.md` §7 の保留解除）・JsonTreeView
styled ラッパー追加（#829、`tree_view` #753 の派生）・button の icon-only
修飾 variant（`icon_button`/`close_button`）追加（#830、既存 `button`
モジュールの拡張のため新規モジュールは増えない）・Marquee 静的部品追加
（#831、`docs/policy/intentional-non-adoption.md` §3.24 の意図的非採用を
再導入）・ColorSwatch 静的部品追加（#838）・Calendar/DatePicker styled
ラッパー追加（#835、親トラッキング #832、`docs/design/component-coverage-map.md`
保留解除）・charts 基盤（座標スケーリング・SVG ノード木生成・`ChartData`
モデル、#846）・charts 軸/グリッド/凡例/ツールチップ追加（#847。
いずれも公開時点未反映）・PieChart/DonutChart styled 静的部品追加
（#850、charts 基盤（#846）を用いた初のチャート部品。
`docs/policy/intentional-non-adoption.md` §7 の chakra-ui charts 保留を
pie-chart/donut-chart の 2 件で解除）・ScatterChart/RadarChart styled 部品
追加（#851、親 Phase #845、charts 基盤 #846 の上に実装。同 §7 の charts 系
保留を scatter-chart/radar-chart 分のみ解除。いずれも公開時点未反映）・
AngleSlider headless ラッパー追加（#842、`docs/policy/intentional-non-adoption.md`
§3.22 の意図的非採用の再導入。variant 軸のため styled `root` を再定義する
選択的 re-export、状態機械 `AngleSlider` は非再エクスポート）・SignaturePad
headless ラッパー追加（#843、canvas 不使用の決定的 SVG path 方式による
同 §3.22 系の再導入。`qr_code` と同型の選択的 re-export）・ImageCropper
headless ラッパー追加（#844、同 §3.22 の意図的非採用の再導入、先例は
AngleSlider #842。crop 矩形（整数）のみの決定的状態機械。canvas 実切り出し・
pointer ドラッグ配線はスコープ外）を経て 98 の公開モジュール + `charts`
サブモジュール群を持つ（`charts::bar_chart`/`charts::bar_list`/
`charts::bar_segment`/`charts::scatter_chart`/`charts::radar_chart`/
`charts::axis`/`charts::grid`/`charts::legend`/`charts::tooltip`/
`charts::pie`/`charts::data`/`charts::scale`/`charts::svg` は既存の
`pub mod charts;`（#846）配下のサブモジュールであり、`grep -E '^pub mod '`
によるトップレベル公開モジュール集計には計上されない）。98 は
`grep -c '^pub mod ' crates/pre-styled-ui/src/lib.rs` の実測値であり、
本節の記述が長期間更新されず陳腐化していた経緯（イシュー #714）から旧
記載値（86）との単純な差分計算（増分本数の突き合わせ）は行わない。

内訳（元 §2 表と同じ行グルーピング、「由来イシュー」列を温存。API ページ
側は同じ行グルーピングで「部品ページ」列へ差し替えている）:

| 分類 | モジュール | 由来イシュー |
|---|---|---|
| 基盤 | `theme` | #547/#606 |
| 基盤 | `css` | #548 |
| 基盤 | `recipe` | #548/#606/#604（詳細は [`pre-styled-recipe-api.md`](../api/pre-styled-recipe-api.md)） |
| 基盤 | `stylesheet` | #605（CSS 集約・配布ヘルパ） |
| 単純 styled 部品 | `button` / `badge` / `spinner` / `alert` / `card` | #550/#606（`button` は #830 で `icon_button`/`close_button`（chakra `IconButton`/`CloseButton` 相当）を追加。独立部品ではなく `button` recipe の非公開 icon-only 修飾 variant として実装し、`data-scope="button"` を共有する） |
| 単純 styled 部品 | `skeleton` | #764（ローディングプレースホルダー） |
| 単純 styled 部品 | `image` | #770（写真等の静的コンテンツを表示する `<img>`） |
| 単純 styled 部品 | `icon` | #770（インライン SVG の寸法を統一する `<svg>` ラッパー） |
| 単純 styled 部品 | `separator` | #772（区切り線、`<hr>`） |
| 単純 styled 部品 | `highlight` | #775（テキスト中の一致語句を `<mark>` で強調する `<span>` + `<mark>`） |
| 単純 styled 部品 | `visually_hidden` | #776（視覚的には隠すが支援技術には読ませ続けるテキストコンテナ） |
| 単純 styled 部品 | `skip_nav` | #776（WCAG 2.1 SC 2.4.1 Bypass Blocks 対応の「本文へスキップ」リンク） |
| headless ラッパー第 1 弾 | `dialog` / `tabs` / `accordion` / `menu` / `select` | #551 |
| headless ラッパー第 2 弾 | `popover` / `tooltip` | #664 |
| headless ラッパー第 3 弾 | `switch` | #682 |
| headless ラッパー第 4 弾 | `radio_group` | #683 |
| headless ラッパー | `avatar` | #684 |
| headless ラッパー第 5 弾 | `checkbox` | #730 |
| 静的フォーム部品 | `input` / `textarea` / `native_select` | #737 |
| headless ラッパー第 6 弾 | `number_input` | #738 |
| headless ラッパー第 7 弾 | `pin_input` | #739 |
| headless ラッパー第 8 弾 | `password_input` | #740 |
| headless ラッパー第 9 弾 | `slider` | #741 |
| headless ラッパー第 10 弾 | `rating_group` | #742 |
| headless ラッパー | `segment_group` | #743 |
| headless ラッパー第 10 弾 | `tags_input` | #744 |
| headless ラッパー第 11 弾 | `editable` | #745 |
| headless ラッパー | `listbox` | #750 |
| headless ラッパー | `toggle` / `toggle_group` | #746 |
| カード型選択 UI（styled バリエーション） | `checkbox_card` / `radio_card` | #747 |
| headless ラッパー | `combobox` | #749 |
| headless ラッパー | `tree_view` | #753 |
| headless ラッパー（`tree_view` の派生） | `json_tree_view` | #829 |
| headless ラッパー | `pagination` | #751（headless-ui 側の保留解除は #716 → #751） |
| headless ラッパー | `steps` | #752（`docs/api/headless-ui-api.md` §4b.3 の Steps 保留解除） |
| headless ラッパー | `breadcrumb` | #755（`docs/api/headless-ui-api.md` §4b の追加候補消化） |
| headless ラッパー | `carousel` | #754 |
| headless ラッパー | `drawer` | #758 |
| headless ラッパー | `link` / `link_overlay` / `nav_list` | #756（`docs/api/headless-ui-api.md` §4b 追加候補・最優先候補の消化） |
| headless ラッパー | `action_bar` | #762 |
| headless ラッパー | `toast` | #760 |
| headless ラッパー | `hover_card` | #759 |
| headless ラッパー | `toggle_tip` | #761 |
| headless ラッパー | `progress` | #763 |
| 単純 styled 部品（静的） | `tag` / `kbd` / `code` | #768 |
| 状態機械を要しない静的部品 | `status` / `empty_state` | #765 |
| headless ラッパー | `clipboard` | #773 |
| タイポグラフィ静的部品 | `heading` / `text` / `em` / `mark` / `blockquote` / `list` | #771 |
| headless ラッパー | `qr_code` | #774 |
| headless ラッパー（Button recipe 流用） | `download_trigger` | #828 |
| 状態機械を持たない静的表示部品 | `table` / `data_list` | #767 |
| 静的部品（新規 anatomy） | `stat` / `timeline` | #769 |
| headless ラッパー | `floating_panel` | #827 |
| headless ラッパー | `scroll_area` | #825（`docs/design/component-coverage-map.md` 保留解除） |
| headless ラッパー | `splitter` | #826 |
| 単純 styled 部品 | `marquee` | #831 |
| headless ラッパー | `date_input` | #834（`docs/policy/intentional-non-adoption.md` §7・`docs/design/component-coverage-map.md` の date-time 系「保留」を DateInput 分のみ解除） |
| 単純 styled 部品（静的） | `color_swatch` | #838（`docs/design/component-coverage-map.md` 保留解除） |
| headless ラッパー（canvas 非依存） | `color_picker` | #839（親 #837、`docs/design/component-coverage-map.md` 保留解除。`docs/policy/intentional-non-adoption.md` §7 再評価トリガー「canvas 依存部分を隔離し状態機械を純粋関数に保つ設計」充足） |
| headless ラッパー | `file_upload` | #840（`docs/policy/intentional-non-adoption.md` §7 保留解除） |
| headless ラッパー | `calendar` | #835（親トラッキング #832） |
| headless ラッパー | `date_picker` | #835（親トラッキング #832） |
| headless ラッパー | `timer` | #836（`docs/design/component-coverage-map.md` 保留解除） |
| 静的部品（新規 anatomy、charts 基盤上層） | `charts::scatter_chart` / `charts::radar_chart` | #851（親 Phase #845） |
| headless ラッパー | `tour` | #841（`docs/design/component-coverage-map.md` 保留解除、#735） |
| 基盤（外部依存ゼロ SVG 生成） | `charts`（`data`/`scale`/`svg`） | #846（`docs/design/component-coverage-map.md` 保留解除。詳細は `docs/design/charts-foundation-design.md` 参照） |
| `charts` 基盤の消費者（新規 anatomy） | `line_chart` / `area_chart` / `sparkline` | #848 |
| charts（SVG） | `charts::bar_chart` | #849（親 Phase #845） |
| charts（HTML） | `charts::bar_list` | #849（親 Phase #845） |
| charts（HTML） | `charts::bar_segment` | #849（親 Phase #845） |
| 単純 styled 部品（新規 anatomy、charts 基盤の初のチャート部品） | `pie_chart` / `donut_chart` | #850 |
| headless ラッパー（非採用の再導入、先例は AngleSlider） | `angle_slider` | #842 |
| headless ラッパー（非採用の再導入） | `signature_pad` | #843 |
| headless ラッパー（非採用の再導入、先例は AngleSlider #842） | `image_cropper` | #844 |
| headless 由来ユーティリティ（本クレートに固有モジュールなし） | `format` / `Locale`（#853/#854） | クレートルート再エクスポート `pub use fandhe_frontend_headless_ui;` 経由 |
| headless ラッパー（glob 再エクスポート） | `collapsible` | #1682（親 #1670） |

`examples/headless-pre-styled-ui`（#552/#678/#698/#704）は本クレート
v0.4.0 へ統合済みである。旧来 headless-ui のセレクタへ手書きで当てていた
コンポーネント CSS は撤去され（イシュー #689）、`src/main.rs` の
`build_stylesheet()` が生成した CSS を `stylesheet::StyleSheet` で集約する
方式へ切り替え済み。

## 3. §3a headless 型再エクスポート契約の経緯（イシュー #685）

`fandhe-frontend-headless-ui` の 7 モジュール（`tabs`/`accordion`/`dialog`/
`menu`/`select`/`popover`/`tooltip`）を薄くラップする各 pre-styled-ui
モジュールは、本イシュー #685 当時は `pub use fandhe_frontend_headless_ui::<mod>::*;`
で同名モジュールを再エクスポートしていたが、この glob 再エクスポートは
**ラッパー呼び出しに必要な「モジュール外」の headless 型**（`state`/
`data_attrs` モジュール由来）までは届かない。PR #679 で
`fandhe-frontend-docs-site` が `fandhe-frontend-headless-ui` へ直接依存
せざるを得なかったのはこのためである（`Orientation`/`OpenState` を
pre-styled-ui のパスから import できなかった）。

**イシュー #729 以降の変更**: `tabs`/`accordion`/`dialog`/`menu`/`select`
の 5 モジュールは `size` variant クラス付与のため styled `root`（tabs のみ
`tabs`）を各モジュールで新設し、headless 自由関数 `root`（tabs は `tabs`/
`tabs_with_root_attrs`）との名前衝突を避けるため glob 再エクスポートから
選択的 re-export へ切り替えた。`popover`/`tooltip` は引き続き glob 再
エクスポートのまま。

この契約はイシュー #693 で実際に消化され、`fandhe-frontend-docs-site` は
headless-ui への直接依存（`Cargo.toml`・`structure.toml` 双方のエッジ）を
撤去して pre-styled-ui 単独依存へ移行済みである（`crates/docs-site/src/showcase.rs`
の import は本再エクスポート経由に切り替え済み）。

## 4. §3b interactive 層再エクスポート契約の背景・判断根拠（イシュー #712）

### 背景

§3a（イシュー #685）で確立した契約は SSR 描画（`Node` を組み立てて
`render()` する経路）を pre-styled-ui 単独依存で完結させるものだったが、
hydration / dispatch まで書く場合に必要な `fandhe-frontend-interactive` の
公開 API（`Component`/`Hydrate`/`dispatch`/`HydrateError`/
`render_for_hydration`/`HYDRATE_ATTR_PREFIX`/`codec` モジュール/
`DirtyTracked`）は対象外のままだった。実際に
`crates/pre-styled-ui/tests/headless_reexports.rs` は #685 時点で
`fandhe_frontend_interactive::{dispatch, Component}` を dev-dependency 経由で
直接 import しており、「SSR は単独依存で完結するが hydration/dispatch は
完結しない半端な状態」だった（PR #699/#695 の out-of-scope 節で検出）。

### 採用方針: interactive 層をクレート再エクスポートする（案 A）

`fandhe-frontend-headless-ui` に `pub use fandhe_frontend_interactive;`
（クレート再エクスポート）を追加し、`fandhe-frontend-pre-styled-ui` はそれを
推移的に `pub use fandhe_frontend_headless_ui::fandhe_frontend_interactive;`
で再エクスポートする。ルートへの個別型再エクスポート（`Component` 等を
ルート直下へ置く案）は行わない。

**根拠**:

1. **確立済み先例との一貫性**: core について headless-ui（#550）→「クレート
   そのものの再エクスポートで単独依存パスを完結させるエスケープハッチ」、
   pre-styled-ui（#685）→ 推移的再エクスポート、というパターンが既に確立
   している。interactive も同型で扱うのが最も予測可能（AI 保守前提の明示性・
   決定性・機械検証可能性）。
2. **トレイト同一性の保証**: 利用者が interactive を明示依存する現状維持案
   では、利用者側の `fandhe-frontend-interactive` のバージョン指定が
   headless-ui の内部依存とずれた場合、「別バージョンの `Component` を実装
   している」という初学者に解読困難なトレイト不一致エラーを踏み得る。
   再エクスポート経由ならクレート同一性が cargo の解決に依らず常に成立する
   （core 再エクスポートと同じ動機）。
3. **依存グラフ方針への影響ゼロ**: `docs/policy/dependency-graph-policy.md`
   の実測値は不変。`Cargo.toml` の依存エッジ追加は一切なく、
   `structure.toml` の `depends_on` も不変（fw gate 完全一致検証に影響しない）。
4. **不変条件の維持**: pre-styled-ui の「外部依存は
   `fandhe-frontend-headless-ui` のみ」（`crates/pre-styled-ui/Cargo.toml`
   コメント・§3 不変条件 4）を崩さずに実現できる唯一の再エクスポート経路
   である。
5. **ルート個別再エクスポートを見送る理由**: `dispatch` のような汎用名を UI
   クレートのルートへ置くと名前衝突・責務の混濁を招く。#685 でルートへ
   置いた `OpenState`/`Orientation` は docs-site の実利用パス（#693）という
   実績に基づくが、interactive 系項目には現時点で in-repo の実利用者が
   おらず、必要になれば非破壊的に追加できる。

**棄却案 B（現状維持 + 明示依存ガイド）**: 追加実装ゼロで済むが、(a) core と
interactive で「単独依存完結」の到達範囲が非対称になり契約が説明困難、
(b) 上記 2 のトレイト不一致リスクが残る、(c) §3a が掲げた「pre-styled-ui
のみに依存してラッパーを呼び出せる」保証が hydration を含む実用シナリオで
成立しない、ため棄却。

（棚卸し表・固定テスト所在・セキュリティ上の注意（REQ-1）は
[`docs/api/pre-styled-ui-api.md`](../api/pre-styled-ui-api.md) §3b に契約と
して残している。）

## 5. §4 設計方針の詳細

- **テーマトークン**（#547/#606）: 色・スペーシング等のデザイントークンと
  ダークモード切り替えの基盤。chakra-ui の `system`/`recipe` 相当の設計を
  参考にしつつ、静的 SSR 出力（ビルド時に確定する CSS）を前提とする。
  詳細は `theme` モジュール rustdoc を参照。
- **variant API・静的 CSS 生成**（#548/#606/#604）: chakra-ui の slot
  recipe 相当。コンポーネントの見た目バリエーション（size/variant/
  colorPalette 等）を型安全に選択し、対応する静的 CSS を生成する。詳細は
  [`pre-styled-recipe-api.md`](../api/pre-styled-recipe-api.md) を参照。
- **styled 部品**（#550/#551/#664/#682/#683/#684）: #550 は Button 等の
  単純な部品、#551 以降は headless-ui の Accordion/Dialog/Popover/
  Tooltip/Switch/RadioGroup/Avatar 等をラップした styled 版を提供する。

## 6. §4b avatar 是正の来歴（イシュー #684）

`Avatar` 状態機械はあえて再エクスポートしない（PR #695 Bugbot 指摘、イシュー
#684 是正）: `Avatar::root()` は headless 自由関数 `root` へそのまま
委譲するのみで `size`/`shape` variant クラスを一切付与しないため、
再エクスポートすると呼び出し側が styled 層のつもりで `Avatar::root()`
を呼びレイアウトが静かに崩れる事故を誘発する。`Avatar` による状態
管理・hydration が必要な呼び出し側は
`fandhe_frontend_headless_ui::avatar::Avatar` を直接 import すること
（この契約自体は API ページ §4b に残っている）。

## 7. §4c radio_group イシュー来歴

`radio_group` モジュールはイシュー #683（`size`/`palette` 拡張は #708）。
契約自体は API ページ §4c を参照。

## 8. §4d variant 表（元「状態」列全文）・`tabs_with_root_attrs` 新設経緯

（イシュー #708）単純部品（button/badge/spinner）・avatar に続き、headless
状態機械を持つ複合部品ラッパーへ `size`/`color-palette` variant を拡張する。

| 部品 | size | color-palette | 状態 |
|---|---|---|---|
| button/badge/spinner | ✓ | ✓ | 実装済み（#550/#606。button は #830 で icon-only 修飾 variant（`icon_button`/`close_button`）を追加。専用の `icon`/`close-button` 行は設けない: `data-scope="button"` を共有する variant 拡張であり別部品ではないため） |
| avatar | ✓ | – (shape) | 実装済み（#684） |
| switch | ✓ | ✓ | 実装済み（#708） |
| radio-group | ✓ | ✓ | 実装済み（#708） |
| checkbox | ✓ | ✓ | 実装済み（#730） |
| password-input | ✓ | ✓ | 実装済み（#740） |
| input / textarea / native-select | ✓ | – | 実装済み（#737） |
| tabs | ✓ | ✓（selected trigger の強調色） | 実装済み（#729） |
| accordion / dialog / menu / select | ✓ | – | 実装済み（#729） |
| number-input | ✓ | – | 実装済み（#738） |
| pin-input | ✓ | – | 実装済み（#739、palette は第 2 弾展開のフォローアップ） |
| rating-group | ✓ | ✓ | 実装済み（#742） |
| toggle | ✓ | ✓ | 実装済み（#746） |
| toggle-group | ✓ | ✓ | 実装済み（#746、root のみへクラス付与） |
| segment-group | ✓ | – | 実装済み（#743） |
| tags-input | ✓ | – | 実装済み（#744） |
| editable | ✓ | – | 実装済み（#745） |
| checkbox-card / radio-card | ✓ | ✓ | 実装済み（#747） |
| pagination | ✓ | ✓ | 実装済み（#751） |
| steps | ✓ | ✓ | 実装済み（#752） |
| popover / tooltip | 提供しない | 提供しない | 方針確定 |
| tree-view | 提供しない | 提供しない | 実装済み（#753） |
| json-tree-view | 提供しない | 提供しない | 実装済み（#829） |
| toggle-tip | 提供しない | 提供しない | 実装済み（#761） |
| breadcrumb | ✓ | – | 実装済み（#755） |
| drawer | ✓ | – | 実装済み（#758） |
| link | 提供しない | 提供しない | 実装済み（#756） |
| link-overlay / nav-list | 提供しない | 提供しない | 実装済み（#756） |
| table | ✓ | 提供しない | 実装済み（#767） |
| data-list | 提供しない | 提供しない | 実装済み（#767） |
| toast | ✓（`placement`、`group` slot） | ✓（`status`、`root` slot） | 実装済み（#760） |
| tour | 提供しない | ✓（`root` slot） | 実装済み（#841） |
| file-upload | ✓ | – | 実装済み（#840） |

tabs/accordion/dialog/menu/select の実装詳細（イシュー #729）: tabs は他 4
部品と異なり headless 側に root への attrs 注入点自体が存在しなかったため、
追加的（非破壊）な `fandhe_frontend_headless_ui::tabs::tabs_with_root_attrs`
を新設した（`crates/headless-ui/src/tabs.rs` rustdoc 参照。既存 `tabs()`
はこれへ `root_attrs: vec![]` で委譲する薄いラッパーのまま。headless-ui は
非破壊追加のためパッチバンプ）。`Dialog`/`Menu`/`Select`（inherent `root()`
を持つ状態機械型）は `switch::Switch`（#708/#719）と同じ理由で
再エクスポートから除外し（未スタイル root の静かな適用漏れ防止）、選択的
re-export へ切り替えた。

## 9. §4d data-focus-visible 実装時系列・wasm-full 配線経緯（イシュー #709）

- `checkbox` は headless 層の契約（`data_focus_visible`）が確立済みであり、
  イシュー #709 時点では styled ラッパー未実装のため CSS 側の recipe 追加を
  対象外としていたが、#730 で `switch` の `control` と同型の
  `StateCondition::Attr("data-focus-visible")` 規則を実装済み。
- `fandhe-frontend-wasm-full` の focus 配線（`focus_visible` モジュール、
  `keynav`/`events` と同じ 2 層構成）が hidden-input の focusin/focusout
  と `:focus-visible` 判定に基づき、境界パーツ（switch: `root`、
  radio_group: `item`）とその配下で同一 `data-scope` を共有するパーツ
  （switch: `control`、radio_group: `item-control`）の双方へ付け外しする。

## 10. §4e checkbox イシュー来歴（イシュー #730）

`checkbox` モジュールは `fandhe_frontend_headless_ui::checkbox`（イシュー
#535/#595）の 5 anatomy パーツを再エクスポートする。`data-focus-visible`
の属性の付け外しは headless/wasm 層の責務であり、
`fandhe-frontend-wasm-full` の focus 配線に `("checkbox", "hidden-input")
=> Some("root")` のマッピングが #709 時点で登録済みのため、本イシュー
（#730）での wasm 層変更は不要だった。

## 11. §4f input/textarea/native_select スコープ外（イシュー #737）

- `native_select` の indicator パーツ（カスタム矢印）は本イシューのスコープ
  外（フォローアップ）。

## 12. §4g checkbox_card/radio_card スコープ外・chakra-ui/ark-ui 由来判断（イシュー #747）

chakra-ui の `forms/checkbox-card.md`/`forms/radio-card.md` 相当。ark-ui には
対応する headless anatomy が存在しない（chakra-ui 独自の slot recipe）ため、
**`fandhe-frontend-headless-ui` には手を入れず**、pre-styled-ui 層のみで
新規 anatomy を定義した。

**本イシューのスコープ外**（`.claude/rules/out-of-scope-tracking.md` 対応）:

- `fandhe-frontend-wasm-full` の focus/クリック配線（`(scope, part)` を
  `("checkbox-card", "hidden-input") -> "root"`/
  `("radio-card", "item-hidden-input") -> "item"` へ写像し
  `data-focus-visible` を CSS で伝える対応、headless 配線の select
  アクション写像の card scope 対応）。
- `examples/headless-pre-styled-ui` への追随（pre-styled-ui 公開後に
  別 PR で対応）。

## 13. §4h status/empty_state テストファイル列挙・スコープ外（イシュー #765）

- **XSS 回帰**: `tests/xss_escape_styled.rs` に両部品の root children・
  呼び出し側 attrs・`class` 属性・パーツ children の各経路を追加。
- **golden CSS**: `tests/status_empty_state_css.rs` が両部品の `css()` 全文を
  バイト単位で固定する（`toggle_tip_css.rs` の複数部品 1 ファイル前例に
  倣う）。
- **スコープ外**（`.claude/rules/out-of-scope-tracking.md` 対応）:
  `examples/headless-pre-styled-ui` への掲示は crates.io 公開後の追随
  イシューとして扱う（`checkbox_card`/`radio_card` と同型の運用）。

## 14. §4i タイポグラフィ: chakra-ui からの縮約・prose との役割分担（イシュー #771）

### chakra-ui からの縮約（対象外事項）

- Heading の視覚サイズは chakra-ui の `xs`〜`7xl`（9 段階）に対し、
  `crates/pre-styled-ui/src/theme.rs` のテーマトークンが
  `font-size-xs`〜`font-size-4xl`（8 段階）までしか持たないため `sm`〜`4xl`
  （7 段階）へ縮約した。
- `bgGradient` 等の chakra style props、`List.Indicator` のアイコン同梱、
  `Blockquote.Icon` は、本クレートが style props を非採用としている既存
  設計判断（テーマトークン + variant enum のみ）に合わせて非採用。

### prose（記事全体カスケード）との役割分担

chakra-ui の `Prose`（記事全体へ一括カスケード適用するコンポーネント）に
相当する機構は、本クレートへは導入しない。本節の 6 部品はいずれも
「要素単位のオプトイン適用」であり、Markdown 由来の記事本文へ無選別に
カスケード適用する仕組みは持たない。記事全体へのカスケードスタイルは
`fandhe-frontend-docs-site` のサイト骨格 CSS（`crate::site_theme` による
ビルド時生成、出力先 `assets/site.css`。`.docs-content`
配下の `h1`-`h3`/`p`/`ul`/`ol`/`blockquote` 規則）が既に担っており、本
イシューはこの既存機構を置き換えない（詳細な判断根拠は
`crates/pre-styled-ui/src/text.rs` rustdoc、対応表は
`docs/design/component-coverage-map.md` prose.md 行を参照）。

## 15. §4j charts 軸ほか: chakra-ui からの縮約（イシュー #847）

- `tickFormatter`（任意クロージャ）は `TickLabelFormat`（固定 `prefix`/
  `suffix` の 2 フィールド）へ縮約した。ロケール依存の日付フォーマット等は
  非対応。
- インタラクティブ legend（hover で対象系列を強調・click で表示トグル）は
  JS/wasm ランタイム連携が必要なためスコープ外。
- マウス追従型のリッチツールチップは JS 必須のためスコープ外。
- `yAxisId` による二軸チャートは非対応。

## 16. §4k line/area/sparkline: chakra-ui からの縮約（イシュー #848）

- 軸・グリッド・凡例・ツールチップ（`CartesianGrid`/`XAxis`/`YAxis`/
  `ChartLegend`/`ChartTooltip`）は #847 以降。呼び出し側が `svg_root` の
  children として本 3 部品の出力と #847 の軸要素を並べる統合を想定する。
- 積み上げ（`stackId`）・曲線補間（`curveType`）は #847 以降。
- `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途行う
  （`qr_code` の先例と同じ判断）。

## 17. §3c 再エクスポート形式規約の来歴とレビュー結果（イシュー #1062）

### 背景

`crates/pre-styled-ui/` の各 styled ラッパーモジュールが headless の対応
モジュールを再エクスポートする形式は、glob（`pub use
fandhe_frontend_headless_ui::<mod>::*;`）・選択的個別・（styled 側の
ローカル定義による）実質的な shadowing の 3 パターンに分かれており、
使い分け基準が各モジュールの rustdoc に散在していた。親トラッキング #1057
（headless / pre-styled の責務分離整備）配下で、新規 styled 部品追加時の
判断コストと fail-open リスク（#684 の `Avatar` 状態機械 inherent `root()`
未スタイル適用漏れと同型の事故）を抑えるため、判定規約を
`crates/pre-styled-ui/src/lib.rs` へ明文化した（規約 A〜D、§3c 参照）。

### glob 14 箇所のレビュー結果（結論: 全件「維持」。#14 はイシュー #1682 で新設）

判定基準は規約 B の 4 条件。是正内容は「規約への追認 + マーカーコメント
（`REEXPORT-GLOB-REVIEWED:`）追記 + 記録」であり、選択的 re-export への
変更（＝公開 API からの項目削除）は 1 件も行っていない。

| # | モジュール | B-1（`stylesheet()` のみ） | B-2（variant なし） | B-3（属性セレクタのみ） | 判定 |
|---|---|---|---|---|---|
| 1 | `action_bar` | 充足 | 充足 | 充足 | 維持 |
| 2 | `popover` | 充足 | 充足（#708 方針 3 で確定） | 充足 | 維持 |
| 3 | `hover_card` | 充足 | 充足 | 充足 | 維持 |
| 4 | `floating_panel` | 充足 | 充足 | 充足 | 維持 |
| 5 | `navigation_menu` | 充足 | 充足 | 充足 | 維持 |
| 6 | `scroll_area` | 充足 | 充足（#825 で variant 非採用を明記） | 充足 | 維持 |
| 7 | `tree_view` | 充足（イシュー #1578 以前） | 充足（イシュー #1578 以前） | 充足（イシュー #1578 以前） | イシュー #1578 で `size` variant を新設し規約 A（選択的 re-export）へ移行済み。本表は #1062 時点の記録として保持する |
| 8 | `toggle_tip` | 充足 | 充足 | 充足 | 維持 |
| 9 | `tooltip` | 充足 | 充足（#708 方針 3 で確定） | 充足 | 維持 |
| 10 | `timer` | 充足 | 充足（モジュール rustdoc でスコープ外明記） | 充足 | 維持 |
| 11 | `json_tree_view` | 充足 | 充足 | 充足 | 維持 |
| 12 | `toolbar` | 充足 | 充足 | 充足 | 維持 |
| 13 | `menubar` | 充足 | 充足 | 充足 | 維持 |
| 14 | `collapsible` | 充足 | 充足 | 充足 | 新設（#1682） |

### 「意図的 shadowing」の実態確認

イシュー本文の「意図的 shadowing」は 2 通りに読めるため、両方を実地確認
した。

- **(a) glob 由来の名前を同名の明示 `pub use` が上書きする Rust の
  shadowing**: 上表 13 モジュール（#1062 時点の記録。`tree_view` はイシュー
  #1578 で規約 A（選択的 re-export）へ移行済みだが、state 由来の型は移行後も
  変わらず明示再エクスポートを維持しているため、本節の実態確認は現在も
  13 モジュールぶん成立する）はいずれも `pub use
  fandhe_frontend_headless_ui::state::{OpenState, …}` 等を glob と併記する
  が、対応する headless モジュール側は `use crate::state::{…}`（非公開
  import）であり glob 経路に載らない。headless 側で `pub use` を持つのは
  `toolbar.rs`（`ToggleGroup`/`MultiToggleGroup`）と `json_tree_view.rs`
  （`TreeView`）の 2 件のみで、pre-styled 側の明示再エクスポート名
  （`Orientation`/`TreeViewAction`/`state` 系）とは一切衝突しない。→
  現時点で (a) の実例は 0 件（規約 C は将来の混入防止として置く）。
- **(b) styled 側のローカル定義が headless の同名項目を置き換えるパターン**:
  `avatar`/`breadcrumb`/`nav_list`/`clipboard`/`toggle`/`segment_group`/
  `checkbox_group`/`file_upload`/`date_input`/`angle_slider` 他多数で
  「styled `root` を再定義し、headless の同名自由関数・状態機械型はあえて
  再エクスポートしない」形が確立済み。→ これが実質的な「意図的
  shadowing」の運用実体であり、選択的 re-export（規約 A）によって Rust
  レベルの暗黙 shadowing を回避している構図。

### 兄弟イシューとの責務境界

- **#1064**（`crates/docs-site/tests/` に headless 63 部品 / pre-styled
  107 部品のラップ状態を機械可視化する契約テストを追加）と役割を分離する。
  本イシュー（#1062）は `crates/pre-styled-ui/tests/` 内で再エクスポート
  **形式**の規約適合のみを検査し、部品の対応関係（ラップ済み /
  pre-styled-only / 未ラップ）には立ち入らない。
- `crates/headless-ui/` 側の再エクスポート方針（`positioning.rs` の層帰属
  含む）は #1065 のスコープであり本件では変更しない。

### semver 判定

本 PR の変更は `src/lib.rs` の rustdoc 追加・各 glob モジュールへの
`REEXPORT-GLOB-REVIEWED:` コメント追加・`tests/reexport_policy.rs` 新設の
みで、公開 API（型・関数・再エクスポート項目）は一切変化しない。PR 本文へ
`version-bump-exempt: fandhe-frontend-pre-styled-ui`（理由: rustdoc・
コメント・テスト追加のみで公開 API は不変）を宣言し、`version-bump-guard`
の対象から除外した。

## golden テスト（バイト一致）の更新手順（イシュー #1427）

`crates/pre-styled-ui/tests/*_css.rs` の golden（バイト一致）テストを更新
する際の手順・部品対応表・禁止事項は
`docs/internal/pre-styled-ui-golden-test-update-guide.md` に切り出した。
Phase 1 以降の各部品スタイル調整 PR ではこちらを参照する。
