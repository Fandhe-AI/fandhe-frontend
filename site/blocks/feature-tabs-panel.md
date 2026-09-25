# feature-tabs-panel

`badge` / `heading` / `text` / `tabs` / `image` / `card` / `icon` / `button` /
`progress` の 9 部品を合成した、タブで切り替える feature セクションです。
5 つの形を縦に並べます。

- **基準形（対応表 ID R1158）**: 見出しの下に下線タブ（`TabsVariant::Line`）
  を並べ、選んだタブのパネルだけにテキストと画像を表示します。
- **形 C（対応表 ID R0478）**: ピル型タブ（`TabsVariant::Enclosed`）+ 画像を
  持たない単一カラムのパネル。
- **形 D（対応表 ID R0479）**: 1 パネルの中に複数のフィーチャー行を積み、
  行ごとに画像とテキストの左右を入れ替えます。
- **形 E（対応表 ID R0481）**: 中央寄せの見出し + タブ + パネル内カード
  グリッド。末尾に「すべての機能を見る」の静的 CTA を添えます。
- **形 F（対応表 ID R0104）**: トリガーに短い説明文と進捗バーを添えます。
  自動切替は行いません。

docs サイトは JS ハイドレーションを行わないため、各形とも実物の `tabs`
コンポーネント（選択できるように見えるが実際には切り替わらないトリガー
ボタンを持つ、他の Themes 部品ページの Demo と同じ静的プレビュー）は
選択中のタブ 1 個だけを描画します。残りのタブの内容は、`tabs` を複製せず
見出しキャプション付きの非対話表示（トリガーボタン・`role="tab"`・
`tabindex` を持たない）として静的に併記し、すべてのパネル本文が常に可視
のまま静的 HTML に現れるようにしています。

文言・カードの見出しと説明・進捗値はすべて架空のもので、データ取得・送信は
行わない静的な表示例です。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
// #2773: 形 F（進捗バー付きトリガー）が `Progress` を直接組み立てるための
// import。`pre_styled_ui::progress` は `Progress` 自体を再エクスポートしない
// 契約（同モジュール rustdoc 参照）ため、`showcase.rs::progress_section` と
// 同じ経路（`pre_styled_ui::fandhe_frontend_headless_ui`）で取り込む。
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 基準形（R1158）1 タブ分のデータ（架空文言）。
struct PanelData {
    value: &'static str,
    label: &'static str,
    title: &'static str,
    body: &'static str,
    image_src: &'static str,
}

/// 基準形の 4 タブ分（#2773 が追加する形もこの配列を再利用できる）。
const PANELS: [PanelData; 4] = [
    PanelData {
        value: "design",
        label: "設計",
        title: "型で不変条件を保証する設計",
        body: "コンポーネント境界と状態遷移を型で表現し、実行時ではなくコンパイル時に誤りを検出します。",
        image_src: dummy_assets::SCREENSHOT_SRC,
    },
    PanelData {
        value: "integration",
        label: "連携",
        title: "既存システムへの段階的な組み込み",
        body: "部分埋め込みからフル機能構成まで、必要な範囲だけを選んで既存ページへ組み込めます。",
        image_src: dummy_assets::PRODUCT_SRC,
    },
    PanelData {
        value: "operations",
        label: "運用",
        title: "単一実行ファイルでの安定運用",
        body: "サーバーとアセットをひとまとめにし、Docker イメージ 1 枚で決定的にデプロイできます。",
        image_src: dummy_assets::BACKGROUND_SRC,
    },
    PanelData {
        value: "analytics",
        label: "分析",
        title: "ビルド成果物の可視化",
        body: "依存グラフとバンドルサイズを継続的に計測し、変化を CI 上で追跡できます。",
        image_src: dummy_assets::LOGO_SRC,
    },
];

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
/// 本イシューでは基準形（R1158）の 1 件のみを出す（#2773 が形を追加する
/// 際に再利用する）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// セクション見出し（badge + 見出し + 説明文）。ページ側が `## Demo` として
/// `h2` を出すため、見出しは `HeadingLevel::H3` にする。`centered` は形 E
/// （[`variant_card_grid`]、対応表 ID R0481「中央見出し」）専用の中央寄せ
/// フック（`feature_image_cards::header_start` の `data-align` と同型の
/// 判断、[`LAYOUT_CSS`] 側で対応する規則を持つ）。
fn section_header(eyebrow: &'static str, title: &'static str, lead: &'static str) -> Node {
    section_header_aligned(eyebrow, title, lead, false)
}

