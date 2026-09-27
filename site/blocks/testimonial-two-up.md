# testimonial-two-up

`heading` / `text` / `icon` / `blockquote` / `avatar` / `separator` の合成
例（既存部品のみで組んだ、2 件の推薦文を左右に並べるセクションです）。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R1363。集約元は同形の R0725 と、上部に見出しを追加した R0728 の 2 件です。
出典の固有名・ファイル名は記載しません）。

「見出しなし・2 件並び」と「見出し付き・2 件並び」の 2 形を併記します。
各列はロゴ・引用文・著者（アバター・氏名・役職）の順に並び、著者行は
引用文の長さが列ごとに異なっても列の下端で揃います。列の間には区切り線を
表示し、`48rem` 未満では縦積み + 横向きの区切り線、`48rem` 以上では
2 列 + 縦向きの区切り線へ切り替わります（区切り線は横向き・縦向きを
それぞれ 1 個ずつ出力し表示/非表示を切り替えるため、表示中の向きと
`aria-orientation` が常に一致します）。本 Demo は静的な表示例であり、
`<form>` 要素を一切持たず、送信処理・データ取得を行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::ColorPalette;
use fandhe_frontend_pre_styled_ui::Orientation;
use fandhe_frontend_pre_styled_ui::Size;

const LOGO_ATTR: &str = "data-blocks-testimonial-two-up-logo";
const QUOTE_ATTR: &str = "data-blocks-testimonial-two-up-quote";
const AVATAR_ATTR: &str = "data-blocks-testimonial-two-up-avatar";
const DIVIDER_ATTR: &str = "data-blocks-testimonial-two-up-divider";

/// 抽象図形ロゴ（列ごとに異なる幾何形状。実在の企業ロゴを模さない）。
fn logo_icon(company: &str, path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Xl,
            label: Some(company),
            ..IconProps::default()
        },
        vec![(LOGO_ATTR, "")],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// 著者行（アバター + 氏名 + 役職）。
fn author(index: usize) -> Node {
    div(
        vec![("class", "blocks-testimonial-two-up-author")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![(AVATAR_ATTR, "")],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    "",
                    vec![],
                )],
            ),
            div(
                vec![],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(dummy_assets::PERSON_NAMES[index])],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(
                            dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 列 1 個（ロゴ → 引用文 → 著者行）。
fn column(index: usize, company: &str, logo_path_d: &'static str) -> Node {
    div(
        vec![("class", "blocks-testimonial-two-up-column")],
        vec![
            logo_icon(company, logo_path_d),
            blockquote::root(
                BlockquoteVariant::Plain,
                ColorPalette::default(),
                vec![(QUOTE_ATTR, "")],
                vec![
                    blockquote::content(
                        vec![],
                        vec![text(dummy_assets::TESTIMONIAL_QUOTES[index])],
                    ),
                    blockquote::caption(vec![], vec![author(index)]),
                ],
            ),
        ],
    )
}

/// 2 件並びの本体（列 2 個 + 区切り線 2 個）。
fn pair(first: usize, second: usize) -> Node {
    div(
        vec![("class", "blocks-testimonial-two-up-root")],
        vec![
            column(
                first,
                dummy_assets::COMPANY_NAMES[first % dummy_assets::COMPANY_NAMES.len()],
                "M12 2L21 7V17L12 22L3 17V7Z",
            ),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Horizontal,
                    ..SeparatorProps::default()
                },
                vec![(DIVIDER_ATTR, "horizontal")],
            ),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Vertical,
                    ..SeparatorProps::default()
                },
                vec![(DIVIDER_ATTR, "vertical")],
            ),
            column(
                second,
                dummy_assets::COMPANY_NAMES[second % dummy_assets::COMPANY_NAMES.len()],
                "M12 2L22 12L12 22L2 12Z",
            ),
        ],
    )
}

/// caption（並記された各形の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-testimonial-two-up-caption")],
        vec![text(label)],
    )
}

/// 見出しなし・2 件並び（主参照 R1363 / 集約元 R0725）。
fn instance_plain() -> Node {
    pair(0, 1)
}

/// 見出し付き・2 件並び（集約元 R0728）。
fn instance_with_heading() -> Node {
    div(
        vec![],
        vec![
            div(
                vec![("class", "blocks-testimonial-two-up-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("利用者の声")],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("実際に使い続けているお客様からいただいた感想です。")],
                    ),
                ],
            ),
            pair(2, 3),
        ],
    )
}

/// `testimonial-two-up` の Demo 本体（2 形併記）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-two-up-stack")],
        vec![
            caption("見出しなし・2 件並び"),
            instance_plain(),
            caption("見出し付き・2 件並び"),
            instance_with_heading(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R1363、主参照）からの意図的な差分は次のとおりです。

- 集約元 R0725 は主参照と同形のため、「見出しなし・2 件並び」の形として
  そのまま反映しました。
- 集約元 R0728 は上部に見出しを追加した形のため、「見出し付き・2 件並び」
  の形として反映しました。
- 参照元の配色・実ロゴ・実写真・実文言は持ち込まず、`--fandhe-*` トークン・
  抽象図形ロゴ・共通ダミー素材のアバター・架空の引用文/人名/役職/社名へ
  置き換えました。
- 参照元の狭幅レイアウトは列の入れ替えのみですが、本 block は区切り線の
  向きも横 → 縦へ切り替える構成にしています（区切り線 2 個の表示/非表示
  切替、モジュール doc「区切り線を横 / 縦の 2 個で切り替える理由」節参照）。
