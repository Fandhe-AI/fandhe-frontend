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
//! `<input type="checkbox" class="docs-sidebar-toggle">` + `<label>` の
//! チェックボックスハックを先頭に含み、`< 768px` での折りたたみをタッチ
//! 操作でも開閉できるようにする。JS 不要、`nav_list` 本体の markup は
//! 変更しない）・`main.docs-main`（中央コンテンツ、`article.docs-content`
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
//! インラインスニペット（[`crate::script::inline_theme_bootstrap`]）と
//! `<script src>`（[`crate::script::SCRIPT_REL_PATH`]、`defer`）を含める
//! （イシュー #951。旧「JS を含めない」宣言はこの変更で終了した）。
//!
//! `div.docs-header-actions` の第 1 子として検索ブロック
//! （`div.docs-search`）を無条件出力する（イシュー #958）。`input.docs-search-input`
//! の `data-search-index` 属性が [`search_index::REL_PATH`] を [`asset_href`]
//! 経由で参照し、`crate::script::SITE_JS` の第 3 IIFE が初回フォーカス時に
//! `fetch()` する唯一の実装点となる（インデックス JSON 自体は本モジュールが
//! HTML へインライン化しない、`crate::search_index` モジュール doc の
//! セキュリティ不変条件参照）。検索ブロック・結果一覧は既定 `hidden` とし、
//! `SITE_JS` が配線完了後にのみ可視化する（`.docs-theme-toggle` と同型の
//! progressive enhancement 契約、`crate::script` モジュール doc 手順 5 参照）。
//! `<form>` で包まない（JS 無効時に Enter キーでのフォーム送信を誘発しない
//! ため）。`input.docs-search-input` の直前に視覚上のみ clip で隠す
//! `label.docs-search-label`（`for="docs-search-input"`）を置く
//! （fandhe-backend の docs サイトとデザインを統一するための追加、
//! `crate::site_theme::STRUCTURAL_CSS` の `.docs-search-label` 参照）。

use std::collections::HashSet;

use fandhe_frontend_core::{
    a, article, aside, button, div, el, h2, header, li, main_tag, nav, text, ul, Node,
};
use fandhe_frontend_pre_styled_ui::skip_nav as ps_skip_nav;

use crate::script;
use crate::search_index;

/// GitHub リポジトリへの絶対 URL（ヘッダーの GitHub リンクが参照する
/// 単一実装点）。`site/nav.toml` の `[site]` スキーマは拡張しない
/// （nav スキーマの変更は #939 の管轄。ブランド文字列 `"fandhe-frontend"`
/// が既に本モジュールへハードコードされている先例に倣う）。
const REPOSITORY_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 目次に載せる見出しレベルの上限（イシュー #950）。`h2` を第 1 段
/// （`TOC_MAX_LEVEL - 1`）、`h3` を第 2 段（`TOC_MAX_LEVEL`）とし、
/// [`toc_nav`] はこれを超える `level` の [`TocEntry`] を出力しない。
///
/// 実測（`docs/api/headless-ui-api.md`）では [`heading_level`] が `h2`/`h3`
/// しか返さないため階層は最初から 2 段であり、本定数は将来
/// [`heading_level`] の収集対象が `h4` 以降へ拡張された場合でも右目次を
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

/// サイドバー折りたたみチェックボックスハック（`input[type=checkbox]`）に
/// 付与する `id`。直後の `label.docs-sidebar-toggle-label` の `for` 属性が
/// 参照する単一実装点。
pub const SIDEBAR_TOGGLE_ID: &str = "docs-sidebar-toggle";