/// [`section_header`] の実体（`centered` 付き）。
fn section_header_aligned(
    eyebrow: &'static str,
    title: &'static str,
    lead: &'static str,
    centered: bool,
) -> Node {
    let mut attrs = vec![("class", "blocks-feature-tabs-panel-header")];
    if centered {
        attrs.push(("data-align", "center"));
    }
    div(
        attrs,
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(lead)],
            ),
        ],
    )
}

/// パネル内容（テキスト列 → 画像列、DOM 順は固定）。`reverse` が `true` の
/// とき [`LAYOUT_CSS`] の lg ブレークポイントで左右の `grid-column` を
/// 入れ替える（形 D・[`panel_alternating_rows`] 参照。DOM 順自体は変えない
/// ため読み上げ順は不変）。
fn panel_row_reversible(data: &PanelData, reverse: bool) -> Node {
    let mut attrs = vec![("class", "blocks-feature-tabs-panel-row")];
    if reverse {
        attrs.push(("data-blocks-feature-tabs-panel-reverse", ""));
    }
    div(
        attrs,
        vec![
            div(
                vec![("class", "blocks-feature-tabs-panel-copy")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(data.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(data.body)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-feature-tabs-panel-media")],
                vec![image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        aspect_ratio: AspectRatio::Landscape,
                        shape: ImageShape::Rounded,
                        ..ImageProps::new(data.image_src, "")
                    },
                    vec![("data-blocks-feature-tabs-panel-image", "")],
                )],
            ),
        ],
    )
}

/// [`panel_row_reversible`] の既定形（`reverse: false`）。基準形（R1158）・
/// 形 C・形 F が使う。
fn panel_row(data: &PanelData) -> Vec<Node> {
    vec![panel_row_reversible(data, false)]
}

/// 形 C（対応表 ID R0478）専用のパネル内容: 画像を持たない単一カラム
/// （見出し + 本文のみ）。「単一パネル」の差分を、既存 2 列の
/// [`panel_row`] とは別クラス（`.blocks-feature-tabs-panel-single`）で
/// 表現する。
fn panel_single(data: &PanelData) -> Vec<Node> {
    vec![div(
        vec![("class", "blocks-feature-tabs-panel-single")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(data.title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(data.body)],
            ),
        ],
    )]
}

/// 形 D（対応表 ID R0479）専用のパネル内容: 1 パネルの中に複数のフィーチャー
/// 行を積み、行ごとに左右を入れ替える（`feature_alternating_rows` と同型の
/// 発想をパネル内スケールへ適用。奇数添字（2 行目・4 行目…）だけ
/// `reverse: true` にする）。
fn panel_alternating_rows(items: &[PanelData]) -> Vec<Node> {
    items
        .iter()
        .enumerate()
        .map(|(index, data)| panel_row_reversible(data, index % 2 == 1))
        .collect()
}

/// [`PANELS`] から `tabs::tabs` の `items` を組み立てる（#2773 が variant を
/// 変えつつ再利用する共通ヘルパ）。
fn panel_items() -> Vec<TabItem<'static>> {
    PANELS
        .iter()
        .map(|data| TabItem {
            value: data.value,
            trigger: vec![text(data.label)],
            content: panel_row(data),
            disabled: false,
        })
        .collect()
}

