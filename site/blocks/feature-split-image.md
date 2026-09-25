# feature-split-image

`badge` / `heading` / `text` / `button` / `image` / `blockquote` /
`avatar` / `icon` の 8 部品を合成した、テキスト列 + 4:3 画像列の 2 列
feature セクションです。基準形（badge + 見出し + 説明文 + ボタン 2 個 +
4:3 画像）に加え、説明文の下にチェック付き箇条書きを添える形（lg
（64rem）以上で画像を左へ入れ替える）、ボタン列の下に発言者付き引用を
添える形、2 列の下に 3 列の feature 一覧を続ける形の計 4 形を並べて
示します。

いずれの形も DOM 順は常にテキスト列 → 画像列で固定し、左右の入れ替えは
grid 配置だけで行います。文言はすべて架空のもので、データ取得・送信は
行わない静的な表示例です。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 自作の幾何アイコン（線画。モジュール doc「チェックリストのアイコン」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
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

/// チェックマークの幾何アイコン。
fn check_icon() -> Node {
    geo_icon("M5 13l4 4L19 7")
}

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
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

/// 形 B のチェック付き項目 4 件（架空文言）。
const CHECKLIST_ITEMS: [&str; 4] = [
    "既定エスケープ済みの HTML 出力",
    "外部依存ゼロの描画コア",
    "決定的なビルド成果物",
    "型で保証されたコンポーネント境界",
];

/// 形 D の feature 一覧 3 件（見出し + 説明）。
const MINI_FEATURES: [(&str, &str); 3] = [
    (
        "静的な表示のみ",
        "JS ハイドレーションを行わない、決定的な静的表示専用の構成です。",
    ),
    (
        "単一実行ファイル配布",
        "サーバー・アセットをひとまとめにし、Docker イメージ 1 枚で配布できます。",
    ),
    (
        "既定エスケープ",
        "テキスト補間は必ず既定エスケープを経由し、迂回経路を新設しません。",
    ),
];

/// チェック付き項目 1 件分（チェックマーク + 短い文言）。
fn checklist_item(label: &'static str) -> Node {
    li(
        vec![("class", "blocks-feature-split-image-checklist-item")],
        vec![check_icon(), text(label)],
    )
}

/// A/B/C 共通のボタン列（Solid + Outline の 2 個）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-feature-split-image-actions")],
        vec![
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("今すぐ試す")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("資料を見る")],
            ),
        ],
    )
}

/// テキスト列共通の先頭 3 要素（badge + 見出し + 説明文）。
fn copy_header(eyebrow: &'static str, title: &'static str, body: &'static str) -> Vec<Node> {
    vec![
        badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
        heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
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
            vec![text(body)],
        ),
    ]
}

/// 4:3 画像列（[`AspectRatio::Landscape`]・角丸・装飾扱いの `alt=""`）。
fn media(src: &'static str) -> Node {
    div(
        vec![("class", "blocks-feature-split-image-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Landscape,
                shape: ImageShape::Rounded,
                ..ImageProps::new(src, "")
            },
            vec![("data-blocks-feature-split-image-image", "")],
        )],
    )
}

/// 形 A（R0471 基準形）: テキスト列（badge + 見出し + 説明 + ボタン 2 個）+
/// 4:3 画像列。
fn variant_basic() -> Node {
    let copy = div(vec![("class", "blocks-feature-split-image-copy")], {
        let mut children = copy_header(
            "新機能",
            "テキストと画像を左右に並べて紹介する",
            "要点を短い説明文にまとめ、隣に画面を添えて価値を素早く伝えます。",
        );
        children.push(actions());
        children
    });

    div(
        vec![("class", "blocks-feature-split-image-row")],
        vec![copy, media(dummy_assets::PRODUCT_SRC)],
    )
}

/// 形 B（R0485 + R0482）: 説明文の下にチェック付き箇条書きを追加し、
/// lg 以上で画像を左へ入れ替える（DOM 順はテキストが先のまま）。
fn variant_checklist() -> Node {
    let checklist = ul(
        vec![("class", "blocks-feature-split-image-checklist")],
        CHECKLIST_ITEMS
            .iter()
            .map(|label| checklist_item(label))
            .collect(),
    );

    let copy = div(vec![("class", "blocks-feature-split-image-copy")], {
        let mut children = copy_header(
            "チェックリスト",
            "満たすべき要件を、その場で確認できる",
            "説明文だけでなく要点を列挙することで、導入前に必要な条件を一目で把握できます。",
        );
        children.push(checklist);
        children
    });

    div(
        vec![
            ("class", "blocks-feature-split-image-row"),
            ("data-blocks-feature-split-image-reverse", ""),
        ],
        vec![copy, media(dummy_assets::BACKGROUND_SRC)],
    )
}

