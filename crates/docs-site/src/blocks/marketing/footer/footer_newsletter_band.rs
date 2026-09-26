//! `footer-newsletter-band` block（イシュー #2852。親トラッキング「Blocks
//! 目的別パーツ拡充」配下、区分 marketing / footer）。
//!
//! 既存の [`footer_newsletter`](super::footer_newsletter)（newsletter を
//! リンク列と並ぶ 1 カラムへ収める形）とは異なり、本 block は
//! **newsletter 帯（band）がリンク列から独立して全幅に広がる**形を持つ。
//! 帯の配置が異なる 3 形を 1 つの Demo に縦に並べる。
//!
//! # 使用部品
//!
//! `field` / `input` / `button` / `link` / `separator` の 5 部品のみを
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。ページサイズ予算（下記節参照）の制約から、当初案にあった
//! `icon`（SNS リンク）・`visually-hidden`（帯フォームの非表示ラベル）・
//! `text`（styled）は使わず、メールラベルは可視のまま `field::label` を
//! 直接出力し、見出し・ラベルの類は `fandhe_frontend_core::text` の
//! 素のテキストノードで表す（差分メモ参照）。
//!
//! # 3 形（帯の位置）
//!
//! - **形 A**（対応表 ID R0964・基準形）: リンク列グリッド → 帯 →
//!   区切り線 → 最下段。
//! - **形 B**（R0495）: 帯 → リンク列グリッド → 区切り線 → 最下段（A と
//!   同じ部品のまま帯とリンク列グリッドの順序だけを入れ替える）。
//! - **形 C**（R0492・簡素形）: リンク列を持たず、インラインナビ + 帯を
//!   同じ上段に横並びで置く → 区切り線 → 最下段。
//!
//! 帯（[`band`]）は A・B ではリンク列グリッドと並ぶ独立区画として、C では
//! 上段内の右側要素として、いずれも同一のヘルパーで描画する（3 形合計で
//! 帯はちょうど 3 個）。
//!
//! # 狭い画面幅では帯の見出しとフォームを縦に積む
//!
//! `footer_newsletter` と同じ閾値 `@media (max-width: 47.99rem)` で、帯
//! （`[data-fb]`）と A/B のリンク列（`.ft`）を `flex-direction: column`
//! へ切り替える。
//!
//! # `<form>` を使わない・送信処理を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。送信ボタンは `button::button` の既定 `type="button"` の
//! まま送信先を持たず、実際の送信処理・バリデーションは利用者自身の
//! Rust コードで実装する（`docs/policy/intentional-non-adoption.md`
//! §3.25）。リンク先はすべて `Fandhe-AI` の実在 GitHub リポジトリへの
//! 外部絶対 URL（[`REPO`]）であり `href="#"` は使わない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `field::root`/`button::button`/`link::root`/`separator` は
//! `drop_class_attr`（または同型の固定属性マージ）により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、帯の目印は
//! `data-fb` 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。
//! 素の `div`/`footer` には `class`（`ft`/`fl`）がそのまま効く。
//!
//! # ページサイズ予算（検索インデックス、イシュー #957）
//!
//! `assets/search-index.json` は 1 ページあたり 4,096 バイト・全体
//! `MAX_INDEX_BYTES` 上限を持ち（`docs/design/docs-site-search-design.md`
//! §3-4・§10）、`MAX_INDEX_BYTES` はこれ以上引き上げないハードルールが
//! 確定している（同文書 §10-5・§10-6）。既存の総ページ数がこの上限へ
//! 迫っているため、本 block は md 原稿の説明文・コードを意図的に簡素に
//! 保つ（helper 関数を極力共有・1 関数化し、装飾的な doc コメントを削る）。
//! 個別ページの索引精度を犠牲にする「その場しのぎ」の是非は同文書
//! §10 が既存ページの追加圧縮について述べたものであり、新規ページを
//! 最初から簡潔に書くことはこれに反しない。実測（base 取り込み時点、
//! `cargo test -p fandhe-frontend-docs-site` 実行時）で総インデックスが
//! `MAX_INDEX_BYTES` を超過することを検知したため、当初計画にあった
//! `icon`（SNS リンク）・`visually-hidden`（帯フォームの非表示ラベル）の
//! 2 部品を本 block から削り、メールラベルは可視のまま出す（差分メモ
//! 参照）。恒久対処（セクション粒度インデックス分割）はイシュー #3173
//! を参照する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, footer, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::{self, FieldOrientation, FieldRootProps};
use fandhe_frontend_pre_styled_ui::input::{self, FieldIds, FieldProps, InputProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};

const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

fn link(label: &'static str) -> Node {
    link::root(REPO, &LinkProps::default(), vec![], vec![text(label)])
}

fn top() -> Node {
    div(vec![("class", "fnb-t")], vec![link("製品")])
}

fn band(id: &'static str) -> Node {
    let f = FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let signup = div(
        vec![],
        vec![
            field::root(
                &FieldRootProps {
                    orientation: FieldOrientation::Vertical,
                },
                &f,
                vec![],
                vec![
                    field::label(&f, vec![], vec![text("メール")]),
                    input::input(&InputProps::default(), &f, vec![("type", "email")]),
                ],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("購読")]),
        ],
    );
    div(vec![("data-fnb-band", "")], vec![text("最新情報"), signup])
}

