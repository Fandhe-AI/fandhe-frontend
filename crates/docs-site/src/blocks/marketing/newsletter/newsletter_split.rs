//! `newsletter-split` block（イシュー #2796。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R1097（基準形）・R1098
//! （ブランド色背景）・R1101（暗色カード内で `xl` 横並び）の 3 件を集約
//! した合成例。Marketing / Newsletter カテゴリ最初の block
//! （`docs/design/docs-site-blocks-section.md` §18 の卒業手順に従い
//! `newsletter.rs` から本ディレクトリへ改名した）。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（`contact_split_info` と同じ
//! ライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `field` / `input` / `button` / `link` /
//! `visually_hidden` / `card` の 8 部品のみを合成する（[`BLOCK`] の
//! `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 3 インスタンスで tone 差分を表現する
//!
//! [`cta_split_actions`](super::super::cta::cta_split_actions) と同型の
//! 「1 block・複数インスタンス縦積み」構成で集約元 3 件を表す。
//!
//! - `plain`（R1097 基準形）: 素の `div`、背景なし。
//! - `accent`（R1098 ブランド色背景）: 素の `div` に
//!   `--fandhe-color-accent` 面を敷く。
//! - `card`（R1101 暗色カード + `xl` 横並び）: [`card::root`]
//!   （`CardVariant::Elevated`）を `fg`/`bg` 反転トークンで暗色化する
//!   （固定の暗色 hex は使わない。ダークテーマでは明色カードへ反転するが、
//!   テーマトークン準拠を優先する意図的な簡略化）。
//!
//! # レイアウト（`lg` で 2 列、`card` tone のみ `xl` まで縦積み）
//!
//! 各行は既定で縦積み（見出し領域の下に登録フォーム）。`64rem`
//! （[`Breakpoint::Lg`](fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg)）
//! 以上で左右 2 列（`minmax(0,1fr) minmax(0,1fr)`）へ切り替える。`card`
//! tone のみ R1101 の「`xl` で横並び」差分を表すため、`lg`〜`xl` 間は
//! 縦積みのまま据え置き、`80rem`
//! （[`Breakpoint::Xl`](fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Xl)）
//! 以上でのみ 2 列化する（`contact_split_info`/`cta_split_actions` と同じ
//! 「テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! ためリテラル値を直書きする」判断）。入力欄 + 送信ボタンは
//! `banner_email_signup` と同じく `40rem` 以上で横並びへ切り替える。
//!
//! # tone 上書きの詳細度
//!
//! `plain`/`accent` は素の `div` へ
//! `[data-blocks-newsletter-split-row][data-blocks-newsletter-split-tone="…"]`
//! （属性 2 個 = 詳細度 (0,2,0)）で背景・文字色を上書きする。`card` tone は
//! [`card::root`] の recipe base（詳細度 (0,3,0)）より高い詳細度が必要な
//! ため、`.blocks-newsletter-split [data-scope="card"][data-part="root"]
//! [data-blocks-newsletter-split-tone="card"]`（1 クラス + 3 属性 =
//! 詳細度 (0,4,0)）で上書きする（`cta_split_actions` と同じ判断軸）。
//! `card::body` 自身も余白を持つため `padding: 0` で二重適用を防ぐ
//! （`cta_split_actions` の Bugbot 指摘の再発防止と同じ判断）。
//!
//! `accent`/`card` 面のボタンは既定 Primary が背景と同化するため、
//! tone 祖先セレクタ + `[data-blocks-newsletter-split-submit]`
//! （詳細度 (0,4,0)）で反転配色へ上書きする。入力欄は `input` recipe 自身が
//! `--fandhe-color-bg` 面を持つため上書きしない。説明文・プライバシー文は
//! `accent`/`card` 面では [`TextVariant::Plain`]（`color` 宣言なし →
//! 祖先の色を継承）、`plain` 面では [`TextVariant::Muted`] を使う
//! （`cta_split_actions::instance_inverted` と同じ理由: `Muted` は自身に
//! `color: var(--fandhe-color-fg-muted)` を持つため祖先の反転配色を上書き
//! してコントラストを崩す）。プライバシー文中の [`link::root`]
//! （`ColorPalette::Neutral`）も同じ理由で `accent`/`card` 面の文字色・
//! hover 色を継承しないため、`banner_full_width_bar` と同じ
//! `[data-blocks-newsletter-split-tone="…"] [data-scope="link"][data-part="root"]`
//! （詳細度 (0,3,0)、通常時・`:hover` の 2 規則）で `color: inherit` へ
//! 上書きする。
//!
//! # 可視ラベルの代わりに `visually_hidden` + `<label for>`
//!
//! [`banner_email_signup`](super::super::banner::banner_email_signup) と
//! 同じく可視ラベルを出さず、[`visually_hidden::root`] で包んだ
//! [`field::label`] が `<label for>` の関連付けでアクセシブル名を確保する。
//! 3 インスタンス分の `id` はそれぞれ一意にし、`for`/`id` の重複を作らない。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # リンク先を固定リポジトリ URL にする・`mailto`/`href="#"` を使わない
//!
//! `crate::blocks` の他 block と同じく [`link::root`] + 固定 URL
//! （[`REPO`]）で「実際に押せる」導線を表す。実在するプライバシーポリシー
//! ページは無いため、可視テキストは「プライバシーポリシー」を名乗らず
//! 遷移先と一致する「プロジェクトリポジトリ」とし、周辺の文言も法的な
//! 同意を主張しない案内文へ変更する（`banner_cookie_consent`
//! 「リンク先を外部リポジトリの URL にする理由」節と同じ判断。イシュー
//! #2796 PR レビュー指摘: 「プライバシーポリシー（GitHub）」の直後に
//! 「同意したことになります」と表示すると、実在しない文書への法的同意を
//! 誤って主張することになる）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。送信ボタンは `button::button` の既定 `type="button"` のまま
//! 送信先を持たない。文言はすべて独自の架空の日本語ダミー（実企業名・
//! 実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// メールアドレス入力 + 送信ボタン + プライバシー文（`signup` 領域）。
/// 可視ラベルは出さず `visually_hidden::root` で包んだ `field::label` が
/// `<label for>` の関連付けを担う（モジュール doc「可視ラベルの代わりに」
/// 節参照）。`tone` は `plain` 以外の面で説明文の配色を継承へ切り替える
/// ため（モジュール doc「tone 上書きの詳細度」節）に使う。
fn signup(tone: &'static str, email_field_id: &'static str) -> Node {
    let email_field = FieldProps {
        id: email_field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let orientation = FieldRootProps {
        orientation: FieldOrientation::Vertical,
    };
    let privacy_variant = if tone == "plain" {
        TextVariant::Muted
    } else {
        TextVariant::Plain
    };

    div(
        vec![("class", "blocks-newsletter-split-signup")],
        vec![
            div(
                vec![("class", "blocks-newsletter-split-controls")],
                vec![
                    field::root(
                        &orientation,
                        &email_field,
                        vec![],
                        vec![
                            visually_hidden::root(
                                vec![],
                                vec![field::label(
                                    &email_field,
                                    vec![],
                                    vec![text("メールアドレス")],
                                )],
                            ),
                            input::input(
                                &InputProps::default(),
                                &email_field,
                                vec![
                                    ("type", "email"),
                                    ("autocomplete", "email"),
                                    ("placeholder", "you@example.com"),
                                ],
                            ),
                        ],
                    ),
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-newsletter-split-submit", "")],
                        vec![text("登録する")],
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    variant: privacy_variant,
                    ..TextProps::default()
                },
                vec![("data-blocks-newsletter-split-privacy", "")],
                vec![
                    text("登録に関する詳細は "),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("プロジェクトリポジトリ")],
                    ),
                    text(" をご確認ください。"),
                ],
            ),
        ],
    )
}

