# feature-vertical-tabs

`heading` / `text` / `tabs` / `image` / `icon` の 5 部品を合成した、左列に
縦並びの feature タブ、右列に選択中 feature の詳細（見出し・チェック付き
機能一覧・画像）を置くセクションです。4 つの機能（ビルド・デプロイ・観測・
保護）それぞれをタブ選択済みにした 4 インスタンスを並記し、機能一覧が
主体のパネル形と画像主体のパネル形を交互に見せています。

docs サイトは JS ハイドレーションを行わないため、各インスタンスは 1 タブを
選択済みの状態で固定表示しています。実物の `tabs::tabs`（`role="tablist"`/
`role="tab"`/`<button>`）は一切使わず、タブ列は非対話な `<div>` で見た目
だけを模しています。さらに、`data-scope="tabs"`/`data-part="list"|
"trigger"|"content"` という pre-styled-ui の `tabs` recipe と同じセレクタも
一切使わず、block 固有 class（`blocks-feature-vertical-tabs-tablist`/
`-tab`/`-tab-active`/`-panel`）だけで見た目を独自に定義しています。recipe
の `cursor: pointer`/hover 面まで意図せず継承し「クリックできそう」に
見えてしまう不整合を避けるためです。選択中パネルの本文はそのインスタンス
へ直接描画し、残り 3 パネルは同 block の別インスタンスでそれぞれ選択済み
として表示されるため、切り替え手段を持たない静的な並記であることが構造
からも読み取れます。幅 lg（64rem）以上ではタブ列が左に縦並び、パネルが
右に表示されます。幅 lg 未満ではタブ列がパネルの上へ積まれますが、タブ列
自体は縦並びのまま（横並びへは変わらない）です。`role`/`aria-*` を一切
出力しないため、幅による見た目と意味論の食い違いも構造的に発生しません。
各タブの trigger 先頭には自作の幾何アイコンを添えています。

