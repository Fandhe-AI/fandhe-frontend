# navbar-docs-site

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `input-group` /
`input` / `kbd` / `link` / `button` / `icon` / `tab-nav` の 8 部品を合成
した、ドキュメントサイト用の 1 段ナビバーです。Blocks セクションは新規
部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

検索トリガーの位置違い・リンクのボタン化という「項目配置の差」を、Demo
を増やさず次の 3 variant の並記で示します。

- **中央検索**: ロゴ + `navigation_menu` を左に、中央に検索欄
  （`input_group` + `kbd::group`「Ctrl」「K」のショートカットヒント）、
  右端にリポジトリ・テーマ切替・主操作ボタンを配置したもの
- **右寄せ検索・ボタン型トリガー**: ロゴ + `tab_nav`（現在ページあり）を
  左に、右端へボタン型の検索トリガー・リポジトリ・テーマ切替・主操作
  ボタンをまとめて配置したもの
- **狭幅（アイコン化）**: リンク・検索・リポジトリ・主操作をアイコン
  ボタン/アイコンリンクへ縮めたもの。ビューポート幅にもリサイズにも
  連動しない状態を固定した静的な variant です（実在 `href` を持つ
  `<a>` のままなので到達性は保たれます）

本 Demo は静的な表示例であり、`<form>` 要素を持たず、検索処理・送信・
永続化・認証を一切行いません。押しても何も起きません（無 JS 制約、
`docs/policy/intentional-non-adoption.md` §3.25）。ブランド名
（`Nimbus Docs`）は架空です。アイコンは自作の単純な幾何図形で、
lucide 等の著作物ではありません。

