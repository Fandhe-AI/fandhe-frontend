//! 吹き出し部品（`Tooltip`、イシュー #2644、Phase 6「Overlay・Feedback」）。
//!
//! 画面設計図で「ここに補足の吹き出しが出る」という配置を示す、本文 +
//! 三角形の指示子（矢印）だけを持つ非インタラクティブなローファイ・
//! プレースホルダー。実際にホバー・フォーカスで開閉する tooltip では
//! ない（対象要素へのトリガースロット・位置合わせ計算も持たない）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::tooltip` showcase
//! （`/wireframes/tooltip/`）から呼ばれる。`label` は
//! [`fandhe_frontend_core::text`] のみで流し込むため、既定エスケープ
//! （REQ-1）は本モジュールが独自に保証する必要はなく core 側の契約に
//! 委譲される。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §2 のライセンス保留により、
//! blocks.pm の Tooltip が持つ Figma プロパティ（Size / Pointer / Text）は
//! 書き写さない。代わりに §6 の汎用変換規約と、本リポジトリの
//! Themes/Primitives Tooltip が既に持つ `side`（Floating UI 相当、既定
//! top）の語彙から独立に設計した（`site/wireframes/tooltip.md` の
//! 「原案差分メモ」節も参照）。
//!
//! **型名を [`TooltipSide`] とし `Pointer` にしない理由**:
//! 「矢印がどの辺に付くか」を名前にすると blocks.pm の Figma プロパティ
//! 構造をそのまま写すことになり§2 の保留に抵触しかねない。また本
//! リポジトリ既存の `side`（吹き出しが対象から見てどちら側に出るか）と
//! 意味が逆になり混乱を招く。[`TooltipSide`] は「吹き出しが対象の
//! どちら側に出るか」を表し、矢印は反対側の辺から対象の方を向いて出る
//! （`Top` のとき吹き出しは対象の上に出て、矢印は吹き出しの下辺から
//! 下向きに出る）。この対応関係を後から「逆では」と誤って修正されない
//! よう原稿にも明記する。
//!
//! [`TooltipSide`] は部品ローカルの型とし [`crate::props`] へは昇格しない
//! （`props.rs` の「部品をまたいで再利用が見えた時点で共通型へ昇格する」
//! 方針に従い、他部品での需要が見えるまでは保留する）。
//!
//! Bold/Primary/Disabled/Active・アイコンスロットは意図的に持たない。
//! 吹き出しは常に単一の反転配色で表し、強調のバリエーションや無効・
//! 選択といった表示状態を持たせる必要がない。
//!
//! # 対話セマンティクスは出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! `role="tooltip"`・`aria-*`（`aria-describedby` 等も含む）・`title`
//! 属性・`tabindex`・`style`・`on*` を一切出力しない。`<button>`/`<a>`/
//! `<input>`/`<select>` も出力しない。対象要素（トリガー）へのスロットや
//! 対象への位置合わせ（Floating UI 相当）も持たない。位置計算は
//! wasm-full の責務であり、この部品は「吹き出しの見た目」だけを表す。
//! 実際に操作可能な tooltip が必要な利用者には Primitives/Themes の
//! Tooltip を案内する（`site/wireframes/tooltip.md` 参照）。

use fandhe_frontend_core::{el_owned, text, Node};

use crate::class::class_list;
use crate::size::Size;

/// 本文パートの class（部品ルートなしで単独使用しない、[`tooltip`] 専用）。
const CONTENT_CLASS: &str = "fw-wire-tooltip-content";

/// 矢印パートの class（部品ルートなしで単独使用しない、[`tooltip`] 専用）。
const ARROW_CLASS: &str = "fw-wire-tooltip-arrow";

/// 吹き出しが対象から見てどちら側に出るかを表す（部品ローカル型）。
///
/// 矢印は反対側の辺に付き対象の方を向く（例: `Top` なら吹き出しは対象の
/// 上に出て、矢印は吹き出しの下辺から下向きに出る）。モジュール doc の
/// 「API 設計の由来」節も参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipSide {
    /// 対象の上に出る（既定）。矢印は下辺から下向き。
    #[default]
    Top,
    /// 対象の右に出る。矢印は左辺から左向き。
    Right,
    /// 対象の下に出る。矢印は上辺から上向き。
    Bottom,
    /// 対象の左に出る。矢印は右辺から右向き。
    Left,
}

impl TooltipSide {
    /// 全方向を宣言順（`Top`〜`Left`）で列挙する。テスト・showcase が
    /// 全方向を回すために使う。
    pub const ALL: [TooltipSide; 4] = [
        TooltipSide::Top,
        TooltipSide::Right,
        TooltipSide::Bottom,
        TooltipSide::Left,
    ];

    /// この方向に対応する修飾 class（`fw-wire-tooltip-side-<top|right|bottom|left>`）。
    pub const fn class(self) -> &'static str {
        match self {
            TooltipSide::Top => "fw-wire-tooltip-side-top",
            TooltipSide::Right => "fw-wire-tooltip-side-right",
            TooltipSide::Bottom => "fw-wire-tooltip-side-bottom",
            TooltipSide::Left => "fw-wire-tooltip-side-left",
        }
    }
}

