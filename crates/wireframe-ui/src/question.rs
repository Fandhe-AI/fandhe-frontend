//! 質問項目部品（`Question`、イシュー #2630、Phase 4「Forms B」）。
//!
//! ラベル + 任意の補足説明 + フォームコントロール + 任意のヒントという
//! 「1 つの質問項目」の配置イメージだけを示す、非インタラクティブな
//! ローファイ・プレースホルダー。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::question` showcase
//! （`/wireframes/question/`）から呼ばれる。`fandhe_frontend_core::text`
//! のみでテキストを流し込むため、既定エスケープ（REQ-1）は本モジュールが
//! 独自に保証する必要はなく core 側の契約に委譲される。
//!
//! # API 設計の由来
//!
//! blocks.pm の Question 部品の Figma プロパティ構成（`Type`/`Description`/
//! `Hint` 相当のトグル等）はそのまま転写したものではなく、
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立
//! 設計した（`site/wireframes/question.md` の「原案差分メモ」節も参照）。
//! blocks.pm の外観・anatomy・プロパティ構成の実装への転用は書面許諾が
//! 得られるまで保留されている（同文書 §2、イシュー #2602）ため、本部品の
//! 引数構成は blocks.pm の Figma プロパティを参照・書き写さない。
//!
//! # コントロールは `Node` スロット
//!
//! フォームコントロール（[`crate::select`]・[`crate::switch`] 等）は
//! 専用の variant を持たず `control: Node` スロットへ委ねる（設計文書
//! §6「instance swap は `Node` スロット引数で受ける」の典型）。呼び出し側は
//! コントロール自身の見た目（`Active`/`Disabled` 等）を自分で組み立てて
//! 渡し、本モジュールはそれをそのまま 1 子として包むだけで検査・再加工を
//! 行わない。
//!
//! # `<label>`・`<input>` 等は出力しない（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7（非対話制約）に従い、
//! ラベルは `for` 関連付けのない対話要素風の意味論に踏み込まないよう
//! `<label>` ではなく `div` で出力する。`<input>`/`<fieldset>`/`<legend>`・
//! `role`/`aria-*`/`tabindex`/`style`/`on*` は一切出力しない。実際に操作
//! 可能な質問項目が必要な利用者には Themes（`/themes/field/`）/
//! Primitives（`/primitives/field/`）を案内する（`site/wireframes/question.md`
//! 参照）。

use fandhe_frontend_core::{div, el_owned, text, Node};

use crate::class::class_list;
use crate::size::Size;

/// パート class（部品ルートなしで単独使用しない、[`question`] 専用）。
const LABEL_CLASS: &str = "fw-wire-question-label";
const DESCRIPTION_CLASS: &str = "fw-wire-question-description";
const CONTROL_CLASS: &str = "fw-wire-question-control";
const HINT_CLASS: &str = "fw-wire-question-hint";

/// 質問項目 CSS（5 セレクタ）。[`crate::css::PARTS`] へ登録される。
///
/// 値はトークン（`--fw-wire-*`）と [`crate::size::css`] が定義するカスタム
/// プロパティを `var()` で参照するのみで書き写さない。`.fw-wire-question-control`
/// は差し込まれるコントロール自身の幅指定を尊重する薄いラッパである。
pub const QUESTION_CSS: &str = "\
.fw-wire-question {
  display: flex;
  flex-direction: column;
  gap: 0.375em;
  max-width: 100%;
  box-sizing: border-box;
  color: var(--fw-wire-ink);
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  line-height: 1.4;
}
.fw-wire-question-label {
  font-weight: 600;
}
.fw-wire-question-description {
  color: var(--fw-wire-ink-muted);
}
.fw-wire-question-control {
  display: block;
  max-width: 100%;
}
.fw-wire-question-hint {
  font-size: 0.875em;
  color: var(--fw-wire-ink-muted);
}
";

