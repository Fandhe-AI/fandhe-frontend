//! `pricing-single-split` block（イシュー #2867。親トラッキング #2807
//! 「Blocks 目的別パーツ拡充」配下、対応表 ID R1144 を主参照とし、R0200
//! （支払周期選択の radio card 版）を集約した合成例。単一プランを大きく
//! 見せる 2 カラム構成で、左カラムに説明・機能一覧、右カラムに価格パネルを
//! 置く。
//!
//! # 使用部品
//!
//! `heading` / `text` / `card` / `button` / `list` / `icon` / `radio-card` /
//! `separator` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! **Marketing / Pricing カテゴリで `radio-card`（イシュー #747/#1490/#1492）
//! を初めて使用する block**（既存の `pricing-tiers-morph`/`pricing-usage-
//! slider`/`pricing-comparison-table` はいずれも `radio-card` を使わない）。
//!
//! # Demo を 2 インスタンス並べる理由（原案差分の表現）
//!
//! 主参照 R1144（プラン説明 + 価格パネルの単純な 2 カラム）に対し、集約元
//! R0200（左カラムへ支払周期〔月額／年額〕を選ぶ radio card を追加した形）
//! を差分として持つ。1 つの Demo へ両形を無理に詰め込むと「radio card が
//! ある版」の存在が読み取りにくくなるため、`plan_split(false)`（標準形）と
//! `plan_split(true)`（支払周期選択付き）の 2 インスタンスを縦に並べる
//! （`faq-accordion-centered` が全件 open のみを描く単一インスタンス方針とは
//! 逆に、`login`/`signup` 系 block が variant 差分を複数インスタンスで示す
//! 前例に近い判断）。両インスタンスは `data-blocks-pricing-single-split-
//! variant="standard"`/`"billing"` で区別できる。
//!
//! # 支払周期 radio card をネイティブ disabled にする理由
//!
//! [`radio_card::item_hidden_input`] は有効なネイティブ `<input
//! type="radio">` であり、`disabled` を渡さない構成では docs サイトが JS
//! ハイドレーションを行わなくてもラベルクリック・キーボード操作でブラウザ
//! が `checked` をネイティブに切り替えてしまう。一方 `item`/`item-control`/
//! `item-indicator` の見た目（`data-state="checked"`）は SSR 時の `checked`
//! 引数から固定生成されるため追従せず、実際に選択・送信される値と支援
//! 技術が認識する状態・カスタム radio の視覚表示が食い違う。
//! `contact_split_form_image`（`crates/docs-site/src/blocks/marketing/
//! contact/contact_split_form_image.rs`）が `radio_group` の同種の問題を
//! `RadioGroupProps { disabled: true, .. }` で解決した判断を踏襲し、
//! `root`・全 [`billing_item`]（`item`/`item_hidden_input`/`item_control`/
//! `item_indicator`）へ `disabled: true` を共有する。ネイティブ `disabled`
//! 属性でフォーカス・操作を不能にし、状態が二度と変化しないことを構造的に
//! 保証する（クライアント側の状態配線を追加する代替案は採らない。無 JS の
//! 静的合成例という block 全体の設計方針に反するため）。
//! `disabled_declarations()`（既定 `opacity: 0.5` + `cursor: not-allowed`）は
//! [`LAYOUT_CSS`] で中和し、通常の radio card と同じ見た目に保つ
//! （`contact_split_form_image` と同型の中和パターン）。年額を選んだ状態
//! （`checked: true`）で固定する。
//!
//! # 参照について
//!
//! 主参照は対応表 ID R1144、集約元は R0200。取得手段・ファイル名・出典名・
//! 内部識別子は記載しない（他 block と同じライセンス上の転記制限、対応表
//! ID のみを記す）。取り込むのは領域の配置と部品構成という構造のみで、
//! 文言・配色・装飾・アイコンは独自に書く。参照との差分は
//! `site/blocks/pricing-single-split.md` の「原案差分メモ」節に記載する。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、プラン名は `HeadingLevel::H3`
//! を、「含まれる機能」の小見出しは `HeadingLevel::H4` を使う（既存 block と
//! 同じ判断）。価格パネル内の価格表示も `HeadingLevel::H3` とする（本文の
//! 見出し階層に割り込まない独立パネルのため `contact_split_form_image` の
//! フォーム見出しと同じ H3 段を用いる）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`text`/`card::root`/`list::root`/`icon`/`button`/
//! `radio_card::root`/`separator` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有
//! のスタイルフックは `data-blocks-pricing-single-split-*` 属性で渡す
//! （`list::item`/`list::indicator`/`radio_card::item`/`radio_card::
//! item_control`/`radio_card::item_indicator` は `drop_class_attr` を経由
//! しないが、既存部品の scope セレクタとの衝突を避けるため同じく data 属性
//! フックへ統一する）。素の `div` には `class` がそのまま効くため、レイア
//! ウトは `.blocks-pricing-single-split-*` クラスセレクタを使う。ルート
//! class（`blocks-pricing-single-split-layout`）は [`Block::demo_class`]
//! （`blocks-pricing-single-split`）と意図的に別名にする（既存 block と同じ
//! Bugbot 教訓の回避）。
//!
//! # id / ARIA の方針
//!
//! id は支払周期選択の [`radio_card::label`] のみに付与する
//! （[`BILLING_LABEL_ID`]）。[`radio_card::root`] の `labelled_by` をこの id
//! と一致させ、`aria-labelledby` が宙に浮かないようにする
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が検証）。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
//! 48rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と
//! 同じ判断）。狭幅（既定）は本文と価格パネルを縦に積み、`48rem` 以上で
//! 2 カラム grid（本文 3fr・価格パネル 2fr）に切り替える。機能一覧は
//! `40rem` 以上で 2 列 grid にする。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 支払周期選択の見出し [`radio_card::label`] の id（モジュール doc「id /
/// ARIA の方針」節）。
const BILLING_LABEL_ID: &str = "blocks-pricing-single-split-billing-label";

