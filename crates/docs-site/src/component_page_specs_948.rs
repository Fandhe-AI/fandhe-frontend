//! イシュー #948（親 #928 Phase 4、トラッキング #924）が供給する部品ページ
//! 原稿データ。Typography / Utilities / Charts 系と、本文が明示列挙した他
//! カテゴリ 11 件を合わせた 28 ページ分の [`ComponentPageSpec`] を持つ
//! （原稿分割はイシュー番号単位であり、`docs/design/docs-site-component-pages.md`
//! §5 の IA カテゴリ単位ではない。33 ページの厳密な内訳・分割算術は
//! 実装 PR 本文を参照）。
//!
//! # 対象 28 ページ（`ComponentPageSpec` を登録するモード A）
//!
//! - Typography: `blockquote` `code` `em` `heading` `highlight` `kbd` `list`
//!   `mark` `text`
//! - Utilities: `visually-hidden`
//! - Charts: `charts`（共通 API） `area-chart` `bar-chart` `bar-list`
//!   `bar-segment` `donut-chart` `line-chart` `pie-chart` `radar-chart`
//!   `scatter-chart` `sparkline`
//! - 他カテゴリ（Interactive/Forms/Data Display から本文明示分）:
//!   `download-trigger` `qr-code` `timer` `color-picker` `calendar`
//!   `date-picker` `date-input`
//!
//! `angle-slider` / `clipboard` / `image-cropper` / `signature-pad` /
//! `skip-nav` の 5 ページ（モード B）は
//! [`crate::showcase::COMPONENT_PAGES`] に未登録で
//! [`crate::showcase::generated_content`] が `None` を返すため、**本モジュール
//! へは** spec を登録してはならない（登録してもデッドコードになる）。ただし
//! これは「原稿執筆を諦める」の意味ではない: #979 が導入した
//! [`crate::component_page::ComponentPageSpec::demo`]（Demo フォールバック
//! 供給口）経由であれば `ComponentPageSpec` を登録しても Demo 節を含め
//! 到達可能である。実際に angle-slider / image-cropper / pin-input /
//! signature-pad の 4 件は [`crate::component_specs::forms`] が、
//! clipboard / skip-nav の 2 件は [`crate::component_specs::interactive_utilities`]
//! （イシュー #1155）がこの機構でそれぞれ充填済みであり、この 6 ページ
//! とも `site/themes/*.md` は H1 + 導入文のみを保つ現状のままである（本
//! モジュールへの登録のみが禁止で、Demo フォールバック経由の登録は別
//! モジュールで行われている）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! 本モジュールが供給するのは `&'static str` 定数（[`ComponentPageSpec`]
//! の `features`/`arguments`/`keyboard`/`aria`）と `fn() -> Node`
//! （`examples`）のみである。`raw_html()` および HTML 文字列の直接組み立て
//! （`format!("<td>{}</td>", …)`）は一切使わない。`examples` が組み立てる
//! `Node` 木も `fandhe_frontend_core`/`fandhe_frontend_pre_styled_ui` の
//! ノード木 API のみを経由し、`render()` の既定エスケープを経る
//! （`crates/docs-site/tests/component_pages.rs` の
//! `features_and_table_cells_escape_xss_payloads` が回帰を固定する）。
//!
//! # レイアウト class の契約
//!
//! Examples 内で使ってよい非 `docs-` ラッパ class は
//! `showcase::stylesheet()` の `SHOWCASE_LAYOUT_CSS` に実在する
//! `showcase-row`/`showcase-stack` のみ（`crates/docs-site/tests/site_css_contract.rs`
//! の `component_page_render_introduces_no_class_outside_the_contract` が
//! 固定する）。本モジュールの [`row`]/[`stack`] ヘルパはこの 2 class のみを
//! 出力する（`showcase.rs` 内の同名 private ヘルパと同型、`pub(crate)` では
//! なく本モジュール内 `fn` として複製し、`showcase.rs` の変更範囲をゼロに
//! 保つ）。

use fandhe_frontend_core::{div, el, text, Node};

use fandhe_frontend_pre_styled_ui::area_chart::{self, AreaChartProps};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::calendar::{self, PlainDate};
use fandhe_frontend_pre_styled_ui::charts::bar_chart::{
    self, BarChartProps, Orientation as BarChartOrientation,
};
use fandhe_frontend_pre_styled_ui::charts::bar_list;
use fandhe_frontend_pre_styled_ui::charts::bar_segment;
use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
use fandhe_frontend_pre_styled_ui::charts::radar_chart::{self, RadarChartProps};
use fandhe_frontend_pre_styled_ui::charts::scatter_chart::{
    self, ScatterChartProps, ScatterData, ScatterSeries,
};
use fandhe_frontend_pre_styled_ui::code::{code, CodeProps, CodeVariant};
use fandhe_frontend_pre_styled_ui::color_picker;
use fandhe_frontend_pre_styled_ui::color_swatch::{Color, Rgb};
use fandhe_frontend_pre_styled_ui::date_input::{self, DateSegment};
use fandhe_frontend_pre_styled_ui::date_picker;
use fandhe_frontend_pre_styled_ui::donut_chart::{donut_chart, DonutChartProps};
use fandhe_frontend_pre_styled_ui::download_trigger::{self, DownloadTriggerProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::color_picker::{
    Channel, ColorPicker,
};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::date_input::DateInput;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::dispatch;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::highlight::{highlight, HighlightProps, HighlightVariant};
use fandhe_frontend_pre_styled_ui::kbd::{kbd, KbdProps, KbdVariant};
use fandhe_frontend_pre_styled_ui::line_chart::{self, LineChartProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::mark::{mark, MarkProps, MarkVariant};
use fandhe_frontend_pre_styled_ui::pie_chart::{pie_chart, PieChartProps};
use fandhe_frontend_pre_styled_ui::qr_code;
use fandhe_frontend_pre_styled_ui::sparkline::{self, SparklineProps};
use fandhe_frontend_pre_styled_ui::text::{text as styled_text, TextProps, TextSize, TextWeight};
use fandhe_frontend_pre_styled_ui::timer::{self, Timer, TimerControl, TimerUnit};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, OpenState, Size};

use crate::component_page::{ArgRow, AriaRow, ComponentPageSpec, ExampleEntry, KeyRow};

/// 横並びのデモ列（`showcase.rs::row` と同型、[`SHOWCASE_LAYOUT_CSS`] の
/// `.showcase-row` を使う）。
fn row(children: Vec<Node>) -> Node {
    div(vec![("class", "showcase-row")], children)
}

/// 縦積みのデモ列（`showcase.rs::stack` と同型）。
fn stack(children: Vec<Node>) -> Node {
    div(vec![("class", "showcase-stack")], children)
}

// ---------------------------------------------------------------------
// Typography（Heading / Text / Em / Mark / Blockquote / List / Highlight /
// Kbd / Code）
// ---------------------------------------------------------------------

fn heading_example() -> Node {
    row(vec![
        heading(
            HeadingLevel::H2,
            &HeadingProps {
                size: HeadingSize::Xl2,
            },
            vec![],
            vec![text("見出し (h2, size=xl2)")],
        ),
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl,
            },
            vec![],
            vec![text("見出し (h3, size=xl)")],
        ),
    ])
}

const HEADING_ARGUMENTS: &[ArgRow] = &[
    ArgRow {
        name: "level",
        kind: "HeadingLevel",
        default: "（必須）",
        description: "レンダリングする HTML タグ（h1〜h6）。意味論レベルを表す。",
    },
    ArgRow {
        name: "props.size",
        kind: "HeadingSize",
        default: "Xl",
        description: "視覚サイズ（xs/sm/md/lg/xl/xl2/xl3/xl4）。タグ選択（意味論）とは独立した軸。",
    },
];

