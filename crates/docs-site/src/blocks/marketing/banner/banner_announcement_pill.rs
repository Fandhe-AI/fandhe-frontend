//! `banner-announcement-pill` block（イシュー #2739。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充」配下、Phase 1 親 #2738「マーケティング A」の
//! block の 1 つ。Marketing / Banner カテゴリは既にディレクトリ化済み
//! （イシュー #2741「banner-email-signup」が同カテゴリ最初の block として
//! 空雛形から `git mv` した実績があり、本ファイルはその既存カテゴリへの
//! 追加である。カテゴリの卒業手順は `docs/design/docs-site-blocks-section.md`
//! §18 参照）。
//!
//! # レイアウト
//!
//! 中央寄せの角丸ピル 1 個の中に、短い告知文と矢印アイコンを横並びに
//! 置く。ピル全体を 1 個のリンクにする。2 つのバリエーション（基準形＝
//! ピルだけの告知リンク／利用者の声を示す重なりアバター群付き）を、
//! 明暗 2 種の見た目とあわせて計 4 パターン Demo に並べる。参照元
//! （private 素材）の文言・配色・装飾・アイコンは持ち込まず、独自の
//! 抽象図形と英語ダミー文言のみで構成する。
//!
//! # 使用部品
//!
//! `link`（ピル本体、1 個のクリック可能領域） / `icon`（矢印、自作幾何
//! SVG） / `avatar`（重なりアバター群、[`crate::blocks::dummy_assets`] の
//! 架空素材）の 3 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `link::root`/`icon::icon`/`avatar::root` は `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、これら
//! 3 パーツへの Demo 固有スタイルは `data-blocks-banner-announcement-
//! pill-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する
//! （`footer_newsletter`/`testimonials_stack` と同型の判断）。一方 Demo
//! 骨格を組む素の `div`/`span`/`avatar::group` には `class` がそのまま
//! 効くため、それらは従来どおり `.blocks-banner-announcement-pill-*`
//! クラスセレクタを使う。
//!
//! # `href` に絶対 URL を使う理由
//!
//! `Block::demo` は `fn() -> Node` で `base_path` を受け取らないため、
//! サイト内の絶対パスをリンク先にできない。`linkcheck::check_links` は
//! fragment のみの `href="#"`（対応する id が無い場合）を broken として
//! fail-closed に検知するため、`footer_newsletter`/`footer_sticky_reveal`
//! と同じ判断でリポジトリの絶対 URL（[`REPO`]）をリンク先の定数にする。
//!
//! # 明暗 2 種をトークン反転で表現する理由
//!
//! docs サイトのテーマ切り替え（`:root[data-theme="dark"]`/
//! `prefers-color-scheme`）は `:root` にのみ効くため、同一ページ内で
//! 「明るい面」「暗い面」を同時に見せることはできない。本 Demo は
//! `data-blocks-banner-announcement-pill-surface="dark"` を付けた面で
//! `background: var(--fandhe-color-fg)` / `color: var(--fandhe-color-bg)`
//! へトークンを反転させることで、無 JS のまま 2 面を並べて見せる
//! （docs サイトを dark テーマで閲覧すると明暗の見た目が入れ替わる点は
//! 許容済みのトレードオフ）。
//!
//! # アバター群を `aria-hidden` にする a11y 判断
//!
//! ピル全体が 1 個のリンクであり、アクセシブルネームは可視テキスト
//! （告知文の `span`）から決まることが望ましい。重なりアバター群は
//! 装飾的な社会的証明であり、リンク名へ余計な文言（架空イニシャル）が
//! 混入しないよう `avatar::group` に `aria-hidden="true"` を付与する。
//! 矢印アイコンも `icon::IconProps::label: None`（既定）のまま渡し、
//! `aria-hidden="true"` を保つ。
//!
//! # `<form>` を出さない（REQ-1）
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` は出力しない。
//! 送信処理・遷移先の検証は本 block の範囲外である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先（`docs/design/docs-site-blocks-section.md` §3 と同じ判断で
/// 実在の自リポジトリへの絶対 URL を使う。モジュール doc「`href` に
/// 絶対 URL を使う理由」参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// [`avatar_stack`] が使う架空イニシャル 3 件（[`dummy_assets::PERSON_NAMES`]
/// 先頭 3 名から手書きした定数）。
const AVATAR_INITIALS: [&str; 3] = ["HF", "EV", "KB"];

/// 矢印アイコン（右向き、2 本の `path` からなる自作幾何 SVG）。装飾用途
/// のため `label: None`（既定）のまま渡し `aria-hidden="true"` を保つ。
fn arrow_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![("data-blocks-banner-announcement-pill-arrow", "")],
        vec![
            el(
                "path",
                vec![
                    ("d", "M5 12h14"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "m13 6 6 6-6 6"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 重なりアバター群（社会的証明の装飾。モジュール doc「アバター群を
/// `aria-hidden` にする a11y 判断」参照）。[`dummy_assets::AVATAR_SRC`]
/// （ビルド時生成 SVG、`data:` URI ではない）を画像に使い、
/// [`AVATAR_INITIALS`] を fallback イニシャルに使う。
fn avatar_stack() -> Node {
    let avatars: Vec<Node> = AVATAR_INITIALS
        .iter()
        .map(|initials| {
            avatar::root(
                &AvatarProps {
                    size: Size::Xs,
                    stacked: true,
                    ..AvatarProps::default()
                },
                vec![("data-blocks-banner-announcement-pill-avatar", "")],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
                    avatar::fallback(ImageStatus::Loaded, vec![], vec![text(*initials)]),
                ],
            )
        })
        .collect();
    avatar::group(
        vec![
            ("class", "blocks-banner-announcement-pill-avatars"),
            ("aria-hidden", "true"),
        ],
        avatars,
    )
}

/// ピル本体を 1 個組み立てる。`dark` はピル自身の配色反転
/// （`data-blocks-banner-announcement-pill-tone="dark"`）、`with_avatars`
/// は先頭への [`avatar_stack`] の有無（R0407/R0408 の差分）を表す。
fn pill(dark: bool, with_avatars: bool) -> Node {
    let mut attrs = vec![("data-blocks-banner-announcement-pill-pill", "")];
    if dark {
        attrs.push(("data-blocks-banner-announcement-pill-tone", "dark"));
    }

    let mut children: Vec<Node> = Vec::new();
    if with_avatars {
        children.push(avatar_stack());
    }
    let message = if with_avatars {
        "Trusted by thousands of builders"
    } else {
        "Blocks gallery just got a big update"
    };
    children.push(span(
        vec![("class", "blocks-banner-announcement-pill-label")],
        vec![
            span(
                vec![("class", "blocks-banner-announcement-pill-highlight")],
                vec![text("New")],
            ),
            span(
                vec![("class", "blocks-banner-announcement-pill-message")],
                vec![text(message)],
            ),
        ],
    ));
    children.push(arrow_icon());

    link::root(REPO, &LinkProps::default(), attrs, children)
}

/// 1 面分（Light/Dark どちらか一方 × 基準形/アバター付きどちらか一方）を
/// 組み立てる。`caption` は Demo 上で R0407/R0408・明暗の違いを読み取れる
/// ようにするキャプション文言。
fn cell(caption: &'static str, dark: bool, with_avatars: bool) -> Node {
    let mut attrs = vec![("class", "blocks-banner-announcement-pill-cell")];
    if dark {
        attrs.push(("data-blocks-banner-announcement-pill-surface", "dark"));
    }
    div(
        attrs,
        vec![
            span(
                vec![("class", "blocks-banner-announcement-pill-caption")],
                vec![text(caption)],
            ),
            pill(dark, with_avatars),
        ],
    )
}

/// `banner-announcement-pill` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。基準形（R0407）とアバター付き（R0408）を、明暗 2 種の
/// 面として計 4 パターン並べる（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-banner-announcement-pill-grid")],
        vec![
            cell("Light", false, false),
            cell("Dark", true, false),
            cell("Light · with avatars", false, true),
            cell("Dark · with avatars", true, true),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/banner-announcement-pill/",
    title: "banner-announcement-pill",
    category: BlockCategory::Banner,
    rust_source: "crates/docs-site/src/blocks/marketing/banner/banner_announcement_pill.rs",
    demo_class: "blocks-banner-announcement-pill",
    parts: &[
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `banner_announcement_pill` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型で
/// ファイル内 private として `super::stylesheet`（`crate::blocks::
/// stylesheet`）から連結される）。
///
/// 文字色は `link` recipe の base `color`（詳細度 (0,2,0)）と競合する
/// のを避けるため、ピル root（`[data-scope="link"][data-part="root"]`）の
/// `color` は上書きせず、子の `span`/`svg` 側だけに色を持たせる
/// （モジュール doc「CSS フックの選び方」参照）。
///
/// 矢印アイコンの色上書きは `icon` recipe の base `color: currentColor`
/// （`[data-scope="icon"][data-part="root"]`、詳細度 (0,2,0)）に必ず
/// 勝てる詳細度を持たせる必要がある。単一の属性セレクタ
/// `[data-blocks-banner-announcement-pill-arrow]`（詳細度 (0,1,0)）では
/// ソース順に関わらず icon recipe に負けて `currentColor`（`link` の
/// アクセントカラー）を継承してしまうため、親ピルの属性セレクタと
/// 連結した子孫セレクタ（属性セレクタ 2 個 = 詳細度 (0,2,0)）へ
/// 揃えている。`blocks.css` は `pre-styled-ui.css` より後に `<link>` される
/// ため、詳細度が並んだ場合はソース順で本 CSS が勝つ。
///
/// 重なりアバターのリング色（`box-shadow`）反転も同じ理由で詳細度を
/// 揃える必要がある。`avatar` recipe の `stacked` variant（`[data-scope=
/// "avatar"][data-part="root"].fd-avatar--stack-stacked`、詳細度
/// (0,3,0)）に必ず勝つよう、ピル root 自身が持つ 2 属性セレクタ
/// （`[data-blocks-banner-announcement-pill-pill]`・
/// `[data-blocks-banner-announcement-pill-tone="dark"]`、いずれも同じ
/// `link::root` 要素に付く）と子孫の `[data-blocks-banner-announcement-
/// pill-avatar]` を連結した属性セレクタ 3 個（詳細度 (0,3,0)）へ揃え、
/// ソース順（`blocks.css` が後勝ち）で `--fandhe-color-bg`（明るいまま）
/// から `--fandhe-color-fg-muted`（tone dark の背景色と同じ）へ上書きする。
///
/// アバター付きバリエーションが Demo グリッドをはみ出さないよう、
/// `.blocks-banner-announcement-pill-cell`（グリッドアイテム）へ
/// `min-width: 0` を明示している。CSS Grid のアイテムは既定で
/// `min-width: auto`（内容の min-content 基準）を取るため、これが無いと
/// `grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr))` の
/// トラック幅 16rem を、アバター群 + 折り返さない告知文の min-content
/// 幅が上回ってトラックごとはみ出す（デスクトップ 4 カラム・狭幅
/// ビューポート双方で発生）。あわせてピル本体
/// （`[data-blocks-banner-announcement-pill-pill]`）と
/// `.blocks-banner-announcement-pill-label` へ `flex-wrap: wrap` /
/// `min-width: 0` を足し、セルが縮んだ場合はアバター群・ラベルが
/// 複数行に折り返してピル内部でのオーバーフローも防ぐ。
const LAYOUT_CSS: &str = "\
.blocks-banner-announcement-pill-grid {\n  display: grid;\n  gap: 1rem;\n  grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));\n}\n\
.blocks-banner-announcement-pill-cell {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 0.75rem;\n  padding: 2rem 1rem;\n  min-width: 0;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg);\n  border: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-banner-announcement-pill-surface=\"dark\"] {\n  background: var(--fandhe-color-fg);\n  border-color: var(--fandhe-color-fg);\n}\n\
.blocks-banner-announcement-pill-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-banner-announcement-pill-surface=\"dark\"] .blocks-banner-announcement-pill-caption {\n  color: var(--fandhe-color-bg);\n}\n\
[data-blocks-banner-announcement-pill-pill] {\n  display: inline-flex;\n  align-items: center;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-1-5) 1rem;\n  border-radius: var(--fandhe-radius-full);\n  border: 1px solid var(--fandhe-color-border);\n  background: var(--fandhe-color-bg-subtle);\n  font-size: 0.875rem;\n  max-width: 100%;\n  transition: border-color var(--fandhe-motion-duration-fast) var(--fandhe-motion-easing-standard);\n}\n\
[data-blocks-banner-announcement-pill-pill]:hover {\n  border-color: var(--fandhe-color-border-emphasized);\n}\n\
[data-blocks-banner-announcement-pill-tone=\"dark\"] {\n  background: var(--fandhe-color-fg-muted);\n  border-color: var(--fandhe-color-border-emphasized);\n}\n\
.blocks-banner-announcement-pill-label {\n  display: inline-flex;\n  align-items: center;\n  flex-wrap: wrap;\n  min-width: 0;\n  gap: 0.375rem;\n  color: var(--fandhe-color-fg);\n}\n\
[data-blocks-banner-announcement-pill-tone=\"dark\"] .blocks-banner-announcement-pill-label {\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-banner-announcement-pill-highlight {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\
.blocks-banner-announcement-pill-message {\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-banner-announcement-pill-tone=\"dark\"] .blocks-banner-announcement-pill-message {\n  color: var(--fandhe-color-bg);\n}\n\
[data-blocks-banner-announcement-pill-pill] [data-blocks-banner-announcement-pill-arrow] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-banner-announcement-pill-tone=\"dark\"] [data-blocks-banner-announcement-pill-arrow] {\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-banner-announcement-pill-avatars {\n  display: inline-flex;\n}\n\
[data-blocks-banner-announcement-pill-pill][data-blocks-banner-announcement-pill-tone=\"dark\"] [data-blocks-banner-announcement-pill-avatar] {\n  box-shadow: 0 0 0 2px var(--fandhe-color-fg-muted);\n}\n\
";