文言・データはすべて架空のもので、データ取得・送信は行わない静的な表示
例です。`<form>` は使用せず、送信先を持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el_owned, span, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を複製
/// しないための単純図形、`feature_split_list_image::geo_icon` と同型の判断。
/// `Size::Sm` 固定。チェックマーク・trigger アイコンの双方をこの 1 つの
/// ヘルパへ統一する）。`attrs` は呼び出し側の CSS フック注入用
/// （trigger アイコンは `data-blocks-feature-vertical-tabs-trigger-icon`
/// を渡す）。
fn geo_icon(path_d: &'static str, attrs: Vec<(&'static str, &'static str)>) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        attrs,
        vec![fandhe_frontend_core::el(
            "path",
            vec![
                ("d", path_d),
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

/// チェックマークの幾何アイコン（機能一覧の各項目に添える）。
fn check_icon() -> Node {
    geo_icon("M5 12l4 4L19 7", vec![])
}

/// trigger 先頭アイコン共通の CSS フック属性。
const TRIGGER_ICON_ATTRS: [(&str, &str); 1] =
    [("data-blocks-feature-vertical-tabs-trigger-icon", "")];

/// 歯車の幾何アイコン（`build` タブの trigger に添える）。
fn gear_icon() -> Node {
    geo_icon(
        "M12 8a4 4 0 100 8 4 4 0 000-8z M12 2v3 M12 19v3 M4.2 4.2l2.1 2.1 M17.7 17.7l2.1 2.1 M2 12h3 M19 12h3 M4.2 19.8l2.1-2.1 M17.7 6.3l2.1-2.1",
        TRIGGER_ICON_ATTRS.to_vec(),
    )
}

/// 稲妻の幾何アイコン（`deploy` タブの trigger に添える）。
fn bolt_icon() -> Node {
    geo_icon("M13 3L5 14h5l-1 7 8-11h-5z", TRIGGER_ICON_ATTRS.to_vec())
}

/// 円の幾何アイコン（`observe` タブの trigger に添える）。
fn circle_icon() -> Node {
    geo_icon(
        "M12 3a9 9 0 100 18 9 9 0 000-18z",
        TRIGGER_ICON_ATTRS.to_vec(),
    )
}

/// 盾の幾何アイコン（`secure` タブの trigger に添える）。
fn shield_icon() -> Node {
    geo_icon(
        "M12 3l7 3v5c0 5-3.5 8.5-7 10-3.5-1.5-7-5-7-10V6z",
        TRIGGER_ICON_ATTRS.to_vec(),
    )
}

/// 機能一覧 1 項目分の架空データ。
struct DetailPoint {
    title: &'static str,
    body: &'static str,
}

/// タブ 1 枚分の架空データ（trigger のタイトル/説明 + パネルの見出し/機能
/// 一覧 3 件/画像）。
struct FeatureTab {
    /// タブ識別 value（ASCII kebab-case）。
    value: &'static str,
    /// trigger 先頭に添える自作幾何アイコン（`Block.demo: fn() -> Node` と
    /// 同型の const 互換フィールド）。
    icon: fn() -> Node,
    /// trigger タイトル。
    title: &'static str,
    /// trigger の短い説明。
    summary: &'static str,
    /// パネル見出し。
    panel_title: &'static str,
    points: [DetailPoint; 3],
    image_src: &'static str,
}

/// 4 タブ分のデータ（画像はタブごとに異なる `dummy_assets` 定数を割り当てる）。
const FEATURES: [FeatureTab; 4] = [
    FeatureTab {
        value: "build",
        icon: gear_icon,
        title: "ビルド",
        summary: "型で表現された構造から静的ファイルを組み立てます。",
        panel_title: "決定的なビルド",
        points: [
            DetailPoint {
                title: "外部依存ゼロの描画コア",
                body: "描画コアは外部クレートに依存しません。",
            },
            DetailPoint {
                title: "型で表現するスロットと props",
                body: "コンポーネントの構造は Rust の型で表現されます。",
            },
            DetailPoint {
                title: "決定的な出力",
                body: "同じ入力からは常に同じ静的ファイルを生成します。",
            },
        ],
        image_src: dummy_assets::PRODUCT_SRC,
    },
    FeatureTab {
        value: "deploy",
        icon: bolt_icon,
        title: "デプロイ",
        summary: "単一実行ファイルへまとめて配布できます。",
        panel_title: "単一バイナリ配布",
        points: [
            DetailPoint {
                title: "SSR/SSG を単一実行ファイルへ",
                body: "サーバー機能を単一バイナリへまとめられます。",
            },
            DetailPoint {
                title: "Docker 想定の配布形態",
                body: "コンテナイメージへそのまま組み込めます。",
            },
            DetailPoint {
                title: "オフライン決定性",
                body: "配布物は同一構成から常に同じ内容になります。",
            },
        ],
        image_src: dummy_assets::SCREENSHOT_SRC,
    },
    FeatureTab {
        value: "observe",
        icon: circle_icon,
        title: "観測",
        summary: "機械検証可能な構成で挙動を追跡します。",
        panel_title: "機械検証可能な構成",
        points: [
            DetailPoint {
                title: "構造マニフェストによる検証",
                body: "依存関係を構造マニフェストが機械検証します。",
            },
            DetailPoint {
                title: "無 JS の静的表示",
                body: "JS ハイドレーションを行わない決定的な表示です。",
            },
            DetailPoint {
                title: "依存グラフ上限の遵守",
                body: "標準構成の依存パッケージ数は上限内に収めます。",
            },
        ],
        image_src: dummy_assets::BACKGROUND_SRC,
    },
    FeatureTab {
        value: "secure",
        icon: shield_icon,
        title: "保護",
        summary: "既定エスケープと限定された unsafe 境界で守ります。",
        panel_title: "既定エスケープと安全な境界",
        points: [
            DetailPoint {
                title: "既定エスケープ",
                body: "テキスト補間は既定でエスケープされます。",
            },
            DetailPoint {
                title: "限定された unsafe 境界",
                body: "描画コア・状態管理コアでは unsafe を使用しません。",
            },
            DetailPoint {
                title: "明示的なオプトイン API",
                body: "エスケープの迂回経路は明示的な API に限られます。",
            },
        ],
        image_src: dummy_assets::LOGO_SRC,
    },
];

/// セクション見出し（heading H3 + リード文）を組み立てる。ページ側が
/// `## Demo` として `h2` を出すため H3 にする（`feature_split_list_image`
/// と同じ判断）。
fn section_header() -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-header")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text("4 つの機能を確認する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-vertical-tabs-lead", "")],
                vec![core_text(
                    "機能ごとにタブを選択した状態のパネルを並べて掲載しています。",
                )],
            ),
        ],
    )
}

/// trigger の中身（アイコン + span、phrasing content 制約に従う。モジュール
/// doc「trigger 内は phrasing content だけで組む」節参照。`<svg>` は
/// phrasing content に該当するため制約に抵触しない）。
fn trigger_body(tab: &FeatureTab) -> Vec<Node> {
    vec![
        (tab.icon)(),
        span(
            vec![("data-blocks-feature-vertical-tabs-trigger-body", "")],
            vec![
                span(
                    vec![("data-blocks-feature-vertical-tabs-trigger-title", "")],
                    vec![core_text(tab.title)],
                ),
                span(
                    vec![("data-blocks-feature-vertical-tabs-trigger-desc", "")],
                    vec![core_text(tab.summary)],
                ),
            ],
        ),
    ]
}

