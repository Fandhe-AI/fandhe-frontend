//! 部品ページ雛形レンダラ（イシュー #942、親 #927、設計
//! `docs/design/docs-site-component-pages.md` §7）。
//!
//! # 役割・呼び出し文脈
//!
//! `crate::build::build_site` は `page.path` ごとに Rust 生成コンテンツを
//! 1 回だけ照会する（`showcase::generated_content` 直呼びだった箇所を本
//! モジュールの [`generated_content`] へ置き換える）。本モジュールは
//! [`crate::showcase`]（イシュー #941 で `path -> 部品デモ` レジストリへ
//! 分解済み）の出力を「Demo」節として受け取り、Radix / Ark UI 流の
//! 読み物構造（Demo → Features → Anatomy → API Reference → Examples →
//! Accessibility の 6 節、H2 固定）へ組み立て直す。
//!
//! - [`showcase::PAGE_PATH`]（`/components/pre-styled-ui/` 索引ページ）は
//!   イシュー #943 で索引（凡例 + カテゴリ別リンク集）へ改組済みであり、
//!   Rust 生成コンテンツを持たない（[`showcase::generated_content`] が
//!   `None` を返すため、本モジュールも同ページに対しては `None` を返す）。
//! - [`showcase::COMPONENT_PAGES`] レジストリの部品ページ（`/components/<kebab>/`）
//!   のみ、本モジュールの 6 節合成を適用する。
//!
//! # Anatomy の機械導出（受け入れ条件 2）
//!
//! パーツ構成（Anatomy）は headless-ui 側の `Anatomy::part`（`data-scope`/
//! `data-part` 属性）から**機械導出**し、docs 側に手書きのパーツ一覧を
//! 持たない。一次情報は Demo 節（[`showcase::generated_content`] の出力）
//! を走査して得た `data-scope`/`data-part` の出現のみであり、デモが
//! 描画しなかったパーツ（例: accordion デモの `item-indicator`）は
//! 導出結果に含まれない（**デモが実際に描画したパーツの部分集合**である
//! ことを `tests/component_pages.rs` が固定する）。将来 Anatomy の完全
//! 列挙が必要になった場合、headless-ui へパーツ列挙 API を追加する案は
//! 公開クレートのバンプとイシュー #693 方針（headless-ui への直接依存を
//! 持たない）への抵触を伴うため本 PR では見送る。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本モジュールはノード木 API（[`fandhe_frontend_core::el`] /
//! [`fandhe_frontend_core::text`] とそのタグヘルパー）のみで組み立てる。
//! `raw_html()` および HTML 文字列の直接組み立て（`format!("<td>{}</td>", …)`）
//! は使わない。Anatomy コードブロック・`data-*` 属性表・CSS 変数表は
//! いずれも [`text`] 経由のリテラルテキストとして出力し、`render()` が
//! 常にエスケープする（`tests/component_pages.rs` が XSS 回帰として固定）。
//!
//! # 節の省略規則
//!
//! `docs/design/docs-site-component-pages.md` §7 のとおり、各節は内容が
//! 空なら見出しごと省略する（想定外の空 HTML を出さない）。Demo が無い
//! パス（[`showcase::component_page_paths`] 未登録）は本モジュールの対象外
//! として `None` を返す。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use fandhe_frontend_core::{
    code, div, el, h2, h3, li, p, pre, table, tbody, td, text, th, thead, tr, ul, Node,
};

use crate::component_specs;
use crate::showcase;

/// 引数表（`API Reference` 節）1 行。Phase 4（#945〜#948）が原稿データを
/// 供給するまでは [`COMPONENT_SPECS`] に該当エントリが無いため空。
#[derive(Debug, Clone, Copy)]
pub struct ArgRow {
    /// 引数名（例: `variant`）。
    pub name: &'static str,
    /// 型（例: `ButtonVariant`）。
    pub kind: &'static str,
    /// 既定値（無ければ空文字列）。
    pub default: &'static str,
    /// 説明文。
    pub description: &'static str,
}

/// キーボード操作表（`Accessibility` 節）1 行。
#[derive(Debug, Clone, Copy)]
pub struct KeyRow {
    /// キー表記（例: `ArrowDown`）。
    pub key: &'static str,
    /// 動作説明。
    pub description: &'static str,
}

