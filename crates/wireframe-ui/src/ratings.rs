//! 星評価部品（`Ratings`、イシュー #2631、Phase 4「Forms B」）。
//!
//! 星 [`STAR_COUNT`] 個のうち塗った個数だけを視覚的に示す、非インタラクティブな
//! ローファイ・プレースホルダー。実際に評価を選択できる `<input type="radio">`
//! 群・`role="radiogroup"`・キーボード操作・ポインタ操作のいずれも実装しない
//! 表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::ratings` showcase
//! （`/wireframes/ratings/`）から呼ばれる。`&str` 引数を持たない（評価は
//! `u8`、サイズは [`Size`]）ため、既定エスケープ（REQ-1）の対象となる動的
//! 文字列は本モジュールに存在しない（構造的に充足、
//! `crates/wireframe-ui/tests/ratings.rs` 冒頭コメント・
//! `site/wireframes/ratings.md` の「原案差分メモ」節も参照）。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約（星評価は
//! `rating: u8` の 1 引数へ畳む）から独立設計した（同 §2 の保留、
//! イシュー #2602、`site/wireframes/ratings.md` の「原案差分メモ」節も
//! 参照）。blocks.pm の Figma プロパティ（Size・Rating・星ごと bool）は
//! 参照・書き写さず、`rating`/`size` の 2 引数のみを持つ。`Active`/
//! `Disabled` 軸・ラベル・最大数（`max`）引数は発明しない
//! （`question`/`slider` と同じ判断）。
//!
//! 星の描画には新規のジオメトリを起こさず、既存の SVG アイコン基盤
//! （[`crate::icon::star`]）をそのまま再利用する（`docs/design/wireframe-ui-architecture.md`
//! §11.4 のスロット規約・§2「ジオメトリの出自」の方針に沿う）。
//!
//! # 非対話制約（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7 に従い、ルートは `div` と
//! し `role`/`aria-*`/`tabindex`/`style`/`on*`・`<input type="radio">`・
//! `<button>`・`<a href>` は一切出力しない。塗り状態は
//! [`crate::props::Active`] の `data-active` のみで表現し、評価値そのもの
//! を `data-rating`/`data-value` のような表示状態用途外の `data-*` へ
//! 出力することもしない（`docs/design/wireframe-ui-architecture.md` §5
//! 「表示状態を示す `data-*` まで」の責務境界、`slider` の「値は
//! `data-*` へ出さない」判断と同じ）。星の `<svg>` 自体は `aria-hidden`/
//! `focusable="false"` を持つ（[`crate::icon`] の出力契約）。ルートへ
//! `role="img"`/`aria-label` を付けることもしない（`icon` 部品
//! （イシュー #2652）との継ぎ目を侵さない）。実際に操作可能な評価入力
//! が必要な利用者には Themes（`/themes/rating-group/`）/
//! Primitives（`/primitives/rating-group/`）を案内する
//! （`site/wireframes/ratings.md` 参照）。

use fandhe_frontend_core::{el_owned, Node};

use crate::class::class_list;
use crate::icon;
use crate::props::Active;
use crate::size::Size;

/// 星の総数（固定）。[`ratings`] はこの個数の星を常に描く。
pub const STAR_COUNT: u8 = 5;

/// 各星のパート class（部品ルートなしで単独使用しない、[`ratings`] 専用）。
const STAR_CLASS: &str = "fw-wire-ratings-star";

/// 星評価 CSS（4 規則）。[`crate::css::PARTS`] へ登録される。
///
/// 塗り表現は `.fw-wire-ratings-star[data-active] .fw-wire-icon-glyph`
/// の `fill: currentColor` で行う。SVG の presentation attribute
/// `fill="none"`（[`crate::icon`] の出力契約）より CSS の `fill` 宣言が
/// 優先されるため、[`crate::icon`] 側の出力契約を変えずに塗りを表現
/// できる。色は `--fw-wire-*` トークンのみを参照し（`ColorPalette` や
/// `--fandhe-*` は使わない、グレースケール方針）、`[data-active]` 単独
/// セレクタは書かない（`crates/wireframe-ui/tests/common_api.rs` の
/// 「`.` で始まる行はすべて `.fw-wire-` プレフィックス」走査に引っかから
/// ないよう `.fw-wire-ratings-star[data-active]` の形で書く、
/// `docs/design/wireframe-ui-architecture.md` §10.4）。
pub const RATINGS_CSS: &str = "\
.fw-wire-ratings {
  display: inline-flex;
  align-items: center;
  gap: calc(var(--fw-wire-control-size, 2rem) * 0.1);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-line);
}
.fw-wire-ratings-star {
  display: inline-flex;
  line-height: 0;
}
.fw-wire-ratings-star[data-active] {
  color: var(--fw-wire-ink);
}
.fw-wire-ratings-star[data-active] .fw-wire-icon-glyph {
  fill: currentColor;
}
";

/// 星評価を組み立てる。
///
/// - `rating`: 塗る星の数。[`STAR_COUNT`]（5）超は [`STAR_COUNT`] へ
///   クランプする（全域関数、panic しない）。
/// - `size`: [`Size`] 5 段。星のサイズは [`crate::icon::star`] を経由して
///   `--fw-wire-font-size`（[`crate::size::css`]）へ反映される。
///
/// 先頭から `rating`（クランプ後）個の星へ `data-active=""` を付与する
/// （[`crate::props::Active`] を再利用、塗り済みの意味で消費する。
/// `select`/`radio`/`switch`/`checkbox` に続く実消費者）。root class の
/// 順序は `fw-wire-ratings <size>`。`role`/`aria-*`/`tabindex`/`style`/
/// `on*`・`<input>`/`<button>`/`<a href>` は一切出力しない
/// （`docs/design/wireframe-ui-architecture.md` §5/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{ratings, Size, STAR_COUNT};
///
/// let node = ratings(4, Size::Md);
/// let html = render(&node);
/// assert!(html.starts_with(r#"<div class="fw-wire-ratings fw-wire-size-md">"#));
/// assert_eq!(html.matches(r#"class="fw-wire-ratings-star""#).count(), STAR_COUNT as usize);
/// assert_eq!(html.matches(r#"data-active="""#).count(), 4);
///
/// // 5 超は 5 へクランプされる。
/// let clamped = render(&ratings(9, Size::Md));
/// assert_eq!(clamped.matches(r#"data-active="""#).count(), STAR_COUNT as usize);
///
/// // 非対話制約: `<input>`/`<button>`/`role`/`tabindex`/`style` は一切
/// // 出力しない（`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!html.contains("<input"));
/// assert!(!html.contains("<button"));
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains("tabindex"));
/// assert!(!html.contains(" style=\""));
/// assert!(!html.contains("data-rating"));
/// ```
#[must_use]
pub fn ratings(rating: u8, size: Size) -> Node {
    let clamped = if rating > STAR_COUNT {
        STAR_COUNT
    } else {
        rating
    };

    let class = class_list("fw-wire-ratings", &[Some(size.class())]);

    let stars: Vec<Node> = (0..STAR_COUNT)
        .map(|index| {
            let mut attrs: Vec<(String, String)> =
                vec![("class".to_string(), STAR_CLASS.to_string())];
            if let Some(attr) = Active(index < clamped).attr() {
                attrs.push(attr);
            }
            el_owned("span", attrs, vec![icon::star(size)])
        })
        .collect();

    el_owned("div", vec![("class".to_string(), class)], stars)
}