/// `id`（呼び出し側が `blocks-feature-tabs-panel-<接尾辞>` の形で完全指定
/// する）・variant・選択中タブ・`items` を引数に取る tabs 組み立てヘルパ
/// （5 形すべてが再利用する）。`id` を呼び出し側が組み立てる形にすることで
/// 「id の基底は常に `blocks-feature-tabs-panel-` で始まる」という
/// モジュール doc「id 規約」節の不変条件を、内部で接尾辞から静的文字列へ
/// 変換するテーブル（変換漏れがあっても素通りしてしまう）を持たずに
/// 呼び出し箇所ごとのリテラルとして機械的に確認できるようにする。
fn tabs_panel(
    id: &'static str,
    variant: TabsVariant,
    selected: &'static str,
    items: Vec<TabItem<'static>>,
) -> Node {
    tabs::tabs(
        variant,
        Size::Md,
        ColorPalette::Accent,
        &TabsProps {
            id,
            selected,
            orientation: Orientation::Horizontal,
            activation_mode: ActivationMode::Automatic,
            loop_focus: true,
            indicator: false,
        },
        items,
    )
}

/// 実物の `tabs::tabs` を複製せず、キャプション付きの非対話表示として
/// `content` を静的に併記する（モジュール doc「無 JS での扱い」節参照。
/// Codex P1 指摘の是正: 操作可能に見えて実際には切り替わらないトリガー
/// ボタン〔`role="tab"`/`type="button"`/`tabindex`〕を反復して出さない）。
/// `content` は `panel_row`/`panel_single`/`panel_alternating_rows`/
/// `panel_card_grid` のいずれかの出力を受け取り、`tabs::tabs`/`ANATOMY` を
/// 一切経由しないため `role`/`tabindex`/`<button>` を持たない（5 形すべてが
/// 再利用する）。
fn tab_preview(caption: String, content: Vec<Node>) -> Node {
    let mut children = vec![styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(caption)],
    )];
    children.extend(content);
    div(
        vec![("class", "blocks-feature-tabs-panel-preview")],
        children,
    )
}

/// 「「{label}」タブを選択した場合のプレビュー」キャプションを組み立てる
/// [`tab_preview`] の薄いラッパ（基準形・形 C・形 F が使う共通文言）。
fn panel_state_preview(data: &PanelData) -> Node {
    tab_preview(
        format!("「{}」タブを選択した場合のプレビュー", data.label),
        panel_row(data),
    )
}

/// 基準形（R1158）: 見出し + 下線タブ（[`TabsVariant::Line`]）+
/// テキスト/画像パネル。docs サイトは JS ハイドレーションを行わないため
/// （モジュール doc「無 JS での扱い」節）、実物の `tabs::tabs`（[`PANELS`]
/// 先頭の `design` を選択済みとする 1 個だけ）を描画したあと、残り 3
/// パネルは [`panel_state_preview`] による非対話プレビューとして併記する。
/// これにより 4 パネルすべての本文が常に可視のまま静的 HTML に現れる。
fn variant_basic() -> Node {
    let mut children = vec![
        section_header(
            "機能紹介",
            "タブで切り替える機能セクション",
            "見出しの下にタブを並べ、選んだタブの内容だけを表示します。",
        ),
        tabs_panel(
            "blocks-feature-tabs-panel-basic",
            TabsVariant::Line,
            PANELS[0].value,
            panel_items(),
        ),
    ];
    for panel in &PANELS[1..] {
        children.push(panel_state_preview(panel));
    }
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
    )
}

/// 形 C（対応表 ID R0478）: ピル型タブ（[`TabsVariant::Enclosed`]）+ 単一
/// カラムのパネル（[`panel_single`]、画像を持たない）。[`PANELS`] の先頭 2
/// 件のみ使う（重複が過大にならないよう絞る、モジュール doc「#2772 と
/// #2773 の分担」節）。
fn variant_pill() -> Node {
    let items = vec![
        TabItem {
            value: PANELS[0].value,
            trigger: vec![text(PANELS[0].label)],
            content: panel_single(&PANELS[0]),
            disabled: false,
        },
        TabItem {
            value: PANELS[1].value,
            trigger: vec![text(PANELS[1].label)],
            content: panel_single(&PANELS[1]),
            disabled: false,
        },
    ];
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        vec![
            section_header(
                "コンパクト表示",
                "ピル型タブで切り替える単一パネル",
                "画像を持たない単一カラムのパネルを、ピル型のタブで切り替えます。",
            ),
            tabs_panel(
                "blocks-feature-tabs-panel-pill",
                TabsVariant::Enclosed,
                PANELS[0].value,
                items,
            ),
            tab_preview(
                format!("「{}」タブを選択した場合のプレビュー", PANELS[1].label),
                panel_single(&PANELS[1]),
            ),
        ],
    )
}