/// WAI-ARIA 対応表（`Accessibility` 節）1 行。
#[derive(Debug, Clone, Copy)]
pub struct AriaRow {
    /// 属性名（例: `aria-expanded`）。
    pub attribute: &'static str,
    /// 動作説明。
    pub description: &'static str,
}

/// `Examples` 節 1 件（見出し + 説明文 + Rust 生成ノード）。
#[derive(Debug, Clone, Copy)]
pub struct ExampleEntry {
    /// H3 見出しテキスト。
    pub title: &'static str,
    /// 説明文。
    pub description: &'static str,
    /// デモ本体を組み立てる関数（`fn` ポインタ、`const` テーブルに埋め込める）。
    pub render: fn() -> Node,
}

/// 部品 1 ページ分の原稿データ（Phase 4 充填対象）。
///
/// Demo（[`showcase`] 由来）・Anatomy（機械導出）・`data-*` 属性表・CSS
/// 変数表（いずれも機械導出）を**除く**、原稿側供給が必要な項目のみを持つ。
/// 未登録パス（[`COMPONENT_SPECS`] に該当エントリなし）は [`ComponentPageSpec::EMPTY`]
/// として扱われ、Features/API 引数表/Examples/Accessibility の 4 節が
/// すべて省略される（Phase 3 の段階でビルドを赤くしないための既定動作）。
#[derive(Debug, Clone, Copy)]
pub struct ComponentPageSpec {
    /// `Features` 節の箇条書き。
    pub features: &'static [&'static str],
    /// `API Reference` 節の引数表（`Arguments`）。
    pub arguments: &'static [ArgRow],
    /// `Examples` 節のエントリ列。
    pub examples: &'static [ExampleEntry],
    /// `Accessibility` 節のキーボード操作表。
    pub keyboard: &'static [KeyRow],
    /// `Accessibility` 節の WAI-ARIA 対応表。
    pub aria: &'static [AriaRow],
    /// Demo フォールバック供給口（イシュー #945）。[`showcase::COMPONENT_PAGES`]
    /// に該当エントリを持たない部品（`showcase.rs` を Phase 4 で編集しない
    /// ための機構）のために、`Demo` 節を組み立てる `fn` ポインタを保持する。
    /// [`showcase::generated_content`] が `None` を返した場合のみ本フィールド
    /// を照会する（[`generated_content`] 参照）。両方 `None`（`showcase` 未登録
    /// かつ本フィールドも `None`）ならページ全体が `None`（従来どおり Markdown
    /// のみのページとして扱う）。
    pub demo: Option<fn() -> Node>,
}

impl ComponentPageSpec {
    /// 全節が空の既定値（未登録パス用）。
    pub const EMPTY: ComponentPageSpec = ComponentPageSpec {
        features: &[],
        arguments: &[],
        examples: &[],
        keyboard: &[],
        aria: &[],
        demo: None,
    };
}

/// `path -> ComponentPageSpec` レジストリを供給するカテゴリ別テーブルの集約。
/// Phase 4（#945〜#948）の各 issue はカテゴリ 1 個につき 1 モジュール
/// （[`component_specs`] 配下）を追加し、本配列へ 1 行追記する想定
/// （[`spec_for`] が全テーブルを線形探索するため、モジュール間の重複パスは
/// 想定しない）。
const SPEC_TABLES: &[&[(&str, ComponentPageSpec)]] = &[component_specs::forms::SPECS];

/// `page_path` に対応する [`ComponentPageSpec`] を返す。未登録パスは
/// [`ComponentPageSpec::EMPTY`]（fail-closed で「節を省略」側へ倒す）。
fn spec_for(page_path: &str) -> ComponentPageSpec {
    SPEC_TABLES
        .iter()
        .flat_map(|table| table.iter())
        .find(|(path, _)| *path == page_path)
        .map(|(_, spec)| *spec)
        .unwrap_or(ComponentPageSpec::EMPTY)
}

/// 部品ノード木の走査で想定外に深いネストへ迷い込んだ場合の安全弁。
/// 本モジュールの入力は Rust コードが静的に組み立てるショーケースの
/// デモ木（外部入力に由来しない）だが、`markdown.rs::MAX_DEPTH` と同じ
/// 考え方でスタックオーバーフローを構造的に避ける（A04 対策）。
const MAX_WALK_DEPTH: usize = 64;

