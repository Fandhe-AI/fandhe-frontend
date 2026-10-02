//! `pricing-tiers-extra-row` block（イシュー #2876/#2877。親 #2875「横並びの
//! プランカード + カード群と同じ幅の補足行」配下、対応表 ID R0205 を主参照
//! とする合成例。規模の大きい親 issue を 2 分割し、前半（#2876）が骨格
//! （カード列のレイアウト・レスポンシブ切替）と主参照 R0205 の形（全プラン
//! 共通の機能グリッドを下段に置く）を仕上げ、後半（本 issue #2877）が機能
//! 項目の toggle tip（R0202）と他 2 案の補足行（R0202 のカスタムプラン
//! 問い合わせカード・R1143 の割引プラン横長行）を追加した。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `card` / `button` / `list` / `icon` /
//! `separator` / `toggle-tip` の 9 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs`
//! が検証する）。
//!
//! # 静的表示（無 JS、初期状態で固定）
//!
//! docs サイトは JS ハイドレーションを一切行わないため、本 Demo は状態機械
//! を持たない。プラン・価格・機能はすべて架空の固定値（初期状態のまま）で
//! ある。
//!
//! # レイアウトとブレークポイント
//!
//! 既定（狭い幅）はプランカード 3 枚を縦積みし、`>= 64rem`
//! （[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]）で横 3 列へ
//! 切り替える（mobile-first の `min-width` メディアクエリ、`pricing_seats_
//! split`/`pricing_slider_tiers` と同じブレークポイントに揃える）。補足行
//! （[`extra_common`]・[`extra_custom_plan`]・[`extra_discount`]）はカード列
//! と同じグリッドの全列にまたがる（`grid-column: 1 / -1`）ため、幅は常に
//! カード群と一致する。狭い幅ではカード縦積みの直後に続く。
//!
//! # 機能項目の toggle tip（状態並記、R0202 の集約）
//!
//! 一部の機能項目（Growth の「優先サポート」・Scale の「SLA 保証」・共通
//! 機能の「監査ログ 30 日保持」）へ [`fandhe_frontend_pre_styled_ui::
//! toggle_tip`] の補足を添える。docs サイトは JS ハイドレーションを行わない
//! ため trigger を押しても開閉は変化しない。そこで (1) 3 件の状態を
//! `Open`/`Closed`/`Closed` に固定して両状態を静的に並記し、(2) trigger は
//! 無 JS で機能しないことを示すため `disabled: true` にする
//! （`header_mega_menu::state_label` と同じ判断軸）。trigger のアクセシブル
//! ネームは `aria-label="<機能名> の補足"` を固定文字列で付与し、子には
//! 装飾用の info アイコン（`label: None` → `aria-hidden`）を置く。
//! `positioner` は `.blocks-demo` の `overflow-x: auto` の枠内で絶対配置が
//! 見切れないよう、[`LAYOUT_CSS`] で `position: static` へ中和する
//! （showcase の `pre-styled-showcase [data-scope="toggle-tip"]
//! [data-part="positioner"]` と同じ判断）。toggle tip を含む機能項目は
//! `flex-wrap: wrap` にし、open 時の content が折り返して表示されるように
//! する。
//!
//! `Closed` 固定の 2 件（Scale の「SLA 保証」・共通機能の「監査ログ 30 日
//! 保持」）は headless 層（`crates/headless-ui/src/toggle_tip.rs`）が
//! `Closed` の `positioner`/`content` へ `hidden` 存在属性を付与するため、
//! toggle tip 経由では補足文を静的に閲覧できない。[`feature_note`] は
//! `Closed` の場合のみ `.blocks-pricing-tiers-extra-row-tip-note`
//! （常時可視・muted）で補足文を別途添え、無 JS でも読める状態を保証する
//! （`Open` は `content` 自体が非 `hidden` のため重複させない、イシュー
//! #2877 Codex レビュー指摘）。
//!
//! # 補足行 3 案の並記（R0205/R0202/R1143）
//!
//! カードは 1 組のみ描画し、補足行をグリッド全幅で 3 つ縦に並べる。各行の
//! 先頭に [`state_label`]（`header_mega_menu::state_label` と同型）で案の
//! 種別を示す。
//!
//! - **補足行 A**（[`extra_common`]、R0205）: 全プラン共通機能のグリッド
//!   （前半 #2876 の主参照）。
//! - **補足行 B**（[`extra_custom_plan`]、R0202）: カスタムプラン問い合わせ
//!   カード。`card::root(Outline)` に見出し・説明・`button`（Outline、
//!   「問い合わせる」）を横並び（狭幅は縦積み）で配置する。
//! - **補足行 C**（[`extra_discount`]、R1143）: 割引プランの横長行。
//!   `badge`「20% OFF」+ 見出し + 説明 + `button`（Ghost、「割引を申請する」）
//!   を配置する。参照元 R1143 が持つ暗色帯・カード重ねの装飾は持ち込まない
//!   （既存トーンへ揃えるための意図的な省略、原稿の「原案差分メモ」節
//!   参照）。
//!
//! # 機能リストのチェックが装飾扱いである理由（a11y）
//!
//! 各プランカードの機能リスト・補足行の共通機能グリッドとも「含まれる
//! 機能一覧」であり、可否の情報はチェックマーク自体には宿らない
//! （`pricing_seats_split` と同じ判断）。チェックアイコンは
//! [`fandhe_frontend_pre_styled_ui::list::indicator`]（常に
//! `aria-hidden="true"`）へ収め、項目本文だけを意味のある情報として支援
//! 技術へ伝える。
//!
//! # 推奨強調に badge を使う理由
//!
//! 本 block の使用部品仕様（8 部品）に `badge` を含むため、
//! `pricing_slider_tiers` と同じ形（推奨プランの見出し脇へ
//! [`fandhe_frontend_pre_styled_ui::badge::badge`] の平文「おすすめ」）で
//! 強調する。加えて [`fandhe_frontend_pre_styled_ui::card::CardVariant::
//! Elevated`] + `data-blocks-pricing-tiers-extra-row-card="featured"`
//! （[`LAYOUT_CSS`] が accent 色の 2px 枠を追加）で視覚的にも強調する。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `card::root` /
//! `button::button` / `list::root` / `icon::icon` / `separator::separator`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-pricing-tiers-extra-row-*` 属性で渡す。素の `div`・`span`・
//! `card::header`/`card::body`/`card::footer`/`list::item`（`drop_class_attr`
//! を経由しない）には名前空間分離のため `.blocks-pricing-tiers-extra-row-*`
//! クラスを使う。ルート class（`blocks-pricing-tiers-extra-row-layout`）は
//! [`Block::demo_class`]（`blocks-pricing-tiers-extra-row`）とは意図的に
//! 別名にする（`blog_list_image` 等と同じ Bugbot 教訓の回避）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。プラン名・価格・機能文言はすべて架空のもの（実在の企業名・
//! PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::toggle_tip::{self, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// プラン 1 件分（架空、実在の製品・企業とは無関係）。
struct Plan {
    name: &'static str,
    description: &'static str,
    /// 月額（USD、表示用の固定文字列）。
    price: &'static str,
    cta: &'static str,
    /// 含まれる機能（架空、実在の製品名を含まない）。
    features: &'static [&'static str],
    /// 推奨プランかどうか（強調表示、モジュール doc「推奨強調に badge を
    /// 使う理由」節）。
    featured: bool,
}

