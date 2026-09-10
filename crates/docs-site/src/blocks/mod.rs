//! Blocks（`/blocks/`）セクションの Demo 供給機構（イシュー #2088、設計は
//! `docs/design/docs-site-blocks-section.md` を正とする）。
//!
//! # 役割・呼び出し文脈
//!
//! shadcn/ui Blocks 相当の「既存部品（Themes/Primitives）を合成した実例」を
//! 掲載するセクション。`crate::showcase`（Themes 部品ページ）・
//! `crate::primitive_showcase`（Primitives 部品ページ）と並ぶ第 3 の
//! Rust 生成コンテンツ供給元だが、[`crate::component_page`] の
//! `generated_content`/[`crate::component_page::Layer`] 経路には**一切乗せない**
//! 独立分岐として `crate::build::build_site` から呼ばれる（
//! `Layer::from_page_path` は `/primitives/` 以外をすべて `Layer::Themes` と
//! 判定する全域関数のため、Blocks ページを同経路へ通すと `/themes/` 用の
//! `pre-styled-ui.css` 配線判定と混線し得る。混線を構造で避けるための分離）。
//!
//! # ページ組み立て方式（既存の「後方追記」との違い）
//!
//! `component_page::generated_content` は Markdown 本文の**後ろへ**生成
//! コンテンツを追記するが、Blocks ページの IA（H1 → Demo → 使用部品 →
//! 手書き Rust コード → 任意の差分メモ）は生成コンテンツを本文の**途中**
//! （最初の `h2` の直前）へ挿入する必要がある。[`insert_generated_sections`]
//! がこれを担い、`crate::build::build_site` は `render_markdown` の直後・
//! `linkcheck::rewrite_md_links` の前でこれを適用する（生成ノードは
//! 絶対パス href のみを持ち `.md` 相対リンクを含まないため、rewrite の
//! 前後どちらで挿入しても結果は変わらない）。
//!
//! # レジストリ構造（後続イシュー #2089〜#2095 が複製する契約）
//!
//! [`Block`] 1 件 = 1 block ページ。`rust_source` は
//! [`crate::blocks`]（本モジュール）配下の実装ファイルパスを指し、
//! `crates/docs-site/tests/blocks_code_drift.rs` が手書き Markdown の
//! ```rust フェンスとの一致検証に使う。`parts` は Demo が実際に使用する
//! Themes/Primitives 部品への相互リンク一覧（`crates/docs-site/tests/
//! blocks_contract.rs` が各 `path` の妥当性を検証する動機は
//! `linkcheck::check_links` の fail-closed 保証と同型）。
//!
//! # マーカー規約（`.rs` 側 ⇔ `.md` 側の一致検証、イシュー #2088 §2.5）
//!
//! 各 block 実装モジュール（例: [`login_01`]）は `// blocks-code:begin` /
//! `// blocks-code:end` の行マーカーで `use` 宣言 + `pub fn demo() -> Node`
//! を囲む。対応する `site/blocks/<kebab>.md` の最初の ```rust フェンス本文が
//! マーカー内の行（両端マーカー行を除く、末尾空白のみ trim 許容）と完全一致
//! することを [`crate::blocks`] 配下ソースへ機械検証する
//! （`blocks_code_drift.rs`）。モジュール doc（`//!`）はマーカー外。
//!
//! # `<form>` を使わない（`crate::layout` モジュール doc 参照）
//!
//! docs サイトは JS 非活性時に Enter キーで暗黙 submit が起きないよう
//! `<form>` 要素を出力しない方針を持つ（`tests/layout_render.rs`）。
//! login/signup 系 block は入力欄を持つが、`<form>` で包まず `div` で
//! 構造化し、ボタンは `fandhe_frontend_pre_styled_ui::button::button` の
//! 既定 `type="button"` のまま用いる（暗黙 submit も起き得ない）。
//! `crates/docs-site/tests/blocks_contract.rs` がこの不変条件を
//! fail-closed に固定し、後続の login/signup 4 件（#2089〜#2091）が継承する。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! マークアップはすべて `fandhe_frontend_core` のノード木 API と
//! `fandhe_frontend_pre_styled_ui` のパート関数のみで組み立てる。
//! `raw_html()`・HTML 文字列の直接組み立て（`format!("<div>{}</div>", …)`）は
//! 使わない。ブランド名・ダミー文字列は架空のもの（実企業名・実クレデンシャル・
//! PII を含まない）に限る。ログイン/サインアップ系 block は認証処理・送信先を
//! 一切持たない静的な合成例である旨を Markdown 原稿側の導入文で明記する。
//!
//! # CSS の置き場
//!
//! [`stylesheet`] がビルド時生成する専用 CSS（[`STYLESHEET_REL_PATH`]）が
//! `.blocks-demo`（全幅デモ枠、`.docs-content` カラム内で最大幅・横スクロール
//! はデモ枠内に閉じ込める）と block 固有レイアウト（[`Block::demo_class`]）を
//! 持つ。block ページには本 CSS に加えて `crate::showcase::STYLESHEET_REL_PATH`
//! （pre-styled-ui 全 recipe。合成に使う部品自体の見た目）も配線する
//! （`/blocks/` 索引ページには配線しない）。

