//! `api-reference-param-accordion` block（イシュー #3100。対応表 ID R0197
//! を主参照とする合成例）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `code` / `accordion` の 5 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト仕様（イシュー #3100 本文の要約）
//!
//! 1. 枠の上端に列見出し行（「名前」「型」）を置く
//! 2. その下にパラメータ 1 件ごとの開閉式の行を並べる。開いた行には
//!    説明と既定値を出す
//! 3. 静的表示では先頭行だけを開く
//! 4. 枠・見出し行・各行の角丸は外枠 1 つでまとめる
//!
//! # 静的表示（無 JS、先頭行のみ open・全行 disabled）
//!
//! [`super::super::super::marketing::faq::faq_accordion_centered`] 等が
//! 「全件 open」を選ぶのに対し、本 block は参照元（R0197）の「先頭 1 件
//! のみ open」を採用する（差分は `site/blocks/api-reference-param-
//! accordion.md` の「原案差分メモ」節に記す）。全行を
//! `AccordionProps { disabled: true, .. }` + ネイティブ `disabled` +
//! `aria-disabled="true"` で固定する理由は同ファイルと同じ: 無 JS の docs
//! サイトで `item_trigger` が操作可能なまま閉じた行を残すと、クリック・
//! Enter/Space が no-op になり本文が事実上到達不能になるため
//! （`docs/policy/intentional-non-adoption.md` の UI 部品責務境界）。
//! `disabled_declarations()`（既定 `opacity: 0.5`）は [`LAYOUT_CSS`] で
//! 中和する。
//!
//! # 外枠をまとめる方法
//!
//! 列見出し行（[`columns_header`]）を `accordion::root` の最初の子として
//! 置くことで、recipe 既定の root の border + `border-radius:
//! var(--fandhe-radius-lg)` が見出し行と全行を 1 枠で囲む。`root` は
//! `drop_class_attr` で呼び出し側 `class` を落とすため、Demo 固有の CSS
//! フックは `data-blocks-api-reference-param-accordion-root` 属性で渡す
//! （headless の root は子の種類を制限しない）。
//!
//! # 列の揃え方
//!
//! 列見出し行とトリガーのラベルは同じ `grid-template-columns`
//! （`minmax(0, 3fr) minmax(0, 2fr)`）を共有する。見出し行は root 配下に
//! あるため、recipe が root に定義する
//! `--fandhe-accordion-trigger-padding` をそのまま padding に使え、
//! trigger と左端が揃う（右側にはインジケータ幅ぶんの
//! `padding-inline-end` を足して列位置を合わせる）。
//!
//! # トリガーの子を 2 個に保つ理由
//!
//! styled `accordion::root` の item-trigger recipe は
//! `justify-content: space-between` を前提に「直接の子 2 個」のレイアウト
//! を取るため、[`param_trigger_label`] のラベル `span` と
//! `item_indicator` の 2 個だけに保つ（`faq_accordion_centered` と同じ
//! 判断）。
//!
//! # id 接頭辞と ARIA 対応
//!
//! `id`/`aria-controls`/`aria-labelledby` は
//! `blocks-api-reference-param-accordion-{index}-{trigger|content}` の形で
//! 項目添字から一意に導出する（`blocks_contract.rs` の id 重複・宙に浮いた
//! 参照検査が全 block 横断で検証する）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする。各トリガーは WAI-ARIA APG のアコーディオン
//! パターンに合わせて `<h4>` で包む（`el` で直接組み立て、`heading::heading`
//! は使わない。ページ本文の見出し階層に割り込ませない部品固有の構造要素
//! であるため）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` は `styled_text` として
//! 取り込み、`code::code` はモジュール修飾で呼ぶ（`faq_accordion_centered`
//! と同じ回避方法）。
//!
//! # 文言
//!
//! 架空の API パラメータ（ページネーション・ソート順・必須の
//! プロジェクト ID）を `const PARAMS` 配列で持つ。実在サービスの API 名・
//! 固有名詞は含まない。`crate::blocks::dummy_assets` は使わない
//! （`faq_accordion_centered` と同じく、コードフェンス単体で読める例に
//! 保つため）。
//!
//! # `<form>` を持たない・送信/取得を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 架空の API パラメータ 1 件（名前・型・必須か・説明・既定値）。
struct Param {
    name: &'static str,
    ty: &'static str,
    required: bool,
    description: &'static str,
    default: Option<&'static str>,
}