/// 見出し + 説明（`copy` 領域）。`tone` に応じて説明文の配色を切り替える
/// （モジュール doc「tone 上書きの詳細度」節）。
fn copy(tone: &'static str, headline: &'static str, description: &'static str) -> Node {
    let description_variant = if tone == "plain" {
        TextVariant::Muted
    } else {
        TextVariant::Plain
    };
    div(
        vec![("class", "blocks-newsletter-split-copy")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(headline)],
            ),
            styled_text::text(
                &TextProps {
                    variant: description_variant,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
        ],
    )
}

/// 見出し + 登録フォームの行 1 件（`tone`: `plain`/`accent`/`card`。
/// モジュール doc「3 インスタンスで tone 差分を表現する」節参照）。
fn split_row(tone: &'static str, email_field_id: &'static str) -> Node {
    div(
        vec![
            ("data-blocks-newsletter-split-row", ""),
            ("data-blocks-newsletter-split-tone", tone),
        ],
        vec![
            copy(
                tone,
                "毎月の更新情報をメールで受け取る",
                "新しい部品・block の追加を月 1 回まとめてお届けします。",
            ),
            signup(tone, email_field_id),
        ],
    )
}

/// `newsletter-split` の Demo 本体（`plain`/`accent`/`card` の 3 tone を
/// 縦積みで並記する。呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    let card_row = split_row("card", "blocks-newsletter-split-email-card");
    div(
        vec![("class", "blocks-newsletter-split-layout")],
        vec![
            split_row("plain", "blocks-newsletter-split-email-plain"),
            split_row("accent", "blocks-newsletter-split-email-accent"),
            card::root(
                CardProps {
                    variant: CardVariant::Elevated,
                    ..CardProps::default()
                },
                vec![("data-blocks-newsletter-split-tone", "card")],
                vec![card::body(vec![], vec![card_row])],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/newsletter-split/",
    title: "newsletter-split",
    category: BlockCategory::Newsletter,
    rust_source: "crates/docs-site/src/blocks/marketing/newsletter/newsletter_split.rs",
    demo_class: "blocks-newsletter-split",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `newsletter_split` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-newsletter-split-*` と
/// `[data-blocks-newsletter-split-*]` のみを用いる（`contact_split_info`
/// と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-newsletter-split` だが、`demo()` が
/// 返すルート `div` の class は `blocks-newsletter-split-layout` という
/// 別名にする（`contact_split_info`/`cta_split_actions` と同じ Bugbot
/// 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-newsletter-split-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-newsletter-split-row] {\n  display: grid;\n  grid-template-columns: 1fr;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
.blocks-newsletter-split-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-newsletter-split-signup {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-newsletter-split-controls {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-newsletter-split-tone=\"accent\"] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n}\n\
.blocks-newsletter-split [data-scope=\"card\"][data-part=\"root\"][data-blocks-newsletter-split-tone=\"card\"] {\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-newsletter-split [data-scope=\"card\"][data-part=\"body\"] [data-blocks-newsletter-split-row] {\n  padding: 0;\n}\n\
[data-blocks-newsletter-split-tone=\"accent\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-newsletter-split-submit] {\n  background: var(--fandhe-color-accent-fg);\n  color: var(--fandhe-color-accent);\n}\n\
[data-blocks-newsletter-split-tone=\"card\"] [data-scope=\"button\"][data-part=\"root\"][data-blocks-newsletter-split-submit] {\n  background: var(--fandhe-color-bg);\n  color: var(--fandhe-color-fg);\n}\n\
[data-blocks-newsletter-split-tone=\"accent\"] [data-scope=\"link\"][data-part=\"root\"],\n[data-blocks-newsletter-split-tone=\"card\"] [data-scope=\"link\"][data-part=\"root\"] {\n  color: inherit;\n}\n\
[data-blocks-newsletter-split-tone=\"accent\"] [data-scope=\"link\"][data-part=\"root\"]:hover,\n[data-blocks-newsletter-split-tone=\"card\"] [data-scope=\"link\"][data-part=\"root\"]:hover {\n  color: inherit;\n}\n\
@media (min-width: 40rem) {\n  .blocks-newsletter-split-controls {\n    flex-direction: row;\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-newsletter-split-row] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n  }\n  [data-blocks-newsletter-split-row][data-blocks-newsletter-split-tone=\"card\"] {\n    grid-template-columns: 1fr;\n  }\n}\n\
@media (min-width: 80rem) {\n  [data-blocks-newsletter-split-row][data-blocks-newsletter-split-tone=\"card\"] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/field/input/button/link/
    /// visually-hidden/card）の anatomy をすべて実際に出力していることと、
    /// 行数・ボタン数・リンク先を固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"visually-hidden\"",
            "data-scope=\"card\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-newsletter-split-row").count(),
            3,
            "demo should render exactly 3 rows (plain / accent / card tones)"
        );
        assert_eq!(
            html.matches(&format!("href=\"{REPO}\"")).count(),
            3,
            "3 repository links should point to the fixed repository URL"
        );
    }

    /// `<form>` を出力しない・XSS 回帰の不変条件を固定する
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "mailto:",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// 送信ボタンはちょうど 3 個、いずれも `type="button"`。
    #[test]
    fn demo_has_exactly_three_type_button_buttons() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 3);
    }

    /// visually-hidden な `<label for>` と同じ `id` を持つ input が 3 組
    /// 存在し、`id` の重複がない（アクセシブル名の関連付けを固定）。
    #[test]
    fn demo_label_for_matches_input_id() {
        let html = render(&demo());
        for id in [
            "blocks-newsletter-split-email-plain",
            "blocks-newsletter-split-email-accent",
            "blocks-newsletter-split-email-card",
        ] {
            let for_attr = format!(r#"for="{id}-control""#);
            let id_attr = format!(r#"id="{id}-control""#);
            assert!(html.contains(&for_attr), "{for_attr} should be present");
            assert!(html.contains(&id_attr), "{id_attr} should be present");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・tone 上書きセレクタを
    /// 持ち、`<` を含まない（REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 80rem)"));
        assert!(LAYOUT_CSS.contains("[data-blocks-newsletter-split-tone=\"accent\"]"));
        assert!(LAYOUT_CSS.contains("[data-blocks-newsletter-split-tone=\"card\"]"));
    }

    /// プライバシーリンクが `accent`/`card` の反転背景で文字色・hover 色を
    /// 継承する上書きセレクタを持つ（codex/Bugbot 指摘の固定、PR #3246）。
    #[test]
    fn layout_css_overrides_privacy_link_color_on_inverted_tones() {
        for tone in ["accent", "card"] {
            let normal = format!(
                "[data-blocks-newsletter-split-tone=\"{tone}\"] [data-scope=\"link\"][data-part=\"root\"]"
            );
            let hover = format!("{normal}:hover");
            assert!(
                LAYOUT_CSS.contains(&normal),
                "missing normal-state link color override for tone={tone}"
            );
            assert!(
                LAYOUT_CSS.contains(&hover),
                "missing hover-state link color override for tone={tone}"
            );
        }
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-newsletter-split-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-newsletter-split-layout");
    }
}
