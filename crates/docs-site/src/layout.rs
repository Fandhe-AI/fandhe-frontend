//! docs サイトの 3 カラムページ骨格（左ナビ / 中央コンテンツ / 右目次）。
//!
//! タイトル・サイドバー・本文の各 [`Node`] から、DOCTYPE を除いた完全な
//! HTML 文書 `Node`（`<html>` 要素）を組み立てる。生成した `Node` は
//! `fandhe_frontend_server::ssg::generate_pages()`（`crates/server/src/ssg.rs`）が
//! `<!DOCTYPE html>` を前置して書き出す契約であり、本モジュールは
//! DOCTYPE を出力しない（後続イシュー #470 がビルドエントリで接続する）。
//!
//! 骨格は `docs/design/docs-site-three-column-redesign.md` §3.1 の DOM/class
//! 契約に従い、`div.docs-container` 配下に `aside.docs-sidebar`（左ナビ。
//! `nav_list` 本体の markup は変更しない。`< 768px` では非表示で、代わりに
//! ヘッダー末尾の checkbox hack（`input.docs-nav-drawer-toggle` + `label` +
//! `nav.docs-nav-drawer`、全セクションへ移れるナビ drawer、イシュー #3674、
//! JS 不要）が担う）・`main.docs-main`（中央コンテンツ、`article.docs-content`
//! を内包）・見出しが存在するページのみ第 3 子として出現する
//! `aside.docs-toc-aside`（右目次、内側に `nav.docs-toc` をそのまま配置。
//! `nav.docs-toc` は `h2.docs-toc-title`（"On this page"）を先頭に持ち、
//! `aria-labelledby` で自身に紐付ける。見出しレベルは `TOC_MAX_LEVEL`
//! を超えるものを除外し、最大 2 段（`h2`/`h3`）で固定する。現在地
//! ハイライト（`aria-current="location"`）は
//! `crate::script::SITE_JS` のみが実行時に付与し、SSG が出力する静的
//! markup には含めない。イシュー #950）
//! の最大 3 カラムを出力する（イシュー #907）。breakpoint による表示制御
//! （狭幅で目次列→ナビ列の順に畳む）は構造 CSS（`crate::site_theme` が
//! 生成する `assets/site.css`）側の責務であり、本モジュールは DOM 順・
//! class 名の契約のみを担う。見出しが存在しないページでは
//! `div.docs-container` に `docs-container--no-toc` 修飾 class を付与し、
//! 3 カラム帯域（`min-width: 1200px`）で右目次列のグリッドトラックを
//! 収縮させる（`crate::site_theme::STRUCTURAL_CSS` 参照、Bugbot 指摘 #916
//! 是正）。
//!
//! 右目次カラムは `min-width: 1200px` 未満で構造 CSS が `display: none` に
//! 切り替えるため、タブレット・モバイル幅では到達手段が失われる（この事象
//! 自体は `docs/reports/docs-site-redesign-regression-report.md` §3.2/§10.1 で
//! 許容判定済み）。本モジュールはその判定を維持したまま、`main.docs-main`
//! の第 1 子（かつ SkipNav のスキップ先ターゲットより前）に見出しがある
//! ページのみ折りたたみ目次 `nav.docs-toc-inline > details`（[`toc_inline`]）
//! を出力し、`< 1200px` での JS 非依存な代替到達手段とする（イシュー
//! #1080）。`>= 1200px` では構造 CSS 側で非表示に切り替わり右目次カラムと
//! 重複しない。`class="docs-toc"` を共有しない不変条件は [`toc_inline`]
//! rustdoc 参照。
//!
//! `fandhe_frontend_app::page_shell` との差分: `page_shell` は
//! `/static/style.css` と `hydrate.js` をハードコードした `String` を返す
//! CSR/SSR 向けの実装であり docs には流用できないため、本モジュールは
//! `base_path` を考慮したアセット参照（[`asset_href`]）を持つ `Node` 返却の
//! 別実装として新規に用意する。docs サイトはハイドレーションを行わない
//! （`data-hydrate`/`data-bind-*` 束縛点を持たない）が、テーマトグル
//! （ダーク/ライト切替）・GitHub リンクのため `<head>` に FOUC 抑止の
//! 同期の `<script src>`（[`crate::script::THEME_INIT_REL_PATH`]、stylesheet より前）と
//! `<script src>`（[`crate::script::SCRIPT_REL_PATH`]、`defer`）を含める
//! （イシュー #951。旧「JS を含めない」宣言はこの変更で終了した）。
//!
//! `div.docs-header-actions` の第 1 子として検索ブロック
//! （`div.docs-search`）を無条件出力する（イシュー #958）。`input.docs-search-input`
//! の `data-search-index` 属性が [`search_index::REL_PATH`] を [`asset_href`]
//! 経由で参照し、`crate::script::SITE_JS` の第 3 IIFE が初回フォーカス時に
//! `fetch()` する唯一の実装点となる（インデックス JSON 自体は本モジュールが
//! HTML へインライン化しない、`crate::search_index` モジュール doc の
//! セキュリティ不変条件参照）。#3672 以降は、ダイアログを開いた時点で明示的に
//! 読む。入力欄と結果一覧は `dialog#docs-search-dialog` の中に置き、ヘッダーには
//! 検索ボタンのみを置く。検索ブロック・結果一覧は既定 `hidden` とし、
//! `SITE_JS` が配線完了後にのみ可視化する（`.docs-theme-toggle` と同型の
//! progressive enhancement 契約、`crate::script` モジュール doc 手順 5 参照）。
//! `<form>` で包まない（JS 無効時に Enter キーでのフォーム送信を誘発しない
//! ため）。`input.docs-search-input` の直前に視覚上のみ clip で隠す
//! `label.docs-search-label`（`for="docs-search-input"`）を置く
//! （fandhe-backend の docs サイトとデザインを統一するための追加、
//! `crate::site_theme::STRUCTURAL_CSS` の `.docs-search-label` 参照）。

use std::collections::HashSet;

use fandhe_frontend_core::{
    a, article, aside, div, el, h2, header, li, main_tag, nav, text, ul, Node,
};
use fandhe_frontend_pre_styled_ui::{
    badge as ps_badge, button as ps_button, icon as ps_icon, input_group as ps_input_group,
    kbd as ps_kbd, link as ps_link, skip_nav as ps_skip_nav, Size,
};

use crate::script;
use crate::search_index;

/// GitHub リポジトリへの絶対 URL（ヘッダーの GitHub リンクが参照する
/// 単一実装点）。`site/nav.toml` の `[site].repository_url` で上書きできる（#3720）。
///
/// 旧方針「`[site]` スキーマは拡張しない」は #3715 で見直した。外部リポジトリから
/// docs-site を使えるよう、`[site]` へ任意キー（`brand` / `repository_url` ほか）を
/// 足す設計を `docs/design/docs-site-external-use.md` に記録している。本定数は
/// `repository_url` 未指定時の既定値になる（実装済み、#3720。指定は [`SiteChrome`] 経由）。ライセンス本文への
/// リンクと帰属表記は設定で変えない（同文書の帰属表記の節）。
pub(crate) const REPOSITORY_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// ヘッダーのブランド名の既定値（`[site].brand` 未指定時、#3720）。
pub(crate) const DEFAULT_BRAND: &str = "fandhe-frontend";

