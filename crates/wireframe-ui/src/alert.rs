//! 警告バナー部品（`Alert`、イシュー #2646、Phase 6「Overlay・Feedback」）。
//!
//! 横長の警告・通知バナーの配置イメージだけを示す、アイコン + タイトル +
//! 任意の説明文からなる非インタラクティブなローファイ・プレースホルダー。
//! blocks.pm に対応部品は存在せず、wireframe-ui 独自追加部品である
//! （`docs/design/wireframe-ui-architecture.md` §8 の「独自追加部品」
//! 区分、`site/wireframes/alert.md` の「原案差分メモ」節も参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::alert` showcase
//! （`/wireframes/alert/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # `Severity` は部品ローカルの型（`props.rs` へ昇格しない）
//!
//! [`Severity`] は本モジュール内で定義する。`props.rs` は並行進行中の
//! 兄弟イシュー（Phase 6 の他部品）が同時に触る共有ファイルであり、他部品
//! での需要が見えるまでは部品ローカルに留める方針（`tooltip::TooltipSide`
//! と同じ判断）。重要度は表示状態（`data-*`）ではなく見た目のバリアント
//! であるため、部品固有の修飾 class（`fw-wire-alert-<info|warning|error>`）
//! で表す。
//!
//! # アイコンは `Option<Node>` スロット
//!
//! [`crate::icon`] には info/warning/error 専用のグリフがない。新しい
//! アイコンを追加すると影響範囲が広がるため、`link`（イシュー #2618）・
//! `file_drop`（イシュー #2633）と同じ `Option<Node>` アイコンスロット
//! 規約（`docs/design/wireframe-ui-architecture.md` §11.4）を採用する。
//! 重要度からアイコンを自動選択せず、呼び出し側が任意の既存アイコンを
//! 渡す設計とする。`None` のときはアイコンのパート要素自体を出力しない。
//!
//! # 対話セマンティクスは出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! `role="alert"`・`aria-live`・`aria-*`・`tabindex`・`style`・`on*` を
//! 一切出力しない。閉じるボタンや `<button>` も出力しない。実際に
//! アクセシブルな alert が必要な利用者には Themes の Alert
//! （`/themes/alert/`）を案内する（`site/wireframes/alert.md` 参照）。

use fandhe_frontend_core::{el_owned, span, text, Node};

use crate::class::class_list;
use crate::size::Size;

/// 本文（タイトル + 説明文）ラッパの class（部品ルートなしで単独使用しない、
/// [`alert`] 専用）。
const BODY_CLASS: &str = "fw-wire-alert-body";
/// タイトルパートの class（部品ルートなしで単独使用しない、[`alert`] 専用）。
const TITLE_CLASS: &str = "fw-wire-alert-title";
/// 説明文パートの class（部品ルートなしで単独使用しない、[`alert`] 専用）。
const DESCRIPTION_CLASS: &str = "fw-wire-alert-description";
/// アイコンパートの class（部品ルートなしで単独使用しない、[`alert`] 専用）。
const ICON_CLASS: &str = "fw-wire-alert-icon";

/// 警告バナーの重要度。既定は `Info`。
///
/// モジュール doc「`Severity` は部品ローカルの型」節の通り、部品ローカル
/// の型として定義する（`props.rs` へは昇格しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Severity {
    /// 情報（既定）。
    #[default]
    Info,
    /// 警告。
    Warning,
    /// エラー。
    Error,
}

impl Severity {
    /// 全重要度を宣言順（`Info`〜`Error`）で列挙する。テスト・showcase が
    /// 走査に使う。
    pub const ALL: [Severity; 3] = [Severity::Info, Severity::Warning, Severity::Error];

    /// 重要度名の文字列表現（`"info"`/`"warning"`/`"error"`）。
    pub const fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }

    /// この重要度に対応する修飾 class（`fw-wire-alert-<段階>`）。
    pub const fn class(self) -> &'static str {
        match self {
            Severity::Info => "fw-wire-alert-info",
            Severity::Warning => "fw-wire-alert-warning",
            Severity::Error => "fw-wire-alert-error",
        }
    }
}