/// 3 プラン（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` と同じ名称・
/// 価格水準に揃える）。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        description: "小さなチームがまず試すための最小構成です。",
        price: "$9",
        cta: "Starter を選ぶ",
        features: &["プロジェクト数 3 件まで", "コミュニティサポート"],
        featured: false,
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向けに機能を拡張した構成です。",
        price: "$29",
        cta: "Growth を選ぶ",
        features: &["プロジェクト数無制限", "優先サポート"],
        featured: true,
    },
    Plan {
        name: "Scale",
        description: "大規模なチーム向けの上位構成です。",
        price: "$79",
        cta: "Scale を選ぶ",
        features: &["専任担当者", "SLA 保証"],
        featured: false,
    },
];

/// 全プラン共通の機能（下段の補足行、モジュール doc「レイアウトと
/// ブレークポイント」節・R0205 の主参照）。
const COMMON_FEATURES: &[&str] = &[
    "SSL 証明書の自動更新",
    "99.9% 稼働率 SLA",
    "監査ログ 30 日保持",
    "2 要素認証",
    "リージョン選択",
    "週次バックアップ",
];

/// 機能項目への補足 toggle tip 表（機能名, content id, 補足文, 初期状態）。
/// モジュール doc「機能項目の toggle tip」節参照。1 件だけ `Open` にして
/// 両状態を静的に並記する。`content` の id は固定値（重複禁止・
/// `blocks_contract.rs` の dangling aria/重複 id 検査対象）。
const FEATURE_NOTES: &[(&str, &str, &str, OpenState)] = &[
    (
        "優先サポート",
        "blocks-pricing-tiers-extra-row-tip-1",
        "営業日 24 時間以内に一次回答します。",
        OpenState::Open,
    ),
    (
        "SLA 保証",
        "blocks-pricing-tiers-extra-row-tip-2",
        "月間稼働率 99.9% を下回った場合に利用料を返金します。",
        OpenState::Closed,
    ),
    (
        "監査ログ 30 日保持",
        "blocks-pricing-tiers-extra-row-tip-3",
        "保持期間の延長はオプションで承ります。",
        OpenState::Closed,
    ),
];