/// 機能一覧 1 項目（チェックアイコン + タイトル + 説明）。
fn detail_point(point: &DetailPoint) -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-point")],
        vec![
            check_icon(),
            div(
                vec![("class", "blocks-feature-vertical-tabs-point-text")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Bold,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(point.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(point.body)],
                    ),
                ],
            ),
        ],
    )
}

/// パネルの構成違い（#2776「パネルを画像主体にした別の形」）。DOM 順を
/// 直接入れ替える（`order` は使わない、`feature_split_list_image` と同じ
/// 判断）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum PanelLayout {
    /// 既定: 見出し → 機能一覧 → 画像（#2775 と同じ、画像は下寄せ）。
    ListFirst,
    /// 画像主体: 見出し → 画像 → 機能一覧（画像は見出しと機能一覧の両側に
    /// 余白を持つ。見出しは `margin: 0` のため上マージンを省くと見出しに
    /// 密着してしまう、#2776 codex-review Medium 是正）。
    ImageFirst,
}

/// パネル画像（レイアウトごとに別の CSS フックを使うため、見出しの直後・
/// 一覧の直前いずれの位置でも margin が競合しない）。
fn panel_image(tab: &FeatureTab, layout: PanelLayout) -> Node {
    let hook = match layout {
        PanelLayout::ListFirst => "data-blocks-feature-vertical-tabs-image",
        PanelLayout::ImageFirst => "data-blocks-feature-vertical-tabs-image-primary",
    };
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Landscape,
            shape: ImageShape::Rounded,
            ..ImageProps::new(tab.image_src, "")
        },
        vec![(hook, "")],
    )
}

/// パネル（content）の中身（見出し H4 + `layout` に応じた一覧/画像の順序）。
fn panel_body(tab: &FeatureTab, layout: PanelLayout) -> Vec<Node> {
    let heading_node = heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![core_text(tab.panel_title)],
    );
    let points = div(
        vec![("class", "blocks-feature-vertical-tabs-points")],
        tab.points.iter().map(detail_point).collect(),
    );
    let image_node = panel_image(tab, layout);
    match layout {
        PanelLayout::ListFirst => vec![heading_node, points, image_node],
        PanelLayout::ImageFirst => vec![heading_node, image_node, points],
    }
}

/// 状態違い・パネル形違いの並記に使う見出し（`feature_accordion_image::
/// variant_label` と同一実装）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(label)],
    )
}

/// 実物の `tabs::tabs` を一切使わない非対話タブ列（モジュール doc「無 JS
/// での扱い（実物の `tabs::tabs` は一切使わない）」節・「recipe セレクタ
/// から完全に切り離す理由」節、`feature_tabs_panel::static_tab_list` と
/// 同型の判断）。`role`/`tabindex`/`<button>` に加え、`data-scope`/
/// `data-part`/`data-state`/`data-orientation`（`fandhe_frontend_pre_
/// styled_ui::tabs` の recipe が読むセレクタ）もいずれも持たない `div` の
/// みで構成し、block 固有 class（`blocks-feature-vertical-tabs-tablist`/
/// `-tab`/`-tab-active`）だけで見た目を独自に定義する（recipe の
/// `cursor: pointer`/hover 面/フォーカスリング等インタラクティブ向け
/// スタイルを一切継承しないため操作可能に見えない、Bugbot Medium 是正）。
/// ラベルのみを持つ装飾要素として `aria-hidden="true"` を付与し支援技術の
/// ツリーから除外する（本 block の trigger はいずれもタイトル/説明のみで
/// 進捗等の実情報を持たないため常に付与してよい）。
fn static_tab_list(id_prefix: &'static str, selected: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-tablist")],
        FEATURES
            .iter()
            .map(|tab| {
                let class = if tab.value == selected {
                    "blocks-feature-vertical-tabs-tab blocks-feature-vertical-tabs-tab-active"
                } else {
                    "blocks-feature-vertical-tabs-tab"
                };
                el_owned(
                    "div",
                    vec![
                        ("class".to_string(), class.to_string()),
                        ("aria-hidden".to_string(), "true".to_string()),
                        (
                            "id".to_string(),
                            format!("{id_prefix}-trigger-{0}", tab.value),
                        ),
                    ],
                    trigger_body(tab),
                )
            })
            .collect(),
    )
}

