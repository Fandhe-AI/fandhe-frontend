//! `media-object` block（イシュー #3228。Application 区分の
//! `MediaObject` カテゴリ）。「画像（またはアイコン）+
//! 見出し + 説明文」を横並びにする media object の**整列パターン集**。
//!
//! # 使用部品
//!
//! `item` / `image` の 2 部品のみを合成する（[`BLOCK`] の `parts` に一致
//! させる契約）。新しい UI 部品は追加しない。
//!
//! # 8 パターンを静的併記する（無 JS のため）
//!
//! イシュー本文の対応表 R1062（主参照・上揃え基準形）を軸に、集約元
//! 7 件（R1063〜R1069）の差分を、無 JS でも読み取れるよう 8 インスタンス
//! として縦に併記する。`_/blocks-intake/` の対応ファイルは本イシュー着手
//! 時点で本 worktree に存在しないため、対応表 ID のみを記す
//! （`page_heading_avatar.rs` と同じ扱い）。
//!
//! | variant | 対応 | 表現 |
//! |---|---|---|
//! | `top` | R1062（主参照） | `align-items: flex-start`（基準形） |
//! | `center` | R1063 | `align-items: center` |
//! | `bottom` | R1064 | `align-items: flex-end` |
//! | `stretch` | R1065 | media を行高いっぱいに伸ばす |
//! | `right` | R1066 | `flex-direction: row-reverse` |
//! | `stack` | R1067 | 狭幅で縦積み |
//! | `stack-full` | R1068 | 狭幅で画像を全幅化 |
//! | `nested` | R1069 | 本文内に小さい media object を入れ子 |
//!
//! `stack`/`stack-full` の狭幅表現は、ビューポート全体ではなくデモ枠内の
//! パネル幅で切り替える必要があるため `@media` ではなく `@container`
//! （`navbar_two_row.rs` と同型のパターン、[`LAYOUT_CSS`] 参照）を使う。
//! パネルへ `data-blocks-media-object-narrow` を付与して `max-width` で
//! 実測幅を狭めることで、同一ビューポートのまま静的に折り返し後の見た目を
//! 再現する。
//!
//! # `<form>` を出さない・アプリケーションロジックを持たない
//!
//! ボタン・リンク・`<form>`・`id` 属性は一切出力しない静的な整列パターン
//! 集であり、送信処理・検証・永続化を一切持たない
//! （`docs/policy/intentional-non-adoption.md` §3.25 規則 1）。
//!
//! # ダミー素材について
//!
//! 画像は `crate::blocks::dummy_assets::PRODUCT_SRC`（モノトーン抽象図形の
//! SVG、`build.rs` がビルド時に書き出す）を使い、装飾画像として
//! `alt=""` を付与する。見出し・説明文はすべて独自に書いた架空の文言で
//! あり、実在の人物・企業・クレデンシャルとは無関係。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::item::{self, ItemMediaVariant, ItemRootProps};

/// media（画像）パーツを組み立てる。`extra_attr` は `variant` 固有の
/// フック用 `data-*` 属性（例: `-media-nested`）。
fn media_image(extra_attr: (&'static str, &'static str)) -> Node {
    item::media(
        ItemMediaVariant::Image,
        vec![extra_attr],
        vec![image::image(
            &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
            vec![],
        )],
    )
}

/// content（見出し + 説明文）パーツを組み立てる。
fn body(title: &str, description: &str) -> Node {
    body_with_extra(title, description, Vec::new())
}

/// content（見出し + 説明文 + 追加の子ノード）パーツを組み立てる。`nested`
/// インスタンスが入れ子の media object を本文末尾へ追加するために使う
/// （[`body`] はこの追加子ノードが空の特殊形）。
fn body_with_extra(title: &str, description: &str, extra: Vec<Node>) -> Node {
    let mut children = vec![
        item::title(vec![], vec![text(title)]),
        item::description(vec![], vec![text(description)]),
    ];
    children.extend(extra);
    item::content(vec![], children)
}

/// パネル見出し（短い説明、[`el`] で `<p>` として直接組み立てる。見出し
/// 階層は docs ページ H1 → 生成 H2「Demo」の配下のため `h2`/`h3` を増やさず
/// `p` で十分）。
fn caption(text_content: &str) -> Node {
    el(
        "p",
        vec![("class", "blocks-media-object-caption")],
        vec![text(text_content)],
    )
}