/// 形 C（R0474 + R0940）: ボタン列の下に発言者付き引用を置き、画像には
/// 画面画像（[`dummy_assets::SCREENSHOT_SRC`]）を使う。
fn variant_testimonial() -> Node {
    let quote = blockquote::root(
        BlockquoteVariant::default(),
        ColorPalette::default(),
        vec![("data-blocks-feature-split-image-quote", "")],
        vec![
            blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
            blockquote::caption(
                vec![("class", "blocks-feature-split-image-meta")],
                vec![
                    avatar::root(
                        &AvatarProps::default(),
                        vec![("data-blocks-feature-split-image-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                    div(
                        vec![("class", "blocks-feature-split-image-byline")],
                        vec![
                            div(vec![], vec![text(dummy_assets::PERSON_NAMES[0])]),
                            div(vec![], vec![text(dummy_assets::JOB_TITLES[0])]),
                        ],
                    ),
                ],
            ),
        ],
    );

    let copy = div(vec![("class", "blocks-feature-split-image-copy")], {
        let mut children = copy_header(
            "導入事例",
            "実際に使うチームの声を添えて紹介する",
            "機能の説明だけでなく、利用しているチームの一言を添えることで説得力を高められます。",
        );
        children.push(actions());
        children.push(quote);
        children
    });

    div(
        vec![("class", "blocks-feature-split-image-row")],
        vec![copy, media(dummy_assets::SCREENSHOT_SRC)],
    )
}

/// 形 D（R0098）の feature 一覧 1 件分（幾何アイコン + 見出し + 説明）。
fn mini_feature((title, body): &(&'static str, &'static str)) -> Node {
    div(
        vec![("class", "blocks-feature-split-image-feature")],
        vec![
            geo_icon("M12 2l3 7h7l-5.5 4.5L18 21l-6-4-6 4 1.5-7.5L2 9h7z"),
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(*title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(*body)],
            ),
        ],
    )
}

/// 形 D（R0098）: 形 A と同型の 2 列（ボタン 2 個）の下に、3 列の feature
/// 一覧を続ける。
fn variant_with_features() -> Node {
    let copy = div(vec![("class", "blocks-feature-split-image-copy")], {
        let mut children = copy_header(
            "まとめて紹介",
            "2 列の下に、関連する特長を並べて続ける",
            "主要な訴求の直後に、補足となる特長を 3 列で並べて理解を後押しします。",
        );
        children.push(actions());
        children
    });

    let row = div(
        vec![("class", "blocks-feature-split-image-row")],
        vec![copy, media(dummy_assets::LOGO_SRC)],
    );

    let features = div(
        vec![("class", "blocks-feature-split-image-features")],
        MINI_FEATURES.iter().map(mini_feature).collect(),
    );

    div(
        vec![("class", "blocks-feature-split-image-with-features")],
        vec![row, features],
    )
}

/// `feature-split-image` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（モジュール doc「4 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-split-image-layout")],
        vec![
            variant_label("画像 1 枚 + テキスト列（対応表 ID R0471 基準形）"),
            variant_basic(),
            variant_label("チェックリスト + 左右入れ替え（対応表 ID R0485/R0482）"),
            variant_checklist(),
            variant_label("発言者付き引用（対応表 ID R0474/R0940）"),
            variant_testimonial(),
            variant_label("3 列 feature 一覧付き（対応表 ID R0098）"),
            variant_with_features(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0471 基準形・R0485・R0474・R0940・R0482・R0098。出典の
固有名・ファイル名は記載しません）からの意図的な差分は次のとおりです。

- R0471 の基準形はそのまま採用し、画像は 4:3（`AspectRatio::Landscape`）
  に固定しました。
- R0485 のチェック付き箇条書きは、独立した参照としてではなく基準形の
  説明文の下へ追加する差分として統合しました。
- R0482 の左右入れ替えは、チェック付き箇条書きを持つ形へ適用し、
  `nth-child` ではなく `data-blocks-feature-split-image-reverse` 属性で
  明示しました（DOM 順は常にテキスト → 画像のまま）。
- R0474 の発言者付き引用と R0940 の画面画像は 1 つの形へ統合し、引用は
  `blockquote` + `avatar`、画像は画面画像のプレースホルダーで表しました。
- R0098 の 3 列 feature 一覧は、基準形と同型の 2 列の下に続ける形として
  独立させました。
- 参照元の配色・装飾・アイコンは持ち込まず、幾何学的な自作アイコン
  （lucide 等の著作物を複製しない単純な折れ線）のみを使用しました。
- 見出しは 1 段下げて `h3`、feature 一覧の見出しは `h4` にしました
  （ページ側が `## Demo` として `h2` を出すため）。
- 画像は `dummy_assets` のプレースホルダー + `alt=""`（装飾扱い）を使用し、
  実在のブランド・人物・企業とは無関係の架空データです。