/// 質問項目を組み立てる。
///
/// - `label`: 必須。質問文（常に太字で表示、`Bold` 軸は使わない）。
/// - `description`: 省略可能なラベル直下の補足説明。`None` のときは
///   パート要素自体を出力しない（空要素を残さない）。
/// - `control`: フォームコントロールのスロット（[`crate::select`]・
///   [`crate::switch`] 等の戻り値をそのまま渡す。instance swap 相当）。
///   本モジュールは内容を検査・再加工せず 1 子としてそのまま包む。
/// - `hint`: 省略可能なコントロール直下のヒント（小さめの補助文言）。
///   `None` のときはパート要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として付与し、
///   ラベル/説明/ヒントのフォントサイズにのみ効く。`control` に渡す
///   コントロール自身は自分の size class を持つため、呼び出し側は同じ
///   `size` を両方へ渡すこと（コントロール側の高さ・フォントサイズは
///   本モジュールが制御しない）。
///
/// テキストはいずれも [`fandhe_frontend_core::text`] のみで流し込み
/// （REQ-1 既定エスケープ）、`<label>`/`<input>`/`role`/`aria-*`/`tabindex`/
/// `style`/`data-*` は一切付与しない（`docs/design/wireframe-ui-architecture.md`
/// §5/§7、Question はルート自体の表示状態軸を持たない部品のため `data-*`
/// も不要。`control` スロット内部の `data-*`（コントロール自身の状態表現）
/// はそのまま透過する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::{div, render, text};
/// use fandhe_frontend_wireframe_ui::{question, Size};
///
/// let control = div(vec![("class", "probe")], vec![text("コントロール")]);
/// let node = question("お名前は？", Some("フルネームで入力してください"), control, Some("例: 山田太郎"), Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-question fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-question-label""#));
/// assert!(html.contains("お名前は？"));
/// assert!(html.contains(r#"class="fw-wire-question-description""#));
/// assert!(html.contains("フルネームで入力してください"));
/// assert!(html.contains(r#"class="fw-wire-question-control""#));
/// assert!(html.contains("probe"));
/// assert!(html.contains(r#"class="fw-wire-question-hint""#));
/// assert!(html.contains("例: 山田太郎"));
///
/// // description・hint を省略するとパート要素自体が出力されない。
/// let minimal_control = div(vec![], vec![text("c")]);
/// let minimal = question("質問文のみ", None, minimal_control, None, Size::Md);
/// let minimal_html = render(&minimal);
/// assert!(!minimal_html.contains("fw-wire-question-description"));
/// assert!(!minimal_html.contains("fw-wire-question-hint"));
///
/// // XSS 回帰: label/description/hint はいずれも既定エスケープを経由する。
/// let payload_control = div(vec![], vec![text("c")]);
/// let escaped = question(
///     "<script>alert(1)</script>",
///     Some("\"><img src=x onerror=alert(1)>"),
///     payload_control,
///     Some("<script>alert(2)</script>"),
///     Size::Md,
/// );
/// let escaped_html = render(&escaped);
/// assert!(!escaped_html.contains("<script>alert(1)</script>"));
/// assert!(!escaped_html.contains("<script>alert(2)</script>"));
/// assert!(escaped_html.contains("&lt;script&gt;"));
/// ```
#[must_use]
pub fn question(
    label: &str,
    description: Option<&str>,
    control: Node,
    hint: Option<&str>,
    size: Size,
) -> Node {
    let class = class_list("fw-wire-question", &[Some(size.class())]);

    let mut children: Vec<Node> = vec![div(vec![("class", LABEL_CLASS)], vec![text(label)])];
    if let Some(description) = description {
        children.push(div(
            vec![("class", DESCRIPTION_CLASS)],
            vec![text(description)],
        ));
    }
    children.push(div(vec![("class", CONTROL_CLASS)], vec![control]));
    if let Some(hint) = hint {
        children.push(div(vec![("class", HINT_CLASS)], vec![text(hint)]));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
