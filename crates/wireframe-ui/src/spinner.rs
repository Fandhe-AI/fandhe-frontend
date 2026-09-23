//! ローディングインジケータ部品（`Spinner`、イシュー #2649、
//! Phase 6「Overlay・Feedback」）。
//!
//! 円弧だけを描く円形のプレースホルダー。読み込み中であることを画面
//! 設計図上で示す非インタラクティブなローファイ・プレースホルダーであり、
//! 実際に回転するローディングインジケータではない。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::spinner` showcase
//! （`/wireframes/spinner/`）から呼ばれる。`&str` 引数を持たないため、
//! REQ-1 の既定エスケープが問題になる経路自体が存在しない（本モジュール
//! の doctest・`crates/wireframe-ui/tests/spinner.rs` はこの構造的な
//! 充足を確認する。`slider` 部品〔イシュー #2628、`crates/wireframe-ui/tests/slider.rs`
//! 冒頭 doc〕の先例に倣い、テストのために label 引数を新設することは
//! しない）。
//!
//! # API 設計の由来
//!
//! blocks.pm に同名部品はなく `wireframe-ui` 独自追加の 1 つ
//! （`docs/design/wireframe-ui-architecture.md` §8）。公開 API は
//! `size: Size` 1 引数のみで、props 構造体は導入しない（イシューの
//! 想定どおり）。
//!
//! **CSS リングのみで描き SVG を使わない**: [`crate::icon`] の `icon::ALL`
//! は 21 種の固定レジストリでこれを検証する件数テストを持つため、
//! `icon.rs` へ追加すると当該契約を壊す。代わりに `border` の 1/4 だけを
//! 濃色にした円で円弧を表現する（`SPINNER_CSS` 参照）。
//!
//! **アニメーションは付けない（静的表示）**: `@keyframes`・`animation`・
//! 回転は一切含めない。イシューの「静的表示では円弧のみ」という要件と、
//! 本クレートの SSR 専用・非インタラクティブという位置づけに合わせた
//! 判断（`site/wireframes/spinner.md` の「原案差分メモ」節も参照）。
//!
//! **ARIA・状態を持たない**: `role`/`aria-*`/`tabindex`/`style`/`data-*`
//! はいずれも出力しない。Spinner は表示状態軸を持たない（`Active`/
//! `Disabled` のいずれも消費しない）。実際にアクセシブルな読み込み中
//! 表示が必要な利用者には Themes の Spinner を案内する
//! （`site/wireframes/spinner.md` 参照）。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::size::Size;

/// スピナー CSS（グレースケール、`ColorPalette` 非依存）。
/// [`crate::css::PARTS`] へ登録される。
///
/// 寸法・線幅・色はすべて [`crate::tokens`]・[`crate::size`] の
/// カスタムプロパティを `var()` で参照し、値を直接書き写さない
/// （`docs/design/wireframe-ui-architecture.md` §10「値を書き写さない」）。
/// 薄いリング全体（`--fw-wire-line-subtle`）の上に、上辺だけ濃い色
/// （`--fw-wire-ink`）の境界を重ねて円弧を表す。
pub const SPINNER_CSS: &str = "\
.fw-wire-spinner {
  display: inline-block;
  box-sizing: border-box;
  width: var(--fw-wire-control-size);
  height: var(--fw-wire-control-size);
  border: var(--fw-wire-line-width) solid var(--fw-wire-line-subtle);
  border-top-color: var(--fw-wire-ink);
  border-radius: 50%;
  vertical-align: middle;
  flex-shrink: 0;
}
";

/// ローディングインジケータを組み立てる。
///
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、リングの直径は `var()` 経由でこの段階に連動する（線幅は
///   `--fw-wire-line-width` 固定で size 非連動）。
///
/// 子要素を持たない単一の `<span>` として出力する。`role`/`aria-*`/
/// `tabindex`/`style`/`data-*` は一切出力しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{spinner, Size};
///
/// let node = spinner(Size::Md);
/// let html = render(&node);
/// assert_eq!(html, r#"<span class="fw-wire-spinner fw-wire-size-md"></span>"#);
///
/// // 全段階で決定的にルート class が 1 個だけ付く。
/// for size in Size::ALL {
///     let html = render(&spinner(size));
///     assert!(html.contains(size.class()));
///     assert!(html.starts_with("<span"));
///     assert!(html.ends_with("</span>"));
/// }
///
/// // 対話セマンティクス・表示状態のいずれも出力しない。
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains(" aria-"));
/// assert!(!html.contains(" tabindex=\""));
/// assert!(!html.contains(" style=\""));
/// assert!(!html.contains("data-"));
/// ```
#[must_use]
pub fn spinner(size: Size) -> Node {
    let class = class_list("fw-wire-spinner", &[Some(size.class())]);
    el_owned("span", vec![("class".to_string(), class)], vec![])
}
