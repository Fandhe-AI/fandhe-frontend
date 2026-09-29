//! `action-panel-inline` block（イシュー #2953。親トラッキング #2951
//! 「Blocks アプリケーション B」配下、phase:4）。「タイトルの下に説明文、
//! その右側に操作（ボタン/トグルスイッチ）を横並びで置く区画」の合成例。
//! Application / Action Panel カテゴリ最初の block
//! （`docs/design/docs-site-blocks-section.md` §18 の卒業手順に従い
//! `action_panel.rs` から本ディレクトリへ改名した）。
//!
//! # 使用部品
//!
//! `card` / `heading` / `text` / `button` / `switch` の 5 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 3 インスタンスでレイアウトの差分を表現する（無 JS のため静的併記）
//!
//! 1 block・3 インスタンス縦積みの構成で、次の 3 通りのレイアウトを併記
//! する（イシュー本文の参照: 主 R0733・集約元 R0734・R0735）。
//!
//! - `inline-button`（R0733 代表構成）: 見出し＋説明文の右側にボタン 1 個。
//! - `top-right-button`（R0734 ボタン右上固定）: [`fandhe_frontend_pre_styled_ui::card`]
//!   の `data-has-action`/`action` スロットをそのまま再利用し、独自 CSS を
//!   追加しない（同スロットが header を grid 2 列化し action を右上へ固定
//!   する既存 recipe、イシュー #2046）。
//! - `inline-switch`（R0735 トグルスイッチ版）: `inline-button` と同じ行
//!   構造で、右側をボタンからトグルスイッチへ差し替える。
//!
//! # レイアウト（`40rem` 未満で操作を説明文の下へ回す）
//!
//! `inline-button`/`inline-switch` の行は既定で縦積み（説明文 → 操作）。
//! `40rem` 以上で左右配置（説明文 | 操作）へ切り替える（テーマの
//! breakpoint トークンは `@media` 条件式の中では解決できないため、他の
//! block と同じくリテラル値を直書きする）。`top-right-button` は card の
//! recipe が既に grid レイアウトを担うため、本 block 側の CSS は関与しない。
//!
//! # switch は `disabled: true` の `checked` 初期状態で固定する
//! （イシュー #2953 実装後の指摘対応。PR #3407 codex/review 指摘）
//!
//! 本 Demo は無 JS のため、`switch::hidden_input` が出力する native
//! checkbox はクリック/Space で `checked` を切り替えられる一方、
//! `root`/`control`/`thumb` の `data-state` は描画時の固定値のまま
//! 更新されない。`disabled` を付与しなかった当初実装では、支援技術が
//! 伝える状態（native checkbox の実際の checked）と画面表示（`control`/
//! `thumb` の位置）が操作後に食い違う不変条件違反を起こしていた。
//! `pricing_seats_split` の年払い switch（`disabled: true` により
//! `SwitchProps::disabled` を経由して `hidden_input` へ native
//! `disabled` を出力し、操作自体を実際に抑止する）と同じ解決を採る。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たず、`switch::hidden_input` も送信先を持たない。文言はすべて独自の
//! 架空の日本語ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 見出し（`<h3>`）。3 インスタンス共通で使う。
fn panel_heading(title: &'static str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Lg,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(title)],
    )
}

/// 説明文（ミュート）。3 インスタンス共通で使う。
fn description(body: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(body)],
    )
}