/// パネル外枠。`narrow` が `true` のパネルは
/// `data-blocks-media-object-narrow`（[`LAYOUT_CSS`] の `max-width: 20rem`
/// 規則）を追加で持つ。`stack`/`stack-full` は `@container` 分岐の実測幅
/// 確保用、`stretch`（Bugbot 指摘、イシュー #3228 PR #3402）は説明文を
/// 折り返させて行高を media の既定 6rem 角より高くし、`align-items:
/// stretch` が実際に media を伸長させる様子を可視化する用途で使う
/// （デモ幅が広いままだと 1 行に収まり `top` と見分けがつかない）。
fn panel(variant: &'static str, caption_text: &str, narrow: bool, item_node: Node) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![
        ("class", "blocks-media-object-panel"),
        ("data-blocks-media-object-variant", variant),
    ];
    if narrow {
        attrs.push(("data-blocks-media-object-narrow", ""));
    }
    div(attrs, vec![caption(caption_text), item_node])
}

/// `root`（item）パーツ。`variant` を `data-blocks-media-object-variant`
/// へ渡し、既定の整列（`top`）以外は [`LAYOUT_CSS`] の対応セレクタが上書く。
fn media_object_root(variant: &'static str, media: Node, content: Node) -> Node {
    item::root(
        ItemRootProps::default(),
        vec![
            ("data-blocks-media-object-root", ""),
            ("data-blocks-media-object-variant", variant),
        ],
        vec![media, content],
    )
}

/// `nested`（R1069）の入れ子部分。外側の `nowrap`/`align-items` 規則を
/// 継承させないため `data-blocks-media-object-root` は付与せず、media のみ
/// `-media-nested` フックを持つ（[`LAYOUT_CSS`] 参照）。
fn nested_media_object() -> Node {
    item::root(
        ItemRootProps::default(),
        vec![],
        vec![
            media_image(("data-blocks-media-object-media-nested", "")),
            body(
                "関連するアップデート",
                "同じ整列パターンを縮小して本文内に再利用した例です。",
            ),
        ],
    )
}

/// `top`（R1062・主参照）インスタンス。
fn top_instance() -> Node {
    panel(
        "top",
        "上揃え（基準形）",
        false,
        media_object_root(
            "top",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "新しいダッシュボードのレイアウト",
                "サイドバーと本文の比率を見直し、狭い画面での折り返しを改善しました。",
            ),
        ),
    )
}

/// `center`（R1063）インスタンス。
fn center_instance() -> Node {
    panel(
        "center",
        "縦中央揃え",
        false,
        media_object_root(
            "center",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "通知設定の見直し",
                "メール通知とアプリ内通知を個別に切り替えられるようにしました。",
            ),
        ),
    )
}

/// `bottom`（R1064）インスタンス。
fn bottom_instance() -> Node {
    panel(
        "bottom",
        "下揃え",
        false,
        media_object_root(
            "bottom",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "エクスポート機能の追加",
                "表示中のデータを CSV 形式でダウンロードできるようになりました。",
            ),
        ),
    )
}

/// `stretch`（R1065）インスタンス。
fn stretch_instance() -> Node {
    panel(
        "stretch",
        "画像を行高いっぱいに伸ばす",
        true,
        media_object_root(
            "stretch",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "検索結果の並べ替え",
                "更新日時・関連度・名前の 3 種類から並べ替え基準を選べます。狭いカラムで折り返すことで、media が行高いっぱいに伸びる様子を確認できます。",
            ),
        ),
    )
}

/// `right`（R1066）インスタンス。
fn right_instance() -> Node {
    panel(
        "right",
        "画像を右側に配置",
        false,
        media_object_root(
            "right",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "キーボードショートカットの追加",
                "よく使う操作をキーボードだけで完結できるようにしました。",
            ),
        ),
    )
}

/// `stack`（R1067）インスタンス。狭幅パネル（`@container`）で縦積みに
/// 折り返す。
fn stack_instance() -> Node {
    panel(
        "stack",
        "狭幅で縦積み",
        true,
        media_object_root(
            "stack",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "タグ機能のプレビュー",
                "アイテムに自由なタグを付けて絞り込めるようになります。",
            ),
        ),
    )
}

/// `stack-full`（R1068）インスタンス。狭幅パネルで縦積み + 画像を全幅化。
fn stack_full_instance() -> Node {
    panel(
        "stack-full",
        "狭幅で画像を全幅化",
        true,
        media_object_root(
            "stack-full",
            media_image(("data-blocks-media-object-media", "")),
            body(
                "共有リンクの有効期限",
                "共有リンクに有効期限を設定し、期限切れ後は自動で無効になります。",
            ),
        ),
    )
}

