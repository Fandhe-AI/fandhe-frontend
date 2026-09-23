//! Wireframes（`/wireframes/`）セクションの Demo 供給機構（イシュー #2607、
//! 設計は `docs/design/wireframe-ui-architecture.md` §12 を正とする）。
//!
//! # 役割・呼び出し文脈
//!
//! `fandhe-frontend-wireframe-ui`（Primitives/Themes と独立した第 3 の UI
//! コンポーネント層、SSR 専用・非インタラクティブ）の部品ページを供給する。
//! `crate::showcase`（Themes）・`crate::primitive_showcase`（Primitives）・
//! `crate::blocks`（Blocks）と並ぶ第 4 の Rust 生成コンテンツ供給元だが、
//! [`crate::component_page`] の `generated_content`/[`crate::component_page::Layer`]
//! 経路には**一切乗せない**独立分岐として `crate::build::build_site` から
//! 呼ばれる（`crate::blocks` と同じ判断。`Layer::from_page_path` は
//! `/primitives/` 以外をすべて `Layer::Themes` と判定する全域関数のため、
//! Wireframes ページを同経路へ通すと `/themes/` 用の `pre-styled-ui.css`
//! 配線判定と混線し得る）。Wireframes は Themes/Primitives の原稿レジストリ
//! （`SPEC_TABLES`）・Anatomy/`data-*`/CSS 変数表の機械導出をいずれも使わない
//! ため、`Layer` へ統合するメリットがない（設計文書 §12 D1）。
//!
//! # ページ組み立て方式
//!
//! `crate::blocks` と同型で、Markdown 本文の**最初の `h2` の直前**へ
//! 「Demo」「引数表」の 2 節を挿入する（後方追記ではない）。[`insert_generated_sections`]
//! がこれを担い、`crate::build::build_site` は `render_markdown` の直後・
//! `linkcheck::rewrite_md_links` の前でこれを適用する。
//!
//! # レジストリ契約（Phase 1〜8 が複製する契約、設計文書 §12）
//!
//! [`Wireframe`] 1 件 = 1 部品ページ。#2607 時点では空レジストリだった
//! （掲載予定 49 部品の kebab はすべて Phase 1〜8（#2608〜#2665）の各部品
//! イシューが所有するため、#2607 で雛形実例ページを同梱すると kebab の
//! 衝突が起きる、設計文書 §12 D2）。Phase 2「テキスト・注釈」の
//! `annotation` 部品（イシュー #2617）が最初の要素を追加した。
//!
//! # CSS の置き場
//!
//! [`stylesheet`] がビルド時生成する専用 CSS（[`STYLESHEET_REL_PATH`]）は
//! `fandhe_frontend_wireframe_ui::wireframe_css()`（全部品 CSS を
//! `wireframe-ui` 側の `css::PARTS` へ集約済み）と、本モジュールが持つ
//! デモ枠専用 CSS（[`DEMO_CLASS`]）のみで構成する。部品追加時に本モジュールの
//! [`stylesheet`] を編集する必要はない（`crate::blocks` と異なり、部品ごとの
//! `LAYOUT_CSS` 追記点を持たない契約、設計文書 §12 D4）。
//!
//! # ダークモード（設計文書 §12 D5）
//!
//! デモ枠は `color-scheme: light` を固定で持つ。`wireframe-ui` のモノクロ
//! トークン（`--fw-wire-*`）は固定 light 値のみを持つ紙面メタファーであり、
//! docs サイトのテーマトグルでは反転させない。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 生成コンテンツはすべて `fandhe_frontend_core` のノード木 API で組み立て、
//! `raw_html()`・HTML 文字列の直接組み立て（`format!("<div>{}</div>", …)`）は
//! 使わない。[`ArgRow`] の各フィールドは `&'static str` に限定し、利用者入力が
//! 引数表へ流れ込む経路を型で塞ぐ。

mod annotation;
mod button;
mod divider;
mod grid;
mod link;
mod radio;
mod rich_text;
mod select;
mod stack;

