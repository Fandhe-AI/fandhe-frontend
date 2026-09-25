# feature-tabs-panel

`badge` / `heading` / `text` / `image` / `card` / `icon` / `button` /
`progress` の 8 部品を合成した、タブ切り替え feature セクションの見た目を
再現する合成例です。5 つの形を縦に並べます。実物の `tabs` コンポーネントは
使わず、タブ列は見た目だけを模した静的表示のため使用部品には含めません
（下記の説明を参照）。

- **基準形（対応表 ID R1158）**: 見出しの下に下線タブ風の見た目を並べ、
  選択中タブの本文としてテキストと画像を表示します。
- **形 C（対応表 ID R0478）**: ピル型タブの見た目 + 画像を持たない単一
  カラムのパネル。
- **形 D（対応表 ID R0479）**: 1 パネルの中に複数のフィーチャー行を積み、
  行ごとに画像とテキストの左右を入れ替えます。
- **形 E（対応表 ID R0481）**: 中央寄せの見出し + タブ風の見た目 + パネル内
  カードグリッド。末尾に「すべての機能を見る」の静的 CTA を添えます。
- **形 F（対応表 ID R0104）**: トリガーに短い説明文と進捗バーを添えます。
  自動切替は行いません。

docs サイトは JS ハイドレーションを行わないため、本 block は実物の `tabs`
コンポーネントを一切使いません。タブ列は `role`/`tabindex`/`<button>` を
持たない非対話表示（クリックしても何も起きない）でタブの見た目だけを
再現し、選択中タブの本文を直接描画します。残りのタブの内容も、見出し
キャプション付きの非対話表示として静的に併記するため、すべてのパネル本文
が常に可視のまま静的 HTML に現れます（実際に切り替わるのではなく、全形の
全パネルが同時に表示され続けます）。

文言・カードの見出しと説明・進捗値はすべて架空のもので、データ取得・送信は
行わない静的な表示例です。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, el_owned, text, Node};
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
// `Orientation` は `progress` モジュール（`Progress::new` の引数型）から取り込む。
// 旧実装は `tabs::Orientation` を使っていたが、本 block は実物の `tabs::tabs`
// を一切使わない（下記「無 JS での扱い」節参照）ため `tabs` モジュール自体を
// import しない。
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

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

/// [`panel_alternating_rows`] の複数行を 1 グループへ束ねる
/// （`.blocks-feature-tabs-panel-rows`）。選択中セット（`.variant` 直下）と
/// プレビュー側セット（[`tab_preview`] でラップ）の双方が同じグループ
/// class を経由することで、行間の間隔（[`LAYOUT_CSS`] の `gap`）が
/// セット間で食い違わないようにする（Bugbot 指摘の是正: 束ねずに `.row` を
/// 直接並べると、選択中セットは `.variant` の `gap` + `.row` 自身の
/// `padding-top` が二重に積み上がる一方、プレビュー側は `.preview` の
/// `gap` のみで `padding-top` が打ち消されており、同じ見た目であるべき
/// 2 セットの行間が大きく異なっていた）。
fn rows_group(rows: Vec<Node>) -> Node {
    div(vec![("class", "blocks-feature-tabs-panel-rows")], rows)
}

/// [`PANELS`] から [`static_tab_list`] の `(value, trigger)` 組を組み立てる
/// （#2773 が variant を変えつつ再利用する共通ヘルパ）。
fn panel_tab_labels() -> Vec<(&'static str, Vec<Node>)> {
    PANELS
        .iter()
        .map(|data| (data.value, vec![text(data.label)]))
        .collect()
}