/// 吹き出し CSS（グレースケール、`ColorPalette` 非依存）。
/// [`crate::css::PARTS`] へ登録される。
///
/// 本文は `--fw-wire-ink-muted` の地に `--fw-wire-paper` の文字（コントラスト
/// 比はおよそ 7:1）で表し、黒塗り二値ではなくグレースケールの読みやすさを
/// 優先する（`docs/design/wireframe-ui-architecture.md` §3）。方向ごとの
/// 違いは `flex-direction` の切り替えのみで表し、DOM の順序（本文 → 矢印）
/// は常に固定する。
pub const TOOLTIP_CSS: &str = "\
.fw-wire-tooltip {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  max-width: 100%;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-tooltip-content {
  padding: 0.375em 0.625em;
  max-width: 16em;
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-ink-muted);
  color: var(--fw-wire-paper);
  overflow-wrap: anywhere;
}
.fw-wire-tooltip-arrow {
  flex: 0 0 auto;
  width: 0;
  height: 0;
  border: 0.375em solid transparent;
}
.fw-wire-tooltip.fw-wire-tooltip-side-top {
  flex-direction: column;
}
.fw-wire-tooltip.fw-wire-tooltip-side-top .fw-wire-tooltip-arrow {
  border-top-color: var(--fw-wire-ink-muted);
  border-bottom-width: 0;
}
.fw-wire-tooltip.fw-wire-tooltip-side-bottom {
  flex-direction: column-reverse;
}
.fw-wire-tooltip.fw-wire-tooltip-side-bottom .fw-wire-tooltip-arrow {
  border-bottom-color: var(--fw-wire-ink-muted);
  border-top-width: 0;
}
.fw-wire-tooltip.fw-wire-tooltip-side-left {
  flex-direction: row;
}
.fw-wire-tooltip.fw-wire-tooltip-side-left .fw-wire-tooltip-arrow {
  border-left-color: var(--fw-wire-ink-muted);
  border-right-width: 0;
}
.fw-wire-tooltip.fw-wire-tooltip-side-right {
  flex-direction: row-reverse;
}
.fw-wire-tooltip.fw-wire-tooltip-side-right .fw-wire-tooltip-arrow {
  border-right-color: var(--fw-wire-ink-muted);
  border-left-width: 0;
}
";

/// 吹き出しを組み立てる。
///
/// - `label`: 吹き出しの本文。[`fandhe_frontend_core::text`] のみで流し
///   込む（REQ-1 既定エスケープ）。空文字列でも panic せず、空の本文
///   パートを出力する。
/// - `side`: [`TooltipSide`] 吹き出しが対象のどちら側に出るか（既定
///   `Top`）。修飾 class `fw-wire-tooltip-side-<top|right|bottom|left>` を
///   ルートへ付与する。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、寸法は `em` 基準で拡縮する。
///
/// DOM は常に「本文 → 矢印」の順で出力する（方向の違いは CSS の
/// `flex-direction` だけで切り替える）。`role`/`aria-*`/`tabindex`/
/// `style`/`on*` は一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{tooltip, Size};
/// use fandhe_frontend_wireframe_ui::tooltip::TooltipSide;
///
/// let node = tooltip("補足説明", TooltipSide::Top, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-tooltip fw-wire-size-md fw-wire-tooltip-side-top""#));
/// assert_eq!(html.matches(r#"class="fw-wire-tooltip-content""#).count(), 1);
/// assert_eq!(html.matches(r#"class="fw-wire-tooltip-arrow""#).count(), 1);
/// assert!(html.contains("補足説明"));
///
/// // 本文 → 矢印の順は方向によらず一定。
/// let content_idx = html.find("fw-wire-tooltip-content").unwrap();
/// let arrow_idx = html.find("fw-wire-tooltip-arrow").unwrap();
/// assert!(content_idx < arrow_idx);
///
/// // 全方向で修飾 class がちょうど 1 個だけ付く。
/// for side in TooltipSide::ALL {
///     let html = render(&tooltip("説明", side, Size::Md));
///     assert!(html.contains(side.class()));
/// }
///
/// // 空文字列でも panic しない。
/// let empty = tooltip("", TooltipSide::Top, Size::Md);
/// assert!(render(&empty).contains(r#"class="fw-wire-tooltip-content""#));
///
/// // XSS 回帰: 本文は既定エスケープを経由する。
/// let escaped = tooltip("<script>alert(1)</script>", TooltipSide::Top, Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
///
/// // 対話セマンティクスは一切出力しない。
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains(" aria-"));
/// assert!(!html.contains(" tabindex=\""));
/// assert!(!html.contains(" title=\""));
/// assert!(!html.contains("<button"));
/// ```
#[must_use]
pub fn tooltip(label: &str, side: TooltipSide, size: Size) -> Node {
    let class = class_list("fw-wire-tooltip", &[Some(size.class()), Some(side.class())]);

    let content = el_owned(
        "div",
        vec![("class".to_string(), CONTENT_CLASS.to_string())],
        vec![text(label)],
    );
    let arrow = el_owned(
        "div",
        vec![("class".to_string(), ARROW_CLASS.to_string())],
        vec![],
    );

    el_owned(
        "div",
        vec![("class".to_string(), class)],
        vec![content, arrow],
    )
}