const HEADING_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "素の h1〜h6 意味論をタグとしてそのまま維持しつつ、視覚サイズを variant として独立に切り替える",
        "colorPalette 軸を持たない単一 recipe 静的部品",
        "chakra-ui の 9 段階サイズをテーマトークン範囲（xs〜4xl の 8 段階）へ縮約済み",
    ],
    arguments: HEADING_ARGUMENTS,
    examples: &[ExampleEntry {
        title: "タグとサイズの独立軸",
        description: "レンダリングするタグ（h1〜h6）と視覚サイズ（xs〜4xl）を独立に選べます。",
        render: heading_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn text_example() -> Node {
    stack(vec![
        styled_text(
            &TextProps {
                size: TextSize::Sm,
                ..TextProps::default()
            },
            vec![],
            vec![text("本文テキスト（size=sm）")],
        ),
        styled_text(
            &TextProps {
                size: TextSize::Xl,
                ..TextProps::default()
            },
            vec![],
            vec![text("本文テキスト（size=xl）")],
        ),
        styled_text(
            &TextProps {
                weight: TextWeight::Bold,
                ..TextProps::default()
            },
            vec![],
            vec![text("本文テキスト（weight=bold）")],
        ),
    ])
}

const TEXT_ARGUMENTS: &[ArgRow] = &[
    ArgRow {
        name: "size",
        kind: "TextSize",
        default: "Md",
        description: "フォントサイズ・行間の視覚サイズ軸（xs/sm/md/lg/xl/xl2/xl3/xl4）。",
    },
    ArgRow {
        name: "weight",
        kind: "TextWeight",
        default: "Normal",
        description:
            "フォントウェイトの視覚軸（normal/medium/semibold/bold）。イシュー #1442 で追加。",
    },
];

const TEXT_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "素の <p> 要素を size（xs〜xl4 の 8 段階）・weight（normal/medium/semibold/bold）でスタイル化した本文テキスト部品",
        "variant・colorPalette 軸は持たない最小構成",
    ],
    arguments: TEXT_ARGUMENTS,
    examples: &[ExampleEntry {
        title: "size・weight 軸",
        description: "size（xs〜xl4）・weight（normal〜bold）を独立に選べます。",
        render: text_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn em_example() -> Node {
    row(vec![el(
        "p",
        vec![],
        vec![
            text("この文の"),
            fandhe_frontend_pre_styled_ui::em::em(vec![], vec![text("強調部分")]),
            text("は重要です。"),
        ],
    )])
}

const EM_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "素の <em> 要素をそのまま styled 化した強調テキスト部品",
        "variant 軸を持たない最小部品（link_overlay と同型）",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "文中の強調",
        description: "地の文の一部を <em> で強調します。",
        render: em_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn mark_example() -> Node {
    row(vec![
        mark(&MarkProps::default(), vec![], vec![text("subtle")]),
        mark(
            &MarkProps {
                variant: MarkVariant::Solid,
                ..MarkProps::default()
            },
            vec![],
            vec![text("solid")],
        ),
        mark(
            &MarkProps {
                variant: MarkVariant::Text,
                ..MarkProps::default()
            },
            vec![],
            vec![text("text")],
        ),
        mark(
            &MarkProps {
                variant: MarkVariant::Plain,
                ..MarkProps::default()
            },
            vec![],
            vec![text("plain")],
        ),
    ])
}

const MARK_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "テキストの一部を強調する単一 slot 静的部品",
        "variant（subtle/solid/text/plain）4 種、colorPalette 6 値を持つ（badge と同型の単一 recipe パターン）",
        "subtle は palette 連動の淡色背景 + 本文継承の文字色、text は透明背景 + 本文継承の文字色 + medium ウェイト（イシュー #1439 で chakra-ui 基準へ是正）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "MarkVariant",
            default: "Subtle",
            description: "見た目のバリアント（subtle/solid/text/plain）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Neutral",
            description: "colorPalette 軸。",
        },
    ],
    examples: &[ExampleEntry {
        title: "variant 4 種",
        description: "subtle/solid/text/plain の見た目を並べます。",
        render: mark_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn blockquote_example() -> Node {
    row(vec![blockquote::root(
        BlockquoteVariant::Subtle,
        ColorPalette::Accent,
        vec![],
        vec![
            blockquote::content(
                vec![],
                vec![text("プレーンな HTML / JavaScript / CSS を尊重する。")],
            ),
            blockquote::caption(vec![], vec![text("— fandhe-frontend CLAUDE.md")]),
        ],
    )])
}

const BLOCKQUOTE_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root（figure）/content（素の blockquote）/caption（figcaption）の 3 パーツで構成する",
        "content が素の <blockquote> タグのため引用の HTML 意味論を保つ",
        "variant（subtle/solid/plain）と colorPalette（6 値、root のみ）を持つ",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "BlockquoteVariant",
            default: "Subtle",
            description: "見た目のバリアント（subtle: 背景なし・muted 罫線のみ〔既定〕 / solid: 塗りつぶし / plain: 背景なし・強い accent 罫線）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸（root のみに適用）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "content + caption",
        description: "引用本文（content）と出典（caption）を組み合わせます。",
        render: blockquote_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn list_example() -> Node {
    stack(vec![
        list::root(
            ListType::Unordered,
            ListVariant::Marker,
            vec![],
            vec![
                list::item(vec![], vec![text("SSR")]),
                list::item(vec![], vec![text("SPA")]),
                list::item(vec![], vec![text("SSG")]),
            ],
        ),
        list::root(
            ListType::Ordered,
            ListVariant::Marker,
            vec![],
            vec![
                list::item(vec![], vec![text("計画")]),
                list::item(vec![], vec![text("実装")]),
                list::item(vec![], vec![text("検証")]),
            ],
        ),
    ])
}

/// `ListVariant::Plain` + [`list::indicator`] の実例（イシュー #1438、
/// indicator の間隔・整列と Plain variant の item 整列を Demo 上で視覚
/// 確認できるようにする）。
fn list_plain_indicator_example() -> Node {
    list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![],
        vec![
            list::item(
                vec![],
                vec![
                    list::indicator(vec![], vec![text("✓")]),
                    text("既定エスケープ"),
                ],
            ),
            list::item(
                vec![],
                vec![
                    list::indicator(vec![], vec![text("✓")]),
                    text("forbid(unsafe_code)"),
                ],
            ),
        ],
    )
}

const LIST_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "root（ul/ol）/item（li）/indicator（span aria-hidden）の 3 パーツで構成する",
        "ListType（Unordered/Ordered）でレンダリングするタグそのものを選ぶ",
        "ListVariant（marker/plain）でマーカー表示の有無を切り替える",
        "indicator は常時 aria-hidden=\"true\"（呼び出し側が外せない fail-closed）",
        "marker（箇条書きの点・番号）は本文より淡いグレー（fg.muted）で描く",
        "indicator はテキストとの間隔・行頭揃えを持ち、plain variant の item は複数行でも行頭が揃う",
    ],
    arguments: &[
        ArgRow {
            name: "list_type",
            kind: "ListType",
            default: "Unordered",
            description: "レンダリングするタグ（ul/ol）を選ぶ引数。",
        },
        ArgRow {
            name: "variant",
            kind: "ListVariant",
            default: "Marker",
            description: "マーカー（箇条書き記号・番号）表示の有無。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "順序なし・順序ありの 2 種",
            description: "ListType で ul/ol を切り替えます。",
            render: list_example,
        },
        ExampleEntry {
            title: "plain + indicator（カスタムマーカー）",
            description: "ListVariant::Plain と indicator を組み合わせ、アイコン等のカスタムマーカーを行頭に揃えて表示します。",
            render: list_plain_indicator_example,
        },
    ],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn highlight_example() -> Node {
    stack(vec![
        highlight(
            &HighlightProps {
                query: &["brown fox"],
                ..HighlightProps::default()
            },
            vec![],
            "The quick brown fox jumps over the lazy dog",
        ),
        highlight(
            &HighlightProps {
                query: &["o"],
                match_all: true,
                ..HighlightProps::default()
            },
            vec![],
            "The quick brown fox jumps over the lazy dog",
        ),
        highlight(
            &HighlightProps {
                query: &["LAZY"],
                ignore_case: true,
                ..HighlightProps::default()
            },
            vec![],
            "The quick brown fox jumps over the lazy dog",
        ),
    ])
}

fn highlight_variant_example() -> Node {
    row(vec![
        highlight(
            &HighlightProps {
                query: &["subtle"],
                variant: HighlightVariant::Subtle,
                ..HighlightProps::default()
            },
            vec![],
            "subtle",
        ),
        highlight(
            &HighlightProps {
                query: &["solid"],
                variant: HighlightVariant::Solid,
                ..HighlightProps::default()
            },
            vec![],
            "solid",
        ),
        highlight(
            &HighlightProps {
                query: &["text"],
                variant: HighlightVariant::Text,
                ..HighlightProps::default()
            },
            vec![],
            "text",
        ),
        highlight(
            &HighlightProps {
                query: &["plain"],
                variant: HighlightVariant::Plain,
                ..HighlightProps::default()
            },
            vec![],
            "plain",
        ),
    ])
}

const HIGHLIGHT_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "テキスト中の一致語句を <mark> で囲んで強調する",
        "正規表現ではなく決定的な部分文字列検索のみで一致判定する（ReDoS の面を持たない）",
        "query（複数可）・match_all（全一致 or 最初の 1 件）・ignore_case（ASCII 限定）の 3 プロパティを持つ",
        "variant（subtle/solid/text/plain）4 種、colorPalette 6 値を持つ（mark と同一語彙、イシュー #1435）",
    ],
    arguments: &[
        ArgRow {
            name: "query",
            kind: "&[&str]",
            default: "&[]",
            description: "強調する語句（複数可）。空文字列の要素は無視する。",
        },
        ArgRow {
            name: "ignore_case",
            kind: "bool",
            default: "false",
            description: "大文字小文字を区別しない一致（ASCII の範囲のみ）。",
        },
        ArgRow {
            name: "match_all",
            kind: "bool",
            default: "false",
            description: "true なら全一致箇所、false なら最初の 1 箇所のみ強調する。",
        },
        ArgRow {
            name: "variant",
            kind: "HighlightVariant",
            default: "Subtle",
            description: "見た目のバリアント（subtle/solid/text/plain）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Accent",
            description: "colorPalette 軸。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "単一一致・全一致・大文字小文字無視",
            description: "query / match_all / ignore_case の組み合わせを並べます。",
            render: highlight_example,
        },
        ExampleEntry {
            title: "variant 4 種",
            description: "subtle/solid/text/plain の見た目を並べます。",
            render: highlight_variant_example,
        },
    ],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn kbd_example() -> Node {
    row(vec![
        kbd(&KbdProps::default(), vec![], vec![text("Ctrl")]),
        text(" + "),
        kbd(&KbdProps::default(), vec![], vec![text("K")]),
        kbd(
            &KbdProps {
                variant: KbdVariant::Subtle,
                ..KbdProps::default()
            },
            vec![],
            vec![text("subtle")],
        ),
        kbd(
            &KbdProps {
                variant: KbdVariant::Outline,
                ..KbdProps::default()
            },
            vec![],
            vec![text("outline")],
        ),
    ])
}

