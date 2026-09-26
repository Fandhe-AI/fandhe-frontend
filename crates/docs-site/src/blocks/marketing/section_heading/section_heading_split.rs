//! `section-heading-split` block（イシュー #2798。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下）。「タグラインの下に、左列の大見出しと
//! 右列の説明・操作を置き、`lg` 以上で横並びにする」セクション見出しの
//! 合成例。Marketing / Section Heading カテゴリ最初の block
//! （`docs/design/docs-site-blocks-section.md` §18 の卒業手順に従い
//! `section_heading.rs` から本ディレクトリへ改名した）。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない（`newsletter_split`
//! と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `button` / `field` / `input_group` /
//! `input` / `clipboard` / `visually_hidden` の 9 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 5 インスタンスで variant/tone 差分を表現する
//!
//! [`newsletter_split`](super::super::newsletter::newsletter_split) と
//! 同型の「1 block・複数インスタンス縦積み」構成で、集約元 7 件を
//! 次の 5 行へ集約して表す。
//!
//! - 行 1 `variant="description" tone="plain"`: 説明文のみの基準形。
//! - 行 2 `variant="description" tone="accent"`: 基準形と同じ内容だが
//!   `--fandhe-color-accent` 面のアクセント背景差分。
//! - 行 3 `variant="actions" tone="plain"`: 説明 + ボタン 2 個（Primary /
//!   Outline）。
//! - 行 4 `variant="email" tone="plain"`: 説明 + メールアドレス入力
//!   （`field` > `input_group` > [`input`, `input_group::addon` >
//!   `button`]）。
//! - 行 5 `variant="command" tone="plain"`: 説明 + コマンドのコピー欄
//!   （`clipboard`）。
//!
//! # レイアウト（`lg` で 2 列、右列は下端揃え）
//!
//! 各行は既定で縦積み（タグライン → 見出し → 右列の順）。`64rem`
//! （[`Breakpoint::Lg`](fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg)）
//! 以上で左列（見出し）・右列（説明・操作）の 2 列へ切り替え、
//! `align-items: end` で右列の下端を見出しブロックの下端へ揃える
//! （テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! ため、`newsletter_split` 等と同じくリテラル値を直書きする）。`lg`
//! 未満は 1 列の縦積みのため、揃えの問題は生じない。
//!
//! # tone=accent の配色
//!
//! `plain`/`accent` は素の `div` へ
//! `[data-blocks-section-heading-split-row][data-blocks-section-heading-split-tone="…"]`
//! （属性 2 個 = 詳細度 (0,2,0)）で背景・文字色を上書きする
//! （`newsletter_split` と同じ判断軸）。説明文は `accent` 面では
//! [`TextVariant::Plain`]（`color` 宣言なし → 祖先の色を継承）、`plain`
//! 面では [`TextVariant::Muted`] を使う（`Muted` は自身に
//! `color: var(--fandhe-color-fg-muted)` を持つため祖先の反転配色を
//! 上書きしてコントラストを崩すため、`newsletter_split` と同じ理由）。
//! タグライン（`badge`）は `ColorPalette::Neutral` を使い、accent 面でも
//! 淡色背景に溶けないコントラストを保つ。
//!
//! # 可視ラベルの代わりに `visually_hidden` + `<label for>`
//!
//! [`hero_email_signup`](super::super::hero::hero_email_signup) と
//! 同じく、email 行・command 行のラベルは可視ラベルを出さず
//! [`visually_hidden::root`] で包んだ [`field::label`] /
//! [`clipboard::label`] が `<label for>` の関連付けでアクセシブル名を
//! 確保する。email 行の `id` と command 行の clipboard `input` の `id`
//! はそれぞれ一意にし、`for`/`id` の重複を作らない。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # clipboard は無 JS のため idle 状態で固定する
//!
//! コピー実処理・自動リセットはクライアント配線層（wasm-full）の責務
//! であり、無 JS の docs サイトでは `copied = false` の idle 状態のみを
//! 静的に描画する（`interactive_utilities.rs` の Clipboard Demo と同型）。
//! コピー対象値は架空ではなく自プロジェクトの実コマンドにする
//! （機微情報を含まない公開クレート名のみ）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない。文言はすべて独自の架空の日本語ダミー（実企業名・実
//! クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// タグライン（`badge`。行の最上段、`grid-column: 1 / -1` で全幅を占める
/// ことは [`LAYOUT_CSS`] 側で宣言する）。
fn tagline(label: &'static str) -> Node {
    badge::badge(
        &BadgeProps {
            palette: ColorPalette::Neutral,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 左列（大見出し）。
fn heading_col(headline: &'static str) -> Node {
    div(
        vec![("class", "blocks-section-heading-split-heading")],
        vec![heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                ..HeadingProps::default()
            },
            vec![],
            vec![text(headline)],
        )],
    )
}

/// 説明文（`tone` に応じて配色を切り替える。モジュール doc「tone=accent
/// の配色」節参照）。
fn description(tone: &'static str, body: &'static str) -> Node {
    let variant = if tone == "plain" {
        TextVariant::Muted
    } else {
        TextVariant::Plain
    };
    styled_text::text(
        &TextProps {
            variant,
            ..TextProps::default()
        },
        vec![],
        vec![text(body)],
    )
}

/// 右列: 説明のみ（行 1・行 2）。
fn aside_description(tone: &'static str, body: &'static str) -> Node {
    div(
        vec![("class", "blocks-section-heading-split-aside")],
        vec![description(tone, body)],
    )
}

/// 右列: 説明 + ボタン 2 個（行 3）。
fn aside_actions(body: &'static str) -> Node {
    div(
        vec![("class", "blocks-section-heading-split-aside")],
        vec![
            description("plain", body),
            div(
                vec![("data-blocks-section-heading-split-actions", "")],
                vec![
                    button::button(&ButtonProps::default(), vec![], vec![text("詳しく見る")]),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("資料をダウンロード")],
                    ),
                ],
            ),
        ],
    )
}

