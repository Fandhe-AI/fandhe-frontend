//! マウスカーソル・プレースホルダー部品（`Cursor`、イシュー #2642、
//! Phase 5「Navigation」）。
//!
//! 画面設計上の「マウスカーソル（矢印・手のひら）」を、任意の名前タグを
//! 添えて示す非インタラクティブなローファイ・プレースホルダー。共同編集
//! カーソル（他利用者の名前チップ付きポインタ）のような配置イメージを
//! 静的に示す用途を想定する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::cursor` showcase
//! （`/wireframes/cursor/`）から呼ばれる。`fandhe_frontend_core::text` の
//! みでラベルを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # アイコンは新規追加（`link`/`file_drop` とは異なる判断）
//!
//! `link`（イシュー #2618）・`tag`（イシュー #2619）・`file_drop`
//! （イシュー #2633）は既存 [`crate::icon`] の中から呼び出し側が選んで
//! 渡す `Option<Node>` スロット規約を採用したが、本部品はグリフそのもの
//! （矢印・手のひらの形）が部品の本体であり、代わりに使える既存アイコンが
//! 1 つもない。そのため [`crate::icon::cursor_arrow`]/
//! [`crate::icon::cursor_hand`] を新規追加し（`icon::ALL` へ登録済み、
//! `docs/design/wireframe-ui-architecture.md` §11.7）、[`cursor`] が
//! [`CursorKind`] に応じて内部でどちらか一方を呼ぶ設計とした（`ratings`
//! が `icon::star` を再利用するのと同型に、本部品は自分専用のグリフを
//! アイコン基盤側へ持つ）。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §2 の保留（イシュー #2602）
//! に従い、blocks.pm の Figma プロパティ構成を参照・書き写していない。
//! `CursorKind` は本イシューの要求範囲（矢印・手のひらの 2 種）のみを
//! 持つ部品ローカルの列挙型とし、`crate::props` へは昇格させない（部品を
//! またいだ再利用が現時点で見えていないため）。`Active`/`Disabled` は
//! 持たない（hover 状態相当は `CursorKind::Hand` で表現できるため、
//! `Active` 軸を追加すると意味が重複する）。詳細は
//! `site/wireframes/cursor.md` の「原案差分メモ」節も参照。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::icon;
use crate::size::Size;

/// カーソルの見た目の種類。
///
/// 既定は [`CursorKind::Arrow`]。blocks.pm の Figma「Type」プロパティの
/// 値を転写したものではなく、本イシューの要求範囲（矢印・手のひら）から
/// 独立に設計した最小構成である（他の種類が必要になった場合は variant を
/// 追加するだけで拡張できる）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CursorKind {
    /// 矢印カーソル（既定）。
    #[default]
    Arrow,
    /// 手のひら（ポインタ）カーソル。
    Hand,
}

impl CursorKind {
    /// 全種類を宣言順で列挙する。テスト・showcase が走査に使う。
    pub const ALL: [CursorKind; 2] = [CursorKind::Arrow, CursorKind::Hand];

    /// 種類名の文字列表現（`"arrow"`/`"hand"`）。
    pub const fn as_str(self) -> &'static str {
        match self {
            CursorKind::Arrow => "arrow",
            CursorKind::Hand => "hand",
        }
    }

    /// この種類に対応する修飾 class（`fw-wire-cursor-<種類>`）。
    pub const fn class(self) -> &'static str {
        match self {
            CursorKind::Arrow => "fw-wire-cursor-arrow",
            CursorKind::Hand => "fw-wire-cursor-hand",
        }
    }
}

/// 名前タグのパート class（部品ルートなしで単独使用しない、[`cursor`] 専用）。
const LABEL_CLASS: &str = "fw-wire-cursor-label";

/// カーソル CSS（3 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// グリフは [`crate::icon::ICON_GLYPH_CSS`] の `currentColor` 描画（線画の
/// み）を、`.fw-wire-cursor .fw-wire-icon-glyph` で塗り（`fill`）だけ上書き
/// してモノクロの立体感を出す。`size::SCALE` の値は書き写さず `var()` を
/// 参照する（`docs/design/wireframe-ui-architecture.md` §10「値を書き写さ
/// ない」）。
pub const CURSOR_CSS: &str = "\
.fw-wire-cursor {
  display: inline-flex;
  align-items: flex-start;
  gap: 0.25em;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-cursor .fw-wire-icon-glyph {
  fill: var(--fw-wire-paper);
}
.fw-wire-cursor-label {
  margin-top: 0.75em;
  padding: 0.125em 0.5em;
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
  font-size: 0.75em;
  line-height: 1.4;
  white-space: nowrap;
}
";

/// マウスカーソルを組み立てる。
///
/// - `kind`: [`CursorKind`]。`Arrow`（既定）/`Hand` のいずれかのグリフを
///   選ぶ（内部で [`crate::icon::cursor_arrow`]/[`crate::icon::cursor_hand`]
///   を呼ぶ）。
/// - `label`: 省略可能な名前タグ。`Some` のときのみ `fw-wire-cursor-label`
///   パート要素を出力する（共同編集者名のチップのような用途）。`None` の
///   ときは要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class・グリフの両方に反映する。
///
/// ラベルは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`href`/`on*` は一切
/// 付与しない。`<a>`/`<button>` も出力しない（`docs/design/wireframe-ui-architecture.md`
/// §7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{cursor, CursorKind, Size};
///
/// let node = cursor(CursorKind::Hand, Some("にゃんこ"), Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-cursor fw-wire-cursor-hand fw-wire-size-md""#));
/// assert!(html.contains(r#"data-icon="cursor-hand""#));
/// assert!(html.contains(r#"class="fw-wire-cursor-label""#));
/// assert!(html.contains("にゃんこ"));
///
/// // label を省略すると要素自体を出力しない。
/// let minimal = cursor(CursorKind::Arrow, None, Size::Md);
/// let minimal_html = render(&minimal);
/// assert!(minimal_html.contains(r#"data-icon="cursor-arrow""#));
/// assert!(!minimal_html.contains("fw-wire-cursor-label"));
///
/// // XSS 回帰: ラベルは既定エスケープを経由する。
/// let escaped = cursor(CursorKind::Arrow, Some("<script>alert(1)</script>"), Size::Md);
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn cursor(kind: CursorKind, label: Option<&str>, size: Size) -> Node {
    let class = class_list("fw-wire-cursor", &[Some(kind.class()), Some(size.class())]);

    let glyph = match kind {
        CursorKind::Arrow => icon::cursor_arrow(size),
        CursorKind::Hand => icon::cursor_hand(size),
    };

    let mut children: Vec<Node> = vec![glyph];
    if let Some(label) = label {
        children.push(span(vec![("class", LABEL_CLASS)], vec![text(label)]));
    }

    el_owned("span", vec![("class".to_string(), class)], children)
}
