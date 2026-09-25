//! `feature-tabs-panel` block（親イシュー #2771「Blocks 目的別パーツ拡充
//! ツリー」配下、区分 marketing / カテゴリ Feature に属する。規模 L のため
//! 前半 #2772（骨格・R1158 基準形）と後半 #2773（本コミット。残り形の追加・
//! 状態表示・原稿仕上げ）に分割済み。タブで切り替える feature セクションの
//! 合成例で、対応表の主参照 ID は R1158（基準形）であり、集約元として
//! R0104/R0478/R0479/R0481 を持つ（取得手段・ファイル名・内部識別子は
//! 記載しない）。
//!
//! # #2772 と #2773 の分担
//!
//! #2772 は骨格（[`demo`] のレイアウト root）と基準形 R1158（[`variant_basic`]）
//! のみを実装した。本イシュー（#2773）はそれを [`tabs_panel`] 経由で
//! 再利用し、残り 4 形を追加する:
//!
//! - **形 C（対応表 ID R0478）**: [`variant_pill`]。ピル型見た目のタブ列
//!   + 単一カラムのパネル（[`panel_single`]）。
//! - **形 D（対応表 ID R0479）**: [`variant_alternating`]。1 タブ内に複数の
//!   フィーチャー行を持ち、行ごとに左右を入れ替える
//!   （[`panel_alternating_rows`]）。
//! - **形 E（対応表 ID R0481）**: [`variant_card_grid`]。中央寄せ見出し +
//!   タブ + パネル内カードグリッド（[`panel_card_grid`]）。
//! - **形 F（対応表 ID R0104）**: [`variant_progress_trigger`]。トリガーに
//!   説明文 + 進捗バーを添える（[`trigger_with_progress`]）。docs サイトは
//!   元々無 JS のため「自動切替を持たない」制約は構造的に満たされる。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `tabs` / `image` / `card` / `icon` /
//! `button` / `progress` の 9 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。`card`/`icon` は形 E のカードグリッド、
//! `button` は形 E 末尾の CTA（`disabled: true` の「押しても何も起きない」
//! 静的表示、`feature_accordion_image::category_button` と同型の判断）、
//! `progress` は形 F のトリガーで使う。
//!
//! # 無 JS での扱い（実物の `tabs::tabs` は一切使わない）
//!
//! docs サイトは JS ハイドレーションを行わないため、`tabs::tabs` の
//! trigger（`role="tab"`/`type="button"`/`tabindex`）はクリック・キーボード
//! 操作をしても実際には一切切り替わらないにもかかわらず操作可能に見える。
//! 当初は [`PANELS`] の要素数分の `tabs::tabs` インスタンスを選択状態違いで
//! 縦に並べていた（計 16 個の操作可能に見えるトリガー）が、これを 1 形
//! 1 インスタンスへ絞ってもなお 5 形合計 13 個の同種トリガーが残り、UI の
//! アクセシビリティ契約に反すると判定された（Codex P1 指摘、`docs/design/
//! docs-site-blocks-section.md` §19 参照）。無 JS の docs サイトで実際に
//! 切り替えられるようにする経路はないため、本 block は実物の `tabs::tabs`
//! を**一切使わない**。タブ列は [`static_tab_list`]（`data-scope="tabs"`/
//! `data-part="list"`/`"trigger"` の `div` のみで [`LAYOUT_CSS`] の見た目を
//! 再現し、`role`/`tabindex`/`<button>` を一切持たない非対話表示）で視覚上
//! だけ模し、選択中パネルの本文は直接（[`panel_row`] 等）描画する。残りの
//! パネルは見出しキャプション付きの非対話表示（[`panel_state_preview`]/
//! [`tab_preview`]）として併記し、全パネル本文が常に可視のまま静的 HTML に
//! 現れるようにする。
//!
//! # id 規約
//!
//! [`static_tab_list`] は id 規約に合わせ `blocks-feature-tabs-panel-<形の
//! 接尾辞>-list` を持つ（形ごとに `-basic`/`-pill`/`-alternating`/`-cards`/
//! `-progress` を使い分けて衝突を構造的に避ける）。実物の `tabs::tabs` を
//! 使わないため `aria-controls`/`aria-labelledby` による相互参照は発生せず、
//! `{id}-trigger-{value}`/`{id}-content-{value}` の機械生成もない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約と `static_tab_list` の制約）
//!
//! `badge::badge`/`heading::heading`/`styled_text::text`/`image::image` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-tabs-panel-*` 属性で渡す。[`static_tab_list`] は
//! `tabs::tabs` と同じ `data-scope="tabs"`/`data-part="list"`/`"trigger"`
//! 属性を持つが呼び出し側から attrs を追加で受け取らないため、パーツへの
//! スタイルは [`LAYOUT_CSS`] 側でレイアウト root からの子孫セレクタ
//! （`[data-scope="tabs"][data-part="..."]`）として当てる:
//!
//! - `[data-part="list"]` に `overflow-x: auto` を与え、狭い幅でタブ列が
//!   はみ出してページ全体が横スクロールするのを list 自身の横スクロールへ
//!   閉じ込める（モバイルでのタブ列横スクロール）。`padding-bottom: 1px` は
//!   trigger の下線用 `margin-bottom: -1px` がスクロール領域の境界で
//!   クリップされる分を吸収する。
//! - `[data-part="trigger"]` の `margin-bottom` を pre-styled-ui 既定の
//!   `-1px` から `-2px` へ上書きする（Bugbot Low 指摘の是正）。`list` の
//!   `padding-bottom: 1px`（直前の箇条書き）は `list` の下端（＝下線を
//!   重ねたい `border-bottom: 1px` の位置）を 1px 押し下げるため、
//!   `trigger` 側の重ね量を素の `-1px` のままにすると選択中トリガーの
//!   2px 下線が `list` の 1px 罫線と重ならず 3px の二重線に見えてしまう。
//!   `padding-bottom` が押し下げた 1px 分を追加で相殺し、選択中トリガーの
//!   下線を `list` の罫線へ正しく重ねる。[`static_tab_list`] は
//!   `tabindex`/フォーカス自体を持たないため `:focus-visible` の補正は
//!   不要（`tabs::tabs` を使っていた旧実装からの削除点）。同じ規則で
//!   `cursor: default` を上書きし、pre-styled-ui recipe 既定の
//!   `cursor: pointer`（属性セレクタのみで tag を問わないため、`button`
//!   を使わない [`static_tab_list`] にもそのまま当たる）が視覚的な
//!   クリック可能感を残さないようにする。
//!
//! レイアウト root の class（`blocks-feature-tabs-panel-layout`）は
//! [`Block::demo_class`]（`blocks-feature-tabs-panel`）と意図的に別名にする
//! （既存 block と同じ Bugbot 教訓の回避）。
//!
//! # レスポンシブ（モバイルでのタブ列横スクロール・1 列↔2 列切り替え）
//!
//! `.blocks-feature-tabs-panel-row` は既定で flex column（テキストの下に
//! 画像）。`@media (min-width: 64rem)`（他の Marketing / Feature block と
//! 同じ [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`] のリテラル
//! 値。テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! ため直書きする）では 12 列 grid にし、テキストを 5 列・画像を 7 列へ
//! 割り当てる。DOM 順は常にテキスト → 画像で固定し（読み上げ順を変えない）、
//! 配置は `grid-column` 指定だけで行う。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`crate::blocks::dummy_assets`] のビルド時生成
//! プレースホルダー SVG のみを使い、`alt=""`（装飾扱い）で出力する
//! （`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// [`PANELS`] から [`static_tab_list`] の `(value, trigger)` 組を組み立てる
/// （#2773 が variant を変えつつ再利用する共通ヘルパ）。
fn panel_tab_labels() -> Vec<(&'static str, Vec<Node>)> {
    PANELS
        .iter()
        .map(|data| (data.value, vec![text(data.label)]))
        .collect()
}

