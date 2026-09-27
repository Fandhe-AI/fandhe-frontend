# testimonial-split-image

`fandhe-frontend-pre-styled-ui` の `blockquote` / `image` / `avatar` /
`rating-group` / `button` / `link` / `icon` / `separator` 部品を合成した、
人物写真と推薦文を横並びにするセクションです。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集で
あることに注意してください（主参照は対応表 ID R1360、集約元は R0356・
R0357・R0362・R0726・R0729・R1361。出典の固有名・ファイル名は記載しません）。

md（`48rem`）以上では片側に人物写真、反対側に引用文・著者情報・任意の
ロゴ/リンクを置く 2 カラム構成です。狭い幅では写真（または著者列）→
引用文の順に縦積みになります。3 つの構成（縦長写真 + 星評価 + CTA ボタン /
暗色帯からはみ出す写真 / 写真の代わりに著者列）を静的インスタンスとして
並べています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。CTA ボタンは `type="button"` のまま送信先を
持ちません。文言・氏名・社名はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。ロゴは実在ブランドを模さない自作の
幾何図形です。星評価は他ユーザーの平均評価を表す readonly の静的表示です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Orientation, Size};

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// [`REPO`] の accessible name（遷移先と食い違わない固定文字列）。
const REPO_LABEL: &str = "fandhe-frontend の GitHub リポジトリ";

/// 抽象図形ロゴ（六角形の輪郭。実在の企業ロゴを模さない、
/// `testimonial_background_image` と同じ形状）。
fn logo_icon(company: &str) -> Node {
    icon(
        &IconProps {
            size: Size::Lg,
            label: Some(company),
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![("d", "M12 2L21 7V17L12 22L3 17V7Z")],
            vec![],
        )],
    )
}

/// 星評価（readonly。他ユーザーの平均評価を表す静的表示、
/// `hero_social_proof.rs::rating` と同型）。`label_id` は variant ごとに
/// 一意化する（モジュール doc「星評価は readonly」節）。
fn rating(label_id: &str) -> Node {
    let g = RatingGroup::new(5, Some(5), true);
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let label = rating_group::label(
        &props,
        Some(label_id),
        vec![],
        vec![visually_hidden::root(vec![], vec![text("Average rating")])],
    );
    let items: Vec<Node> = (1..=g.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: g.is_checked(i),
                    highlighted: g.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(label_id), vec![], items);
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![label, control],
    )
}

