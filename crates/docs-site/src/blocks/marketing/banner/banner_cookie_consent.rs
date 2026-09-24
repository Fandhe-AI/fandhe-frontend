//! `banner-cookie-consent` block（イシュー #2740。親トラッキング #2738
//! 「目的別パーツ拡充」配下、Marketing / Banner カテゴリの最初の block）。
//!
//! # 使用部品
//!
//! `callout`（外枠）+ `heading`（見出し）+ `text`（説明文、`link` を内包）+
//! `button`（拒否/同意の 2 個）+ `icon`（自作の抽象 Cookie アイコン）の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 4 形態を並べて見せる（集約元の複数バリアントを 1 ページで示す）
//!
//! `demo()` は次の 4 形態を縦に並べる。いずれも中身の構成（見出し・説明・
//! ポリシーリンク・拒否/同意ボタン）は共通で、配置とパネル形状だけが
//! 異なる。
//!
//! 1. 右下寄せカード（基準形）
//! 2. 中央寄せカード
//! 3. 左寄せカード
//! 4. 全幅・上罫線バー（カードではなく画面幅いっぱいのバー）
//!
//! 各形態は [`stage`] が用意する「ビューポート枠」の中に配置し、キャプ
//! ションで形態名を示す。
//!
//! # 固定配置を Demo 枠内で相対配置に置き換える
//!
//! 実際の Cookie 同意通知は `position: fixed` で画面に貼り付くが、docs
//! サイトは JS ハイドレーションを行わない静的ページであり、`position:
//! fixed` を Demo 内で使うとページ本文を覆い隠してしまう。そのため
//! [`LAYOUT_CSS`] は `position: fixed` を使わず、[`stage`] の
//! `flex`/`justify-content` による相対配置で「画面の下端に寄る」見た目を
//! 再現する（`crate::blocks` モジュール doc の Demo 制約と同じ判断）。
//!
//! # `<form>` を使わない・アプリケーションロジックを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。「Decline」「Accept all」ボタンは `button::button` の既定
//! `type="button"` のままで送信先を持たない。Cookie の読み書き・同意状態
//! の保存・送信処理はいずれも持ち込まず、実装時は利用者自身の Rust
//! コードが担う（`docs/policy/intentional-non-adoption.md` §3.25、UI
//! コンポーネント層はアプリケーションロジックを内包しない）。
//!
//! # リンク先を外部リポジトリの URL にする理由
//!
//! [`Block::demo`] は `fn() -> Node`（`base_path` を受け取らない）ため、
//! サイト内絶対パスを組み立てられない。`href="#"`（死リンク）は
//! `crates/docs-site/tests/blocks_contract.rs` が禁止するため、他 block
//! （`footer_newsletter`/`footer_sticky_reveal` 等）と同じ判断で
//! `Fandhe-AI` の実在 GitHub リポジトリへの外部絶対 URL を用いる。ただし
//! 実在するプライバシーポリシーページは無いため、リンクの可視テキストは
//! 「Privacy policy」ではなく実際の遷移先と一致する「project repository」
//! とする（支援技術の利用者へ誤ったリンク目的を提示しない。イシュー
//! #2740 PR レビュー指摘）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `callout::root`/`heading::heading`/`text::text`/`button::button`/
//! `link::root`/`icon::icon` はいずれも `drop_class_attr` により呼び出し
//! 側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有スタイル
//! は `data-blocks-banner-cookie-consent-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。`callout::icon`/`callout::text`（drop を
//! 経由しない `ANATOMY.part` 直呼び）と素の `div`/`section`/`p` には
//! `class` がそのまま効く。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, p, section, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::callout::{self, CalloutProps, CalloutVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::text::{self as body_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 実在の遷移先（`href="#"` を使わないための外部絶対 URL、
/// `footer_newsletter`/`footer_sticky_reveal` と同じ判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の抽象 Cookie アイコン（円の輪郭 + 小さな点 3 個）。実在サービスの
/// アイコン・ロゴは写さず独自に描く。
fn cookie_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            ..IconProps::default()
        },
        vec![("data-blocks-banner-cookie-consent-icon", "")],
        vec![
            el(
                "circle",
                vec![
                    ("cx", "12"),
                    ("cy", "12"),
                    ("r", "9"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "9"),
                    ("cy", "10"),
                    ("r", "1.3"),
                    ("fill", "currentColor"),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "14"),
                    ("cy", "9"),
                    ("r", "1"),
                    ("fill", "currentColor"),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "13"),
                    ("cy", "14"),
                    ("r", "1.2"),
                    ("fill", "currentColor"),
                ],
                vec![],
            ),
        ],
    )
}