## Rust コード
```rust
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::FieldProps;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::kbd::{self, KbdProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::tab_nav;
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照。
/// `header_simple_bar::REPO` と同一 URL）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// docs サイトのトップページ（`site/index.md` が原稿、ロゴの遷移先）。
/// `linkcheck` は Demo 出力内の絶対パスを `base_path` 込みで解決するため、
/// `crate::blocks::marketing::footer::footer_link_columns` と同じく実在
/// するデプロイ先 URL を直接使う契約にする。
const HOME_URL: &str = "https://fandhe-ai.github.io/fandhe-frontend/";
/// docs サイト「Guide」の実在ページ（`site/nav.toml` `index_path = "/guides/"`）。
const GUIDE_URL: &str = "https://fandhe-ai.github.io/fandhe-frontend/guides/";
/// docs サイト「API Reference」の実在ページ（同 `index_path = "/api/"`）。
const API_REFERENCE_URL: &str = "https://fandhe-ai.github.io/fandhe-frontend/api/";
/// docs サイト「Themes」の実在ページ（同 `index_path = "/themes/"`）。
const THEMES_URL: &str = "https://fandhe-ai.github.io/fandhe-frontend/themes/";

/// ドキュメント系ナビ項目（value, label, href）。実在するデプロイ先 URL
/// のみを指す（上記定数群参照）。
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("guides", "ガイド", GUIDE_URL),
    ("api", "API", API_REFERENCE_URL),
    ("themes", "コンポーネント", THEMES_URL),
];

/// 自作の単純な幾何アイコン（塗り。装飾用途のため `label: None`）。
/// 閉じた領域（矩形・円等）を表す `d` にのみ使う。開いた線分（`M...L`
/// のみで閉じない部分パス）を混在させると、その部分は面積 0 で
/// `fill` が効かず描画されない（Bugbot 指摘、イシュー #2927 PR
/// レビュー）。線分を含むアイコンは [`stroke_icon`] を使うこと。
fn geo_icon(d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// 自作の単純な幾何アイコン（線画。装飾用途のため `label: None`）。
/// 光線・罫線のような開いた線分は `fill`（面積 0 で不可視）ではなく
/// `stroke` で描く必要があるため、[`geo_icon`] と分けて `fill="none"` +
/// `stroke="currentColor"` を明示する（虫眼鏡の柄・テーマ切替の陽光線・
/// ドキュメントリンクの罫線に使用）。
fn stroke_icon(d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.8"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 虫眼鏡アイコン（検索欄・検索ボタン共通）。柄（`m20 17-5.2-5.2` の
/// 開いた線分）は塗りでは不可視なため [`stroke_icon`] を使う。
fn search_icon() -> Node {
    stroke_icon("M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2")
}

/// ロゴ（幾何図形アイコン + 架空のブランド名テキスト。href はサイト
/// トップページ [`HOME_URL`]）。`compact` が `true` のときはワードマーク
/// テキストを描画せず、代わりに `aria-label` でアクセシブルネームを保つ
/// （`narrow` variant 専用。ワードマークがロゴ・検索・ドキュメント 3 件・
/// リポジトリ・テーマ切替・主操作の計 7 項目と並んで 24rem を超えて
/// はみ出していた、イシュー #2927 PR レビュー〔codex/Bugbot〕指摘）。
fn logo(compact: bool) -> Node {
    let mut attrs = vec![("data-blocks-navbar-docs-site-logo", "")];
    let mut children = vec![geo_icon("M4 4h16v16H4zM8 8h8v8H8z")];
    if compact {
        attrs.push(("aria-label", "Nimbus Docs"));
    } else {
        children.push(span(vec![], vec![text("Nimbus Docs")]));
    }
    link::root(HOME_URL, &LinkProps::default(), attrs, children)
}

/// メインナビ（`navigation_menu`。トリガー・パネルを持たない単純なリンク
/// 集合で、`header_simple_bar::nav` と同型）。
fn docs_nav_menu(aria_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        aria_label,
        vec![("data-blocks-navbar-docs-site-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            NAV_ITEMS
                .iter()
                .map(|(value, label, href)| {
                    navigation_menu::item(
                        OpenState::Closed,
                        false,
                        &props,
                        value,
                        vec![],
                        vec![navigation_menu::link(
                            href,
                            false,
                            vec![],
                            vec![text(*label)],
                        )],
                    )
                })
                .collect(),
        )],
    )
}

/// メインナビ（`tab_nav`。「見た目は tabs、意味論はナビゲーション」の
/// end-search variant 向け）。この Demo 自体は `/blocks/navbar-docs-site/`
/// に表示され [`NAV_ITEMS`] のいずれとも一致しないため、どの項目にも
/// `current` を立てない（閲覧中のページと食い違う `current` 表示は
/// 支援技術へ誤った現在位置を伝える、イシュー #2927 PR レビュー指摘）。
fn docs_tab_nav(aria_label: &str) -> Node {
    tab_nav::root(
        Size::Md,
        aria_label,
        vec![("data-blocks-navbar-docs-site-tabs", "")],
        NAV_ITEMS
            .iter()
            .map(|(_value, label, href)| tab_nav::link(href, false, vec![], vec![text(*label)]))
            .collect(),
    )
}

/// 検索ショートカットヒント（`kbd::group`「Ctrl」「K」）。
fn shortcut() -> Node {
    kbd::group(
        vec![],
        vec![
            kbd::kbd(&KbdProps::default(), vec![], vec![text("Ctrl")]),
            kbd::kbd(&KbdProps::default(), vec![], vec![text("K")]),
        ],
    )
}

/// 中央寄せの検索欄（`input_group` + `input`。可視ラベルは出さず
/// `aria-label` で代える、モジュール doc「使用部品」節）。
fn search_field(variant: &'static str) -> Node {
    let field_id = format!("blocks-navbar-docs-site-search-{variant}");
    let field = FieldProps {
        id: &field_id,
        ids: Default::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    input_group::root(
        &group_props,
        vec![("data-blocks-navbar-docs-site-search", "")],
        vec![
            input_group::addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![search_icon()],
            ),
            input::input(
                &InputProps::default(),
                &field,
                vec![
                    ("type", "search"),
                    ("aria-label", "ドキュメントを検索"),
                    ("autocomplete", "off"),
                    ("placeholder", "検索..."),
                ],
            ),
            input_group::addon(
                InputGroupAlign::InlineEnd,
                &group_props,
                vec![],
                vec![shortcut()],
            ),
        ],
    )
}

/// 右寄せのボタン型検索トリガー（end-search variant。押しても何も起きない
/// 静的表示）。
fn search_button() -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            ..ButtonProps::default()
        },
        vec![("data-blocks-navbar-docs-site-search-button", "")],
        vec![search_icon(), text("検索"), shortcut()],
    )
}

/// アイコンボタン型検索トリガー（`size` は呼び出し側の variant に合わせる。
/// `narrow` は他の全アイコン項目と揃えて [`Size::Sm`] を使い、[`logo`] の
/// doc コメントに記した 24rem 超過を避ける）。
fn search_icon_button(size: Size) -> Node {
    button::icon_button(
        &ButtonProps {
            size,
            ..ButtonProps::default()
        },
        "ドキュメントを検索",
        vec![("data-blocks-navbar-docs-site-search-button", "")],
        vec![search_icon()],
    )
}

/// 外部リポジトリへのアイコンリンク（`aria-label` で到達性を保つ）。
fn repo_link() -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![
            ("aria-label", "GitHub リポジトリを開く"),
            ("data-blocks-navbar-docs-site-repo", ""),
        ],
        vec![geo_icon(
            "M12 2C6.5 2 2 6.5 2 12c0 4.4 2.9 8.2 6.9 9.5.5.1.7-.2.7-.5v-1.7c-2.8.6-3.4-1.2-3.4-1.2-.5-1.1-1.1-1.4-1.1-1.4-.9-.6.1-.6.1-.6 1 .1 1.5 1 1.5 1 .9 1.5 2.3 1 2.9.8.1-.6.3-1 .6-1.3-2.2-.3-4.6-1.1-4.6-4.9 0-1.1.4-2 1-2.7-.1-.2-.4-1.2.1-2.5 0 0 .8-.3 2.7 1a9.2 9.2 0 0 1 4.9 0c1.9-1.3 2.7-1 2.7-1 .5 1.3.2 2.3.1 2.5.6.7 1 1.6 1 2.7 0 3.8-2.4 4.6-4.6 4.9.3.3.6.9.6 1.8v2.6c0 .3.2.6.7.5C19.1 20.2 22 16.4 22 12c0-5.5-4.5-10-10-10z",
        )],
    )
}

/// テーマ切替（サイト本体の `.docs-theme-toggle`（`site.js` が掴む唯一の
/// セレクタ）とは別の class/id を持つ、押しても何も起きない静的表示）。
/// 陽光線は開いた線分のため [`stroke_icon`] で描く（塗りでは不可視、
/// Bugbot 指摘）。`size` は [`search_icon_button`] と同じ理由で呼び出し側の
/// variant に合わせる。
fn theme_toggle(size: Size) -> Node {
    button::icon_button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size,
            ..ButtonProps::default()
        },
        "配色テーマを切り替え",
        vec![("data-blocks-navbar-docs-site-theme-toggle", "")],
        vec![stroke_icon(
            "M12 4V2M12 22v-2M4.9 4.9 3.5 3.5M20.5 20.5l-1.4-1.4M4 12H2M22 12h-2M4.9 19.1l-1.4 1.4M20.5 3.5l-1.4 1.4M12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10z",
        )],
    )
}

/// 主操作ボタン（押しても何も起きない静的表示）。
fn primary(size: Size) -> Node {
    button::button(
        &ButtonProps {
            size,
            ..ButtonProps::default()
        },
        vec![("data-blocks-navbar-docs-site-primary", "")],
        vec![text("はじめる")],
    )
}

/// 主操作のアイコンボタン版（narrow variant 専用。文言の代わりにロケット
/// 状の幾何アイコンを使う）。
fn primary_icon_button(size: Size) -> Node {
    button::icon_button(
        &ButtonProps {
            size,
            ..ButtonProps::default()
        },
        "はじめる",
        vec![("data-blocks-navbar-docs-site-primary", "")],
        vec![geo_icon("M12 2 4 20l8-4 8 4z")],
    )
}

/// `narrow` variant 専用のドキュメント系アイコンリンク（[`NAV_ITEMS`] の
/// 1 件を表す）。ページ罫線は開いた線分のため [`stroke_icon`] で描く
/// （塗りでは不可視、`docs_nav_menu`/`docs_tab_nav` と同じ 3 項目を
/// アイコン化しても到達性を落とさない、codex レビュー指摘: narrow が
/// ガイドのみ残し API/Themes への導線を落としていた）。
fn docs_icon_link(label: &str, href: &str) -> Node {
    let aria_label = format!("{label}を開く");
    link::root(
        href,
        &LinkProps::default(),
        vec![
            ("aria-label", aria_label.as_str()),
            ("data-blocks-navbar-docs-site-docs-link", ""),
        ],
        vec![stroke_icon("M4 3h16v18H4zM8 7h8M8 11h8M8 15h4")],
    )
}

/// キャプション行。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-navbar-docs-site-caption")],
        vec![text(label)],
    )
}

/// `center-search` variant（1 段バー: ロゴ + navigation_menu / 中央の
/// 検索欄 / リポジトリ + テーマ切替 + 主操作）。
fn center_search() -> Node {
    div(
        vec![
            ("data-blocks-navbar-docs-site-bar", ""),
            ("data-blocks-navbar-docs-site-variant", "center-search"),
        ],
        vec![
            div(
                vec![("data-blocks-navbar-docs-site-start", "")],
                vec![logo(false), docs_nav_menu("メイン（中央検索）")],
            ),
            div(
                vec![("data-blocks-navbar-docs-site-center", "")],
                vec![search_field("center-search")],
            ),
            div(
                vec![("data-blocks-navbar-docs-site-end", "")],
                vec![repo_link(), theme_toggle(Size::Md), primary(Size::Md)],
            ),
        ],
    )
}

/// `end-search` variant（1 段バー: ロゴ + tab_nav / 右寄せのボタン型検索
/// トリガー + リポジトリ + テーマ切替 + 主操作）。
fn end_search() -> Node {
    div(
        vec![
            ("data-blocks-navbar-docs-site-bar", ""),
            ("data-blocks-navbar-docs-site-variant", "end-search"),
        ],
        vec![
            div(
                vec![("data-blocks-navbar-docs-site-start", "")],
                vec![logo(false), docs_tab_nav("メイン（右寄せ検索）")],
            ),
            div(
                vec![("data-blocks-navbar-docs-site-end", "")],
                vec![
                    search_button(),
                    repo_link(),
                    theme_toggle(Size::Md),
                    primary(Size::Md),
                ],
            ),
        ],
    )
}

/// `narrow` variant（狭幅表示。ドキュメント系リンク・検索・リポジトリを
/// アイコンボタン/アイコンリンクへ縮める。ビューポート幅・リサイズには
/// 連動しない固定状態、モジュール doc「3 variant を 1 つの Demo に縦
/// 並記する」節参照）。ロゴ縮約・[`Size::Sm`] 統一（イシュー #2927 PR
/// レビュー指摘）だけでは 7 項目が `max-inline-size: 24rem`（[`LAYOUT_CSS`]）
/// に収まらないため、`[data-blocks-navbar-docs-site-end]` へ narrow 限定の
/// `flex-wrap: wrap` を適用し折り返しで幅超過を防ぐ（イシュー #2927 PR
/// レビュー再指摘）。
fn narrow() -> Node {
    div(
        vec![
            ("data-blocks-navbar-docs-site-bar", ""),
            ("data-blocks-navbar-docs-site-variant", "narrow"),
        ],
        vec![
            div(
                vec![("data-blocks-navbar-docs-site-start", "")],
                vec![logo(true)],
            ),
            div(
                vec![("data-blocks-navbar-docs-site-end", "")],
                [search_icon_button(Size::Sm)]
                    .into_iter()
                    .chain(
                        NAV_ITEMS
                            .iter()
                            .map(|(_, label, href)| docs_icon_link(label, href)),
                    )
                    .chain([
                        repo_link(),
                        theme_toggle(Size::Sm),
                        primary_icon_button(Size::Sm),
                    ])
                    .collect(),
            ),
        ],
    )
}

/// `navbar-docs-site` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。3 variant を縦に並べる（モジュール doc「3 variant を 1 つの
/// Demo に縦並記する」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-navbar-docs-site-stack", "")],
        vec![
            caption("中央検索"),
            center_search(),
            caption("右寄せ検索・ボタン型トリガー"),
            end_search(),
            caption("狭幅（アイコン化）"),
            narrow(),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照 R0151 は代表構成（中央検索）としてそのまま反映しています
- R0152 / R0153 / R0154 / R0156 は検索トリガーの位置・リンクのボタン化の
  差分であり、右寄せ検索・ボタン型トリガーの variant へ集約しています
- 配色違いは既存のテーマトークン（`--fandhe-*`）にそのまま吸収されるため、
  Demo は増やさず本メモでのみ言及します

関連情報:
[Navigation Menu](../themes/navigation-menu.md) /
[Input Group](../themes/input-group.md) /
[Input](../themes/input.md) /
[Kbd](../themes/kbd.md) /
[Link](../themes/link.md) /
[Button](../themes/button.md) /
[Icon](../themes/icon.md) /
[Tab Nav](../themes/tab-nav.md)