fn variant(order: u8, id: &'static str) -> Node {
    let mut body = match order {
        0 => vec![top(), band(id)],
        1 => vec![band(id), top()],
        _ => vec![div(vec![("class", "fnb-c")], vec![link("製品"), band(id)])],
    };
    body.push(separator(&SeparatorProps::default(), vec![]));
    body.push(div(vec![], vec![text("© 2026")]));
    footer(vec![], body)
}

pub fn demo() -> Node {
    div(
        vec![("class", "fnb-l")],
        vec![
            text("A"),
            variant(0, "fnb-a"),
            text("B"),
            variant(1, "fnb-b"),
            text("C"),
            variant(2, "fnb-c"),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/footer-newsletter-band/",
    title: "footer-newsletter-band",
    category: BlockCategory::Footer,
    rust_source: "crates/docs-site/src/blocks/marketing/footer/footer_newsletter_band.rs",
    demo_class: "fnb",
    parts: &[
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
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `footer_newsletter_band` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。ルート class（`fnb-l`）は
/// [`Block::demo_class`]（`fnb`）と意図的に別名にする（既存 block と
/// 同じ Bugbot 教訓の回避）。CSS フックは他 block と同じ
/// `blocks-<kebab>-*` 相当の具体性を持たせた `fnb-*` 接頭辞
/// （`footer-newsletter-band` の略）で命名し、`blocks.css` が全 block
/// 共有のグローバルスタイルシートであるため他 block との class 名衝突を
/// 避ける。
const LAYOUT_CSS: &str = "\
.fnb-l {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.fnb-l > footer {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  padding: var(--fandhe-space-6);\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
.fnb-t {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n}\n\
.fnb-c {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n}\n\
[data-fnb-band] {\n  display: flex;\n  justify-content: space-between;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-4) var(--fandhe-space-6);\n  border-top: 1px solid var(--fandhe-color-border);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-fnb-band] > div {\n  display: flex;\n  align-items: flex-end;\n  gap: var(--fandhe-space-2);\n  flex: 0 1 24rem;\n}\n\
@media (max-width: 47.99rem) {\n  \
[data-fnb-band] {\n    flex-direction: column;\n    align-items: stretch;\n  }\n\
  [data-fnb-band] > div {\n    flex-basis: auto;\n  }\n\
  .fnb-t {\n    flex-direction: column;\n    align-items: flex-start;\n  }\n\
  .fnb-c {\n    flex-direction: column;\n    align-items: stretch;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（field/input/button/link/separator）の anatomy を
    /// すべて実際に出力していること、帯がちょうど 3 個であること、
    /// 見出し要素（h1〜h6）を出さないことを固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"field\" data-part=\"root\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-fnb-band").count(),
            3,
            "demo should render exactly 3 bands (A/B/C)"
        );
        for h in ["<h1", "<h2", "<h3", "<h4", "<h5", "<h6"] {
            assert!(!html.contains(h), "demo should not render heading elements");
        }
    }

    /// C にはリンク列 grid（`.fnb-t`）が無いこと
    /// を固定する（A・B の 2 回のみ出現）。
    #[test]
    fn demo_variant_c_has_no_link_columns() {
        let html = render(&demo());
        assert_eq!(
            html.matches("fnb-t").count(),
            2,
            "only A and B should render the link-columns grid"
        );
    }

    /// A と B で帯とリンク列 grid の DOM 順が逆であること（モジュール doc
    /// 「3 形」節の回帰ガード）。
    #[test]
    fn demo_variant_a_and_b_have_reversed_band_order() {
        let html = render(&demo());
        let band_positions: Vec<usize> = html
            .match_indices("data-fnb-band")
            .map(|(idx, _)| idx)
            .collect();
        let top_positions: Vec<usize> = html.match_indices("fnb-t").map(|(idx, _)| idx).collect();
        assert_eq!(band_positions.len(), 3);
        assert_eq!(top_positions.len(), 2);
        // 形 A（先に出現するレイアウト）: リンク列 grid が帯より先。
        assert!(
            top_positions[0] < band_positions[0],
            "形 A はリンク列 grid が帯より先であるべき"
        );
        // 形 B（次に出現するレイアウト）: 帯がリンク列 grid より先。
        assert!(
            band_positions[1] < top_positions[1],
            "形 B は帯がリンク列 grid より先であるべき"
        );
    }

    /// `<form>`/`type="submit"`/`href="#"`/`data:`/`mailto:` を出力しない
    /// ことを固定する（`crate::blocks` モジュール doc）。
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

    /// 購読ボタンはちょうど 3 個、いずれも `type="button"`。
    #[test]
    fn demo_has_exactly_three_type_button_buttons() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"type="button""#).count(), 3);
    }

    /// `<label for>` と同じ `id` を持つ input が 3 組存在すること
    /// （アクセシブル名の関連付けを固定）。
    #[test]
    fn demo_label_for_matches_input_id() {
        let html = render(&demo());
        for id in ["fnb-a", "fnb-b", "fnb-c"] {
            let for_attr = format!(r#"for="{id}-control""#);
            let id_attr = format!(r#"id="{id}-control""#);
            assert!(html.contains(&for_attr), "{for_attr} should be present");
            assert!(html.contains(&id_attr), "{id_attr} should be present");
        }
    }

    /// [`LAYOUT_CSS`] が `<` を含まず（REQ-1: `</style>` によるスタイル脱出
    /// を防ぐ）、狭幅の `@media` を持つことを固定する。
    #[test]
    fn layout_css_has_no_angle_bracket_and_has_narrow_media() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"fnb-l\""));
        assert_ne!(super::BLOCK.demo_class, "fnb-l");
    }
}