/// 1 形態ぶんの Cookie 同意通知本体を組み立てる。`layout` は
/// `"card"`/`"bar"`（[`LAYOUT_CSS`] の属性セレクタ値）、`label` は
/// ランドマークの名前（形態ごとに一意にし、支援技術が複数形態を区別
/// できるようにする）。
fn consent_banner(layout: &'static str, label: &'static str) -> Node {
    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![("data-blocks-banner-cookie-consent-title", "")],
        vec![text("We value your privacy")],
    );

    let description = body_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-banner-cookie-consent-description", "")],
        vec![
            text(
                "We use cookies to improve your experience and remember your \
                 preferences at Northwind Labs. See our ",
            ),
            link::root(
                REPO,
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("project repository")],
            ),
            text(" for details."),
        ],
    );

    let actions = div(
        vec![("class", "blocks-banner-cookie-consent-actions")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-banner-cookie-consent-decline", "")],
                vec![text("Decline")],
            ),
            button::button(
                &ButtonProps {
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-banner-cookie-consent-accept", "")],
                vec![text("Accept all")],
            ),
        ],
    );

    section(
        vec![
            ("aria-label", label),
            ("data-blocks-banner-cookie-consent-banner", layout),
        ],
        vec![callout::root(
            &CalloutProps {
                variant: CalloutVariant::Surface,
                size: Size::Md,
                palette: ColorPalette::Neutral,
            },
            vec![("data-blocks-banner-cookie-consent-callout", layout)],
            vec![
                callout::icon(vec![], vec![cookie_icon()]),
                callout::text(
                    vec![("class", "blocks-banner-cookie-consent-body")],
                    vec![title, description, actions],
                ),
            ],
        )],
    )
}

/// 1 形態ぶんの「ビューポート枠」。`align` は [`LAYOUT_CSS`] の
/// `data-blocks-banner-cookie-consent-align` 値、`caption` は形態名。
fn stage(align: &'static str, caption: &'static str, banner: Node) -> Node {
    div(
        vec![
            ("class", "blocks-banner-cookie-consent-stage"),
            ("data-blocks-banner-cookie-consent-align", align),
        ],
        vec![
            p(
                vec![("class", "blocks-banner-cookie-consent-caption")],
                vec![text(caption)],
            ),
            banner,
        ],
    )
}