/// 実物の `tabs::tabs` を一切使わない非対話タブ列（モジュール doc「無 JS
/// での扱い」節、Codex P1 是正）。block 固有 class
/// （`.blocks-feature-tabs-panel-tablist`/`-tab`）のみを `div` に与え、
/// [`LAYOUT_CSS`] 側で見た目を独自に再現する。`data-scope="tabs"`/
/// `data-part="list"`/`"trigger"` 等 pre-styled-ui の tabs recipe が使う
/// セレクタとは意図的に一致しない属性構造にする（モジュール doc「CSS
/// フックの選び方」節、Bugbot 再指摘の是正: recipe とセレクタを共有すると
/// `:hover:not([data-disabled])` 規則が子孫〔形 F の進捗バー等〕を経由して
/// も当たり続け、`pointer-events: none` による個別の打ち消しでは防ぎ
/// きれなかった）。`role`/`tabindex`/`<button>` も一切持たないため操作
/// 可能に見えない。`id_prefix` は呼び出し側が
/// `blocks-feature-tabs-panel-<接尾辞>` の形で完全指定する（モジュール doc
/// 「id 規約」節）。
///
/// `hide_from_assistive_tech` が `true` の trigger には `aria-hidden="true"`
/// を付与し、装飾要素として支援技術のツリーから除外する（`tabs::tabs` の
/// `indicator` パーツと同じ判断。ラベルのみを持つ基準形・形 C・形 D・形 E）。
/// `false` の trigger（形 F、[`trigger_with_progress`]）は `role="progressbar"`
/// と進捗値を含む実情報を持つため `aria-hidden` を付けない（Codex P1 是正:
/// 全 trigger 一律 `aria-hidden` にすると進捗情報が支援技術から読めなくなる。
/// `role`/`tabindex` を持たない div のままなので `aria-hidden` を外しても
/// 操作可能に見えるようにはならない）。
fn static_tab_list(
    id_prefix: &'static str,
    selected: &'static str,
    items: Vec<(&'static str, Vec<Node>)>,
    hide_from_assistive_tech: bool,
    pill: bool,
) -> Node {
    let mut list_attrs = vec![
        (
            "class".to_string(),
            "blocks-feature-tabs-panel-tablist".to_string(),
        ),
        ("id".to_string(), format!("{id_prefix}-list")),
    ];
    if pill {
        // 形 C（対応表 ID R0478、ピル型）専用フック。実物の `tabs::tabs` を
        // 使わないためレシピの `TabsVariant::Enclosed`（root への variant
        // class 付与、`fandhe_frontend_pre_styled_ui::tabs` rustdoc「variant」
        // 節参照）を経由できず、pre-styled-ui の Enclosed 実装と同じトークン
        // （`--fandhe-color-bg-muted`/`--fandhe-radius-md`/`--fandhe-space-1`/
        // `--fandhe-color-bg`/`--fandhe-shadow-sm`）を [`LAYOUT_CSS`] 側で
        // 直接再現する（Codex/Bugbot P2 是正: 形 C が下線型のまま変化
        // していなかった不具合）。選択中タブは背景色 + `box-shadow` のみで
        // 表現しているが、Windows 強制配色モード（`forced-colors: active`）
        // は色をシステム色へ強制し `box-shadow` も `none` へ丸めるため、
        // このままでは選択中/非選択中が判別できなくなる（Codex P1 是正）。
        // `tabs::stylesheet` の Enclosed forced-colors 対応（`crate::tabs`
        // rustdoc「forced-colors 対応」節）と同じ
        // `border: 1px solid CanvasText` を [`LAYOUT_CSS`] 側へ直接追記して
        // 選択状態を境界線で補強する。
        list_attrs.push((
            "data-blocks-feature-tabs-panel-pill".to_string(),
            String::new(),
        ));
    }
    el_owned(
        "div",
        list_attrs,
        items
            .into_iter()
            .map(|(value, trigger)| {
                let state = if value == selected {
                    "active"
                } else {
                    "inactive"
                };
                let mut attrs = vec![
                    (
                        "class".to_string(),
                        "blocks-feature-tabs-panel-tab".to_string(),
                    ),
                    ("data-state".to_string(), state.to_string()),
                ];
                if hide_from_assistive_tech {
                    attrs.push(("aria-hidden".to_string(), "true".to_string()));
                }
                el_owned("div", attrs, trigger)
            })
            .collect(),
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

/// 基準形（R1158）: 見出し + タブ列 + テキスト/画像パネル。docs サイトは
/// JS ハイドレーションを行わないため（モジュール doc「無 JS での扱い」
/// 節）、[`static_tab_list`]（[`PANELS`] 先頭の `design` を選択済みとする
/// 非対話タブ列）+ その本文（[`panel_row`]）を描画したあと、残り 3 パネルは
/// [`panel_state_preview`] による非対話プレビューとして併記する。これにより
/// 4 パネルすべての本文が常に可視のまま静的 HTML に現れる。
fn variant_basic() -> Node {
    let mut children = vec![
        section_header(
            "機能紹介",
            "タブで切り替える機能セクション",
            "見出しの下にタブを並べ、選択中タブの内容を表示し、残り 3 件は切り替え例として併記します。",
        ),
        static_tab_list(
            "blocks-feature-tabs-panel-basic",
            PANELS[0].value,
            panel_tab_labels(),
            true,
            false,
        ),
    ];
    children.extend(panel_row(&PANELS[0]));
    for panel in &PANELS[1..] {
        children.push(panel_state_preview(panel));
    }
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
    )
}

/// 形 C（対応表 ID R0478）: ピル型見た目のタブ列 + 単一カラムのパネル
/// （[`panel_single`]、画像を持たない）。[`PANELS`] の先頭 2 件のみ使う
/// （重複が過大にならないよう絞る、モジュール doc「#2772 と #2773 の分担」
/// 節）。
fn variant_pill() -> Node {
    let mut children = vec![
        section_header(
            "コンパクト表示",
            "ピル型タブで切り替える単一パネル",
            "画像を持たない単一カラムのパネルを、ピル型のタブで切り替えます。",
        ),
        static_tab_list(
            "blocks-feature-tabs-panel-pill",
            PANELS[0].value,
            vec![
                (PANELS[0].value, vec![text(PANELS[0].label)]),
                (PANELS[1].value, vec![text(PANELS[1].label)]),
            ],
            true,
            true,
        ),
    ];
    children.extend(panel_single(&PANELS[0]));
    children.push(tab_preview(
        format!("「{}」タブを選択した場合のプレビュー", PANELS[1].label),
        panel_single(&PANELS[1]),
    ));
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
    )
}

/// 形 D（対応表 ID R0479）: 1 タブの中に複数のフィーチャー行を積み、行
/// ごとに左右を入れ替える（[`panel_alternating_rows`]）。[`PANELS`] を
/// 2 件ずつ 2 組に分け、タブ切り替えで組を入れ替える。
fn variant_alternating() -> Node {
    let mut children = vec![
        section_header(
            "詳しい紹介",
            "パネル内で左右を入れ替える複数行",
            "1 つのパネルに複数の項目を積み、行ごとに画像とテキストの左右を入れ替えます。",
        ),
        static_tab_list(
            "blocks-feature-tabs-panel-alternating",
            "set-a",
            vec![
                ("set-a", vec![text("セット A")]),
                ("set-b", vec![text("セット B")]),
            ],
            true,
            false,
        ),
    ];
    children.push(rows_group(panel_alternating_rows(&PANELS[0..2])));
    children.push(tab_preview(
        "「セット B」タブを選択した場合のプレビュー".to_string(),
        vec![rows_group(panel_alternating_rows(&PANELS[2..4]))],
    ));
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
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

/// 形 E（対応表 ID R0481）: 中央寄せ見出し + タブ列 + パネル内カードグリッド。
fn variant_card_grid() -> Node {
    let mut children = vec![
        section_header_aligned(
            "多角的に紹介",
            "カードグリッドで機能をまとめる",
            "タブを切り替えると、紹介するカードの組が変わります。",
            true,
        ),
        static_tab_list(
            "blocks-feature-tabs-panel-cards",
            "features",
            vec![
                ("features", vec![text("特長")]),
                ("cases", vec![text("導入事例")]),
            ],
            true,
            false,
        ),
    ];
    children.extend(panel_card_grid(&CARDS_FEATURES));
    children.push(tab_preview(
        "「導入事例」タブを選択した場合のプレビュー".to_string(),
        panel_card_grid(&CARDS_CASES),
    ));
    children.push(grid_cta());
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
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
                // `aria-labelledby` の自動配線は headless/styled いずれの層の
                // 責務でもない（`fandhe_frontend_pre_styled_ui::progress`
                // rustdoc「イシュー #2049」節「(4)」参照）ため、呼び出し側で
                // `aria-label` を明示する（Codex P1 是正: 無地の `role=
                // "progressbar"` だけでは支援技術から進捗の意味が読めない）。
                vec![("aria-label", label)],
                vec![p.track(vec![], vec![progress::range(&p, vec![])])],
            ),
        ],
    )]
}

