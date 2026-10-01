//! `navbar-docs-site` block（イシュー #2927。Application / Navbar
//! カテゴリ）。
//!
//! ドキュメントサイト用のナビバー。左にロゴとドキュメント系リンク、
//! 検索トリガー、右端に外部リポジトリへのリンク・テーマ切替・主操作
//! ボタンを組み合わせた合成例（`narrow` variant の右端項目群のみ、幅
//! 超過時に折り返す。詳細は下記 `narrow` 行と [`narrow`] 関数 doc 参照）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `input-group` / `input` / `kbd` / `link` / `button` /
//! `icon` / `tab-nav` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。新しい UI 部品は作らない。`field` は `parts` に含めない
//! （`FieldProps` 型を [`input::input`] 呼び出しに使うのみで、可視ラベルは
//! 出さず `aria-label` で代える。`app_shell_sidebar_header` と同型の判断）。
//!
//! # 3 variant を 1 つの Demo に縦並記する
//!
//! 検索トリガーの位置違い・リンクのボタン化という「項目配置の差」を、
//! Demo を増やさず 3 variant の並記で読み取れるようにする（対応表 ID の
//! 対応は `site/blocks/navbar-docs-site.md` の「集約元との差分メモ」節、
//! 対応表 ID のみを記す契約）。
//!
//! | variant | 左 | 検索 | 右端 |
//! |---|---|---|---|
//! | `center-search` | ロゴ + `navigation_menu` | 中央の `input_group`（検索欄 + `kbd::group`「Ctrl」「K」） | リポジトリ + テーマ切替 + 主操作 |
//! | `end-search` | ロゴ + `tab_nav`（`current` はどの項目にも立てない。下記 [`docs_tab_nav`] doc 参照） | 右寄せのボタン型検索トリガー（`button` + `kbd::group`） | リポジトリ + テーマ切替 + 主操作 |
//! | `narrow` | ロゴのみ | アイコンボタンに縮めた検索トリガー | [`nav_items`] 3 件（ガイド・API・コンポーネント）を個別アイコンリンク化 + リポジトリ・テーマ切替・主操作（すべてアイコンボタン、`Size::Sm`） |
//!
//! `narrow` はビューポート幅にもリサイズにも連動しない、**状態を固定した
//! 静的な variant** である（`app_shell_sidebar_header` の narrow と同じ
//! 判断）。リンクはアイコン化しても実在 `href` を持つ `<a>` のままなので、
//! 到達性は保たれる（レスポンシブでの自動縮小・非表示は行わない。狭幅で
//! ナビへ到達できなくなる回帰を構造的に避ける、
//! `header_simple_bar`/`header_floating_pill` と同じ教訓）。
//!
//! # 検索 input はどこにも送信しない・`<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` を出力しない。
//! 検索欄・検索ボタン・テーマ切替・主操作ボタンはいずれも静的な初期状態を
//! 表示するのみで、検索処理・送信・永続化・認証は行わない
//! （`docs/policy/intentional-non-adoption.md` §3.25）。ブランド名は架空
//! （実企業名・実クレデンシャル・PII を含まない）。
//!
//! # 無 JS のため検索ボタン・テーマ切替・主操作ボタンは disabled 固定
//!
//! 検索欄本体（`input_group`/`input`）は文字入力自体は可能な素の
//! text input のため `disabled: false` のまま保つが、押しても何も起きない
//! ボタン（[`search_button`]/[`search_icon_button`]/[`theme_toggle`]/
//! [`primary`]/[`primary_icon_button`]）はいずれも `disabled: true` で
//! ネイティブ `disabled`・`data-disabled`・`aria-disabled="true"` を付与し、
//! 無 JS であることを支援技術・視覚の双方に明示する（`navbar_with_search`
//! の通知ボタン・アバターメニュー trigger と同じ判断）。[`LAYOUT_CSS`] の
//! `[data-disabled]` 複合セレクタで opacity/cursor を通常のボタンと同じ
//! 見た目に中和する（Cursor Bugbot 指摘、イシュー #2927 PR #3373 レビュー）。
//!
//! # `site.js` のセレクタと衝突しない
//!
//! `crates/docs-site` の `assets/site.js`（`src/script.rs`）は
//! `.docs-theme-toggle` / `.docs-search-input` / `.docs-search` /
//! `#docs-search-results` を `document.querySelector` で掴む。本 Demo は
//! これらの class/id を一切使わない（使うと本物のサイト JS が Demo 要素へ
//! 誤って配線されてしまう）。CSS フックはすべて
//! `data-blocks-navbar-docs-site-*` 属性、素の `div`/`p` のみ
//! `.blocks-navbar-docs-site-*` クラスで配置する。
//!
//! # `id`/`aria-label` は variant ごとに一意にする
//!
//! 3 variant を 1 Demo へ並記するため、検索 input の id・ナビ/タブの
//! `aria-label` はすべて variant 名の suffix で分ける
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # アイコンは自作の単純幾何図形
//!
//! lucide 等の実アイコンセット由来の path は複製せず、単純な矩形/円/線の
//! 幾何図形を自作する（装飾用途のため `label: None`）。
//!
//! # href の方針
//!
//! ドキュメント系リンクはサイト内に実在する `/guides/`/`/api/`/`/themes/`、
//! ロゴは `/`、外部リポジトリは実在の自リポジトリ URL（`header_simple_bar`
//! の `REPO` 定数と同じ URL）。`href="#"` は使わない（`linkcheck` が拒否
//! する死リンクのため）。
//!
//! # CSS フックの選び方
//!
//! `navigation_menu`/`input_group`/`input`/`kbd`/`link`/`button`/`icon`/
//! `tab_nav` の各パーツは `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-navbar-docs-site-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! styled パーツの `[data-scope][data-part]` base 宣言（詳細度 `(0,2,0)`）
//! に対抗できるよう同じ属性を含む複合セレクタで対応する（
//! `header_simple_bar` の「CSS フックに data 属性を使う理由」節と同じ
//! 教訓）。素の `div`/`p` には `class` がそのまま効くため、配置は
//! `.blocks-navbar-docs-site-*` クラスセレクタで行う。DOM 順は「左 →
//! 検索 → 右端」で固定し `order` は使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// 実在の自リポジトリ URL（サイト内ページではなく GitHub 上の外部
/// リポジトリを指すため `base_path` の対象外。`href` の方針、モジュール
/// doc 参照。`header_simple_bar::REPO` と同一 URL）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// docs サイトのトップページ（`site/index.md` が原稿、ロゴの遷移先）。
///
/// [`crate::blocks::Block::demo`] は `fn() -> Node` のため実行時引数を
/// 受け取れないが、[`crate::blocks::demo_base_path`]（`insert_generated_sections`
/// が `(block.demo)()` 呼び出し直前にセットする thread-local）から
/// `crate::build::build_site` へ実際に渡された `base_path` を読む。
/// コンパイル時に埋め込んだ `site/nav.toml` から `base_path` だけを
/// 抜き出す従来方式は、`build_site` へ渡す `repo_root`/`base_path` が
/// docs-site 自身のソースツリーと異なる場合に食い違い得たため廃止した
/// （イシュー #2927 PR #3373 レビュー〔codex〕指摘）。
fn home_url() -> String {
    crate::layout::asset_href(&crate::blocks::demo_base_path(), "/")
}
/// docs サイト「Guide」の実在ページ（`site/nav.toml` `index_path = "/guides/"`）。
fn guide_url() -> String {
    crate::layout::asset_href(&crate::blocks::demo_base_path(), "/guides/")
}
/// docs サイト「API Reference」の実在ページ（同 `index_path = "/api/"`）。
fn api_reference_url() -> String {
    crate::layout::asset_href(&crate::blocks::demo_base_path(), "/api/")
}
/// docs サイト「Themes」の実在ページ（同 `index_path = "/themes/"`）。
fn themes_url() -> String {
    crate::layout::asset_href(&crate::blocks::demo_base_path(), "/themes/")
}

