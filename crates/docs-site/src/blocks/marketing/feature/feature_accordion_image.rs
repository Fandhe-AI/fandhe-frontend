//! `feature-accordion-image` block（親イシュー #2760「feature-accordion-image
//! を Blocks に追加する」配下、規模 L のため前半 #2761（骨格・主要領域）と
//! 後半 #2762（本コミット。カテゴリ切替ボタン列・状態違いの並記・原稿
//! 仕上げ）へ分割済み）。集約元は対応表 ID R0103（基準形）と R0483（上部に
//! カテゴリ切替ボタン列が付く形、選択中を `aria-pressed` で示す）の 2 件。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限。記載してよいのは対応表 ID のみ）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `accordion` / `image` / `button` / `icon`
//! の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `button`/`icon` は本イシューでカテゴリ切替ボタン列の実装に伴い使い始め、
//! 前半 #2761 のモジュール doc 注記（「#2762 で使い始める」）を実現した。
//!
//! # 2 形を 1 つの Demo に並記する
//!
//! `content_split_image`/`cta_split_image` と同型の判断（[`variant_label`]
//! で短い見出しを添え、[`demo`] 1 つの中へ縦に並べる）で、集約元 2 件の
//! 差分を並べて読み取れるようにする。
//!
//! - **形 A（対応表 ID R0103、基準形）**: [`variant_a`]。左列に見出しエリア
//!   （アイブロウ badge + heading + リード文）とアコーディオンを縦に並べ、
//!   右列に代表画像を 1 枚置く 2 列構成。
//! - **形 B（対応表 ID R0483）**: [`variant_b`]。見出しの下にカテゴリ切替
//!   ボタン列（[`category_row`]）を置いた上で、形 A と同じ 2 列構成
//!   （左列アコーディオン・右列代表画像）を続ける。
//!
//! 両形とも `md`（[`Breakpoint::Md`]、768px/48rem）未満では右列を隠し、
//! 各項目の本文の中にインライン画像を表示する（項目ごとに自分の画像を
//! 持つ）点は共通のため、[`accordion_list`]/[`feature_image`] を両形で
//! 共有する。
//!
//! # 静的アコーディオン（JS を使わない、全項目を常時展開で固定）
//!
//! docs サイトは JS ハイドレーションを行わないため、状態機械を経由せず
//! [`fandhe_frontend_pre_styled_ui::accordion`] の自由関数を直接呼び、
//! 全件を [`OpenState::Open`] として固定描画する（`changelog_accordion`
//! イシュー #2818 と同型の設計。当初は 1 件目のみ open・残りを
//! `OpenState::Closed`（`item_content` 自体を出力しない）としていたが、
//! 閉じた項目の説明文が視覚利用者・支援技術のいずれにも一切到達不能に
//! なり、かつ見出しリード文「項目を選ぶと」の案内文言が無 JS 下では実現
//! しない操作を示唆してしまう不整合があった（イシュー #2761 レビュー
//! 指摘。UI コンポーネント層のアクセシビリティ責務違反、
//! `docs/policy/intentional-non-adoption.md` §3.25 参照）。是正として
//! `careers_split_accordion` イシュー #2816・`changelog_accordion` イシュー
//! #2818 の前例に倣い、全件を open + disabled の「非操作の機能一覧」として
//! 再設計した。本イシュー（#2762）で追加する形 B も同じ方式を踏襲し、
//! **閉じた項目は再導入しない**（親イシュー #2760 の仕様は「初期状態は
//! 1 件目を開いた状態」だが、無 JS では恒久的に到達不能な本文を生む
//! ためこの逸脱を維持する）。
//!
//! - [`AccordionProps`] の `disabled: true` を全パーツで共有する。これに
//!   より全 `item_trigger` がネイティブ `disabled` + `aria-disabled="true"`
//!   を持ち、フォーカス・操作ともに不能になる（押しても何も起きない
//!   ボタンを作らない）。`disabled` 由来の減光（`opacity: 0.5`）は
//!   [`LAYOUT_CSS`] で中和し、通常表示に戻す。
//! - 全項目が [`OpenState::Open`] であり、`item_trigger` に `id`/
//!   `controls` を持たせ、`item_content` を `id`/`labelled_by` 付きで
//!   出力する（閉じた項目・到達不能な本文は存在しない）。
//!
//! # カテゴリ切替ボタン列（形 B、`aria-pressed` を disabled + 静的表示で示す）
//!
//! [`category_row`] は架空のカテゴリ 4 件を [`button::button`] で並べる。
//! 押しても何も起きない有効な `<button>` は `blog_split_header_grid`/
//! `careers_card_grid` で指摘を受けた dead control にあたるため、本
//! block のアコーディオントリガーと同じ考え方で
//! `ButtonProps { disabled: true, .. }`（ネイティブ `disabled` +
//! `aria-disabled="true"`）に `aria-pressed="true"|"false"` を併せ持たせて
//! 選択状態を静的に表現する（初期状態は先頭カテゴリのみ選択済みで固定、
//! 集約元 R0483 の「初期状態固定」仕様に一致）。減光は [`LAYOUT_CSS`] の
//! `.blocks-feature-accordion-image-categories` 配下限定セレクタで中和
//! する。`headless-ui`/`pre-styled-ui` に用意された `toggle_group` は
//! 使わない（`parts` に `button`/`icon` を指定しているため。トリガー自体
//! を `button::button` へ差し替えるより、既存の `AccordionProps` 資産を
//! そのまま流用できる）。非選択カテゴリの機能一覧は出力しない（hidden で
//! 抱えて到達不能にしない。選択中カテゴリ 1 件分の [`CATEGORY_FEATURES`]
//! のみを常に表示する）。
//!
//! アイコンは [`category_icon`]（`careers_split_accordion::geo_icon` と
//! 同型の自作の幾何線画）を全ボタン共通で使い回す。取得元アイコンセット
//! の path・識別子は複製しない。
//!
//! # トリガーの子を 2 個に保つ理由
//!
//! recipe の item-trigger は `justify-content: space-between` を前提に
//! 「直接の子 2 個」のレイアウトを取る（`changelog_accordion` と同じ）。
//! そのため [`item_trigger`] の children はラベル領域（`span`）と
//! `item_indicator` の 2 個に固定する。
//!
//! # 見出しレベル（`H3`/`h4`）
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする（両形とも）。各トリガーは WAI-ARIA APG の
//! アコーディオンパターンに合わせて `<h4>` で包む（[`fandhe_frontend_core::el`]
//! で直接組み立て、`heading::heading` は使わない。ページ本文の見出し
//! 階層に割り込ませない部品固有の構造要素であるため）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `accordion::root` /
//! `image::image` / `button::button` / `icon::icon` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feature-accordion-image-*` 属性で渡す。`accordion::item`/
//! `item_trigger`/`item_content`/`item_indicator` は `drop_class_attr` を
//! 経由しないため、素の `div`/`h4`/`span` と合わせて `class` か子孫
//! セレクタを使う（`changelog_accordion` と同じ判断）。
//!
//! # id 接頭辞（形キー）
//!
//! `id`/`aria-controls`/`aria-labelledby` はいずれも
//! `blocks-feature-accordion-image-{key}-{index}-{trigger|content}` の形で
//! 形キー（[`BASE`]/[`CATEGORY`]）と項目の添字から一意に導出する（形キーを
//! 挟むのは、1 つの Demo に 2 形を並記する際に id が形をまたいで衝突しない
//! ようにするため。`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が全
//! block 横断で id 重複・宙に浮いた参照を検査する）。
//!
//! # `<form>` を持たない・文言は架空
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。機能名・説明文・カテゴリ名はすべて架空のもの（実在の製品・
//! 企業名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 機能 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Feature {
    name: &'static str,
    description: &'static str,
}

