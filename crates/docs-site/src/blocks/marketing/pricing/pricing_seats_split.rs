//! `pricing-seats-split` block（イシュー #2865/#2866。親 #2864「Marketing /
//! Pricing block 追加」配下、対応表 ID R0204 を主参照とする合成例。規模の
//! 大きい親 issue を 2 分割した後半（#2866）であり、前半（#2865）が
//! 実装した骨格（2 カラムのレイアウトとレスポンシブ切り替え・座席数
//! 入力・年払い switch・プランカード 3 枚の価格表示）へ、機能リスト
//! （`list` + `icon`）・推奨プランの強調・月払い状態の並記を追加して
//! 仕上げる（取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! 契約、対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `number-input` / `switch` / `card` / `button` /
//! `list` / `icon` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # 静的表示（無 JS、2 状態を並記して固定）
//!
//! docs サイトは JS ハイドレーションを一切行わないため、実行時の状態切替
//! （座席数入力の増減・年払い switch のトグル）は表現できない。代わりに
//! 「年払い」「月払い」の 2 状態を [`STATES`] として並記し、Demo は縦に
//! 2 ブロックを描画する（`changelog_timeline` 等が集約元との差分を並記で
//! 示す先例と同型）。座席数は両状態とも 5 に固定し、周期のみを変えることで
//! 価格差を比較しやすくする。数値入力の増減トリガーは押しても何も起きない
//! no-op になるため、`readonly` にしたうえで
//! [`fandhe_frontend_pre_styled_ui::number_input`] の増減トリガーを
//! `disabled: true` にしてフォーカス不能・操作不能であることを支援技術・
//! キーボード双方に明示する（`header_flyout_menu` 等の前例と同じ判断）。
//! switch も `readonly: true` の固定表示に留める。
//!
//! 価格は座席数と課金周期から決定的に計算する固定関数（[`monthly_total`]）
//! であり、実行時のスライダー・入力連動（`pricing_usage_slider` が明示的に
//! 実装しないのと同じ判断、`docs/policy/intentional-non-adoption.md` §3.25
//! の責務境界）は行わない。
//!
//! # レイアウトとブレークポイント
//!
//! 既定（狭い幅）は入力パネルの下にプランカード 3 枚を縦積みし、
//! `>= 64rem`（[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]）
//! で左に入力パネル・右にカード列を並べる 2 カラムのグリッドへ切り替える
//! （mobile-first の `min-width` メディアクエリ、親 issue のレイアウト
//! 仕様に従う）。この 2 カラム規則は状態ごとの本体（`.blocks-pricing-
//! seats-split-split`）へ適用し、状態間の縦並び（`.blocks-pricing-seats-
//! split-layout`/`.blocks-pricing-seats-split-state`）とは分離する。
//!
//! # 状態の並記と `id` の引数化
//!
//! [`DemoState`] が座席数入力の `id`（`input_id`）を保持し、
//! [`controls`]/[`plan_card`] の双方へ引数として渡す。2 インスタンス
//! （年払い/月払い）を同一ページへ並記するため、`id`/`for`/
//! `aria-controls` を固定文字列にすると重複・宙に浮いた参照になる
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` の対象）。
//! switch は `id` を出力しない（`fandhe_frontend_headless_ui::switch`
//! の anatomy 契約）ため状態間の衝突を考慮しない。
//!
//! # 機能リストのチェックが装飾扱いである理由（a11y）
//!
//! 兄弟 block `comparison_cards`（`Included`/`Excluded` の可否）と異なり、
//! 本 block の機能リストは「このプランに含まれる機能一覧」であり、
//! 各項目は常に「含まれる」で情報の可否を運ばない（可否という情報が
//! チェックマークそのものに宿らない）。そのためチェックアイコンは
//! [`fandhe_frontend_pre_styled_ui::list::indicator`]（常に
//! `aria-hidden="true"` の装飾用パーツ）へ収め、項目本文（プレーン
//! テキスト）だけを意味のある情報として支援技術へ伝える。
//!
//! # 推奨強調に badge を使わない理由
//!
//! 親 issue の使用部品仕様（8 部品）に `badge` を含まないため、Demo が
//! 出力する `data-scope` 集合と [`BLOCK`] の `parts` 宣言を一致させる
//! 契約（`blocks_contract.rs`）を崩さないよう、推奨強調は
//! [`fandhe_frontend_pre_styled_ui::card::CardVariant::Elevated`] +
//! `data-blocks-pricing-seats-split-card="featured"`（[`LAYOUT_CSS`] が
//! accent 色の 2px 枠を追加）+ header 先頭の平文「おすすめ」で表す
//! （`comparison_cards` の badge 強調と同じ発想を badge 抜きで再現する）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `number_input::root` /
//! `switch::root` / `card::root` / `button::button` / `list::root` /
//! `icon::icon` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-pricing-seats-split-*` 属性で渡す。素の `div`・
//! `card::header`/`card::body`/`card::footer`/`list::item`（`drop_class_attr`
//! を経由しない）には名前空間分離のため `.blocks-pricing-seats-split-*`
//! クラスを使う（`comparison_cards` 等と同じ判断）。ルート class
//! （`blocks-pricing-seats-split-layout`）は [`Block::demo_class`]
//! （`blocks-pricing-seats-split`）とは意図的に別名にする（`blog_list_image`
//! 等と同じ Bugbot 教訓の回避）。
//!
//! # 割引前価格の取り消し線に平文の「通常」を添える理由（a11y）
//!
//! `<s>`（打ち消し線）は読み上げで区別されにくいため、直前へ平文の
//! 「通常」を置いて視覚に依存しない形で割引前価格であることを伝える
//! （年払い状態のみ。月払い状態は割引がないため出さない）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。プラン名・価格・座席数・機能文言はすべて架空のもの（実在の
//! 企業名・サービス名・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// Demo が並記する状態 1 件分（架空）。座席数入力の `id`（`input_id`）を
/// 状態ごとに変えることで、2 インスタンス並記時の id 重複・宙に浮いた
/// `aria-controls`/`for` 参照を避ける（モジュール doc「状態の並記と `id`
/// の引数化」節）。
struct DemoState {
    /// 状態のラベル（並記時の見出し）。
    label: &'static str,
    /// 座席数（両状態で揃え、周期のみ変える）。
    seats: u32,
    /// 年払いかどうか。
    annual: bool,
    /// 座席数入力の `id`（状態ごとに固有）。
    input_id: &'static str,
}

