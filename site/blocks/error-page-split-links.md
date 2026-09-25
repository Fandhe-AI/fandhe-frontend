# error-page-split-links

`empty-state` / `heading` / `text` / `item` / `icon` の 5 部品を合成した、
本文 + 案内リンクの 2 カラム 404 です。`64rem` 以上の幅では左に「タグ
ライン → 大見出し → 説明文」を縦に積み、右に「ホーム」「ガイド」
「examples」への案内リンクを縦に並べます。各行はアイコン・ラベル・
説明文をまとめた複合行で、行全体がクリック対象です（ボタンは使いません）。
狭い幅では 1 カラムになり、本文の下にリンク群が続きます。

イシュー本文は使用部品の候補に `link` を挙げていますが、案内リンクの
行全体をクリック対象にするには `item::root` の `href` 経路で組む必要が
あり、その `<a>` の中へさらに `link::root`（同じく `<a>` を描画する）を
入れ子にすると不正な HTML になるため、`link` は使わず `item` の `href`
経路だけで構成しました。

遷移先はすべて実在する docs サイト内のページです（ホーム・ガイド・
examples）。文言はすべて架空のダミーです。`<form>` は使わず、状態変更・
送信処理は一切行いません。

主参照・集約元はともに対応表 ID R0583 です。

## Rust コード

```rust
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
```

## 原案差分メモ

参照元（対応表 ID R0583）は全画面高さの 2 カラム 404 で、左に本文、右に
行き先の案内リストを配置します。集約元も同じ R0583 のため、Demo の並記は
不要です。

- 参照元の全画面高さの表示は、Demo 枠内の `min-height`（`22rem`）へ変え
  ました。
- アイコンは外部アイコンセットのパスデータを転記せず、自作の単純な線画
  （家・書類・並んだ枠・シェブロン）にしました。
- 遷移先は実在する docs サイト内のページ（ホーム・ガイド・examples）に
  しました。参照元の 3 件目「ブログ」に相当するセクションは docs サイトに
  無いため、代わりに examples を使いました。
- イシュー候補の `link` 部品ではなく、`item::root` の `href` 経路で行全体を
  クリック対象にしました（`<a>` の入れ子を避けるため）。
- `empty_state::actions`（ボタン群スロット）は使わず、案内リンクは右カラム
  の `item::group` として独立させました。
- 文言・配色は本リポジトリの既存トークン（`--fandhe-*`）と独自の英語
  ダミーへ書き直しました。
