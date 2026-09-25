//! `error-page-split-links` block（イシュー #2842。親トラッキング #2807
//! 「Blocks マーケティング B」配下。対応表 ID R0583 を主参照・集約元とする
//! 合成例。本文 + 案内リンクの 2 カラム 404）。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! # 使用部品（イシュー候補の `link` は使わない）
//!
//! `empty-state` / `heading` / `text` / `item` / `icon` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! イシュー本文が候補に挙げる `link` はあえて使わない。案内リンク 1 行は
//! アイコン・ラベル・説明文を持つ複合行全体がクリック対象になる必要が
//! あるが、[`fandhe_frontend_pre_styled_ui::item::root`] に `href` を渡した
//! `<a>` の中へさらに [`fandhe_frontend_pre_styled_ui::link::root`]（同じく
//! `<a>` を描画する）を入れ子にすると不正な HTML（`<a>` の入れ子）になる。
//! `link::root` を `item::title` の中だけに限定する案も検討したが、それでは
//! 行の大半（アイコン・説明文）がクリックできなくなる。よって各行は
//! [`fandhe_frontend_pre_styled_ui::item::root`] の `href` 経路だけで組み、
//! `link` は宣言しない（`error_page_centered.rs` が `button` を撤去した
//! 判断と同型: 部品候補のうち構造上使えないものを外し、モジュール doc に
//! 理由を残す）。
//!
//! # 構成（`item::group` による右カラム、`indicator`/`actions` を持たない
//! 左カラム）
//!
//! 左カラムは [`fandhe_frontend_pre_styled_ui::empty_state::root`] の直下に
//! `content`（タグライン・見出し・説明のみ）を置き、`indicator`
//! （装飾グリフ）・`actions`（ボタン群）は使わない。案内リンクは
//! ボタン群ではなく独立した右カラムのため。
//!
//! 右カラムは [`fandhe_frontend_pre_styled_ui::item::group`]
//! （`role="group"` + `aria-label="Helpful links"`）でまとめた 3 行の
//! [`fandhe_frontend_pre_styled_ui::item::root`]。各行は
//! `media`（アイコン）+ `content`（`title`/`description`）+
//! `actions`（末尾のシェブロン、装飾のため `aria-hidden`）で構成する。
//!
//! # レイアウト（狭い幅は 1 カラム、`64rem` 以上で 2 カラム）
//!
//! `.blocks-error-page-split-links-root` は既定で 1 カラムの縦積み
//! （本文の下にリンク群が続く）。`@media (min-width: 64rem)` で 2 カラムに
//! 切り替える。テーマのブレークポイントトークンは `@media` 条件式の中では
//! 解決できないため、`cta_feature_links.rs`/`contact_split_form_info.rs` と
//! 同じリテラル値（`64rem`）を使う。
//!
//! # 遷移先の方針（参照元の「ブログ」を「examples」へ置き換え）
//!
//! 遷移先はすべて実在する docs サイト内のページにし、表示文言と遷移先が
//! 食い違わないようにする（`error_page_centered.rs` の codex レビュー
//! 是正、イシュー #2837 PR #3212 と同じ教訓）。参照元の 3 件目「ブログ」に
//! 相当するセクションは docs サイトに存在しないため、代わりに
//! `/examples/` を使う（`blog_grid_image.rs` の前例と同じく、存在しない
//! セクションは作り込まず既存のセクションだけで構成する）。`href="#"` は
//! 使わない。
//!
//! - ホーム → `"../../"`（本 block のページ `/blocks/
//!   error-page-split-links/` から見た `/` への相対参照）
//! - ドキュメント → `"../../guides/"`
//! - examples → `"../../examples/"`
//!
//! # アイコンは自作の単純図形
//!
//! 参照元のアイコン（外部アイコンセットのパスデータ）は転記しない
//! （ライセンス上の転記制限、`contact_image_info.rs` の `geo_icon` と同型の
//! 判断）。家（ホーム）・書類（ガイド）・並んだ枠（examples）・右向き
//! シェブロン（行末、装飾）の 4 種を自前の単純な線画で描く。
//!
//! # `id` を出力しない・`<form>` を持たない・送信処理を行わない
//!
//! 他の block と同じく `id` 属性は一切出力しない。`crate::blocks` モジュール
//! doc「`<form>` を使わない」節・「セキュリティ不変条件」節に従い、本 Demo
//! はフォーム・状態機械を持たない静的な合成例である。文言はすべて架空の
//! ダミーであり、実企業名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps, ItemVariant};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextVariant, TextWeight,
};

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する（`contact_image_info.rs` の
/// `geo_icon` と同型の私有ヘルパ、private のため共有できず本ファイル内へ
/// 複製する）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// ホームアイコン（三角の屋根 + 台形の家）。
fn home_icon() -> Node {
    geo_icon("M3 11l9-8 9 8 M5 10v10h5v-6h4v6h5V10")
}