const KBD_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "キーボード入力・ショートカット表示のための単一 recipe styled 部品（<kbd>）",
        "variant（raised/subtle/outline）3 種、size 5 段、colorPalette 6 値を持つ（イシュー #1436、code と同型の単一 recipe パターン）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "KbdVariant",
            default: "Raised",
            description: "見た目のバリアント（raised/subtle/outline）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ軸（xs/sm/md/lg/xl）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Neutral",
            description: "colorPalette 軸。",
        },
    ],
    examples: &[ExampleEntry {
        title: "キーの組み合わせ・variant 3 種",
        description: "複数の kbd をテキストで連結してショートカットを表現し、raised/subtle/outline の見た目を並べます。",
        render: kbd_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn code_example() -> Node {
    row(vec![
        code(&CodeProps::default(), vec![], vec![text("subtle")]),
        code(
            &CodeProps {
                variant: CodeVariant::Solid,
                ..CodeProps::default()
            },
            vec![],
            vec![text("solid")],
        ),
        code(
            &CodeProps {
                variant: CodeVariant::Outline,
                ..CodeProps::default()
            },
            vec![],
            vec![text("outline")],
        ),
    ])
}

const CODE_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "インラインコード片表示のための単一 recipe styled 部品（<code>）",
        "chakra-ui の CodeBlock（複数行・シンタックスハイライト）相当は対象外確定済み",
        "variant（solid/subtle/outline）3 種、size 5 段、colorPalette 6 値を持つ（イシュー #1432、mark と同型の単一 recipe パターン）",
    ],
    arguments: &[
        ArgRow {
            name: "variant",
            kind: "CodeVariant",
            default: "Subtle",
            description: "見た目のバリアント（solid/subtle/outline）。",
        },
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "サイズ軸（xs/sm/md/lg/xl）。",
        },
        ArgRow {
            name: "palette",
            kind: "ColorPalette",
            default: "Neutral",
            description: "colorPalette 軸。",
        },
    ],
    examples: &[ExampleEntry {
        title: "variant 3 種",
        description: "solid/subtle/outline の見た目を並べます。",
        render: code_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

// ---------------------------------------------------------------------
// Utilities（VisuallyHidden）
// ---------------------------------------------------------------------

fn visually_hidden_example() -> Node {
    use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
    row(vec![button(
        &ButtonProps::default(),
        vec![],
        vec![
            el("span", vec![("aria-hidden", "true")], vec![text("★")]),
            visually_hidden::root(vec![], vec![text("お気に入りに追加")]),
        ],
    )])
}

const VISUALLY_HIDDEN_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "視覚的には隠す（clip 手法）が支援技術には読ませ続けるテキストコンテナ",
        "aria-hidden は一切出力しない（子孫テキストをアクセシブルネームとして読ませる用途）",
        "アイコンのみのボタンへ補足テキストを添える等の用途に使う",
    ],
    arguments: &[],
    examples: &[ExampleEntry {
        title: "アイコンのみのボタンへの補足テキスト",
        description: "装飾アイコン（aria-hidden）+ VisuallyHidden のテキストでアクセシブルネームを与えます。aria-label を併用すると accessible-name 計算で aria-label が勝ち VisuallyHidden テキストが読み上げられなくなるため併用しません。",
        render: visually_hidden_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

// ---------------------------------------------------------------------
// Charts（共通 API）
// ---------------------------------------------------------------------

fn charts_overview_example() -> Node {
    use fandhe_frontend_pre_styled_ui::charts::axis::{self, AxisProps};
    use fandhe_frontend_pre_styled_ui::charts::grid::{self, GridProps};
    use fandhe_frontend_pre_styled_ui::charts::legend::{self, LegendProps};
    use fandhe_frontend_pre_styled_ui::charts::scale::LinearScale;
    use fandhe_frontend_pre_styled_ui::charts::svg::{svg_root, ViewBox};
    use fandhe_frontend_pre_styled_ui::charts::tooltip;

    let data = ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
        vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
    )
    .expect("固定サンプルは常に有効");
    let (plot_left, plot_right) = (30.0, 190.0);
    let (plot_top, plot_bottom) = (10.0, 110.0);
    let (min, max) = data.domain();
    let y_scale = LinearScale::new((min, max), (plot_bottom, plot_top))
        .expect("domain() は非退化な値域を返す");
    let y_ticks = y_scale.ticks(3).expect("target=3 は許容範囲内");
    let y_tick_positions: Vec<f64> = y_ticks.iter().map(|&t| y_scale.scale(t)).collect();
    let mut children = vec![
        grid::cartesian_grid(
            (plot_left, plot_right),
            (plot_top, plot_bottom),
            &[],
            &y_tick_positions,
            &GridProps::default(),
        )
        .expect("有限な range/ticks のみを渡す"),
        axis::y_axis(&y_scale, &y_ticks, plot_left, &AxisProps::default())
            .expect("有限な ticks のみを渡す"),
        axis::x_axis_categories(
            (plot_left, plot_right),
            data.categories(),
            plot_bottom,
            &AxisProps::default(),
        )
        .expect("categories は非空・range は有限"),
    ];
    let band = (plot_right - plot_left) / data.categories().len() as f64;
    for (i, &v) in data.series()[0].values.iter().enumerate() {
        let cx = plot_left + (i as f64 + 0.5) * band;
        let cy = y_scale.scale(v);
        let label = tooltip::datum_label(&data.categories()[i], &data.series()[0].name, v);
        children.push(tooltip::datum(cx, cy, 3.0, &label, vec![]));
    }
    let view_box = ViewBox::new(0.0, 0.0, 200.0, 120.0).expect("固定寸法は正の有限値");
    let chart = svg_root(&view_box, vec![("aria-label", "monthly visits")], children);
    let legend_node = legend::legend(
        &data,
        &LegendProps {
            title: Some("Series".to_string()),
        },
    );
    row(vec![chart, legend_node])
}

const CHARTS_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "外部依存ゼロの SVG ノード木生成ヘルパー群（軸・グリッド・凡例・ツールチップ）",
        "charts::axis / charts::grid / charts::legend / charts::tooltip の 4 サブモジュールで構成する",
        "マウス追従型のツールチップは持たず、ブラウザネイティブの <title> + aria-label によるホバー詳細表示のみ（JS 不要）",
        "系列の各データ点は charts::series_color_var(index) の固定色循環で着色する",
    ],
    arguments: &[
        ArgRow {
            name: "axis::y_axis / x_axis_categories",
            kind: "fn(...) -> Result<Node, ChartError>",
            default: "",
            description: "Y 軸・カテゴリ X 軸を組み立てる。",
        },
        ArgRow {
            name: "grid::cartesian_grid",
            kind: "fn(...) -> Result<Node, ChartError>",
            default: "",
            description: "格子線（CartesianGrid）を組み立てる。",
        },
        ArgRow {
            name: "legend::legend",
            kind: "fn(&ChartData, &LegendProps) -> Node",
            default: "",
            description: "系列名の凡例を組み立てる（infallible）。",
        },
        ArgRow {
            name: "tooltip::datum",
            kind: "fn(...) -> Node",
            default: "",
            description: "データ点（<circle>）+ <title> によるホバー詳細を組み立てる（infallible）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "軸 + グリッド + ツールチップ + 凡例の合成",
        description: "本ページ配下の各チャート部品が共通で使う基盤 API の最小合成例です。系列を結ぶ折れ線・棒等は LineChart/BarChart 等の個別部品ページを参照してください。",
        render: charts_overview_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn bar_charts_sample_data() -> ChartData {
    ChartData::new(
        vec![
            "Jan".to_string(),
            "Feb".to_string(),
            "Mar".to_string(),
            "Apr".to_string(),
        ],
        vec![
            Series::new("visits", vec![120.0, 200.0, 150.0, 80.0]),
            Series::new("signups", vec![20.0, 35.0, 28.0, 12.0]),
        ],
    )
    .expect("固定サンプルはカテゴリ数・系列長が一致する")
}

fn bar_chart_example() -> Node {
    let data = bar_charts_sample_data();
    let vertical = bar_chart::root(
        &data,
        BarChartProps::default(),
        "monthly visits and signups",
    )
    .expect("固定サンプルは domain・viewBox とも常に有効");
    let horizontal = bar_chart::root(
        &data,
        BarChartProps {
            orientation: BarChartOrientation::Horizontal,
            ..BarChartProps::default()
        },
        "monthly visits and signups (horizontal)",
    )
    .expect("固定サンプルは domain・viewBox とも常に有効");
    stack(vec![row(vec![vertical]), row(vec![horizontal])])
}

const BAR_CHART_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ChartData（複数系列）+ LinearScale + SVG ノード木生成ヘルパーのみで組み立てるグループ棒グラフ",
        "orientation（vertical/horizontal）で棒の向きを切り替える",
        "軸線・グリッド・凡例・ツールチップは charts（共通 API）側の別部品",
    ],
    arguments: &[
        ArgRow {
            name: "orientation",
            kind: "Orientation",
            default: "Vertical",
            description: "カテゴリ軸の向き（vertical: 棒が縦に伸びる / horizontal: 棒が横に伸びる）。",
        },
        ArgRow { name: "width", kind: "f64", default: "480.0", description: "viewBox の幅（px 相当）。" },
        ArgRow { name: "height", kind: "f64", default: "300.0", description: "viewBox の高さ（px 相当）。" },
    ],
    examples: &[ExampleEntry {
        title: "縦/横 orientation",
        description: "同じデータを vertical/horizontal で描画します。",
        render: bar_chart_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn bar_list_example() -> Node {
    let data = bar_charts_sample_data();
    row(vec![
        bar_list::root(&data, "visits").expect("固定サンプルの visits 系列は常に存在する")
    ])
}

const BAR_LIST_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ランキング型バーリスト。単一系列を対象に、系列内最大値に対する比率でバー幅を決める",
        "カテゴリ順（挿入順）にそのまま描画する（降順表示は呼び出し側で ChartData::sort_by_series を事前適用）",
    ],
    arguments: &[ArgRow {
        name: "series_name",
        kind: "&str",
        default: "（必須）",
        description: "描画対象の系列名。ChartData に存在しない場合は ChartError を返す。",
    }],
    examples: &[ExampleEntry {
        title: "visits 系列のランキング",
        description: "4 カテゴリの visits 系列を比率バーで表示します。",
        render: bar_list_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn bar_segment_example() -> Node {
    let data = bar_charts_sample_data();
    row(vec![
        bar_segment::root(&data, "visits").expect("固定サンプルの visits 系列合計は 0 ではない")
    ])
}

const BAR_SEGMENT_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "系列合計に対する各カテゴリの構成比を 100% 積み上げバーとして表示する",
        "合計が 0 の系列は構成比が定義できないため構築時に拒否される（ChartError::ZeroTotal）",
    ],
    arguments: &[ArgRow {
        name: "series_name",
        kind: "&str",
        default: "（必須）",
        description: "構成比の対象系列名。",
    }],
    examples: &[ExampleEntry {
        title: "構成比バー",
        description: "visits 系列の月別構成比を 100% 積み上げで表示します。",
        render: bar_segment_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn line_chart_example() -> Node {
    let data = ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
        vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
    )
    .expect("固定サンプルは常に有効");
    row(vec![line_chart::line_chart(
        &LineChartProps::new(&data, "monthly visits"),
        vec![],
    )
    .expect("固定サンプルは常に有効")])
}

const LINE_CHART_ARGUMENTS: &[ArgRow] = &[
    ArgRow {
        name: "data",
        kind: "&ChartData",
        default: "（必須）",
        description: "描画する系列データ。",
    },
    ArgRow {
        name: "aria_label",
        kind: "&str",
        default: "（必須）",
        description: "svg 要素の aria-label（データ可視化のため必須引数）。",
    },
    ArgRow {
        name: "size",
        kind: "Size",
        default: "Md",
        description: "root の CSS 表示高さを切替える寸法 variant。",
    },
    ArgRow {
        name: "width / height",
        kind: "f64",
        default: "300.0 / 150.0",
        description: "viewBox の座標系寸法。",
    },
];

const LINE_CHART_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "charts 基盤（座標スケーリング・SVG ノード木生成）を使った折れ線チャート",
        "系列色は charts::series_color_var(index) の固定色循環（color-palette 軸は非提供）",
        "積み上げ・曲線補間は非対応（charts 共通 API 側の別イシュー）",
    ],
    arguments: LINE_CHART_ARGUMENTS,
    examples: &[ExampleEntry {
        title: "単一系列の折れ線",
        description: "3 カテゴリ 1 系列の折れ線を描画します。",
        render: line_chart_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn area_chart_example() -> Node {
    let data = ChartData::new(
        vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
        vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
    )
    .expect("固定サンプルは常に有効");
    row(vec![area_chart::area_chart(
        &AreaChartProps::new(&data, "monthly visits"),
        vec![],
    )
    .expect("固定サンプルは常に有効")])
}

const AREA_CHART_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "系列ごとに折れ線 + domain 下端へ閉じた塗りつぶし面を重ねて描く自己完結チャート",
        "積み上げ・曲線補間は charts 共通 API 側の別イシュー",
        "size（Xs〜Xl）で表示高さを切り替える",
    ],
    arguments: LINE_CHART_ARGUMENTS,
    examples: &[ExampleEntry {
        title: "単一系列の面グラフ",
        description: "3 カテゴリ 1 系列の面グラフを描画します。",
        render: area_chart_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn sparkline_example() -> Node {
    let values = [10.0, 30.0, 20.0, 40.0];
    row(vec![sparkline::sparkline(
        &SparklineProps::new(&values, "weekly trend"),
        vec![],
    )
    .expect("固定サンプルは常に有効")])
}

const SPARKLINE_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "軸・ラベルなしの縮小チャート。単一系列専用（&[f64]）で複数系列は LineChart/AreaChart を使う",
        "既定 viewBox は 112×48（chakra w={28} h={12} トークン相当）",
    ],
    arguments: &[
        ArgRow { name: "values", kind: "&[f64]", default: "（必須）", description: "描画する単一系列の値列。" },
        ArgRow { name: "aria_label", kind: "&str", default: "（必須）", description: "svg 要素の aria-label。" },
        ArgRow { name: "size", kind: "Size", default: "Md", description: "root の CSS 表示高さを切替える寸法 variant。" },
    ],
    examples: &[ExampleEntry {
        title: "週次トレンド",
        description: "4 点の値列を軸なしの縮小チャートで表示します。",
        render: sparkline_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn pie_chart_example() -> Node {
    let data = ChartData::new(
        vec![
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
        ],
        vec![Series::new("revenue", vec![400.0, 300.0, 300.0, 200.0])],
    )
    .expect("固定サンプルは常に有効な ChartData を構築できる");
    let size_row = row([Size::Sm, Size::Md, Size::Lg]
        .into_iter()
        .map(|size| {
            pie_chart(
                &PieChartProps {
                    size,
                    ..PieChartProps::default()
                },
                &data,
                vec![],
            )
            .expect("固定サンプルは常に描画に成功する")
        })
        .collect());
    let with_labels = pie_chart(
        &PieChartProps {
            show_labels: true,
            ..PieChartProps::default()
        },
        &data,
        vec![],
    )
    .expect("固定サンプルは常に描画に成功する");
    stack(vec![size_row, row(vec![with_labels])])
}

const PIE_CHART_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "外部依存ゼロの SVG ノード木生成による円グラフ",
        "size（sm/md/lg）で --fandhe-pie-chart-size を切り替える",
        "show_labels を有効にするとカテゴリ名ラベルをセグメント上に描画する",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "寸法 variant。",
        },
        ArgRow {
            name: "aria_label",
            kind: "Option<&str>",
            default: "None",
            description: "None なら既定の \"pie chart\" を使う。",
        },
        ArgRow {
            name: "show_labels",
            kind: "bool",
            default: "false",
            description: "true ならカテゴリ名ラベルをセグメント上に描画する。",
        },
    ],
    examples: &[ExampleEntry {
        title: "size とラベル表示",
        description: "size 3 段とラベル表示ありの掲示です。",
        render: pie_chart_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn donut_chart_example() -> Node {
    let data = ChartData::new(
        vec![
            "Q1".to_string(),
            "Q2".to_string(),
            "Q3".to_string(),
            "Q4".to_string(),
        ],
        vec![Series::new("revenue", vec![400.0, 300.0, 300.0, 200.0])],
    )
    .expect("固定サンプルは常に有効な ChartData を構築できる");
    let size_row = row([Size::Sm, Size::Md, Size::Lg]
        .into_iter()
        .map(|size| {
            donut_chart(
                &DonutChartProps {
                    size,
                    ..DonutChartProps::default()
                },
                &data,
                vec![],
            )
            .expect("固定サンプルは常に描画に成功する")
        })
        .collect());
    let thin_ring = donut_chart(
        &DonutChartProps {
            inner_ratio: 0.85,
            show_labels: true,
            ..DonutChartProps::default()
        },
        &data,
        vec![],
    )
    .expect("inner_ratio=0.85 は許容範囲内であり常に描画に成功する");
    stack(vec![size_row, row(vec![thin_ring])])
}

const DONUT_CHART_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "外部依存ゼロの SVG ノード木生成によるドーナツグラフ",
        "inner_ratio（既定 0.6）で内径を調整できる",
        "show_labels を有効にするとカテゴリ名ラベルをセグメント上に描画する",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "寸法 variant。",
        },
        ArgRow {
            name: "inner_ratio",
            kind: "f64",
            default: "0.6",
            description: "外径に対する内径の比率。0.0 < ratio < 1.0 の範囲・有限値であること。",
        },
        ArgRow {
            name: "show_labels",
            kind: "bool",
            default: "false",
            description: "true ならカテゴリ名ラベルをセグメント上に描画する。",
        },
    ],
    examples: &[ExampleEntry {
        title: "size と内径調整",
        description: "size 3 段と、内径を細くした薄いリング（inner_ratio=0.85）の掲示です。",
        render: donut_chart_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn scatter_chart_example() -> Node {
    let data = ScatterData::new(vec![
        ScatterSeries::new(
            "cohort a",
            vec![(1.0, 2.0), (2.0, 4.5), (3.0, 3.0), (4.0, 6.0), (5.0, 5.5)],
        ),
        ScatterSeries::new(
            "cohort b",
            vec![(1.5, 1.0), (2.5, 2.5), (3.5, 4.0), (4.5, 3.5)],
        ),
    ])
    .expect("固定サンプルは常に有効");
    row(vec![scatter_chart::root(
        &data,
        ScatterChartProps::default(),
        "cohort comparison",
    )
    .expect("固定サンプルは domain・viewBox とも常に有効")])
}

const SCATTER_CHART_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "散布図専用の ScatterData（系列ごとの (x, y) 座標列）+ LinearScale（x/y 双方）+ SVG ノード木生成ヘルパーのみで組み立てる外部依存ゼロの散布図",
        "軸線・グリッド・凡例・ツールチップは charts（共通 API）側の別部品",
    ],
    arguments: &[
        ArgRow { name: "width", kind: "f64", default: "480.0", description: "viewBox の幅（px 相当）。" },
        ArgRow { name: "height", kind: "f64", default: "300.0", description: "viewBox の高さ（px 相当）。" },
        ArgRow { name: "point_radius", kind: "f64", default: "4.0", description: "点マーカーの半径（px 相当）。" },
    ],
    examples: &[ExampleEntry {
        title: "2 系列の散布図",
        description: "cohort a/b の 2 系列を重ねて表示します。",
        render: scatter_chart_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn radar_chart_example() -> Node {
    let data = ChartData::new(
        vec![
            "speed".to_string(),
            "power".to_string(),
            "range".to_string(),
            "control".to_string(),
            "armor".to_string(),
        ],
        vec![
            Series::new("mercury", vec![80.0, 60.0, 40.0, 90.0, 55.0]),
            Series::new("venus", vec![50.0, 85.0, 70.0, 45.0, 65.0]),
        ],
    )
    .expect("固定サンプルはカテゴリ数・系列長が一致する");
    row(vec![radar_chart::root(
        &data,
        RadarChartProps::default(),
        "stat comparison",
    )
    .expect(
        "固定サンプルは軸数 3 以上・非負値・viewBox とも常に有効",
    )])
}

const RADAR_CHART_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ChartData（カテゴリ = 軸、系列 = ポリゴン）+ LinearScale + SVG ノード木生成ヘルパーのみで組み立てる外部依存ゼロのレーダーチャート",
        "頂点角度は θ_i = -π/2 + i・2π/n（12 時方向開始・時計回り）の決定的な式で算出する",
        "凡例・ツールチップは charts（共通 API）側の別部品",
    ],
    arguments: &[ArgRow {
        name: "size",
        kind: "f64",
        default: "300.0",
        description: "viewBox の一辺の長さ（正方形、px 相当）。",
    }],
    examples: &[ExampleEntry {
        title: "2 系列のレーダーチャート",
        description: "5 軸（speed/power/range/control/armor）× 2 系列（mercury/venus）を重ねて表示します。",
        render: radar_chart_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

// ---------------------------------------------------------------------
// 他カテゴリ（DownloadTrigger / QrCode / Timer / ColorPicker / Calendar /
// DatePicker / DateInput）
// ---------------------------------------------------------------------

fn download_trigger_example() -> Node {
    row(vec![
        download_trigger::root(
            &DownloadTriggerProps::default(),
            "",
            Some("sample-report.pdf"),
            vec![],
            vec![text("Download report")],
        ),
        download_trigger::root(
            &DownloadTriggerProps {
                variant: fandhe_frontend_pre_styled_ui::button::ButtonVariant::Outline,
                ..DownloadTriggerProps::default()
            },
            "",
            Some("sample-report.pdf"),
            vec![],
            vec![text("Outline")],
        ),
    ])
}

const DOWNLOAD_TRIGGER_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "ダウンロードリンク（<a download>）を button 部品と同じ variant/size/colorPalette 軸で styled 化する",
        "button::ButtonVariant を再利用するため見た目は Button と統一される",
    ],
    arguments: &[
        ArgRow { name: "variant", kind: "ButtonVariant", default: "Solid", description: "見た目 variant（button と共通）。" },
        ArgRow { name: "size", kind: "Size", default: "Md", description: "サイズ variant。" },
        ArgRow { name: "palette", kind: "ColorPalette", default: "Accent", description: "colorPalette 軸。" },
        ArgRow { name: "href", kind: "&str", default: "（必須）", description: "ダウンロード対象のリソース URL。" },
        ArgRow { name: "file_name", kind: "Option<&str>", default: "None", description: "download 属性へ指定するファイル名。" },
        ArgRow { name: "attrs", kind: "Vec<(&str, &str)>", default: "", description: "root（<a>）パーツへ合成する追加属性。" },
        ArgRow { name: "children", kind: "Vec<Node>", default: "", description: "リンクラベルとなる子ノード。" },
    ],
    examples: &[ExampleEntry {
        title: "variant の切り替え",
        description: "Solid/Outline の 2 種を並べます。",
        render: download_trigger_example,
    }],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn qr_code_example() -> Node {
    let matrix = qr_code::encode(
        "https://fandhe-frontend.example/",
        qr_code::ErrorCorrectionLevel::M,
    )
    .expect("固定 URL はバージョン 40 容量内に収まる");
    let demo = |size: Size| {
        qr_code::root(
            size,
            vec![],
            vec![qr_code::frame(
                &matrix,
                qr_code::DEFAULT_QUIET_ZONE,
                Some("QR code linking to https://fandhe-frontend.example/"),
                vec![],
                vec![qr_code::pattern(
                    &matrix,
                    qr_code::DEFAULT_QUIET_ZONE,
                    vec![],
                )],
            )],
        )
    };
    row(vec![
        demo(Size::Xs),
        demo(Size::Sm),
        demo(Size::Md),
        demo(Size::Lg),
        demo(Size::Xl),
    ])
}

/// overlay 付き 1 態を掲示する例（イシュー #1565: overlay の中央固定・
/// 背景・角丸を視覚確認するための追加例）。
fn qr_code_overlay_example() -> Node {
    let matrix = qr_code::encode(
        "https://fandhe-frontend.example/",
        qr_code::ErrorCorrectionLevel::Q,
    )
    .expect("固定 URL はバージョン 40 容量内に収まる");
    let with_overlay = qr_code::root(
        Size::Lg,
        vec![],
        vec![
            qr_code::frame(
                &matrix,
                qr_code::DEFAULT_QUIET_ZONE,
                Some("QR code linking to https://fandhe-frontend.example/"),
                vec![],
                vec![qr_code::pattern(
                    &matrix,
                    qr_code::DEFAULT_QUIET_ZONE,
                    vec![],
                )],
            ),
            qr_code::overlay(vec![], vec![text("FW")]),
        ],
    );
    row(vec![with_overlay])
}

const QR_CODE_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "外部依存ゼロの QR Model 2（ISO/IEC 18004）byte モードエンコーダによる QR コード表示",
        "size（xs/sm/md/lg/xl）で --fandhe-qr-code-size を切り替える",
        "Overlay パーツは frame 中央に --fandhe-qr-code-size の 1/3 幅で固定され、背景・角丸付きでロゴ等の呼び出し側コンテンツの可読性を確保する",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "寸法 variant。",
        },
        ArgRow {
            name: "encode(data, level)",
            kind: "fn(&str, ErrorCorrectionLevel) -> Result<Matrix, QrError>",
            default: "",
            description: "文字列を QR コード行列へエンコードする（headless-ui 側の純粋関数）。",
        },
    ],
    examples: &[
        ExampleEntry {
            title: "size 5 段",
            description: "同一 URL を xs/sm/md/lg/xl で表示します。",
            render: qr_code_example,
        },
        ExampleEntry {
            title: "overlay 付き",
            description: "ロゴ等を中央に重ねる場合の表示です。読み取り精度を確保するため error correction level は Q 以上を推奨します。",
            render: qr_code_overlay_example,
        },
    ],
    keyboard: &[],
    aria: &[],
    demo: None,
};

fn timer_example() -> Node {
    let mut t = Timer::countdown(90_000, 1_000);
    dispatch(&mut t, "timer:start", "");
    dispatch(&mut t, "timer:tick", "35000");
    let (_, _, minutes, seconds) = t.display_segments();
    row(vec![t.root(
        vec![],
        vec![
            timer::area(
                vec![],
                vec![
                    timer::item(
                        TimerUnit::Minutes,
                        vec![],
                        vec![
                            timer::item_value(
                                TimerUnit::Minutes,
                                vec![],
                                vec![text(timer::format_segment(minutes))],
                            ),
                            timer::item_label(TimerUnit::Minutes, vec![], vec![text("Min")]),
                        ],
                    ),
                    timer::separator(vec![], vec![text(":")]),
                    timer::item(
                        TimerUnit::Seconds,
                        vec![],
                        vec![
                            timer::item_value(
                                TimerUnit::Seconds,
                                vec![],
                                vec![text(timer::format_segment(seconds))],
                            ),
                            timer::item_label(TimerUnit::Seconds, vec![], vec![text("Sec")]),
                        ],
                    ),
                ],
            ),
            timer::control(
                vec![],
                vec![
                    timer::action_trigger(TimerControl::Pause, vec![], vec![text("Pause")]),
                    timer::action_trigger(TimerControl::Reset, vec![], vec![text("Reset")]),
                ],
            ),
        ],
    )])
}

const TIMER_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "tick 注入型・idle/running/paused/completed の決定的状態機械（fandhe-frontend-interactive の dispatch 経由）",
        "countdown（カウントダウン）と count_up（カウントアップ）の 2 モードを持つ",
        "実 tick 駆動（setInterval）は fandhe-frontend-wasm-full 側のスコープ（本ページは SSR 静的掲示）",
        "root の data-state（completed / paused）に応じた item-value の色切り替え・action-trigger の hover / focus ring",
    ],
    arguments: &[
        ArgRow { name: "Timer::countdown", kind: "fn(start_ms, interval_ms) -> Timer", default: "", description: "カウントダウン型 Timer を構築する。" },
        ArgRow { name: "Timer::count_up", kind: "fn(target_ms, interval_ms) -> Timer", default: "", description: "カウントアップ型 Timer を構築する。" },
    ],
    examples: &[ExampleEntry {
        title: "90 秒カウントダウン（35 秒経過）",
        description: "start → 35 秒分の tick を注入した running 状態を固定表示します。",
        render: timer_example,
    }],
    keyboard: &[],
    aria: &[
        AriaRow { attribute: "aria-live", description: "経過表示の更新を支援技術へ通知する（headless-ui 側の root パーツが付与）。" },
    ],
    demo: None,
};

fn color_picker_example() -> Node {
    let state = ColorPicker::from_color(Color::from_rgba(Rgb::new(0x3b, 0x82, 0xf6), 0xcc));
    row(vec![color_picker::content(
        state.state(),
        None,
        vec![],
        vec![
            color_picker::area(
                &state,
                vec![],
                vec![
                    color_picker::area_background(&state, vec![], vec![]),
                    color_picker::area_thumb(&state, false, vec![], vec![]),
                ],
            ),
            color_picker::channel_slider(
                Channel::Hue,
                &state,
                vec![],
                vec![
                    color_picker::channel_slider_track(Channel::Hue, &state, vec![], vec![]),
                    color_picker::channel_slider_thumb(Channel::Hue, &state, false, vec![], vec![]),
                ],
            ),
        ],
    )])
}

const COLOR_PICKER_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "彩度・明度の 2 次元グラデーション（Area）+ 色相/アルファスライダー + HEX 入力からなる複合部品",
        "canvas を使わず、すべて CSS グラデーション + 検証済み整数割合の custom property のみで描画する",
        "開いた状態を固定して掲示する（実際のドラッグ挙動は wasm 層のスコープ外）",
    ],
    arguments: &[
        ArgRow { name: "ColorPicker::from_color", kind: "fn(Color) -> ColorPicker", default: "", description: "初期色から状態機械を構築する。" },
        ArgRow { name: "channel", kind: "Channel", default: "", description: "channel_slider が制御するチャンネル（Hue/Alpha）。" },
        ArgRow { name: "attrs", kind: "Vec<(&str, &str)>", default: "", description: "root パーツへ合成する追加属性。" },
        ArgRow { name: "children", kind: "Vec<Node>", default: "", description: "root 配下の子ノード（通常 trigger/area を含む）。" },
    ],
    examples: &[ExampleEntry {
        title: "Area + 色相スライダー",
        description: "彩度・明度のグラデーション領域と色相スライダーを組み合わせます。",
        render: color_picker_example,
    }],
    keyboard: &[],
    aria: &[
        AriaRow { attribute: "aria-valuenow", description: "スライダー（thumb）の現在値を支援技術へ伝える。" },
        AriaRow { attribute: "aria-valuetext", description: "スライダーの現在値の読み上げ用テキスト表現。" },
    ],
    demo: None,
};

fn calendar_example() -> Node {
    let today = PlainDate::new(2026, 7, 22).unwrap();
    let selected = PlainDate::new(2026, 7, 15).unwrap();
    let weekday_labels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let header_row = calendar::table_row(
        vec![],
        weekday_labels
            .iter()
            .map(|l| calendar::table_head_cell(vec![], vec![text(*l)]))
            .collect(),
    );
    let first_of_month = PlainDate::new(2026, 7, 1).unwrap();
    let grid_start = first_of_month.add_days(-2).unwrap();
    let body_rows: Vec<Node> = (0..5)
        .map(|week| {
            let cells: Vec<Node> = (0..7)
                .map(|day| {
                    let date = grid_start.add_days(week * 7 + day).unwrap();
                    let is_selected = date == selected;
                    let is_today = date == today;
                    let is_outside = date.month() != 7 || date.year() != 2026;
                    calendar::table_cell(
                        is_selected,
                        vec![],
                        vec![calendar::day_trigger(
                            date,
                            is_selected,
                            is_today,
                            is_outside,
                            false,
                            None,
                            vec![],
                            vec![text(date.day().to_string())],
                        )],
                    )
                })
                .collect();
            calendar::table_row(vec![], cells)
        })
        .collect();
    row(vec![calendar::root(
        Size::Md,
        vec![],
        vec![
            calendar::heading(
                Some("spec-948-calendar-heading"),
                vec![],
                vec![text("July 2026")],
            ),
            calendar::prev_trigger(false, vec![], vec![text("‹")]),
            calendar::next_trigger(false, vec![], vec![text("›")]),
            calendar::table(
                Some("spec-948-calendar-heading"),
                vec![],
                vec![
                    calendar::table_header(vec![], vec![header_row]),
                    calendar::table_body(vec![], body_rows),
                ],
            ),
        ],
    )])
}

const CALENDAR_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "role=\"grid\" の月グリッド静的掲示（headless-ui の Calendar に recipe CSS を適用）",
        "今日・選択日・表示月外セルの見た目を data-* 属性連動で区別する",
        "キーボードナビゲーション・クリック挙動は wasm 層のスコープ（本ページは SSR 静的表示）",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "寸法 variant。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード（通常 Heading/Table を含む）。",
        },
    ],
    examples: &[ExampleEntry {
        title: "2026-07 の月グリッド",
        description: "週開始 Monday で、今日（07-22）・選択日（07-15）を固定表示します。",
        render: calendar_example,
    }],
    keyboard: &[
        KeyRow {
            key: "ArrowLeft / ArrowRight / ArrowUp / ArrowDown",
            description: "日付グリッド内をフォーカス移動する（wasm 層実装）。",
        },
        KeyRow {
            key: "Enter / Space",
            description: "フォーカス中の日付を選択する（wasm 層実装）。",
        },
    ],
    aria: &[AriaRow {
        attribute: "aria-selected",
        description: "選択中の日付セルに付与する。",
    }],
    demo: None,
};

