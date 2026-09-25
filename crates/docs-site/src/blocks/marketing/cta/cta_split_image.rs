//! `cta-split-image` block（イシュー #2759。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Phase 1（#2738、マーケティング A）
//! に属する。画像付きの 2 列分割 CTA を、主参照 R0883（基準形）に
//! R0448/R0450/R0875/R0884/R0885 を集約した 4 形として合成する。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない（`super::cta_feature_links`
//! と同じライセンス上の転記制限、対応表 ID のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `button` / `image` / `card` / `icon` /
//! `link` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 4 形を 1 つの Demo に並記する
//!
//! [`super::super::content::content_split_image`] と同型に、4 形を
//! [`variant_label`] で見出しを付けながら [`demo`] 1 つの中へ縦に並べる。
//!
//! | 形 | 対応 ID | 内容 |
//! |----|---------|------|
//! | A 基準形 | R0883 | 画像 1 枚（片側）+ badge/heading/text/button（反対側） |
//! | B カード形 | R0448/R0450/R0875 | [`variant_card`] を tone（暗色/アクセント）違いで 2 インスタンス。カードの端まで全面に画像を広げる |
//! | C チェック付き | R0884 | カード内に写真 + 見出し + 本文 + チェック付き項目 6 件 + リンク |
//! | D タイル | R0885 | 画像 1 枚の代わりに 2×2 のタイル 4 枚 |
//!
//! # B: カードの端まで全面に画像を広げる方法
//!
//! [`card::root`] は既定で padding・border を持つため、[`variant_card`] は
//! `[data-blocks-cta-split-image-card]` 属性へ `padding: 0; overflow: hidden;
//! border: 0;` を上書きし、画像を [`card::root`] の直接の子として
//! [`card::body`]（コピー列）と横に並べる。画像はカード枠の内側全面へ
//! `object-fit: cover` + `height: 100%` で広がる（`content_with_testimonial::
//! photo_quote` が写真を `position: absolute` の暗幕オーバーレイで扱ったのに
//! 対し、本 block は画像とコピー列を横並びグリッドの 2 トラックとして扱う点が
//! 差分）。
//!
//! # B: 反転配色（暗色 / アクセント）と `color: inherit` 上書き
//!
//! [`card::root`] へ直接 `background`/`color` を上書きすることで、内側の
//! [`heading::heading`]（自身の `color` 宣言を持たないため素直に継承する）・
//! [`text::text`]（既定 `TextVariant::Plain` は `color` 宣言を持たないため
//! 同様に継承する）は追加の上書きなしで反転配色に追従する。[`button::button`]
//! はどの `ButtonVariant` でも独自の `background`/`color` を持つため、
//! `[data-blocks-cta-split-image-cta]` 属性へ `background: var(--fandhe-color-bg);
//! color: var(--fandhe-color-fg);` を明示上書きし、暗色・アクセントいずれの
//! カード面に対しても一定のコントラストを確保する（[`content_with_testimonial`]
//! の `blockquote` に対する `color: inherit` 上書きと同じ判断軸だが、button は
//! 明示的な反転色を選ぶ方が両トーンで安定するため異なる手当てを選んだ）。
//! B は badge を使わない（badge は全 variant で `color`/`background` を必ず
//! 自前で持つため、2 トーン分の追加上書きを避けるための意図的な省略。
//! badge 自体は A/D で使用済みのため [`BLOCK::parts`] 契約は満たす）。
//!
//! # 詳細度: `[data-scope]` を含めた 3 セレクタ構成
//!
//! [`super::super::content::content_split_image`] と同じ判断軸で、
//! `card::root`・`button::button` の recipe（詳細度 (0,2,0) 程度）に確実に
//! 勝つため、上書きは `[data-scope="card"][data-part="root"][data-blocks-
//! cta-split-image-*]` のように `data-scope`/`data-part` を含めた 3 セレクタ
//! 構成（詳細度 (0,3,0) 以上）で行う。
//!
//! # C: チェック付き項目のアイコン
//!
//! チェックマークは [`super::super::content::content_split_image::geo_icon`]
//! と同型の自作幾何アイコン（lucide 等の著作物を複製しない単純な折れ線）を
//! 本ファイル内へ複製する（クレート内の private 関数のため共有できない）。
//!
//! # link の `href` を固定の外部絶対 URL にする
//!
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない
//! （`super::cta_feature_links` と同じ制約）。C の誘導リンクは実在する
//! GitHub リポジトリへの外部絶対 URL（[`REPO`]）に固定し、`external: true`
//! で `rel="noopener noreferrer"` を付与する。死リンク `href="#"` は使わない。
//!
//! # `id`/`aria-labelledby` を出力しない・複数インスタンス
//!
//! B は 2 インスタンス（tone 違い）を持つため、`id` 属性は一切使わない
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、全ての見出しは
//! `HeadingLevel::H3` を使う。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と `fandhe_frontend_core::text`
//! が同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # ブレークポイントをリテラルで直書きする理由
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と
//! 同じ判断）。
//!
//! # `drop_class_attr` と CSS フックの選び方
//!
//! `badge::badge`/`heading::heading`/`text::text`/`button::button`/
//! `image::image`/`card::root`/`card::body`/`icon::icon`/`link::root` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-cta-split-image-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性
//! セレクタで対応する。素の `div`/`ul`/`li` には `class` がそのまま効くため、
//! グリッド・タイル配置等は従来どおり `.blocks-cta-split-image-*` クラス
//! セレクタを使う。レイアウト root の class（`blocks-cta-split-image-layout`）
//! は [`Block::demo_class`]（`blocks-cta-split-image`）と意図的に別名にする
//! （既存 block と同じ Bugbot 教訓の回避）。
//!
//! # `crate::blocks::dummy_assets` を直接参照しない理由（コードフェンス自己完結）
//!
//! `crate::blocks::dummy_assets` は `pub(crate)` のため、Markdown 原稿の
//! Rust コードフェンス（`// blocks-code:begin`/`:end` マーカー内、クレート
//! 外から読めるコード例として提示される）から `use crate::blocks::
//! dummy_assets;` は単体ではコンパイルできない懸念があるが、実際には他の
//! block（[`super::super::content::content_split_image`] 等）も同じ経路で
//! `use crate::blocks::dummy_assets;` を持ち込んでおり、原稿は「実装からの
//! 引用」であって単体コンパイル対象ではない契約（`blocks_code_drift.rs` は
//! バイト一致のみを検証し、原稿単体のコンパイルは要求しない）のため、本
//! block も同じ経路を採用する。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たず、値は一切送信されない。文言はすべて架空のもの（実企業名・実
//! クレデンシャル・PII を含まない）。画像は
//! [`crate::blocks::dummy_assets`] のビルド時生成プレースホルダー SVG のみ
//! を使い、`alt=""`（装飾扱い）で出力する（`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「link の `href` を固定の外部絶対
/// URL にする」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の幾何アイコン（線画。モジュール doc「C: チェック付き項目の
/// アイコン」節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を
/// 明示し、`icon` の `<svg>` 側が固定で持つ `fill="currentColor"`
/// （塗り面）を上書きして線画（ストローク）として描画する。
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