/// `[site]` 由来でヘッダーへ渡す設定値（#3720）。`crate::nav::Site::chrome` が既定値解決済みで
/// 作り、`docs_page_with_chrome` が受ける。`Default` は fandhe-frontend 自身の値
/// （未指定時の出力を現行とバイト一致させる）。後続の任意キー追加時の拡張点。
#[derive(Debug, Clone, Copy)]
pub struct SiteChrome<'a> {
    /// `a.docs-brand` の可視テキスト（`text()` 経由でエスケープされる）。
    pub brand: &'a str,
    /// ヘッダー GitHub リンクの href（`parse_nav` が `https://` を検証済み）。
    pub repository_url: &'a str,
}

impl Default for SiteChrome<'_> {
    fn default() -> Self {
        Self {
            brand: DEFAULT_BRAND,
            repository_url: REPOSITORY_URL,
        }
    }
}

/// pre-styled-ui のアイコン用に `path` 1 本の SVG 子ノードを作る。
fn icon_path(d: &'static str) -> Node {
    el("path", vec![("d", d)], vec![])
}

/// ブランドリンク（`a.docs-brand`、#908/#3606）。favicon と同じ図案
/// （[`crate::favicon::mark_node`]）を `span[aria-hidden]` で包んで置く。
/// `mark_node` 自身が `role="img" aria-label` を持つため、可視テキストと
/// 二重に読み上げさせないようラッパーで隠す（favicon.rs の想定どおりの使い方）。
/// `a.docs-brand` が `header-inner` の第 1 子である順序契約は呼び出し側が守る。
fn brand(root_href: &str, brand_text: &str) -> Node {
    a(
        vec![("href", root_href), ("class", "docs-brand")],
        vec![
            el(
                "span",
                vec![("class", "docs-brand-mark"), ("aria-hidden", "true")],
                vec![crate::favicon::mark_node()],
            ),
            text(brand_text),
        ],
    )
}

/// ブランド横のバージョン badge（#3606）。表示は `core v{version}`
/// （`fandhe-frontend-core` の版数、選定理由は
/// `docs/design/docs-site-styled-blocks-redesign.md` §3.2）。版数の取得に失敗した
/// 場合は `None`（badge を出さない fail-closed、[`crate::site_version`] 参照）。
/// `a.docs-brand` の内側に入れない（リンクの名前に版数を混ぜないため）。
fn brand_version() -> Option<Node> {
    let version = crate::site_version::core_version()?;
    let label = format!("core v{version}");
    Some(el(
        "span",
        vec![("class", "docs-brand-version")],
        vec![ps_badge::badge(
            &ps_badge::BadgeProps {
                size: Size::Sm,
                ..ps_badge::BadgeProps::default()
            },
            vec![],
            vec![text(&label)],
        )],
    ))
}