/// 形 D（対応表 ID R0479）: 1 タブの中に複数のフィーチャー行を積み、行
/// ごとに左右を入れ替える（[`panel_alternating_rows`]）。[`PANELS`] を
/// 2 件ずつ 2 組に分け、タブ切り替えで組を入れ替える。
fn variant_alternating() -> Node {
    let items = vec![
        TabItem {
            value: "set-a",
            trigger: vec![text("セット A")],
            content: panel_alternating_rows(&PANELS[0..2]),
            disabled: false,
        },
        TabItem {
            value: "set-b",
            trigger: vec![text("セット B")],
            content: panel_alternating_rows(&PANELS[2..4]),
            disabled: false,
        },
    ];
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        vec![
            section_header(
                "詳しい紹介",
                "パネル内で左右を入れ替える複数行",
                "1 つのパネルに複数の項目を積み、行ごとに画像とテキストの左右を入れ替えます。",
            ),
            tabs_panel(
                "blocks-feature-tabs-panel-alternating",
                TabsVariant::Line,
                "set-a",
                items,
            ),
            tab_preview(
                "「セット B」タブを選択した場合のプレビュー".to_string(),
                panel_alternating_rows(&PANELS[2..4]),
            ),
        ],
    )
}

/// 形 E（対応表 ID R0481）1 枚分のカードデータ（架空）。
struct GridCard {
    title: &'static str,
    body: &'static str,
}

/// 形 E タブ「特長」のカード 3 枚（架空）。
const CARDS_FEATURES: [GridCard; 3] = [
    GridCard {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
    },
    GridCard {
        title: "外部依存ゼロの描画コア",
        body: "描画コアは外部クレートに依存せず、サプライチェーンの露出面を抑えます。",
    },
    GridCard {
        title: "単一実行ファイル配布",
        body: "SSR/SSG のいずれも単一バイナリへまとめてデプロイできます。",
    },
];

/// 形 E タブ「導入事例」のカード 3 枚（架空）。
const CARDS_CASES: [GridCard; 3] = [
    GridCard {
        title: "段階的な組み込み",
        body: "部分埋め込みからフル機能構成まで、必要な範囲だけを選べます。",
    },
    GridCard {
        title: "既存チームでの運用",
        body: "既存の CI・レビュー体制へそのまま組み込んで運用できます。",
    },
    GridCard {
        title: "継続的な計測",
        body: "依存グラフとバンドルサイズを継続的に計測し、変化を追跡できます。",
    },
];

/// 形 E のカードグリッド用アイコン（自作の幾何線画。取得元アイコンセットの
/// path・識別子は複製しない、`feature_accordion_image::category_icon` と
/// 同型の判断）。
fn grid_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M12 3l9 4.5-9 4.5-9-4.5z"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// カード 1 枚（アイコン + 見出し + 説明）を組み立てる。
fn grid_card(card: &GridCard) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-tabs-panel-card", "")],
        vec![card::body(
            vec![("class", "blocks-feature-tabs-panel-card-body")],
            vec![
                grid_icon(),
                heading(
                    HeadingLevel::H4,
                    &HeadingProps::default(),
                    vec![],
                    vec![text(card.title)],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(card.body)],
                ),
            ],
        )],
    )
}

/// 形 E 専用のパネル内容: カードグリッド。
fn panel_card_grid(cards: &[GridCard]) -> Vec<Node> {
    vec![div(
        vec![("class", "blocks-feature-tabs-panel-card-grid")],
        cards.iter().map(grid_card).collect(),
    )]
}