/// 縦並びタブ列（見た目のみ模す）+ 選択中パネル本体を組み立てる（並記
/// インスタンス間で再利用する共通ヘルパ。`id_prefix` は呼び出し側が
/// リテラルで完全指定する）。実物の `tabs::tabs` を使わないため、選択中
/// パネル 1 件のみを直接描画する（他パネルは同 block の別インスタンスで
/// 可視になる、モジュール doc「全パネルを静的に読めるようにする」節）。
fn vertical_tabs(id_prefix: &'static str, selected: &'static str, layout: PanelLayout) -> Node {
    let selected_tab = FEATURES
        .iter()
        .find(|tab| tab.value == selected)
        .unwrap_or_else(|| panic!("unknown tab value: {selected}"));
    let content = el_owned(
        "div",
        vec![
            (
                "class".to_string(),
                "blocks-feature-vertical-tabs-panel".to_string(),
            ),
            (
                "id".to_string(),
                format!("{id_prefix}-content-{0}", selected_tab.value),
            ),
        ],
        panel_body(selected_tab, layout),
    );
    div(
        vec![("class", "blocks-feature-vertical-tabs-tabs-root")],
        vec![static_tab_list(id_prefix, selected), content],
    )
}

/// `feature-vertical-tabs` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。[`FEATURES`] の 4 タブ
/// それぞれを選択済みにした 4 インスタンスを縦に並べる（モジュール doc
/// 「全パネルを静的に読めるようにする」節参照。#2776 の codex-review P1
/// 是正）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-layout")],
        vec![
            section_header(),
            variant_label("ビルド（機能一覧が主体）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-build",
                "build",
                PanelLayout::ListFirst,
            ),
            variant_label("デプロイ（画像主体）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-deploy",
                "deploy",
                PanelLayout::ImageFirst,
            ),
            variant_label("観測（機能一覧が主体）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-observe",
                "observe",
                PanelLayout::ListFirst,
            ),
            variant_label("保護（画像主体）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-secure",
                "secure",
                PanelLayout::ImageFirst,
            ),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0480。主参照であり他に集約元はないため、いわゆる
「集約元の差分」節は該当しません。出典の固有名・ファイル名は記載しません）
からの意図的な差分は次のとおりです。

- 参照元は横並び（Horizontal）のタブ + 見た目だけを縦並びに寄せる CSS
  という構成でしたが、本実装は `fandhe_frontend_pre_styled_ui::tabs` の
  recipe が既に持つ `data-orientation="vertical"` 相当の縦並び規則を CSS
  フックとしてそのまま採用しました。タブ列自体は lg 未満でも縦並びのまま
  変えず、`root` だけを縦積みにしてタブ列をパネルの上へ積みます。
- 実物の `fandhe_frontend_pre_styled_ui::tabs::tabs`（`role="tablist"`/
  `role="tab"`/`<button>`）は使わず、`data-scope`/`data-part` 属性のみを
  持つ非対話な `<div>` でタブ列の見た目だけを模しています。無 JS の docs
  サイトでは trigger をクリックしても実際には切り替わらないため、
  `role="tablist"`/`role="tab"`/`aria-orientation` を出力すると「操作可能な
  UI」と誤って伝わる不整合が生じる（codex-review 指摘）ためです。選択中
  パネルの本文はそのインスタンスへ直接描画し、残り 3 パネルは同 block の
  別インスタンスで可視になります（`feature_tabs_panel::static_tab_list`
  と同型の対処）。
- 参照元の背景帯・装飾・実際の文言は持ち込まず、文言はすべて独自の架空
  のもの（日本語）にしました。
- 見出しは `h3`/`h4` に下げました（ページ側が `## Demo` として `h2` を
  出すため）。
- 画像は `dummy_assets` のプレースホルダーと `alt=""`（装飾扱い）にしま
  した。
- 参照元にはない trigger 先頭のアイコンを、lucide 等の既存アイコンセットの
  path を複製しない自作の単純な線画（歯車・稲妻・円・盾）として追加しま
  した。機能一覧のチェックマークと同じヘルパで統一しています。
- 対応表 ID は R0480 のみで集約元は実質 1 件のため、いわゆる「集約元の
  差分」は存在しません。代わりに、単一参照からの意図的な派生として 4 つの
  見せ方を並記し、設計上の可動域と全パネルの静的な到達可能性を示して
  います。4 タブ（`build`/`deploy`/`observe`/`secure`）それぞれを選択済みに
  した 4 インスタンスで、機能一覧が主体のパネル（`build`/`observe`）と
  画像が見出し直後に来る画像主体のパネル（`deploy`/`secure`）を交互に
  見せています。
