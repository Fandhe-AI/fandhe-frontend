//! `logo-cloud-split` block（イシュー #2795。Marketing / Logo Cloud
//! カテゴリの最初の block。対応表 ID R1058（主参照）・R0565/R0149/R0566/
//! R0145/R0567（集約元）を構造の参照元とする合成例。見出し左 + ロゴ 2 列
//! グリッド右）。取得手段・ファイル名・内部コンポーネント識別子は記載
//! しない（`docs/design/motion-reference-adoption-policy.md` §9 と同じ
//! ライセンス上の転記制限）。参照ファイル置き場（`_/blocks-intake/`）は
//! 本実装時点で手元に存在せず、イシュー本文のレイアウト仕様・集約元の
//! 一行説明のみを根拠に設計した（参照元の文言・配色・装飾の転記が構造的
//! に起きない）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `link` の 4 部品のみを合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証
//! する）。新しい UI 部品は追加しない。当初は CTA 2 本を `button` で
//! 組み立てていたが、遷移先を持たない `type="button"` のまま「無料で
//! 始める」「導入事例を見る」という遷移を期待させる文言を出すのは
//! 押しても何も起きない dead control になるという指摘（イシュー #2795
//! codex レビュー是正 1 回目、PR #3247）を受けて [`link::root`] へ置き換えた
//! （`blog_split_header_grid` の「すべての記事を見る」是正と同じ判断軸）。
//! しかし [`Block::demo`] は `base_path` を受け取れず（`base_path` を要求
//! する制約は次項参照）、遷移先を実ページごとに分けられないため、CTA 2 本
//! を同一 [`REPO`] へ揃えたところ、今度は「無料で始める」「導入事例を
//! 見る」という文言が実際の遷移先（本リポジトリ）と一致しないという指摘
//! （同イシュー codex レビュー 2 回目）を受けた。実ページを用意できない
//! 以上、文言側を遷移先に合わせる方針を採り、CTA 文言を「GitHub で見る」
//! 「Star をつける」へ変更した（いずれも GitHub リポジトリページ上で実際に
//! 行える操作を指す、誇張のない文言）。
//!
//! 当初は CTA の下にもう 1 本「導入企業の一覧（GitHub）」というリンクを
//! 置いていたが、遷移先は同じ [`REPO`]（リポジトリのトップページ）であり
//! 「一覧」という実在しないページを指すラベルだった（同イシュー codex
//! レビュー 3 回目、PR #3247）。実在する遷移先に合わせたラベルへ書き換える
//! よりも、同じ [`REPO`] へ重複して遷移するだけの導線を削る方が実態と
//! 齟齬のない構成になるため、この 3 本目のリンクは削除した（CTA 2 本の
//! 「GitHub で見る」で同じ遷移は既にカバーされている）。
//!
//! # 3 形構成（集約元との差分）
//!
//! 集約元 6 件を [`variant_basic`]/[`variant_bordered`]/[`variant_dark`]
//! の 3 形へ統合する（`variant_label` で対応 ID を示しながら [`demo`] 1 つ
//! の中へ縦に並記する、`cta_split_image` と同じ構成手段）。
//!
//! - **基準形**（R1058 主参照）: tagline + 見出し + 説明 + CTA ボタン 2 本
//!   の左列、枠なしロゴ 2 列グリッドの右列
//! - **淡色枠タイル形**（R0149/R0566/R0145 集約）: 見出し + 説明のみの
//!   左列（CTA なし。R0145「見出しのみ」の差分を表す）、淡色枠タイルへ
//!   ロゴを収めた 2 列グリッドの右列
//! - **暗色固定形**（R0567 集約）: 基準形と同じ左列構成を暗色面上に配置
//!
//! 「横並び」（R0565）は Demo を増やさず本節で言及するに留める。基準形の
//! グリッド列数を変えるだけで表現できるため、原稿の差分メモで扱う
//! （Demo を増やすと原稿フェンスが長くなるだけで新しい構造を示さない）。
//!
//! # ロゴ・社名は架空
//!
//! 6 ロゴとも `crate::blocks::dummy_assets::LOGO_SRC`（同一の抽象バッジ
//! SVG）を使い、`dummy_assets::COMPANY_NAMES` の架空社名をキャプション
//! として添えて視覚的に区別する。実在ブランドのロゴ・商標・企業名は
//! 一切使わない。tagline の文言も「導入企業（デモ用の架空サンプル）」と
//! 明記し、見出しの「多くのチームに選ばれています」がロゴ・社名同様
//! 架空の一例であって実際の導入実績ではないことをコメント頼みにせず
//! Demo 内の可視テキストとして示す（codex レビュー指摘、イシュー #2795
//! PR #3247）。