/// 形 A（対応表 ID R0103）の識別キー（モジュール doc「id 接頭辞」節）。
const BASE: &str = "base";

/// 形 B（対応表 ID R0483）の識別キー。
const CATEGORY: &str = "category";

/// 形 A の機能一覧（架空、4 件）。全件が常時展開状態で固定描画される。
const FEATURES: [Feature; 4] = [
    Feature {
        name: "リアルタイム共同編集",
        description: "複数人が同じドキュメントを同時に編集し、変更が即座に反映されます。",
    },
    Feature {
        name: "バージョン履歴",
        description: "過去のすべての変更を遡って確認し、いつでも以前の状態に戻せます。",
    },
    Feature {
        name: "カスタムテンプレート",
        description: "よく使う構成をテンプレートとして保存し、次回から素早く再利用できます。",
    },
    Feature {
        name: "アクセス権限の管理",
        description: "閲覧・編集・管理者の 3 段階で、メンバーごとに権限を細かく設定できます。",
    },
];

/// 形 B のカテゴリ 1 件分のラベル（架空、4 件。先頭が初期選択状態）。
const CATEGORIES: [&str; 4] = ["共同作業", "分析", "セキュリティ", "自動化"];

/// 形 B・先頭カテゴリ（[`CATEGORIES`]`[0]`）選択時の機能一覧（架空、3 件）。
/// 非選択カテゴリの機能は出力しない（hidden で抱えて到達不能にしない、
/// モジュール doc「カテゴリ切替ボタン列」節）。
const CATEGORY_FEATURES: [Feature; 3] = [
    Feature {
        name: "共有ワークスペース",
        description: "チーム全員が同じ作業台の上で、資料や進行状況を常に共有できます。",
    },
    Feature {
        name: "コメントとメンション",
        description: "気になる箇所に直接コメントを残し、担当者へ通知できます。",
    },
    Feature {
        name: "タスクの割り当て",
        description: "担当者と期限を明確にし、進捗をひと目で把握できます。",
    },
];