/// 架空のクエリパラメータ一覧（実在サービスの API 名は含まない）。
const PARAMS: [Param; 4] = [
    Param {
        name: "limit",
        ty: "integer",
        required: false,
        description: "1 ページあたりの最大件数。",
        default: Some("20"),
    },
    Param {
        name: "cursor",
        ty: "string",
        required: false,
        description: "前回応答の next_cursor を渡すとその続きから取得する。",
        default: None,
    },
    Param {
        name: "order",
        ty: "\"asc\" | \"desc\"",
        required: false,
        description: "作成日時順のソート方向。",
        default: Some("\"desc\""),
    },
    Param {
        name: "project_id",
        ty: "string",
        required: true,
        description: "対象プロジェクトの識別子。",
        default: None,
    },
];

/// 枠の上端に置く列見出し行（「名前」「型」）。トリガーのラベル
/// grid（[`param_trigger_label`]）と同じ列幅比を共有する。
fn columns_header() -> Node {
    div(
        vec![("class", "blocks-api-reference-param-accordion-columns")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("class", "blocks-api-reference-param-accordion-col-name")],
                vec![text("名前")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![("class", "blocks-api-reference-param-accordion-col-type")],
                vec![text("型")],
            ),
        ],
    )
}

/// トリガーのラベル（名前 + 必須/任意バッジ、型）。
fn param_trigger_label(param: &Param) -> Node {
    span(
        vec![(
            "class",
            "blocks-api-reference-param-accordion-trigger-label",
        )],
        vec![
            span(
                vec![("class", "blocks-api-reference-param-accordion-name")],
                vec![
                    code::code(&CodeProps::default(), vec![], vec![text(param.name)]),
                    badge::badge(
                        &BadgeProps {
                            variant: if param.required {
                                BadgeVariant::Solid
                            } else {
                                BadgeVariant::Subtle
                            },
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text(if param.required { "必須" } else { "任意" })],
                    ),
                ],
            ),
            span(
                vec![("class", "blocks-api-reference-param-accordion-type")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(param.ty)],
                )],
            ),
        ],
    )
}

/// 説明 + 既定値（`description`/`code` の 2 行）。
fn param_detail(param: &Param) -> Node {
    div(
        vec![],
        vec![
            styled_text::text(&TextProps::default(), vec![], vec![text(param.description)]),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![
                    text("既定値: "),
                    match param.default {
                        Some(default) => {
                            code::code(&CodeProps::default(), vec![], vec![text(default)])
                        }
                        None => text("既定値なし"),
                    },
                ],
            ),
        ],
    )
}

/// パラメータ 1 件分の accordion item。先頭（`index == 0`）のみ
/// [`OpenState::Open`]、他は [`OpenState::Closed`]。全行 `disabled: true`
/// 固定（モジュール doc「静的表示」節）。
fn param_item(index: usize, param: &Param) -> Node {
    let state = if index == 0 {
        OpenState::Open
    } else {
        OpenState::Closed
    };
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-api-reference-param-accordion-{index}-trigger");
    let content_id = format!("blocks-api-reference-param-accordion-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![(
                    "class",
                    "blocks-api-reference-param-accordion-trigger-heading",
                )],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    param.name,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        param_trigger_label(param),
                        item_indicator(state, false, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            item_content(
                state,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![param_detail(param)],
            ),
        ],
    )
}

