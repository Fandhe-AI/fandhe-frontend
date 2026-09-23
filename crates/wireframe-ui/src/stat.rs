//! 数値指標カード部品（`Stat`、イシュー #2656、Phase 7「Data display」）。
//!
//! ラベル + 大きな数値 + 任意の増減インジケータからなる、非インタラクティブな
//! ローファイ・プレースホルダー。blocks.pm に対応部品は存在せず、
//! wireframe-ui 独自追加部品である（`docs/design/wireframe-ui-architecture.md`
//! §8 の「独自追加部品」区分、`site/wireframes/stat.md` の「原案差分メモ」
//! 節も参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::stat` showcase
//! （`/wireframes/stat/`）から呼ばれる。`fandhe_frontend_core::text` のみで
//! テキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが独自に
//! 保証する必要はなく core 側の契約に委譲される。
//!
//! # `StatTrend`/`StatDelta` は部品ローカルの型（`props.rs` へ昇格しない）
//!
//! [`StatTrend`]・[`StatDelta`] は本モジュール内で定義する。`props.rs` は
//! 並行進行中の兄弟イシューが同時に触る共有ファイルであり、他部品での需要が
//! 見えるまでは部品ローカルに留める方針（`alert::Severity`・
//! `tooltip::TooltipSide` と同じ判断）。
//!
//! delta を `Option<&str>` ではなく `Option<StatDelta<'_>>`（公開構造体、
//! `menu::MenuItem` と同型）としたのは、増減インジケータには向きの情報が
//! 要るためである。文字列の先頭 `+`/`-` から向きを推測する方式は暗黙の
//! 文字列解析になるため採らない。この形なら、delta が存在しないのに向きだけ
//! 存在するという矛盾した状態を型で作れない。
//!
//! # インジケータのグリフは既存アイコンを再利用する
//!
//! [`StatTrend::Up`]/[`StatTrend::Down`] は [`crate::icon::caret_up`]/
//! [`crate::icon::caret_down`] を固定パートとして出力する。[`StatTrend::Flat`]
//! （既定）はグリフを出力しない。新しいアイコンを [`crate::icon`] へ追加
//! すると影響範囲が広がるため、既存の caret アイコンを再利用する
//! （`cursor`〔イシュー #2642〕は、使えるアイコンがなかったための例外）。
//!
//! # 数値は不透明な文字列として扱う
//!
//! `value`・`delta.value` の解析・整形は一切行わない
//! （`docs/policy/intentional-non-adoption.md` §3.23「数値・日時整形は UI
//! コンポーネント層の責務外」と同じ判断軸）。
//!
//! # 対話セマンティクスは出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! `role`・`aria-*`・`tabindex`・`style`・`on*` を一切出力しない。
//! `<button>`/`<a>`/`<input>`/`<select>` も出力しない。実際にアクセシブルな
//! 統計表示が必要な利用者には Themes の Stat（`/themes/stat/`）を案内する
//! （`site/wireframes/stat.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::icon;
use crate::size::Size;

/// ラベルパートの class（部品ルートなしで単独使用しない、[`stat`] 専用）。
const LABEL_CLASS: &str = "fw-wire-stat-label";
/// 数値パートの class（部品ルートなしで単独使用しない、[`stat`] 専用）。
const VALUE_CLASS: &str = "fw-wire-stat-value";
/// 増減パート（アイコン + 増減値のラッパ）の class（部品ルートなしで
/// 単独使用しない、[`stat`] 専用）。
const DELTA_CLASS: &str = "fw-wire-stat-delta";
/// 増減アイコンパートの class（部品ルートなしで単独使用しない、[`stat`]
/// 専用）。
const DELTA_ICON_CLASS: &str = "fw-wire-stat-delta-icon";
/// 増減値パートの class（部品ルートなしで単独使用しない、[`stat`] 専用）。
const DELTA_VALUE_CLASS: &str = "fw-wire-stat-delta-value";

/// 増減の向き。既定は `Flat`（変化なし）。
///
/// モジュール doc「`StatTrend`/`StatDelta` は部品ローカルの型」節の通り、
/// 部品ローカルの型として定義する（`props.rs` へは昇格しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatTrend {
    /// 上昇。
    Up,
    /// 下降。
    Down,
    /// 変化なし（既定）。
    #[default]
    Flat,
}

impl StatTrend {
    /// 全向きを宣言順（`Up`〜`Flat`）で列挙する。テスト・showcase が
    /// 走査に使う。
    pub const ALL: [StatTrend; 3] = [StatTrend::Up, StatTrend::Down, StatTrend::Flat];

    /// 向き名の文字列表現（`"up"`/`"down"`/`"flat"`）。
    pub const fn as_str(self) -> &'static str {
        match self {
            StatTrend::Up => "up",
            StatTrend::Down => "down",
            StatTrend::Flat => "flat",
        }
    }

    /// この向きに対応する修飾 class（`fw-wire-stat-<段階>`）。
    pub const fn class(self) -> &'static str {
        match self {
            StatTrend::Up => "fw-wire-stat-up",
            StatTrend::Down => "fw-wire-stat-down",
            StatTrend::Flat => "fw-wire-stat-flat",
        }
    }
}

/// 増減インジケータの中身（増減値の文言 + 向き）。
///
/// `menu::MenuItem` と同型の公開構造体。delta が存在するのに向きだけ
/// 存在しないという矛盾した状態を型で作れないようにするための設計
/// （モジュール doc参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatDelta<'a> {
    /// 増減値の不透明な文字列表現（例: `"+12%"`）。整形はしない。
    pub value: &'a str,
    /// 増減の向き。
    pub trend: StatTrend,
}