/// 支払周期選択のネイティブ `<input>` の共通 `name`。
const BILLING_NAME: &str = "blocks-pricing-single-split-billing";

/// 含まれる機能の一覧（架空の文言、実在の企業名・個人情報は含まない）。
const FEATURES: [&str; 8] = [
    "無制限のプロジェクト作成",
    "チームメンバー招待（上限なし）",
    "権限管理・監査ログ",
    "優先サポート窓口",
    "外部連携 API アクセス",
    "月次利用状況レポート",
    "シングルサインオン対応",
    "99.9% の稼働率 SLA",
];

/// 含まれる機能を表す装飾用のチェック図形（意味を持たないため `label` は
/// `None`〔既定〕のまま。参照元のアイコン形状・内部識別子は持ち込まない）。
fn check_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M5 12.5l4 4L19 7"),
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

/// 機能一覧 1 行分（`list::item`）。
fn feature_item(label: &'static str) -> Node {
    list::item(
        vec![],
        vec![
            list::indicator(vec![], vec![check_icon()]),
            styled_text::text(&TextProps::default(), vec![], vec![text(label)]),
        ],
    )
}

/// 含まれる機能の一覧本体（2 列 grid は [`LAYOUT_CSS`] 側で担う）。
fn feature_list() -> Node {
    list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-single-split-features", "")],
        FEATURES.iter().copied().map(feature_item).collect(),
    )
}

/// 支払周期 radio card 1 件（`item`/`item_hidden_input`/`item_control`/
/// `item_indicator` の組み立て、モジュール doc「id / ARIA の方針」節）。
/// ネイティブ操作不能にするため常に `disabled: true` で描く（モジュール
/// doc「支払周期 radio card をネイティブ disabled にする理由」節）。
fn billing_item(checked: bool, value: &'static str, label: &'static str) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some(BILLING_NAME), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![radio_card::item_text(vec![], vec![text(label)])],
                    ),
                ],
            ),
        ],
    )
}