/// `banner-cookie-consent` の Demo 本体（モジュール doc「4 形態を並べて
/// 見せる」節参照）。呼び出しごとに決定的な `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-banner-cookie-consent-stack")],
        vec![
            stage(
                "end",
                "Bottom-right card",
                consent_banner("card", "Cookie consent (bottom-right card)"),
            ),
            stage(
                "center",
                "Centered card",
                consent_banner("card", "Cookie consent (centered card)"),
            ),
            stage(
                "start",
                "Bottom-left card",
                consent_banner("card", "Cookie consent (bottom-left card)"),
            ),
            stage(
                "bar",
                "Full-width bar",
                consent_banner("bar", "Cookie consent (full-width bar)"),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/banner-cookie-consent/",
    title: "banner-cookie-consent",
    category: BlockCategory::Banner,
    rust_source: "crates/docs-site/src/blocks/marketing/banner/banner_cookie_consent.rs",
    demo_class: "blocks-banner-cookie-consent",
    parts: &[
        Part {
            label: "Callout",
            path: "/themes/callout/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `banner_cookie_consent` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル非公開の定数として [`BLOCK::layout_css`] 経由で
/// `super::stylesheet` から連結される）。
///
/// `position: fixed` を使わない理由・sm 未満（640px 未満）でボタンを
/// 縦積み全幅にする理由はモジュール doc参照。ほぼ全セレクタは
/// `.blocks-banner-cookie-consent-*`/`[data-blocks-banner-cookie-consent-*]`
/// の名前空間に収めるが、`"bar"` 形態の callout 上書き（角丸解除・
/// 上辺のみのボーダー）だけは例外的に `[data-scope="callout"]
/// [data-part="root"]` を先頭へ連結する。`callout` recipe の base 規則
/// （`[data-scope="callout"][data-part="root"]`、詳細度 (0,2,0)）と
/// `Surface` variant 規則（同セレクタ + variant class、詳細度 (0,3,0)）が
/// 先に `border`/`border-color` を宣言しているため、この block 固有の
/// 単一属性セレクタ（詳細度 (0,1,0)）のままでは負けて反映されない
/// （角丸・全周ボーダーのカードのまま描画される）。`data-scope`/
/// `data-part` を連結して詳細度を (0,3,0) へ揃え、`blocks.css` が
/// `pre-styled-ui.css` より後に `<link>` される読み込み順序
/// （`crate::build::build_site` の `extra_stylesheets` 追加順）で
/// 同詳細度の後勝ちにより上書きを成立させる。
///
/// キャプション（[`stage`] の生 `p`）も同様の詳細度不足を持っていた
/// （PR レビュー指摘）: 単一クラスセレクタ（詳細度 (0,1,0)）では
/// `crates/docs-site/src/site_theme.rs` の `.docs-content p`（詳細度
/// (0,1,1)、`margin: 0 0 1.05rem`）に負け、キャプションを枠内下端へ
/// 寄せる `margin: 0 0 auto` が反映されずキャプションがバナー位置へ
/// 潰れていた。`p.blocks-banner-cookie-consent-caption` へ型セレクタを
/// 連結して詳細度を (0,1,1) の同点へ揃え、`blocks.css` が `site.css`
/// （`.docs-content p` の定義元）より後に `<link>` される読み込み順序で
/// 同詳細度の後勝ちにより上書きを成立させる。
///
/// キャプションと [`consent_banner`] は [`stage`] の同一 column flex に
/// 同居する 2 個の flex item であるため、`[data-blocks-banner-cookie-
/// consent-align]` の `align-items`（バナー本体のクロス軸配置を切り替える
/// ための宣言）がキャプションの位置にも波及していた（PR #3151 レビュー
/// 指摘）: `"end"`/`"center"` ステージではキャプションのフレームラベルが
/// 右寄せ・中央寄せされ、`"bar"` ステージでは `padding: 0`（バナーを枠
/// 全幅へ張り出させるための宣言）によりキャプションも一緒に破線の端まで
/// 詰まっていた。`align-self: flex-start` をキャプション自身へ明示する
/// ことで、親の `align-items` 値に関わらずキャプションのクロス軸位置
/// （常に左）を固定する（`align-self` は同一要素の `align-items` を
/// 上書きする専用プロパティであり、別要素間の詳細度比較を経ずに常に
/// 有効になる）。`"bar"` ステージでは、これでキャプションの水平位置は
/// 安定するが、親の `padding: 0` によりキャプション自身の左右インセット
/// も失われたままになるため、`[data-blocks-banner-cookie-consent-
/// align="bar"] > p.blocks-banner-cookie-consent-caption` へキャプション
/// 専用の `padding`（[`stage`] の既定インセットと同じ値）を明示的に
/// 復元し、バナー本体だけを枠端まで張り出させる。
const LAYOUT_CSS: &str = "\
.blocks-banner-cookie-consent-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6, 1.5rem);\n}\n\
.blocks-banner-cookie-consent-stage {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  justify-content: flex-end;\n  min-height: 16rem;\n  padding: var(--fandhe-space-4, 1rem);\n  border: 1px dashed var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md, 0.5rem);\n  background: var(--fandhe-color-bg);\n  overflow: hidden;\n}\n\
p.blocks-banner-cookie-consent-caption {\n  align-self: flex-start;\n  margin: 0 0 auto;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-banner-cookie-consent-align=\"end\"] {\n  align-items: flex-end;\n}\n\
[data-blocks-banner-cookie-consent-align=\"center\"] {\n  align-items: center;\n}\n\
[data-blocks-banner-cookie-consent-align=\"start\"] {\n  align-items: flex-start;\n}\n\
[data-blocks-banner-cookie-consent-align=\"bar\"] {\n  align-items: stretch;\n  padding: 0;\n}\n\
[data-blocks-banner-cookie-consent-align=\"bar\"] > p.blocks-banner-cookie-consent-caption {\n  padding: var(--fandhe-space-4, 1rem) var(--fandhe-space-4, 1rem) 0;\n}\n\
[data-blocks-banner-cookie-consent-banner=\"card\"] {\n  width: 100%;\n  max-width: 24rem;\n}\n\
[data-blocks-banner-cookie-consent-banner=\"bar\"] {\n  width: 100%;\n}\n\
[data-scope=\"callout\"][data-part=\"root\"][data-blocks-banner-cookie-consent-callout=\"bar\"] {\n  border-radius: 0;\n  border-width: 1px 0 0;\n  border-top-color: var(--fandhe-color-border);\n}\n\
.blocks-banner-cookie-consent-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2, 0.5rem);\n}\n\
[data-blocks-banner-cookie-consent-title],\n[data-blocks-banner-cookie-consent-description] {\n  margin: 0;\n}\n\
.blocks-banner-cookie-consent-actions {\n  display: flex;\n  gap: var(--fandhe-space-2, 0.5rem);\n  flex-wrap: wrap;\n  justify-content: flex-end;\n}\n\
@media (max-width: 39.99rem) {\n  .blocks-banner-cookie-consent-actions {\n    flex-direction: column;\n  }\n  [data-blocks-banner-cookie-consent-decline],\n  [data-blocks-banner-cookie-consent-accept] {\n    width: 100%;\n  }\n  [data-blocks-banner-cookie-consent-banner=\"card\"] {\n    max-width: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    #[test]
    fn layout_css_never_uses_position_fixed() {
        assert!(
            !LAYOUT_CSS.contains("position: fixed"),
            "Demo must replace the real fixed placement with an in-flow stage \
             (module doc \"固定配置を Demo 枠内で相対配置に置き換える\" section)"
        );
    }

    #[test]
    fn layout_css_bar_override_outranks_callout_surface_variant() {
        // `callout` recipe の Surface variant 規則（詳細度 (0,3,0)、
        // `crates/pre-styled-ui/src/callout.rs` 参照）に勝つには、この
        // block 固有セレクタも `[data-scope="callout"][data-part="root"]`
        // を連結して詳細度 (0,3,0) 以上にする必要がある（単一属性セレクタ
        // （0,1,0）のみでは負けて反映されない不具合の回帰防止）。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"callout\"][data-part=\"root\"]\
             [data-blocks-banner-cookie-consent-callout=\"bar\"]"
        ));
    }

    #[test]
    fn layout_css_caption_outranks_docs_content_paragraph_margin() {
        // `.docs-content p`（`crates/docs-site/src/site_theme.rs`、詳細度
        // (0,1,1)）に勝つには、この block 固有セレクタも型セレクタ `p` を
        // 連結して詳細度 (0,1,1) 以上にする必要がある（単一クラス
        // セレクタ（0,1,0）のみでは負けてキャプションがバナー位置へ
        // 潰れる不具合の回帰防止）。
        assert!(LAYOUT_CSS.contains("p.blocks-banner-cookie-consent-caption {"));
    }

    #[test]
    fn layout_css_caption_alignment_is_independent_of_stage_align_items() {
        // キャプションと `consent_banner` は [`stage`] の同一 column flex を
        // 共有する 2 個の flex item であり、`align-items`（バナー本体の
        // クロス軸配置切り替え）がキャプション位置にも波及していた
        // 不具合の回帰防止（PR #3151 レビュー指摘）。`align-self` で
        // キャプション自身のクロス軸位置を親の `align-items` 値に関わらず
        // 固定する。
        assert!(LAYOUT_CSS
            .contains("p.blocks-banner-cookie-consent-caption {\n  align-self: flex-start;"));
    }

    #[test]
    fn layout_css_bar_caption_keeps_padding_despite_stage_padding_reset() {
        // `"bar"` ステージの `padding: 0`（バナー本体を枠全幅へ張り出させる
        // ための宣言）はキャプションにも波及し、破線の端まで詰まって
        // しまっていた（PR #3151 レビュー指摘）。キャプション専用の
        // `padding` 復元規則がキャプションの左右インセットを取り戻す
        // ことを固定する。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-banner-cookie-consent-align=\"bar\"] > \
             p.blocks-banner-cookie-consent-caption {\n  padding:"
        ));
    }

    #[test]
    fn layout_css_stacks_actions_full_width_below_sm() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 39.99rem)"));
        assert!(LAYOUT_CSS
            .contains(".blocks-banner-cookie-consent-actions {\n    flex-direction: column;"));
        assert!(LAYOUT_CSS.contains("[data-blocks-banner-cookie-consent-decline],\n  [data-blocks-banner-cookie-consent-accept] {\n    width: 100%;"));
    }

    #[test]
    fn demo_composes_four_layouts_with_expected_parts() {
        let html = render(&demo());

        // `callout::root`/`icon`/`text` の 3 パーツがいずれも
        // `data-scope="callout"` を持つため、4 形態 × 3 パーツで 12 回。
        assert_eq!(html.matches("data-scope=\"callout\"").count(), 12);
        assert_eq!(
            html.matches(r#"data-scope="callout" data-part="root""#)
                .count(),
            4
        );
        assert_eq!(html.matches("type=\"button\"").count(), 8);
        assert!(html.contains("fd-button--variant-outline"));
        assert!(html.contains("fd-button--variant-solid"));
        assert!(html.contains(r#"href="https://github.com/Fandhe-AI/fandhe-frontend""#));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("data:"));

        for align in ["end", "center", "start", "bar"] {
            assert!(html.contains(&format!(
                "data-blocks-banner-cookie-consent-align=\"{align}\""
            )));
        }
    }
}
