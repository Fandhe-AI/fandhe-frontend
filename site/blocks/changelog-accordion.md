# changelog-accordion

`heading` / `text` / `badge` / `accordion` / `list` / `image` の 6 部品を合成した、アコーディオン型の changelog レイアウトです。左寄せの見出し・リード文の下に、リリース単位で個別の枠に囲まれたアコーディオン項目を縦に並べます。

- 静的表示です。docs サイトは無 JS のため、全リリースを常時展開した状態で固定描画しています。項目のトリガーはネイティブ `disabled` 属性・`aria-disabled="true"` を持ち、実際に開閉操作をすることはできません（イシュー #2818 レビュー指摘の是正。当初は先頭 2 件のみ開いた状態にしていましたが、閉じた項目の本文が `hidden` 属性で到達不能になり、操作不能な `<button>` を残す形になっていたため、全件表示へ変更しました）。
- 文言・バージョン番号・日付・変更点はすべて架空のものです。
- データ取得・送信は行わず、`<form>` は使いません。
- 狭い幅でも 1 列のまま幅いっぱいに広がります（ブレークポイントによる段組み切り替えはありません）。
- 集約元は 1 件のみです（対応表 ID R0046）。取り込んだのは領域配置・部品構成・状態の見せ方といった構造のみで、文言・配色・装飾は取り込んでいません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

use crate::blocks::dummy_assets;

/// リリース 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Release {
    version: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    tags: &'static [&'static str],
    image_src: &'static str,
    changes: &'static [&'static str],
}

/// リリース一覧（架空、4 件。モジュール doc「静的表示」節のとおり全件を
/// open + disabled で固定描画する）。
const RELEASES: [Release; 4] = [
    Release {
        version: "v2.4.0",
        date_iso: "2026-09-18",
        date_label: "2026年9月18日",
        title: "アコーディオン型 changelog を追加",
        tags: &["新機能"],
        image_src: dummy_assets::SCREENSHOT_SRC,
        changes: &[
            "リリース単位で区切って表示する changelog レイアウトを追加",
            "全リリースを常時展開表示するデモ表示に対応",
        ],
    },
    Release {
        version: "v2.3.0",
        date_iso: "2026-09-10",
        date_label: "2026年9月10日",
        title: "ダミー素材ヘルパを共通化",
        tags: &["改善", "内部"],
        image_src: dummy_assets::PRODUCT_SRC,
        changes: &[
            "プレースホルダー画像・文言の生成をヘルパへ一元化",
            "block ごとの個別実装によるブレを解消",
        ],
    },
    Release {
        version: "v2.2.1",
        date_iso: "2026-09-02",
        date_label: "2026年9月2日",
        title: "画像スロットの表示崩れを修正",
        tags: &["修正"],
        image_src: dummy_assets::BACKGROUND_SRC,
        changes: &["狭い幅での画像の縦横比崩れを修正"],
    },
    Release {
        version: "v2.2.0",
        date_iso: "2026-08-20",
        date_label: "2026年8月20日",
        title: "変更点リストの表示を刷新",
        tags: &["改善"],
        image_src: dummy_assets::SCREENSHOT_SRC,
        changes: &[
            "変更点を種別ごとに読みやすく整理",
            "長い項目でも折り返しが崩れないよう調整",
        ],
    },
];

/// `<time datetime>` を組み立てる（機械可読な ISO 値と表示値は常に同じ日を
/// 指す組にする不変条件、`blog_list_image` と同じ判断）。
fn release_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-changelog-accordion-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// トリガーのラベル領域（version・日付・タイトルの 3 点。トリガーの
/// 直接の子は本要素と `item_indicator` の 2 個に保つ、モジュール doc
/// 「トリガーの子を 2 個に保つ理由」節）。
fn trigger_label(release: &Release) -> Node {
    fandhe_frontend_core::span(
        vec![("class", "blocks-changelog-accordion-trigger-label")],
        vec![
            fandhe_frontend_core::span(
                vec![("class", "blocks-changelog-accordion-version")],
                vec![text(release.version)],
            ),
            release_date(release.date_iso, release.date_label),
            fandhe_frontend_core::span(
                vec![("class", "blocks-changelog-accordion-title")],
                vec![text(release.title)],
            ),
        ],
    )
}

/// アコーディオン本文（タグ badge 群 + 画像 + 変更点リスト）。
fn release_body(release: &Release) -> Node {
    let tags = div(
        vec![("class", "blocks-changelog-accordion-tags")],
        release
            .tags
            .iter()
            .map(|tag| {
                badge::badge(
                    &BadgeProps::default(),
                    vec![("data-blocks-changelog-accordion-tag", "")],
                    vec![text(*tag)],
                )
            })
            .collect(),
    );

    let figure = div(
        vec![("class", "blocks-changelog-accordion-figure")],
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Rounded,
                ..ImageProps::new(release.image_src, "")
            },
            vec![("data-blocks-changelog-accordion-image", "")],
        )],
    );

    let changes = list::root(
        ListType::Unordered,
        ListVariant::Marker,
        vec![("data-blocks-changelog-accordion-changes", "")],
        release
            .changes
            .iter()
            .map(|change| list::item(vec![], vec![text(*change)]))
            .collect(),
    );

    div(
        vec![("class", "blocks-changelog-accordion-body")],
        vec![tags, figure, changes],
    )
}

/// リリース 1 件分の accordion item（トリガー + 本文）。
///
/// 全件を [`OpenState::Open`] + `disabled: true` で固定する（モジュール doc
/// 「静的表示」節）。`disabled` はネイティブ `disabled` 属性・
/// `aria-disabled="true"` を [`item_trigger`] へ反映させ、無 JS のため
/// クリック・キーボードでは開閉できないことを支援技術・キーボード操作の
/// 双方に明示する。
fn release_item(index: usize, release: &Release) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-changelog-accordion-{index}-trigger");
    let content_id = format!("blocks-changelog-accordion-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h4",
                vec![("class", "blocks-changelog-accordion-trigger-heading")],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    release.version,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        trigger_label(release),
                        item_indicator(state, false, &props, vec![], vec![text("▾")]),
                    ],
                )],
            ),
            item_content(
                state,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![release_body(release)],
            ),
        ],
    )
}

/// `changelog-accordion` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-changelog-accordion-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("更新履歴")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("各リリースの変更点をまとめています。")],
            ),
        ],
    );

    let items: Vec<Node> = RELEASES
        .iter()
        .enumerate()
        .map(|(index, release)| release_item(index, release))
        .collect();

    let list = div(
        vec![("class", "blocks-changelog-accordion-list")],
        vec![accordion::root(
            Size::Md,
            &AccordionProps::default(),
            vec![("data-blocks-changelog-accordion-root", "")],
            items,
        )],
    );

    div(
        vec![("class", "blocks-changelog-accordion-layout")],
        vec![header, list],
    )
}
```

## 原案差分メモ

- 見出しレベルを 1 段下げました（ページ側の `## Demo` に合わせるため、セクション見出しは `h3`）。
- トリガーから装飾的なバッジを外し、version・日付・タイトルの 3 点のみに絞りました。
- 全件を open + disabled に固定した静的表示にしました（JS の状態機械は再現していません。当初は先頭 2 件のみ open でしたが、無 JS では開閉できず閉じた項目の本文が到達不能になるため、全件表示へ変更しました）。
- アイコンを使わず、開閉インジケータは既存部品のテキスト「▾」にしました。
- 画像はモノトーンの共通ダミー素材にしました。
- 文言をすべて独自に書き直しました。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- 日付には機械可読な `datetime` 属性を付けています。