/// `inline-button`（R0733 代表構成）: 見出し＋説明文の右側にボタン 1 個。
fn panel_inline_button() -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-action-panel-inline-panel", "inline-button")],
        vec![
            card::header(vec![], vec![panel_heading("二要素認証")]),
            card::body(
                vec![("class", "blocks-action-panel-inline-row")],
                vec![
                    description("サインイン時にワンタイムコードの入力を求めます。"),
                    div(
                        vec![("data-blocks-action-panel-inline-action", "")],
                        vec![button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("有効にする")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `top-right-button`（R0734 ボタン右上固定）: card の `data-has-action`/
/// `action` スロットをそのまま再利用する（本 block 側は独自 CSS を持たない、
/// モジュール doc「使用部品」節参照）。
fn panel_top_right_button() -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-action-panel-inline-panel", "top-right-button")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    panel_heading("請求先メールアドレス"),
                    card::action(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("変更する")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![description("請求書と領収書の送付先を変更します。")],
            ),
        ],
    )
}

/// `inline-switch`（R0735 トグルスイッチ版）: `inline-button` と同じ行構造
/// で、右側をトグルスイッチへ差し替える。
fn panel_inline_switch() -> Node {
    let switch_props = SwitchProps {
        // モジュール doc「switch は `disabled: true` の `checked` 初期状態で
        // 固定する」節参照。native checkbox の操作を実際に抑止し、
        // 操作後の状態不一致（AT が伝える状態と `data-state` 固定表示の
        // 食い違い）を構造的に防ぐ。
        disabled: true,
        ..SwitchProps::default()
    };
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-action-panel-inline-panel", "inline-switch")],
        vec![
            card::header(vec![], vec![panel_heading("メール通知")]),
            card::body(
                vec![("class", "blocks-action-panel-inline-row")],
                vec![
                    description("週次レポートと重要なお知らせをメールで受け取ります。"),
                    div(
                        vec![("data-blocks-action-panel-inline-action", "")],
                        vec![switch::root(
                            Size::Md,
                            ColorPalette::Accent,
                            true,
                            &switch_props,
                            vec![],
                            vec![
                                switch::label(
                                    true,
                                    &switch_props,
                                    vec![],
                                    vec![text("通知を受け取る")],
                                ),
                                switch::hidden_input(
                                    "blocks-action-panel-inline-notify",
                                    "on",
                                    true,
                                    &switch_props,
                                    vec![],
                                ),
                                switch::control(
                                    true,
                                    &switch_props,
                                    vec![],
                                    vec![switch::thumb(true, &switch_props, vec![], vec![])],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `action-panel-inline` の Demo 本体（3 レイアウトを縦積みで並記する。
/// 呼び出しごとに同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-action-panel-inline-layout")],
        vec![
            panel_inline_button(),
            panel_top_right_button(),
            panel_inline_switch(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/action-panel-inline/",
    title: "action-panel-inline",
    category: BlockCategory::ActionPanel,
    rust_source: "crates/docs-site/src/blocks/application/action_panel/action_panel_inline.rs",
    demo_class: "blocks-action-panel-inline",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
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
            label: "Switch",
            path: "/themes/switch/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `action_panel_inline` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-action-panel-inline-*` と
/// `[data-blocks-action-panel-inline-*]` のみを用いる
/// （`top-right-button` は card 自身の recipe に委譲するため対象外）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-action-panel-inline` だが、`demo()`
/// が返すルート `div` の class は `blocks-action-panel-inline-layout`
/// という別名にする（`card_heading_toolbar` 等と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-action-panel-inline-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-action-panel-inline-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-action-panel-inline-action] {\n  display: flex;\n  flex-shrink: 0;\n  align-items: center;\n}\n\
@media (min-width: 40rem) {\n  .blocks-action-panel-inline-row {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: center;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（card/heading/text/button/switch）の anatomy を
    /// すべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"switch\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// パネルはちょうど 3 件（inline-button / top-right-button /
    /// inline-switch）。
    #[test]
    fn demo_panel_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-action-panel-inline-panel")
                .count(),
            3
        );
    }

    /// `top-right-button` バリアントが card の `data-has-action`/`action`
    /// スロットを再利用していること（本 block 側で独自 CSS を持たない
    /// 契約、モジュール doc「使用部品」節参照）。
    #[test]
    fn top_right_variant_uses_card_action_slot() {
        let html = render(&demo());
        assert_eq!(html.matches("data-has-action=\"\"").count(), 1);
        assert_eq!(html.matches("data-part=\"action\"").count(), 1);
    }

    /// switch は `checked`・`disabled` の初期状態で固定され、
    /// `role="switch"` を持つ（モジュール doc「switch は `disabled: true`
    /// の `checked` 初期状態で固定する」節。操作後の状態不一致を防ぐため
    /// native checkbox の操作自体を抑止する契約を固定する）。
    #[test]
    fn switch_is_checked_initial_state() {
        let html = render(&demo());
        assert_eq!(html.matches("role=\"switch\"").count(), 1);
        assert!(html.contains("checked=\"\""));
        assert!(html.contains("data-state=\"checked\""));
        assert!(html.contains("disabled=\"\""));
    }

    /// `<form>` を出力しない・XSS 回帰の不変条件を固定する
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まない
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-action-panel-inline-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-action-panel-inline-layout");
    }
}