/// 形 E 末尾の CTA ボタン。押しても何も起きない静的表示のため
/// `disabled: true`（ネイティブ `disabled` + `aria-disabled="true"` を
/// 自動付与、`feature_accordion_image::category_button` と同型の判断で
/// dead control 化を防ぐ）。
fn grid_cta() -> Node {
    div(
        vec![("class", "blocks-feature-tabs-panel-cta")],
        vec![button::button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                size: Size::Sm,
                disabled: true,
                ..ButtonProps::default()
            },
            vec![],
            vec![text("すべての機能を見る")],
        )],
    )
}

/// 形 E（対応表 ID R0481）: 中央寄せ見出し + タブ + パネル内カードグリッド。
fn variant_card_grid() -> Node {
    let items = vec![
        TabItem {
            value: "features",
            trigger: vec![text("特長")],
            content: panel_card_grid(&CARDS_FEATURES),
            disabled: false,
        },
        TabItem {
            value: "cases",
            trigger: vec![text("導入事例")],
            content: panel_card_grid(&CARDS_CASES),
            disabled: false,
        },
    ];
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        vec![
            section_header_aligned(
                "多角的に紹介",
                "カードグリッドで機能をまとめる",
                "タブを切り替えると、紹介するカードの組が変わります。",
                true,
            ),
            tabs_panel(
                "blocks-feature-tabs-panel-cards",
                TabsVariant::Line,
                "features",
                items,
            ),
            tab_preview(
                "「導入事例」タブを選択した場合のプレビュー".to_string(),
                panel_card_grid(&CARDS_CASES),
            ),
            grid_cta(),
        ],
    )
}

/// 形 F（対応表 ID R0104）専用のトリガー内容: ラベル + 短い説明文 +
/// 進捗バー（[`Progress`]）。自動切替は持たず初期タブ固定のまま
/// （docs サイトは元々無 JS のため、この制約は構造的に満たされる）。
fn trigger_with_progress(
    label: &'static str,
    description: &'static str,
    percent: f64,
) -> Vec<Node> {
    let p = Progress::new(0.0, 100.0, Some(percent), Orientation::Horizontal);
    vec![div(
        vec![("class", "blocks-feature-tabs-panel-trigger-progress")],
        vec![
            styled_text::text(&TextProps::default(), vec![], vec![text(label)]),
            styled_text::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
            progress::root(
                &p,
                &ProgressProps {
                    size: Size::Sm,
                    ..ProgressProps::default()
                },
                None,
                vec![],
                vec![p.track(vec![], vec![progress::range(&p, vec![])])],
            ),
        ],
    )]
}

/// 形 F（対応表 ID R0104）: [`PANELS`] の先頭 3 件を使い、各トリガーへ
/// 架空の固定進捗値を持たせる（[`trigger_with_progress`]）。
fn variant_progress_trigger() -> Node {
    const PERCENTS: [f64; 3] = [100.0, 55.0, 20.0];
    let items: Vec<TabItem<'static>> = PANELS[0..3]
        .iter()
        .zip(PERCENTS)
        .map(|(data, percent)| TabItem {
            value: data.value,
            trigger: trigger_with_progress(data.label, "進捗の目安", percent),
            content: panel_row(data),
            disabled: false,
        })
        .collect();
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        vec![
            section_header(
                "進捗を添えて紹介",
                "トリガーに説明と進捗バーを添える",
                "各タブのトリガーに短い説明と進捗の目安を添えます。自動切替は行いません。",
            ),
            tabs_panel(
                "blocks-feature-tabs-panel-progress",
                TabsVariant::Line,
                PANELS[0].value,
                items,
            ),
            tab_preview(
                format!("「{}」タブを選択した場合のプレビュー", PANELS[1].label),
                panel_row(&PANELS[1]),
            ),
            tab_preview(
                format!("「{}」タブを選択した場合のプレビュー", PANELS[2].label),
                panel_row(&PANELS[2]),
            ),
        ],
    )
}