/// 列見出し行 + パラメータ行を内包する外枠（`accordion::root` 1 つに
/// まとめる、モジュール doc「外枠をまとめる方法」節）。
fn params_root() -> Node {
    let mut children: Vec<Node> = vec![columns_header()];
    children.extend(
        PARAMS
            .iter()
            .enumerate()
            .map(|(index, param)| param_item(index, param)),
    );

    accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-api-reference-param-accordion-root", "")],
        children,
    )
}

/// `api-reference-param-accordion` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-api-reference-param-accordion-layout")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("リクエストパラメータ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("行をクリックすると説明と既定値が開きます。")],
            ),
            params_root(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/api-reference-param-accordion/",
    title: "api-reference-param-accordion",
    category: BlockCategory::ApiReference,
    rust_source: "crates/docs-site/src/blocks/docs/api_reference/api_reference_param_accordion.rs",
    demo_class: "blocks-api-reference-param-accordion",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Accordion",
            path: "/themes/accordion/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `api_reference_param_accordion` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。
///
/// セレクタは `.blocks-api-reference-param-accordion-*` と
/// `[data-blocks-api-reference-param-accordion-*]`、および styled
/// accordion の `[data-scope="accordion"]` 系セレクタへの子孫結合子付き
/// 上書き（disabled 中和・列揃え、モジュール doc「静的表示」「列の揃え方」
/// 節）のみを用いる。狭い幅では `minmax(0, …)` の grid と
/// `overflow-wrap: anywhere`（code）で横はみ出しを防ぎ、`@media` は使わない。
const LAYOUT_CSS: &str = "\
.blocks-api-reference-param-accordion-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-api-reference-param-accordion-root] {\n  overflow: hidden;\n}\n\
.blocks-api-reference-param-accordion-columns {\n  display: grid;\n  grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-accordion-trigger-padding, var(--fandhe-space-3) var(--fandhe-space-4));\n  padding-inline-end: calc(var(--fandhe-space-6) + var(--fandhe-space-4));\n  background: var(--fandhe-color-bg-subtle);\n  border-block-end: 1px solid var(--fandhe-color-border-muted);\n}\n\
.blocks-api-reference-param-accordion-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
.blocks-api-reference-param-accordion-trigger-label {\n  display: grid;\n  grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);\n  gap: var(--fandhe-space-4);\n  align-items: center;\n  inline-size: 100%;\n  text-align: start;\n}\n\
.blocks-api-reference-param-accordion-name {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  min-inline-size: 0;\n}\n\
.blocks-api-reference-param-accordion-type {\n  min-inline-size: 0;\n}\n\
.blocks-api-reference-param-accordion-name code,\n.blocks-api-reference-param-accordion-type code {\n  overflow-wrap: anywhere;\n}\n\
[data-blocks-api-reference-param-accordion-root] [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, PARAMS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 5 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"code\"",
            "data-scope=\"accordion\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 先頭行のみ open・残り全行 closed・全行 disabled であることを固定
    /// する（モジュール doc「静的表示」節）。
    #[test]
    fn demo_renders_only_first_row_open_and_all_disabled() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            1,
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"data-part="item" data-state="closed""#)
                .count(),
            PARAMS.len() - 1,
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-disabled="true""#).count(),
            PARAMS.len(),
            "html={html}"
        );
    }

    /// [`LAYOUT_CSS`] が disabled トリガーの既定 `opacity: 0.5` を中和し、
    /// 列見出し行のグリッド列幅をトリガーラベルと共有すること（モジュール
    /// doc「列の揃え方」節）。
    #[test]
    fn layout_css_neutralizes_disabled_and_shares_grid_columns() {
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        let columns_rule = "grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);";
        assert_eq!(
            LAYOUT_CSS.matches(columns_rule).count(),
            2,
            "columns_header と trigger_label の双方に同じ列幅比が必要: {LAYOUT_CSS}"
        );
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`faq_accordion_centered` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-api-reference-param-accordion-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-api-reference-param-accordion-layout"
        );
    }
}