mod dashboard_01;
mod login_01;
mod login_04;
mod sidebar_03;
mod sidebar_07;
mod signup_01;

use fandhe_frontend_core::{a, div, h2, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::{StyleSheet, StylesheetError};

use crate::layout;

/// Blocks 専用 CSS の出力先（`out_dir` 起点の相対パス）。`crate::build::build_site`
/// が [`stylesheet`] の内容をこのパスへ書き出し、`/blocks/<kebab>/` ページの
/// みが `<link>` で参照する（`/blocks/` 索引ページには配線しない）。
pub const STYLESHEET_REL_PATH: &str = "assets/blocks.css";

/// [`insert_generated_sections`] が Demo ラッパへ常に付与する class
/// （block 固有 class は [`Block::demo_class`] として追加で付与する）。
pub const DEMO_CLASS: &str = "blocks-demo";

/// Demo 節の全幅ラッパ（`.blocks-demo`）のみを持つ共通 CSS
/// （モジュール doc「CSS の置き場」節参照）。block 固有のレイアウト規則は
/// 各モジュール側の `pub(super) const LAYOUT_CSS`（`login_01` を含む全 block
/// が個別に持つ、下記「block 固有 CSS の置き場」節参照）に置く。
///
/// # セレクタが `class` と `[data-*]` で混在する理由（イシュー #2088 PR #2277
/// codex-review P1 / Cursor Bugbot 指摘の是正）
///
/// `fandhe_frontend_pre_styled_ui::card::root` / `field::root` /
/// `button::button` は variant クラスを自ら付与するパーツであり、
/// `crate::class_attr::drop_class_attr`（pre-styled-ui 側）により呼び出し側
/// `attrs` の `class` を黙って除去してから合成する契約を持つ。このため
/// `login_01.rs` はこれら 3 パーツの Demo 固有スタイルを `class` ではなく
/// 呼び出し側 `attrs` にそのまま残る `data-*` 属性（`data-blocks-login-01-*`）
/// で渡し、本 CSS 側も `[data-blocks-login-01-*]` 属性セレクタで対応する
/// （`blocks-login-01-card`/`-field`/`-submit`）。一方 `card::header`/
/// `card::body`（variant を持たず `attrs` をそのまま連結する）や素の `div`
/// には `class` がそのまま効くため、それらは従来どおり `.blocks-login-01-*`
/// クラスセレクタのままでよい（`-password-row`/`-actions`/`-signup-row`）。
/// 後続 block（#2089〜#2095）が `card::root`/`field::root`/`button::button`
/// を使う際は同じ判断（対象パーツが `drop_class_attr` を経由するか）で
/// `class` か `data-*` かを選ぶ。
/// 実際に生成 HTML へ属性が出力され CSS 側のセレクタと対になっていることは
/// `crates/docs-site/tests/blocks_contract.rs` が固定する。
///
/// # block 固有 CSS の置き場（イシュー #2089 以降の分離、並列進行対策）
///
/// 本 `LAYOUT_CSS` は `.blocks-demo` の共通枠のみを持つ（`login_01` も
/// #2092 で `pub(super) const LAYOUT_CSS` へ分離済み）。#2089〜#2095 が
/// 並列進行する状況で全 block が単一定数へ追記すると PR 間で必ず衝突するため、
/// 各モジュール側が個別に `pub(super) const LAYOUT_CSS` を持ち、[`stylesheet`]
/// が `push_css` を複数回呼んで連結する（詳細は
/// `docs/design/docs-site-blocks-section.md` §10 追記節）。
const LAYOUT_CSS: &str = "\
.blocks-demo {\n  max-width: 100%;\n  overflow-x: auto;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: 0.5rem;\n  padding: 1.5rem;\n  margin: 0 0 1.5rem;\n  background: var(--fandhe-color-bg-subtle);\n}\n";

/// 使用部品一覧の 1 件（`## 使用部品` の `<li><a>`）。`path` は
/// `/themes/<kebab>/` または `/primitives/<kebab>/` を指す。
#[derive(Debug, Clone, Copy)]
pub struct Part {
    /// リンクの表示テキスト（部品名）。
    pub label: &'static str,
    /// リンク先ページの絶対パス（`base_path` を含まない、`layout::asset_href`
    /// が付与する）。
    pub path: &'static str,
}

/// block 1 件分のレジストリエントリ（モジュール doc「レジストリ構造」節）。
#[derive(Clone, Copy)]
pub struct Block {
    /// `site/nav.toml` の `page.path` と一致する block ページの絶対パス。
    pub path: &'static str,
    /// block 名（`## Demo` 節の見出しにはならないが `title` 属性等の将来利用
    /// に備え保持する。現状は [`insert_generated_sections`] からは未使用）。
    pub title: &'static str,
    /// 対応する実装ファイルの repo 相対パス（`blocks_code_drift.rs` が
    /// マーカー内容と手書き Markdown のフェンスを突合する際に使う）。
    pub rust_source: &'static str,
    /// Demo ラッパへ [`DEMO_CLASS`] に加えて付与する block 固有 class
    /// （[`LAYOUT_CSS`] のセレクタと一致させる）。
    pub demo_class: &'static str,
    /// Demo が使用する Themes/Primitives 部品一覧（`## 使用部品`）。
    pub parts: &'static [Part],
    /// Demo 本体を組み立てる純関数。呼び出しごとに決定的な `Node` を返す
    /// （状態機械を持たない、他の Rust 生成コンテンツ供給元と同じ設計）。
    pub demo: fn() -> Node,
}

/// Blocks レジストリ本体。`site/nav.toml` の `/blocks/*` ページ（索引を除く）
/// との三方突合を `crates/docs-site/tests/blocks_nav.rs` が固定する。
pub const BLOCKS: &[Block] = &[
    login_01::BLOCK,
    login_04::BLOCK,
    dashboard_01::BLOCK,
    sidebar_07::BLOCK,
    sidebar_03::BLOCK,
    signup_01::BLOCK,
];

/// `page_path` に対応する [`Block`] を返す（block ページでなければ `None`）。
/// `crate::build::build_site` が「このページを Blocks 専用分岐に乗せるか」を
/// 判定する唯一の入口。
#[must_use]
pub fn block_for_path(page_path: &str) -> Option<&'static Block> {
    BLOCKS.iter().find(|block| block.path == page_path)
}