/// レイアウトが固定 `id` として出力する要素の `id` 一覧。
///
/// [`with_heading_anchors`] が本文見出しの自動生成 slug を採番する前に
/// この全件を予約するための single source of truth（セキュリティ監査の
/// Low 指摘、イシュー #950 の再発防止）。[`TOC_HEADING_ID`] のみを予約する
/// 実装だったため、`site/**.md` の見出しテキストが偶然
/// [`SEARCH_INPUT_ID`]・[`SEARCH_RESULTS_ID`]・[`SIDEBAR_TOGGLE_ID`] へ
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
    SIDEBAR_TOGGLE_ID,
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
    // `docs-search-input`/`docs-search-results`/`docs-sidebar-toggle` は
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
fn slugify(text: &str) -> String {
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
/// （イシュー #1080）。`>= 1200px` は [`crate::site_theme::STRUCTURAL_CSS`]
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
///   目次側は `aria-label` でランドマーク名を与える（値は [`TOC_HEADING_TEXT`]
///   を [`toc_nav`] と共有し文言のドリフトを防ぐ）。
/// - **既定で閉（`open` 属性なし）**: 本文の初期表示位置を押し下げない。
///   開閉はネイティブ `<details>` の挙動であり JS を要さない
///   （`crate::nav::group_node` の `details.docs-nav-group` と同型の
///   ディスクロージャパターン、イシュー #940 の先例に揃える）。
pub fn toc_inline(entries: &[TocEntry]) -> Option<Node> {
    let items = toc_items(entries)?;
    let summary = el(
        "summary",
        vec![("class", "docs-toc-inline-summary")],
        vec![text(TOC_HEADING_TEXT.to_string())],
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
    docs_page_with_assets(title, base_path, sidebar, body, &[], None)
}

/// [`docs_page`] の拡張版。`extra_stylesheets`（`assets/` 起点の相対パス列）を
/// `assets/site.css` の後に追加の `<link rel="stylesheet">` として `<head>` へ
/// 差し込む。
///
/// Rust 生成コンテンツページ（`crate::showcase` が pre-styled-ui コンポーネント
/// を実レンダリングするショーケース、イシュー #520 系）だけが、
/// `StyleSheet::write_css_file` で書き出す専用 CSS
/// （`assets/pre-styled-ui.css`）を参照するために `crate::build::build_site`
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
pub fn docs_page_with_assets(
    title: &str,
    base_path: &str,
    sidebar: Node,
    body: Node,
    extra_stylesheets: &[&str],
    header_nav: Option<Node>,
) -> Node {
    let (annotated_body, toc_entries) = with_heading_anchors(body);
    let toc = toc_nav(&toc_entries);
    // 狭幅帯域（`< 1200px`）向けの折りたたみ目次（イシュー #1080）。`toc` と
    // 同じ `toc_items` から導出されるため、「右目次は出るが折りたたみ目次は
    // 出ない」といった不整合は構造的に起こらない（`toc_inline` rustdoc 参照）。
    let toc_inline_nav = toc_inline(&toc_entries);

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
        el("title", vec![], vec![text(title.to_string())]),
    ];
    // FOUC 抑止のインラインスニペット（イシュー #951）。全 `<link
    // rel="stylesheet">` より前に同期実行させ、保存済みテーマがあれば
    // CSS 適用前に `data-theme` を確定させる。`script::inline_theme_bootstrap`
    // が `None`（エスケープ安全性検証に落ちた）場合は `<script>` 自体を
    // 出力しない fail-closed（`crate::script` モジュール doc 参照）。
    if let Some(bootstrap) = script::inline_theme_bootstrap() {
        head_children.push(el("script", vec![], vec![text(bootstrap)]));
    }
    head_children.push(el(
        "style",
        vec![],
        vec![text("@view-transition { navigation: auto; }")],
    ));
    head_children.push(el(
        "link",
        vec![
            ("rel", "stylesheet"),
            (
                "href",
                &asset_href(base_path, crate::site_theme::STYLESHEET_REL_PATH),
            ),
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

    let mut header_children = vec![a(
        vec![("href", &root_href), ("class", "docs-brand")],
        vec![text("fandhe-frontend")],
    )];
    if let Some(nav_node) = header_nav {
        header_children.push(nav_node);
    }
    // ヘッダー右側のアクション群（GitHub リンク・テーマトグル、イシュー
    // #951）。`header_nav` の有無に関わらず無条件で第 3 子（or 第 2 子）と
    // して出力する（層 1 契約「見出しあり/なし両方のフィクスチャで出現する
    // こと」と同様、`docs_page`/`docs_page_with_assets` いずれの経路でも
    // 出現させるため条件分岐を作らない。`crate::site_theme::STRUCTURAL_CSS`
    // 参照）。
    header_children.push(div(
        vec![("class", "docs-header-actions")],
        vec![
            // 検索ブロック（イシュー #958）。既定 `hidden`、`<form>` で包まない
            // （JS 無効時に Enter で送信させないため、モジュール doc 参照）。
            // `input` は `role="combobox"`/`aria-controls`/`aria-expanded`/
            // `aria-autocomplete` で `ul#docs-search-results` と結合する
            // （WAI-ARIA combobox パターン。開閉・選択状態の更新は
            // `crate::script::SITE_JS` のみが行う）。
            div(
                vec![("class", "docs-search"), ("hidden", "")],
                vec![
                    // 視覚上は clip 手法で隠すラベル（`crate::site_theme::
                    // STRUCTURAL_CSS` の `.docs-search-label` 参照）。fandhe-backend
                    // の docs サイトとデザインを統一するため、`for`/`id` 対応を
                    // 持つ label 要素を追加した（backend `layout.rs` の
                    // `label.docs-search-label` と同型）。`id` は本ラベルの
                    // `for` 属性からのみ参照され、`crate::script::SITE_JS` は
                    // 引き続き `class="docs-search-input"` で要素を取得する
                    // （id 追加は既存の class セレクタ経路に影響しない）。
                    //
                    // ラベル文言は backend と同一の `"Search"` とする。下の
                    // `input` が `aria-label` を持つため、アクセシブル名の計算
                    // 順序上このラベルのテキストが読み上げられることはなく
                    // （`aria-label` が `<label>` より優先される）、実効的には
                    // `for`/`id` による関連付けの器として機能する。placeholder
                    // と同一文言を重複させても利用者には届かないため、backend
                    // との一致を優先した。
                    el(
                        "label",
                        vec![("class", "docs-search-label"), ("for", SEARCH_INPUT_ID)],
                        vec![text("Search")],
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
                            ("data-search-index", &search_index_href),
                        ],
                        vec![],
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
            ),
            // `target="_blank"` + `rel="noopener noreferrer"`（OWASP A05:
            // tabnabbing 対策。開いた先から `window.opener` を操作される
            // 経路と Referer 漏えいを防ぐ）。
            a(
                vec![
                    ("href", REPOSITORY_URL),
                    ("class", "docs-github-link"),
                    ("target", "_blank"),
                    ("rel", "noopener noreferrer"),
                ],
                vec![text("GitHub")],
            ),
            // 既定 `hidden`（JS 無効時・`site.js` の読み込み失敗時は
            // `crate::site_theme::STRUCTURAL_CSS` の `.docs-theme-toggle[hidden]`
            // が非表示を担保し、`prefers-color-scheme` 追従へ退避する）。
            // 可視化・イベント配線は `crate::script::SITE_JS` のみが行う
            // （`crate::script` モジュール doc 手順 5 参照）。
            button(
                vec![
                    ("type", "button"),
                    ("class", "docs-theme-toggle"),
                    ("hidden", ""),
                    ("aria-label", "Toggle color theme"),
                    ("aria-pressed", "false"),
                ],
                vec![text("Theme")],
            ),
        ],
    ));
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

    // `< 768px` の左ナビ折りたたみをタッチ操作でも開閉できるようにする
    // チェックボックスハック（設計文書 §3.2 の「マウス操作ユーザー向けの
    // 明示的な開閉トリガー」を採用、JS 不要）。開閉状態の唯一の情報源は
    // このチェックボックスの `:checked`（`crate::site_theme::STRUCTURAL_CSS`
    // 参照）とし、`:focus-within` は開状態の判定に加えない（キーボード
    // 操作でチェックを外してもフォーカスがナビ内に残っている限り閉じられ
    // ない回帰を避けるため、Bugbot 指摘 #916 是正）。チェックボックス自体は
    // Tab フォーカス・Space 操作の対象として DOM 上に残り続けるため、
    // クリップされたリンクへも Tab で到達しトグルを Space で開閉できる
    // （sr-only パターン、`display: none`/`visibility: hidden` にしない
    // 理由）。`nav_list`（`sidebar` 引数）自体の markup は変更しない
    // （設計文書 §3.4 の不変条件）。
    let sidebar_toggle_id = SIDEBAR_TOGGLE_ID;
    let sidebar_toggle = el(
        "input",
        vec![
            ("type", "checkbox"),
            ("id", sidebar_toggle_id),
            ("class", "docs-sidebar-toggle"),
        ],
        vec![],
    );
    let sidebar_toggle_label = el(
        "label",
        vec![
            ("for", sidebar_toggle_id),
            ("class", "docs-sidebar-toggle-label"),
        ],
        vec![text("Menu".to_string())],
    );

    // `div.docs-container` の子は「左ナビ / 中央コンテンツ / 右目次」の
    // 3 カラム順（設計文書 §3.1）。右目次カラムは見出しが 1 つも無いページ
    // では出力しない（`aside.docs-toc-aside` 自体を省略する。§3.3 の方針。
    // `nav.docs-toc` 単体で空 `nav` を出さない [`toc_nav`] の既存契約と揃える）。
    let has_toc = toc.is_some();
    let mut container_children = vec![
        aside(
            vec![("class", "docs-sidebar")],
            vec![sidebar_toggle, sidebar_toggle_label, sidebar],
        ),
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
    let container_class = if has_toc {
        "docs-container"
    } else {
        "docs-container docs-container--no-toc"
    };

    let body_node = el(
        "body",
        vec![],
        vec![
            skip_nav_link,
            header_node,
            div(vec![("class", container_class)], container_children),
        ],
    );

    el("html", vec![("lang", "ja")], vec![head, body_node])
}
