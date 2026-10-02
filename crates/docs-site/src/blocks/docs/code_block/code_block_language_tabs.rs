//! `code-block-language-tabs` block（Docs / Code Block カテゴリ、対応表 ID
//! R0058 を主参照、R0059/R0060 を構成差分として集約した合成例）。
//! 「ヘッダー帯（任意のタイトル・言語切替タブ・コピー操作）＋選択中言語の
//! コード表示」を持つコードブロックの合成例。
//!
//! # 使用部品
//!
//! `tabs` / `code` / `clipboard` / `text` の 4 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 3 インスタンスの併記
//!
//! 各インスタンスで選択言語・ヘッダー構成を変え、3 言語すべてのパネルが
//! どこかで可視になるようにする。
//!
//! - **A（タイトルなし、R0058 基準形）**: 選択言語 Rust。`tabs`
//!   （[`fandhe_frontend_pre_styled_ui::tabs::TabsVariant::Line`]）とコピー
//!   操作だけの最小構成。
//! - **B（大文字タイトル + 控えめなタブ、R0059）**: 選択言語 TOML。タイトルは
//!   `text` を大文字・字間広め・小さめ・muted に装飾する。タブは
//!   [`fandhe_frontend_pre_styled_ui::tabs::TabsVariant::Enclosed`] +
//!   `Size::Sm` で控えめにする。
//! - **C（淡色ヘッダー帯 + 両端寄せ、R0060）**: 選択言語 Shell。ヘッダー帯を
//!   `--fandhe-color-bg-subtle` 系トークンの淡色面にし、タイトルを左・タブと
//!   コピーを右へ両端寄せする。
//!
//! # 無 JS での扱い（選択されていないタブは disabled 固定）
//!
//! docs サイトは無 JS のため `tabs::tabs` は非選択パネルへ `hidden` を
//! 付与し、閲覧者はタブを切り替えられない。`notification_tray_tabs` と
//! 同型の判断で、選択されていない言語の trigger は `disabled: true` に
//! 固定し、押しても何も起きない dead control を残さない。タブが切り替わ
//! らないため、ヘッダーのコピー値（`clipboard::root` の `data-value`）は
//! 表示中のコードと常に一致する。
//!
//! この `disabled` 固定は本 demo（[`demo`] が返す静的 HTML）専用の措置で
//! あり、`frame` 自体は非公開ヘルパーで配布 API ではない。JS が有効な実
//! アプリへこの構成を移植する際は `disabled: *value != selected` の行を
//! 削除し、`tabs::tabs` 通常のクライアント側切替に任せる。その場合
//! `clipboard::root` の `data-value` は本 demo のように描画時の `selected`
//! へ固定せず、選択中タブの変化に追従させて同期を保つこと（固定したまま
//! 移植すると、タブ切替後もコピー値だけが初期選択言語のまま取り残され、
//! このレビュー指摘が懸念する食い違いが実際に起こる）。
//!
//! # `display: contents` によるグリッド配置
//!
//! [`fandhe_frontend_pre_styled_ui::tabs::tabs`] の root は役割を持たない
//! `div`（`data-scope="tabs" data-part="root"`）であり、[`LAYOUT_CSS`] は
//! これへ `display: contents` を与えて list/content を [`frame`] の
//! グリッドへ直接の子として展開する（root 自体は `role` を持たないため
//! アクセシビリティ上の副作用はない）。ヘッダー帯（任意のタイトル・タブ列・
//! コピー操作）とコード表示をそれぞれ `grid-template-areas` の
//! `title`/`tabs`/`copy`/`code` 領域へ割り当て、狭い幅
//! （`@container (max-width: 30rem)`）ではタブ列をヘッダーの 2 行目へ
//! 折り返す。コンテナクエリは対象要素自身ではなく祖先のコンテインメント
//! コンテキストを参照する仕様のため、`container-type: inline-size` は
//! `@container` の対象である [`frame`]（`.blocks-code-block-language-tabs-frame`）
//! 自身ではなく、その親 `.blocks-code-block-language-tabs-layout`（[`demo`]
//! の最外周 `div`）へ置く。3 インスタンスはいずれも layout 幅いっぱいに
//! 広がるため、幅の基準を frame から layout へ変えても折り返し挙動は
//! 変わらない。
//!
//! # フォーカスリングと `overflow: hidden`
//!
//! [`frame`] は角丸のグリッド枠を `overflow: hidden` でクリップする
//! （[`LAYOUT_CSS`] 参照）。既定の `outline-offset`（pre-styled-ui の
//! フォーカスリング規約、正方向オフセット）のままだと、frame の縁に接する
//! tabs trigger・clipboard trigger の `:focus-visible` アウトラインが縁の
//! 外側へはみ出し、この `overflow: hidden` にクリップされてキーボード
//! フォーカス時に見えなくなる。[`LAYOUT_CSS`] は両 trigger の
//! `:focus-visible` のみ `outline-offset: -2px`（内側）へ上書きし、
//! アウトラインを frame の内側に収めてクリップを避ける（他部品・他ページの
//! 既定 `outline-offset` には影響しない、本 block 限定の上書き）。
//!
//! # `<form>` を持たない
//!
//! 本 Demo は `<form>` を出力しない。ボタンは `tabs::tabs`/
//! `clipboard::trigger` の既定 `type="button"` のまま用いる。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 架空の Rust サンプル（`fandhe_frontend_core` の実在 API のみを使う）。
const RUST_CODE: &str = "use fandhe_frontend_core::{render, p, text};\n\nfn main() {\n    let html = render(&p(vec![], vec![text(\"Hello\")]));\n    println!(\"{html}\");\n}";