fn date_picker_example() -> Node {
    let today = PlainDate::new(2026, 7, 22).unwrap();
    let selected = PlainDate::new(2026, 7, 15).unwrap();
    let weekday_labels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let header_row = calendar::table_row(
        vec![],
        weekday_labels
            .iter()
            .map(|l| calendar::table_head_cell(vec![], vec![text(*l)]))
            .collect(),
    );
    let first_of_month = PlainDate::new(2026, 7, 1).unwrap();
    let grid_start = first_of_month.add_days(-2).unwrap();
    let body_rows: Vec<Node> = (0..5)
        .map(|week| {
            let cells: Vec<Node> = (0..7)
                .map(|day| {
                    let date = grid_start.add_days(week * 7 + day).unwrap();
                    let is_selected = date == selected;
                    let is_today = date == today;
                    let is_outside = date.month() != 7 || date.year() != 2026;
                    calendar::table_cell(
                        is_selected,
                        vec![],
                        vec![calendar::day_trigger(
                            date,
                            is_selected,
                            is_today,
                            is_outside,
                            false,
                            None,
                            vec![],
                            vec![text(date.day().to_string())],
                        )],
                    )
                })
                .collect();
            calendar::table_row(vec![], cells)
        })
        .collect();
    row(vec![date_picker::root(
        Size::Md,
        OpenState::Open,
        vec![],
        vec![
            date_picker::label(
                Some("spec-948-date-picker-label"),
                vec![],
                vec![text("Delivery date")],
            ),
            date_picker::control(
                OpenState::Open,
                vec![],
                vec![
                    date_picker::input(Some("2026-07-15"), false, None, vec![]),
                    date_picker::trigger(
                        OpenState::Open,
                        false,
                        Some("spec-948-date-picker-content"),
                        vec![],
                        vec![text("📅")],
                    ),
                ],
            ),
            date_picker::positioner(
                OpenState::Open,
                vec![],
                vec![date_picker::content(
                    OpenState::Open,
                    Some("spec-948-date-picker-content"),
                    Some("spec-948-date-picker-label"),
                    vec![],
                    vec![calendar::table(
                        None,
                        vec![],
                        vec![
                            calendar::table_header(vec![], vec![header_row]),
                            calendar::table_body(vec![], body_rows),
                        ],
                    )],
                )],
            ),
        ],
    )])
}

