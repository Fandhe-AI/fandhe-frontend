# pricing-tiers-extra-row

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `card` /
`button` / `list` / `icon` / `separator` / `toggle-tip` を合成した、横並びの
プランカード + カード群と同じ幅の補足行を持つ料金プランの合成例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な例です。docs サイトは JS ハイドレーションを一切行わない
ため、状態切替は行わず初期状態のまま固定表示します。プラン名・価格・機能
名・補足文はすべて架空の値です。

プランカードを 3 枚横に並べ、下段にカード群と同じ幅の補足行を 3 案並べて
続けます。狭い幅ではカードを縦に積み、補足行はその下に続きます。各機能
項目は「このプランに含まれる機能」であり可否情報を持たないため、チェック
マークは装飾（`list::indicator`、常に `aria-hidden`）として扱い、項目本文
だけを意味のある情報として伝えます。推奨プラン（Growth）は輪郭を強調した
カード + 見出し先頭の `badge`「おすすめ」で強調しています。

一部の機能項目（Growth の「優先サポート」・Scale の「SLA 保証」・共通機能の
「監査ログ 30 日保持」）には補足 `toggle-tip` を添えています。docs サイトは
JS ハイドレーションを行わないため、trigger を押しても開閉状態は変化しま
せん。そこで 3 件のうち 1 件を開状態、残り 2 件を閉状態に固定し、両方の
見た目を静的に並記しています。trigger は無 JS では機能しないため
`disabled` にし、アクセシブルネームは `aria-label="<機能名> の補足"` を
固定で付与しています。
閉状態は `hidden` により補足文が非表示になるため、閉状態の 2 件
（「SLA 保証」「監査ログ 30 日保持」）は補足文を常時可視のテキストとし
ても別途表示し、無 JS でも読めるようにしています。

補足行はカード群と同じグリッド幅で 3 案を縦に並べています。

- **補足行 A**: 全プラン共通機能のグリッド。
- **補足行 B**: カスタムプラン問い合わせカード。
- **補足行 C**: 割引プラン（非営利・教育機関向け）の横長行。

各行の先頭には案の種別を示すラベルを添えています。

## Rust コード

```rust
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
```

## 原案差分メモ

- **R0205**（全プラン共通機能）: 補足行 A としてそのまま実装。プラン共通の
  機能をグリッドで並べる構成を主参照とした。
- **R0202**（機能ごとの補足）: 機能項目への toggle tip とカスタムプラン
  問い合わせカードの 2 箇所へ分けて実装した。toggle tip は既存の
  `heading`/`h4` 見出しレベルに揃え、カスタムプランカードは `card::root`
  （Outline）で他のプランカードと視覚言語を統一した。
- **R1143**（割引プラン）: 補足行 C の横長行として実装した。参照元が持つ
  暗色帯・カード重ねの装飾は、既存トーン（`--fandhe-*` トークンのみ・生色
  なし）へ揃えるため意図的に採用しなかった。
- **toggle tip の状態**: docs サイトは JS ハイドレーションを行わないため、
  trigger を押しても開閉は変化しない。3 件のうち 1 件を開状態に固定して
  両状態を静的に並記し、trigger は `disabled` にして無 JS では操作できない
  ことを示した。
- **positioner の配置**: `.blocks-demo` の横スクロール枠内で絶対配置が
  見切れないよう、`positioner` を `position: static` へ中和してフロー内へ
  配置した。
- **見出しレベル**: セクション見出しは H3、カード内・補足行内の見出しは
  H4 で統一した。
- **文言**: プラン名・価格・機能名・補足文・カスタムプラン文言はすべて
  架空のものであり、実在の企業・製品・PII を含まない。

関連情報: [Toggle Tip](../themes/toggle-tip.md) / [Card](../themes/card.md) /
[Badge](../themes/badge.md)