/// `page_path` が Rust 生成コンテンツを持つページなら、Markdown 本文の後ろへ
/// 追記する `Node` 木を返す。
///
/// - [`showcase::COMPONENT_PAGES`] レジストリの部品ページは 6 節（Demo /
///   Features / Anatomy / API Reference / Examples / Accessibility）を
///   合成して返す。
/// - [`showcase::PAGE_PATH`]（索引ページ）を含め、レジストリに未登録の
///   パスは `None`（Markdown のみの通常ページ。索引ページの本文は
///   `site/components-pre-styled-ui.md` 側で完結する、イシュー #943）。
/// - [`showcase::COMPONENT_PAGES`] に無いパスでも、[`ComponentPageSpec::demo`]
///   が `Some` を返せば Demo 節を供給できる（イシュー #945、`showcase.rs` を
///   Phase 4 で編集しないための機構。デモを持たない部品向け）。
#[must_use]
pub fn generated_content(page_path: &str) -> Option<Node> {
    let spec = spec_for(page_path);
    let demo = match showcase::generated_content(page_path) {
        Some(node) => node,
        None => (spec.demo?)(),
    };
    Some(render_component_page(page_path, demo, &spec))
}

/// [`generated_content`] の本体。`demo` は [`showcase::generated_content`]
/// の生出力（部品名 `h2` を含む）、`spec` は原稿データ。テストが合成
/// フィクスチャで全 6 節の順序を固定できるよう `pub` で公開する。
#[must_use]
pub fn render_component_page(page_path: &str, demo: Node, spec: &ComponentPageSpec) -> Node {
    let scope = resolve_anatomy_scope(page_path, &demo);
    let anatomy_parts = scope
        .as_deref()
        .map(|s| collect_anatomy_parts(&demo, s))
        .unwrap_or_default();

    let data_attrs = scope
        .as_deref()
        .map(|s| collect_data_attrs_from_tree(&demo, s))
        .unwrap_or_default();

    let mut sections = Vec::new();
    sections.push(demo_section(demo));
    if let Some(s) = features_section(spec) {
        sections.push(s);
    }
    if let Some(s) = anatomy_section(&anatomy_parts) {
        sections.push(s);
    }
    if let Some(s) = api_reference_section(scope.as_deref(), spec, &data_attrs) {
        sections.push(s);
    }
    if let Some(s) = examples_section(spec) {
        sections.push(s);
    }
    if let Some(s) = accessibility_section(spec) {
        sections.push(s);
    }

    div(vec![("class", "pre-styled-showcase")], sections)
}

/// `Demo` 節: [`showcase`] の生出力から先頭の部品名 `h2`（重複見出し。
/// §3.3 参照）を 1 個だけ除去して `Demo` 見出し配下へ格納する。
fn demo_section(demo: Node) -> Node {
    let stripped = strip_demo_heading(demo);
    el(
        "section",
        vec![],
        vec![h2(vec![], vec![text("Demo")]), stripped],
    )
}

/// `showcase::generated_content` が返す `div.pre-styled-showcase > section >
/// [h2, p, …]` の先頭 `section` 直下の先頭 `h2` だけを取り除く。`h2` が
/// 見つからない場合は無加工で返す（fail-closed に「壊さない」側へ倒す）。
fn strip_demo_heading(demo: Node) -> Node {
    let Node::Element {
        tag,
        attrs,
        children,
    } = demo
    else {
        return demo;
    };
    if tag != "div" {
        return Node::Element {
            tag,
            attrs,
            children,
        };
    }
    let mut children = children;
    if !children.is_empty() {
        let first = children.remove(0);
        children.insert(0, strip_first_h2_in_section(first));
    }
    Node::Element {
        tag,
        attrs,
        children,
    }
}