/// Markdown ブロック列（[`crate::markdown::render_markdown`] の戻り値）へ、
/// `page_path` が block ページのときだけ「Demo」「使用部品」の 2 節を
/// 最初の `h2` の直前へ挿入する（モジュール doc「ページ組み立て方式」節）。
/// 該当しないページ・`h2` が 1 個も無い block ページ（末尾へ追加）の
/// いずれでも全域に振る舞う。
#[must_use]
pub fn insert_generated_sections(page_path: &str, base_path: &str, blocks: Vec<Node>) -> Vec<Node> {
    let Some(block) = block_for_path(page_path) else {
        return blocks;
    };

    let demo_class = format!("{DEMO_CLASS} {}", block.demo_class);
    let parts_items: Vec<Node> = block
        .parts
        .iter()
        .map(|part| {
            li(
                vec![],
                vec![a(
                    vec![("href", layout::asset_href(base_path, part.path).as_str())],
                    vec![text(part.label)],
                )],
            )
        })
        .collect();

    let generated = vec![
        h2(vec![], vec![text("Demo")]),
        div(vec![("class", demo_class.as_str())], vec![(block.demo)()]),
        h2(vec![], vec![text("使用部品")]),
        ul(vec![], parts_items),
    ];

    splice_before_first_h2(blocks, generated)
}