//!
//! # 暗色固定（ライト/ダーク切替に追随しない）
//!
//! 「暗色固定」は現在のテーマ（ライト/ダーク）に関わらず常に暗色面で
//! 表示する形であり、`data-blocks-logo-cloud-split-tone="dark"` を
//! ラッパへ付与して表す。`var(--fandhe-color-fg)`/`var(--fandhe-color-bg)`
//! はライト/ダーク双方の値を持つテーマ依存トークンであり、そのまま
//! 使うとダークテーマ下で反転して背景が明色・文字が暗色になり本契約を
//! 満たせない（`cta_split_image` の `tone` はテーマ追随の切替 UI 配下で
//! 使われるため theme token のままでよいが、本 block の「暗色固定」は
//! テーマに関わらない固定色が必要という別要件）。このため
//! `--fandhe-color-fg`/`--fandhe-color-bg` のライトテーマ値
//! （`#111111`/`#ffffff` 相当の明暗）をリテラル値として直書きし、
//! テーマ切替の影響を受けない固定の暗色面にする（`LAYOUT_CSS` の
//! ブレークポイント直書き同様、テーマトークンでは表現できない値を
//! リテラルで補う判断）。
//!
//! # ブレークポイント（lg=64rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （`contact_split_info` 等と同じ判断）。`lg` 未満では見出し列の下へ
//! ロゴグリッドを縦積みし、`lg` 以上で「左見出し + 右ロゴ」の 2 カラムへ
//! 切り替える。ロゴグリッド自体は全幅で常に 2 列固定とする（ロゴが
//! 小さく、狭幅でも 2 列で崩れないため）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # リンク先を固定リポジトリ URL にする理由
//!
//! `crate::blocks` の他 block と同じく `link::root` + 固定 URL
//! （[`REPO`]）で「実際に押せる」導線を表す。`href="#"` は使わない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。CTA は前項のとおり `link::root` のみで構成し、`<button>` は
//! 一切出力しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`、
/// `cta_split_image::variant_label` と同型）。
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

/// ロゴ 1 件（同一の抽象バッジ SVG + 架空社名キャプション）。`bordered`
/// が true のとき淡色枠タイルへ収める（淡色枠タイル形の差分）。
fn logo_item(company: &'static str, bordered: bool) -> Node {
    // alt は空文字にする（隣接する `caption` が同じ社名を可視テキストとして
    // 持つため、`alt` にも同じ文字列を入れるとスクリーンリーダーが同名を
    // 二重読み上げしてしまう。`logo_cloud_marquee::logo` と同じ判断）。
    let logo_image = image(
        &ImageProps {
            fit: ImageFit::Contain,
            ..ImageProps::new(dummy_assets::LOGO_SRC, "")
        },
        vec![("data-blocks-logo-cloud-split-logo", "")],
    );
    let caption = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(company)],
    );
    if bordered {
        div(
            vec![("data-blocks-logo-cloud-split-tile", "")],
            vec![logo_image, caption],
        )
    } else {
        div(vec![], vec![logo_image, caption])
    }
}

/// ロゴ 6 件を 2 列グリッドへ並べる（[`dummy_assets::COMPANY_NAMES`] の
/// 先頭 6 件を使う）。
fn logo_grid(bordered: bool) -> Node {
    div(
        vec![("class", "blocks-logo-cloud-split-grid")],
        dummy_assets::COMPANY_NAMES
            .iter()
            .take(6)
            .map(|company| logo_item(company, bordered))
            .collect(),
    )
}

/// 基準形・暗色固定形で共通の左列（tagline + 見出し + 説明 + CTA ボタン
/// 2 本 + GitHub リンク）。
fn copy_with_cta() -> Node {
    div(
        vec![("class", "blocks-logo-cloud-split-copy")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入企業（デモ用の架空サンプル）")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("多くのチームに選ばれています")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "様々な規模のチームが日々の開発にご利用いただいています。",
                )],
            ),
            div(
                vec![("class", "blocks-logo-cloud-split-cta")],
                vec![
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("GitHub で見る")],
                    ),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("Star をつける")],
                    ),
                ],
            ),
        ],
    )
}

/// 基準形（R1058 主参照）: CTA 付き左列 + 枠なしロゴ 2 列グリッド。
fn variant_basic() -> Node {
    div(
        vec![
            ("class", "blocks-logo-cloud-split-row"),
            ("data-blocks-logo-cloud-split-row", ""),
        ],
        vec![copy_with_cta(), logo_grid(false)],
    )
}