/// 実物の `tabs::tabs` を一切使わない非対話タブ列（モジュール doc「無 JS
/// での扱い」節、Codex P1 是正）。`data-scope="tabs"`/`data-part="list"`/
/// `"trigger"` を `tabs::tabs` と同じ属性値で `div` のみに与え、
/// [`LAYOUT_CSS`] の `[data-scope="tabs"][data-part="..."]` セレクタによる
/// 見た目をそのまま再利用しつつ、`role`/`tabindex`/`<button>` は一切持たない
/// ため操作可能に見えない。`aria-hidden="true"` を各 trigger へ付与し、
/// 装飾要素として支援技術のツリーから除外する（`tabs::tabs` の
/// `indicator` パーツと同じ判断）。`id_prefix` は呼び出し側が
/// `blocks-feature-tabs-panel-<接尾辞>` の形で完全指定する（モジュール doc
/// 「id 規約」節）。
fn static_tab_list(
    id_prefix: &'static str,
    selected: &'static str,
    items: Vec<(&'static str, Vec<Node>)>,
) -> Node {
    el_owned(
        "div",
        vec![
            ("data-scope".to_string(), "tabs".to_string()),
            ("data-part".to_string(), "list".to_string()),
            ("id".to_string(), format!("{id_prefix}-list")),
        ],
        items
            .into_iter()
            .map(|(value, trigger)| {
                let state = if value == selected {
                    "active"
                } else {
                    "inactive"
                };
                el_owned(
                    "div",
                    vec![
                        ("data-scope".to_string(), "tabs".to_string()),
                        ("data-part".to_string(), "trigger".to_string()),
                        ("data-state".to_string(), state.to_string()),
                        ("aria-hidden".to_string(), "true".to_string()),
                    ],
                    trigger,
                )
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
            "見出しの下にタブを並べ、選んだタブの内容だけを表示します。",
        ),
        static_tab_list(
            "blocks-feature-tabs-panel-basic",
            PANELS[0].value,
            panel_tab_labels(),
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
        ),
    ];
    children.extend(panel_alternating_rows(&PANELS[0..2]));
    children.push(tab_preview(
        "「セット B」タブを選択した場合のプレビュー".to_string(),
        panel_alternating_rows(&PANELS[2..4]),
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-tabs-panel/",
    title: "feature-tabs-panel",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_tabs_panel.rs",
    demo_class: "blocks-feature-tabs-panel",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Progress",
            path: "/themes/progress/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_tabs_panel` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。素の `div` へのフックは
/// `.blocks-feature-tabs-panel-*` クラスセレクタ、`tabs` パーツへのフックは
/// `[data-scope="tabs"][data-part="..."]` の子孫セレクタで行い、他 block や
/// 部品の素のセレクタへ影響させない（モジュール doc「CSS フックの選び方」
/// 節参照）。
const LAYOUT_CSS: &str = "\
.blocks-feature-tabs-panel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-tabs-panel-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-tabs-panel-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n  max-width: 40rem;\n}\n\
.blocks-feature-tabs-panel-layout [data-scope=\"tabs\"][data-part=\"list\"] {\n  overflow-x: auto;\n  overflow-y: hidden;\n  padding-bottom: 1px;\n}\n\
.blocks-feature-tabs-panel-layout [data-scope=\"tabs\"][data-part=\"trigger\"] {\n  margin-bottom: -2px;\n  cursor: default;\n}\n\
.blocks-feature-tabs-panel-row {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding-top: var(--fandhe-space-6);\n}\n\
.blocks-feature-tabs-panel-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-feature-tabs-panel-preview {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feature-tabs-panel-preview .blocks-feature-tabs-panel-row,\n\
.blocks-feature-tabs-panel-preview .blocks-feature-tabs-panel-single,\n\
.blocks-feature-tabs-panel-preview .blocks-feature-tabs-panel-card-grid {\n  padding-top: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-tabs-panel-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-feature-tabs-panel-header[data-align=\"center\"] {\n  align-items: center;\n  text-align: center;\n  max-width: 40rem;\n  margin-left: auto;\n  margin-right: auto;\n}\n\
.blocks-feature-tabs-panel-single {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding-top: var(--fandhe-space-6);\n  max-width: 36rem;\n}\n\
.blocks-feature-tabs-panel-card-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n  padding-top: var(--fandhe-space-6);\n}\n\
.blocks-feature-tabs-panel-card-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feature-tabs-panel-cta {\n  display: flex;\n  justify-content: center;\n}\n\
.blocks-feature-tabs-panel-trigger-progress {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 8rem;\n  text-align: left;\n  white-space: normal;\n}\n\
.blocks-feature-tabs-panel-trigger-progress [data-scope=\"progress\"][data-part=\"root\"] {\n  width: 100%;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-tabs-panel-row {\n    display: grid;\n    grid-template-columns: repeat(12, minmax(0, 1fr));\n    column-gap: var(--fandhe-space-10);\n    align-items: center;\n  }\n  \
.blocks-feature-tabs-panel-row > .blocks-feature-tabs-panel-copy {\n    grid-column: 1 / span 5;\n    grid-row: 1;\n  }\n  \
.blocks-feature-tabs-panel-row > .blocks-feature-tabs-panel-media {\n    grid-column: 7 / span 6;\n    grid-row: 1;\n  }\n  \
.blocks-feature-tabs-panel-row[data-blocks-feature-tabs-panel-reverse] > .blocks-feature-tabs-panel-copy {\n    grid-column: 8 / span 5;\n  }\n  \
.blocks-feature-tabs-panel-row[data-blocks-feature-tabs-panel-reverse] > .blocks-feature-tabs-panel-media {\n    grid-column: 1 / span 6;\n  }\n  \
.blocks-feature-tabs-panel-card-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 9 部品を出力すること、非対話制約（`<form>` 不在・
    /// `data:` URI 不在・`href="#"` 不在）を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"tabs\"",
            "data-scope=\"image\"",
            "data-scope=\"card\"",
            "data-scope=\"icon\"",
            "data-scope=\"button\"",
            "data-scope=\"progress\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("type=\"button\""));
        for absent in ["<form", "src=\"data:", "href=\"#\""] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// 5 形（基準形/R0478/R0479/R0481/R0104）それぞれの [`super::
    /// static_tab_list`] が選択中ラベルにちょうど 1 件
    /// `data-state="active"` を与え、残りは `data-state="inactive"` である
    /// こと。trigger 総数は 4（基準形）+2（R0478）+2（R0479）+2（R0481）+3
    /// （R0104）= 13、選択中はインスタンスごとに 1 件で計 5 件、残り 8 件が
    /// inactive。
    #[test]
    fn each_static_tab_list_has_exactly_one_active_label() {
        let html = render(&demo());
        assert_eq!(html.matches("data-part=\"trigger\"").count(), 13);
        assert_eq!(html.matches("data-state=\"active\"").count(), 5);
        assert_eq!(html.matches("data-state=\"inactive\"").count(), 8);
    }

    /// `tabs::tabs` を一切使わないため、`role="tab"`/`role="tabpanel"` が
    /// demo 出力に一切現れないこと（Codex P1 是正の回帰テスト。モジュール
    /// doc「無 JS での扱い」節）。`<button` は形 E 末尾の CTA
    /// （[`super::grid_cta`]）1 個のみ。各 trigger は `aria-hidden="true"`
    /// を持ち、支援技術のツリーから除外される装飾要素として描画される。
    #[test]
    fn no_tabs_instance_is_interactive_and_previews_are_not_either() {
        let html = render(&demo());
        assert_eq!(html.matches("role=\"tab\"").count(), 0);
        assert_eq!(html.matches("role=\"tabpanel\"").count(), 0);
        assert_eq!(html.matches("tabindex").count(), 0);
        // trigger は `data-state` の直後に `aria-hidden="true"` を持つ
        // （[`super::static_tab_list`] の属性順）。image/icon 等の他部品も
        // 装飾目的で `aria-hidden="true"` を出すため、この部分文字列に
        // 絞って trigger 由来分のみを数える。
        assert_eq!(
            html.matches("data-state=\"active\" aria-hidden=\"true\"")
                .count(),
            5
        );
        assert_eq!(
            html.matches("data-state=\"inactive\" aria-hidden=\"true\"")
                .count(),
            8
        );
        // プレビュー件数: 基準形 3 + R0478 1 + R0479 1 + R0481 1 + R0104 2 = 8。
        assert_eq!(
            html.matches("class=\"blocks-feature-tabs-panel-preview\"")
                .count(),
            8
        );
        assert_eq!(html.matches("<button").count(), 1);
    }

    /// 全パネルの本文見出しが、非対話プレビュー・選択中パネルのいずれかの
    /// 経路で常に静的 HTML に現れること（codex P1 指摘の回帰テスト: 選択中
    /// 以外のパネル本文が静的ページから一切読めなくなっていた不具合の
    /// 再発防止）。
    #[test]
    fn every_panel_title_is_always_visible() {
        let html = render(&demo());
        for panel in &super::PANELS {
            assert!(
                html.contains(panel.title),
                "expected panel title {:?} to appear in demo output",
                panel.title
            );
        }
    }

    /// 基準形の [`super::static_tab_list`] の id が id 規約
    /// （`blocks-feature-tabs-panel-basic-list`）に従うこと（重複 id 検知
    /// テストへの対応、モジュール doc「id 規約」節）。
    #[test]
    fn static_tab_list_id_uses_the_expected_prefix() {
        let html = render(&demo());
        assert!(html.contains("id=\"blocks-feature-tabs-panel-basic-list\""));
    }

    /// [`LAYOUT_CSS`] が想定するタブ列の横スクロール・下線の重ね合わせ
    /// 補正（Bugbot Low 是正）・lg ブレークポイント・tabs パーツへの子孫
    /// セレクタを持つこと。
    #[test]
    fn layout_css_declares_scrollable_tablist_and_lg_grid() {
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
        assert!(LAYOUT_CSS.contains("margin-bottom: -2px"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("[data-scope=\"tabs\"][data-part=\"list\"]"));
        assert!(LAYOUT_CSS.contains("repeat(12, minmax(0, 1fr))"));
    }

    /// [`LAYOUT_CSS`] が #2773 で追加した新規セレクタ（形 D の左右入れ替え・
    /// 形 E のカードグリッド・形 E の中央寄せ見出し・形 F の進捗トリガー）を
    /// 持つこと。
    #[test]
    fn layout_css_declares_variant_specific_selectors() {
        assert!(LAYOUT_CSS.contains("[data-blocks-feature-tabs-panel-reverse]"));
        assert!(LAYOUT_CSS.contains(".blocks-feature-tabs-panel-card-grid"));
        assert!(LAYOUT_CSS.contains("[data-align=\"center\"]"));
        assert!(LAYOUT_CSS.contains(".blocks-feature-tabs-panel-trigger-progress"));
    }

    /// 5 形の id 接頭辞（`-basic`/`-pill`/`-alternating`/`-cards`/`-progress`）
    /// がすべて出力へ現れ、重複なく併存すること（モジュール doc「id 規約」
    /// 節。`crates/docs-site/tests/blocks_contract.rs::
    /// demo_output_has_no_dangling_aria_references_or_duplicate_ids` の
    /// 横断検査を補う個別固定）。
    #[test]
    fn each_variant_uses_a_distinct_id_prefix() {
        let html = render(&demo());
        for prefix in [
            "blocks-feature-tabs-panel-basic-",
            "blocks-feature-tabs-panel-pill-",
            "blocks-feature-tabs-panel-alternating-",
            "blocks-feature-tabs-panel-cards-",
            "blocks-feature-tabs-panel-progress-",
        ] {
            assert!(
                html.contains(prefix),
                "expected id prefix {prefix} to appear in demo output"
            );
        }
    }

    /// 形 F（[`super::variant_progress_trigger`]）のトリガーが
    /// `role="progressbar"` と `aria-valuenow`/`data-value` を持つ進捗
    /// バーを含むこと（[`super::trigger_with_progress`] が組み立てる
    /// [`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::
    /// progress::Progress`] の出力契約）。
    #[test]
    fn progress_trigger_variant_renders_a_progressbar_with_a_value() {
        let html = render(&demo());
        assert!(html.contains("role=\"progressbar\""));
        assert!(html.contains("aria-valuenow="));
    }

    /// 形 E（[`super::variant_card_grid`]）のカードグリッドが
    /// [`super::CARDS_FEATURES`]/[`super::CARDS_CASES`] の見出しをすべて
    /// 含むこと（`card::root`/[`super::grid_icon`] の組み立て回帰）。
    #[test]
    fn card_grid_variant_renders_every_card_title() {
        let html = render(&demo());
        for card in super::CARDS_FEATURES
            .iter()
            .chain(super::CARDS_CASES.iter())
        {
            assert!(
                html.contains(card.title),
                "expected card title {:?} to appear in demo output",
                card.title
            );
        }
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-tabs-panel-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-feature-tabs-panel-layout");
    }
}