/// 支払周期選択欄（見出し + radio card 2 択。狭幅は縦積み・`48rem` 以上は
/// 横並び、モジュール doc「ブレークポイント」節）。年額を選んだ状態
/// （`checked: true`）で固定する静的表示。
fn billing_toggle() -> Node {
    div(
        vec![("class", "blocks-pricing-single-split-billing")],
        vec![
            radio_card::label(Some(BILLING_LABEL_ID), vec![], vec![text("お支払い周期")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<Orientation>,
                Some(BILLING_LABEL_ID),
                vec![],
                vec![
                    billing_item(false, "monthly", "月額払い"),
                    billing_item(true, "yearly", "年額払い（2 か月分お得）"),
                ],
            ),
        ],
    )
}

/// 左カラム（プラン名・説明・機能一覧。`with_billing` のときのみ末尾へ
/// 支払周期選択を追加する）。
fn main_column(with_billing: bool) -> Node {
    let mut children = vec![
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl2,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("スタンダードプラン")],
        ),
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(
                "必要な機能を過不足なく揃えた、成長中のチーム向けの単一プランです。",
            )],
        ),
        separator::separator(&SeparatorProps::default(), vec![]),
        heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![],
            vec![text("含まれる機能")],
        ),
        feature_list(),
    ];
    if with_billing {
        children.push(billing_toggle());
    }
    div(
        vec![("class", "blocks-pricing-single-split-main")],
        children,
    )
}

/// 右カラム（価格パネル）。`with_billing` の値に応じて年額表示へ切り替える
/// （支払周期選択の固定状態〔年額選択済み〕と一致させる）。
fn price_panel(with_billing: bool) -> Node {
    let (period_label, price_value, note) = if with_billing {
        (
            "年額払い",
            "¥320,000 / 年",
            "月あたり ¥26,667 相当（2 か月分お得な価格です）。",
        )
    } else {
        ("月額払い", "¥32,000 / 月", "契約期間の縛りはありません。")
    };
    card::root(
        CardProps {
            variant: CardVariant::Subtle,
            ..CardProps::default()
        },
        vec![("data-blocks-pricing-single-split-panel", "")],
        vec![card::body(
            vec![],
            vec![
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(period_label)],
                ),
                heading(
                    HeadingLevel::H3,
                    &HeadingProps {
                        size: HeadingSize::Xl4,
                        ..HeadingProps::default()
                    },
                    vec![],
                    vec![text(price_value)],
                ),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text("税込表示です。")],
                ),
                button::button(
                    &ButtonProps::default(),
                    vec![("data-blocks-pricing-single-split-cta", "")],
                    vec![text("このプランで始める")],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(note)],
                ),
            ],
        )],
    )
}

/// Demo 1 インスタンス分（本文カラム + 価格パネルの行、モジュール doc
/// 「Demo を 2 インスタンス並べる理由」節）。
fn plan_split(with_billing: bool) -> Node {
    let variant = if with_billing { "billing" } else { "standard" };
    div(
        vec![
            ("class", "blocks-pricing-single-split-row"),
            ("data-blocks-pricing-single-split-variant", variant),
        ],
        vec![main_column(with_billing), price_panel(with_billing)],
    )
}