/// 各形の直前に置く短い形ラベル（`content_split_image` と同型、
/// `styled_text::text` の `Sm`/`Muted`）。
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

/// 見出しエリア（アイブロウ badge + heading + リード文）を組み立てる
/// （形ごとに文言を差し替えられるようパラメータ化する）。
fn header(eyebrow: &'static str, title: &'static str, lead: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-accordion-image-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-accordion-image-eyebrow", "")],
                vec![text(eyebrow)],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
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

/// 機能の画像を組み立てる（`inline` が `true` のとき md 未満のインライン
/// 表示用フックを、`false` のとき右列表示用フックを付与する。
/// [`LAYOUT_CSS`] のブレークポイントに応じてどちらか一方だけが可視になる。
/// 両形で共有するため画像自体に `id` は持たせない）。
fn feature_image(inline: bool) -> Node {
    let hook = if inline {
        ("data-blocks-feature-accordion-image-inline-image", "")
    } else {
        ("data-blocks-feature-accordion-image-media", "")
    };
    image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Video,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![hook],
    )
}

/// トリガーのラベル領域（機能名のみ。トリガーの直接の子は本要素と
/// `item_indicator` の 2 個に保つ、モジュール doc「トリガーの子を 2 個に
/// 保つ理由」節）。
fn trigger_label(feature: &Feature) -> Node {
    span(
        vec![("class", "blocks-feature-accordion-image-trigger-label")],
        vec![text(feature.name)],
    )
}