/// `nested`（R1069）インスタンス。本文内に小さい media object を入れ子。
fn nested_instance() -> Node {
    panel(
        "nested",
        "本文内に小さい media object を入れ子",
        false,
        media_object_root(
            "nested",
            media_image(("data-blocks-media-object-media", "")),
            body_with_extra(
                "リリースノート 2026.9",
                "今回のリリースでは以下の変更を行いました。",
                vec![nested_media_object()],
            ),
        ),
    )
}

/// `media-object` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// 主参照（R1062・top）を先頭に、8 インスタンスを縦に並べる。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-media-object-layout")],
        vec![
            top_instance(),
            center_instance(),
            bottom_instance(),
            stretch_instance(),
            right_instance(),
            stack_instance(),
            stack_full_instance(),
            nested_instance(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/media-object/",
    title: "media-object",
    category: BlockCategory::MediaObject,
    rust_source: "crates/docs-site/src/blocks/application/media_object/media_object_alignments.rs",
    demo_class: "blocks-media-object",
    parts: &[
        Part {
            label: "Item",
            path: "/themes/item/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `media_object_alignments` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。`item` recipe の既定
/// （`flex-wrap: wrap`・`align-items: center`・media `space-10` 角）に対し
/// 詳細度 (0,3,0) 以上で上書きする。`display: none` は使わない
/// （狭幅パターンも縦積みで内容を保つ）。
const LAYOUT_CSS: &str = "\
.blocks-media-object-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-media-object-panel {\n  container-type: inline-size;\n  container-name: blocks-media-object;\n}\n\
.blocks-media-object-panel[data-blocks-media-object-narrow] {\n  max-width: 20rem;\n}\n\
.blocks-media-object-caption {\n  margin: 0 0 var(--fandhe-space-2);\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-root] {\n  flex-wrap: nowrap;\n  align-items: flex-start;\n}\n\
[data-scope=\"item\"][data-part=\"media\"][data-blocks-media-object-media] {\n  width: 6rem;\n  height: 6rem;\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-variant=\"center\"] {\n  align-items: center;\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-variant=\"bottom\"] {\n  align-items: flex-end;\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-variant=\"stretch\"] {\n  align-items: stretch;\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-variant=\"stretch\"] > [data-scope=\"item\"][data-part=\"media\"][data-blocks-media-object-media] {\n  height: auto;\n  align-self: stretch;\n}\n\
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-variant=\"right\"] {\n  flex-direction: row-reverse;\n}\n\
[data-scope=\"item\"][data-part=\"media\"][data-blocks-media-object-media-nested] {\n  width: 2.5rem;\n  height: 2.5rem;\n}\n\
@container blocks-media-object (max-width: 30rem) {\n  \
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-variant=\"stack\"],\n  \
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-variant=\"stack-full\"] {\n    flex-direction: column;\n    align-items: stretch;\n  }\n  \
[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-variant=\"stack-full\"] > [data-scope=\"item\"][data-part=\"media\"][data-blocks-media-object-media] {\n    width: 100%;\n    height: 10rem;\n  }\n\
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
        assert!(html.contains("data-scope=\"item\""));
        assert!(html.contains("data-scope=\"image\""));
    }

    #[test]
    fn demo_wires_all_eight_variant_hooks() {
        let html = demo_html();
        for variant in [
            "top",
            "center",
            "bottom",
            "stretch",
            "right",
            "stack",
            "stack-full",
            "nested",
        ] {
            assert!(
                html.contains(&format!("data-blocks-media-object-variant=\"{variant}\"")),
                "missing variant hook: {variant}"
            );
        }
    }

    #[test]
    fn nested_variant_contains_inner_item() {
        let html = demo_html();
        // root item は 8 パネル分 + 入れ子 1 個 = 9 個以上。
        assert!(html.matches("data-part=\"root\"").count() >= 9);
        assert!(html.contains("data-blocks-media-object-media-nested"));
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("id=\""));
    }

    #[test]
    fn layout_css_is_safe_and_uses_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-media-object (max-width: 30rem)"));
        assert!(LAYOUT_CSS.contains("row-reverse"));
        assert!(LAYOUT_CSS.contains("align-self: stretch;"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    #[test]
    fn override_selectors_beat_item_recipe() {
        // recipe 既定（`flex-wrap: wrap`/`align-items: center`/media
        // `space-10` 角）に負けないことの回帰ガード（詳細度 (0,3,0)）。
        assert!(LAYOUT_CSS
            .contains("[data-scope=\"item\"][data-part=\"root\"][data-blocks-media-object-root]"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"item\"][data-part=\"media\"][data-blocks-media-object-media]"
        ));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-media-object-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-media-object-layout");
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn uses_shared_dummy_assets() {
        let html = demo_html();
        assert!(html.contains(super::dummy_assets::PRODUCT_SRC));
    }
}