/// ガイド（ドキュメント）アイコン（角の折れた紙 + 本文の横線 2 本）。
fn guides_icon() -> Node {
    geo_icon("M6 3h8l4 4v14H6z M14 3v4h4 M9 12h6 M9 16h6")
}

/// examples アイコン（並んだ 4 枠、サンプル一覧のイメージ）。
fn examples_icon() -> Node {
    geo_icon("M4 4h6v6H4z M14 4h6v6h-6z M4 14h6v6H4z M14 14h6v6h-6z")
}

/// 行末の右向きシェブロン（装飾。`icon` は既定で `aria-hidden="true"` を
/// 付与するため追加の `aria-hidden` 指定は不要）。
fn chevron_icon() -> Node {
    geo_icon("M9 5l7 7-7 7")
}

/// 案内リンク 1 行（アイコン + ラベル + 説明文。行全体が [`item::root`] の
/// `href` によりクリック対象になる。モジュール doc「使用部品」節参照）。
fn help_link(
    glyph: Node,
    label: &'static str,
    description: &'static str,
    href: &'static str,
) -> Node {
    item::root(
        ItemRootProps {
            href: Some(href),
            variant: ItemVariant::Outline,
            ..ItemRootProps::default()
        },
        vec![("data-blocks-error-page-split-links-link", "")],
        vec![
            item::media(ItemMediaVariant::Icon, vec![], vec![glyph]),
            item::content(
                vec![],
                vec![
                    item::title(vec![], vec![text(label)]),
                    item::description(vec![], vec![text(description)]),
                ],
            ),
            item::actions(vec![], vec![chevron_icon()]),
        ],
    )
}

