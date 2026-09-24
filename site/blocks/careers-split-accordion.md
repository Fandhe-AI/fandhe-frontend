# careers-split-accordion

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` /
`accordion` / `icon` / `button` の 6 部品を合成した、見出し左 + 求人
アコーディオン一覧の合成例です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集である
ことに注意してください。

幅 md（48rem）以上では 2 カラム（左に見出し・説明、右に求人アコーディオン
一覧）になり、それより狭い幅では見出しの下にアコーディオンが縦に続きます。
アコーディオンの各項目トリガーには職種名と部署 badge を、本文には
説明文・勤務地/雇用形態のメタ行（アイコン付き）・応募ボタンを配置します。
docs サイトは JS ハイドレーションを行わず開閉を切り替える手段がないため、
4 件すべてを開いた状態で固定表示し、求人内容へ常時到達できるようにします。

本 Demo は静的な表示例であり、`<form>` 要素を持たず、送信処理・応募処理を
一切行いません。応募ボタンは `type="button"` のまま送信先を持たない静的な
ボタンです。実際の応募フローを実装する場合は、利用者自身の Rust コードで
書いてください（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界:
UI コンポーネント層はアプリケーションロジックを内包しません）。

参照元は対応表 ID R0034（集約元 1 件）です。構造だけを参照し、文言・配色・
装飾・アイコンは持ち込んでいません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{self, AccordionProps, OpenState};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 求人 1 件分の静的データ（架空の職種・部署・説明・勤務地・雇用形態）。
struct Job {
    dept: &'static str,
    title: &'static str,
    summary: &'static str,
    location: &'static str,
    employment: &'static str,
}

/// 4 件の求人定義。無 JS のため全件を開いた状態で Demo 表示する
/// （モジュール doc「無 JS のため全件展開で固定表示」参照）。
const JOBS: &[Job] = &[
    Job {
        dept: "エンジニアリング",
        title: "バックエンドエンジニア",
        summary: "サーバーサイド API の設計・実装・運用を担当します。",
        location: "リモート（日本国内）",
        employment: "正社員",
    },
    Job {
        dept: "デザイン",
        title: "プロダクトデザイナー",
        summary: "利用者体験の調査からビジュアルデザインまで一貫して担当します。",
        location: "東京オフィス",
        employment: "正社員",
    },
    Job {
        dept: "セールス",
        title: "フィールドセールス",
        summary: "既存顧客との関係構築と新規商談の推進を担当します。",
        location: "大阪オフィス",
        employment: "契約社員",
    },
    Job {
        dept: "カスタマーサポート",
        title: "サポートスペシャリスト",
        summary: "問い合わせ対応とナレッジベースの整備を担当します。",
        location: "リモート（日本国内）",
        employment: "業務委託",
    },
];

/// 自作の単純な線画アイコン（`d` は呼び出し側が座標を選ぶ、
/// `sidebar_03::geo_icon`/`footer_newsletter::checkmark_icon` と同型）。
/// lucide 等の既存アイコンセットの path は複製しない。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
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

/// シェブロン（下向き、アコーディオンの開閉インジケータ）。
fn chevron_icon() -> Node {
    geo_icon("M6 9l6 6 6-6")
}

/// ピン（勤務地メタ行）。
fn pin_icon() -> Node {
    geo_icon("M12 21s7-7.5 7-12a7 7 0 1 0-14 0c0 4.5 7 12 7 12zM12 11a2 2 0 1 0 0-4 2 2 0 0 0 0 4z")
}

/// 時計（雇用形態メタ行）。
fn clock_icon() -> Node {
    geo_icon("M12 7v5l3 3M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z")
}

/// 矢印（応募ボタン）。
fn arrow_icon() -> Node {
    geo_icon("M5 12h14M13 6l6 6-6 6")
}

/// メタ行 1 項目（アイコン + テキストの `span`）。
fn meta_item(item_icon: Node, label: &str) -> Node {
    span(
        vec![("data-blocks-careers-split-accordion-meta-item", "")],
        vec![item_icon, text(label)],
    )
}

/// 求人 1 件分の `item`（トリガー + 本文）を組み立てる。
fn job_item(index: usize, job: &Job, accordion_props: &AccordionProps) -> Node {
    // 無 JS のため開閉を切り替える手段がなく、`OpenState::Closed` にすると
    // `item_content` に `hidden` が付与され本文（勤務地・雇用形態・応募
    // ボタン）へ到達不能になる（P1 指摘、イシュー #2816）。全件を
    // `OpenState::Open` に固定し、求人内容を常時閲覧・操作可能にする。
    let state = OpenState::Open;
    let value = format!("job-{}", index + 1);
    let trigger_id = format!("blocks-careers-split-accordion-job-{}-trigger", index + 1);
    let content_id = format!("blocks-careers-split-accordion-job-{}-content", index + 1);
    let apply_label = format!("応募する（{}）", job.title);

    let trigger_label = span(
        vec![("class", "blocks-careers-split-accordion-trigger-label")],
        vec![
            span(vec![], vec![text(job.title)]),
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    palette: ColorPalette::Neutral,
                    ..BadgeProps::default()
                },
                vec![("data-blocks-careers-split-accordion-dept", "")],
                vec![text(job.dept)],
            ),
        ],
    );

    let trigger = el(
        "h4",
        vec![("class", "blocks-careers-split-accordion-item-heading")],
        vec![accordion::item_trigger(
            state,
            false,
            accordion_props,
            value.as_str(),
            Some(trigger_id.as_str()),
            Some(content_id.as_str()),
            vec![],
            vec![
                trigger_label,
                accordion::item_indicator(
                    state,
                    false,
                    accordion_props,
                    vec![],
                    vec![chevron_icon()],
                ),
            ],
        )],
    );

    let body = div(
        vec![("class", "blocks-careers-split-accordion-body")],
        vec![
            styled_text::text(&TextProps::default(), vec![], vec![text(job.summary)]),
            div(
                vec![("class", "blocks-careers-split-accordion-meta")],
                vec![
                    meta_item(pin_icon(), job.location),
                    meta_item(clock_icon(), job.employment),
                ],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![
                    ("data-blocks-careers-split-accordion-apply", ""),
                    ("aria-label", apply_label.as_str()),
                ],
                vec![span(vec![], vec![text("応募する")]), arrow_icon()],
            ),
        ],
    );

    let content = accordion::item_content(
        state,
        false,
        accordion_props,
        Some(content_id.as_str()),
        Some(trigger_id.as_str()),
        vec![],
        vec![body],
    );

    accordion::item(
        state,
        false,
        accordion_props,
        vec![],
        vec![trigger, content],
    )
}

/// `careers-split-accordion` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let accordion_props = AccordionProps::default();

    let header = div(
        vec![("class", "blocks-careers-split-accordion-header")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-careers-split-accordion-tagline", "")],
                vec![text("採用情報")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("一緒にチームを育てる仲間を募集しています")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "募集中のポジションから、興味のある職種をご覧ください。",
                )],
            ),
        ],
    );

    let items: Vec<Node> = JOBS
        .iter()
        .enumerate()
        .map(|(index, job)| job_item(index, job, &accordion_props))
        .collect();

    let list = accordion::root(
        Size::Md,
        &accordion_props,
        vec![("data-blocks-careers-split-accordion-list", "")],
        items,
    );

    div(
        vec![("class", "blocks-careers-split-accordion-layout")],
        vec![header, list],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0034。出典の固有名・ファイル名は記載しません）からの
意図的な差分は次のとおりです。

- ブレークポイントを md（48rem）へ変更し、`grid-template-columns` による
  シンプルな 2 カラム切り替え（左固定幅・右 2 倍幅）にしました。
- アコーディオンの見出し（トリガー）ラッパーには `h3` ではなく `h4` を
  使いました（ページ側目次収集が h2/h3 のみを対象とするため、求人名が
  目次へ混入するのを避けています）。
- アイコンは既存アイコンセットの複製ではなく、シェブロン/ピン/時計/矢印の
  4 種を自作の単純な線画（`icon` + `path`）として描きました。
- 求人件数は 4 件に固定し、文言（職種名・部署・説明・勤務地・雇用形態）は
  すべて独自の架空のものへ書き直しました。
- 開閉状態は無 JS のため切り替えられません。当初は先頭 1 件のみ開いた
  状態で固定していましたが、閉状態の本文（求人説明・勤務地・雇用形態・
  応募ボタン）へ閲覧者が到達できなくなるため、4 件すべてを開いた状態で
  固定表示するよう変更しました。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Accordion](../themes/accordion.md) /
[Icon](../themes/icon.md) / [Button](../themes/button.md)