/// `pricing-single-split` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。標準形（R1144）と支払周期選択付き（R0200）の 2 インスタンス
/// を縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-single-split-layout")],
        vec![plan_split(false), plan_split(true)],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-single-split/",
    title: "pricing-single-split",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_single_split.rs",
    demo_class: "blocks-pricing-single-split",
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
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_single_split` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、他 block と同型で本ファイル内 private
/// 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-pricing-single-split-*` と
/// `[data-blocks-pricing-single-split-*]`、および styled radio-card の
/// `[data-scope="radio-card"]` 系セレクタへの子孫結合子付き上書き（disabled
/// 中和、モジュール doc「支払周期 radio card をネイティブ disabled にする
/// 理由」節）のみを用い、他 block や部品の素のセレクタへ影響させない。
///
/// disabled 中和セレクタは
/// `[data-blocks-pricing-single-split-variant="billing"] [data-scope=
/// "radio-card"][data-part="item"][data-disabled]`（属性セレクタ 1 個の
/// 祖先 + 3 個の子孫、詳細度 (0,4,0)）を使う。`radio_card::stylesheet` の
/// `item` disabled 規則（`[data-scope="radio-card"][data-part="item"]
/// [data-disabled]`、属性セレクタ 3 個、詳細度 (0,3,0)）より詳細度が高い
/// ため、`opacity`/`cursor` を確実に上書きできる
/// （`contact_split_form_image`/`faq_accordion_centered` と同型の判断）。
/// `item` の `opacity: 1` により祖先の減光が解除されるため、`item-control`/
/// `item-indicator` 側で個別に中和する必要はない。
const LAYOUT_CSS: &str = "\
.blocks-pricing-single-split-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-pricing-single-split-row {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-pricing-single-split-main {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-pricing-single-split-billing {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-pricing-single-split-features] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-pricing-single-split-panel] [data-scope=\"card\"][data-part=\"body\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-pricing-single-split-panel] [data-scope=\"button\"] {\n  inline-size: 100%;\n}\n\
[data-blocks-pricing-single-split-variant=\"billing\"] [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 40rem) {\n  \
[data-blocks-pricing-single-split-features] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 48rem) {\n  \
.blocks-pricing-single-split-row {\n    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);\n    align-items: start;\n  }\n  \
[data-blocks-pricing-single-split-variant=\"billing\"] [data-scope=\"radio-card\"][data-part=\"root\"] {\n    flex-direction: row;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BILLING_LABEL_ID, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    /// Demo が期待する 8 種の部品・非対話制約を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\"",
            "data-scope=\"button\"",
            "data-scope=\"list\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(
            html.contains("<svg"),
            "demo should render icon svg elements"
        );
        assert_eq!(html.matches(r#"type="button""#).count(), 2);
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 支払周期選択付きインスタンスの radio がすべてネイティブ disabled
    /// であり、年額のみが選択済みであることを固定する（モジュール doc
    /// 「支払周期 radio card をネイティブ disabled にする理由」節）。標準形
    /// インスタンスには radio が一切現れないことも合わせて固定する。
    #[test]
    fn billing_variant_radio_is_natively_disabled_and_yearly_checked() {
        let html = demo_html();
        assert_eq!(html.matches(r#"type="radio""#).count(), 2);
        assert_eq!(html.matches(" disabled=\"\"").count(), 2);
        assert_eq!(html.matches(" checked").count(), 1);
        assert!(html.contains(r#"data-blocks-pricing-single-split-variant="standard""#));
        assert!(html.contains(r#"data-blocks-pricing-single-split-variant="billing""#));
    }

    /// [`radio_card::label`] の id が [`BILLING_LABEL_ID`] と一致し、
    /// `aria-labelledby` が同じ id を指すこと（宙に浮いた ARIA 参照が無い
    /// ことの個別固定）。
    #[test]
    fn billing_group_is_labelled_by_its_label() {
        let html = demo_html();
        let id_attr = format!("id=\"{BILLING_LABEL_ID}\"");
        let labelledby_attr = format!("aria-labelledby=\"{BILLING_LABEL_ID}\"");
        assert!(html.contains(&id_attr));
        assert!(html.contains(&labelledby_attr));
    }

    /// [`LAYOUT_CSS`] が狭幅で縦積み・`48rem` 以上で 2 カラム grid・機能
    /// 一覧の 2 列 grid・disabled 中和を持つことを固定する（モジュール doc
    /// 「ブレークポイント」節）。
    #[test]
    fn layout_css_stacks_on_narrow_and_neutralizes_disabled() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("cursor: default;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-pricing-single-split-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-pricing-single-split-layout"
        );
    }
}