/// `strip_demo_heading` が委譲する内側の除去処理。`section` 直下の先頭
/// 子要素が `h2` の場合にのみそれを取り除く。
fn strip_first_h2_in_section(node: Node) -> Node {
    let Node::Element {
        tag,
        attrs,
        children,
    } = node
    else {
        return node;
    };
    if tag != "section" {
        return Node::Element {
            tag,
            attrs,
            children,
        };
    }
    let mut children = children;
    if !children.is_empty() {
        let is_h2 = matches!(&children[0], Node::Element { tag, .. } if *tag == "h2");
        if is_h2 {
            children.remove(0);
        }
    }
    let new_children = children;
    Node::Element {
        tag,
        attrs,
        children: new_children,
    }
}

/// `Features` 節。`spec.features` が空なら節ごと省略する。
fn features_section(spec: &ComponentPageSpec) -> Option<Node> {
    if spec.features.is_empty() {
        return None;
    }
    let items = spec
        .features
        .iter()
        .map(|feature| li(vec![], vec![text(*feature)]))
        .collect();
    Some(el(
        "section",
        vec![],
        vec![h2(vec![], vec![text("Features")]), ul(vec![], items)],
    ))
}

/// `Anatomy` 節。導出パーツが 0 件なら節ごと省略する。パーツ名をインデント
/// 表現（入れ子深さ = 半角スペース 2 個/段）で `pre > code` に列挙する
/// （設計 §7 のコードブロック形式。`raw_html` を使わずリテラルテキストで
/// 出力するため `text()` に委ねる）。
fn anatomy_section(parts: &[AnatomyPart]) -> Option<Node> {
    if parts.is_empty() {
        return None;
    }
    let mut body = String::new();
    for (idx, part) in parts.iter().enumerate() {
        if idx > 0 {
            body.push('\n');
        }
        body.push_str(&"  ".repeat(part.depth));
        body.push_str(&part.name);
    }
    Some(el(
        "section",
        vec![],
        vec![
            h2(vec![], vec![text("Anatomy")]),
            pre(vec![], vec![code(vec![], vec![text(body)])]),
        ],
    ))
}

/// `API Reference` 節。引数表（原稿供給）・`data-*` 属性表（機械導出）・
/// CSS 変数表（機械導出）の 3 表すべてが空なら節ごと省略する。
fn api_reference_section(
    scope: Option<&str>,
    spec: &ComponentPageSpec,
    data_attrs: &[DataAttrRow],
) -> Option<Node> {
    let arguments_table = arguments_table(spec.arguments);
    let data_attrs_table = data_attrs_table(data_attrs);
    let css_vars = scope.map(collect_css_vars_for_scope).unwrap_or_default();
    let css_vars_table = css_vars_table(&css_vars);

    if arguments_table.is_none() && data_attrs_table.is_none() && css_vars_table.is_none() {
        return None;
    }

    let mut children = vec![h2(vec![], vec![text("API Reference")])];
    if let Some(t) = arguments_table {
        children.push(h3(vec![], vec![text("Arguments")]));
        children.push(t);
    }
    if let Some(t) = data_attrs_table {
        children.push(h3(vec![], vec![text("Data Attributes")]));
        children.push(t);
    }
    if let Some(t) = css_vars_table {
        children.push(h3(vec![], vec![text("CSS Variables")]));
        children.push(t);
    }
    Some(el("section", vec![], children))
}

/// 引数表（`Arguments`）。空なら `None`。
fn arguments_table(rows: &[ArgRow]) -> Option<Node> {
    if rows.is_empty() {
        return None;
    }
    let header = tr(
        vec![],
        vec![
            th(vec![], vec![text("Name")]),
            th(vec![], vec![text("Type")]),
            th(vec![], vec![text("Default")]),
            th(vec![], vec![text("Description")]),
        ],
    );
    let body_rows = rows
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
        .collect();
    Some(table(
        vec![],
        vec![thead(vec![], vec![header]), tbody(vec![], body_rows)],
    ))
}

/// `data-*` 属性表（`Data Attributes`）。空なら `None`。
fn data_attrs_table(rows: &[DataAttrRow]) -> Option<Node> {
    if rows.is_empty() {
        return None;
    }
    let header = tr(
        vec![],
        vec![
            th(vec![], vec![text("Part")]),
            th(vec![], vec![text("Attribute")]),
            th(vec![], vec![text("Observed Values")]),
        ],
    );
    let body_rows = rows
        .iter()
        .map(|row| {
            tr(
                vec![],
                vec![
                    td(vec![], vec![text(row.part.clone())]),
                    td(vec![], vec![text(row.attr.clone())]),
                    td(vec![], vec![text(row.values.clone())]),
                ],
            )
        })
        .collect();
    Some(table(
        vec![],
        vec![thead(vec![], vec![header]), tbody(vec![], body_rows)],
    ))
}