const DATE_PICKER_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "popover 基盤 + Calendar 合成の日付選択部品",
        "popover が開いた状態を固定表示し、positioner はフロー内配置へ中和する",
        "size（sm/md/lg）で寸法を切り替える",
    ],
    arguments: &[
        ArgRow {
            name: "size",
            kind: "Size",
            default: "Md",
            description: "寸法 variant。",
        },
        ArgRow {
            name: "state",
            kind: "OpenState",
            default: "Closed",
            description: "popover の開閉状態。",
        },
        ArgRow {
            name: "attrs",
            kind: "Vec<(&str, &str)>",
            default: "",
            description: "root パーツへ合成する追加属性。",
        },
        ArgRow {
            name: "children",
            kind: "Vec<Node>",
            default: "",
            description: "root 配下の子ノード。",
        },
    ],
    examples: &[ExampleEntry {
        title: "開いた状態の掲示",
        description: "入力欄 + トリガー + Calendar 月グリッドを popover 内に合成します。",
        render: date_picker_example,
    }],
    keyboard: &[KeyRow {
        key: "Escape",
        description: "popover を閉じる（wasm 層実装）。",
    }],
    aria: &[AriaRow {
        attribute: "aria-expanded",
        description: "トリガーが popover の開閉状態を伝える。",
    }],
    demo: None,
};