/// `blocks` の先頭から見て最初の `h2` の直前へ `generated` を挿入する。
/// `h2` が存在しない場合は末尾へ追加する（fail-open にしない代わりに、
/// `crates/docs-site/tests/blocks_contract.rs` が節順序を検証して
/// 不整合を検知する）。
fn splice_before_first_h2(mut blocks: Vec<Node>, generated: Vec<Node>) -> Vec<Node> {
    let insert_at = blocks
        .iter()
        .position(|node| matches!(node, Node::Element { tag, .. } if *tag == "h2"))
        .unwrap_or(blocks.len());
    let tail = blocks.split_off(insert_at);
    blocks.extend(generated);
    blocks.extend(tail);
    blocks
}

/// Blocks 専用 CSS を組み立てる（`primitive_showcase::stylesheet` と同型の
/// 「各生成 CSS ファイルは単独でも自己完結する」規約に従い、テーマトークン
/// 定義（`Theme::default()`）を含める）。
///
/// # Errors
///
/// [`LAYOUT_CSS`] は静的な検証済み文字列であり実質的に失敗しないが、型
/// レベルでの契約（`StyleSheet::push_css` の検証、`<`・制御文字の拒否）を
/// 呼び出し元（`crate::build::build_site`）へ伝播する。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&Theme::default());
    sheet.push_css(LAYOUT_CSS)?;
    sheet.push_css(login_01::LAYOUT_CSS)?;
    sheet.push_css(login_04::LAYOUT_CSS)?;
    sheet.push_css(dashboard_01::LAYOUT_CSS)?;
    sheet.push_css(sidebar_07::LAYOUT_CSS)?;
    sheet.push_css(sidebar_03::LAYOUT_CSS)?;
    sheet.push_css(signup_01::LAYOUT_CSS)?;
    Ok(sheet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{h1, p, render};

    #[test]
    fn block_for_path_finds_registered_block() {
        assert!(block_for_path("/blocks/login-01/").is_some());
        assert!(block_for_path("/blocks/no-such-block/").is_none());
    }

    #[test]
    fn insert_generated_sections_is_noop_for_non_block_pages() {
        let blocks = vec![p(vec![], vec![text("hello")])];
        let result = insert_generated_sections("/blocks/", "/fandhe-frontend", blocks.clone());
        assert_eq!(result, blocks);
    }

    #[test]
    fn insert_generated_sections_splices_before_first_h2() {
        let blocks = vec![
            h1(vec![], vec![text("login-01")]),
            h2(vec![], vec![text("Rust コード")]),
        ];
        let result = insert_generated_sections("/blocks/login-01/", "/fandhe-frontend", blocks);
        // 挿入結果: [h1相当, Demo(h2), div, 使用部品(h2), ul, 元の h2("Rust コード")]
        assert_eq!(result.len(), 6);
        let html = render(&div(vec![], result));
        assert!(html.contains(">Demo<"));
        assert!(html.contains("使用部品"));
        assert!(html.contains("Rust コード"));
        assert!(html.contains(r#"class="blocks-demo blocks-login-01""#));
        assert!(html.contains(r#"href="/fandhe-frontend/themes/card/""#));
    }

    #[test]
    fn insert_generated_sections_appends_when_no_h2_present() {
        let blocks = vec![p(vec![], vec![text("only a paragraph")])];
        let result = insert_generated_sections("/blocks/login-01/", "/fandhe-frontend", blocks);
        // 元の p の後ろへ Demo(h2)/div/使用部品(h2)/ul の 4 要素が追加される。
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn stylesheet_builds_and_contains_demo_frame_overflow() {
        let sheet = stylesheet().expect("stylesheet must build");
        assert!(sheet.as_css().contains(".blocks-demo"));
        assert!(sheet.as_css().contains("overflow-x: auto"));
    }
}