/// CSS 変数表（`CSS Variables`）。空なら `None`。
fn css_vars_table(rows: &[CssVarRow]) -> Option<Node> {
    if rows.is_empty() {
        return None;
    }
    let header = tr(
        vec![],
        vec![
            th(vec![], vec![text("Variable")]),
            th(vec![], vec![text("Default")]),
        ],
    );
    let body_rows = rows
        .iter()
        .map(|row| {
            tr(
                vec![],
                vec![
                    td(vec![], vec![text(row.name.clone())]),
                    td(vec![], vec![text(row.default.clone())]),
                ],
            )
        })
        .collect();
    Some(table(
        vec![],
        vec![thead(vec![], vec![header]), tbody(vec![], body_rows)],
    ))
}

/// `Examples` 節。空なら `None`。
fn examples_section(spec: &ComponentPageSpec) -> Option<Node> {
    if spec.examples.is_empty() {
        return None;
    }
    let mut children = vec![h2(vec![], vec![text("Examples")])];
    for example in spec.examples {
        children.push(h3(vec![], vec![text(example.title)]));
        children.push(p(vec![], vec![text(example.description)]));
        children.push((example.render)());
    }
    Some(el("section", vec![], children))
}

/// `Accessibility` 節。キーボード操作表・WAI-ARIA 対応表の両方が空なら
/// 節ごと省略する。
fn accessibility_section(spec: &ComponentPageSpec) -> Option<Node> {
    if spec.keyboard.is_empty() && spec.aria.is_empty() {
        return None;
    }
    let mut children = vec![h2(vec![], vec![text("Accessibility")])];
    if !spec.keyboard.is_empty() {
        let header = tr(
            vec![],
            vec![
                th(vec![], vec![text("Key")]),
                th(vec![], vec![text("Description")]),
            ],
        );
        let rows = spec
            .keyboard
            .iter()
            .map(|row| {
                tr(
                    vec![],
                    vec![
                        td(vec![], vec![text(row.key)]),
                        td(vec![], vec![text(row.description)]),
                    ],
                )
            })
            .collect();
        children.push(h3(vec![], vec![text("Keyboard Interactions")]));
        children.push(table(
            vec![],
            vec![thead(vec![], vec![header]), tbody(vec![], rows)],
        ));
    }
    if !spec.aria.is_empty() {
        let header = tr(
            vec![],
            vec![
                th(vec![], vec![text("Attribute")]),
                th(vec![], vec![text("Description")]),
            ],
        );
        let rows = spec
            .aria
            .iter()
            .map(|row| {
                tr(
                    vec![],
                    vec![
                        td(vec![], vec![text(row.attribute)]),
                        td(vec![], vec![text(row.description)]),
                    ],
                )
            })
            .collect();
        children.push(h3(vec![], vec![text("WAI-ARIA")]));
        children.push(table(
            vec![],
            vec![thead(vec![], vec![header]), tbody(vec![], rows)],
        ));
    }
    Some(el("section", vec![], children))
}

// ---------------------------------------------------------------------
// Anatomy 機械導出（ノード木走査）
// ---------------------------------------------------------------------

/// 導出した 1 パーツの出現（部品名 + 入れ子深さ）。深さは対象スコープの
/// anatomy パーツ同士の入れ子のみを数え、途中に挟まる無関係な `div` 等
/// では加算しない（§3.4/§3.5、`collect_anatomy_parts` 参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct AnatomyPart {
    name: String,
    depth: usize,
}