fn date_input_example() -> Node {
    let build = |id_prefix: &str, state: &DateInput, size: Size, disabled: bool| {
        date_input::root(
            size,
            disabled,
            state.is_invalid(),
            vec![],
            vec![
                date_input::label(
                    disabled,
                    state.is_invalid(),
                    Some(&format!("{id_prefix}-year")),
                    vec![],
                    vec![text("Date")],
                ),
                date_input::control(
                    disabled,
                    state.is_invalid(),
                    vec![],
                    vec![
                        date_input::segment_group(
                            disabled,
                            state.is_invalid(),
                            vec![],
                            vec![
                                state.segment(DateSegment::Year, disabled, false, vec![]),
                                state.segment(DateSegment::Month, disabled, false, vec![]),
                                state.segment(DateSegment::Day, disabled, false, vec![]),
                            ],
                        ),
                        state.hidden_input(&format!("{id_prefix}-value"), disabled, vec![]),
                    ],
                ),
            ],
        )
    };
    let filled_state = DateInput::new(Some(2026), Some(7), Some(22), None, None);
    let filled = build("spec-948-date-input-filled", &filled_state, Size::Md, false);
    let empty_state_value = DateInput::default();
    let empty = build(
        "spec-948-date-input-empty",
        &empty_state_value,
        Size::Md,
        false,
    );
    row(vec![filled, empty])
}