/// 架空の `Cargo.toml` 断片（具体的なバージョン番号は陳腐化を避けるため
/// 書かない）。
const TOML_CODE: &str = "[dependencies]\nfandhe-frontend-core = { version = \"...\" }";

/// 架空のシェルコマンド列。
const SHELL_CODE: &str = "cargo add fandhe-frontend-core\ncargo run";

/// 言語ごとのコードを引く。
fn code_for(lang: &str) -> &'static str {
    match lang {
        "rust" => RUST_CODE,
        "toml" => TOML_CODE,
        "shell" => SHELL_CODE,
        _ => unreachable!("code_block_language_tabs only declares rust/toml/shell"),
    }
}

/// 言語タブ列 + コピー対象コードを持つヘッダー帯 1 個を組み立てる。
///
/// - `variant`: インスタンス識別子（`"a"`/`"b"`/`"c"`、id の一意化と
///   [`LAYOUT_CSS`] 側の `data-blocks-code-block-language-tabs-variant`
///   セレクタ分岐に使う）。
/// - `title`: ヘッダーに表示するタイトル文言（`None` なら非表示、A 版）。
/// - `selected`: SSR 時点で選択表示する言語（`"rust"`/`"toml"`/`"shell"`）。
/// - `tabs_variant`/`tabs_size`: タブの見た目（B/C 版の控えめな外観差分）。
fn frame(
    variant: &str,
    title: Option<&str>,
    selected: &str,
    tabs_variant: TabsVariant,
    tabs_size: Size,
) -> Node {
    let frame_id = format!("blocks-code-block-language-tabs-{variant}");
    let tabs_id = format!("{frame_id}-tabs");

    let langs = [("rust", "Rust"), ("toml", "TOML"), ("shell", "Shell")];
    let items = langs
        .iter()
        .map(|(value, label)| TabItem {
            value,
            trigger: vec![text(*label)],
            content: vec![el(
                "pre",
                vec![("class", "blocks-code-block-language-tabs-pre")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(code_for(value))],
                )],
            )],
            // 選択されていない言語は disabled 固定（モジュール doc「無 JS
            // での扱い」節参照）。押しても選択状態・パネルが変わらない
            // dead control を残さない。
            disabled: *value != selected,
        })
        .collect();

    let props = TabsProps {
        id: tabs_id.as_str(),
        selected,
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let tabs_node = tabs::tabs(
        tabs_variant,
        tabs_size,
        ColorPalette::default(),
        &props,
        items,
    );

    let title_node = title.map(|label| {
        div(
            vec![("class", "blocks-code-block-language-tabs-title")],
            vec![styled_text::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(label)],
            )],
        )
    });

    let copy_node = div(
        vec![("class", "blocks-code-block-language-tabs-copy")],
        vec![clipboard::root(
            code_for(selected),
            false,
            vec![],
            vec![clipboard::trigger(
                false,
                vec![("aria-label", "コードをコピー")],
                vec![
                    clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                    clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                ],
            )],
        )],
    );

    let mut children = Vec::with_capacity(3);
    if let Some(title_node) = title_node {
        children.push(title_node);
    }
    children.push(tabs_node);
    children.push(copy_node);

    div(
        vec![
            ("id", frame_id.as_str()),
            ("class", "blocks-code-block-language-tabs-frame"),
            ("data-blocks-code-block-language-tabs-variant", variant),
        ],
        children,
    )
}

/// 版の差分を短く説明するキャプション（`ai_chat_code_preview::caption` と
/// 同型のパターン）。
fn caption(label: &str) -> Node {
    el(
        "p",
        vec![("class", "blocks-code-block-language-tabs-caption")],
        vec![text(label)],
    )
}