/// 補足 toggle tip の装飾アイコン（info 形。参照元の形状は持ち込まない
/// 独自図形。常に `aria-hidden`）。
fn info_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "circle",
                vec![
                    ("cx", "12"),
                    ("cy", "12"),
                    ("r", "9"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M12 11v5M12 8v.01"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// [`FEATURE_NOTES`] に一致する機能名なら補足 toggle tip を添えた
/// トリガーを返す（モジュール doc「機能項目の toggle tip」節参照）。
///
/// headless 層の `positioner`/`content` は `OpenState::Closed` のとき
/// `hidden` 存在属性を付与するため（`crates/headless-ui/src/toggle_tip.rs`）、
/// docs サイトの無 JS 静的 Demo では `Closed` 固定の 2 件（Scale の
/// 「SLA 保証」・共通機能の「監査ログ 30 日保持」）は toggle tip 経由では
/// 補足文を一切閲覧できない（Codex レビュー指摘、イシュー #2877）。
/// 開閉トリガーの静的並記（モジュール doc「機能項目の toggle tip」節）は
/// 維持したまま、`Closed` の場合のみ [`LAYOUT_CSS`] の
/// `.blocks-pricing-tiers-extra-row-tip-note` で常時可視の補足文を別途
/// 添える（`Open` は `content` 自体が非 `hidden` で可視なため重複させない）。
fn feature_note(feature: &str) -> Option<Node> {
    let (_, content_id, note, state) = FEATURE_NOTES
        .iter()
        .find(|(name, ..)| *name == feature)?
        .to_owned();
    let mut children = vec![toggle_tip::root(
        state,
        vec![],
        vec![
            toggle_tip::trigger(
                state,
                true,
                Some(content_id),
                vec![("aria-label", &format!("{feature} の補足"))],
                vec![info_icon()],
            ),
            toggle_tip::positioner(
                state,
                vec![],
                vec![toggle_tip::content(
                    state,
                    Some(content_id),
                    vec![],
                    vec![text(note)],
                )],
            ),
        ],
    )];
    if matches!(state, OpenState::Closed) {
        children.push(span(
            vec![("class", "blocks-pricing-tiers-extra-row-tip-note")],
            vec![text(note)],
        ));
    }
    Some(div(
        vec![("class", "blocks-pricing-tiers-extra-row-tip")],
        children,
    ))
}

/// 機能項目 1 件分（チェックインジケータ + 本文 + 任意の toggle tip）。
/// プランカード・共通機能グリッドの両方の機能リストから共用する。
fn feature_item(feature: &str) -> Node {
    let mut children = vec![
        list::indicator(vec![], vec![check_icon()]),
        span(vec![], vec![text(feature)]),
    ];
    if let Some(tip) = feature_note(feature) {
        children.push(tip);
    }
    list::item(
        vec![("class", "blocks-pricing-tiers-extra-row-feature")],
        children,
    )
}

/// 機能リストのチェックマーク（装飾。モジュール doc「機能リストの
/// チェックが装飾扱いである理由」節参照）。参照元の形状は持ち込まない
/// 独自図形。
fn check_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M5 12.5l4.5 4.5L19 7"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// セクション見出し（H3 見出し + リード文）。
fn section_header() -> Node {
    div(
        vec![("class", "blocks-pricing-tiers-extra-row-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("チームの規模に合わせて選べるプラン")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "すべてのプランに共通機能が含まれます。詳細は下記をご確認ください。",
                )],
            ),
        ],
    )
}