/// A/D で共通のコピー列（eyebrow badge + 見出し + 本文 + CTA ボタン 1 つ）。
fn copy_column(
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
    cta: &'static str,
) -> Node {
    div(
        vec![("class", "blocks-cta-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text(eyebrow)]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
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
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(cta)],
            ),
        ],
    )
}

/// 形 A（R0883 基準形）: 画像 1 枚（片側）+ コピー列（反対側）。DOM 順は
/// 画像 → コピー列（lg 未満の 1 列表示で画像が上に来るようにするため）。
fn variant_basic() -> Node {
    div(
        vec![("class", "blocks-cta-split-image-basic-grid")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
                },
                vec![("data-blocks-cta-split-image-basic-image", "")],
            ),
            copy_column(
                "新機能",
                "導入から公開まで、迷わず進める",
                "テンプレートと雛形を組み合わせ、初期構築にかかる時間を短縮します。",
                "今すぐ試す",
            ),
        ],
    )
}

/// 形 B（R0448/R0450/R0875 集約）: 暗色またはアクセント色のカードへ収め、
/// 画像をカード端まで全面に広げる。`tone`（`"dark"`/`"accent"`）で 2
/// インスタンス描画する（モジュール doc「B: カードの端まで全面に画像を
/// 広げる方法」節）。
fn variant_card(
    tone: &'static str,
    title: &'static str,
    body: &'static str,
    cta: &'static str,
) -> Node {
    let copy = card::body(
        vec![("data-blocks-cta-split-image-card-copy", "")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(body)]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-cta-split-image-cta", "")],
                vec![text(cta)],
            ),
        ],
    );

    let media = div(
        vec![("class", "blocks-cta-split-image-card-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-cta-split-image-card-image", "")],
        )],
    );

    card::root(
        CardProps::default(),
        vec![
            ("data-blocks-cta-split-image-card", ""),
            ("data-blocks-cta-split-image-tone", tone),
        ],
        vec![media, copy],
    )
}