/// 淡色枠タイル形（R0149/R0566/R0145 集約）: 見出し + 説明のみの左列
/// （CTA なし）+ 淡色枠タイルのロゴ 2 列グリッド。
fn variant_bordered() -> Node {
    let left = div(
        vec![("class", "blocks-logo-cloud-split-copy")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Medium,
                    ..TextProps::default()
                },
                vec![],
                vec![text("導入企業（デモ用の架空サンプル）")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("信頼できるパートナー企業")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("業界を代表する企業と協業しています。")],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-logo-cloud-split-row"),
            ("data-blocks-logo-cloud-split-row", ""),
        ],
        vec![left, logo_grid(true)],
    )
}

/// 暗色固定形（R0567 集約）: 基準形と同じ左列構成を暗色面上に配置する
/// （モジュール doc「暗色固定」節）。
fn variant_dark() -> Node {
    div(
        vec![("data-blocks-logo-cloud-split-tone", "dark")],
        vec![div(
            vec![
                ("class", "blocks-logo-cloud-split-row"),
                ("data-blocks-logo-cloud-split-row", ""),
            ],
            vec![copy_with_cta(), logo_grid(true)],
        )],
    )
}

/// `logo-cloud-split` の Demo 本体（3 形を縦に並記）。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-logo-cloud-split-layout")],
        vec![
            variant_label("基準形（R1058）"),
            variant_basic(),
            variant_label("淡色枠タイル形（R0149/R0566/R0145）"),
            variant_bordered(),
            variant_label("暗色固定形（R0567）"),
            variant_dark(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/logo-cloud-split/",
    title: "logo-cloud-split",
    category: BlockCategory::LogoCloud,
    rust_source: "crates/docs-site/src/blocks/marketing/logo_cloud/logo_cloud_split.rs",
    demo_class: "blocks-logo-cloud-split",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `logo_cloud_split` 固有のレイアウト規則。セレクタは
/// `.blocks-logo-cloud-split-*` と `[data-blocks-logo-cloud-split-*]` の
/// みを用い、他 block や部品の素のセレクタへ影響させない
/// （`contact_split_info` 等と同じ名前空間分離）。ロゴ画像の高さ上限
/// （`[data-blocks-logo-cloud-split-logo]`）は image recipe の
/// `[data-scope="image"][data-part="root"]`（属性セレクタ 2 個）に詳細度で
/// 負けるため、`[data-scope="image"][data-part="root"][data-blocks-logo-
/// cloud-split-logo]`（属性セレクタ 3 個）へ揃える
/// （`contact_split_form_image`/`contact_form_testimonial` 等と同じ回避）。
/// 暗色固定形（`variant_dark`）配下の `text`/`link` は `TextVariant::Muted`/
/// `ColorPalette::Neutral` が明色背景向けの固定色を出すため、
/// `[data-blocks-logo-cloud-split-tone="dark"]` スコープ内で `color:
/// inherit` へ上書きし、暗色ラッパが設定する `color: var(--fandhe-color-
/// bg)` を継承させてコントラストを保つ（`banner_full_width_bar` の
/// `tone="dark"` 配下 `link`/`button` と同じ回避）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-logo-cloud-split` だが、`demo()` が
/// 返すルート `div` の class は `blocks-logo-cloud-split-layout` という
/// 別名にする（既存 block と同じ Bugbot 教訓の回避。ページ側が
/// `demo_class` を `.blocks-demo` の隣に付与するラッパーと block 自身の
/// レイアウトルートを区別するため）。
const LAYOUT_CSS: &str = "\
.blocks-logo-cloud-split-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-logo-cloud-split-row {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-logo-cloud-split-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-logo-cloud-split-cta {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n  align-items: center;\n}\n\
.blocks-logo-cloud-split-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n  align-items: center;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-logo-cloud-split-logo] {\n  height: 3rem;\n  width: auto;\n  max-width: 100%;\n}\n\
[data-blocks-logo-cloud-split-tile] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-4);\n  border-radius: var(--fandhe-radius-md);\n  border: 1px solid var(--fandhe-color-border);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-logo-cloud-split-tone=\"dark\"] {\n  padding: var(--fandhe-space-8);\n  border-radius: var(--fandhe-radius-lg);\n  background: #111111;\n  color: #ffffff;\n}\n\
[data-blocks-logo-cloud-split-tone=\"dark\"] [data-blocks-logo-cloud-split-tile] {\n  border-color: #f7f7f7;\n  background: transparent;\n}\n\
[data-blocks-logo-cloud-split-tone=\"dark\"] [data-scope=\"text\"][data-part=\"root\"],\n[data-blocks-logo-cloud-split-tone=\"dark\"] [data-scope=\"link\"][data-part=\"root\"] {\n  color: inherit;\n}\n\
@media (min-width: 64rem) {\n  .blocks-logo-cloud-split-row {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n    align-items: center;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, REPO};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/image/link）の anatomy をすべて
    /// 実際に出力していることと、行数・ロゴ枚数・リンク先を固定する
    /// （`contact_split_info_composes_expected_parts` と同型）。CTA を
    /// `button` から `link::root` へ置き換えたため（モジュール doc「使用
    /// 部品」節参照）、`data-scope="button"`・`type="button"` の非出現も
    /// あわせて固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(
            !html.contains("data-scope=\"button\""),
            "demo should not render any button part (CTA is link::root only)"
        );
        assert_eq!(
            html.matches("data-blocks-logo-cloud-split-row").count(),
            3,
            "demo should render exactly 3 rows (basic / bordered / dark)"
        );
        assert_eq!(
            html.matches("data-blocks-logo-cloud-split-logo").count(),
            18,
            "demo should render exactly 18 logo images (6 logos x 3 rows)"
        );
        assert_eq!(
            html.matches(r#"type="button""#).count(),
            0,
            "demo should never render a <button>; CTA is link::root only"
        );
        assert_eq!(
            html.matches(&format!("href=\"{REPO}\"")).count(),
            4,
            "2 CTA links per row (basic / dark rows) should point to the fixed repository URL"
        );
        assert!(html.contains(r#"data-blocks-logo-cloud-split-tone="dark""#));
    }

    /// 非対話・XSS 回帰の不変条件（`crate::blocks` モジュール doc）を固定
    /// する。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・列数・tone セレクタを
    /// 持ち、`<` を含まないこと（REQ-1: `</style>` によるスタイル脱出を
    /// 防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints_and_column_counts() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2"));
        assert!(LAYOUT_CSS.contains("data-blocks-logo-cloud-split-tone=\"dark\""));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定、既存 block と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-logo-cloud-split-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-logo-cloud-split-layout");
    }

    /// ロゴの高さセレクタが image recipe の `[data-scope="image"][data-
    /// part="root"]`（属性セレクタ 2 個）と同数以上の詳細度を持つこと
    /// （Bugbot 指摘の回帰固定）。
    #[test]
    fn logo_height_selector_matches_image_recipe_specificity() {
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-logo-cloud-split-logo]"
        ));
    }

    /// ロゴ画像の `alt` は空文字であり、隣接するキャプション（社名）と
    /// 二重読み上げにならないこと（Bugbot 指摘の回帰固定）。
    #[test]
    fn logo_image_alt_is_empty_to_avoid_duplicate_announcement() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"alt="""#).count(),
            18,
            "all 18 logo images should have an empty alt (caption carries the company name)"
        );
    }

    /// 暗色固定形の `text`/`link` が `color: inherit` で暗色ラッパの前景色を
    /// 継承し、`TextVariant::Muted`/`ColorPalette::Neutral` の固定色のまま
    /// 残らないこと（Bugbot 指摘の回帰固定）。
    #[test]
    fn dark_variant_overrides_muted_text_and_link_color() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-logo-cloud-split-tone=\"dark\"] [data-scope=\"text\"][data-part=\"root\"]"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-logo-cloud-split-tone=\"dark\"] [data-scope=\"link\"][data-part=\"root\"]"
        ));
        assert!(LAYOUT_CSS.contains("color: inherit;"));
    }

    /// 暗色固定形はテーマ依存トークン（`var(--fandhe-color-fg)`/
    /// `var(--fandhe-color-bg)`）ではなくリテラル値で固定されており、
    /// ダークテーマ下で反転しないこと（codex レビュー P1 指摘の回帰固定、
    /// イシュー #2795 PR #3247）。
    #[test]
    fn dark_variant_uses_literal_colors_not_theme_tokens() {
        assert!(LAYOUT_CSS.contains("background: #111111;\n  color: #ffffff;"));
        assert!(!LAYOUT_CSS.contains(
            "[data-blocks-logo-cloud-split-tone=\"dark\"] {\n  padding: var(--fandhe-space-8);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-fg);"
        ));
    }
}