/// ドキュメント系ナビ項目（value, label, href）。`base_path` を反映した
/// サイト内 root-relative href のみを指す（上記関数群参照）。
fn nav_items() -> Vec<(&'static str, &'static str, String)> {
    vec![
        ("guides", "ガイド", guide_url()),
        ("api", "API", api_reference_url()),
        ("themes", "コンポーネント", themes_url()),
    ]
}

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

/// [`geo_icon`] の変種。内側に別の閉じた矩形/円を重ねて「くり抜き」を
/// 表現する `d`（例: ロゴの外枠 + 内側正方形）向けに `fill-rule="evenodd"`
/// を付与する。既定の `fill-rule="nonzero"` では同方向に描いた 2 つの
/// 閉領域が単純に合算され内側が塗りつぶされたまま見えなくなるため
/// （Bugbot 指摘、イシュー #2927 PR #3373 レビュー）、[`geo_icon`] とは
/// 分けて明示する。単一閉領域の [`geo_icon`] 呼び出し（例: GitHub
/// アイコン・三角形）には影響しない。
fn geo_icon_evenodd(d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", d), ("fill-rule", "evenodd")], vec![])],
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
/// トップページ [`home_url`]）。`compact` が `true` のときはワードマーク
/// テキストを描画せず、代わりに `aria-label` でアクセシブルネームを保つ
/// （`narrow` variant 専用。ワードマークがロゴ・検索・ドキュメント 3 件・
/// リポジトリ・テーマ切替・主操作の計 7 項目と並んで 24rem を超えて
/// はみ出していた、イシュー #2927 PR レビュー〔codex/Bugbot〕指摘）。
fn logo(compact: bool) -> Node {
    let mut attrs = vec![("data-blocks-navbar-docs-site-logo", "")];
    let mut children = vec![geo_icon_evenodd("M4 4h16v16H4zM8 8h8v8H8z")];
    if compact {
        attrs.push(("aria-label", "Nimbus Docs"));
    } else {
        children.push(span(vec![], vec![text("Nimbus Docs")]));
    }
    link::root(&home_url(), &LinkProps::default(), attrs, children)
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
            nav_items()
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
/// に表示され [`nav_items`] のいずれとも一致しないため、どの項目にも
/// `current` を立てない（閲覧中のページと食い違う `current` 表示は
/// 支援技術へ誤った現在位置を伝える、イシュー #2927 PR レビュー指摘）。
fn docs_tab_nav(aria_label: &str) -> Node {
    tab_nav::root(
        Size::Md,
        aria_label,
        vec![("data-blocks-navbar-docs-site-tabs", "")],
        nav_items()
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

/// 右寄せのボタン型検索トリガー（end-search variant。無 JS のため
/// `disabled: true` 固定で押しても何も起きないことを明示する。
/// `navbar_with_search::notify` と同じ判断）。
fn search_button() -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-navbar-docs-site-search-button", "")],
        vec![search_icon(), text("検索"), shortcut()],
    )
}

/// アイコンボタン型検索トリガー（`size` は呼び出し側の variant に合わせる。
/// `narrow` は他の全アイコン項目と揃えて [`Size::Sm`] を使い、[`logo`] の
/// doc コメントに記した 24rem 超過を避ける）。[`search_button`] と同じ
/// 理由で `disabled: true` 固定。
fn search_icon_button(size: Size) -> Node {
    button::icon_button(
        &ButtonProps {
            size,
            disabled: true,
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
/// セレクタ）とは別の class/id を持つ、無 JS のため `disabled: true` 固定
/// で押しても何も起きないことを明示する静的表示）。
/// 陽光線は開いた線分のため [`stroke_icon`] で描く（塗りでは不可視、
/// Bugbot 指摘）。`size` は [`search_icon_button`] と同じ理由で呼び出し側の
/// variant に合わせる。
fn theme_toggle(size: Size) -> Node {
    button::icon_button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size,
            disabled: true,
            ..ButtonProps::default()
        },
        "配色テーマを切り替え",
        vec![("data-blocks-navbar-docs-site-theme-toggle", "")],
        vec![stroke_icon(
            "M12 4V2M12 22v-2M4.9 4.9 3.5 3.5M20.5 20.5l-1.4-1.4M4 12H2M22 12h-2M4.9 19.1l-1.4 1.4M20.5 3.5l-1.4 1.4M12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10z",
        )],
    )
}

/// 主操作ボタン（無 JS のため `disabled: true` 固定で押しても何も起きない
/// ことを明示する静的表示）。
fn primary(size: Size) -> Node {
    button::button(
        &ButtonProps {
            size,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-navbar-docs-site-primary", "")],
        vec![text("はじめる")],
    )
}

/// 主操作のアイコンボタン版（narrow variant 専用。文言の代わりにロケット
/// 状の幾何アイコンを使う）。[`primary`] と同じ理由で `disabled: true` 固定。
fn primary_icon_button(size: Size) -> Node {
    button::icon_button(
        &ButtonProps {
            size,
            disabled: true,
            ..ButtonProps::default()
        },
        "はじめる",
        vec![("data-blocks-navbar-docs-site-primary", "")],
        vec![geo_icon("M12 2 4 20l8-4 8 4z")],
    )
}

/// `narrow` variant 専用のドキュメント系アイコンリンク（[`nav_items`] の
/// 1 件を表す）。ページ罫線は開いた線分のため [`stroke_icon`] で描く
/// （塗りでは不可視、`docs_nav_menu`/`docs_tab_nav` と同じ 3 項目を
/// アイコン化しても到達性を落とさない、codex レビュー指摘: narrow が
/// ガイドのみ残し API/Themes への導線を落としていた）。
///
/// `key`（[`nav_items`] の値）ごとに固有アイコンへ切り替える（Bugbot
/// 指摘、イシュー #2927 PR #3373 レビュー: 3 リンクが同一アイコン・
/// 可視テキストなしで行き先を判別できなかった）。`aria-label` は
/// 支援技術向けの名前を保つのみで、視覚的な判別はアイコン形状の違いが
/// 担う。
fn docs_icon_link(key: &str, label: &str, href: &str) -> Node {
    let aria_label = format!("{label}を開く");
    let icon_node = match key {
        // API: 山括弧（コードの意匠、開いた線分のため stroke で描く）。
        "api" => stroke_icon("M8 6 3 12l5 6M16 6l5 6-5 6"),
        // コンポーネント（Themes）: 2x2 のスウォッチ格子（閉じた矩形 4 件、塗り）。
        "themes" => geo_icon("M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z"),
        // ガイド: 罫線付きページ（既定、開いた線分のため stroke で描く）。
        _ => stroke_icon("M4 3h16v18H4zM8 7h8M8 11h8M8 15h4"),
    };
    link::root(
        href,
        &LinkProps::default(),
        vec![
            ("aria-label", aria_label.as_str()),
            ("data-blocks-navbar-docs-site-docs-link", ""),
        ],
        vec![icon_node],
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
                        nav_items()
                            .iter()
                            .map(|(key, label, href)| docs_icon_link(key, label, href)),
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/navbar-docs-site/",
    title: "navbar-docs-site",
    category: BlockCategory::Navbar,
    rust_source: "crates/docs-site/src/blocks/application/navbar/navbar_docs_site.rs",
    demo_class: "blocks-navbar-docs-site",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
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
            label: "Kbd",
            path: "/themes/kbd/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Tab Nav",
            path: "/themes/tab-nav/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（`docs/design/docs-site-blocks-section.md` §10 追記節に
/// 従い、並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突を避け
/// 本モジュール側の定数へ分離する）。`--fandhe-*` トークンのみを使う。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-navbar-docs-site {\n  padding: 0;\n  overflow-x: auto;\n}\n\
[data-blocks-navbar-docs-site-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-navbar-docs-site-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-navbar-docs-site-bar] {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  background: var(--fandhe-color-bg);\n  min-inline-size: 52rem;\n}\n\
[data-blocks-navbar-docs-site-variant=\"narrow\"] {\n  min-inline-size: 0;\n  max-inline-size: 24rem;\n}\n\
[data-blocks-navbar-docs-site-start] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-navbar-docs-site-center] {\n  flex: 1 1 auto;\n  display: flex;\n  justify-content: center;\n}\n\
[data-scope=\"input-group\"][data-part=\"root\"][data-blocks-navbar-docs-site-search] {\n  inline-size: 100%;\n  max-inline-size: 20rem;\n}\n\
[data-blocks-navbar-docs-site-end] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-inline-start: auto;\n}\n\
[data-blocks-navbar-docs-site-variant=\"narrow\"] [data-blocks-navbar-docs-site-end] {\n  flex-wrap: wrap;\n  justify-content: flex-end;\n  row-gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-navbar-docs-site-logo] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-semibold, 600);\n  white-space: nowrap;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-docs-site-search-button][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-docs-site-theme-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-docs-site-primary][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// 8 部品すべてが Demo 出力に含まれること。
    #[test]
    fn demo_composes_all_eight_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"kbd\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"tab-nav\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 非対話制約（`<form>` なし・`href="#"` なし・`data:` src なし）を
    /// 満たし、ボタンは `type="button"` であること。
    #[test]
    fn demo_has_no_form_no_dead_link_no_data_uri() {
        let html = render(&demo());
        assert!(!html.contains("<form"), "demo should never contain <form>");
        assert!(
            !html.contains("href=\"#\""),
            "demo should never contain a dead href=\"#\" link"
        );
        assert!(
            !html.contains("src=\"data:"),
            "demo should never contain a data: URI"
        );
        assert!(
            html.contains("type=\"button\""),
            "buttons should be type=\"button\""
        );
    }

    /// 3 variant がそれぞれちょうど 1 回出ること。
    #[test]
    fn demo_contains_each_variant_exactly_once() {
        let html = render(&demo());
        for variant in ["center-search", "end-search", "narrow"] {
            let needle = format!("data-blocks-navbar-docs-site-variant=\"{variant}\"");
            assert_eq!(
                html.matches(&needle).count(),
                1,
                "variant {variant} should appear exactly once"
            );
        }
    }

    /// サイト本体の `site.js` が掴むセレクタ（`.docs-theme-toggle`/
    /// `.docs-search-input`/`.docs-search`/`#docs-search-results`）を
    /// 一切含まないこと（モジュール doc「`site.js` のセレクタと衝突
    /// しない」節参照）。
    #[test]
    fn demo_does_not_reuse_site_js_selectors() {
        let html = render(&demo());
        for needle in [
            "docs-theme-toggle",
            "docs-search-input",
            "\"docs-search\"",
            "docs-search-results",
        ] {
            assert!(
                !html.contains(needle),
                "demo should not reuse the site.js hook {needle}"
            );
        }
    }

    /// 無 JS の no-op ボタン（検索・テーマ切替・主操作）が `data-disabled`
    /// を持ち、実在の検索 input は disabled にならないこと
    /// （Cursor Bugbot 指摘、イシュー #2927 PR #3373 レビュー）。
    #[test]
    fn no_op_buttons_are_disabled_but_search_input_is_not() {
        for html in [
            render(&search_button()),
            render(&search_icon_button(Size::Md)),
            render(&theme_toggle(Size::Md)),
            render(&primary(Size::Md)),
            render(&primary_icon_button(Size::Md)),
        ] {
            assert!(
                html.contains("data-disabled"),
                "no-op button should carry data-disabled: {html}"
            );
        }
        let search_field_html = render(&search_field("center-search"));
        assert!(
            !search_field_html.contains("data-disabled"),
            "the real search input should remain enabled"
        );
    }

    /// [`crate::blocks::insert_generated_sections`] が渡す `base_path` が
    /// このブロックのロゴ・ナビリンクへ反映されること（ビルド対象の
    /// `base_path` と食い違わない、イシュー #2927 PR #3373 レビュー
    /// 〔codex〕指摘の回帰）。
    #[test]
    fn demo_links_follow_the_base_path_passed_by_build_site() {
        let html_with_prefix = render(&fandhe_frontend_core::div(
            vec![],
            crate::blocks::insert_generated_sections(BLOCK.path, "/prefix", vec![]),
        ));
        assert!(
            html_with_prefix.contains("href=\"/prefix/\""),
            "home link should honor the base_path passed to insert_generated_sections"
        );
        assert!(
            html_with_prefix.contains("href=\"/prefix/guides/\""),
            "guide link should honor the base_path passed to insert_generated_sections"
        );

        let html_without_prefix = render(&fandhe_frontend_core::div(
            vec![],
            crate::blocks::insert_generated_sections(BLOCK.path, "", vec![]),
        ));
        assert!(
            html_without_prefix.contains("href=\"/\""),
            "home link should fall back to root when base_path is empty"
        );
    }
}