impl<'a> StatDelta<'a> {
    /// [`StatDelta`] を組み立てる。
    #[must_use]
    pub const fn new(value: &'a str, trend: StatTrend) -> Self {
        Self { value, trend }
    }
}

/// 数値指標カード CSS（モノクロ、`ColorPalette` 非依存）。[`crate::css::PARTS`]
/// へ登録される。
///
/// - カード: `--fw-wire-line` の枠・`--fw-wire-radius` の角丸・
///   `--fw-wire-paper` の地・`--fw-wire-ink` の文字。
/// - label: `--fw-wire-ink-muted`。
/// - value: 大きめの `font-size` + `font-weight: 600`。
/// - delta: 向きを色ではなくグレースケールだけで区別する（up は太字、
///   down/flat は `--fw-wire-ink-muted`）。
pub const STAT_CSS: &str = "\
.fw-wire-stat {
  display: inline-flex;
  flex-direction: column;
  gap: 0.25em;
  box-sizing: border-box;
  max-width: 100%;
  padding: 0.75em 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
}
.fw-wire-stat-label {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-stat-value {
  font-size: 2em;
  font-weight: 600;
  line-height: 1.1;
}
.fw-wire-stat-delta {
  display: inline-flex;
  align-items: center;
  gap: 0.25em;
  font-size: 0.875em;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-stat-delta-icon .fw-wire-icon-glyph {
  width: 1em;
  height: 1em;
}
.fw-wire-stat-up {
  font-weight: 600;
  color: var(--fw-wire-ink);
}
";

/// 数値指標カードを組み立てる。
///
/// - `label`: 必須。指標名の文言。
/// - `value`: 必須。不透明な文字列として扱う数値表現（整形はしない）。
/// - `delta`: 省略可能な増減インジケータ（[`StatDelta`]）。`None` のときは
///   パート要素自体を出力しない（空要素を残さない）。`Some` かつ
///   `trend` が `Up`/`Down` のときのみ [`crate::icon::caret_up`]/
///   [`crate::icon::caret_down`] を固定パートとして出力する（`Flat` は
///   グリフを出力しない）。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与する。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`on*`/`<button>` は
/// 一切付与しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::stat::{StatDelta, StatTrend};
/// use fandhe_frontend_wireframe_ui::{stat, Size};
///
/// let node = stat(
///     "売上",
///     "¥1,234,567",
///     Some(StatDelta::new("+12%", StatTrend::Up)),
///     Size::Md,
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-stat fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-stat-label""#));
/// assert!(html.contains("売上"));
/// assert!(html.contains(r#"class="fw-wire-stat-value""#));
/// assert!(html.contains("¥1,234,567"));
/// assert!(html.contains(r#"class="fw-wire-stat-delta fw-wire-stat-up""#));
/// assert!(html.contains("<svg"));
/// assert!(html.contains("+12%"));
///
/// // delta を省略するとパート要素自体が出力されない。
/// let minimal = stat("在庫", "42", None, Size::Md);
/// let minimal_html = render(&minimal);
/// assert!(!minimal_html.contains("fw-wire-stat-delta"));
/// assert!(!minimal_html.contains("<svg"));
///
/// // Flat はグリフを出力しない。
/// let flat = stat(
///     "在庫",
///     "42",
///     Some(StatDelta::new("±0", StatTrend::Flat)),
///     Size::Md,
/// );
/// let flat_html = render(&flat);
/// assert!(flat_html.contains("fw-wire-stat-flat"));
/// assert!(!flat_html.contains("<svg"));
///
/// // XSS 回帰: label/value/delta.value はいずれも既定エスケープを経由する。
/// let escaped = stat(
///     "<script>alert(1)</script>",
///     "\"><img src=x onerror=alert(1)>",
///     Some(StatDelta::new(
///         "<script>alert(2)</script>",
///         StatTrend::Down,
///     )),
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(!escaped_html.contains("<script>alert(2)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// assert!(escaped_html.contains("&quot;"));
/// ```
#[must_use]
pub fn stat(label: &str, value: &str, delta: Option<StatDelta<'_>>, size: Size) -> Node {
    let class = class_list("fw-wire-stat", &[Some(size.class())]);

    let mut children: Vec<Node> = vec![
        span(vec![("class", LABEL_CLASS)], vec![text(label)]),
        span(vec![("class", VALUE_CLASS)], vec![text(value)]),
    ];

    if let Some(delta) = delta {
        let delta_class = class_list(DELTA_CLASS, &[Some(delta.trend.class())]);
        let mut delta_children: Vec<Node> = Vec::new();
        let glyph = match delta.trend {
            StatTrend::Up => Some(icon::caret_up(size)),
            StatTrend::Down => Some(icon::caret_down(size)),
            StatTrend::Flat => None,
        };
        if let Some(glyph) = glyph {
            delta_children.push(el_owned(
                "span",
                vec![("class".to_string(), DELTA_ICON_CLASS.to_string())],
                vec![glyph],
            ));
        }
        delta_children.push(span(
            vec![("class", DELTA_VALUE_CLASS)],
            vec![text(delta.value)],
        ));
        children.push(el_owned(
            "span",
            vec![("class".to_string(), delta_class)],
            delta_children,
        ));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
