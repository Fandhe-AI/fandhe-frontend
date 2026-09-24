//! `cta-signup-celebrate` block（イシュー #2550。親 #2530「Phase 7: Motion+
//! 部品化」配下、Motion+ `sections/cta-sections` 由来の合成例で、
//! `crate::blocks` モジュール doc の契約を `login_01`〜`cta_banner_magnetic`
//! に続いて 11 件目に実装する）。
//!
//! # 使用部品
//!
//! `card`（構造）+ `field`/`input`（メールアドレス）+ `button`（送信、
//! `data-fandhe-confetti-trigger` を付与）の 4 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。「送信完了」パネルの
//! チェックマークは `signup_05`/`sidebar_03` と同型の自作幾何アイコン
//! （[`checkmark_icon`]）であり `icon` を `parts` へ列挙しない
//! （`pricing_tiers_morph` の `border_beam` と同じ「単体 Themes ページを
//! 持たない装飾は列挙しない」方針）。
//!
//! # confetti の使用（既存機構をそのまま合成、新規実装なし）
//!
//! `data-fandhe-confetti-trigger="cta-signup-celebrate-canvas"`（送信ボタン）
//! と `data-fandhe-confetti-canvas`（対象 `<canvas id="cta-signup-celebrate-
//! canvas">`）は `crates/wasm-full/src/confetti.rs`（イシュー #2533、既に
//! 実装済み）がそのまま消費する opt-in 属性の組である。本 block は
//! confetti 自体を新規実装せず、既存機構を「使う側」として合成する。
//!
//! # no-JS（docs サイト）での「2 状態併記」
//!
//! docs サイトは JS ハイドレーションを一切行わない
//! （`crate::blocks` モジュール doc 参照）。このため本 Demo では実際の
//! confetti 発火（クリック委譲）は起きない。`sidebar_07`/
//! `pricing_tiers_morph` と同じ「2 状態併記」パターンを採用し、「送信前」
//! カード（email 入力 + confetti トリガー付き送信ボタン + canvas）と
//! 「送信完了（celebrate）」カード（チェックマーク + 完了メッセージ）を
//! 縦に並べて静的に掲示する（各カードに [`LAYOUT_CSS`] のキャプションを
//! 付ける）。実アプリで `wasm-full` の `confetti` feature（既定 on）を
//! 有効にし両属性を著者が付与すると、実際のクリックで発火する。
//!
//! # `<form>` を使わない・実データを持たない・アプリロジックを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。送信ボタンは `button::button` の既定 `type="button"` のまま
//! 送信先を持たず、値は一切送信されない。メールアドレス例・完了メッセージ
//! はいずれも架空のものであり、実企業名・実クレデンシャル・PII を含まない
//! 静的な合成例である（`docs/policy/intentional-non-adoption.md` §3.25:
//! UI コンポーネント層はアプリケーションロジックを内包しない）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`button::button`/`icon::icon` は `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有スタイルは `data-blocks-cta-signup-celebrate-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（`crate::blocks`
//! モジュール doc「CSS フックが `class` と `[data-*]` で混在する理由」節
//! 参照）。一方 `card::header`/`body`/`title`/`description`（variant を
//! 持たず `attrs` をそのまま連結する）と素の `div`/`canvas` には `class` が
//! そのまま効く。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::Size;

/// confetti 発火対象 canvas の `id`（`data-fandhe-confetti-trigger` の値と
/// 一致させる、モジュール doc「confetti の使用」節参照）。
const CANVAS_ID: &str = "cta-signup-celebrate-canvas";

/// 自作のチェックマークアイコン（著作物を複製しない、`signup_05`/
/// `sidebar_03` の `geo_icon` と同型）。
fn checkmark_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Lg,
            ..IconProps::default()
        },
        vec![("data-blocks-cta-signup-celebrate-check", "")],
        vec![el(
            "path",
            vec![
                ("d", "M20 6L9 17l-5-5"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 「送信前」カード（email 入力 + confetti トリガー付き送信ボタン +
/// canvas）。
fn before_submit_card() -> Node {
    let email_field = FieldProps {
        id: "blocks-cta-signup-celebrate-email",
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

    card::root(
        CardProps::default(),
        vec![("data-blocks-cta-signup-celebrate-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("Get notified at launch")]),
                    card::description(
                        vec![],
                        vec![text("We will only email you once, when we ship.")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    field::root(
                        &orientation,
                        &email_field,
                        vec![("data-blocks-cta-signup-celebrate-field", "")],
                        vec![
                            field::label(&email_field, vec![], vec![text("Email")]),
                            input::input(
                                &InputProps::default(),
                                &email_field,
                                vec![("type", "email"), ("placeholder", "m@example.com")],
                            ),
                        ],
                    ),
                    button::button(
                        &ButtonProps::default(),
                        vec![
                            ("data-blocks-cta-signup-celebrate-submit", ""),
                            ("data-fandhe-confetti-trigger", CANVAS_ID),
                        ],
                        vec![text("Notify me")],
                    ),
                    el(
                        "canvas",
                        vec![
                            ("id", CANVAS_ID),
                            ("data-fandhe-confetti-canvas", ""),
                            ("data-blocks-cta-signup-celebrate-canvas", ""),
                            ("aria-hidden", "true"),
                        ],
                        vec![],
                    ),
                ],
            ),
        ],
    )
}

/// 「送信完了（celebrate）」カード（チェックマーク + 完了メッセージ）。
fn after_submit_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-cta-signup-celebrate-card", "")],
        vec![card::body(
            vec![("data-blocks-cta-signup-celebrate-celebrate", "")],
            vec![
                checkmark_icon(),
                card::title(vec![], vec![text("You're all set!")]),
                card::description(vec![], vec![text("We'll email you the moment we launch.")]),
            ],
        )],
    )
}

/// `cta-signup-celebrate` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-cta-signup-celebrate-stack", "")],
        vec![
            div(
                vec![("data-blocks-cta-signup-celebrate-caption", "")],
                vec![text("Before submission")],
            ),
            before_submit_card(),
            div(
                vec![("data-blocks-cta-signup-celebrate-caption", "")],
                vec![text("After submission (celebrate)")],
            ),
            after_submit_card(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cta-signup-celebrate/",
    title: "cta-signup-celebrate",
    category: BlockCategory::Cta,
    rust_source: "crates/docs-site/src/blocks/marketing/cta/cta_signup_celebrate.rs",
    demo_class: "blocks-cta-signup-celebrate",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
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
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cta_signup_celebrate` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。`sidebar_07`/`pricing_tiers_morph` と
/// 同型で `pub(super)` として `super::stylesheet` から連結される）。
const LAYOUT_CSS: &str = "\
[data-blocks-cta-signup-celebrate-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  max-width: 24rem;\n  margin: 0 auto;\n}\n\
[data-blocks-cta-signup-celebrate-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-cta-signup-celebrate-field] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n  margin-bottom: 1rem;\n}\n\
[data-blocks-cta-signup-celebrate-submit] {\n  width: 100%;\n}\n\
[data-blocks-cta-signup-celebrate-canvas] {\n  display: block;\n  width: 100%;\n  height: 12rem;\n  margin-top: 0.75rem;\n  pointer-events: none;\n}\n\
[data-blocks-cta-signup-celebrate-celebrate] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 0.5rem;\n  text-align: center;\n  padding: 1.5rem 0;\n}\n\
[data-blocks-cta-signup-celebrate-check] {\n  color: var(--fandhe-color-fg-success, var(--fandhe-color-fg));\n}\n";