/// `code-block-language-tabs` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-code-block-language-tabs-layout")],
        vec![
            caption("A: タイトルなし・タブとコピーのみ（基準形）"),
            frame("a", None, "rust", TabsVariant::Line, Size::Md),
            caption("B: 大文字タイトル・控えめなタブ"),
            frame(
                "b",
                Some("依存関係に追加"),
                "toml",
                TabsVariant::Enclosed,
                Size::Sm,
            ),
            caption("C: 淡色ヘッダー帯・両端寄せ"),
            frame(
                "c",
                Some("セットアップ"),
                "shell",
                TabsVariant::Line,
                Size::Sm,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/code-block-language-tabs/",
    title: "code-block-language-tabs",
    category: BlockCategory::CodeBlock,
    rust_source: "crates/docs-site/src/blocks/docs/code_block/code_block_language_tabs.rs",
    demo_class: "blocks-code-block-language-tabs",
    parts: &[
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `code_block_language_tabs` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。選択されていない content の
/// 非表示は部品側（`tabs::tabs` の `hidden`/`data-state="inactive"`）が
/// 担い、本 CSS は `display: contents`（グリッド展開用）以外の非表示宣言を
/// 持たない。
const LAYOUT_CSS: &str = "\
.blocks-code-block-language-tabs-layout {\n  container-type: inline-size;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-code-block-language-tabs-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-code-block-language-tabs-frame {\n  display: grid;\n  grid-template-columns: auto 1fr auto;\n  grid-template-areas: \"title tabs copy\" \"code code code\";\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
.blocks-code-block-language-tabs-frame [data-scope=\"tabs\"][data-part=\"trigger\"]:focus-visible,\n.blocks-code-block-language-tabs-frame [data-scope=\"clipboard\"][data-part=\"trigger\"]:focus-visible {\n  outline-offset: -2px;\n}\n\
.blocks-code-block-language-tabs-frame[data-blocks-code-block-language-tabs-variant=\"a\"] {\n  grid-template-areas: \"tabs tabs copy\" \"code code code\";\n}\n\
.blocks-code-block-language-tabs-title {\n  grid-area: title;\n  display: flex;\n  align-items: center;\n  min-width: 0;\n  padding-inline-start: var(--fandhe-space-4);\n}\n\
.blocks-code-block-language-tabs-title [data-scope=\"text\"] {\n  margin: 0;\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
.blocks-code-block-language-tabs-frame > [data-scope=\"tabs\"][data-part=\"root\"] {\n  display: contents;\n}\n\
.blocks-code-block-language-tabs-frame [data-scope=\"tabs\"][data-part=\"list\"] {\n  grid-area: tabs;\n  --fandhe-tabs-list-border-bottom: none;\n  flex-wrap: wrap;\n  min-width: 0;\n  padding-inline: var(--fandhe-space-2);\n}\n\
.blocks-code-block-language-tabs-frame[data-blocks-code-block-language-tabs-variant=\"c\"] [data-scope=\"tabs\"][data-part=\"list\"] {\n  justify-content: flex-end;\n}\n\
.blocks-code-block-language-tabs-frame [data-scope=\"tabs\"][data-part=\"content\"] {\n  grid-area: code;\n}\n\
.blocks-code-block-language-tabs-copy {\n  grid-area: copy;\n  display: flex;\n  align-items: center;\n  padding-inline-end: var(--fandhe-space-3);\n}\n\
.blocks-code-block-language-tabs-pre {\n  grid-column: 1 / -1;\n  margin: 0;\n  overflow-x: auto;\n  padding: var(--fandhe-space-4);\n  font-family: var(--fandhe-font-font-mono);\n}\n\
.blocks-code-block-language-tabs-frame .blocks-code-block-language-tabs-pre [data-scope=\"code\"][data-part=\"root\"] {\n  background: transparent;\n  padding: 0;\n  border-radius: 0;\n}\n\
.blocks-code-block-language-tabs-frame[data-blocks-code-block-language-tabs-variant=\"c\"]::before {\n  content: \"\";\n  grid-column: 1 / -1;\n  grid-row: 1;\n  background: var(--fandhe-color-bg-subtle);\n}\n\
@container (max-width: 30rem) {\n  \
.blocks-code-block-language-tabs-frame {\n    grid-template-columns: 1fr auto;\n    grid-template-areas: \"title copy\" \"tabs tabs\" \"code code\";\n  }\n  \
.blocks-code-block-language-tabs-frame[data-blocks-code-block-language-tabs-variant=\"a\"] {\n    grid-template-areas: \". copy\" \"tabs tabs\" \"code code\";\n  }\n  \
.blocks-code-block-language-tabs-frame[data-blocks-code-block-language-tabs-variant=\"c\"]::before {\n    grid-row: 1 / 3;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"tabs\"",
            "data-scope=\"code\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    /// 3 インスタンスそれぞれで選択中（active）の content がちょうど 1 枚、
    /// 非選択は `hidden` であること。
    #[test]
    fn each_instance_has_exactly_one_active_panel() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"tabs\" data-part=\"root\"")
                .count(),
            3
        );
        // `data-state` は trigger・content の両方に出力されるため、選択中
        // 1 言語につき 2 箇所 × 3 インスタンス = 6 件。
        assert_eq!(html.matches("data-state=\"active\"").count(), 6);
        // 非選択 2 言語 × trigger/content 2 箇所 × 3 インスタンス = 12 件。
        assert_eq!(html.matches("data-state=\"inactive\"").count(), 12);
    }

    /// 選択言語の和集合が Rust/TOML/Shell の 3 言語すべてを含むこと。
    #[test]
    fn selected_languages_cover_all_three() {
        let html = demo_html();
        // サニティ: Rust コード片は非同期を含まない。
        assert!(!html.contains("async fn"));
        assert!(html.contains("fn main"));
        assert!(html.contains("[dependencies]"));
        assert!(html.contains("cargo add fandhe-frontend-core"));
    }

    /// 選択されていない trigger は `disabled` であること（dead control を
    /// 残さない契約）。
    #[test]
    fn non_selected_triggers_are_disabled() {
        let html = demo_html();
        // 1 インスタンスにつき非選択 2 個 × 3 インスタンス = 6 個。
        assert_eq!(html.matches("data-disabled=\"\"").count(), 6);
    }

    /// 各インスタンスの clipboard `data-value` が選択中パネルのコードと
    /// 一致すること（モジュール doc「無 JS での扱い」節参照）。
    #[test]
    fn clipboard_value_matches_selected_panel() {
        let html = demo_html();
        assert!(html.contains("data-value=\"use fandhe_frontend_core"));
        assert!(html.contains("data-value=\"[dependencies]"));
        assert!(html.contains("data-value=\"cargo add fandhe-frontend-core"));
    }

    #[test]
    fn no_form_and_all_buttons_are_type_button() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn no_data_uri_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
    }

    #[test]
    fn ids_have_no_duplicates() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");
    }

    #[test]
    fn layout_css_is_safe_and_uses_container_queries() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container (max-width: 30rem)"));
        // `display: contents`（グリッド展開用）以外に `display: none` を
        // 持たない（非表示は部品側の `hidden` 属性が担う契約）。
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    /// コンテナクエリは対象要素自身ではなく祖先のコンテインメント
    /// コンテキストを参照する仕様のため、`container-type` は `@container`
    /// の対象（`.blocks-code-block-language-tabs-frame`）ではなく祖先の
    /// `.blocks-code-block-language-tabs-layout` に置かれていることを固定
    /// する（frame 自身に置くと祖先コンテインメントが存在せず、幅が
    /// 30rem 以下でも折り返しが発動しない）。
    #[test]
    fn container_type_is_declared_on_ancestor_not_query_target() {
        assert!(LAYOUT_CSS
            .contains(".blocks-code-block-language-tabs-layout {\n  container-type: inline-size;"));
        assert!(!LAYOUT_CSS
            .contains(".blocks-code-block-language-tabs-frame {\n  container-type: inline-size;"));
    }

    /// frame の `overflow: hidden`（角丸クリップ用）がキーボードフォーカス
    /// リングを巻き込まないよう、tabs/clipboard の両 trigger の
    /// `:focus-visible` を負のオフセット（内側）へ上書きしていることを
    /// 固定する（正のオフセットのままだとアウトラインが frame の縁の外側
    /// へはみ出しクリップされる）。
    #[test]
    fn focus_rings_are_inset_to_avoid_overflow_clipping() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"tabs\"][data-part=\"trigger\"]:focus-visible,\n.blocks-code-block-language-tabs-frame [data-scope=\"clipboard\"][data-part=\"trigger\"]:focus-visible {\n  outline-offset: -2px;"
        ));
    }

    /// [`super::frame`] の `[data-scope="code"]` 上書きセレクタが recipe の
    /// variant クラス（`[data-scope="code"][data-part="root"].fd-code--*`、
    /// 詳細度 (0,3,0)）を上回ることを固定する（`ai_chat_code_preview.rs` 等
    /// 既存の `*_selector_outweighs_recipe_base` と同型の判断軸）。
    #[test]
    fn code_selector_outweighs_recipe_base() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-code-block-language-tabs-frame .blocks-code-block-language-tabs-pre [data-scope=\"code\"][data-part=\"root\"] {\n  background: transparent;"
        ));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
