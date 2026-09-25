//! `error-page-split-image` block（イシュー #2841。親トラッキング #2807
//! 「Blocks 目的別パーツ拡充ツリー Phase 2、マーケティング B」配下、
//! `crate::blocks::marketing::error_page` カテゴリ 2 件目の block。
//! `error_page_background_image`（全面背景 + 中央寄せ）に続く、本文 +
//! 画像の 2 カラム構成の 404 ページ）。
//!
//! # 出典に関する注記
//!
//! 主参照は対応表 ID R1104（左にロゴ・左寄せ本文・下端の補助リンク、
//! 右に全高画像）、副次的に対応表 ID R0582（CTA 2 個 + 右画像）の構成を
//! 参照する。両者の構造のみを参照し、Rust/CSS で独自に再実装する。
//! 出典の固有名・ファイル名・文言・配色は持ち込まない（架空の文言を
//! 独自に書く）。参照元の全画面高さは持ち込まず、Demo 枠内の `min-height`
//! による表示へ変更した（`error_page_background_image` と同型の判断）。
//!
//! # 集約元 2 件の畳み込み方（原案差分メモに詳細を記載）
//!
//! R1104 の要素（ロゴ・左寄せ本文・戻るリンク・下端の補助リンク・
//! lg 以上だけの全高右画像）をすべて反映したうえで、R0582 の
//! 「CTA 2 個」を actions 行の 2 つ目の導線（「Contact support」）として
//! 畳み込む。右画像は両者に共通するため 1 つにまとめる。
//!
//! `Contact support` は当初 `button::button`（`ButtonVariant::Outline`）で
//! 実装していたが、404 ページの主要導線であるにもかかわらず遷移先を
//! 一切持たない非対話要素になっていたため（レビュー指摘、イシュー
//! #2841）、[`link::root`] へ変更し [`REPO`] への実在する遷移先を持たせた
//! （「補助リンクのリンク先」節参照）。これにより本 block は
//! `button::button` を使わなくなった。
//!
//! # 使用部品
//!
//! `empty-state`（メッセージコンテナ）/ `heading`（見出し）/ `text`
//! （ロゴ横の社名・エラーコード・説明文）/ `link`（戻るリンク・
//! 「Contact support」・補助リンク 2 個の計 4 個）/ `image`（右カラムの
//! 全高画像）/ `icon`（ロゴ・戻るリンクの矢印。自作の単純幾何図形）の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `empty_state::root`/`heading::heading`/`text::text`/`link::root`/
//! `image::image`/`icon::icon` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、本 block 固有のフックは
//! `data-blocks-error-page-split-image-*` の `data-*` 属性で渡す
//! （`crate::blocks` モジュール doc・`error_page_background_image` と同じ
//! 判断軸）。`empty_state::content`/`title`/`description`/`actions` と
//! 素の `div` は `attrs` をそのまま透過するため、それらも同じ `data-*`
//! 属性で統一する（フック名の一貫性を優先し、`class` と `data-*` を
//! 混在させない）。
//!
//! # `empty-state` の左寄せ上書きに詳細度合わせが必要な理由
//!
//! `empty_state::root`/`content`/`actions` の recipe base は中央寄せ
//! （`justify-content: center`/`align-items: center`/`text-align: center`）
//! を `[data-scope="empty-state"][data-part="<slot>"]`（詳細度 (0,2,0)）で
//! 宣言する。本 block は左寄せへ変更する必要があり、単独の
//! `[data-blocks-error-page-split-image-*]`（詳細度 (0,1,0)）のままでは
//! CSS の詳細度規則によりソース順に関わらず base 側が勝つ
//! （`error_page_background_image` のモジュール doc「背景画像フックの
//! 詳細度」節と同型の問題）。是正として base と同じ 2 属性セレクタへ
//! 本 block 固有フックを前置した 3 属性セレクタ
//! （`[data-scope="empty-state"][data-part="<slot>"][data-blocks-
//! error-page-split-image-*]`、詳細度 (0,3,0)）を用い、詳細度を base より
//! 高くしたうえでソース順（`blocks.css` は `pre-styled-ui.css` より後に
//! 読み込まれる）で後勝ちさせる。
//!
//! # `image` を全高にする詳細度合わせ・アウトオブフロー化
//!
//! `image::image` の recipe base は `[data-scope="image"][data-part="root"]`
//! （詳細度 (0,2,0)）で `height: auto` を持つ。全高にする本 block 固有
//! フックも同じ判断で `[data-scope="image"][data-part="root"][data-blocks-
//! error-page-split-image-image]`（詳細度 (0,3,0)）へ前置する。
//!
//! 当初は `.blocks-error-page-split-image-media`（grid item）側へ
//! `min-height: 100%` のみを与え、grid の既定 `align-items: stretch` で
//! 高さが伝播することを期待していたが、画像自体には `height` の確定した
//! containing block が与えられず、狭い画面幅で撮った元画像のアスペクト比
//! （横長）のまま表示され左カラム脇に余白ができる不具合があった（Bugbot
//! 指摘、イシュー #2841）。是正として `login_04`/`error_page_background_image`
//! と同型の絶対配置パターンへ変更した:
//! `.blocks-error-page-split-image-media` へ `position: relative`
//! を与え、画像フック側は `position: absolute; inset: 0;` +
//! `object-fit: cover` で親いっぱいに敷き詰める。これにより画像の高さは
//! 常に `.media` 列の実高さ（grid stretch で決まる）に一致し、
//! パーセンテージ高さの解決可否に依存しなくなる。
//!
//! # 画像は共通ダミー素材
//!
//! `crate::blocks::dummy_assets::SCREENSHOT_SRC`（ビルド時生成のモノトーン
//! スクリーンショット枠 SVG）を使う。`error_page_background_image` が使う
//! `BACKGROUND_SRC` とは別の素材にして、集約元の違いを視覚的にも区別する。
//! `data:` URI は `is_safe_url`（REQ-1）が拒否するため使わない（イシュー
//! #1562 の教訓）。
//!
//! # 補助リンクのリンク先はラベルの意味に対応した実在 URL
//!
//! `crate::blocks::marketing::contact::contact_info_columns` と同じ判断で、
//! `href="#"`（死リンク）は出力しない（`crate::blocks` モジュール doc の
//! 「`<form>` を使わない」節と同じく、実際に機能しない `#` を出力しない
//! 方針）。当初は「Contact support」を除く 2 つの補助リンク（Help
//! center・System status）を同一の [`REPO`] へ揃えていたが、ラベルと
//! 遷移先が一致せず利用者が期待する情報に到達できないとの指摘（Bugbot/
//! codex、イシュー #2841）を受け、3 リンクそれぞれへラベルの意味に近い
//! 別々の実在 URL を割り当てた:
//!
//! - **Contact support**（`Contact support` の CTA、[`link::root`] へ変更。
//!   「集約元 2 件の畳み込み方」節参照）: [`REPO`]（本フレームワークの
//!   問い合わせ・課題報告の実質的な受け口）
//! - **Help center**: `"../../guides/"`（docs サイト内の実在ページ
//!   `/guides/`、`site/nav.toml` 登録済み。「Back to home」の `"../../"`
//!   と同じ相対パス起点）
//! - **System status**: [`STATUS_URL`]（`REPO` 配下の GitHub Actions
//!   実行状況ページ。CI の稼働状況を示す実在サブページであり `REPO` 単体
//!   とは異なる URL）
//!
//! 区切り点は DOM を増やさず CSS の `::before` 疑似要素で描く。
//!
//! # ロゴ・戻る矢印は自作の単純幾何図形
//!
//! 実在ブランドのロゴ・商標を模した SVG は持ち込まない（`login_04::geo_icon`
//! と同型の判断）。ロゴは装飾用の抽象六角形、戻る矢印は左向き三角形の
//! 単純な `path` のみで構成する。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。静的表示のみで送信処理は一切持たず、リンクはすべて
//! 「補助リンクのリンク先はラベルの意味に対応した実在 URL」節の実在
//! ページへの遷移のみを行う。文言はすべて架空のものであり、実企業名・
//! 実サービス名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
/// ラベルの意味に対応した実在 URL」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 「System status」の遷移先（[`REPO`] 配下の GitHub Actions 実行状況
/// ページ。同節参照）。
const STATUS_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend/actions";

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
                REPO,
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
                STATUS_URL,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![(HELPER_LINK_ATTR, "")],
                vec![text("System status")],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/error-page-split-image/",
    title: "error-page-split-image",
    category: BlockCategory::ErrorPage,
    rust_source: "crates/docs-site/src/blocks/marketing/error_page/error_page_split_image.rs",
    demo_class: "blocks-error-page-split-image",
    parts: &[
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `error_page_split_image` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型で
/// `pub(super)` ではなく本ファイル内 `const` として [`super::blocks`] から
/// `BLOCK.layout_css` 経由で連結される）。
///
/// `lg` 境界（`min-width: 64rem`）はリポジトリ内の既存 `@media` と同じ値
/// （モジュール doc「`login_04` の `@media` の初使用」節、`login_04.rs`
/// 参照）。既定は右カラム非表示・1 カラムで、`lg` 以上で 2 カラム grid へ
/// 切り替える。右画像を全高にする手段は `align-items: stretch` 頼みでは
/// なく `login_04` と同型の絶対配置（モジュール doc「`image` を全高に
/// する詳細度合わせ・アウトオブフロー化」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-error-page-split-image {\n  padding: 0;\n  overflow: hidden;\n}\n\
.blocks-error-page-split-image-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  min-height: 28rem;\n}\n\
.blocks-error-page-split-image-main {\n  display: flex;\n  flex-direction: column;\n  justify-content: space-between;\n  gap: var(--fandhe-space-10);\n  padding: var(--fandhe-space-8) var(--fandhe-space-6);\n  min-width: 0;\n}\n\
.blocks-error-page-split-image-brand {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"empty-state\"][data-part=\"root\"][data-blocks-error-page-split-image-message] {\n  justify-content: flex-start;\n  padding: 0;\n}\n\
[data-scope=\"empty-state\"][data-part=\"content\"][data-blocks-error-page-split-image-content] {\n  align-items: flex-start;\n  text-align: start;\n  max-width: 32rem;\n}\n\
[data-scope=\"empty-state\"][data-part=\"actions\"][data-blocks-error-page-split-image-actions] {\n  justify-content: flex-start;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-error-page-split-image-code] {\n  color: var(--fandhe-color-accent);\n}\n\
[data-blocks-error-page-split-image-description] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-error-page-split-image-helper {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-4);\n}\n\
.blocks-error-page-split-image-helper [data-blocks-error-page-split-image-helper-link] ~ [data-blocks-error-page-split-image-helper-link]::before {\n  content: \"\\00b7\";\n  margin-right: var(--fandhe-space-3);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-error-page-split-image-media {\n  display: none;\n  position: relative;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-error-page-split-image-image] {\n  display: block;\n  position: absolute;\n  inset: 0;\n  width: 100%;\n  height: 100%;\n  object-fit: cover;\n}\n\
@media (min-width: 64rem) {\n  .blocks-error-page-split-image-layout {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n  }\n  .blocks-error-page-split-image-media {\n    display: block;\n  }\n  .blocks-error-page-split-image-main {\n    padding: var(--fandhe-space-10) var(--fandhe-space-8);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 6 部品（empty-state/heading/text/link/image/icon）を
    /// 実際に出力し、フック属性・文言・使用素材・実在する遷移先が期待
    /// どおりであること、`<form>` 等の非対話制約を固定する（モジュール
    /// doc「使用部品」節・「補助リンクのリンク先はラベルの意味に対応した
    /// 実在 URL」節）。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"empty-state\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(
            html.contains("<svg"),
            "demo should render icon svg (logo + back arrow)"
        );
        assert!(html.contains(r#"src="../../assets/blocks-demo-screenshot.svg""#));
        assert!(html.contains(r#"alt="""#));
        assert!(html.contains(r#"href="../../""#));
        assert!(html.contains(r#"href="../../guides/""#));
        assert!(html.contains(&format!(r#"href="{REPO}""#)));
        assert!(html.contains(&format!(r#"href="{STATUS_URL}""#)));
        assert!(
            html.matches(&format!(r#"href="{REPO}""#)).count() == 1,
            "Contact support should be the sole link pointing at REPO"
        );
        assert!(!html.contains("data-scope=\"button\""));
        assert!(!html.contains(r#"type="button""#));
        for hook in [
            LOGO_ATTR,
            BRAND_NAME_ATTR,
            MESSAGE_ATTR,
            CONTENT_ATTR,
            CODE_ATTR,
            TITLE_ATTR,
            DESCRIPTION_ATTR,
            ACTIONS_ATTR,
            BACK_ATTR,
            CTA_ATTR,
            HELPER_LINK_ATTR,
            IMAGE_ATTR,
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        for text_fragment in [
            "404",
            "This page took a wrong turn",
            "Back to home",
            "Contact support",
            "Help center",
            "System status",
            dummy_assets::COMPANY_NAMES[0],
        ] {
            assert!(
                html.contains(text_fragment),
                "demo should contain {text_fragment}"
            );
        }
        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "mailto:",
            "tel:",
            " id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタと `@media (min-width: 64rem)` を宣言し、
    /// 生の色リテラル（`#`）を持ち込まないこと（モジュール doc「`empty-state`
    /// の左寄せ上書きに詳細度合わせが必要な理由」節・「`image` を全高に
    /// する詳細度合わせ・アウトオブフロー化」節）。
    #[test]
    fn layout_css_declares_all_selectors_and_media_query() {
        for selector in [
            ".blocks-error-page-split-image {",
            ".blocks-error-page-split-image-layout {",
            ".blocks-error-page-split-image-main {",
            ".blocks-error-page-split-image-brand {",
            "[data-scope=\"empty-state\"][data-part=\"root\"][data-blocks-error-page-split-image-message] {",
            "[data-scope=\"empty-state\"][data-part=\"content\"][data-blocks-error-page-split-image-content] {",
            "[data-scope=\"empty-state\"][data-part=\"actions\"][data-blocks-error-page-split-image-actions] {",
            "[data-blocks-error-page-split-image-code] {",
            "[data-blocks-error-page-split-image-description] {",
            ".blocks-error-page-split-image-helper {",
            ".blocks-error-page-split-image-media {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-error-page-split-image-image] {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(!LAYOUT_CSS.contains('#'));
    }

    /// レイアウトのルート class（[`LAYOUT_CLASS`]）が [`BLOCK::demo_class`]
    /// とは別名であり、実際に Demo 出力へ現れること（Bugbot 指摘の教訓、
    /// 実装計画 §3.1 参照）。
    #[test]
    fn layout_root_class_is_distinct_from_demo_class_and_renders() {
        assert_ne!(LAYOUT_CLASS, BLOCK.demo_class);
        let html = render(&demo());
        assert!(html.contains(&format!("class=\"{LAYOUT_CLASS}\"")));
    }
}