use fandhe_frontend_core::{div, h2, p, table, tbody, td, text, th, thead, tr, Node};
use fandhe_frontend_pre_styled_ui::{StyleSheet, StylesheetError};

/// Wireframes 専用 CSS の出力先（`out_dir` 起点の相対パス）。`crate::build::build_site`
/// が [`stylesheet`] の内容をこのパスへ書き出し、`/wireframes/<kebab>/` ページの
/// みが `<link>` で参照する（`/wireframes/` 索引ページには配線しない）。
pub const STYLESHEET_REL_PATH: &str = "assets/wireframes.css";

/// [`insert_generated_sections`] が Demo 節のラッパへ常に付与する class。
pub const DEMO_CLASS: &str = "wireframes-demo";

/// 「原案差分メモ」節（原稿側の手書き H2）の見出し文言。設計文書 §12 D7 の
/// 契約テスト（Phase 1 以降が実効化）が本定数を用いて照会する。
pub const DIFF_NOTES_HEADING: &str = "原案差分メモ";

/// 引数表（`## 引数表`）1 行分。`component_page::ArgRow` と同型だが独立定義
/// とし、`Layer` 側の型との結合を作らない（設計文書 §12、モジュール doc参照）。
#[derive(Debug, Clone, Copy)]
pub struct ArgRow {
    /// 引数名。
    pub name: &'static str,
    /// 型（表示用の文字列表現）。
    pub kind: &'static str,
    /// 既定値（無ければ `"-"` 等の表示用プレースホルダーを渡す）。
    pub default: &'static str,
    /// 説明。
    pub description: &'static str,
}

/// Wireframes 部品 1 件分のレジストリエントリ。
#[derive(Clone, Copy)]
pub struct Wireframe {
    /// `site/nav.toml` の `page.path` と一致する部品ページの絶対パス
    /// （例: `/wireframes/button/`）。
    pub path: &'static str,
    /// 部品名（現状は未使用。将来利用に備え保持する）。
    pub title: &'static str,
    /// 引数表（`## 引数表`）の行一覧。0 行なら「引数はありません」を表示する。
    pub args: &'static [ArgRow],
    /// Demo 本体を組み立てる純関数。呼び出しごとに決定的な `Node` を返す。
    pub demo: fn() -> Node,
}

/// Wireframes レジストリ本体。`site/nav.toml` の `/wireframes/*` ページ
/// （索引を除く）との三方突合を `crates/docs-site/tests/wireframes_nav.rs`
/// が固定する。#2607 時点では空だったが、Phase 2「テキスト・注釈」の
/// [`annotation::WIREFRAME`]（イシュー #2617）を皮切りに、Phase 1
/// 「レイアウト骨格」の [`grid::WIREFRAME`]（イシュー #2611）・
/// [`divider::WIREFRAME`]（イシュー #2612）・[`stack::WIREFRAME`]
/// （イシュー #2610）、Phase 2 の [`link::WIREFRAME`]（イシュー #2618）・
/// [`rich_text::WIREFRAME`]（イシュー #2616）・Phase 3「Forms A」の
/// [`button::WIREFRAME`]（イシュー #2621）・[`select::WIREFRAME`]
/// （イシュー #2624）・[`radio::WIREFRAME`]（イシュー #2626、選択状態は
/// `props::Active` を再利用）が続いた。Phase 1・3 以降（#2608〜#2665）の
/// 残りの各部品イシューが自分の [`Wireframe`] 定数を 1 要素ずつ追記する。
pub const WIREFRAMES: &[Wireframe] = &[
    annotation::WIREFRAME,
    grid::WIREFRAME,
    divider::WIREFRAME,
    stack::WIREFRAME,
    link::WIREFRAME,
    rich_text::WIREFRAME,
    button::WIREFRAME,
    select::WIREFRAME,
    radio::WIREFRAME,
];

