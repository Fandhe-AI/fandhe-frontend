//! `empty-state-setup-steps` block（イシュー #2971。親トラッキング #2951
//! 「Blocks 目的別パーツ拡充」配下）。対応表 ID R0376 を主参照とし、集約元
//! はこれと同一 1 件のため文言・構成に差分を持たない合成例（大きめの空
//! 状態 + 導入手順 3 段）。`_/blocks-intake/` の対応ファイルは本イシュー
//! 着手時点で本 worktree に存在しないため、原稿・本コメントには対応表 ID
//! のみを記す（`page_heading_meta.rs`〔イシュー #2933〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `empty-state` / `steps` / `button` / `icon` の 4 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。新しい UI 部品は
//! 追加しない。
//!
//! # 構成
//!
//! `empty_state::root`（アイコン・見出し・説明・作成ボタン）の下に
//! `steps::root`（3 段の導入手順、各段は番号 + 見出し + 説明）を縦に積む。
//! `<button>` は空状態の「最初のプロジェクトを作成」1 個のみで、Issue の
//! 「作成ボタンは空状態の直下に 1 個だけ」要件を満たす。
//!
//! # `steps::trigger`/`content`/`separator` を置かない理由
//!
//! - **`trigger`**: 実 `<button>` であり、無 JS の docs サイトでは押しても
//!   何も起きない dead control になる（`error_page_centered.rs` PR #3212
//!   codex レビューと同型の判断）。Issue が求める「作成ボタンは 1 個だけ」
//!   という要件とも整合する。番号・見出し・説明は `item` 直下へ `div`/
//!   `strong`/`text` で直接組み立てる（`indicator` は親が `trigger` である
//!   ことを要求しないため構造上問題ない）。
//! - **`content`**: 非 current の `content` は `data-state="closed"` で
//!   非表示になり、3 段すべての説明を常時表示できない。説明は素の `div`
//!   （[`step_body`]）で書く。
//! - **`separator`**: 各 `item` が「番号 + 見出し + 説明」の縦積みブロック
//!   になるため、番号間の接続線は意味を持たない。
//!
//! # `Steps` の状態
//!
//! `Steps::new(3, 0, Orientation::Horizontal)` で固定する（無 JS のため
//! 状態機械は動かず、最初の手順を current・残りを incomplete とする静的
//! 表示。「ここから始める」の導入手順という趣旨と一致する）。
//!
//! # 狭幅では縦並び（`@container`）
//!
//! `steps::root` の `list`/`item` recipe 既定は横並び（`(0,2,0)` の
//! 詳細度）だが、[`LAYOUT_CSS`] が `40rem` 未満で縦並びへ強制する。
//! Demo 枠（`.blocks-demo`）の幅はブラウザビューポート幅と一致しない
//! （デモ枠が `40rem` 未満でもウィンドウが広ければ横並びのままになる）
//! ため、`@media (min-width: ...)` ではなく `@container`（コンテナクエリ）
//! で判定する（`empty_state_invite_team`/`profile_detail_datalist` と
//! 同型のパターン）。ラッパー `.blocks-empty-state-setup-steps-root` へ
//! `container-type: inline-size` を宣言し、コンテナ幅基準で切り替える。
//!
//! # アイコンは自作の単純幾何図形
//!
//! `page_heading_meta.rs::geo_icon` と同型で、[`icon::icon`] へ独自の
//! `<path d="...">` を渡すのみ（実在ブランドのアイコンセットは使わない）。
//!
//! # `id` を出力しない
//!
//! 他の block（`error_page_centered` 等）と同じく、宙に浮いた ARIA 参照・
//! id 重複を構造的に避けるため `id` 属性は一切出力しない。
//!
//! # `<form>` を持たない・送信処理を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先
//! を持たない（`docs/policy/intentional-non-adoption.md` §3.25：バリデー
//! ション・送信処理は UI コンポーネント層の責務外）。文言はすべて独自の
//! 架空ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, strong, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::empty_state::{
    self, EmptyStateIndicatorVariant, EmptyStateProps,
};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の幾何アイコン（角丸の四角に「+」を重ねた単純な折れ線、
/// `page_heading_meta::geo_icon` と同型で [`icon::icon`] へ独自の `path`/
/// `rect` を渡すのみ）。`aria-hidden="true"` の装飾用 SVG で、実在
/// ブランドのロゴ・商標は模さない。
fn folder_icon() -> Node {
    icon(
        &IconProps::default(),
        // `empty_state::indicator` は `font-size` を Size 連動（Lg で拡大）
        // させ、子アイコンが `1em` で追従する設計（`empty_state.rs` モジュール
        // doc「`_icon: { boxSize: 1em }`」節）。`icon::icon` の `Size` variant
        // は固定 rem 実寸のため、インライン style で上書きして追従させる
        // （インライン style は recipe が発行するクラスより詳細度で勝つ、
        // `empty_state_card_header::folder_plus_icon` と同型の判断）。
        vec![("style", "width: 1em; height: 1em;")],
        vec![
            el(
                "rect",
                vec![
                    ("x", "3"),
                    ("y", "5"),
                    ("width", "18"),
                    ("height", "14"),
                    ("rx", "2"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M12 10v6M9 13h6"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 導入手順 1 段分（番号インジケータ + 見出し + 説明）を組み立てる内部
/// ヘルパ。`steps::trigger`/`content` を使わず `item` 直下へ静的な構造を
/// 置く（モジュール冒頭「`steps::trigger`/`content`/`separator` を置かない
/// 理由」節参照）。
fn step<'a>(s: &Steps, index: usize, heading: &'a str, description: &'a str) -> Node {
    // `aria-current="step"` は本来 `trigger` のみに付与される
    // （`steps.rs` モジュール doc）が、本 Demo は `trigger` を置かないため
    // （モジュール冒頭「`steps::trigger`/`content`/`separator` を置かない
    // 理由」節）現在地が支援技術へ一切伝わらない。`item` は同属性を
    // 予約しないため、current な段にのみ明示付与して代替する。
    let item_attrs = if index == s.step() {
        vec![("aria-current", "step")]
    } else {
        vec![]
    };
    steps::item(
        s,
        index,
        item_attrs,
        vec![
            steps::indicator(s, index, vec![], vec![text((index + 1).to_string())]),
            div(
                vec![("data-blocks-empty-state-setup-steps-step-body", "")],
                vec![
                    div(
                        vec![("data-blocks-empty-state-setup-steps-step-title", "")],
                        vec![strong(vec![], vec![text(heading)])],
                    ),
                    div(vec![], vec![text(description)]),
                ],
            ),
        ],
    )
}

/// `empty-state-setup-steps` の Demo 本体（大きめの空状態 + 導入手順 3 段。
/// 最初の手順を current・残りを incomplete で固定する）。呼び出しごとに
/// 同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let s = Steps::new(3, 0, Orientation::Horizontal);

    let message = empty_state::root(
        &EmptyStateProps {
            size: Size::Lg,
            ..EmptyStateProps::default()
        },
        vec![("data-blocks-empty-state-setup-steps-message", "")],
        vec![empty_state::content(
            vec![],
            vec![
                empty_state::indicator_with(
                    EmptyStateIndicatorVariant::Boxed,
                    vec![],
                    vec![folder_icon()],
                ),
                empty_state::title(vec![], vec![text("まだプロジェクトがありません")]),
                empty_state::description(
                    vec![],
                    vec![text(
                        "最初のプロジェクトを作成すると、ここに一覧が表示されます。",
                    )],
                ),
                empty_state::actions(
                    vec![],
                    vec![button::button(
                        &ButtonProps::default(),
                        vec![],
                        vec![text("最初のプロジェクトを作成")],
                    )],
                ),
            ],
        )],
    );

    let steps_root = steps::root(
        Size::Md,
        ColorPalette::Accent,
        &s,
        vec![("data-blocks-empty-state-setup-steps-steps", "")],
        vec![steps::list(
            &s,
            vec![],
            vec![
                step(
                    &s,
                    0,
                    "プロジェクトを作成",
                    "名前と説明を入力して、新しいプロジェクトを作ります。",
                ),
                step(
                    &s,
                    1,
                    "メンバーを招待",
                    "メールアドレスでチームメンバーを招待します。",
                ),
                step(
                    &s,
                    2,
                    "最初のタスクを登録",
                    "着手するタスクを追加して作業を始めます。",
                ),
            ],
        )],
    );

    div(
        vec![("class", "blocks-empty-state-setup-steps-root")],
        vec![message, steps_root],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/empty-state-setup-steps/",
    title: "empty-state-setup-steps",
    category: BlockCategory::EmptyState,
    rust_source: "crates/docs-site/src/blocks/application/empty_state/empty_state_setup_steps.rs",
    demo_class: "blocks-empty-state-setup-steps",
    parts: &[
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
        Part {
            label: "Steps",
            path: "/themes/steps/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `empty_state_setup_steps` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節。セレクタは
/// `.blocks-empty-state-setup-steps-*` と
/// `[data-blocks-empty-state-setup-steps-*]`、および本 block スコープ
/// （`.blocks-empty-state-setup-steps` 祖先）配下の `steps` recipe 属性
/// セレクタのみを用い、他 block や部品の素のセレクタへ影響させない。
///
/// `list`/`item` の既定（横並び、`(0,2,0)` の詳細度）を `40rem` 未満で
/// 縦並びへ強制する（モジュール冒頭「狭幅では縦並び（`@container`）」節
/// 参照。詳細度 `(0,3,0)` の複合セレクタで recipe に勝つ）。
const LAYOUT_CSS: &str = "\
.blocks-empty-state-setup-steps-root {\n  display: grid;\n  gap: var(--fandhe-space-10);\n  padding: var(--fandhe-space-12) var(--fandhe-space-6);\n  justify-items: center;\n  text-align: center;\n  container-type: inline-size;\n  container-name: blocks-empty-state-setup-steps;\n}\n\
[data-blocks-empty-state-setup-steps-message] {\n  max-width: 36rem;\n  width: 100%;\n}\n\
[data-blocks-empty-state-setup-steps-steps] {\n  width: 100%;\n  max-width: 56rem;\n}\n\
.blocks-empty-state-setup-steps [data-scope=\"steps\"][data-part=\"list\"] {\n  flex-direction: column;\n  align-items: stretch;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-empty-state-setup-steps [data-scope=\"steps\"][data-part=\"item\"] {\n  align-items: flex-start;\n  text-align: left;\n}\n\
@container blocks-empty-state-setup-steps (min-width: 40rem) {\n  \
.blocks-empty-state-setup-steps [data-scope=\"steps\"][data-part=\"list\"] {\n    flex-direction: row;\n    align-items: flex-start;\n  }\n  \
.blocks-empty-state-setup-steps [data-scope=\"steps\"][data-part=\"item\"] {\n    flex: 1 1 0;\n  }\n  \
.blocks-empty-state-setup-steps [data-scope=\"steps\"][data-part=\"item\"]:last-child {\n    flex: 1 1 0;\n  }\n\
}\n\
[data-blocks-empty-state-setup-steps-step-title] {\n  font-weight: 600;\n}\n\
[data-blocks-empty-state-setup-steps-step-body] {\n  display: grid;\n  gap: var(--fandhe-space-1);\n  color: var(--fandhe-color-fg-muted);\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が期待するフック・部品 anatomy・非対話制約を満たしていることを
    /// 固定する（`error_page_centered.rs` と同型の個別回帰）。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = demo_html();
        for hook in [
            "data-blocks-empty-state-setup-steps-message",
            "data-blocks-empty-state-setup-steps-steps",
            "data-blocks-empty-state-setup-steps-step-body",
            "data-blocks-empty-state-setup-steps-step-title",
            "data-scope=\"empty-state\" data-part=\"root\"",
            "data-scope=\"empty-state\" data-part=\"indicator\"",
            "data-scope=\"empty-state\" data-part=\"title\"",
            "data-scope=\"empty-state\" data-part=\"description\"",
            "data-scope=\"empty-state\" data-part=\"actions\"",
            "data-scope=\"steps\" data-part=\"root\"",
            "data-scope=\"steps\" data-part=\"list\"",
            "data-scope=\"steps\" data-part=\"item\"",
            "data-scope=\"steps\" data-part=\"indicator\"",
            "data-state=\"current\"",
            "data-state=\"incomplete\"",
            "aria-current=\"step\"",
        ] {
            assert!(html.contains(hook), "demo output should contain {hook}");
        }
        for absent in [
            "<form",
            "href=",
            "src=\"data:",
            "<script",
            "id=\"",
            "data-scope=\"steps\" data-part=\"trigger\"",
            "data-scope=\"steps\" data-part=\"content\"",
            "data-scope=\"steps\" data-part=\"separator\"",
        ] {
            assert!(
                !html.contains(absent),
                "demo output should not contain {absent}"
            );
        }
        assert_eq!(
            html.matches("<button").count(),
            1,
            "demo should have exactly 1 <button> (the empty-state create action)"
        );
        assert_eq!(html.matches("type=\"button\"").count(), 1);
        assert_eq!(
            html.matches("aria-current=\"step\"").count(),
            1,
            "aria-current=\"step\" should be on exactly the current step's item"
        );
    }

    /// [`LAYOUT_CSS`] が全セレクタ・狭幅ブレークポイントを宣言し、リテラル
    /// 色を持ち込まないことを固定する。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-empty-state-setup-steps-root",
            "[data-blocks-empty-state-setup-steps-message]",
            "[data-blocks-empty-state-setup-steps-steps]",
            "[data-blocks-empty-state-setup-steps-step-title]",
            "[data-blocks-empty-state-setup-steps-step-body]",
            ".blocks-empty-state-setup-steps [data-scope=\"steps\"][data-part=\"list\"]",
            ".blocks-empty-state-setup-steps [data-scope=\"steps\"][data-part=\"item\"]",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("container-name: blocks-empty-state-setup-steps;"));
        assert!(LAYOUT_CSS.contains("@container blocks-empty-state-setup-steps (min-width: 40rem)"));
        assert!(!LAYOUT_CSS.contains("@media"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
    }
}