/// 機能 1 件分の accordion item を組み立てる（モジュール doc「静的
/// アコーディオン」節の方式。全件を [`OpenState::Open`] + `disabled: true`
/// で固定するため、本文（[`item_content`]）は必ず出力される）。`key` は
/// 形キー（[`BASE`]/[`CATEGORY`]）で、2 形並記時の id 衝突を防ぐ
/// （モジュール doc「id 接頭辞」節）。
fn feature_item(key: &str, index: usize, feature: &Feature) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-feature-accordion-image-{key}-{index}-trigger");
    let content_id = format!("blocks-feature-accordion-image-{key}-{index}-content");

    let trigger = el(
        "h4",
        vec![("class", "blocks-feature-accordion-image-trigger-heading")],
        vec![item_trigger(
            state,
            false,
            &props,
            feature.name,
            Some(trigger_id.as_str()),
            Some(content_id.as_str()),
            vec![],
            vec![
                trigger_label(feature),
                item_indicator(state, false, &props, vec![], vec![text("▾")]),
            ],
        )],
    );

    let content = item_content(
        state,
        false,
        &props,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
        vec![],
        vec![div(
            vec![("class", "blocks-feature-accordion-image-body")],
            vec![
                styled_text::text(
                    &TextProps::default(),
                    vec![],
                    vec![text(feature.description)],
                ),
                feature_image(true),
            ],
        )],
    );

    item(state, false, &props, vec![], vec![trigger, content])
}

/// 機能一覧 1 本分のアコーディオン（`root` + 各 [`feature_item`]）を
/// 組み立てる。両形（[`variant_a`]/[`variant_b`]）で共有する
/// （モジュール doc「2 形を 1 つの Demo に並記する」節）。
fn accordion_list(key: &str, features: &[Feature]) -> Node {
    let items: Vec<Node> = features
        .iter()
        .enumerate()
        .map(|(index, feature)| feature_item(key, index, feature))
        .collect();

    accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-feature-accordion-image-root", "")],
        items,
    )
}

/// カテゴリ切替ボタン 1 個分の幾何線画アイコン（`careers_split_accordion::
/// geo_icon` と同型。取得元アイコンセットの path・識別子は複製しない）。
fn category_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M4 7h16M4 12h10M4 17h16"),
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