/// `page_path` に対応する [`Wireframe`] を返す（部品ページでなければ `None`）。
/// `crate::build::build_site` が「このページを Wireframes 専用分岐に乗せるか」
/// を判定する唯一の入口。
#[must_use]
pub fn wireframe_for_path(page_path: &str) -> Option<&'static Wireframe> {
    WIREFRAMES.iter().find(|w| w.path == page_path)
}

/// Markdown ブロック列（[`crate::markdown::render_markdown`] の戻り値）へ、
/// `page_path` が Wireframes 部品ページのときだけ「Demo」「引数表」の 2 節を
/// 最初の `h2` の直前へ挿入する（モジュール doc「ページ組み立て方式」節）。
/// 実体は [`WIREFRAMES`] を使う [`insert_generated_sections_with`] への委譲。
#[must_use]
pub fn insert_generated_sections(page_path: &str, base_path: &str, blocks: Vec<Node>) -> Vec<Node> {
    insert_generated_sections_with(WIREFRAMES, page_path, base_path, blocks)
}

/// [`insert_generated_sections`] の本体。レジストリをパラメータ化すること
/// で、[`WIREFRAMES`] が空の間もユニットテストが合成エントリで経路を検証
/// できる（`build_path`/`out_dir` 未使用の `base_path` 引数は `crate::blocks`
/// と同一シグネチャに揃えるための予約引数。将来の相互リンク実装に備える）。
#[must_use]
pub fn insert_generated_sections_with(
    registry: &[Wireframe],
    page_path: &str,
    _base_path: &str,
    blocks: Vec<Node>,
) -> Vec<Node> {
    let Some(wireframe) = registry.iter().find(|w| w.path == page_path) else {
        return blocks;
    };

    let arg_rows: Vec<Node> = if wireframe.args.is_empty() {
        Vec::new()
    } else {
        wireframe
            .args
            .iter()
            .map(|row| {
                tr(
                    vec![],
                    vec![
                        td(vec![], vec![text(row.name)]),
                        td(vec![], vec![text(row.kind)]),
                        td(vec![], vec![text(row.default)]),
                        td(vec![], vec![text(row.description)]),
                    ],
                )
            })
            .collect()
    };

    let args_section: Node = if wireframe.args.is_empty() {
        p(vec![], vec![text("引数はありません")])
    } else {
        table(
            vec![],
            vec![
                thead(
                    vec![],
                    vec![tr(
                        vec![],
                        vec![
                            th(vec![], vec![text("引数")]),
                            th(vec![], vec![text("型")]),
                            th(vec![], vec![text("既定値")]),
                            th(vec![], vec![text("説明")]),
                        ],
                    )],
                ),
                tbody(vec![], arg_rows),
            ],
        )
    };

    let generated = vec![
        h2(vec![], vec![text("Demo")]),
        div(vec![("class", DEMO_CLASS)], vec![(wireframe.demo)()]),
        h2(vec![], vec![text("引数表")]),
        args_section,
    ];

    splice_before_first_h2(blocks, generated)
}

/// `blocks` の先頭から見て最初の `h2` の直前へ `generated` を挿入する。
/// `h2` が存在しない場合は末尾へ追加する（`crate::blocks` と同型の
/// fail-closed 方針。`crates/docs-site/tests/wireframes_contract.rs` が
/// 節順序を検証して不整合を検知する）。
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

/// デモ枠（`.wireframes-demo`）専用 CSS。`--fw-wire-*` トークンを参照する
/// （設計文書 §12 D5「固定 light 値のみを持つ紙面メタファー」）。
const LAYOUT_CSS: &str = "\
.wireframes-demo {\n  max-width: 100%;\n  overflow-x: auto;\n  border: 1px solid var(--fw-wire-line);\n  border-radius: var(--fw-wire-radius);\n  padding: 1.5rem;\n  margin: 0 0 1.5rem;\n  background: var(--fw-wire-paper);\n  color: var(--fw-wire-ink);\n  color-scheme: light;\n}\n";