/// `error-page-split-links` の Demo 本体（左: タグライン → 大見出し →
/// 説明文、右: 案内リンク 3 行）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let message = empty_state::root(
        &EmptyStateProps::default(),
        vec![("data-blocks-error-page-split-links-message", "")],
        vec![empty_state::content(
            vec![("class", "blocks-error-page-split-links-content")],
            vec![
                styled_text::text(
                    &TextProps {
                        weight: TextWeight::Semibold,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-error-page-split-links-tagline", "")],
                    vec![text("404 error")],
                ),
                empty_state::title(
                    vec![],
                    vec![heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl3,
                            ..HeadingProps::default()
                        },
                        vec![("data-blocks-error-page-split-links-title", "")],
                        vec![text("We couldn't find that page")],
                    )],
                ),
                empty_state::description(
                    vec![],
                    vec![styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-error-page-split-links-description", "")],
                        vec![text(
                            "The page you're looking for may have been moved, renamed, or is temporarily unavailable.",
                        )],
                    )],
                ),
            ],
        )],
    );

    let links = item::group(
        "Helpful links",
        vec![("data-blocks-error-page-split-links-links", "")],
        vec![
            help_link(
                home_icon(),
                "Back to home",
                "Start again from the documentation home page.",
                "../../",
            ),
            help_link(
                guides_icon(),
                "Read the guides",
                "Step-by-step guides for building with the framework.",
                "../../guides/",
            ),
            help_link(
                examples_icon(),
                "Browse examples",
                "See complete sample projects you can adapt.",
                "../../examples/",
            ),
        ],
    );

    div(
        vec![("class", "blocks-error-page-split-links-root")],
        vec![message, links],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/error-page-split-links/",
    title: "error-page-split-links",
    category: BlockCategory::ErrorPage,
    rust_source: "crates/docs-site/src/blocks/marketing/error_page/error_page_split_links.rs",
    demo_class: "blocks-error-page-split-links",
    parts: &[
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
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
            label: "Item",
            path: "/themes/item/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `error_page_split_links` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で本ファイル内 private
/// 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-error-page-split-links-*` と
/// `[data-blocks-error-page-split-links-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`careers_card_grid` と同じ名前空間分離）。
///
/// 左カラム（[`fandhe_frontend_pre_styled_ui::empty_state`]）は recipe が
/// `root`/`content` の両方に `align-items: center`/`justify-content:
/// center`（`content` は `text-align: center` も）を持つ（詳細度 (0,2,0)）。
/// 本 block は左寄せにするため、`root` は `data-blocks-*-message` 属性、
/// `content` は `class="blocks-error-page-split-links-content"` を combinator
/// にして詳細度 (0,3,0) の上書きセレクタにする（`error_page_centered.rs` の
/// `actions` 上書きと同型の判断、Bugbot 指摘 PR #3212 の教訓を踏襲）。
///
/// 右カラム（[`fandhe_frontend_pre_styled_ui::item::group`]）は gap を
/// 持たないため、`[data-scope="item"][data-part="group"]
/// [data-blocks-error-page-split-links-links]`（同じく詳細度 (0,3,0)）で
/// 行間の gap を追加する。
const LAYOUT_CSS: &str = "\
.blocks-error-page-split-links-root {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  align-items: start;\n  gap: var(--fandhe-space-10);\n  min-height: 22rem;\n  padding: var(--fandhe-space-16) var(--fandhe-space-6);\n}\n\
@media (min-width: 64rem) {\n  .blocks-error-page-split-links-root {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: center;\n  }\n}\n\
[data-scope=\"empty-state\"][data-part=\"root\"][data-blocks-error-page-split-links-message] {\n  --fandhe-empty-state-padding: 0;\n  justify-content: flex-start;\n}\n\
[data-scope=\"empty-state\"][data-part=\"content\"].blocks-error-page-split-links-content {\n  align-items: flex-start;\n  text-align: left;\n}\n\
[data-blocks-error-page-split-links-tagline] {\n  color: var(--fandhe-color-accent);\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
[data-scope=\"item\"][data-part=\"group\"][data-blocks-error-page-split-links-links] {\n  gap: var(--fandhe-space-3);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待するフック・遷移先・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。案内リンク 3 行はいずれも
    /// `data-scope="item" data-part="root"` の `<a href>` であり `<button>`
    /// は出力しないこと、実在するサイト内ページへ遷移することを固定する。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        for hook in [
            "data-blocks-error-page-split-links-message",
            "data-blocks-error-page-split-links-tagline",
            "data-blocks-error-page-split-links-title",
            "data-blocks-error-page-split-links-description",
            "data-blocks-error-page-split-links-links",
            "data-blocks-error-page-split-links-link",
        ] {
            assert!(html.contains(hook), "demo output should contain {hook}");
        }
        assert!(html.contains("href=\"../../\""));
        assert!(html.contains("href=\"../../guides/\""));
        assert!(html.contains("href=\"../../examples/\""));
        assert_eq!(
            html.matches("data-scope=\"item\" data-part=\"root\"")
                .count(),
            3,
            "demo should render exactly 3 item root rows"
        );
        for absent in [
            "<form",
            "<button",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "id=\"",
        ] {
            assert!(
                !html.contains(absent),
                "demo output should not contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタ・`@media` 条件・トークン参照のみを含み
    /// リテラル色を持ち込まないことを固定する。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-error-page-split-links-root",
            "@media (min-width: 64rem)",
            "[data-scope=\"empty-state\"][data-part=\"root\"][data-blocks-error-page-split-links-message]",
            "[data-scope=\"empty-state\"][data-part=\"content\"].blocks-error-page-split-links-content",
            "[data-blocks-error-page-split-links-tagline]",
            "[data-scope=\"item\"][data-part=\"group\"][data-blocks-error-page-split-links-links]",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
    }

    /// 左カラム（empty-state）・右カラム（item group）の上書きセレクタが
    /// (0,3,0) の複合セレクタのまま残っており、単一クラス/属性だけの
    /// (0,2,0) 以下へ後退していないことを固定する（Bugbot 指摘の教訓、
    /// PR #3212 と同型のリグレッション防止）。
    #[test]
    fn overrides_outrank_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"empty-state\"][data-part=\"content\"].blocks-error-page-split-links-content {"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"item\"][data-part=\"group\"][data-blocks-error-page-split-links-links] {"
        ));
        assert!(!LAYOUT_CSS.contains("\n.blocks-error-page-split-links-content {"));
        assert!(!LAYOUT_CSS.contains("\n[data-blocks-error-page-split-links-links] {"));
    }
}