/// 警告バナー CSS（モノクロ、`ColorPalette` 非依存）。[`crate::css::PARTS`]
/// へ登録される。
///
/// - Info: 地は `--fw-wire-paper`、枠は `--fw-wire-line`、説明文は
///   `--fw-wire-ink-muted`。
/// - Warning: 地は `--fw-wire-fill-subtle`、左辺の枠を太くしてグレー
///   スケールでも Info と区別できる手がかりを 1 つ付ける。
/// - Error: 反転配色（地・枠とも `--fw-wire-ink`、文字は `--fw-wire-paper`、
///   説明文は `--fw-wire-fill`）。暗い地の上で `ink-muted` を使うと
///   コントラストが不足するため（`annotation` の `Primary` と同じ判断）。
///
/// アイコンは `stroke="currentColor"`（[`crate::icon`]）のため、反転時も
/// 文字色へ自動追従する。
pub const ALERT_CSS: &str = "\
.fw-wire-alert {
  display: flex;
  align-items: flex-start;
  gap: 0.625em;
  box-sizing: border-box;
  max-width: 100%;
  padding: 0.75em 1em;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-alert-icon {
  display: flex;
  flex-shrink: 0;
  color: var(--fw-wire-ink-muted);
}
.fw-wire-alert-icon .fw-wire-icon-glyph {
  width: 1.25em;
  height: 1.25em;
}
.fw-wire-alert-body {
  display: flex;
  flex-direction: column;
  gap: 0.25em;
  min-width: 0;
}
.fw-wire-alert-title {
  font-weight: 600;
}
.fw-wire-alert-description {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-alert-warning {
  background: var(--fw-wire-fill-subtle);
  border-inline-start-width: calc(var(--fw-wire-line-width) * 3);
}
.fw-wire-alert-error {
  background: var(--fw-wire-ink);
  border-color: var(--fw-wire-ink);
  color: var(--fw-wire-paper);
}
.fw-wire-alert-error .fw-wire-alert-icon {
  color: var(--fw-wire-paper);
}
.fw-wire-alert-error .fw-wire-alert-description {
  color: var(--fw-wire-fill);
}
";

/// 警告バナーを組み立てる。
///
/// - `severity`: [`Severity`] 3 段（既定 `Info`）。修飾 class
///   `fw-wire-alert-<info|warning|error>` をルートへ付与する。
/// - `title`: 必須。常に出力するタイトル文言。
/// - `description`: 省略可能な説明文。`None` のときはパート要素自体を
///   出力しない（空要素を残さない）。
/// - `icon`: 省略可能なアイコンスロット（[`crate::icon::bell`] 等の戻り値
///   をそのまま渡す。`docs/design/wireframe-ui-architecture.md` §11.4）。
///   `None` のときはアイコンのパート要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与する。
///
/// テキストは [`fandhe_frontend_core::text`] のみで流し込み（REQ-1 既定
/// エスケープ）、`role`/`aria-*`/`tabindex`/`style`/`on*`/`<button>` は
/// 一切付与しない。`icon` に渡した `Node` が持つ `data-icon` 属性はアイコン
/// 基盤側の識別子であり、そのまま透過する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::alert::Severity;
/// use fandhe_frontend_wireframe_ui::{alert, icon, Size};
///
/// let node = alert(
///     Severity::Warning,
///     "ストレージ容量が残りわずかです",
///     Some("空き容量が 10% を下回りました。"),
///     Some(icon::bell(Size::Md)),
///     Size::Md,
/// );
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-alert fw-wire-alert-warning fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-alert-icon""#));
/// assert!(html.contains("<svg"));
/// assert!(html.contains(r#"class="fw-wire-alert-title""#));
/// assert!(html.contains("ストレージ容量が残りわずかです"));
/// assert!(html.contains(r#"class="fw-wire-alert-description""#));
/// assert!(html.contains("空き容量が 10% を下回りました。"));
///
/// // description・icon を省略するとパート要素自体が出力されない。
/// let minimal = alert(Severity::Info, "タイトルのみ", None, None, Size::Md);
/// let minimal_html = render(&minimal);
/// assert!(!minimal_html.contains("fw-wire-alert-description"));
/// assert!(!minimal_html.contains("fw-wire-alert-icon"));
/// assert!(!minimal_html.contains("<svg"));
///
/// // 3 段の重要度で修飾 class が切り替わる。
/// for severity in Severity::ALL {
///     let html = render(&alert(severity, "件名", None, None, Size::Md));
///     assert!(html.contains(severity.class()));
/// }
///
/// // XSS 回帰: title/description はいずれも既定エスケープを経由する。
/// let escaped = alert(
///     Severity::Error,
///     "<script>alert(1)</script>",
///     Some("\"><img src=x onerror=alert(1)>"),
///     None,
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// assert!(escaped_html.contains("&quot;"));
/// ```
#[must_use]
pub fn alert(
    severity: Severity,
    title: &str,
    description: Option<&str>,
    icon: Option<Node>,
    size: Size,
) -> Node {
    let class = class_list(
        "fw-wire-alert",
        &[Some(severity.class()), Some(size.class())],
    );

    let mut body_children: Vec<Node> = vec![span(vec![("class", TITLE_CLASS)], vec![text(title)])];
    if let Some(description) = description {
        body_children.push(span(
            vec![("class", DESCRIPTION_CLASS)],
            vec![text(description)],
        ));
    }
    let body = el_owned(
        "div",
        vec![("class".to_string(), BODY_CLASS.to_string())],
        body_children,
    );

    let mut children: Vec<Node> = Vec::new();
    if let Some(icon) = icon {
        children.push(el_owned(
            "span",
            vec![("class".to_string(), ICON_CLASS.to_string())],
            vec![icon],
        ));
    }
    children.push(body);

    el_owned("div", vec![("class".to_string(), class)], children)
}