/// 引用文 + 著者（氏名・役職）を持つ [`blockquote::root`]。
fn quote_block(quote_index: usize, person_index: usize) -> Node {
    blockquote::root(
        BlockquoteVariant::Plain,
        ColorPalette::default(),
        vec![],
        vec![
            blockquote::content(
                vec![],
                vec![text(dummy_assets::TESTIMONIAL_QUOTES[quote_index])],
            ),
            blockquote::caption(
                vec![],
                vec![
                    div(vec![], vec![text(dummy_assets::PERSON_NAMES[person_index])]),
                    div(
                        vec![],
                        vec![text(
                            dummy_assets::JOB_TITLES[person_index % dummy_assets::JOB_TITLES.len()],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// `portrait` variant（主参照 R1360。縦長写真 + ロゴ + 星評価 + 引用文 + CTA）。
fn instance_portrait() -> Node {
    let photo = image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Portrait,
            ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
        },
        vec![("data-blocks-testimonial-split-image-photo", "")],
    );
    let content = div(
        vec![("class", "blocks-testimonial-split-image-content")],
        vec![
            logo_icon(dummy_assets::COMPANY_NAMES[0]),
            rating("blocks-testimonial-split-image-rating-portrait"),
            quote_block(0, 0),
            button::button(&ButtonProps::default(), vec![], vec![text("詳しく見る")]),
        ],
    );
    div(
        vec![
            ("class", "blocks-testimonial-split-image-layout"),
            ("data-blocks-testimonial-split-image-variant", "portrait"),
        ],
        vec![photo, content],
    )
}

/// `band` variant（集約元 R1361・R0356。暗色帯からはみ出す写真 + 引用文 +
/// ロゴ + 外部リンク）。
fn instance_band() -> Node {
    let photo = image::image(
        &ImageProps {
            aspect_ratio: AspectRatio::Portrait,
            ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
        },
        vec![("data-blocks-testimonial-split-image-photo", "")],
    );
    let content = div(
        vec![("class", "blocks-testimonial-split-image-content")],
        vec![
            logo_icon(dummy_assets::COMPANY_NAMES[1]),
            quote_block(1, 1),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text(REPO_LABEL)],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-testimonial-split-image-layout"),
            ("data-blocks-testimonial-split-image-band", ""),
            ("data-blocks-testimonial-split-image-variant", "band"),
        ],
        vec![photo, content],
    )
}

/// `author-column` variant（集約元 R0357・R0362。写真の代わりに円形
/// アバター + 縦罫線の著者列 + 星評価 + 引用文）。
fn instance_author_column() -> Node {
    let author_column = div(
        vec![("class", "blocks-testimonial-split-image-author")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xl,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    "",
                    vec![],
                )],
            ),
            div(vec![], vec![text(dummy_assets::PERSON_NAMES[2])]),
            div(
                vec![],
                vec![text(
                    dummy_assets::JOB_TITLES[2 % dummy_assets::JOB_TITLES.len()],
                )],
            ),
        ],
    );
    // `separator::separator` の垂直バリアントは `align-self: stretch` で高さを
    // 得る契約（`fandhe_frontend_pre_styled_ui::separator` 参照）。`.author`
    // （column flex）の子に置くと stretch が横方向に働き高さ 0 になるため、
    // `.layout` 直下（md 以上で row flex）の兄弟として置き、row 親の高さへ
    // 正しく stretch させる（Bugbot 指摘、PR #3316）。
    let sep = separator::separator(
        &SeparatorProps {
            orientation: Orientation::Vertical,
            ..SeparatorProps::default()
        },
        vec![("data-blocks-testimonial-split-image-separator", "")],
    );
    let content = div(
        vec![("class", "blocks-testimonial-split-image-content")],
        vec![
            logo_icon(dummy_assets::COMPANY_NAMES[2]),
            rating("blocks-testimonial-split-image-rating-author"),
            quote_block(2, 2),
        ],
    );
    div(
        vec![
            ("class", "blocks-testimonial-split-image-layout"),
            (
                "data-blocks-testimonial-split-image-variant",
                "author-column",
            ),
        ],
        vec![author_column, sep, content],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-testimonial-split-image-caption")],
        vec![text(label)],
    )
}

/// `testimonial-split-image` の Demo 本体（3 variant 併記）。呼び出しごとに
/// 同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-split-image-stack")],
        vec![
            caption("縦長写真・ロゴ・星評価・CTA"),
            instance_portrait(),
            caption("暗色帯からはみ出す写真"),
            instance_band(),
            caption("写真の代わりに著者列"),
            instance_author_column(),
        ],
    )
}
```

## 原案差分メモ

- 主参照（対応表 ID R1360、大きな縦長写真）は `portrait` インスタンスに
  対応します。縦長写真の隣に抽象図形ロゴ・星評価（readonly）・引用文・
  CTA ボタンを縦に並べています。
- 集約元（対応表 ID R1361・R0356、暗色帯からはみ出す写真）は `band`
  インスタンスに対応します。CSS の `::before` で暗色帯を敷き、md 以上では
  写真を `order` で右側へ回して帯からはみ出す配置にしています（DOM 順・
  読み上げ順は変えていません）。引用文の下には実在の自リポジトリ URL への
  外部リンクを置いています。
- 集約元（対応表 ID R0357・R0362、写真の代わりに著者列）は
  `author-column` インスタンスに対応します。大きな写真の代わりに円形
  アバター・氏名・役職を縦に並べ、md 以上でのみ表示する縦罫線で本文と
  区切っています。
- 集約元（対応表 ID R0726・R0729）は基本レイアウト（写真と本文の 2 カラム、
  狭幅での縦積み）の参照として `portrait`/`band` インスタンスに反映して
  おり、独立インスタンス化はしていません。
- 写真は隣に氏名見出しが常に出力されるため `alt=""`（装飾扱い）にして
  います。ロゴは装飾用の自作幾何図形（六角形。実在ブランドを模さない）で
  `aria-label` を持つ意味のある画像として扱っています。
- 文言・氏名・社名・ロゴ図形はすべて独自に書いた架空のものです。参照元の
  文言・配色・アイコン形状は持ち込んでいません。

関連情報: [Blockquote](../themes/blockquote.md) / [Image](../themes/image.md) /
[Avatar](../themes/avatar.md) / [Rating Group](../themes/rating-group.md) /
[Button](../themes/button.md) / [Link](../themes/link.md) /
[Icon](../themes/icon.md) / [Separator](../themes/separator.md)