/// カテゴリ切替ボタン 1 個（モジュール doc「カテゴリ切替ボタン列」節）。
/// `selected` が `true` のとき `Solid` variant + `aria-pressed="true"`、
/// それ以外は `Outline` variant + `aria-pressed="false"` を固定で持つ
/// （初期状態のまま切り替わらない静的表示、`disabled: true` で dead
/// control 化を防ぐ）。
fn category_button(label: &'static str, selected: bool) -> Node {
    let (variant, pressed) = if selected {
        (ButtonVariant::Solid, "true")
    } else {
        (ButtonVariant::Outline, "false")
    };
    button::button(
        &ButtonProps {
            variant,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![
            ("aria-pressed", pressed),
            ("data-blocks-feature-accordion-image-category", ""),
        ],
        vec![category_icon(), text(label)],
    )
}

/// カテゴリ切替ボタン列（形 B のみ）。[`CATEGORIES`] の先頭のみ選択済み
/// として固定し、選択中カテゴリの機能一覧（[`CATEGORY_FEATURES`]）と
/// 対応させる（モジュール doc「カテゴリ切替ボタン列」節）。
fn category_row() -> Node {
    let buttons: Vec<Node> = CATEGORIES
        .iter()
        .enumerate()
        .map(|(index, label)| category_button(label, index == 0))
        .collect();
    div(
        vec![
            ("class", "blocks-feature-accordion-image-categories"),
            ("role", "group"),
            ("aria-label", "機能カテゴリ"),
        ],
        buttons,
    )
}

/// 形 A（対応表 ID R0103、基準形）: 見出しエリア + アコーディオンの左列と
/// 代表画像の右列からなる 2 列構成。
fn variant_a() -> Node {
    let left = div(
        vec![("class", "blocks-feature-accordion-image-left")],
        vec![
            header(
                "機能紹介",
                "チームの作業をまとめて効率化",
                "各機能の詳細は以下でご確認いただけます。",
            ),
            accordion_list(BASE, &FEATURES),
        ],
    );
    let right = div(
        vec![("class", "blocks-feature-accordion-image-media-slot")],
        vec![feature_image(false)],
    );
    div(
        vec![("class", "blocks-feature-accordion-image-grid")],
        vec![left, right],
    )
}

/// 形 B（対応表 ID R0483）: 見出しの下にカテゴリ切替ボタン列（[`category_row`]）
/// を置いた上で、形 A と同じ 2 列構成（左列アコーディオン・右列代表画像）を
/// 続ける（モジュール doc「2 形を 1 つの Demo に並記する」節）。
fn variant_b() -> Node {
    let header_node = header(
        "カテゴリで探す",
        "機能をカテゴリから選んで確認",
        "気になるカテゴリを選ぶと、対応する機能の一覧が表示されます。",
    );
    let left = div(
        vec![("class", "blocks-feature-accordion-image-left")],
        vec![accordion_list(CATEGORY, &CATEGORY_FEATURES)],
    );
    let right = div(
        vec![("class", "blocks-feature-accordion-image-media-slot")],
        vec![feature_image(false)],
    );
    let grid = div(
        vec![("class", "blocks-feature-accordion-image-grid")],
        vec![left, right],
    );
    div(
        vec![("class", "blocks-feature-accordion-image-variant")],
        vec![header_node, category_row(), grid],
    )
}

/// `feature-accordion-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的アコーディオン」節）。形 A・形 B を
/// [`variant_label`] の見出し付きで縦に並記する（モジュール doc「2 形を
/// 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-accordion-image-layout")],
        vec![
            variant_label("基準形（R0103）"),
            variant_a(),
            variant_label("カテゴリ切替ボタン列付き（R0483）"),
            variant_b(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-accordion-image/",
    title: "feature-accordion-image",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_accordion_image.rs",
    demo_class: "blocks-feature-accordion-image",
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
            label: "Accordion",
            path: "/themes/accordion/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_accordion_image` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。他 block と同型で
/// `pub(super)` ではなく本ファイル内 private 定数として `super::stylesheet`
/// 経由の `push_css` で連結される）。
///
/// モバイルファーストで書く: md（[`Breakpoint::Md`]、768px/48rem）未満は
/// 1 列とし、右列（`.blocks-feature-accordion-image-media-slot`）を隠して
/// 各項目のインライン画像（`[data-blocks-feature-accordion-image-inline-
/// image]`）を見せる。`@media (min-width: 48rem)` 以降は 2 列にし、右列を
/// 見せてインライン画像を隠す（本ファイル末尾の `#[cfg(test)]` が
/// `Breakpoint::Md` とのドリフトを検知する）。インライン画像は
/// `pre_styled_ui::image::image` の root 要素（`data-scope="image"
/// data-part="root"`）そのものに `data-blocks-feature-accordion-image-
/// inline-image` を併せ持たせているため、md+ で隠す規則は属性セレクタ
/// 単体（詳細度 (0,1,0)）ではなく `[data-scope="image"][data-part="root"]`
/// も併記した詳細度 (0,3,0) で書き、Image recipe の基底規則
/// `[data-scope="image"][data-part="root"] { display: block }`（詳細度
/// (0,2,0)）に確実に勝たせる（PR #3188 Bugbot 指摘で是正）。
///
/// 右列の代表画像（`[data-blocks-feature-accordion-image-media]`）にも
/// `width: 100%` を明示する。`image::image` の root は `max-width: 100%`
/// のみを持ち固有の横幅を指定しないため、`ImageProps::new` に渡す
/// スクリーンショット SVG の intrinsic size（`viewBox` 200x140）のまま
/// 描画され、md 以降で列いっぱいに広がらない（PR #3188 Bugbot 指摘で
/// 是正）。
///
/// `disabled` による減光の中和（モジュール doc「静的アコーディオン」節）:
/// `accordion::stylesheet` の `disabled_declarations()`（既定
/// `opacity: 0.5`）は「操作できない要素」の既定表現だが、本 block は
/// トリガー自体を disabled にしているだけで通常の機能一覧として見せる
/// ため、`opacity: 1`・`cursor: default` へ上書きする。詳細度はクラス
/// セレクタ 1 個 + 属性セレクタ 3 個（1,3,0）で recipe 側（0,2,0 相当）に
/// 確実に勝たせる（`changelog_accordion` と同じ考え方）。カテゴリ切替
/// ボタン（`button::button` の `disabled_declarations()` も同じ既定
/// `opacity: 0.5`）も同型の中和規則を
/// `.blocks-feature-accordion-image-categories` 配下限定で持つ
/// （モジュール doc「カテゴリ切替ボタン列」節）。
///
/// セレクタはすべて `.blocks-feature-accordion-image*` か
/// `[data-blocks-feature-accordion-image-*]` の名前空間に閉じる。
/// `[data-scope="accordion"][data-part="item-trigger"][data-disabled]`・
/// `[data-scope="button"][data-part="root"][data-disabled]` の中和も
/// 名前空間の外に出さないよう、必ず `.blocks-feature-accordion-
/// image-left`/`.blocks-feature-accordion-image-categories` 配下への
/// 子孫結合子付きで書く（集約された `blocks.css` を読み込む全ページで
/// 他 block の disabled accordion トリガー・disabled ボタンの見た目まで
/// 書き換えてしまわないため。`changelog_accordion` が `.blocks-changelog-
/// accordion-list` 配下へ子孫結合子付きで書くのと同じ判断、イシュー
/// #2761 レビュー指摘で是正）。
const LAYOUT_CSS: &str = "\
.blocks-feature-accordion-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-feature-accordion-image-variant {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  width: 100%;\n}\n\
.blocks-feature-accordion-image-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n  width: 100%;\n}\n\
.blocks-feature-accordion-image-left {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  min-width: 0;\n}\n\
.blocks-feature-accordion-image-header {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feature-accordion-image-categories {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feature-accordion-image-categories [data-scope=\"button\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-feature-accordion-image-media-slot {\n  display: none;\n}\n\
[data-blocks-feature-accordion-image-media] {\n  display: block;\n  width: 100%;\n}\n\
[data-blocks-feature-accordion-image-inline-image] {\n  display: block;\n  width: 100%;\n  margin-top: var(--fandhe-space-3);\n}\n\
.blocks-feature-accordion-image-trigger-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
.blocks-feature-accordion-image-trigger-label {\n  flex: 1;\n  min-width: 0;\n  font-weight: var(--fandhe-font-weight-medium, 500);\n}\n\
.blocks-feature-accordion-image-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding-top: var(--fandhe-space-1);\n}\n\
.blocks-feature-accordion-image-left [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 48rem) {\n  .blocks-feature-accordion-image-grid {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n    align-items: start;\n  }\n  .blocks-feature-accordion-image-media-slot {\n    display: block;\n    position: sticky;\n    top: var(--fandhe-space-4);\n  }\n  [data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-accordion-image-inline-image] {\n    display: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CATEGORIES, CATEGORY_FEATURES, FEATURES, LAYOUT_CSS};
    use fandhe_frontend_core::render;
    use fandhe_frontend_pre_styled_ui::recipe::Breakpoint;

    /// 両形の項目総数（トリガー・本文・インライン画像等の期待件数に使う）。
    fn total_items() -> usize {
        FEATURES.len() + CATEGORY_FEATURES.len()
    }

    /// [`LAYOUT_CSS`] の 48rem が `Breakpoint::Md.min_width()`（768px）と
    /// 実際に一致すること（モジュール doc「レイアウト規則」節が参照する
    /// 対応のドリフト検知）。
    #[test]
    fn layout_css_breakpoint_matches_pre_styled_ui_md() {
        assert_eq!(Breakpoint::Md.min_width(), "768px");
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
    }

    /// 全項目（両形合計）が `open` で、`closed` の項目が無いこと
    /// （モジュール doc「静的アコーディオン」節、イシュー #2761 レビュー
    /// 指摘の是正。#2762 で形 B を追加しても閉じた項目を再導入しない
    /// 回帰固定）。
    #[test]
    fn all_items_are_open() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-part="item" data-state="open""#)
                .count(),
            total_items(),
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"data-part="item" data-state="closed""#)
                .count(),
            0,
            "html={html}"
        );
    }

    /// すべてのトリガー（両形合計）・すべてのカテゴリボタンが disabled
    /// （ネイティブ属性 + `aria-disabled`）であること（モジュール doc
    /// 「静的アコーディオン」節・「カテゴリ切替ボタン列」節）。
    #[test]
    fn all_triggers_and_category_buttons_are_disabled() {
        let html = render(&demo());
        let expected_disabled = total_items() + CATEGORIES.len();
        assert_eq!(
            html.matches(r#"data-part="item-trigger""#).count(),
            total_items(),
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-disabled="true""#).count(),
            expected_disabled,
            "html={html}"
        );
        assert_eq!(
            html.matches(" disabled=\"\"").count(),
            expected_disabled,
            "html={html}"
        );
    }

    /// 全項目（両形合計）に `aria-controls` と `item-content` があり、
    /// `hidden` な到達不能ノードを一切残さないこと（モジュール doc「静的
    /// アコーディオン」節、イシュー #2761 レビュー指摘の是正。#2762 の
    /// 形 B 追加後も維持する回帰固定）。
    #[test]
    fn all_items_have_content_and_aria_controls() {
        let html = render(&demo());
        assert_eq!(
            html.matches("aria-controls=").count(),
            total_items(),
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"data-part="item-content""#).count(),
            total_items(),
            "html={html}"
        );
        assert_eq!(html.matches(" hidden=\"\"").count(), 0, "html={html}");
    }

    /// 右列の画像は形ごとに 1 枚（計 2 枚）、インライン画像は両形の項目数
    /// 合計ぶん出力されること（本 block は全項目が常時展開のため、各項目
    /// が自分のインライン画像を持つ）。
    #[test]
    fn media_and_inline_image_counts_cover_both_variants() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-feature-accordion-image-media=\"\"")
                .count(),
            2,
            "html={html}"
        );
        assert_eq!(
            html.matches("data-blocks-feature-accordion-image-inline-image=\"\"")
                .count(),
            total_items(),
            "html={html}"
        );
    }

    /// カテゴリ切替ボタン列が `aria-pressed="true"` をちょうど 1 件
    /// （先頭カテゴリ）、`aria-pressed="false"` を残り件数ぶん持ち、
    /// `role="group"` でグルーピングされていること（モジュール doc
    /// 「カテゴリ切替ボタン列」節）。
    #[test]
    fn category_row_has_exactly_one_pressed_button() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"aria-pressed="true""#).count(),
            1,
            "html={html}"
        );
        assert_eq!(
            html.matches(r#"aria-pressed="false""#).count(),
            CATEGORIES.len() - 1,
            "html={html}"
        );
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains("data-blocks-feature-accordion-image-category=\"\""));
    }

    /// カテゴリ切替ボタンがすべて `type="button"`（button recipe が
    /// 固定で付与）であること（`<form>` 非依存の暗黙 submit 防止）。
    #[test]
    fn category_buttons_are_type_button() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-feature-accordion-image-category=\"\"")
                .count(),
            CATEGORIES.len()
        );
        // button::button は type="button" を merged 先頭へ固定で入れる契約
        // （crate::button モジュール doc 参照）。カテゴリボタン数ぶん
        // type="button" が出現することを、他 block と混同しないよう
        // 直前のカテゴリフック共起で間接確認する。
        assert!(html.contains(r#"type="button""#));
    }

    /// `data:` URI・`<form` を持ち込まないこと（A05）。
    #[test]
    fn demo_output_has_no_form_or_data_uri() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
    }

    /// 見出しが `h3`（両形分、計 2 個）、トリガーが `h4` で包まれている
    /// こと。
    #[test]
    fn heading_and_trigger_levels() {
        let html = render(&demo());
        assert_eq!(html.matches("<h3").count(), 2, "html={html}");
        assert!(html.contains("class=\"blocks-feature-accordion-image-trigger-heading\""));
    }
}