/// 検索ブロック（イシュー #958/#3606/#3672）。ヘッダーには検索ボタンだけを置き、
/// 入力欄と結果一覧は `<dialog>`（`showModal()` で開くモーダル）の中へ置く。
///
/// `div.docs-search` は検索機能全体の可視化ゲートで、既定 `hidden`。JS 無効時・
/// `site.js` 読み込み失敗時・`showModal` 非対応時は、ボタンもダイアログも出ない
/// （無 JS では開けない UI を見せない）。`<form>` で包まない（Enter 送信させない）。
///
/// ボタンの `aria-haspopup="dialog"` / `aria-expanded` は SSR では出力しない。
/// 開閉状態は `crate::script::SITE_JS` が配線完了時に付与して同期する
/// （固定値を SSR に焼くと JS 無効時に嘘の状態を公開するため）。これは
/// `crate::nav::header_nav` の「ナビに role/aria-expanded/aria-haspopup を付けない」
/// 規約とは別で、検索ボタンは文書リンク集ではなくダイアログを開く操作である。
///
/// 入力の枠は pre-styled-ui の `input_group`（前側に虫眼鏡）が担う。入力欄は素の
/// `input.docs-search-input` のまま（pre-styled の `input` は呼び出し側 class を
/// 捨てるため使わない。class・id・role・aria 属性・`data-search-index` は
/// `crate::script::SITE_JS` の契約）。ダイアログは top layer に昇格しても DOM 上は
/// `.docs-header` 配下に残るため、`@scope (.docs-header)` の recipe が効く。
fn search_block(search_index_href: &str) -> Node {
    let group_props = ps_input_group::InputGroupProps {
        disabled: false,
        invalid: false,
    };
    div(
        vec![("class", "docs-search"), ("hidden", "")],
        vec![
            // ラッパー span + 内側 ps_button の型は `theme_toggle` と同じ
            // （`button()` は呼び出し側 class を捨てるため）。
            el(
                "span",
                vec![("class", "docs-search-trigger")],
                vec![ps_button::button(
                    &ps_button::ButtonProps {
                        variant: ps_button::ButtonVariant::Outline,
                        size: Size::Sm,
                        ..ps_button::ButtonProps::default()
                    },
                    vec![
                        ("aria-label", "ドキュメントを検索"),
                        ("aria-keyshortcuts", "/"),
                        ("aria-controls", SEARCH_DIALOG_ID),
                    ],
                    vec![
                        ps_icon::icon(
                            &ps_icon::IconProps {
                                size: Size::Sm,
                                ..ps_icon::IconProps::default()
                            },
                            vec![],
                            vec![icon_path(SEARCH_ICON_PATH)],
                        ),
                        el(
                            "span",
                            vec![("class", "docs-search-trigger-label")],
                            vec![text("検索")],
                        ),
                        // ショートカットの目印。操作は `SITE_JS` の `/` キー処理、
                        // 支援技術へはボタンの `aria-keyshortcuts` で伝える。
                        el(
                            "span",
                            vec![
                                ("aria-hidden", "true"),
                                ("class", "docs-search-trigger-key"),
                            ],
                            vec![ps_kbd::kbd(
                                &ps_kbd::KbdProps {
                                    size: Size::Sm,
                                    ..ps_kbd::KbdProps::default()
                                },
                                vec![],
                                vec![text("/")],
                            )],
                        ),
                    ],
                )],
            ),
            el(
                "dialog",
                vec![
                    ("id", SEARCH_DIALOG_ID),
                    ("class", "docs-search-dialog"),
                    ("aria-label", "ドキュメント内検索"),
                ],
                vec![div(
                    vec![("class", "docs-search-dialog-panel")],
                    vec![
                        // 視覚上は clip 手法で隠すラベル（`.docs-search-label`）。
                        // `input` が `aria-label` を持つため名前には使われない。
                        el(
                            "label",
                            vec![("class", "docs-search-label"), ("for", SEARCH_INPUT_ID)],
                            vec![text("Search")],
                        ),
                        ps_input_group::root(
                            &group_props,
                            vec![],
                            vec![
                                ps_input_group::addon(
                                    ps_input_group::InputGroupAlign::InlineStart,
                                    &group_props,
                                    vec![],
                                    vec![ps_icon::icon(
                                        &ps_icon::IconProps {
                                            size: Size::Sm,
                                            ..ps_icon::IconProps::default()
                                        },
                                        vec![],
                                        vec![icon_path(SEARCH_ICON_PATH)],
                                    )],
                                ),
                                el(
                                    "input",
                                    vec![
                                        ("type", "search"),
                                        ("id", SEARCH_INPUT_ID),
                                        ("class", "docs-search-input"),
                                        ("placeholder", "ドキュメントを検索"),
                                        ("aria-label", "ドキュメント内検索"),
                                        ("role", "combobox"),
                                        ("aria-expanded", "false"),
                                        ("aria-controls", SEARCH_RESULTS_ID),
                                        ("aria-autocomplete", "list"),
                                        ("autocomplete", "off"),
                                        ("data-search-index", search_index_href),
                                    ],
                                    vec![],
                                ),
                            ],
                        ),
                        ul(
                            vec![
                                ("id", SEARCH_RESULTS_ID),
                                ("class", "docs-search-results"),
                                ("role", "listbox"),
                                ("aria-label", "Search results"),
                                ("hidden", ""),
                            ],
                            vec![],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// GitHub リンク（#951/#3606）。`span.docs-github-link` を中立なラッパーとして
/// 残し（class 契約の維持）、内側に pre-styled-ui の `link` を置く。
/// `external: true` が `target="_blank"` と `rel="noopener noreferrer"` を一緒に
/// 付ける（OWASP A05: tabnabbing 対策。attrs で重ねて渡すと重複するため渡さない）。
fn github_link(repository_url: &str) -> Node {
    el(
        "span",
        vec![("class", "docs-github-link")],
        vec![ps_link::root(
            repository_url,
            &ps_link::LinkProps {
                external: true,
                ..ps_link::LinkProps::default()
            },
            vec![],
            vec![
                ps_icon::icon(
                    &ps_icon::IconProps {
                        size: Size::Sm,
                        view_box: "0 0 16 16",
                        ..ps_icon::IconProps::default()
                    },
                    vec![],
                    vec![icon_path(GITHUB_ICON_PATH)],
                ),
                el(
                    "span",
                    vec![("class", "docs-github-label")],
                    vec![text("GitHub")],
                ),
            ],
        )],
    )
}

/// テーマトグル（#951/#3606）。ラッパー `span.docs-theme-toggle` が既定 `hidden`
/// を持ち（JS 無効時・`site.js` 読み込み失敗時は `STRUCTURAL_CSS` の
/// `.docs-theme-toggle[hidden]` が非表示を担保し、`prefers-color-scheme` 追従へ
/// 退避する）、内側に ghost variant の pre-styled-ui `button` を置く。可視化・
/// イベント配線は `crate::script::SITE_JS` のみが行い、可視ラベルは
/// `span.docs-theme-toggle-label` の `textContent` だけを書き換える
/// （ボタン全体を書き換えるとアイコンが消えるため）。
fn theme_toggle() -> Node {
    el(
        "span",
        vec![("class", "docs-theme-toggle"), ("hidden", "")],
        vec![ps_button::button(
            &ps_button::ButtonProps {
                variant: ps_button::ButtonVariant::Ghost,
                size: Size::Sm,
                ..ps_button::ButtonProps::default()
            },
            vec![
                ("aria-label", "Toggle color theme"),
                ("aria-pressed", "false"),
            ],
            vec![
                ps_icon::icon(
                    &ps_icon::IconProps {
                        size: Size::Sm,
                        ..ps_icon::IconProps::default()
                    },
                    vec![],
                    vec![icon_path(CONTRAST_ICON_PATH)],
                ),
                el(
                    "span",
                    vec![("class", "docs-theme-toggle-label")],
                    vec![text("Theme")],
                ),
            ],
        )],
    )
}

/// 虫眼鏡アイコン（24 viewBox・塗り）。自前 path で外部アセットを持ち込まない。
const SEARCH_ICON_PATH: &str = "M15.5 14h-.79l-.28-.27A6.471 6.471 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z";

/// GitHub マーク（16 viewBox・塗り）。
const GITHUB_ICON_PATH: &str = "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z";

/// 明暗コントラスト図形（24 viewBox・塗り）。
const CONTRAST_ICON_PATH: &str = "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18V4c4.41 0 8 3.59 8 8s-3.59 8-8 8z";

/// 目次に載せる見出しレベルの上限（イシュー #950）。`h2` を第 1 段
/// （`TOC_MAX_LEVEL - 1`）、`h3` を第 2 段（`TOC_MAX_LEVEL`）とし、
/// [`toc_nav`] はこれを超える `level` の [`TocEntry`] を出力しない。
///
/// 実測（`docs/api/headless-ui-api.md`）では `heading_level` が `h2`/`h3`
/// しか返さないため階層は最初から 2 段であり、本定数は将来
/// `heading_level` の収集対象が `h4` 以降へ拡張された場合でも右目次を
/// 2 段に固定し続けるための fail-closed なガードである（意図的な深さ制限
/// であって、収集ロジック自体の拡張ではない）。
pub const TOC_MAX_LEVEL: u8 = 3;

/// 右目次見出し（`h2.docs-toc-title`）に付与する `id`。`nav.docs-toc` の
/// `aria-labelledby` が参照する単一実装点。
pub const TOC_HEADING_ID: &str = "docs-toc-heading";

/// [`TOC_HEADING_ID`] の表示テキスト。
const TOC_HEADING_TEXT: &str = "On this page";

/// 検索入力（`input.docs-search-input`）に付与する `id`。直前の
/// `label.docs-search-label` の `for` 属性が参照する単一実装点
/// （イシュー #958、fandhe-backend とのデザイン統一で `label`/`for` を
/// 追加した際に新設。セキュリティ監査の Low 指摘で判明した「固定 id が
/// [`RESERVED_LAYOUT_IDS`] へ未集約だった」問題の是正対象の 1 つ）。
pub const SEARCH_INPUT_ID: &str = "docs-search-input";

/// 検索結果一覧（`ul.docs-search-results`）に付与する `id`。
/// [`SEARCH_INPUT_ID`] を持つ `input` の `aria-controls` 属性が参照する
/// 単一実装点（WAI-ARIA combobox パターン、イシュー #958）。
pub const SEARCH_RESULTS_ID: &str = "docs-search-results";

/// 検索ダイアログ（`dialog.docs-search-dialog`、イシュー #3672）に付与する `id`。
/// 検索ボタンの `aria-controls` と `SITE_JS` が参照する単一実装点。
pub const SEARCH_DIALOG_ID: &str = "docs-search-dialog";

/// ナビ drawer の開閉チェックボックスハック（`input[type=checkbox]`、イシュー #3674）に
/// 付与する `id`。直後の `label.docs-nav-drawer-toggle-label` の `for` 属性が
/// 参照する単一実装点。旧サイドバー Menu トグル（`docs-sidebar-toggle`）の後継。
pub const NAV_DRAWER_TOGGLE_ID: &str = "docs-nav-drawer-toggle";

/// レイアウトが固定 `id` として出力する要素の `id` 一覧。
///
/// [`with_heading_anchors`] が本文見出しの自動生成 slug を採番する前に
/// この全件を予約するための single source of truth（セキュリティ監査の
/// Low 指摘、イシュー #950 の再発防止）。[`TOC_HEADING_ID`] のみを予約する
/// 実装だったため、`site/**.md` の見出しテキストが偶然
/// [`SEARCH_INPUT_ID`]・[`SEARCH_RESULTS_ID`]・[`NAV_DRAWER_TOGGLE_ID`] へ
/// slug 化されると同一 HTML 文書内で `id` が重複し、`label[for]`・
/// `aria-controls` の関連付けが壊れる（スクリーンリーダー利用者への
/// 参照先が不定になるアクセシビリティ回帰）。新しい固定 `id` を
/// `docs_page_with_assets` へ追加する場合は必ず本配列へ追記すること
/// （`crates/docs-site/tests/layout_reserved_ids.rs` がドリフトを検知する）。
///
/// SkipNav の `id`（[`ps_skip_nav::DEFAULT_ID`]）は他クレート
/// （`fandhe-frontend-headless-ui`）が所有する値だが、**本ページが実際に
/// 出力する固定 `id`** である以上、衝突回避の観点では所有者が誰かは
/// 無関係であるため本配列へ含める（本文見出しが `"fandhe-skip-nav"` へ
/// slug 化された場合も、SkipNav リンクの `href="#..."` が本文冒頭ではなく
/// 見出しへ飛んでしまう回帰を防ぐ）。値そのものの定義は他クレートに
/// 委ねたままで、ここでは予約対象として参照するだけに留める。
pub const RESERVED_LAYOUT_IDS: &[&str] = &[
    TOC_HEADING_ID,
    SEARCH_INPUT_ID,
    SEARCH_RESULTS_ID,
    SEARCH_DIALOG_ID,
    NAV_DRAWER_TOGGLE_ID,
    ps_skip_nav::DEFAULT_ID,
];

/// ページ内目次（TOC）の 1 エントリ。
///
/// [`with_heading_anchors`] が本文 `Node` を走査して収集する。`level` は
/// 見出しタグに対応する（`h2` → 2 / `h3` → 3）。`id` はアンカー先の
/// `id` 属性値（新規注入 or 既存採用）、`title` は見出しの表示テキスト。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    /// 見出しレベル（`h2` → 2 / `h3` → 3）。
    pub level: u8,
    /// アンカー先 `id` 属性値。
    pub id: String,
    /// 見出しの表示テキスト（既定エスケープ前のプレーン文字列）。
    pub title: String,
}

/// 本文 `Node` を走査して `h2`/`h3` 見出しを検出し、`id` 属性を注入した
/// 本文と、文書出現順の [`TocEntry`] 列を返す。
///
/// 既存の `id` 属性を持つ見出しはそれを尊重してそのまま採用し、注入しない
/// （静的アンカーへのリンク互換性を壊さないため）。`id` が無い見出しには
/// 見出しテキストから決定的に生成した slug を注入する。同一 slug が複数
/// 生成される場合は `-2` `-3` … を付与して一意化する（同一入力に対して
/// 常に同一出力を返す決定性を保証する。REQ-6 のモード非依存性契約に倣う）。
///
/// 見出しテキストは配下の [`Node::Text`] を出現順に連結して得る
/// （[`Node::RawHtml`] は連結対象に含めない。docs-site クレートは
/// `raw_html()` を使わない方針のため通常は出現しないが、混入した場合でも
/// TOC タイトルに生 HTML 断片を取り込まない防御的実装）。
///
/// `data-scope` 属性を持つ要素（headless-ui コンポーネントの anatomy）の
/// 部分木は走査対象外とし、部品内部の見出し（Accordion trigger の `h3` 等）
/// をアンカー注入・TOC 収集から決定的に除外する。
pub fn with_heading_anchors(body: Node) -> (Node, Vec<TocEntry>) {
    let mut entries = Vec::new();
    let mut used_ids = HashSet::new();
    // 右目次見出し（`h2#docs-toc-heading`、[`toc_nav`] が出力）の id を
    // 走査前に予約する（イシュー #950）。本文側の著者指定 id・自動生成
    // slug が偶然 `docs-toc-heading` と衝突しても、既存の「衝突時は
    // `unique_slug` で採番し直す」分岐がそのまま働き `docs-toc-heading-2`
    // へ回避されるため、id 重複が構造的に起こり得なくなる。
    //
    // 予約対象は右目次見出しだけでなく [`RESERVED_LAYOUT_IDS`]（レイアウトが
    // 出力する固定 id 全件）へ拡張済み（セキュリティ監査の Low 指摘）。
    // `docs-search-input`/`docs-search-results`/`docs-nav-drawer-toggle` は
    // それぞれ `label[for]`・`aria-controls` の関連付け先であり、
    // 本文見出しの slug と衝突して重複 id が発生すると、その関連付けが
    // 壊れてスクリーンリーダー利用者へ参照先が不定に伝わる（HTML 仕様上も
    // id の一意性違反）。TOC 見出し 1 件のみの予約では #950 時点で
    // 想定していなかった他の固定 id が保護対象外のままだったため、
    // [`RESERVED_LAYOUT_IDS`] を単一の情報源として全件を予約する。
    for reserved_id in RESERVED_LAYOUT_IDS {
        used_ids.insert(reserved_id.to_string());
    }
    let annotated = inject_heading_anchors(body, &mut entries, &mut used_ids);
    (annotated, entries)
}

/// [`with_heading_anchors`] の内部再帰実装。木を再構築しながら `h2`/`h3` を
/// 検出する。
fn inject_heading_anchors(
    node: Node,
    entries: &mut Vec<TocEntry>,
    used_ids: &mut HashSet<String>,
) -> Node {
    match node {
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            // headless-ui コンポーネントの anatomy ルート（`data-scope` 属性を
            // 持つ要素）配下の見出しは、文書アウトラインではなく部品構造の
            // 一部（例: Accordion の item trigger を包む `h3`、Card の title
            // `h3`）なので、部分木ごとアンカー注入・TOC 収集の対象外にする。
            // showcase（`crate::showcase`）の生成コンテンツにも本関数が適用
            // されるため、この除外が無いと部品内見出しがページ内目次へ混入
            // する（`tests/site_showcase.rs` が実サイトビルドで固定）。
            if attrs.iter().any(|(name, _)| name == "data-scope") {
                return Node::Element {
                    tag,
                    attrs,
                    children,
                };
            }
            let level = heading_level(tag);
            let new_children: Vec<Node> = children
                .into_iter()
                .map(|c| inject_heading_anchors(c, entries, used_ids))
                .collect();

            let Some(level) = level else {
                return Node::Element {
                    tag,
                    attrs,
                    children: new_children,
                };
            };

            let title = extract_text(&new_children);
            let existing_id = attrs
                .iter()
                .find(|(name, _)| name == "id")
                .map(|(_, value)| value.clone());

            let mut new_attrs = attrs;
            let id = match existing_id {
                Some(id) => {
                    // 著者指定 id が自動生成スラグ（または別の著者指定 id）と衝突する
                    // 場合、`used_ids.insert` は false を返す。ここで戻り値を無視すると
                    // 両見出しが同一 id を持ち TOC・静的 `#...` リンクが最初の見出ししか
                    // 指さなくなる（「既存 id を尊重する」契約は壊さず、衝突時のみ
                    // `unique_slug` で一意な variant を採番する）。
                    if used_ids.insert(id.clone()) {
                        id
                    } else {
                        let generated = unique_slug(&id, used_ids);
                        if let Some(entry) = new_attrs.iter_mut().find(|(name, _)| name == "id") {
                            entry.1 = generated.clone();
                        }
                        generated
                    }
                }
                None => {
                    let generated = unique_slug(&slugify(&title), used_ids);
                    new_attrs.push(("id".to_string(), generated.clone()));
                    generated
                }
            };

            entries.push(TocEntry { level, id, title });
            Node::Element {
                tag,
                attrs: new_attrs,
                children: new_children,
            }
        }
        other => other,
    }
}

/// 見出しタグ名からレベル（`h2` → 2 / `h3` → 3）を判定する。対象外のタグは
/// `None`。
fn heading_level(tag: &str) -> Option<u8> {
    match tag {
        "h2" => Some(2),
        "h3" => Some(3),
        _ => None,
    }
}

/// ノード列配下の [`Node::Text`] を出現順に連結する。[`Node::RawHtml`] は
/// 連結対象に含めない（見出しテキストに生 HTML 断片を混入させないため）。
fn extract_text(nodes: &[Node]) -> String {
    let mut out = String::new();
    for node in nodes {
        extract_text_into(node, &mut out);
    }
    out
}

/// [`extract_text`] の内部再帰実装。
fn extract_text_into(node: &Node, out: &mut String) {
    match node {
        Node::Text(s) => out.push_str(s),
        Node::Element { children, .. } => {
            for child in children {
                extract_text_into(child, out);
            }
        }
        Node::RawHtml(_) => {}
    }
}

/// 見出しテキストから id 用の slug を生成する。小文字化した上で英数字
/// （Unicode 含む。日本語見出しを許容するため）以外の連続を単一 `-` に
/// 置換し、先頭・末尾の `-` を除去する。結果が空文字列になる場合（記号の
/// みの見出し等）は `"section"` にフォールバックする。
pub fn slugify(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut slug = String::with_capacity(lower.len());
    let mut last_was_dash = false;
    for c in lower.chars() {
        if c.is_alphanumeric() {
            slug.push(c);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "section".to_string()
    } else {
        trimmed.to_string()
    }
}

/// `base` を `used_ids` に対して一意化する。既に使われていれば `-2` `-3` …
/// を付与し、決定的に一意な id を返す（採番結果を `used_ids` へ登録する）。
fn unique_slug(base: &str, used_ids: &mut HashSet<String>) -> String {
    if used_ids.insert(base.to_string()) {
        return base.to_string();
    }
    let mut suffix = 2u32;
    loop {
        let candidate = format!("{base}-{suffix}");
        if used_ids.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

/// [`TocEntry`] 列からページ内目次の `nav` `Node` を生成する。
/// [`TOC_MAX_LEVEL`] を超える見出しは目次から除外し、除外後に 1 件も
/// 残らない場合は `None` を返す（目次を出さない。見出しの無いページで
/// 空の `nav` を出力しないため。深さフィルタ適用後の空判定にすることで
/// `crate::layout::docs_page_with_assets` 側の `has_toc` 判定・
/// `docs-container--no-toc` 修飾も自動的に整合する）。
///
/// 各項目には `entry.level` に応じたレベルクラス
/// （`docs-toc-level-2` / `docs-toc-level-3`）を付与し、`h2`/`h3` の階層を
/// CSS 側のインデント表現で区別できるようにする（Bugbot 指摘 b0e41098:
/// 従来はフラットな `<li>` 列で `level` を一切参照しておらず、見出し階層が
/// マークアップ上で表現できなかった）。
///
/// 先頭に `h2.docs-toc-title`（[`TOC_HEADING_ID`]、イシュー #950）を出力し、
/// `nav` へ `aria-labelledby` で紐付ける（ランドマークに名前を与える。
/// WCAG 2.4.1 相当）。現在地ハイライト（`aria-current="location"`）は
/// [`crate::script::SITE_JS`] のみが実行時に付与する契約であり、本関数の
/// 出力には一切含めない（JS 無効・読み込み失敗時は通常のリンク表示の
/// ままにする progressive enhancement、`crate::script` モジュール doc 参照）。
pub fn toc_nav(entries: &[TocEntry]) -> Option<Node> {
    let items = toc_items(entries)?;
    let heading = h2(
        vec![("class", "docs-toc-title"), ("id", TOC_HEADING_ID)],
        vec![text(TOC_HEADING_TEXT.to_string())],
    );
    Some(nav(
        vec![("class", "docs-toc"), ("aria-labelledby", TOC_HEADING_ID)],
        vec![heading, ul(vec![], items)],
    ))
}

/// [`toc_nav`] と [`toc_inline`] が共有する `<li>` 列の生成ロジック
/// （イシュー #1080）。[`TOC_MAX_LEVEL`] を超える見出しを除外し、除外後に
/// 1 件も残らない場合は `None` を返す。両関数がこのヘルパ 1 本を経由する
/// ことで、「右目次は出るが折りたたみ目次は出ない（またはその逆）」
/// といった不整合が構造的に起こり得ない（`docs_page_with_assets` 側の
/// `has_toc` 判定はこの結果に対して行われる）。
fn toc_items(entries: &[TocEntry]) -> Option<Vec<Node>> {
    let items: Vec<Node> = entries
        .iter()
        .filter(|entry| entry.level <= TOC_MAX_LEVEL)
        .map(|entry| {
            let href = format!("#{}", entry.id);
            let level_class = format!("docs-toc-level-{}", entry.level);
            li(
                vec![("class", &level_class)],
                vec![a(vec![("href", &href)], vec![text(entry.title.clone())])],
            )
        })
        .collect();
    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

/// 狭幅帯域（`< 1200px`）で右目次カラム（`aside.docs-toc-aside`）が
/// `display: none` になる代替として、本文冒頭に置く折りたたみ目次
/// （イシュー #1080）。`>= 1200px` は `crate::site_theme::STRUCTURAL_CSS`
/// 側で `display: none` に切り替わり、右目次カラムとの重複表示を避ける
/// （CSS 側の責務。本関数は markup のみを担う）。
///
/// # 設計上の決定（rustdoc に明記し将来の「簡素化」で崩さないための記録）
///
/// - **`class="docs-toc"` を持たせない**: [`crate::script::SITE_JS`] の
///   スクロールスパイは `document.querySelector('.docs-toc')`（DOM 先頭の
///   1 件のみ）で右目次を掴む契約。折りたたみ目次にも同じ class を付けると、
///   `>= 1200px` では DOM 上先に現れる（かつ `display: none` の）折りたたみ
///   側に observer が付いてしまい、右カラムの現在地ハイライト（#950）が
///   無音で死ぬ。専用 class（`docs-toc-inline`/`docs-toc-inline-summary`）
///   のみを新設し、`crate::script` は変更しない。
/// - **[`TOC_HEADING_ID`] を再利用しない**: 右目次の `h2#docs-toc-heading`
///   と同じ `id` を本文冒頭にも付けると同一ページ内で `id` が重複する
///   （HTML 仕様違反・フラグメントリンクの解決先が不定になる）。折りたたみ
///   目次側は `aria-label` でランドマーク名を与える（値は `TOC_HEADING_TEXT`
///   を [`toc_nav`] と共有し文言のドリフトを防ぐ）。
/// - **既定で閉（`open` 属性なし）**: 本文の初期表示位置を押し下げない。
///   開閉はネイティブ `<details>` の挙動であり JS を要さない
///   （`crate::nav::group_node` の `details.docs-nav-group` と同型の
///   ディスクロージャパターン、イシュー #940 の先例に揃える）。
/// - **開閉アイコンは素の `svg`（`icon::icon` を使わない）**: 見出しのある
///   全ページ（recipe 抜きの `site-primitives.css` を読む Primitives
///   ページを含む）が本目次を出す。`fd-icon--size-md` は recipe 側にしか
///   無く未定義になりブラウザ既定サイズで描画されるため、サイズは
///   `.docs-toc-inline-icon` で与える（イシュー #3610）。
pub fn toc_inline(entries: &[TocEntry]) -> Option<Node> {
    let items = toc_items(entries)?;
    let summary = el(
        "summary",
        vec![("class", "docs-toc-inline-summary")],
        vec![text(TOC_HEADING_TEXT.to_string()), toc_inline_icon()],
    );
    let details = el("details", vec![], vec![summary, ul(vec![], items)]);
    Some(nav(
        vec![
            ("class", "docs-toc-inline"),
            ("aria-label", TOC_HEADING_TEXT),
        ],
        vec![details],
    ))
}

/// 折りたたみ目次の開閉を示す下向きシェブロン（装飾のため `aria-hidden`）。
/// 開状態の回転は CSS（`details[open]`）が担い、JS を要さない。
fn toc_inline_icon() -> Node {
    el(
        "svg",
        vec![
            ("class", "docs-toc-inline-icon"),
            ("aria-hidden", "true"),
            ("focusable", "false"),
            ("viewBox", "0 0 24 24"),
            ("fill", "none"),
        ],
        vec![el(
            "path",
            vec![
                ("d", "M6 9l6 6l6 -6"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// `base_path` を考慮したアセット参照パスを生成する（受け入れ条件 3 の
/// 単一実装点。`docs_page` 内のアセットリンク・サイトルートリンクは必ず
/// 本関数を経由し、パス結合ロジックを重複させない）。
///
/// `base_path` の末尾スラッシュ・空文字列は正規化する。`relative` が
/// 空文字列の場合はサイトルート（`base_path` 直下）を指すパスを返す。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_docs_site::layout::asset_href;
///
/// assert_eq!(asset_href("", "assets/site.css"), "/assets/site.css");
/// assert_eq!(
///     asset_href("/fandhe-frontend", "assets/site.css"),
///     "/fandhe-frontend/assets/site.css"
/// );
/// assert_eq!(
///     asset_href("/fandhe-frontend/", "assets/site.css"),
///     "/fandhe-frontend/assets/site.css"
/// );
/// ```
pub fn asset_href(base_path: &str, relative: &str) -> String {
    let trimmed_base = base_path.trim_end_matches('/');
    let trimmed_relative = relative.trim_start_matches('/');

    if trimmed_relative.is_empty() {
        if trimmed_base.is_empty() {
            "/".to_string()
        } else {
            format!("{trimmed_base}/")
        }
    } else if trimmed_base.is_empty() {
        format!("/{trimmed_relative}")
    } else {
        format!("{trimmed_base}/{trimmed_relative}")
    }
}

/// タイトル・`base_path`・サイドバー・本文から完全な HTML 文書 `Node`
/// （`<html>` 要素）を組み立てる。
///
/// 内部で [`with_heading_anchors`] と [`toc_nav`] を適用し、本文中の
/// `h2`/`h3` にアンカーを注入した上でページ内目次を生成する。`title` は
/// [`text`] 経由で、`sidebar`/`body` はそのまま `Node` 木として埋め込むため
/// テキストスロットはすべて既定エスケープ済みで出力される（`raw_html()`・
/// HTML 文字列の直接組み立ては一切行わない）。
///
/// `<!DOCTYPE html>` の前置は呼び出し側
/// （`fandhe_frontend_server::ssg::generate_pages()`）の契約であり、本関数は
/// 文書 `Node` を返すのみで DOCTYPE 文字列を出力しない。
pub fn docs_page(title: &str, base_path: &str, sidebar: Node, body: Node) -> Node {
    docs_page_with_assets(title, base_path, sidebar, body, &[], None, None)
}

/// [`docs_page`] の拡張版。`extra_stylesheets`（`assets/` 起点の相対パス列）を
/// `assets/site.css` の後に追加の `<link rel="stylesheet">` として `<head>` へ
/// 差し込む。
///
/// Rust 生成コンテンツページ（`crate::showcase` が pre-styled-ui コンポーネント
/// を実レンダリングするショーケース、イシュー #520 系）だけが、
/// `fandhe_frontend_server::ssg::generate_assets`（イシュー #1136）で書き出す
/// 専用 CSS（`assets/pre-styled-ui.css`）を参照するために `crate::build::build_site`
/// から呼ばれる。サイト骨格スタイル（`crate::site_theme` がビルド時生成する
/// `assets/site.css`、イシュー #905）とコンポーネント CSS を分離ファイルに
/// 保ち、既存ページのカスケードへ影響させないための注入点であり、Markdown
/// ページは従来どおり [`docs_page`]（追加なし）を使う。href は
/// [`asset_href`] を経由して `base_path` を考慮した単一実装点を守る。
///
/// `header_nav`（イシュー #908）が `Some` の場合、`header.docs-header` 直下の
/// `div.docs-header-inner`（イシュー #949 で新設。`.docs-container` と同じ
/// `max-width`/`margin: 0 auto` を共有し、ヘッダー左端をサイドバー・本文の
/// 左端に揃える計測枠）の第 2 子として `crate::nav::header_nav()` が生成する
/// セクション別ドロップダウンメニューを埋め込む。`None` の場合はブランド
/// リンクのみの従来ヘッダーのまま（[`docs_page`] 経由の呼び出しはこちら）。
///
/// `footer`（イシュー #3609、`crate::site_footer::site_footer()` の戻り値）が
/// `Some` の場合、`<body>` の最後の子（`div.docs-container` の直後）として
/// 出力する。`<footer>` は `main`/`article`/`aside`/`nav`/`section` の子孫に
/// 置くと暗黙ロール `contentinfo` を失い、本文 `Node` へ追記すると TOC・検索
/// インデックスにも混入するため、ヘッダーと同じクロームとして本関数が
/// `body` 直下へ置く。sticky のサイドバー・右目次は包含ブロックである
/// `.docs-container` の内側に限られるので、外の兄弟であるフッターとは構造上
/// 重ならない。`None` なら従来出力とバイト一致する。
pub fn docs_page_with_assets(
    title: &str,
    base_path: &str,
    sidebar: Node,
    body: Node,
    extra_stylesheets: &[&str],
    header_nav: Option<Node>,
    footer: Option<Node>,
) -> Node {
    docs_page_with_layout(
        title,
        base_path,
        sidebar,
        body,
        extra_stylesheets,
        header_nav,
        None,
        footer,
        PageLayout::Docs,
    )
}

/// ページ骨格の種別（イシュー #3612）。
///
/// [`crate::page_sections::PageSection::layout`] が登録表からページ単位で宣言し、
/// [`crate::build::build_site_with`] が [`docs_page_with_layout`] へ渡す。
/// `path == "/"` の直書き判定は汎用エンジンの挙動をフィクスチャへ波及させるため
/// 採らず、登録表を唯一の鍵にする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageLayout {
    /// 左ナビ + 本文 + 右目次の標準ドキュメント骨格。
    #[default]
    Docs,
    /// 全幅ランディング骨格（トップページ）。右目次・折りたたみ目次を出さず、
    /// 広幅ではサイドバーを CSS で隠す。ヘッダー・SkipNav・本文 article の
    /// DOM 順序は [`PageLayout::Docs`] と同一に保つ。
    Landing,
}

/// [`docs_page_with_assets`] の骨格種別指定版。`layout` が
/// [`PageLayout::Landing`] のとき、右目次（`aside.docs-toc-aside`）と折りたたみ
/// 目次（`nav.docs-toc-inline`）を出力せず、コンテナ class を
/// `docs-container docs-landing` にする。`aside.docs-sidebar` は DOM に残す
/// （広幅での非表示は CSS〔`crate::landing::CSS`〕が担い、768px 未満ではナビ drawer が
/// 全セクションへの手段になる）。
///
/// `nav_drawer`（イシュー #3674、`crate::nav::nav_drawer()` の戻り値）が `Some` の場合、
/// `div.docs-header-inner` の**末尾**（`div.docs-header-actions` の後ろ）へ
/// `input.docs-nav-drawer-toggle` → `label` → drawer の順で置く。checkbox が drawer の
/// 前にある兄弟でなければ `:checked ~` で開閉できないための配置で、brand が第 1 子・
/// DOM 順 = 視覚順 = Tab 順の契約（#3659）も保つ。`None` ならこの 3 要素は出ない。
///
/// `<head>` の順序契約: charset → viewport → meta CSP（[`crate::csp`]、#3678）→
/// title → favicon → theme-init → stylesheet 群 → site.js。CSP は meta より前の
/// 要素へ効かないため、全 `script`/`link` より前に置く。
// 公開 API 互換のため引数を構造体化せず、骨格の各スロットを個別引数で受ける（呼び出し元は
// `docs_page_with_assets` 等の薄いラッパーに限られる）。
#[allow(clippy::too_many_arguments)]
pub fn docs_page_with_layout(
    title: &str,
    base_path: &str,
    sidebar: Node,
    body: Node,
    extra_stylesheets: &[&str],
    header_nav: Option<Node>,
    nav_drawer: Option<Node>,
    footer: Option<Node>,
    layout: PageLayout,
) -> Node {
    docs_page_with_chrome(
        &SiteChrome::default(),
        title,
        base_path,
        sidebar,
        body,
        extra_stylesheets,
        header_nav,
        nav_drawer,
        footer,
        layout,
    )
}

/// [`docs_page_with_layout`] の `[site]` 設定指定版（#3720）。`chrome` でヘッダーのブランド名と
/// GitHub リンク先を差し替える。他の引数・出力契約は [`docs_page_with_layout`] と同一
/// （`SiteChrome::default()` ならバイト一致）。`build_site*` と 404 生成から呼ばれる。
#[allow(clippy::too_many_arguments)]
pub fn docs_page_with_chrome(
    chrome: &SiteChrome<'_>,
    title: &str,
    base_path: &str,
    sidebar: Node,
    body: Node,
    extra_stylesheets: &[&str],
    header_nav: Option<Node>,
    nav_drawer: Option<Node>,
    footer: Option<Node>,
    layout: PageLayout,
) -> Node {
    let landing = layout == PageLayout::Landing;
    let (annotated_body, toc_entries) = with_heading_anchors(body);
    let toc = toc_nav(&toc_entries);
    // 狭幅帯域（`< 1200px`）向けの折りたたみ目次（イシュー #1080）。`toc` と
    // 同じ `toc_items` から導出されるため、「右目次は出るが折りたたみ目次は
    // 出ない」といった不整合は構造的に起こらない（`toc_inline` rustdoc 参照）。
    let toc_inline_nav = if landing {
        None
    } else {
        toc_inline(&toc_entries)
    };
    let toc = if landing { None } else { toc };

    // Primitives ページ（専用 CSS を配線するページ）は headless-ui デモへ styled
    // recipe を到達させないため、recipe 抜きの site CSS を読む。
    let site_css_rel_path =
        if extra_stylesheets.contains(&crate::primitive_showcase::STYLESHEET_REL_PATH) {
            crate::site_theme::PRIMITIVES_STYLESHEET_REL_PATH
        } else {
            crate::site_theme::STYLESHEET_REL_PATH
        };
    let mut head_children = vec![
        el("meta", vec![("charset", "utf-8")], vec![]),
        el(
            "meta",
            vec![
                ("name", "viewport"),
                ("content", "width=device-width, initial-scale=1"),
            ],
            vec![],
        ),
        // meta CSP（イシュー #3678）。CSP は meta より前にある要素へ効かないため、
        // `title` と全 `script`/`link` より前に置く。`charset` は先頭 1024 バイト以内
        // に置く必要があるので先頭のまま残す。値は定数のみで通常の属性 API
        // （既定エスケープ）を通る。
        el(
            "meta",
            vec![
                ("http-equiv", "Content-Security-Policy"),
                ("content", crate::csp::CONTENT_SECURITY_POLICY),
            ],
            vec![],
        ),
        el("title", vec![], vec![text(title.to_string())]),
        // SVG favicon（イシュー #3604）。無指定だとブラウザが `base_path` 外の
        // `/favicon.ico` を取りに行き 404 になるため、`base_path` 付きで明示する。
        el(
            "link",
            vec![
                ("rel", "icon"),
                ("type", "image/svg+xml"),
                ("href", &asset_href(base_path, crate::favicon::REL_PATH)),
            ],
            vec![],
        ),
    ];
    // FOUC 抑止のテーマ初期化（イシュー #951/#3676）。`script-src 'self'` の CSP
    // 下で実行できないインライン script をやめ、同期（`defer`/`async`/module なし）
    // の外部 `<script src>` として全 `<link rel="stylesheet">` より前に置く。
    // 同期 script はパーサをブロックするため初回ペイント前に `data-theme` が
    // 確定する。本文は持たず URL は `asset_href` 経由の定数のため、安全性検証
    // （`script::theme_init_js`）は `crate::build::build_site` の書き出し前に行う。
    head_children.push(el(
        "script",
        vec![("src", &asset_href(base_path, script::THEME_INIT_REL_PATH))],
        vec![],
    ));
    // View Transitions の opt-in は site CSS（`site_theme::VIEW_TRANSITION_CSS`）が
    // 担う。CSP `style-src 'self'` のためインライン `<style>` は出さない（#3677）。
    head_children.push(el(
        "link",
        vec![
            ("rel", "stylesheet"),
            ("href", &asset_href(base_path, site_css_rel_path)),
        ],
        vec![],
    ));
    for relative in extra_stylesheets {
        head_children.push(el(
            "link",
            vec![
                ("rel", "stylesheet"),
                ("href", &asset_href(base_path, relative)),
            ],
            vec![],
        ));
    }
    // SkipNav（イシュー #776）専用 CSS は showcase/admonition と異なり
    // 全ページへ無条件に適用する（`crate::skip_nav` モジュール doc 参照）。
    // `crate::build::build_site` が `crate::skip_nav::STYLESHEET_REL_PATH`
    // を全ビルドで無条件に書き出す契約と対をなす。
    head_children.push(el(
        "link",
        vec![
            ("rel", "stylesheet"),
            (
                "href",
                &asset_href(base_path, crate::skip_nav::STYLESHEET_REL_PATH),
            ),
        ],
        vec![],
    ));
    // 全 `<link rel="stylesheet">` の後に `assets/site.js`（イシュー #951）
    // を `defer` で読み込む。`src` はスクリプト本文を含まないため
    // `is_url_attr`/`is_safe_url`（`fandhe_frontend_core`）の既存検証を通る
    // 通常のアセット参照（[`asset_href`] 経由の単一実装点）。
    head_children.push(el(
        "script",
        vec![
            ("src", &asset_href(base_path, script::SCRIPT_REL_PATH)),
            ("defer", ""),
        ],
        vec![],
    ));
    let head = el("head", vec![], head_children);

    // 「on this page」目次は 3 カラム骨格化（イシュー #907、設計文書
    // §3.1/§3.3）に伴い `main.docs-main` の外へ移設し、右目次カラム
    // （`aside.docs-toc-aside`）として `div.docs-container` の第 3 子に置く。
    // `main_children` は折りたたみ目次（任意）・SkipNav ターゲット・本文を
    // 保持する。
    // 折りたたみ目次（イシュー #1080）は `main.docs-main` の第 1 子、かつ
    // SkipNav のスキップ先ターゲットより**前**に置く。これにより「SkipNav の
    // スキップ先は読者が実際に読み始める本文（article）の直前」という直後の
    // コメントの契約を字義どおり維持しつつ、「Skip to content」でページ内
    // 目次を飛ばして本文へ直接到達できる意味論も保たれる。
    // SkipNav のスキップ先ターゲット（イシュー #776）。読者が実際に読み始める
    // 本文（article）の直前に置き、`link` クリック時のプログラム的フォーカス
    // 移動先とする（`fandhe-frontend-headless-ui::skip_nav` の
    // `tabindex="-1"` 契約参照）。
    let mut main_children: Vec<Node> = Vec::new();
    if let Some(inline_toc) = toc_inline_nav {
        main_children.push(inline_toc);
    }
    main_children.push(ps_skip_nav::content(
        ps_skip_nav::DEFAULT_ID,
        vec![],
        vec![],
    ));
    main_children.push(article(
        vec![("class", "docs-content")],
        vec![annotated_body],
    ));

    let root_href = asset_href(base_path, "");
    // ブランドリンクは `class="docs-brand"` を持つ（イシュー #908。従来
    // セレクタ `.docs-header a` はヘッダーナビ内のドロップダウンリンクにも
    // 波及するため、ブランドリンク専用の class へ分離した。
    // `crate::site_theme::STRUCTURAL_CSS` 参照）。
    // 検索インデックス JSON への参照（イシュー #958）。`asset_href` を経由する
    // ことで `crate::script::SITE_JS` の `fetch()` 先が `base_path` を考慮した
    // 単一実装点から生成される（`data-search-index` 属性値のみに URL を持たせ、
    // 本文を HTML へ埋め込まない、`crate::search_index` モジュール doc 参照）。
    let search_index_href = asset_href(base_path, search_index::REL_PATH);

    let mut header_children = vec![brand(&root_href, chrome.brand)];
    if let Some(version) = brand_version() {
        header_children.push(version);
    }
    if let Some(nav_node) = header_nav {
        header_children.push(nav_node);
    }
    // ヘッダー右側のアクション群（検索・GitHub リンク・テーマトグル、イシュー
    // #951/#958/#3606）。`header_nav` の有無に関わらず無条件で出力する
    // （層 1 契約「見出しあり/なし両方のフィクスチャで出現すること」と同様、
    // `docs_page`/`docs_page_with_assets` いずれの経路でも出現させるため
    // 条件分岐を作らない。`crate::site_theme::STRUCTURAL_CSS` 参照）。
    header_children.push(div(
        vec![("class", "docs-header-actions")],
        vec![
            search_block(&search_index_href),
            github_link(chrome.repository_url),
            theme_toggle(),
        ],
    ));
    // ナビ drawer 一式（イシュー #3674）。checkbox（sr-only）は drawer より前の兄弟に
    // 置く必要があり（`:checked ~ .docs-nav-drawer`）、開閉状態の唯一の情報源にする。
    // `autocomplete="off"` は戻る・進むで checked が復元され drawer が開いたまま
    // 表示されるのを防ぐ。`role`/`aria-expanded`/`aria-haspopup` は付けない
    // （checkbox のネイティブ状態が支援技術へ伝わる。`crate::nav::header_nav` rustdoc と同じ判断）。
    let has_nav_drawer = nav_drawer.is_some();
    if let Some(drawer) = nav_drawer {
        header_children.push(el(
            "input",
            vec![
                ("type", "checkbox"),
                ("id", NAV_DRAWER_TOGGLE_ID),
                ("class", "docs-nav-drawer-toggle"),
                ("autocomplete", "off"),
            ],
            vec![],
        ));
        header_children.push(el(
            "label",
            vec![
                ("for", NAV_DRAWER_TOGGLE_ID),
                ("class", "docs-nav-drawer-toggle-label"),
            ],
            vec![el(
                "span",
                vec![("class", "docs-nav-drawer-toggle-text")],
                vec![text("Menu".to_string())],
            )],
        ));
        header_children.push(drawer);
    }
    // ヘッダー内側の計測枠（イシュー #949）。`.docs-header` 自体は罫線
    // （`border-bottom`）を全幅に伸ばすため padding を持たず、子要素は
    // すべてこの `div.docs-header-inner` の内側に置く。`.docs-container`
    // （左ナビ・本文・右目次の 3 カラムを束ねる要素）と同じ
    // `--fandhe-space-docs-container-width` を `max-width` に、
    // `margin: 0 auto` を共有することで、ブランドリンクの左端をサイドバー
    // 配下のリンク文字左端と同一 x 座標に揃える（`crate::site_theme`
    // 側の算式は `STRUCTURAL_CSS` の `.docs-header-inner` 規則コメント参照）。
    let header_inner = div(vec![("class", "docs-header-inner")], header_children);
    let header_node = header(vec![("class", "docs-header")], vec![header_inner]);

    // SkipNav の「本文へスキップ」リンク（イシュー #776）。キーボード操作時
    // のみ視覚的に現れ（`fandhe-frontend-pre-styled-ui::skip_nav` の
    // `:focus-visible` 表示規則）、ページ内で最初にフォーカス可能な要素と
    // なるよう `<body>` 先頭（`header` より前）に置く（WCAG 2.1 SC 2.4.1
    // Bypass Blocks）。
    let skip_nav_link = ps_skip_nav::link(
        ps_skip_nav::DEFAULT_ID,
        vec![],
        vec![text("Skip to content")],
    );

    // `div.docs-container` の子は「左ナビ / 中央コンテンツ / 右目次」の
    // 3 カラム順（設計文書 §3.1）。右目次カラムは見出しが 1 つも無いページ
    // では出力しない（`aside.docs-toc-aside` 自体を省略する。§3.3 の方針。
    // `nav.docs-toc` 単体で空 `nav` を出さない [`toc_nav`] の既存契約と揃える）。
    let has_toc = toc.is_some();
    let mut container_children = vec![
        aside(vec![("class", "docs-sidebar")], vec![sidebar]),
        main_tag(vec![("class", "docs-main")], main_children),
    ];
    if let Some(toc_node) = toc {
        container_children.push(aside(vec![("class", "docs-toc-aside")], vec![toc_node]));
    }

    // 見出しが無いページ（`aside.docs-toc-aside` 自体が出力されない）では
    // `docs-container--no-toc` 修飾 class を付与する。`min-width: 1200px`
    // の 3 カラム grid はこの class の有無で右目次列のグリッドトラックを
    // 収縮させ、見出しの無いページで空の右カラムが残ったまま中央カラムが
    // 狭くなる回帰を避ける（`crate::site_theme::STRUCTURAL_CSS` 参照、
    // Bugbot 指摘 #916 是正）。
    let container_class = if landing {
        "docs-container docs-landing"
    } else if has_toc {
        "docs-container"
    } else {
        "docs-container docs-container--no-toc"
    };
    // ナビ drawer が無いページ（`docs_page`/`docs_page_with_assets` 経由）は、768px 未満でも
    // サイドバーを表示してサイト内ナビを残す（codex P1 指摘、PR #3688）。既存の class 文字列
    // 契約を変えないよう、class ではなく `data-no-nav-drawer` 属性で CSS へ伝える。
    let mut container_attrs = vec![("class", container_class)];
    if !has_nav_drawer {
        container_attrs.push(("data-no-nav-drawer", ""));
    }

    let mut body_children = vec![
        skip_nav_link,
        header_node,
        div(container_attrs, container_children),
    ];
    if let Some(footer_node) = footer {
        body_children.push(footer_node);
    }
    let body_node = el("body", vec![], body_children);

    el("html", vec![("lang", "ja")], vec![head, body_node])
}