/// プランカード 1 枚。
fn plan_card(plan: &Plan) -> Node {
    let (variant, card_state) = if plan.featured {
        (CardVariant::Elevated, "featured")
    } else {
        (CardVariant::Outline, "default")
    };
    let button_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    let mut heading_row_children = vec![heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(plan.name)],
    )];
    if plan.featured {
        heading_row_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("おすすめ")],
        ));
    }
    let header_children = vec![
        div(
            vec![("class", "blocks-pricing-tiers-extra-row-tier-heading")],
            heading_row_children,
        ),
        card::description(vec![], vec![text(plan.description)]),
    ];

    let price_row = div(
        vec![("class", "blocks-pricing-tiers-extra-row-price-row")],
        vec![
            span(
                vec![("class", "blocks-pricing-tiers-extra-row-price")],
                vec![text(plan.price)],
            ),
            span(
                vec![("class", "blocks-pricing-tiers-extra-row-price-period")],
                vec![text(" / 月")],
            ),
        ],
    );

    let features_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-tiers-extra-row-features", "")],
        plan.features
            .iter()
            .map(|feature| feature_item(feature))
            .collect(),
    );

    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-pricing-tiers-extra-row-card", card_state)],
        vec![
            card::header(
                vec![("class", "blocks-pricing-tiers-extra-row-card-header")],
                header_children,
            ),
            card::body(
                vec![("class", "blocks-pricing-tiers-extra-row-card-body")],
                vec![price_row, features_list],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: button_variant,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text(plan.cta)],
                )],
            ),
        ],
    )
}

/// 状態並記の見出し（モジュール doc「補足行 3 案の並記」節、
/// `header_mega_menu::state_label` と同型）。
fn state_label(label: &str) -> Node {
    span(
        vec![("class", "blocks-pricing-tiers-extra-row-state-label")],
        vec![text(label)],
    )
}

/// 補足行 A（主参照 R0205: 全プラン共通の機能グリッド、モジュール doc
/// 「補足行 3 案の並記」節参照）。
fn extra_common() -> Node {
    let common_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-tiers-extra-row-common", "")],
        COMMON_FEATURES
            .iter()
            .map(|feature| feature_item(feature))
            .collect(),
    );

    div(
        vec![
            ("class", "blocks-pricing-tiers-extra-row-extra"),
            ("data-blocks-pricing-tiers-extra-row-extra", "common"),
        ],
        vec![
            separator::separator(&SeparatorProps::default(), vec![]),
            state_label("補足行 A: 全プラン共通機能"),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text("すべてのプランに含まれる機能")],
            ),
            common_list,
        ],
    )
}

/// 補足行 B（R0202: カスタムプラン問い合わせカード、モジュール doc
/// 「補足行 3 案の並記」節参照）。
fn extra_custom_plan() -> Node {
    div(
        vec![
            ("class", "blocks-pricing-tiers-extra-row-extra"),
            ("data-blocks-pricing-tiers-extra-row-extra", "custom"),
        ],
        vec![
            state_label("補足行 B: カスタムプラン問い合わせ"),
            // `card::root` は `drop_class_attr` により呼び出し側の `class` を
            // 常に除去する（`crates/pre-styled-ui/src/card.rs` 参照）ため、
            // レイアウト用フックは `class` ではなく `data-*` 属性で渡し、
            // CSS 側もこの属性セレクタを使う（下部 CSS 定義参照）。
            card::root(
                CardProps {
                    variant: CardVariant::Outline,
                    ..CardProps::default()
                },
                vec![("data-blocks-pricing-tiers-extra-row-custom-card", "")],
                vec![
                    div(
                        vec![("class", "blocks-pricing-tiers-extra-row-custom-body")],
                        vec![
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps {
                                    size: HeadingSize::Lg,
                                    weight: HeadingWeight::Semibold,
                                },
                                vec![],
                                vec![text("Enterprise")],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text("利用規模・契約条件に応じて個別見積もりを作成します。")],
                            ),
                        ],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("問い合わせる")],
                    ),
                ],
            ),
        ],
    )
}

