#![forbid(unsafe_code)]

//! `fandhe-frontend-wireframe-ui`: blocks.pm 参照のローファイ・モノクロ
//! ワイヤーフレーム UI コンポーネント層。
//!
//! # 役割
//!
//! 画面設計初期段階での配置イメージ提示に特化した、非インタラクティブな
//! 表示専用プレースホルダー部品（計 49 部品、blocks.pm 由来 35 + 追加 14）
//! を提供する。詳細は `docs/design/wireframe-ui-architecture.md` を参照。
//!
//! # 責務境界
//!
//! - `fandhe-frontend-headless-ui`（Primitives）・`fandhe-frontend-pre-styled-ui`
//!   （Themes）とは独立した第 3 の UI コンポーネント層であり、両者への
//!   依存・両者からの依存のいずれも持たない
//! - 依存は `fandhe-frontend-core` のみ（外部依存ゼロ）
//! - **SSR 専用**。`wasm-full` 配線（ハイドレーション・インタラクション）
//!   は行わない。REQ-11 の gzip 計測（wasm-full/dist-server 経路）には
//!   非影響
//! - 全部品は非インタラクティブな表示専用プレースホルダーとする。対話的な
//!   WAI-ARIA セマンティクス（`role`/`aria-expanded`/`aria-haspopup` 等）・
//!   `tabindex`・キーボードイベントハンドラ・フォーカス管理・状態遷移は
//!   一切実装しない。ネイティブに対話セマンティクスを持つ HTML 要素
//!   （`button`/`input`/`select`/`a[href]` 等）も出力しない
//!
//! # 不変条件
//!
//! 1. `#![forbid(unsafe_code)]`（REQ-2）
//! 2. `[dependencies]` は `fandhe-frontend-core` のみ（REQ-3）
//! 3. テキスト引数は `fandhe-frontend-core` の既定エスケープ経由（REQ-1）
//!
//! # 現状
//!
//! 共通基盤 API 実装済み（イシュー #2605）: [`Size`]・[`Bold`]/[`Primary`]/
//! [`Active`]/[`Disabled`]/[`Orientation`]（共通型）・モノクロトークン
//! （[`tokens`]）・[`wireframe_css`]（CSS 集約出力）・[`class_list`]。
//! SVG アイコン基盤（[`icon`](mod@icon)、イシュー #2606）実装済み: 12 種以上の
//! ラインアートアイコンと `Node` スロット規約（`docs/design/wireframe-ui-architecture.md`
//! §11）。個別部品は Phase 2「テキスト・注釈」の [`annotation`]
//! （イシュー #2617）から実装を開始し、Phase 1「レイアウト骨格」の
//! [`grid`]（イシュー #2611）・[`divider`]（イシュー #2612、
//! `props::Orientation` の最初の消費者）・[`stack`]（イシュー #2610）・
//! Phase 2 の [`rich_text`]（イシュー #2616）・[`link`]（イシュー #2618、
//! `Option<Node>` アイコンスロット規約 §11.4 の実例、`a[href]` 非出力）・
//! [`paragraph`]（イシュー #2615）、Phase 3「Forms A」の [`button`]
//! （イシュー #2621、`props::Disabled` の最初の実消費者）・[`select`]
//! （イシュー #2624、`props::Active` と `props::Disabled` を `.attr()` で
//! 併用する初の部品）・[`switch`]（イシュー #2627、`Active` を ON 状態の
//! 意味で使い `Disabled` を併用する 2 例目）・[`checkbox`]（イシュー
//! #2625、`props::Active` を「チェック済み」状態として消費する 3 例目）・
//! [`radio`]（イシュー #2626、選択状態は新型を新設せず `props::Active` を
//! 再利用する 4 例目）・[`textarea`]（イシュー #2623、`rows` を行
//! プレースホルダー要素の構造表現とし `style`・ネイティブ `<textarea>` を
//! 使わない設計の 5 例目）・[`slider`]（イシュー #2628、`props::Orientation`
//! と `Active`/`Disabled` を併用する 6 例目。進捗値は 5 刻みの固定 class
//! 集合へ量子化する）・Phase 1 の [`frame`]（イシュー #2609）・Phase 2 の
//! [`tag`]（イシュー #2619）・[`input`]（イシュー #2622、`props::Active`/
//! `props::Disabled` の `.attr()` を Select・Switch・Checkbox・Radio・
//! Textarea・Slider に続いて併用する 7 例目の実消費者）・Phase 4「Forms B」の
//! [`question`]（イシュー #2630、ラベル + 補足説明 + `Node` スロットの
//! コントロール + ヒント。表示状態軸を持たずスロット側へ委ねる）・
//! [`ratings`]（イシュー #2631、`icon::star` を再利用し塗り数を
//! `data-active` で表現する）・[`calendar`]（イシュー #2632、選択日は
//! `props::Active` を再利用し `MAX_WEEKS` で 6 週へ飽和させる）・
//! [`file_drop`]（イシュー #2633、blocks.pm に対応部品がない独自追加部品。
//! アイコンは `link` と同じ `Option<Node>` スロット、表示状態軸を持たない）・
//! Phase 2 の [`text`]（イシュー #2614、単一行テキスト。`<span>` ルート +
//! `white-space: nowrap` + `text-overflow: ellipsis` で 1 行固定表示する。
//! [`paragraph`] の複数行許容とは対になる判断）が続いた。これで
//! Phase 2「テキスト・注釈」は全部品が出揃った。Phase 5「Navigation」の
//! [`tabs`]（イシュー #2638、選択状態は項目ごとの `Active` ではなく
//! `active: Option<usize>` 1 引数で表し、選択中は高々 1 件という不変条件を
//! 型で保証する）・Phase 4「Forms B」の [`stepper`]（イシュー #2634、
//! blocks.pm 対応部品を持たない独自追加部品。完了ステップの状態は
//! `props.rs` へ新型を追加せず部品ローカルの `data-complete` とし、現在
//! ステップは既存 [`props::Active`] を再利用する）・Phase 5「Navigation」の
//! [`nav_item`]（イシュー #2636、先頭・末尾の `Option<Node>` アイコン
//! スロットに加え `Option<&str>` の件数カウンター内部パートを持ち、
//! `Active` はアクティブ状態のグレースケール反転配色として消費する）・
//! [`accordion`]（イシュー #2641、blocks.pm に対応部品がない独自追加部品。
//! 展開状態は新型を新設せず `props::Active` を項目単位で再利用し、
//! 折りたたみ項目の本文スロットは出力しない）・[`pagination`]
//! （イシュー #2640、Phase 5 の 4 番目の部品。ページ項目は
//! `&[Option<&str>]`（`None` がギャップ）で表し、選択状態は
//! `tabs`/`radio` と同じく既存の `props::Active` を再利用する。先頭/前/次/
//! 末尾コントロールは `prev_next`/`first_last` の 2 bool へ畳む）・
//! [`cursor`]（イシュー #2642、Phase 5 の 5 番目の部品。代わりに使える
//! 既存アイコンがないため `icon::cursor_arrow`/`icon::cursor_hand` を
//! 新規追加して消費する。部品ローカルの列挙型 [`CursorKind`] を
//! クレートルートから再エクスポートする初めての例）・
//! [`menu::menu`]（イシュー #2637、Phase 5 の 6 番目の部品。検索欄は
//! `Option<&str>` + 固定パートの [`icon::search`] で表し `<input>` は
//! 出力しない。項目は [`menu::MenuItem`] のスライスで受け、強調状態は
//! 無効項目を指す添字なら優先して外す fail-closed な
//! `active: Option<usize>`）・
//! [`breadcrumbs`]（イシュー #2639、Phase 5 の 7 番目の部品。`tabs` の
//! `Option<usize>` とは異なり選択引数を持たず、`items` が空でない限り
//! 常に最後の項目へ `props::Active` を付与する。区切りは `stepper` と
//! 同じく CSS 擬似要素のみで描く）も続いた。
//! Phase 6「Overlay・Feedback」の最初の部品 [`tooltip`]（イシュー #2644、
//! 方向は部品ローカルの [`tooltip::TooltipSide`] による修飾 class で表す）・
//! Phase 6 の 2 番目の部品 [`toast`]（イシュー #2647、閉じるグリフは
//! [`icon::x`] 固定で instance swap にせず `dismissible: bool` の 1 引数
//! だけで有無を切り替える）・3 番目の部品 [`alert`]（イシュー #2646、
//! 横長の警告バナー。重要度は部品ローカルの [`alert::Severity`] による
//! 修飾 class で表し、`props.rs` へは昇格しない。アイコンは
//! `link`/`file_drop` と同じ `Option<Node>` スロット）・4 番目の部品
//! [`progress`]（イシュー #2648、形状は部品ローカルの
//! [`progress::ProgressShape`] による修飾 class（Bar/Circle）で表し、進捗値は
//! `slider` と同型の 5 刻み固定 class 集合へ量子化する。表示専用のため
//! `Active`/`Disabled` を持たない）・5 番目の部品 [`spinner`]（イシュー
//! #2649、円弧だけを描く静的表示で `@keyframes`/`animation` は持たない）・
//! 6 番目の部品 [`modal`]（イシュー #2645、blocks.pm に対応部品がない
//! 独自追加部品。中央配置は `position: fixed` ではなく in-flow の背景領域 +
//! `place-items: center` で表現し、パネル最大幅は `Size` 5 段の静的ルール
//! として [`crate::css::PARTS`] へ直書きする）が続いた。Phase 7「Data
//! display」の最初の部品 [`avatar`]（イシュー #2651、`content: Option<Node>`
//! が `None` のとき [`icon::user`] へフォールバックする §11.4 からの意図的な
//! 逸脱。円形表示は `crate::frame` の `bordered` と同型の部品固有修飾 class
//! で表す）・2 番目の部品 [`counter`]（イシュー #2655、件数を収めた
//! ピルバッジ。件数は `u32` ではなく `&str` で受け、強調配色は部品
//! ローカルの新型を新設せず共通型 [`props::Primary`] を再利用する。
//! `crate::nav_item` の内部カウンターパートとは独立した部品）・3 番目の
//! 部品 [`emoji`]（イシュー #2654、絵文字は `Option<Node>` アイコン
//! スロットではなく `glyph: &str` の 1 引数へ畳み込む §11.4 からの意図的な
//! 逸脱。空文字列は CSS の `:empty` 規則で破線の円プレースホルダーに
//! する）・4 番目の部品 [`stat`]（イシュー #2656、blocks.pm に対応部品が
//! ない独自追加部品。増減インジケータは `Option<&str>` ではなく
//! [`stat::StatDelta`]（`menu::MenuItem` と同型の公開構造体）で表し、
//! 向きのある `Up`/`Down` は [`icon::caret_up`]/[`icon::caret_down`] を
//! 再利用する）・5 番目の部品 [`card_basic`]（イシュー #2658、先頭・末尾
//! スロットは §11.4 の `Option<Node>` 規約へ統一し `avatar` を内蔵しない
//! 独自設計。`secondary` は [`nav_item`] の `counter` と同じ
//! `Option<&str>` で表す）・6 番目の部品 [`list`]（イシュー #2657、
//! 箇条書き/番号付きリストの配置イメージ。`items: Vec<Node>` を項目
//! ラッパー class で包み、`ordered: bool` は部品固有の修飾 class、
//! マーカー・番号は CSS 擬似要素/カウンタのみで描く。`<ul>`/`<ol>`/`<li>`
//! は出力しない）が続いた。
//! これで Phase 5「Navigation」（tabs/nav_item/accordion/pagination/cursor/
//! menu/breadcrumbs の 7 部品）・Phase 6「Overlay・Feedback」
//! （tooltip/toast/alert/progress/spinner/modal の 6 部品）はいずれも
//! 全部品が出揃った。Phase 8「Media・データ表示」の最初の部品 [`chart`]
//! （イシュー #2663、棒グラフの配置イメージ。値は `u8` 列
//! `values: &[u8]` として受け取り、[`progress`] と同型の 5 刻み量子化・
//! [`props::Orientation`] の再利用（4 例目の消費者）・
//! [`grid::MAX_COLUMNS`] と同じ資源有界化（[`chart::MAX_BARS`]）で
//! 組み立てる。折れ線・面・円・散布・凡例・軸ラベル・複数系列はスコープ
//! 外とする）・2 番目の部品 [`image`]（イシュー #2660、
//! `content: Option<Node>` が `None` のときバツ印プレースホルダーを描き、
//! `Some(node)` のときは子要素を差し替える §11.4 準拠の実例。強調は
//! 共通型 [`props::Primary`] を再利用し、バツ印の色は CSS カスタム
//! プロパティ `--fw-wire-image-x-color` の上書きで反転させる）・
//! 3 番目の部品 [`map`]（イシュー #2664、地図タイルの配置イメージ。
//! ズームは部品ローカル列挙型 [`map::MapZoom`] 3 段、マーカーは
//! [`link`]/[`file_drop`]/[`alert`] と同型の `Option<Node>` アイコン
//! スロット。街路・区画・道路の位置はすべて CSS の固定ルールで描き、
//! `&str` 引数を持たない）・4 番目の部品 [`media`]（イシュー #2661、
//! blocks.pm 上の表示名は Placeholder。`content: Option<Node>` が
//! `None` のとき [`icon::play`] へフォールバックする §11.4 からの意図的
//! な逸脱。動画か静止画かは bool ではなくスロット差し替えで表し、枠は
//! 16:9 固定で `<video>`/`<iframe>` は出力しない）・5 番目の部品
//! [`table`]（イシュー #2662、N 列 × M 行のデータ表プレースホルダー。
//! [`calendar`] と同型の判断で `<table>` を使わず `div`/`span` + CSS
//! grid で表現し、列数は `headers`/`rows` の形から導く）が続き、
//! Phase 8「Media・データ表示」（chart/image/map/media/table の 5 部品）
//! も全部品が出揃った。Phase 7「Data display」の 7 番目の部品
//! [`icon()`](fn@icon)（イシュー #2652、アイコン単体を示す部品。
//! `Node` ではなく `fn(Size) -> Node`（[`icon::IconEntry`] の要素型と
//! 同じ関数ポインタ）をコンストラクタ引数として受け取り、サイズ指定を
//! 1 か所に固定する。`role`/`aria-label` は付けない。[`icon`](mod@icon)
//! モジュールへの追記として実装した）が続いた。残りは Phase 7 の
//! `brand`（8 番目）のみとなった。
//!
//! # class 命名規約
//!
//! 全 class は [`CLASS_PREFIX`]（`fw-wire-`）で始まる。部品ルートは
//! `fw-wire-<kebab>`、部品内パートは `fw-wire-<kebab>-<part>`、共通修飾
//! （部品名を含まない横断 class）は `fw-wire-size-<xs|sm|md|lg|xl>` /
//! `fw-wire-bold` / `fw-wire-primary` / `fw-wire-horizontal|vertical`。
//! 表示状態は class ではなく `data-active`/`data-disabled` で表す。CSS
//! カスタムプロパティは `--fw-wire-*`（pre-styled-ui の `--fandhe-*` とは
//! 意図的に別プレフィックス）。`fw-wire-icon-glyph`（[`icon`](mod@icon) モジュールの
//! 各グリフ関数が返す `<svg>` の class）は、単独使用（他部品の `Node`
//! スロットへ直接渡す場合）と [`icon()`](fn@icon) 部品（イシュー #2652）
//! のパート class としての使用の両方を持つ（`icon()` のルート
//! `fw-wire-icon` の子要素として現れる）。
//! 詳細・追記契約は `docs/design/wireframe-ui-architecture.md` §10 を参照。

