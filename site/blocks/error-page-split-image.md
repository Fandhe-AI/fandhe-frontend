# error-page-split-image

`empty-state` / `heading` / `text` / `link` / `image` / `icon` の合成例
（既存部品のみで組んだ、本文 + 画像の 2 カラム 404 ページです）。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R1104、副次的に対応表 ID R0582 も参照します。出典の固有名・ファイル名は
記載しません）。

左カラムはロゴ・エラーコード・見出し・説明文・戻る導線を左寄せで並べ、
下端にはサポート等の補助リンクを横並びで置きます。右カラムには全高の
画像を配置しますが、これは `lg`（幅 64rem 以上）のときだけで、狭い幅では
画像を隠し左カラムだけを表示します。本 Demo は静的な表示例であり
`<form>` 要素・送信処理は一切持ちませんが、「Back to home」「Contact
support」「Help center」「CI status」の各リンクはいずれもラベルの
意味に対応した実在 URL へ実際に遷移します（死リンク `href="#"` は使い
ません）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextVariant, TextWeight,
};

/// 「Contact support」の遷移先（モジュール doc「補助リンクのリンク先は
/// ラベルの意味に対応した実在 URL」節参照。`error_page_centered` の
/// `ISSUES` と同じ URL・同じ判断）。
const ISSUES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/issues";

/// 「CI status」（表示ラベルを「System status」から変更、同節参照）の
/// 遷移先。GitHub Actions 実行状況ページ。
const CI_STATUS_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend/actions";

const LAYOUT_CLASS: &str = "blocks-error-page-split-image-layout";
const MAIN_CLASS: &str = "blocks-error-page-split-image-main";
const BRAND_CLASS: &str = "blocks-error-page-split-image-brand";
const HELPER_CLASS: &str = "blocks-error-page-split-image-helper";
const MEDIA_CLASS: &str = "blocks-error-page-split-image-media";

const LOGO_ATTR: &str = "data-blocks-error-page-split-image-logo";
const BRAND_NAME_ATTR: &str = "data-blocks-error-page-split-image-brand-name";
const MESSAGE_ATTR: &str = "data-blocks-error-page-split-image-message";
const CONTENT_ATTR: &str = "data-blocks-error-page-split-image-content";
const CODE_ATTR: &str = "data-blocks-error-page-split-image-code";
const TITLE_ATTR: &str = "data-blocks-error-page-split-image-title";
const DESCRIPTION_ATTR: &str = "data-blocks-error-page-split-image-description";
const ACTIONS_ATTR: &str = "data-blocks-error-page-split-image-actions";
const BACK_ATTR: &str = "data-blocks-error-page-split-image-back";
const CTA_ATTR: &str = "data-blocks-error-page-split-image-cta";
const HELPER_LINK_ATTR: &str = "data-blocks-error-page-split-image-helper-link";
const IMAGE_ATTR: &str = "data-blocks-error-page-split-image-image";

/// 装飾用の抽象六角形ロゴ（実ブランドロゴを複製しない、モジュール doc
/// 「ロゴ・戻る矢印は自作の単純幾何図形」参照）。
fn logo_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![(LOGO_ATTR, "")],
        vec![el(
            "path",
            vec![("d", "M12 2 21 7v10l-9 5-9-5V7l9-5Z")],
            vec![],
        )],
    )
}

/// 戻るリンクの左向き矢印（同上）。
fn back_arrow_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", "M14 4 6 12l8 8V4Z")], vec![])],
    )
}

