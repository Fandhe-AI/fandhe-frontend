# onboarding-vertical-steps

左に縦向きのステップ一覧、右に現在ステップの内容（動画枠 + 見出し + 説明文 +
前へ・次へ操作）を並べる 2 カラムのオンボーディングブロックです。
`steps` / `button` / `image` / `heading` / `text` の 5 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0178（代表構成）1 件のみで、集約元はありません。
ステップの題名・見出し・説明文はすべて架空のデータであり、実在の企業・PII
は含みません。動画ソース資産は持たないため、動画枠はビルド時生成の同梱
プレースホルダー画像（16:9）+ 装飾的な「動画を再生」ボタンで代替します。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::steps::Steps;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::steps;
use fandhe_frontend_pre_styled_ui::text::{text as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::Orientation;

/// ステップ 4 件分の題名（[`step_list`] の trigger ラベルに使う架空
/// ダミー）。
const STEP_TITLES: [&str; 4] = [
    "ワークスペースを作成",
    "メンバーを招待",
    "最初のプロジェクトを設定",
    "通知を確認",
];

/// 現在ステップ（step 2）の見出し。[`STEP_TITLES`][2] と対応させる。
const CURRENT_STEP_HEADING: &str = "最初のプロジェクトを設定";

/// 現在ステップ（step 2）の説明文。
const CURRENT_STEP_DESCRIPTION: &str =
    "テンプレートを選び、チームで使う最初のプロジェクトを数分で立ち上げます。\
     動画の手順に沿って進めると、設定は自動的に保存されます。";

/// 縦向きステップ一覧（左カラム）。`showcase::steps_demo` と同型に
/// `item` → `trigger`（`indicator` に番号 + 題名 `text`）+ 末尾以外に
/// `separator` を並べる。
fn step_list(s: &Steps) -> Node {
    let mut items = Vec::new();
    for (index, title) in STEP_TITLES.iter().enumerate() {
        let mut item_children = vec![steps::trigger(
            s,
            index,
            vec![],
            vec![
                steps::indicator(s, index, vec![], vec![core_text((index + 1).to_string())]),
                core_text(*title),
            ],
        )];
        if index + 1 < STEP_TITLES.len() {
            item_children.push(steps::separator(s, index, vec![], vec![]));
        }
        items.push(steps::item(s, index, vec![], item_children));
    }
    steps::list(s, vec![], items)
}

/// 動画枠（16:9 の静止画プレビュー + 装飾的な再生ボタン、モジュール doc
/// 「動画は 16:9 の静止画枠 + 再生ボタンで代替する」節参照）。
fn media_frame() -> Node {
    div(
        vec![("class", "blocks-onboarding-vertical-steps-media")],
        vec![
            image(
                &ImageProps {
                    src: dummy_assets::SCREENSHOT_SRC,
                    alt: "手順動画のプレビュー（静止画）",
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Video,
                    shape: ImageShape::Rounded,
                },
                vec![("data-blocks-onboarding-vertical-steps-video", "")],
            ),
            button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-onboarding-vertical-steps-play", "")],
                vec![core_text("動画を再生")],
            ),
        ],
    )
}

/// 現在ステップ（step 2）の内容（動画枠 + 見出し + 説明文）。
fn current_content(s: &Steps) -> Node {
    steps::content(
        s,
        2,
        vec![],
        vec![
            media_frame(),
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![core_text(CURRENT_STEP_HEADING)],
            ),
            styled_text(
                &TextProps::default(),
                vec![],
                vec![core_text(CURRENT_STEP_DESCRIPTION)],
            ),
        ],
    )
}

/// 前へ・次へ（右カラム下部、モジュール doc「前へ/次へを右カラムに置く
/// 理由」節参照）。
fn nav(s: &Steps) -> Node {
    div(
        vec![("class", "blocks-onboarding-vertical-steps-nav")],
        vec![
            steps::prev_trigger(s, vec![], vec![core_text("前へ")]),
            steps::next_trigger(s, vec![], vec![core_text("次へ")]),
        ],
    )
}

/// `onboarding-vertical-steps` の Demo 本体（呼び出しごとに同一の `Node`
/// を返す純関数。状態は `Steps::new(4, 2, Orientation::Vertical)` 固定、
/// モジュール doc「状態は固定」節参照）。
pub fn demo() -> Node {
    let s = Steps::new(4, 2, Orientation::Vertical);
    let body = steps::body(vec![], vec![current_content(&s), nav(&s)]);
    let root = steps::root(
        Size::Md,
        ColorPalette::Accent,
        &s,
        vec![("data-blocks-onboarding-vertical-steps-root", "")],
        vec![step_list(&s), body],
    );
    div(
        vec![("class", "blocks-onboarding-vertical-steps-stack")],
        vec![root],
    )
}
```

## 原案差分メモ

- 参照構成（R0178）は単一版であり、集約元はありません。
- 参照構成は前へ・次へをステップ一覧側（左）に描きますが、本 block は
  `steps` 部品の縦向きレイアウト契約（`root` の直下は `list` と `body` の
  2 要素のみ、`fandhe-frontend-pre-styled-ui` の `steps::root`/`steps::body`
  rustdoc 参照）を優先し、前へ・次へを右カラム（`body` 側、内容の下）へ
  配置しています。
- 動画は 16:9 のプレースホルダー静止画枠 + 装飾的な「動画を再生」ボタンで
  代替しています。動画ソース資産を持たず、無 JS の docs サイトへ `<video>`
  の再生 UI は持ち込みません。
- 狭幅（コンテナ幅 40rem 未満）ではステップ一覧が内容の上に横並びで表示され、
  ステップ間の接続線（separator）は非表示になります（`@container` による
  コンテナクエリ判定）。

関連情報: [Steps](../themes/steps.md) / [Button](../themes/button.md) /
[Image](../themes/image.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md)