pub mod accordion;
pub mod alert;
pub mod annotation;
pub mod avatar;
pub mod breadcrumbs;
pub mod button;
pub mod calendar;
pub mod card_basic;
pub mod chart;
pub mod checkbox;
pub mod class;
pub mod counter;
pub mod css;
pub mod cursor;
pub mod divider;
pub mod emoji;
pub mod file_drop;
pub mod frame;
pub mod grid;
pub mod icon;
pub mod image;
pub mod input;
pub mod link;
pub mod list;
pub mod map;
pub mod media;
pub mod menu;
pub mod modal;
pub mod nav_item;
pub mod pagination;
pub mod paragraph;
pub mod progress;
pub mod props;
pub mod question;
pub mod radio;
pub mod ratings;
pub mod rich_text;
pub mod select;
pub mod size;
pub mod slider;
pub mod spinner;
pub mod stack;
pub mod stat;
pub mod stepper;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod tag;
pub mod text;
pub mod textarea;
pub mod toast;
pub mod tokens;
pub mod tooltip;

pub use accordion::accordion;
pub use alert::{alert, Severity};
pub use annotation::annotation;
pub use avatar::avatar;
pub use breadcrumbs::breadcrumbs;
pub use button::button;
pub use calendar::{calendar, MAX_WEEKS};
pub use card_basic::card_basic;
pub use chart::{chart, MAX_BARS};
pub use checkbox::checkbox;
pub use class::{class_list, CLASS_PREFIX};
pub use counter::counter;
pub use css::{wireframe_css, PARTS};
pub use cursor::{cursor, CursorKind};
pub use divider::divider;
pub use emoji::emoji;
pub use file_drop::file_drop;
pub use frame::frame;
pub use grid::{grid, MAX_COLUMNS};
pub use icon::icon;
pub use image::image;
pub use input::input;
pub use link::link;
pub use list::list;
pub use map::{map, MapZoom};
pub use media::media;
pub use menu::{menu, MenuItem};
pub use modal::modal;
pub use nav_item::nav_item;
pub use pagination::pagination;
pub use paragraph::paragraph;
pub use progress::{progress, ProgressShape};
pub use props::{Active, Bold, Disabled, Orientation, Primary};
pub use question::question;
pub use radio::radio;
pub use ratings::{ratings, STAR_COUNT};
pub use rich_text::rich_text;
pub use select::select;
pub use size::Size;
pub use slider::slider;
pub use spinner::spinner;
pub use stack::stack;
pub use stat::{stat, StatDelta, StatTrend};
pub use stepper::stepper;
pub use switch::switch;
pub use table::{table, MAX_TABLE_COLUMNS, MAX_TABLE_ROWS};
pub use tabs::tabs;
pub use tag::tag;
pub use text::text;
pub use textarea::{textarea, MAX_ROWS};
pub use toast::toast;
pub use tooltip::{tooltip, TooltipSide};