/// `error-page-split-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let brand = div(
        vec![("class", BRAND_CLASS)],
        vec![
            logo_icon(),
            styled_text::text(
                &TextProps {
                    weight: TextWeight::Semibold,
                    ..TextProps::default()
                },
                vec![(BRAND_NAME_ATTR, "")],
                vec![text(dummy_assets::COMPANY_NAMES[0])],
            ),
        ],
    );

    let code = styled_text::text(
        &TextProps {
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![(CODE_ATTR, "")],
        vec![text("404")],
    );

    let title = empty_state::title(
        vec![(TITLE_ATTR, "")],
        vec![heading::heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                ..HeadingProps::default()
            },
            vec![],
            vec![text("This page took a wrong turn")],
        )],
    );

    let description = empty_state::description(
        vec![(DESCRIPTION_ATTR, "")],
        vec![styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(
                "We could not find the page you were looking for. It may have moved, been renamed, or never existed.",
            )],
        )],
    );

    let actions = empty_state::actions(
        vec![(ACTIONS_ATTR, "")],
        vec![
            link::root(
                "../../",
                &LinkProps::default(),
                vec![(BACK_ATTR, "")],
                vec![back_arrow_icon(), text(" Back to home")],
            ),
            link::root(
                ISSUES,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(CTA_ATTR, "")],
                vec![text("Contact support")],
            ),
        ],
    );

    let message = empty_state::root(
        &EmptyStateProps {
            variant: EmptyStateVariant::Plain,
            ..EmptyStateProps::default()
        },
        vec![(MESSAGE_ATTR, "")],
        vec![empty_state::content(
            vec![(CONTENT_ATTR, "")],
            vec![code, title, description, actions],
        )],
    );

    let helper = div(
        vec![("class", HELPER_CLASS)],
        vec![
            link::root(
                "../../guides/",
                &LinkProps::default(),
                vec![(HELPER_LINK_ATTR, "")],
                vec![text("Help center")],
            ),
            link::root(
                CI_STATUS_URL,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(HELPER_LINK_ATTR, "")],
                vec![text("CI status")],
            ),
        ],
    );

    let main_column = div(vec![("class", MAIN_CLASS)], vec![brand, message, helper]);

    let media_column = div(
        vec![("class", MEDIA_CLASS)],
        vec![image::image(
            &ImageProps::new(dummy_assets::SCREENSHOT_SRC, ""),
            vec![(IMAGE_ATTR, "")],
        )],
    );

    div(
        vec![("class", LAYOUT_CLASS)],
        vec![main_column, media_column],
    )
}
```

**原案差分メモ**

参照（主参照は対応表 ID R1104、副次的に対応表 ID R0582。出典の固有名・
ファイル名は記載しません）からの意図的な差分は次のとおりです。

- R0582 が持つ「CTA 2 個」を、actions 行の 2 つ目の導線（「Contact
  support」）として畳み込みました。右画像は R1104 と R0582 の両方に
  共通するため、1 つにまとめています。当初「Contact support」は
  `ButtonVariant::Outline` のボタンとして実装していましたが、404 ページの
  主要導線であるにもかかわらず遷移先を持たない非対話要素になっていた
  ため、`link::root` へ変更し実在する URL への遷移を持たせています
  （レビュー指摘、イシュー #2841）。
- 参照元は全画面の高さで表示していますが、本 block は Blocks セクションの
  Demo 枠内に収める必要があるため、`min-height` による枠内表示へ変更して
  います。
- ロゴは実在ブランドを模さない抽象的な六角形の自作アイコンとし、社名は
  共通ダミー素材ヘルパの架空の社名を使っています。
- 補助リンクの区切り点は DOM を増やさず CSS の疑似要素で描いています。
  リンク先は当初 3 リンクとも同一の固定リポジトリ URL でしたが、ラベルと
  遷移先が一致せず利用者が期待する情報に到達できないとの指摘（Bugbot/
  codex、イシュー #2841）を受け、Contact support は GitHub の Issues
  ページ（`error-page-centered` と同じ判断。姉妹 block は既に是正済み
  だった）、Help center は docs サイト内の `/guides/`、System status は
  表示ラベルを「CI status」へ改め遷移先はリポジトリの GitHub Actions
  実行状況ページのまま、それぞれラベルの意味に対応する実在 URL を
  割り当てています。
- 右カラムの画像は grid の `align-items: stretch` だけに頼らず、
  `position: absolute` + `object-fit: cover` で親いっぱいに敷き詰める
  アウトオブフロー構成にしています（画像が元のアスペクト比のまま
  縮小表示され余白ができる不具合の是正、イシュー #2841）。
- 配色はすべてテーマトークンのみで構成しており、参照元の文言・配色・
  写真は持ち込んでいません。