/// `page_path` の末尾セグメント（kebab-case）を対象スコープの第一候補と
/// し、デモ内に出現すればそれを採用する。出現しなければデモ内で最も
/// 外側（最小木深さ・最初に出現）の `data-scope` を採用し、どちらも
/// 無ければ `None`（Anatomy 節は省略、§3.4）。
fn resolve_anatomy_scope(page_path: &str, demo: &Node) -> Option<String> {
    let candidate = page_path.trim_matches('/').rsplit('/').next()?.to_string();
    let mut occurrences = Vec::new();
    collect_data_scope_occurrences(demo, 0, &mut 0usize, &mut occurrences);
    if occurrences.iter().any(|(_, _, scope)| *scope == candidate) {
        return Some(candidate);
    }
    occurrences
        .into_iter()
        .min_by_key(|(depth, order, _)| (*depth, *order))
        .map(|(_, _, scope)| scope)
}

/// `node` 配下の `data-scope` 属性の出現を `(木の深さ, 出現順, 値)` として
/// 収集する。`order` は呼び出し全体で単調増加させ、同深さの複数候補から
/// 「最初に出現したもの」を一意に選べるようにする。
fn collect_data_scope_occurrences(
    node: &Node,
    depth: usize,
    order: &mut usize,
    out: &mut Vec<(usize, usize, String)>,
) {
    if depth >= MAX_WALK_DEPTH {
        return;
    }
    if let Node::Element {
        attrs, children, ..
    } = node
    {
        if let Some((_, scope)) = attrs.iter().find(|(name, _)| name == "data-scope") {
            out.push((depth, *order, scope.clone()));
            *order += 1;
        }
        for child in children {
            collect_data_scope_occurrences(child, depth + 1, order, out);
        }
    }
}

/// `scope` の `data-part` 出現を文書順に収集し、対象スコープの anatomy
/// 入れ子深さを付与する。同名パーツが複数回出現する場合は最初の出現の
/// 深さ・順序を採用し重複除去する（デモが複数アイテムを描画する場合の
/// 冗長表示を避ける、§3.5）。
fn collect_anatomy_parts(demo: &Node, scope: &str) -> Vec<AnatomyPart> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut ordered: Vec<AnatomyPart> = Vec::new();
    walk_anatomy_parts(demo, scope, 0, &mut seen, &mut ordered);
    ordered
}

/// [`collect_anatomy_parts`] の内部再帰実装。`anatomy_depth` は対象
/// スコープのパーツ同士の入れ子のみを数える（無関係な `div`/`span` 等の
/// レイアウト要素は加算しない）。
fn walk_anatomy_parts(
    node: &Node,
    scope: &str,
    anatomy_depth: usize,
    seen: &mut BTreeMap<String, usize>,
    ordered: &mut Vec<AnatomyPart>,
) {
    if anatomy_depth >= MAX_WALK_DEPTH {
        return;
    }
    let Node::Element {
        attrs, children, ..
    } = node
    else {
        return;
    };
    let is_target_scope = attrs
        .iter()
        .any(|(name, value)| name == "data-scope" && value == scope);
    let part_name = attrs
        .iter()
        .find(|(name, _)| name == "data-part")
        .map(|(_, value)| value.clone());

    let next_depth = if is_target_scope {
        if let Some(name) = part_name {
            if !seen.contains_key(&name) {
                seen.insert(name.clone(), ordered.len());
                ordered.push(AnatomyPart {
                    name,
                    depth: anatomy_depth,
                });
            }
        }
        anatomy_depth + 1
    } else {
        anatomy_depth
    };

    for child in children {
        walk_anatomy_parts(child, scope, next_depth, seen, ordered);
    }
}

// ---------------------------------------------------------------------
// data-* 属性表の機械導出
// ---------------------------------------------------------------------

/// `data-*` 属性表の 1 行（パーツ名 × 属性名 × 観測値集合）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct DataAttrRow {
    part: String,
    attr: String,
    /// 観測値をカンマ区切りで連結した文字列（決定性のため昇順ソート済み）。
    values: String,
}

/// `demo` を走査し、`scope` の各パーツの `data-*`（`data-scope`/`data-part`
/// を除く）属性を収集する。
fn collect_data_attrs_from_tree(demo: &Node, scope: &str) -> Vec<DataAttrRow> {
    let mut table: BTreeMap<(String, String), std::collections::BTreeSet<String>> = BTreeMap::new();
    walk_data_attrs(demo, scope, &mut table);
    table
        .into_iter()
        .map(|((part, attr), values)| DataAttrRow {
            part,
            attr,
            values: values.into_iter().collect::<Vec<_>>().join(", "),
        })
        .collect()
}