/// `feature-tabs-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。基準形（R1158）に続けて残り 4 形（R0478/R0479/R0481/R0104）を
/// 縦に並べる（モジュール doc「#2772 と #2773 の分担」節）。各形とも無 JS
/// 対応のため、実物の `tabs::tabs` は選択中タブ分のみで残りは非対話
/// プレビュー（各 `variant_*` 参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-tabs-panel-layout")],
        vec![
            variant_label("下線タブ + テキスト/画像パネル（対応表 ID R1158 基準形）"),
            variant_basic(),
            variant_label("ピル型タブ + 単一パネル（対応表 ID R0478）"),
            variant_pill(),
            variant_label("パネル内で左右を入れ替える複数行（対応表 ID R0479）"),
            variant_alternating(),
            variant_label("中央見出し + カードグリッド（対応表 ID R0481）"),
            variant_card_grid(),
            variant_label("説明 + 進捗バー付きトリガー（対応表 ID R0104）"),
            variant_progress_trigger(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R1158 基準形。集約元 R0104/R0478/R0479/R0481。出典の
固有名・ファイル名は記載しません）からの意図的な差分は次のとおりです。

- 参照元の配色・装飾・アイコンは持ち込まず、既存の pre-styled-ui 部品の
  既定スタイルのみで構成しました。アイコン（形 E のカードグリッド）は
  自作の幾何線画のみを使用します。
- 見出しは 1 段下げて `h3`（パネル内・カードは `h4`）にしました（ページ側が
  `## Demo` として `h2` を出すため）。
- 画像は `dummy_assets` のプレースホルダー + `alt=""`（装飾扱い）を使用し、
  実在のブランド・人物・企業とは無関係の架空データです。
- 無 JS 制約に従い、各形とも実物の `tabs` インスタンスは選択中のタブ 1 個
  だけを描画し、残りのタブは `tabs` を複製せず見出しキャプション付きの
  非対話表示（トリガーボタン・`role="tab"`・`tabindex` を持たない）として
  併記しました。当初はパネル数ぶんの `tabs` インスタンスを選択状態違いで
  縦に並べていましたが（`pricing_tiers_morph`/`sidebar_07` と同型の対処）、
  この構成は操作可能に見えて実際には切り替わらないトリガーボタンを複数
  インスタンス分反復して出しており、UI のアクセシビリティ契約に反すると
  の指摘を受けて是正しました（基準形、#2772）。
- **形 C（R0478）**: 参照元の「単一パネル」の意図を、既存 2 列（テキスト +
  画像）の基準形とは別クラスの単一カラム（見出し + 本文のみ、画像なし）で
  表現しました。パネル数は 2 件に絞り、重複が過大にならないようにしました。
- **形 D（R0479）**: 「パネル内でテキストと画像の行を交互配置」する参照元の
  意図を、`feature-alternating-rows` block と同型の発想でパネル内スケールへ
  適用しました。行の左右入れ替えは lg ブレークポイントの `grid-column`
  指定のみで行い、DOM 順（テキスト → 画像で固定）は変えません（読み上げ
  順を変えないため）。
- **形 E（R0481）**: 「中央見出し + タブ + カードグリッド」を、見出しの
  `data-align="center"` フックと、パネル内のカードグリッド（`card`/`icon`）
  で表現しました。カードは 1 タブあたり 3 枚に固定し、末尾に押しても何も
  起きない静的 CTA ボタン（`disabled: true`、`feature_accordion_image::
  category_button` と同型の判断で dead control 化を防ぐ）を添えました。
- **形 F（R0104）**: 「トリガーに説明 + 進捗バーを添える」を、`Progress`
  コンポーネントによる決定的な固定進捗値（架空）で表現しました。docs
  サイトは元々無 JS のため、「自動切替を持たない」という参照元の制約は
  構造的に満たされます。

関連部品: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Tabs](../themes/tabs.md) /
[Image](../themes/image.md) / [Card](../themes/card.md) /
[Icon](../themes/icon.md) / [Button](../themes/button.md) /
[Progress](../themes/progress.md)