/// 右列: 説明 + メールアドレス入力（行 4）。`hero_email_signup` と同じ
/// `field::root` > [`field::label`, `input_group::root` > [`input::input`,
/// `input_group::addon` > `button::button`]] の合成契約に従う。
fn aside_email(body: &'static str, field_id: &'static str) -> Node {
    let email_field = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    div(
        vec![("class", "blocks-section-heading-split-aside")],
        vec![
            description("plain", body),
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
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
                    input_group::root(
                        &group_props,
                        vec![],
                        vec![
                            input::input(
                                &InputProps::default(),
                                &email_field,
                                vec![
                                    ("type", "email"),
                                    ("autocomplete", "email"),
                                    ("placeholder", "you@example.com"),
                                ],
                            ),
                            input_group::addon(
                                InputGroupAlign::InlineEnd,
                                &group_props,
                                vec![],
                                vec![button::button(
                                    &ButtonProps::default(),
                                    vec![("data-blocks-section-heading-split-submit", "")],
                                    vec![text("登録する")],
                                )],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 右列: 説明 + コマンドのコピー欄（行 5）。無 JS のため idle 状態
/// （`copied = false`）で固定する（モジュール doc「clipboard は無 JS の
/// ため idle 状態で固定する」節）。
fn aside_command(body: &'static str, input_id: &'static str) -> Node {
    let command = "cargo add fandhe-frontend-core";
    div(
        vec![("class", "blocks-section-heading-split-aside")],
        vec![
            description("plain", body),
            clipboard::root(
                command,
                false,
                vec![],
                vec![
                    visually_hidden::root(
                        vec![],
                        vec![clipboard::label(
                            false,
                            Some(input_id),
                            vec![],
                            vec![text("インストールコマンド")],
                        )],
                    ),
                    clipboard::control(
                        false,
                        vec![],
                        vec![
                            clipboard::input(command, false, vec![("id", input_id)]),
                            clipboard::trigger(
                                false,
                                vec![],
                                vec![
                                    clipboard::indicator(
                                        false,
                                        false,
                                        vec![],
                                        vec![text("コピー")],
                                    ),
                                    clipboard::indicator(
                                        true,
                                        false,
                                        vec![],
                                        vec![text("コピー済み")],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// セクション見出しの行 1 件（`variant`/`tone` の組で識別する。モジュール
/// doc「5 インスタンスで variant/tone 差分を表現する」節参照）。
fn row(variant: &'static str, tone: &'static str, headline: &'static str, aside: Node) -> Node {
    div(
        vec![
            ("data-blocks-section-heading-split-row", ""),
            ("data-blocks-section-heading-split-variant", variant),
            ("data-blocks-section-heading-split-tone", tone),
        ],
        vec![tagline("お知らせ"), heading_col(headline), aside],
    )
}

/// `section-heading-split` の Demo 本体（5 行を縦積みで並記する。呼び出し
/// ごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-section-heading-split-layout")],
        vec![
            row(
                "description",
                "plain",
                "新しい部品体系をまとめて公開しました",
                aside_description(
                    "plain",
                    "既存の部品と組み合わせられる合成例を随時追加しています。",
                ),
            ),
            row(
                "description",
                "accent",
                "アクセント背景で強調するセクション見出し",
                aside_description(
                    "accent",
                    "重要な告知の直後にこの配色を使うと視線が集まります。",
                ),
            ),
            row(
                "actions",
                "plain",
                "導入事例と資料をまとめてご案内します",
                aside_actions("導入の流れと構成例を 1 つの資料にまとめました。"),
            ),
            row(
                "email",
                "plain",
                "更新情報をメールで受け取る",
                aside_email(
                    "新しい部品・block の追加を月 1 回まとめてお届けします。",
                    "blocks-section-heading-split-email",
                ),
            ),
            row(
                "command",
                "plain",
                "コマンド一つで導入できます",
                aside_command(
                    "以下のコマンドをコピーしてプロジェクトへ追加してください。",
                    "blocks-section-heading-split-command",
                ),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/section-heading-split/",
    title: "section-heading-split",
    category: BlockCategory::SectionHeading,
    rust_source: "crates/docs-site/src/blocks/marketing/section_heading/section_heading_split.rs",
    demo_class: "blocks-section-heading-split",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `section_heading_split` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-section-heading-split-*` と
/// `[data-blocks-section-heading-split-*]` のみを用いる
/// （`newsletter_split` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-section-heading-split` だが、`demo()`
/// が返すルート `div` の class は `blocks-section-heading-split-layout`
/// という別名にする（`newsletter_split`/`contact_split_info` と同じ
/// Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-section-heading-split-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-section-heading-split-row] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4) var(--fandhe-space-8);\n  padding: var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
.blocks-section-heading-split-heading {\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-section-heading-split-aside {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-section-heading-split-actions] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-section-heading-split-tone=\"accent\"] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  grid-column: 1 / -1;\n}\n\
@media (min-width: 40rem) {\n  [data-blocks-section-heading-split-actions] {\n    flex-direction: row;\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-section-heading-split-row] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n    align-items: end;\n  }\n  [data-blocks-section-heading-split-row] [data-scope=\"badge\"] {\n    grid-column: 1 / -1;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（badge/heading/text/button/field/input-group/input/
    /// clipboard/visually-hidden）の anatomy をすべて実際に出力している
    /// ことと、行数を固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-section-heading-split-row")
                .count(),
            5,
            "demo should render exactly 5 rows"
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

    /// ボタン（type=button）はちょうど 4 個: actions 行 2 + email 行の
    /// 送信ボタン 1 + clipboard trigger 1。
    #[test]
    fn demo_type_button_count() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 4);
    }

    /// email 行・command 行の `<label for>` と対応する `id` が一致し、
    /// `id` の重複がないことを固定する。
    #[test]
    fn demo_label_for_matches_input_id() {
        let html = render(&demo());
        let email_for = r#"for="blocks-section-heading-split-email-control""#;
        let email_id = r#"id="blocks-section-heading-split-email-control""#;
        assert!(html.contains(email_for), "{email_for} should be present");
        assert!(html.contains(email_id), "{email_id} should be present");

        let command_for = r#"for="blocks-section-heading-split-command""#;
        let command_id = r#"id="blocks-section-heading-split-command""#;
        assert!(
            html.contains(command_for),
            "{command_for} should be present"
        );
        assert!(html.contains(command_id), "{command_id} should be present");
    }

    /// clipboard は idle 状態で固定する（`data-copied` を持たず、idle の
    /// indicator が可視・copied の indicator が非表示）。
    #[test]
    fn clipboard_is_static_idle() {
        let html = render(&demo());
        assert!(!html.contains("data-copied"));
        assert!(html.contains(">コピー<"));
        assert!(html.contains(r#"hidden="""#));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・tone 上書きセレクタを
    /// 持ち、`<` を含まない（REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("align-items: end"));
        assert!(LAYOUT_CSS.contains("[data-blocks-section-heading-split-tone=\"accent\"]"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-section-heading-split-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-section-heading-split-layout"
        );
    }
}