/// チェック付き項目 1 件分（チェックマーク + 短い文言）。
fn checklist_item(label: &'static str) -> Node {
    li(
        vec![("class", "blocks-cta-split-image-checklist-item")],
        vec![check_icon(), text(label)],
    )
}

/// チェック付き項目 6 件（架空文言）。
const CHECKLIST_ITEMS: [&str; 6] = [
    "既定エスケープ済みの HTML 出力",
    "外部依存ゼロの描画コア",
    "SSR / SSG / CSR の切り替え",
    "型で保証されたコンポーネント境界",
    "決定的なビルド成果物",
    "単一実行ファイルでの配布",
];

/// 形 C（R0884）: カード内に写真 + 見出し + 本文 + チェック付き項目 6 件 +
/// 誘導リンク 1 本。
fn variant_checklist() -> Node {
    let media = div(
        vec![("class", "blocks-cta-split-image-checklist-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-cta-split-image-checklist-image", "")],
        )],
    );

    let copy = div(
        vec![("class", "blocks-cta-split-image-checklist-copy")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("導入前に確認したいポイント")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "既存のチームでもそのまま採用できるよう、次の点を満たしています。",
                )],
            ),
            ul(
                vec![("class", "blocks-cta-split-image-checklist-list")],
                CHECKLIST_ITEMS
                    .iter()
                    .map(|label| checklist_item(label))
                    .collect(),
            ),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("導入事例を見る")],
            ),
        ],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-cta-split-image-checklist-card", "")],
        vec![div(
            vec![("class", "blocks-cta-split-image-checklist-grid")],
            vec![media, copy],
        )],
    )
}

/// タイル 4 枚（同じ絵が並ばないよう 4 種のダミー素材を使う）。
fn tiles() -> Node {
    let sources = [
        dummy_assets::PRODUCT_SRC,
        dummy_assets::AVATAR_SRC,
        dummy_assets::LOGO_SRC,
        dummy_assets::BACKGROUND_SRC,
    ];
    div(
        vec![("class", "blocks-cta-split-image-tiles")],
        sources
            .iter()
            .map(|src| {
                image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        shape: ImageShape::Rounded,
                        ..ImageProps::new(src, "")
                    },
                    vec![("data-blocks-cta-split-image-tile-image", "")],
                )
            })
            .collect(),
    )
}

/// 形 D（R0885）: 画像 1 枚の代わりに 2×2 のタイル 4 枚 + コピー列。
fn variant_tiles() -> Node {
    div(
        vec![("class", "blocks-cta-split-image-tiles-grid")],
        vec![
            tiles(),
            copy_column(
                "ギャラリー",
                "実際の画面をひとまとめに確認",
                "複数の切り口から成果物を見せたいときに、タイル配置で並べて提示できます。",
                "詳しく見る",
            ),
        ],
    )
}