/// 形 F（対応表 ID R0104）: [`PANELS`] の先頭 3 件を使い、各トリガーへ
/// 架空の固定進捗値を持たせる（[`trigger_with_progress`]）。
fn variant_progress_trigger() -> Node {
    const PERCENTS: [f64; 3] = [100.0, 55.0, 20.0];
    let labels: Vec<(&'static str, Vec<Node>)> = PANELS[0..3]
        .iter()
        .zip(PERCENTS)
        .map(|(data, percent)| {
            (
                data.value,
                trigger_with_progress(data.label, "進捗の目安", percent),
            )
        })
        .collect();
    let mut children = vec![
        section_header(
            "進捗を添えて紹介",
            "トリガーに説明と進捗バーを添える",
            "各タブのトリガーに短い説明と進捗の目安を添えます。自動切替は行いません。",
        ),
        static_tab_list(
            "blocks-feature-tabs-panel-progress",
            PANELS[0].value,
            labels,
            false,
            false,
        ),
    ];
    children.extend(panel_row(&PANELS[0]));
    children.push(tab_preview(
        format!("「{}」タブを選択した場合のプレビュー", PANELS[1].label),
        panel_row(&PANELS[1]),
    ));
    children.push(tab_preview(
        format!("「{}」タブを選択した場合のプレビュー", PANELS[2].label),
        panel_row(&PANELS[2]),
    ));
    div(
        vec![("class", "blocks-feature-tabs-panel-variant")],
        children,
    )
}

/// `feature-tabs-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。基準形（R1158）に続けて残り 4 形（R0478/R0479/R0481/R0104）を
/// 縦に並べる（モジュール doc「#2772 と #2773 の分担」節）。各形とも無 JS
/// 対応のため、[`static_tab_list`] による非対話タブ列 + 選択中タブの本文を
/// 描画したあと、残りは非対話プレビュー（各 `variant_*` 参照）。
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
- 無 JS 制約に従い、実物の `tabs` コンポーネントは一切使わず、各形とも
  選択中タブの見た目だけを静的なタブ風表示（`role`/`tabindex`/`<button>`
  を持たない `div`）で再現し、その本文を直接描画します。残りのタブも
  同じ静的タブ風表示 + 見出しキャプション付きの非対話表示として併記し、
  全パネルの本文が常に可視のまま静的 HTML に現れるようにしました。当初は
  パネル数ぶんの `tabs` インスタンスを選択状態違いで縦に並べていましたが
  （`pricing_tiers_morph`/`sidebar_07` と同型の対処）、この構成は操作可能に
  見えて実際には切り替わらないトリガーボタンを複数インスタンス分反復して
  出しており、UI のアクセシビリティ契約に反するとの指摘を受けて是正しました
  （基準形、#2772）。さらに、当初はタブ風表示に pre-styled-ui の tabs
  recipe と同じ `data-scope="tabs"`/`data-part="list"`/`"trigger"` を
  付与し LAYOUT_CSS 側で recipe のセレクタを再利用していましたが、これだと
  recipe の `:hover` 規則がタブ風表示やその子孫（形 F の説明文・進捗バー等）
  にも当たってマウスホバーで背景・文字色が変化し操作可能に見えてしまう
  不具合が見つかりました。`pointer-events: none` による打ち消しを一度試み
  ましたが子孫経由の hover を防ぎきれず、recipe のセレクタと一切一致しない
  block 固有 class（`.blocks-feature-tabs-panel-tablist`/`-tab`）だけで
  見た目を再現する構成へ切り替えました。ピル型の選択状態は背景色 +
  `box-shadow` のみで表しているため Windows 強制配色モードでも判別できる
  よう `forced-colors: active` 用の境界線を追加しています（#2773 残作業）。
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
[Text](../themes/text.md) / [Image](../themes/image.md) /
[Card](../themes/card.md) / [Icon](../themes/icon.md) /
[Button](../themes/button.md) / [Progress](../themes/progress.md)