/// 2 状態（年払い・月払い）を並記する。座席数は両状態とも 5 に固定。
const STATES: [DemoState; 2] = [
    DemoState {
        label: "年払い（初期状態）",
        seats: 5,
        annual: true,
        input_id: "blocks-pricing-seats-split-seats-annual",
    },
    DemoState {
        label: "月払い",
        seats: 5,
        annual: false,
        input_id: "blocks-pricing-seats-split-seats-monthly",
    },
];

/// プラン 1 件分（架空、実在の製品・企業とは無関係）。
struct Plan {
    name: &'static str,
    description: &'static str,
    /// 1 座席あたりの月額（USD、整数）。
    per_seat_usd: u32,
    cta: &'static str,
    /// 含まれる機能（架空、実在の製品名を含まない）。
    features: &'static [&'static str],
    /// 推奨プランかどうか（強調表示、モジュール doc「推奨強調に badge を
    /// 使わない理由」節）。
    featured: bool,
}

/// 3 プラン（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` と同じ
/// 名称・価格水準に揃える）。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        description: "小さなチームがまず試すための最小構成です。",
        per_seat_usd: 9,
        cta: "Starter を選ぶ",
        features: &[
            "プロジェクト数 3 件まで",
            "コミュニティサポート",
            "基本レポート",
        ],
        featured: false,
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向けに機能を拡張した構成です。",
        per_seat_usd: 29,
        cta: "Growth を選ぶ",
        features: &["プロジェクト数無制限", "優先サポート", "高度なレポート"],
        featured: true,
    },
    Plan {
        name: "Scale",
        description: "大規模なチーム向けの上位構成です。",
        per_seat_usd: 79,
        cta: "Scale を選ぶ",
        features: &["専任担当者", "SLA 保証", "監査ログ"],
        featured: false,
    },
];

