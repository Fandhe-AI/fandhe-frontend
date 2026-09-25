//! `feature-vertical-tabs` block（親 #2774「Blocks 目的別パーツ拡充ツリー」
//! 配下、規模 L のため #2775（骨格と主要領域）/ #2776（残り・仕上げ）の
//! 2 sub-issue へ分割済み。いずれも完了）。左列に縦に並んだ feature タブ、
//! 右列に選択中 feature の詳細（見出し・チェック付き機能一覧・画像）を置く
//! 合成例。対応表 ID は R0480（主参照。他に集約元はなく「集約元の差分」は
//! 該当しない）のみを記す（取得手段・ファイル名・内部コンポーネント識別子
//! は記載しない）。
//!
//! # #2775 と #2776 の分担（完了記録）
//!
//! #2775 では骨格と主要領域を実装した: レイアウト root・セクション見出し
//! （`heading` H3 + `text` リード文）・4 タブの [`vertical_tabs`]（先頭タブ
//! 選択済み・[`Orientation::Vertical`]）・各 trigger のタイトル・短い説明・
//! 各パネルの見出し（`heading` H4）・チェック付き機能一覧（`icon` とタイトルと
//! 説明）・画像（`image`）。これで [`BLOCK`] の `parts` が申告する
//! Heading・Text・Tabs・Image・Icon の 5 部品すべてが実際に描画される。
//!
//! #2776 では次を仕上げた: 状態違いの並記（[`demo`] がインスタンス 2 件を
//! 持つ）・trigger 先頭のアイコン（[`FeatureTab::icon`]）・画像主体の別
//! パネル形（[`PanelLayout::ImageFirst`]）・原稿「原案差分メモ」節の本記述。
//! 詳細は下記「1 つの Demo の中で 2 通りの見せ方を並記する」節・「trigger
//! 先頭のアイコン」節を参照。
//!
//! # 1 つの Demo の中で 2 通りの見せ方を並記する
//!
//! 対応表 ID は R0480 のみで集約元は実質 1 件のため、
//! `feature_accordion_image` の「2 形＝2 集約元」（複数の参照からそれぞれ
//! 1 形を作る）とは異なり、本 block は**単一参照からの意図的な派生**として
//! 2 インスタンスを並記する。[`demo`] は [`variant_label`] の見出し付きで、
//! インスタンス 1（既定・`build` タブ選択・[`PanelLayout::ListFirst`]）と
//! インスタンス 2（`secure` タブ選択・[`PanelLayout::ImageFirst`]）を縦に
//! 並べる。前者は #2774 の「初期タブ選択済み固定」要件を、後者は「状態違い
//! の並記」（インスタンス 1 と異なるタブを選択）と「パネルを画像主体にした
//! 別の形」の両方を 1 インスタンスで満たす。`FEATURES`・`dummy_assets` の
//! 既存画像定数はそのまま再利用し、新規データは追加しない。
//!
//! # trigger 先頭のアイコン
//!
//! 参照元 R0480 にはない装飾として、4 タブそれぞれに 1 意匠の自作幾何
//! アイコンを trigger 先頭へ追加した（下記「自作幾何アイコン」節参照）。
//! 隣に可視のタイトル・説明があるため装飾扱い（[`IconProps::label`] は
//! `None` のまま、`aria-hidden="true"`）とする。
//!
//! # 使用部品
//!
//! `heading` / `text` / `tabs` / `image` / `icon` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。新規 UI 部品は追加
//! しない。
//!
//! # 無 JS での扱い
//!
//! docs サイトは JS ハイドレーションを行わない（`crates/docs-site/tests/
//! no_js_contract.rs`）ため、先頭タブ（`build`）を選択済みの固定状態で
//! 描画する。非選択パネルは headless `tabs` が付与する `hidden` 属性で
//! 隠れる（`crates/headless-ui/src/tabs.rs`）。
//!
//! # id 規約
//!
//! 基底 id は `blocks-feature-vertical-tabs-<接尾辞>` とする。インスタンス 1
//! は `-basic`、インスタンス 2 は `-image-primary` を持つ（[`vertical_tabs`]
//! が組み立てる `<id>-trigger-<value>`/`<id>-content-<value>` が全域で一意
//! になるよう、接尾辞をインスタンス間で必ず変える）。
//!
//! # `Orientation::Vertical` を採用する理由（参照元の「Horizontal + 見た目
//! だけ CSS」は採らない）
//!
//! `fandhe_frontend_pre_styled_ui::tabs` の recipe（イシュー #1542/#2039）は
//! `data-orientation="vertical"` に対して root の `display: flex`・list の
//! 縦積み + `border-inline-end`・trigger の `border-inline-end` + 選択中の
//! 強調線・content の `flex: 1` をすでに持つ。lg（64rem）以上の主表示は
//! この recipe だけで賄えるうえ、`aria-orientation="vertical"` が実際の
//! レイアウトと一致する意味論として正しい。**lg 未満だけ** [`LAYOUT_CSS`]
//! が横並びへ上書きする（下記「レスポンシブ」節）。
//!
//! # レスポンシブ（64rem をブレークポイントとする理由・`aria-orientation`
//! のトレードオフ）
//!
//! `< 64rem`（lg 未満）はタブ列をパネルの上へ移して横並び（横スクロール）
//! にする。`>= 64rem` で左の縦タブ列 + 右のパネルの 2 列へ切り替える。
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint`] の `Lg`（1024px =
//! 64rem）と一致するリテラル値 `63.99rem`/`64rem` を [`LAYOUT_CSS`] へ直書き
//! する（`feature_expand`/`feature_split_list_image` と同じ判断）。
//!
//! **トレードオフ**: lg 未満はタブが横に並ぶが `aria-orientation` は
//! `"vertical"` のまま残る。hydration されたページでは矢印キーの移動軸
//! （上下）と見た目の軸（左右）が食い違うが、docs サイトは JS
//! ハイドレーションを一切行わないため実害はない。
//!
//! # trigger 内は phrasing content だけで組む
//!
//! headless の trigger は `<button type="button">` で、許される内容は
//! phrasing content だけである（`heading::heading` の `<h*>` や
//! `styled_text::text` の `<p>` は trigger の中では使えない）。[`trigger_body`]
//! は `span[data-blocks-feature-vertical-tabs-trigger-body]` の中に、
//! タイトル用の `span[data-blocks-feature-vertical-tabs-trigger-title]` と
//! 説明用の `span[data-blocks-feature-vertical-tabs-trigger-desc]` を置き、
//! core の `span`/`text` のみで組む。trigger の base 規則が持つ
//! `white-space: nowrap` を [`LAYOUT_CSS`] で `normal` へ上書きし、`gap` +
//! `flex-direction: column` で縦に積む。lg 未満の横並び時は、trigger 幅の
//! 肥大化を避けるため説明文（`trigger-desc`）を非表示にする（タイトルだけ
//! のタブになる）。
//!
//! # CSS フックの選び方・詳細度の方針
//!
//! `heading`/`text`/`image`/`icon` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo
//! 固有のスタイルフックは `data-blocks-feature-vertical-tabs-*` 属性で
//! 渡す。`tabs`（[`fandhe_frontend_pre_styled_ui::tabs::tabs`]）は root への
//! attrs 注入点を持たないため、レイアウト root の class
//! （[`Block::demo_class`] とは別名の `blocks-feature-vertical-tabs-layout`）
//! を起点にした子孫セレクタ（`.blocks-feature-vertical-tabs-layout
//! [data-scope="tabs"]...`）で上書きする。recipe の
//! `[data-scope][data-part][data-orientation]` 系規則（詳細度 (0,3,0)）・
//! `[data-scope][data-part][data-state][data-orientation]`（(0,4,0)）に
//! 確実に勝つため、上書きは子孫セレクタで 1 クラス分の詳細度を追加する
//! （それぞれ (0,4,0)・(0,5,0) になる）。`image`（recipe 詳細度 (0,2,0)）
//! への上書きも同様に `[data-scope="image"][data-part="root"][data-blocks-
//! feature-vertical-tabs-image]` の 3 セレクタ構成（(0,3,0)）で行う
//! （`feature_split_list_image`/`feature_image_cards` と同型の判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # 自作幾何アイコン
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `feature_split_list_image::geo_icon` と同型の自作ヘルパ [`geo_icon`]
//! （`path` へ `fill="none"` + `stroke="currentColor"` を明示し `icon::icon`
//! の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を上書きして
//! 線画として描画する）で描く。機能一覧のチェックマーク（[`check_icon`]）と
//! 4 タブの trigger アイコン（[`FeatureTab::icon`]、歯車/稲妻/円/盾の単純
//! 図形）はいずれもこのヘルパ 1 つに統一する（重複コード回避）。隣に可視
//! テキストがあるため装飾扱い（[`IconProps::label`] は `None` のまま、
//! `aria-hidden="true"`）とする。
//!
//! # trigger アイコンの CSS
//!
//! `tabs` の trigger recipe（`fandhe_frontend_pre_styled_ui::tabs`）base 規則
//! が既に `display: inline-flex; gap: var(--fandhe-space-2);` を持つため、
//! アイコンとタイトル列の横並び自体は recipe 側で賄える。[`LAYOUT_CSS`] へ
//! 追加するのは `flex-shrink: 0`（縦積みタイトル/説明列に押し潰されない
//! ようにする）のみで、二重定義はしない。
//!
//! # alt を空文字列にする理由
//!
//! いずれの画像も隣接する見出し・機能一覧で内容が伝わる装飾用途のため、
//! `alt=""` にする（`feature_alternating_rows`/`feature_split_list_image`
//! の前例と同じ判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。trigger は headless 由来の `type="button"` で送信先を持たない。
//! 文言はすべて架空のもの（実企業名・実クレデンシャル・PII を含まない）。
//! 画像は [`crate::blocks::dummy_assets`] の各定数（いずれもビルド時生成の
//! プレースホルダー SVG）を使い分け、`alt=""` で出力する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, span, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

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
                vec![core_text("機能を切り替えて確認する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-vertical-tabs-lead", "")],
                vec![core_text(
                    "左のタブを選ぶと、対応する機能の詳細が右側に表示されます。",
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
    /// 画像主体: 見出し → 画像 → 機能一覧（画像は先頭のため下マージンのみ）。
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

/// 縦並び Tabs 本体を組み立てる（並記インスタンス間で再利用する共通
/// ヘルパ。`id` は呼び出し側がリテラルで完全指定する）。
fn vertical_tabs(id: &'static str, selected: &'static str, layout: PanelLayout) -> Node {
    let items: Vec<TabItem<'static>> = FEATURES
        .iter()
        .map(|tab| TabItem {
            value: tab.value,
            trigger: trigger_body(tab),
            content: panel_body(tab, layout),
            disabled: false,
        })
        .collect();
    let props = TabsProps {
        id,
        selected,
        orientation: Orientation::Vertical,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    tabs::tabs(
        TabsVariant::Line,
        Size::Md,
        ColorPalette::Accent,
        &props,
        items,
    )
}

/// `feature-vertical-tabs` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。2 インスタンスを縦に
/// 並べる（モジュール doc「1 つの Demo の中で 2 通りの見せ方を並記する」
/// 節参照）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-vertical-tabs-layout")],
        vec![
            section_header(),
            variant_label("既定（機能一覧が主体・先頭タブ選択）"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-basic",
                "build",
                PanelLayout::ListFirst,
            ),
            variant_label("画像主体のパネル・別タブ選択の例"),
            vertical_tabs(
                "blocks-feature-vertical-tabs-image-primary",
                "secure",
                PanelLayout::ImageFirst,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-vertical-tabs/",
    title: "feature-vertical-tabs",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_vertical_tabs.rs",
    demo_class: "blocks-feature-vertical-tabs",
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
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_vertical_tabs` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-vertical-tabs-*` と `[data-blocks-feature-vertical-tabs-*]`
/// に加え、`tabs`（root attrs 注入点を持たない）への上書きに限り
/// `.blocks-feature-vertical-tabs-layout [data-scope="tabs"]...` の子孫
/// セレクタを用いる（モジュール doc「CSS フックの選び方・詳細度の方針」
/// 節）。他 block や部品の素のセレクタへは影響させない。
const LAYOUT_CSS: &str = "\
.blocks-feature-vertical-tabs-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-feature-vertical-tabs-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-vertical-tabs-lead] {\n  margin: 0;\n}\n\
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"root\"] {\n  display: flex;\n  gap: var(--fandhe-space-8);\n  align-items: flex-start;\n}\n\
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"list\"][data-orientation=\"vertical\"] {\n  flex: 0 0 auto;\n  max-width: 20rem;\n}\n\
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"content\"][data-orientation=\"vertical\"] {\n  flex: 1;\n  min-width: 0;\n}\n\
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"trigger\"][data-orientation=\"vertical\"] {\n  white-space: normal;\n  text-align: start;\n  align-items: flex-start;\n}\n\
[data-blocks-feature-vertical-tabs-trigger-body] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-title] {\n  font-weight: var(--fandhe-font-font-weight-semibold);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-desc] {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
[data-blocks-feature-vertical-tabs-trigger-icon] {\n  flex-shrink: 0;\n}\n\
.blocks-feature-vertical-tabs-points {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin-block: var(--fandhe-space-4) 0;\n}\n\
.blocks-feature-vertical-tabs-point {\n  display: flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feature-vertical-tabs-point-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-vertical-tabs-image] {\n  display: block;\n  width: 100%;\n  margin-top: var(--fandhe-space-4);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-vertical-tabs-image-primary] {\n  display: block;\n  width: 100%;\n  margin: 0 0 var(--fandhe-space-4);\n}\n\
@media (max-width: 63.99rem) {\n  \
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"root\"] {\n    flex-direction: column;\n  }\n  \
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"list\"][data-orientation=\"vertical\"] {\n    flex-direction: row;\n    max-width: none;\n    overflow-x: auto;\n    overflow-y: hidden;\n    border-inline-end: 0;\n    border-bottom: 1px solid var(--fandhe-color-border);\n    padding-bottom: 1px;\n  }\n  \
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"trigger\"][data-orientation=\"vertical\"] {\n    border-inline-end: 0;\n    margin-inline-end: 0;\n    border-bottom: 2px solid transparent;\n    margin-bottom: -1px;\n    flex-shrink: 0;\n    outline-offset: -2px;\n  }\n  \
.blocks-feature-vertical-tabs-layout [data-scope=\"tabs\"][data-part=\"trigger\"][data-state=\"active\"][data-orientation=\"vertical\"] {\n    border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));\n  }\n  \
[data-blocks-feature-vertical-tabs-trigger-desc] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"tabs\"",
            "data-scope=\"image\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
    }

    /// インスタンスごとに 1 タブのみが選択済み・4 パネル中 3 パネルが
    /// `hidden` であること（2 インスタンス分、無 JS 前提の静的固定表示、
    /// モジュール doc「無 JS での扱い」節）。
    #[test]
    fn demo_selects_first_tab_and_hides_other_panels() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 2);
        assert_eq!(html.matches("aria-selected=\"false\"").count(), 6);
        assert_eq!(html.matches(" hidden").count(), 6);
    }

    /// `data-orientation="vertical"`/`aria-orientation="vertical"` が
    /// 出力されること（モジュール doc「`Orientation::Vertical` を採用する
    /// 理由」節）。
    #[test]
    fn demo_declares_vertical_orientation() {
        let html = render(&demo());
        assert!(html.contains("data-orientation=\"vertical\""));
        assert!(html.contains("aria-orientation=\"vertical\""));
    }

    /// 2 インスタンスそれぞれの id 接頭辞が想定どおりであること
    /// （全域で一意になることも兼ねて確認する、モジュール doc「id 規約」
    /// 節）。
    #[test]
    fn demo_uses_expected_id_prefix() {
        let html = render(&demo());
        assert!(html.contains("id=\"blocks-feature-vertical-tabs-basic-trigger-build\""));
        assert!(html.contains("id=\"blocks-feature-vertical-tabs-basic-content-build\""));
        assert!(html.contains("id=\"blocks-feature-vertical-tabs-image-primary-trigger-secure\""));
        assert!(html.contains("id=\"blocks-feature-vertical-tabs-image-primary-content-secure\""));
    }

    /// trigger（`<button>...</button>` の区間）が phrasing content のみで
    /// 構成されること（モジュール doc「trigger 内は phrasing content だけ
    /// で組む」節の不変条件）。
    #[test]
    fn trigger_button_contains_no_block_level_elements() {
        let html = render(&demo());
        let mut start = 0;
        let mut checked = 0;
        while let Some(open_rel) = html[start..].find("<button") {
            let open = start + open_rel;
            let open_end = html[open..].find('>').map(|i| open + i + 1).unwrap();
            let close = html[open_end..]
                .find("</button>")
                .map(|i| open_end + i)
                .unwrap();
            let inner = &html[open_end..close];
            // `"<p"` 単体だと trigger アイコンの `<path>`（svg 要素、phrasing
            // content として許容される）を誤検知するため `"<p "`（属性付き
            // `<p ...>` の開きタグのみ）で判定する（`styled_text::text` は
            // 常に `class`/`data-*` 属性を持つため属性なし `<p>` は存在
            // しない）。
            assert!(
                !inner.contains("<p "),
                "trigger must not contain <p>: {inner}"
            );
            assert!(
                !inner.contains("<h"),
                "trigger must not contain heading tags: {inner}"
            );
            start = close + "</button>".len();
            checked += 1;
        }
        assert_eq!(checked, 8);
    }

    /// trigger アイコンが 2 インスタンス × 4 タブ分（8 件）出力されること
    /// （モジュール doc「trigger 先頭のアイコン」節）。
    #[test]
    fn demo_renders_trigger_icon_for_every_tab_in_every_instance() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-feature-vertical-tabs-trigger-icon=\"\"")
                .count(),
            8
        );
    }

    /// インスタンス 1（[`super::PanelLayout::ListFirst`]）は機能一覧が画像
    /// より DOM 順で先に現れ、インスタンス 2
    /// （[`super::PanelLayout::ImageFirst`]）は画像が機能一覧より先に現れる
    /// こと（モジュール doc「1 つの Demo の中で 2 通りの見せ方を並記する」
    /// 節の不変条件）。
    #[test]
    fn demo_orders_panel_image_and_points_per_instance_layout() {
        let html = render(&demo());
        let basic_start = html
            .find("id=\"blocks-feature-vertical-tabs-basic-content-build\"")
            .expect("instance 1 content should exist");
        let image_primary_start = html
            .find("id=\"blocks-feature-vertical-tabs-image-primary-content-secure\"")
            .expect("instance 2 content should exist");
        assert!(image_primary_start > basic_start);

        let basic_panel = &html[basic_start..image_primary_start];
        let points_index = basic_panel
            .find("blocks-feature-vertical-tabs-points")
            .expect("instance 1 panel should render the points list");
        let image_index = basic_panel
            .find("data-blocks-feature-vertical-tabs-image=\"\"")
            .expect("instance 1 panel should render the ListFirst image hook");
        assert!(
            points_index < image_index,
            "instance 1 (ListFirst) should render points before the image"
        );

        let image_primary_panel = &html[image_primary_start..];
        let image_primary_index = image_primary_panel
            .find("data-blocks-feature-vertical-tabs-image-primary=\"\"")
            .expect("instance 2 panel should render the ImageFirst image hook");
        let points_after_index = image_primary_panel
            .find("blocks-feature-vertical-tabs-points")
            .expect("instance 2 panel should render the points list");
        assert!(
            image_primary_index < points_after_index,
            "instance 2 (ImageFirst) should render the image before points"
        );
    }

    /// [`LAYOUT_CSS`] が lg 未満の横並び上書き・詳細度確保の子孫セレクタを
    /// 持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_overrides() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 63.99rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"tabs\"][data-part=\"list\"][data-orientation=\"vertical\"] {\n    flex-direction: row;"
        ));
        assert!(LAYOUT_CSS.contains("overflow-x: auto;"));
        assert!(LAYOUT_CSS
            .contains("[data-blocks-feature-vertical-tabs-trigger-desc] {\n    display: none;"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（`feature_split_list_image`/`feature_image_cards` と
    /// 同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-vertical-tabs-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-vertical-tabs-layout"
        );
    }
}