/// [`collect_data_attrs_from_tree`] の内部再帰実装。
fn walk_data_attrs(
    node: &Node,
    scope: &str,
    table: &mut BTreeMap<(String, String), std::collections::BTreeSet<String>>,
) {
    let Node::Element {
        attrs, children, ..
    } = node
    else {
        return;
    };
    let is_target_scope = attrs
        .iter()
        .any(|(name, value)| name == "data-scope" && value == scope);
    if is_target_scope {
        let part_name = attrs
            .iter()
            .find(|(name, _)| name == "data-part")
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| "root".to_string());
        for (name, value) in attrs {
            if name == "data-scope" || name == "data-part" {
                continue;
            }
            if !name.starts_with("data-") {
                continue;
            }
            table
                .entry((part_name.clone(), name.clone()))
                .or_default()
                .insert(value.clone());
        }
    }
    for child in children {
        walk_data_attrs(child, scope, table);
    }
}

// ---------------------------------------------------------------------
// CSS 変数表の機械導出
// ---------------------------------------------------------------------

/// CSS 変数表の 1 行。
#[derive(Debug, Clone, PartialEq, Eq)]
struct CssVarRow {
    name: String,
    default: String,
}

/// [`showcase::stylesheet`] の CSS 文字列をプロセス内で一度だけ組み立てて
/// キャッシュする（§3.5「ページごとに再構築しない」）。組み立てに失敗した
/// 場合（[`fandhe_frontend_pre_styled_ui::StylesheetError`]）は `None` を
/// キャッシュし、CSS 変数表のみを省略する（ページ生成自体は継続する
/// fail-closed 方針、`build::build_site` 側の成否判定は変えない）。
fn showcase_css_cache() -> &'static Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        showcase::stylesheet()
            .ok()
            .map(|sheet| sheet.as_css().to_string())
    })
}

/// `scope` の `--fandhe-<scope>-*` 変数を CSS 文字列から抽出する。
/// 抽出は正規表現を使わず素の文字列走査で行う（外部クレート追加禁止、
/// REQ-3）。同名変数が複数回出現する場合は最初に見つかった既定値を採用し
/// `BTreeMap` で名前順に整列する（決定性）。
fn collect_css_vars_for_scope(scope: &str) -> Vec<CssVarRow> {
    let Some(css) = showcase_css_cache().as_deref() else {
        return Vec::new();
    };
    let prefix = format!("--fandhe-{scope}-");
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    let mut search_from = 0usize;
    while let Some(rel) = css[search_from..].find("var(") {
        let var_start = search_from + rel + "var(".len();
        let rest = &css[var_start..];
        let name_end = rest.find([',', ')']).unwrap_or(rest.len());
        let name = rest[..name_end].trim();
        if let Some(after_name) = rest.as_bytes().get(name_end) {
            if name.starts_with(&prefix) {
                let default = if *after_name == b',' {
                    extract_balanced_default(&rest[name_end + 1..])
                } else {
                    String::new()
                };
                found.entry(name.to_string()).or_insert(default);
            }
        }
        // 次の探索は現在の変数名末尾より後ろから再開する（無限ループ防止・
        // ネストした `var(...)` の取りこぼしを防ぐため、`)` までは進めない）。
        search_from = var_start + name_end.max(1);
    }
    found
        .into_iter()
        .map(|(name, default)| CssVarRow { name, default })
        .collect()
}