/// 座席数・課金周期から月額合計を求める決定的な純関数。年払いは 20% 引き
/// （整数演算のみ、四捨五入は行わない）。外部入力・ユーザー入力は受け
/// 取らない。
#[must_use]
fn monthly_total(per_seat_usd: u32, seats: u32, annual: bool) -> u32 {
    if annual {
        per_seat_usd * seats * 8 / 10
    } else {
        per_seat_usd * seats
    }
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

/// 左パネル（見出し・座席数入力・年払い switch）。
fn controls(state: &DemoState) -> Node {
    let number_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let seats_value = state.seats.to_string();

    let seats_input = number_input::root(
        Size::Md,
        false,
        false,
        true,
        vec![("data-blocks-pricing-seats-split-seats", "")],
        vec![
            number_input::label(
                number_flags,
                Some(state.input_id),
                vec![],
                vec![text("座席数")],
            ),
            number_input::control(
                number_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some(state.input_id),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                    number_input::input(
                        "seats",
                        Some(state.input_id),
                        Some(&seats_value),
                        "1",
                        "100",
                        number_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some(state.input_id),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    );

    let switch_props = SwitchProps {
        readonly: true,
        // `readonly` は `data-readonly` を出すのみで native checkbox の
        // クリック/Space による切り替え自体は止めない（headless-ui
        // `SwitchProps::readonly` doc「native トグル操作自体を抑止する
        // 配線は持たない」節）。座席数入力の増減トリガーと同じ判断で
        // `disabled: true` も付与し、[`switch::hidden_input`] へ native
        // `disabled` を出力して操作を実際に抑止する。
        disabled: true,
        ..SwitchProps::default()
    };
    let annual_switch = switch::root(
        Size::Md,
        ColorPalette::Accent,
        state.annual,
        &switch_props,
        vec![("data-blocks-pricing-seats-split-annual", "")],
        vec![
            switch::label(
                state.annual,
                &switch_props,
                vec![],
                vec![text("年払い（20% お得）")],
            ),
            switch::hidden_input("billing-annual", "on", state.annual, &switch_props, vec![]),
            switch::control(
                state.annual,
                &switch_props,
                vec![],
                vec![switch::thumb(state.annual, &switch_props, vec![], vec![])],
            ),
        ],
    );

    let lead_text = if state.annual {
        "座席数 5・年払いの場合の月額例です（本 Demo は静的表示のため入力と連動しません）。"
    } else {
        "座席数 5・月払いの場合の月額例です（本 Demo は静的表示のため入力と連動しません）。"
    };
    let annual_status_text = if state.annual {
        "現在、年払いを選択中です（本 Demo は固定表示のため切替できません）。"
    } else {
        "現在、月払いを選択中です（本 Demo は固定表示のため切替できません）。"
    };

    div(
        vec![("class", "blocks-pricing-seats-split-controls")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("座席数に合わせてプランを選ぶ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-seats-split-lead", "")],
                vec![text(lead_text)],
            ),
            seats_input,
            annual_switch,
            // `disabled: true` の checkbox は支援技術が checked 状態を
            // 安定して読み上げない（ブラウザ・スクリーンリーダーの
            // 組み合わせにより無視され得る）ため、選択中の周期を読める
            // 平文でも明示する（Codex レビュー指摘、イシュー #2865
            // PR #3278）。
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-seats-split-annual-status", "")],
                vec![text(annual_status_text)],
            ),
        ],
    )
}

/// プランカード 1 枚。
fn plan_card(plan: &Plan, state: &DemoState) -> Node {
    let total = monthly_total(plan.per_seat_usd, state.seats, state.annual);
    let regular_total = plan.per_seat_usd * state.seats;

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

    let mut header_children: Vec<Node> = Vec::new();
    if plan.featured {
        header_children.push(styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-pricing-seats-split-recommended", "")],
            vec![text("おすすめ")],
        ));
    }
    header_children.push(heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(plan.name)],
    ));
    header_children.push(card::description(vec![], vec![text(plan.description)]));

    let mut price_row_children = vec![
        span(
            vec![("class", "blocks-pricing-seats-split-price")],
            vec![text(format!("${total}"))],
        ),
        span(
            vec![("class", "blocks-pricing-seats-split-price-period")],
            vec![text(format!(
                " / 月（{} 席・{}）",
                state.seats,
                if state.annual {
                    "年払い"
                } else {
                    "月払い"
                }
            ))],
        ),
    ];
    if state.annual {
        price_row_children.push(span(
            vec![("class", "blocks-pricing-seats-split-price-regular")],
            vec![
                text("通常 "),
                el("s", vec![], vec![text(format!("${regular_total}"))]),
            ],
        ));
    }

    let features_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-seats-split-features", "")],
        plan.features
            .iter()
            .map(|feature| {
                list::item(
                    vec![("class", "blocks-pricing-seats-split-feature")],
                    vec![
                        list::indicator(vec![], vec![check_icon()]),
                        span(vec![], vec![text(*feature)]),
                    ],
                )
            })
            .collect(),
    );

    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-pricing-seats-split-card", card_state)],
        vec![
            card::header(
                vec![("class", "blocks-pricing-seats-split-card-header")],
                header_children,
            ),
            card::body(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-pricing-seats-split-price-row")],
                        price_row_children,
                    ),
                    features_list,
                ],
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