/// `cta-split-image` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数（モジュール doc「4 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cta-split-image-layout")],
        vec![
            variant_label("画像 1 枚 + コピー列（R0883 基準形）"),
            variant_basic(),
            variant_label("暗色カード + 全面画像（R0875/R0450 集約）"),
            variant_card(
                "dark",
                "リリースノートを自動でまとめる",
                "コミット履歴から変更点を抽出し、公開前に要点だけを確認できます。",
                "詳細を見る",
            ),
            variant_label("アクセント色カード + 全面画像（R0448 集約）"),
            variant_card(
                "accent",
                "チームの合言葉を、そのまま画面へ",
                "配色トークンを差し替えるだけで、複数ブランドの画面を作り分けられます。",
                "始める",
            ),
            variant_label("チェック付き項目（R0884）"),
            variant_checklist(),
            variant_label("タイル配置（R0885）"),
            variant_tiles(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/cta-split-image/",
    title: "cta-split-image",
    category: BlockCategory::Cta,
    rust_source: "crates/docs-site/src/blocks/marketing/cta/cta_split_image.rs",
    demo_class: "blocks-cta-split-image",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `cta_split_image` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css`
/// で連結される）。
///
/// セレクタは `.blocks-cta-split-image-*` と
/// `[data-blocks-cta-split-image-*]` のみを用い、他 block や部品の素の
/// セレクタへ影響させない（既存 block と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-cta-split-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-cta-split-image-basic-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-cta-split-image-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: start;\n  min-width: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-cta-split-image-basic-image] {\n  display: block;\n  width: 100%;\n  height: 14rem;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-split-image-card][data-blocks-cta-split-image-tone=\"dark\"] {\n  padding: 0;\n  overflow: hidden;\n  border: 0;\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-split-image-card][data-blocks-cta-split-image-tone=\"accent\"] {\n  padding: 0;\n  overflow: hidden;\n  border: 0;\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-cta-split-image-card-media {\n  height: 12rem;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-cta-split-image-card-image] {\n  display: block;\n  width: 100%;\n  height: 100%;\n  border: 0;\n}\n\
[data-scope=\"card\"][data-part=\"body\"][data-blocks-cta-split-image-card-copy] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: start;\n  padding: var(--fandhe-space-6);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-cta-split-image-cta] {\n  background: var(--fandhe-color-bg);\n  color: var(--fandhe-color-fg);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-split-image-checklist-card] {\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-cta-split-image-checklist-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-cta-split-image-checklist-media {\n  height: 12rem;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-cta-split-image-checklist-image] {\n  display: block;\n  width: 100%;\n  height: 100%;\n}\n\
.blocks-cta-split-image-checklist-copy {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-cta-split-image-checklist-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  margin: 0;\n  padding: 0;\n  list-style: none;\n}\n\
.blocks-cta-split-image-checklist-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-cta-split-image-tiles-grid {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-cta-split-image-tiles {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-cta-split-image-tile-image] {\n  display: block;\n  width: 100%;\n  height: 6rem;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-cta-split-image-basic-grid {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: center;\n  }\n  \
[data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-split-image-card][data-blocks-cta-split-image-tone=\"dark\"] {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  \
[data-scope=\"card\"][data-part=\"root\"][data-blocks-cta-split-image-card][data-blocks-cta-split-image-tone=\"accent\"] {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  \
.blocks-cta-split-image-card-media {\n    height: 100%;\n  }\n  \
[data-scope=\"card\"][data-part=\"body\"][data-blocks-cta-split-image-card-copy] {\n    justify-content: center;\n  }\n  \
.blocks-cta-split-image-checklist-grid {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: center;\n  }\n  \
.blocks-cta-split-image-checklist-media {\n    height: 100%;\n  }\n  \
.blocks-cta-split-image-tiles-grid {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    gap: var(--fandhe-space-8);\n    align-items: center;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"card\"",
            "data-scope=\"icon\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // A(1) + B(1x2) + C(1) + D(4) = 8 枚。
        assert_eq!(html.matches("<img").count(), 8);
        assert_eq!(
            html.matches("blocks-cta-split-image-checklist-item")
                .count(),
            6
        );
        assert!(html.contains("type=\"button\""));
        for absent in ["<form", "src=\"data:", "href=\"#\"", " id=\""] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・カード反転配色の
    /// tone セレクタを持つこと。
    #[test]
    fn layout_css_declares_breakpoint_and_card_tones() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("data-blocks-cta-split-image-tone=\"dark\""));
        assert!(LAYOUT_CSS.contains("data-blocks-cta-split-image-tone=\"accent\""));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-cta-split-image-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-cta-split-image-layout");
    }

    /// dummy_assets の 4 種すべてがタイル形（D）で使われ、同じ絵が並ばない
    /// こと。
    #[test]
    fn tiles_use_four_distinct_dummy_assets() {
        let html = render(&demo());
        for src in [
            dummy_assets::PRODUCT_SRC,
            dummy_assets::AVATAR_SRC,
            dummy_assets::LOGO_SRC,
            dummy_assets::BACKGROUND_SRC,
        ] {
            assert!(html.contains(src), "demo output should reference {src}");
        }
    }
}