const DATE_INPUT_SPEC: ComponentPageSpec = ComponentPageSpec {
    features: &[
        "年/月/日の 3 セグメント個別入力による日付フィールド",
        "fail-closed な日付検証（例: 2024-02-30 は実在しないため invalid 状態）",
        "hidden_input で通常のフォーム送信に載る単一値（ISO 8601）を提供する",
    ],
    arguments: &[
        ArgRow { name: "size", kind: "Size", default: "Md", description: "寸法 variant。" },
        ArgRow { name: "disabled", kind: "bool", default: "false", description: "全セグメントを無効化する。" },
        ArgRow { name: "invalid", kind: "bool", default: "false", description: "3 セグメントいずれかが不正な組み合わせの場合に true。" },
        ArgRow { name: "attrs", kind: "Vec<(&str, &str)>", default: "", description: "root パーツへ合成する追加属性。" },
        ArgRow { name: "children", kind: "Vec<Node>", default: "", description: "root 配下の子ノード（通常 DateSegment 列を含む）。" },
    ],
    examples: &[ExampleEntry {
        title: "入力済み・未入力の 2 態",
        description: "2026-07-22 が入力済みの状態と、3 セグメントとも placeholder 表示の未入力状態を並べます。",
        render: date_input_example,
    }],
    keyboard: &[
        KeyRow { key: "ArrowUp / ArrowDown", description: "フォーカス中セグメントの値を増減する（wasm 層実装）。" },
        KeyRow { key: "ArrowLeft / ArrowRight", description: "セグメント間をフォーカス移動する（wasm 層実装）。" },
    ],
    aria: &[AriaRow { attribute: "aria-invalid", description: "3 セグメントの組み合わせが実在しない日付の場合に付与する。" }],
    demo: None,
};