/// 補足行 C（R1143: 割引プランの横長行、モジュール doc「補足行 3 案の
/// 並記」節参照。参照元の暗色帯・カード重ねは持ち込まない）。
fn extra_discount() -> Node {
    div(
        vec![
            ("class", "blocks-pricing-tiers-extra-row-extra"),
            ("data-blocks-pricing-tiers-extra-row-extra", "discount"),
        ],
        vec![
            state_label("補足行 C: 割引プラン"),
            div(
                vec![("class", "blocks-pricing-tiers-extra-row-discount-row")],
                vec![
                    badge::badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            ..BadgeProps::default()
                        },
                        vec![],
                        vec![text("20% OFF")],
                    ),
                    div(
                        vec![("class", "blocks-pricing-tiers-extra-row-discount-body")],
                        vec![
                            heading(
                                HeadingLevel::H4,
                                &HeadingProps {
                                    size: HeadingSize::Lg,
                                    weight: HeadingWeight::Semibold,
                                },
                                vec![],
                                vec![text("非営利・教育機関向け")],
                            ),
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(
                                    "該当する組織は Growth 以上のプランを割引価格で利用できます。",
                                )],
                            ),
                        ],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("割引を申請する")],
                    ),
                ],
            ),
        ],
    )
}

/// `pricing-tiers-extra-row` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-tiers-extra-row-layout")],
        vec![
            section_header(),
            div(vec![("class", "blocks-pricing-tiers-extra-row-grid")], {
                let mut children: Vec<Node> = PLANS.iter().map(plan_card).collect();
                children.push(extra_common());
                children.push(extra_custom_plan());
                children.push(extra_discount());
                children
            }),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-tiers-extra-row/",
    title: "pricing-tiers-extra-row",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_tiers_extra_row.rs",
    demo_class: "blocks-pricing-tiers-extra-row",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Toggle Tip",
            path: "/themes/toggle-tip/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_tiers_extra_row` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節）。色・間隔はすべて既存
/// トークン（`--fandhe-*`）のみを使う。mobile-first（`min-width: 64rem`）で
/// 3 列へ切り替える（モジュール doc「レイアウトとブレークポイント」節
/// 参照）。
const LAYOUT_CSS: &str = "\
.blocks-pricing-tiers-extra-row-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-pricing-tiers-extra-row-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-tiers-extra-row-grid {\n  display: grid;\n  gap: var(--fandhe-space-6);\n  grid-template-columns: 1fr;\n}\n\
@media (min-width: 64rem) {\n  .blocks-pricing-tiers-extra-row-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
.blocks-pricing-tiers-extra-row-card-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-tiers-extra-row-tier-heading {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-tiers-extra-row-card-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-pricing-tiers-extra-row-price-row {\n  display: flex;\n  align-items: baseline;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-pricing-tiers-extra-row-price {\n  font-size: var(--fandhe-font-font-size-2xl);\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n\
.blocks-pricing-tiers-extra-row-price-period {\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-features] {\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"list\"][data-part=\"item\"].blocks-pricing-tiers-extra-row-feature {\n  display: flex;\n  align-items: center;\n  flex-wrap: wrap;\n  row-gap: var(--fandhe-space-1);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-card=\"featured\"] {\n  border: 2px solid var(--fandhe-color-accent);\n}\n\
.blocks-pricing-tiers-extra-row-extra {\n  grid-column: 1 / -1;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-common] {\n  margin: 0;\n  padding: 0;\n  display: grid;\n  gap: var(--fandhe-space-2) var(--fandhe-space-6);\n  grid-template-columns: 1fr;\n}\n\
@media (min-width: 40rem) {\n  [data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-common] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-common] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
.blocks-pricing-tiers-extra-row-state-label {\n  display: block;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: 600;\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-pricing-tiers-extra-row-layout [data-scope=\"toggle-tip\"][data-part=\"positioner\"] {\n  position: static;\n}\n\
.blocks-pricing-tiers-extra-row-tip {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-pricing-tiers-extra-row-tip-note {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-custom-card] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-4);\n}\n\
@media (min-width: 40rem) {\n  [data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-tiers-extra-row-custom-card] {\n    flex-direction: row;\n    align-items: center;\n    justify-content: space-between;\n  }\n}\n\
.blocks-pricing-tiers-extra-row-custom-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-pricing-tiers-extra-row-discount-row {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-4);\n}\n\
@media (min-width: 40rem) {\n  .blocks-pricing-tiers-extra-row-discount-row {\n    flex-direction: row;\n    align-items: center;\n  }\n}\n\
.blocks-pricing-tiers-extra-row-discount-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  flex: 1;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, COMMON_FEATURES, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 9 種の部品を出力し、`<form>`・`data:` を持たない
    /// こと。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
            "data-scope=\"list\"",
            "data-scope=\"icon\"",
            "data-scope=\"separator\"",
            "data-scope=\"toggle-tip\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
    }

    /// カードが 3 枚で、推奨（featured）は 1 枚だけであること。
    #[test]
    fn demo_has_three_cards_with_one_featured() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-pricing-tiers-extra-row-card="#)
                .count(),
            3
        );
        assert_eq!(
            html.matches(r#"data-blocks-pricing-tiers-extra-row-card="featured""#)
                .count(),
            1
        );
    }

    /// 補足行 3 案（common/custom/discount）が過不足なく 1 件ずつ並記され、
    /// 共通機能の件数が [`COMMON_FEATURES`] と一致すること。
    #[test]
    fn demo_has_three_extra_rows_matching_common_feature_count() {
        let html = render(&demo());
        for kind in ["common", "custom", "discount"] {
            assert_eq!(
                html.matches(&format!(
                    "data-blocks-pricing-tiers-extra-row-extra=\"{kind}\""
                ))
                .count(),
                1,
                "demo output should contain exactly one {kind} extra row"
            );
        }
        assert_eq!(
            html.matches("data-blocks-pricing-tiers-extra-row-common")
                .count(),
            1
        );
        for feature in COMMON_FEATURES {
            assert!(
                html.contains(feature),
                "demo output should contain {feature}"
            );
        }
    }

    /// 機能項目の toggle tip が 3 件描画され、状態並記（open 1 件 /
    /// closed 2 件）・trigger の `disabled`/`aria-label` が固定であること
    /// （モジュール doc「機能項目の toggle tip」節参照）。
    #[test]
    fn demo_has_three_feature_toggle_tips_with_fixed_state_and_disabled_trigger() {
        let html = render(&demo());
        assert_eq!(html.matches("data-scope=\"toggle-tip\"").count(), 3 * 4);
        assert_eq!(html.matches(r#"aria-expanded="true""#).count(), 1);
        assert_eq!(html.matches("data-part=\"content\"").count(), 3);
        assert_eq!(
            html.matches(r#"hidden="""#).count(),
            2 * 2,
            "closed toggle tip contributes hidden on both positioner and content"
        );
        assert_eq!(html.matches(r#" disabled="""#).count(), 3);
        assert_eq!(html.matches("aria-label=\"").count(), 3);
    }

    /// [`LAYOUT_CSS`] が 64rem のブレークポイント条件と、補足行の全幅
    /// （`grid-column: 1 / -1`）規則を持つこと。
    #[test]
    fn layout_css_has_breakpoint_and_full_width_extra_row() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-column: 1 / -1"));
    }

    /// [`LAYOUT_CSS`] が toggle-tip positioner を `position: static` へ
    /// 中和すること（モジュール doc「機能項目の toggle tip」節参照）。
    #[test]
    fn layout_css_neutralizes_toggle_tip_positioner_to_static() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"toggle-tip\"][data-part=\"positioner\"] {\n  position: static;"
        ));
    }

    /// [`super::BLOCK`] の `parts` がモジュール doc「使用部品」節の 9 件と
    /// 一致すること。
    #[test]
    fn block_parts_has_nine_entries() {
        assert_eq!(super::BLOCK.parts.len(), 9);
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-pricing-tiers-extra-row-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-pricing-tiers-extra-row-layout"
        );
    }
}
