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
//! SVG アイコン基盤（[`icon`]、イシュー #2606）実装済み: 12 種以上の
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
//! クレートルートから再エクスポートする初めての例）も続いた。
//! Phase 6「Overlay・Feedback」の最初の部品 [`tooltip`]（イシュー #2644、
//! 方向は部品ローカルの [`tooltip::TooltipSide`] による修飾 class で表す）・
//! Phase 6 の 2 番目の部品 [`toast`]（イシュー #2647、閉じるグリフは
//! [`icon::x`] 固定で instance swap にせず `dismissible: bool` の 1 引数
//! だけで有無を切り替える）・3 番目の部品 [`progress`]（イシュー #2648、
//! 形状は部品ローカルの [`progress::ProgressShape`] による修飾 class
//! （Bar/Circle）で表し、進捗値は `slider` と同型の 5 刻み固定 class 集合へ
//! 量子化する。表示専用のため `Active`/`Disabled` を持たない）が続いた。
//! 残りは Phase 3 の他部品（#2608〜）・Phase 5 の他部品（menu 等）・
//! Phase 6 の他部品で順次追加する。
//!
//! # class 命名規約
//!
//! 全 class は [`CLASS_PREFIX`]（`fw-wire-`）で始まる。部品ルートは
//! `fw-wire-<kebab>`、部品内パートは `fw-wire-<kebab>-<part>`、共通修飾
//! （部品名を含まない横断 class）は `fw-wire-size-<xs|sm|md|lg|xl>` /
//! `fw-wire-bold` / `fw-wire-primary` / `fw-wire-horizontal|vertical`。
//! 表示状態は class ではなく `data-active`/`data-disabled` で表す。CSS
//! カスタムプロパティは `--fw-wire-*`（pre-styled-ui の `--fandhe-*` とは
//! 意図的に別プレフィックス）。部品ルートなしで単独使用する唯一の例外的
//! パート class として `fw-wire-icon-glyph`（[`icon`] のグリフ）を持つ。
//! 詳細・追記契約は `docs/design/wireframe-ui-architecture.md` §10 を参照。

pub mod accordion;
pub mod annotation;
pub mod button;
pub mod calendar;
pub mod checkbox;
pub mod class;
pub mod css;
pub mod cursor;
pub mod divider;
pub mod file_drop;
pub mod frame;
pub mod grid;
pub mod icon;
pub mod input;
pub mod link;
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
pub mod stack;
pub mod stepper;
pub mod switch;
pub mod tabs;
pub mod tag;
pub mod text;
pub mod textarea;
pub mod toast;
pub mod tokens;
pub mod tooltip;

pub use accordion::accordion;
pub use annotation::annotation;
pub use button::button;
pub use calendar::{calendar, MAX_WEEKS};
pub use checkbox::checkbox;
pub use class::{class_list, CLASS_PREFIX};
pub use css::{wireframe_css, PARTS};
pub use cursor::{cursor, CursorKind};
pub use divider::divider;
pub use file_drop::file_drop;
pub use frame::frame;
pub use grid::{grid, MAX_COLUMNS};
pub use input::input;
pub use link::link;
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
pub use stack::stack;
pub use stepper::stepper;
pub use switch::switch;
pub use tabs::tabs;
pub use tag::tag;
pub use text::text;
pub use textarea::{textarea, MAX_ROWS};
pub use toast::toast;
pub use tooltip::{tooltip, TooltipSide};