/// `var(<name>, ` の直後（既定値の先頭）から、対応する `)` までを括弧の
/// 対応を数えながら切り出す。ネストした `var(...)` を既定値に含む形式
/// （`var(--fandhe-x, var(--fandhe-y))`）に対応する。
fn extract_balanced_default(rest: &str) -> String {
    let mut depth = 1i32;
    for (i, c) in rest.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return rest[..i].trim().to_string();
                }
            }
            _ => {}
        }
    }
    rest.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn strip_demo_heading_removes_only_leading_h2_under_first_section() {
        let demo = div(
            vec![("class", "pre-styled-showcase")],
            vec![el(
                "section",
                vec![],
                vec![
                    h2(vec![], vec![text("Button")]),
                    p(vec![], vec![text("desc")]),
                    div(vec![], vec![h2(vec![], vec![text("Nested")])]),
                ],
            )],
        );
        let stripped = strip_demo_heading(demo);
        let html = render(&stripped);
        assert!(!html.contains("<h2>Button</h2>"));
        assert!(html.contains("<h2>Nested</h2>"));
    }

    #[test]
    fn strip_demo_heading_is_noop_without_leading_h2() {
        let demo = div(
            vec![("class", "pre-styled-showcase")],
            vec![el(
                "section",
                vec![],
                vec![p(vec![], vec![text("no heading")])],
            )],
        );
        let before = render(&demo.clone());
        let after = render(&strip_demo_heading(demo));
        assert_eq!(before, after);
    }

    #[test]
    fn collect_anatomy_parts_orders_by_first_occurrence_and_depth() {
        let demo = el(
            "div",
            vec![("data-scope", "accordion"), ("data-part", "root")],
            vec![
                el(
                    "div",
                    vec![("data-scope", "accordion"), ("data-part", "item")],
                    vec![
                        el(
                            "h3",
                            vec![("data-scope", "accordion"), ("data-part", "item-trigger")],
                            vec![],
                        ),
                        el(
                            "div",
                            vec![("data-scope", "accordion"), ("data-part", "item-content")],
                            vec![],
                        ),
                    ],
                ),
                // 2 個目の item は既出パーツ名なので重複除去される。
                el(
                    "div",
                    vec![("data-scope", "accordion"), ("data-part", "item")],
                    vec![],
                ),
            ],
        );
        let parts = collect_anatomy_parts(&demo, "accordion");
        assert_eq!(
            parts,
            vec![
                AnatomyPart {
                    name: "root".to_string(),
                    depth: 0
                },
                AnatomyPart {
                    name: "item".to_string(),
                    depth: 1
                },
                AnatomyPart {
                    name: "item-trigger".to_string(),
                    depth: 2
                },
                AnatomyPart {
                    name: "item-content".to_string(),
                    depth: 2
                },
            ]
        );
    }

    #[test]
    fn resolve_anatomy_scope_prefers_path_kebab_match() {
        let demo = el(
            "div",
            vec![],
            vec![el(
                "div",
                vec![("data-scope", "radio-group"), ("data-part", "root")],
                vec![],
            )],
        );
        assert_eq!(
            resolve_anatomy_scope("/components/radio-group/", &demo),
            Some("radio-group".to_string())
        );
    }

    #[test]
    fn resolve_anatomy_scope_falls_back_to_outermost_scope() {
        let demo = el(
            "div",
            vec![],
            vec![el(
                "div",
                vec![("data-scope", "card"), ("data-part", "root")],
                vec![el(
                    "div",
                    vec![("data-scope", "button"), ("data-part", "root")],
                    vec![],
                )],
            )],
        );
        assert_eq!(
            resolve_anatomy_scope("/components/does-not-exist/", &demo),
            Some("card".to_string())
        );
    }

    #[test]
    fn resolve_anatomy_scope_none_without_any_data_scope() {
        let demo = el("div", vec![], vec![p(vec![], vec![text("plain")])]);
        assert_eq!(resolve_anatomy_scope("/components/plain/", &demo), None);
    }

    #[test]
    fn collect_data_attrs_from_tree_collects_observed_values_deterministically() {
        let demo = el(
            "div",
            vec![],
            vec![
                el(
                    "div",
                    vec![
                        ("data-scope", "accordion"),
                        ("data-part", "item-trigger"),
                        ("data-state", "open"),
                    ],
                    vec![],
                ),
                el(
                    "div",
                    vec![
                        ("data-scope", "accordion"),
                        ("data-part", "item-trigger"),
                        ("data-state", "closed"),
                    ],
                    vec![],
                ),
            ],
        );
        let rows = collect_data_attrs_from_tree(&demo, "accordion");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].part, "item-trigger");
        assert_eq!(rows[0].attr, "data-state");
        assert_eq!(rows[0].values, "closed, open");
    }

    #[test]
    fn extract_balanced_default_handles_nested_var() {
        let rest = "var(--fandhe-space-4)) rest";
        assert_eq!(extract_balanced_default(rest), "var(--fandhe-space-4)");
    }
}