/// 状態 1 件分（ラベル + 2 カラム本体）。
fn state_section(state: &DemoState) -> Node {
    div(
        vec![("class", "blocks-pricing-seats-split-state")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-seats-split-state-label", "")],
                vec![text(state.label)],
            ),
            div(
                vec![("class", "blocks-pricing-seats-split-split")],
                vec![
                    controls(state),
                    div(
                        vec![("class", "blocks-pricing-seats-split-cards")],
                        PLANS.iter().map(|plan| plan_card(plan, state)).collect(),
                    ),
                ],
            ),
        ],
    )
}

/// `pricing-seats-split` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。2 状態（年払い・月払い）を並記する（モジュール doc「静的表示」
/// 節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-seats-split-layout")],
        STATES.iter().map(state_section).collect(),
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-seats-split/",
    title: "pricing-seats-split",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_seats_split.rs",
    demo_class: "blocks-pricing-seats-split",
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
            label: "Number Input",
            path: "/themes/number-input/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
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
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_seats_split` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。色・間隔はすべて既存トークン
/// （`--fandhe-*`）のみを使う。mobile-first（`min-width: 64rem`）で
/// 2 カラムへ切り替える（モジュール doc「レイアウトとブレークポイント」
/// 節参照）。
///
/// 末尾の switch 規則: 年払い切替は `disabled: true`（native トグル操作の
/// 実際の抑止、`switch_props` の doc コメント参照）を持つため switch
/// recipe の `disabled_declarations()`（`opacity: 0.5`）が適用され、
/// 課金オン状態にもかかわらずコントロールと 20% ラベルが利用不可に
/// 見えてしまう。`changelog_accordion`/`faq_split_accordion` 等の静的
/// デモが同種の `disabled` opt-out を `opacity: 1`/`cursor: default` で
/// 打ち消す先例に倣い、本 block でも打ち消す。
const LAYOUT_CSS: &str = "\
.blocks-pricing-seats-split-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-pricing-seats-split-state {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-pricing-seats-split-state-label] {\n  margin: 0;\n}\n\
.blocks-pricing-seats-split-split {\n  display: grid;\n  gap: var(--fandhe-space-8);\n}\n\
@media (min-width: 64rem) {\n  .blocks-pricing-seats-split-split {\n    grid-template-columns: minmax(16rem, 1fr) 2fr;\n    align-items: start;\n  }\n}\n\
.blocks-pricing-seats-split-controls {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-pricing-seats-split-lead] {\n  margin: 0;\n}\n\
.blocks-pricing-seats-split-cards {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  grid-template-columns: 1fr;\n}\n\
@media (min-width: 64rem) {\n  .blocks-pricing-seats-split-cards {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
.blocks-pricing-seats-split-card-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-pricing-seats-split-price-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-pricing-seats-split-price {\n  font-size: var(--fandhe-font-font-size-2xl);\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n\
.blocks-pricing-seats-split-price-period {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-pricing-seats-split-price-regular {\n  width: 100%;\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-seats-split-features] {\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"list\"][data-part=\"item\"].blocks-pricing-seats-split-feature {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-seats-split-card=\"featured\"] {\n  border: 2px solid var(--fandhe-color-accent);\n}\n\
[data-blocks-pricing-seats-split-annual][data-scope=\"switch\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, monthly_total, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 種の部品を出力し、`<form>`・`data:` を持たない
    /// こと。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"number-input\"",
            "data-scope=\"switch\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
            "data-scope=\"list\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
    }

    /// 年払いの割引（20% 引き）が座席数へ正しく反映されること。
    #[test]
    fn monthly_total_applies_annual_discount() {
        assert_eq!(monthly_total(9, 5, true), 36);
        assert_eq!(monthly_total(29, 5, true), 116);
        assert_eq!(monthly_total(79, 5, true), 316);
        assert_eq!(monthly_total(9, 5, false), 45);
    }

    /// カード 6 枚（2 状態 × 3 プラン）・取り消し線 3 件（年払い状態のみ、
    /// 各カードの通常価格）が出力されること。
    #[test]
    fn demo_renders_six_cards_with_strikethrough_only_on_annual() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-pricing-seats-split-card"#)
                .count(),
            6
        );
        assert_eq!(html.matches("<s>").count(), 3);
    }

    /// 月払い状態は月払い表記の周期ラベルを含み、取り消し線を持たない
    /// カード価格になること。
    #[test]
    fn demo_renders_monthly_state_without_discount() {
        let html = render(&demo());
        assert!(html.contains("月払い"));
        assert!(html.contains("/ 月（5 席・月払い）"));
        assert!(html.contains("/ 月（5 席・年払い）"));
        // 月払いの決定的な価格（割引なし）。
        assert!(html.contains("$45"));
        assert!(html.contains("$145"));
        assert!(html.contains("$395"));
    }

    /// 推奨プラン（Growth）が状態ごとに 1 枚（合計 2）あり、CTA の
    /// `Solid`/`Outline` variant が混在すること。
    #[test]
    fn demo_has_one_featured_card_per_state_with_mixed_cta_variants() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"data-blocks-pricing-seats-split-card="featured""#)
                .count(),
            2
        );
        assert!(html.contains("fd-button--variant-solid"));
        assert!(html.contains("fd-button--variant-outline"));
    }

    /// 2 つの座席数入力の `id` が相異なり、`aria-controls` の参照先が
    /// 実在すること（`blocks_contract.rs` の全 block 対象検査を補完する
    /// 軽量な固定回帰）。
    #[test]
    fn demo_has_distinct_input_ids_across_states() {
        let html = render(&demo());
        assert!(html.contains(r#"id="blocks-pricing-seats-split-seats-annual""#));
        assert!(html.contains(r#"id="blocks-pricing-seats-split-seats-monthly""#));
    }

    /// [`LAYOUT_CSS`] が 64rem のブレークポイント条件を持つこと。
    #[test]
    fn layout_css_switches_to_two_columns_at_lg() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }

    /// 年払い switch は `disabled: true` により recipe 側の
    /// `disabled_declarations()`（`opacity: 0.5`）が効くため、
    /// [`LAYOUT_CSS`] がこれを `opacity: 1`/`cursor: default` へ
    /// 打ち消す規則を持つこと（codex Bugbot 指摘の固定回帰）。
    #[test]
    fn layout_css_counteracts_disabled_switch_fade() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-pricing-seats-split-annual][data-scope=\"switch\"][data-part=\"root\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-pricing-seats-split-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-pricing-seats-split-layout");
    }
}