// ---------------------------------------------------------------------
// レジストリ本体
// ---------------------------------------------------------------------

/// イシュー #948 が供給する `path -> ComponentPageSpec` の登録テーブル。
/// [`crate::component_page::SPEC_SOURCES`] から集約される（並列 4 PR の
/// コンフリクトを避けるためフラットな別ファイルとして分離、モジュール doc
/// 参照）。
pub const SPECS: &[(&str, ComponentPageSpec)] = &[
    ("/themes/blockquote/", BLOCKQUOTE_SPEC),
    ("/themes/code/", CODE_SPEC),
    ("/themes/em/", EM_SPEC),
    ("/themes/heading/", HEADING_SPEC),
    ("/themes/highlight/", HIGHLIGHT_SPEC),
    ("/themes/kbd/", KBD_SPEC),
    ("/themes/list/", LIST_SPEC),
    ("/themes/mark/", MARK_SPEC),
    ("/themes/text/", TEXT_SPEC),
    ("/themes/visually-hidden/", VISUALLY_HIDDEN_SPEC),
    ("/themes/charts/", CHARTS_SPEC),
    ("/themes/area-chart/", AREA_CHART_SPEC),
    ("/themes/bar-chart/", BAR_CHART_SPEC),
    ("/themes/bar-list/", BAR_LIST_SPEC),
    ("/themes/bar-segment/", BAR_SEGMENT_SPEC),
    ("/themes/donut-chart/", DONUT_CHART_SPEC),
    ("/themes/line-chart/", LINE_CHART_SPEC),
    ("/themes/pie-chart/", PIE_CHART_SPEC),
    ("/themes/radar-chart/", RADAR_CHART_SPEC),
    ("/themes/scatter-chart/", SCATTER_CHART_SPEC),
    ("/themes/sparkline/", SPARKLINE_SPEC),
    ("/themes/download-trigger/", DOWNLOAD_TRIGGER_SPEC),
    ("/themes/qr-code/", QR_CODE_SPEC),
    ("/themes/timer/", TIMER_SPEC),
    ("/themes/color-picker/", COLOR_PICKER_SPEC),
    ("/themes/calendar/", CALENDAR_SPEC),
    ("/themes/date-picker/", DATE_PICKER_SPEC),
    ("/themes/date-input/", DATE_INPUT_SPEC),
];