/// Wireframes 専用 CSS を組み立てる（モジュール doc「CSS の置き場」節）。
///
/// `push_theme(&Theme::default())` は行わない。`--fandhe-*` トークン
/// （pre-styled-ui）を wireframes.css へ持ち込まないためである（`--fw-wire-*`
/// と意図的に別プレフィックス、設計文書 §10.1/§12 D4）。
///
/// # Errors
///
/// `LAYOUT_CSS`/`wireframe_css()` は静的な検証済み文字列であり実質的に
/// 失敗しないが、型レベルでの契約（`StyleSheet::push_css` の検証、`<`・
/// 制御文字の拒否）を呼び出し元（`crate::build::build_site`）へ伝播する。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_css(fandhe_frontend_wireframe_ui::wireframe_css())?;
    sheet.push_css(LAYOUT_CSS)?;
    Ok(sheet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{h1, render};

    fn sample_wireframe() -> Wireframe {
        Wireframe {
            path: "/wireframes/sample/",
            title: "Sample",
            args: &[ArgRow {
                name: "label",
                kind: "&str",
                default: "-",
                description: "<script>alert(1)</script>",
            }],
            demo: || div(vec![], vec![text("demo body")]),
        }
    }

    #[test]
    fn wireframe_for_path_finds_nothing_in_empty_registry() {
        // `/wireframes/input/` は Phase 3「Forms A」の未実装部品（イシュー
        // #2624 時点では button/select のみ登録済み）であり、恒久的に
        // 未登録のパスとして使える。
        assert!(wireframe_for_path("/wireframes/input/").is_none());
    }

    #[test]
    fn insert_generated_sections_is_noop_for_non_wireframe_pages() {
        let blocks = vec![p(vec![], vec![text("hello")])];
        let result = insert_generated_sections("/wireframes/", "/fandhe-frontend", blocks.clone());
        assert_eq!(result, blocks);
    }

    #[test]
    fn insert_generated_sections_with_splices_before_first_h2() {
        let registry = [sample_wireframe()];
        let blocks = vec![
            h1(vec![], vec![text("sample")]),
            h2(vec![], vec![text("原案差分メモ")]),
        ];
        let result = insert_generated_sections_with(
            &registry,
            "/wireframes/sample/",
            "/fandhe-frontend",
            blocks,
        );
        // 挿入結果: [h1相当, Demo(h2), div, 引数表(h2), table, 元の h2("原案差分メモ")]
        assert_eq!(result.len(), 6);
        let html = render(&div(vec![], result));
        assert!(html.contains(">Demo<"));
        assert!(html.contains("引数表"));
        assert!(html.contains("原案差分メモ"));
        assert!(html.contains(r#"class="wireframes-demo""#));
        assert!(html.contains("demo body"));
        // XSS 回帰: description に含めた <script> がエスケープされること。
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn insert_generated_sections_with_appends_when_no_h2_present() {
        let registry = [sample_wireframe()];
        let blocks = vec![p(vec![], vec![text("only a paragraph")])];
        let result = insert_generated_sections_with(
            &registry,
            "/wireframes/sample/",
            "/fandhe-frontend",
            blocks,
        );
        // 元の p の後ろへ Demo(h2)/div/引数表(h2)/table の 4 要素が追加される。
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn insert_generated_sections_with_shows_no_args_message_for_empty_args() {
        let mut wireframe = sample_wireframe();
        wireframe.args = &[];
        let registry = [wireframe];
        let blocks = vec![p(vec![], vec![text("body")])];
        let result = insert_generated_sections_with(
            &registry,
            "/wireframes/sample/",
            "/fandhe-frontend",
            blocks,
        );
        let html = render(&div(vec![], result));
        assert!(html.contains("引数はありません"));
    }

    #[test]
    fn stylesheet_builds_and_contains_demo_frame_and_wireframe_css() {
        let sheet = stylesheet().expect("stylesheet must build");
        let css = sheet.as_css();
        assert!(css.contains(".wireframes-demo"));
        assert!(css.contains("overflow-x: auto"));
        assert!(css.contains("color-scheme: light"));
        assert!(css.contains(".fw-wire-icon-glyph"));
        assert!(!css.contains("--fandhe-"));
    }
}
