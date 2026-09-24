//! Blocks 共通のデモ用ダミー素材ヘルパ（イシュー #2737）。
//!
//! # 役割・呼び出し文脈
//!
//! 親トラッキング #2731「Blocks 目的別パーツ拡充ツリー」配下で追加される
//! 目的別パーツ block の多くが、商品画像・人物アバター・会社ロゴ・
//! スクリーンショット枠・背景タイルといったプレースホルダー画像や、
//! 人名・社名・価格帯・グラフ用サンプル系列といったダミー文言を必要と
//! する。block ごとに個別実装するとブレ・重複が生じるため、本モジュールへ
//! 一元化する。既存 22 block（`crate::blocks::all_blocks()` 経由で登録
//! 済みのもの）への適用（置き換え）は任意であり、本モジュール新設の
//! スコープには含めない。
//!
//! # `data:` URI を使わない理由（`crate::showcase::image_demo_svg` と同方針）
//!
//! [`fandhe_frontend_core::url::is_safe_url`]（許可スキームは http/https/
//! mailto/tel と相対 URL のみ、REQ-1）は `data:` を拒否し `src` 属性ごと
//! 出力から落とす。旧 Image 節デモが `data:image/svg+xml,...` を使い
//! 「壊れた画像アイコン」表示になっていた不具合（イシュー #1562）と同じ
//! 教訓により、本モジュールの画像ヘルパもすべてビルド時生成の SVG を
//! 相対パスアセットとして書き出す方式に統一する（`crate::build::build_site`
//! が [`IMAGE_ASSETS`] を走査して書き出す。`crate::showcase::image_demo_svg`
//! と同型）。
//!
//! # モノトーン・抽象図形限定の方針
//!
//! 5 種の画像ヘルパはいずれも実在の人物・企業・ブランドを模さない、
//! グレースケールの抽象図形のみで構成する（本モジュール内 `const` の
//! 共通パレットを使い回す）。文言・数値セットも、既知の実データセット
//! 由来の名称（Contoso/Northwind 等）を避けた完全架空のセットである。
//!
//! # 使い方
//!
//! block 実装（`crate::blocks::<section>::<category>::<block>`）からは
//! 画像は `Src` 定数、文言・数値は各 `const` 配列/スライスを直接参照する。
//!
//! ```rust,ignore
//! // 本モジュールは pub(crate) 可視性のためクレート外からは実行できない
//! // （doctest ではなく使用例としてのみ示す）。
//! use fandhe_frontend_pre_styled_ui::image::{image, ImageProps};
//! use fandhe_frontend_core::text;
//!
//! // 人物アバター画像を image パーツへ渡す。
//! let avatar = image(&ImageProps::new(
//!     dummy_assets::AVATAR_SRC,
//!     "プレースホルダーのアバター画像",
//! ));
//!
//! // 架空の人名をそのままテキストノードへ渡す（既定エスケープ経由）。
//! let name = text(dummy_assets::PERSON_NAMES[0]);
//! ```
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! SVG・文言とも [`fandhe_frontend_core`] のノード木 API（`el`/`render`/
//! `text`）のみで組み立てる。`format!("<svg>{}</svg>", ...)` のような
//! 文字列直接組み立ては行わない。生成 SVG は `<script`・イベントハンドラ
//! 属性（`on*=`）を一切含まない（`tests` モジュールで固定）。

// 本モジュールは #2731「Blocks 目的別パーツ拡充ツリー」配下の後続 block
// から利用される共通ヘルパであり、本モジュール新設自体（イシュー #2737）の
// スコープでは `crate::blocks` の他モジュールから未参照のままとなる
// 定数・関数がある（`Src` 定数・文言/数値セット等）。`cargo clippy -- -D
// warnings`（CI）を dead_code 警告で赤くしないための意図的な allow であり、
// 後続 block が実際に参照し始めた時点で個別に外していく想定はしない
// （ヘルパ全体が「将来の消費者向けに公開する API 集合」であるため）。
#![allow(dead_code)]

use fandhe_frontend_core::{el, el_owned, render};

/// モノトーン画像ヘルパ 5 種に共通する背景色（明るいグレー）。
const PALETTE_BG: &str = "#e5e7eb";
/// 輪郭線・二次的な図形に使う中間グレー。
const PALETTE_OUTLINE: &str = "#9ca3af";
/// 強調図形（アイコン本体等）に使う濃いグレー。
const PALETTE_ACCENT: &str = "#6b7280";

/// 商品プレースホルダー画像の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が [`IMAGE_ASSETS`] 経由でこのパスへ
/// [`product_svg`] の内容を書き出す。
pub(crate) const PRODUCT_ASSET_REL_PATH: &str = "assets/blocks-demo-product.svg";
/// block の Demo から `image::ImageProps::new` 等へ渡す `src`（`/blocks/<kebab>/`
/// ページ → `assets/` の相対パス。`crate::showcase::IMAGE_DEMO_SRC` と同じ
/// 深さ 2 階層上の想定）。
pub(crate) const PRODUCT_SRC: &str = "../../assets/blocks-demo-product.svg";

/// 人物アバターのプレースホルダー画像の出力先。
pub(crate) const AVATAR_ASSET_REL_PATH: &str = "assets/blocks-demo-avatar.svg";
/// [`AVATAR_ASSET_REL_PATH`] に対応する `src`。
pub(crate) const AVATAR_SRC: &str = "../../assets/blocks-demo-avatar.svg";

/// 会社ロゴのプレースホルダー画像の出力先。
pub(crate) const LOGO_ASSET_REL_PATH: &str = "assets/blocks-demo-logo.svg";
/// [`LOGO_ASSET_REL_PATH`] に対応する `src`。
pub(crate) const LOGO_SRC: &str = "../../assets/blocks-demo-logo.svg";

/// スクリーンショット枠のプレースホルダー画像の出力先。
pub(crate) const SCREENSHOT_ASSET_REL_PATH: &str = "assets/blocks-demo-screenshot.svg";
/// [`SCREENSHOT_ASSET_REL_PATH`] に対応する `src`。
pub(crate) const SCREENSHOT_SRC: &str = "../../assets/blocks-demo-screenshot.svg";

/// 汎用背景タイルのプレースホルダー画像の出力先。
pub(crate) const BACKGROUND_ASSET_REL_PATH: &str = "assets/blocks-demo-background.svg";
/// [`BACKGROUND_ASSET_REL_PATH`] に対応する `src`。
pub(crate) const BACKGROUND_SRC: &str = "../../assets/blocks-demo-background.svg";

/// [`IMAGE_ASSETS`] の要素型（`(出力先の相対パス, 生成関数)`）。
/// clippy `type_complexity` を避けるための型エイリアス。
pub(crate) type ImageAsset = (&'static str, fn() -> String);

/// 画像ヘルパ 5 種のディスパッチテーブル（`crate::blocks::Block` レジストリ
/// と同型の「単一情報源」設計）。`crate::build::build_site` はこの配列を
/// 1 ループ走査するだけで、`asset_hrefs` への href 登録・
/// `generated_assets` への書き出しの両方を行える（個々の定数を手書きで
/// 列挙しない）。要素は `(出力先の相対パス, 生成関数)`。
pub(crate) const IMAGE_ASSETS: &[ImageAsset] = &[
    (PRODUCT_ASSET_REL_PATH, product_svg),
    (AVATAR_ASSET_REL_PATH, avatar_svg),
    (LOGO_ASSET_REL_PATH, logo_svg),
    (SCREENSHOT_ASSET_REL_PATH, screenshot_svg),
    (BACKGROUND_ASSET_REL_PATH, background_svg),
];

/// 商品プレースホルダー画像（角丸枠 + 抽象的な梱包箱アイコン）。
#[must_use]
pub(crate) fn product_svg() -> String {
    let svg = el(
        "svg",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("viewBox", "0 0 160 160"),
            ("role", "img"),
            ("aria-label", "Placeholder product illustration"),
        ],
        vec![
            el(
                "rect",
                vec![
                    ("x", "4"),
                    ("y", "4"),
                    ("width", "152"),
                    ("height", "152"),
                    ("rx", "12"),
                    ("fill", PALETTE_BG),
                    ("stroke", PALETTE_OUTLINE),
                    ("stroke-width", "2"),
                ],
                vec![],
            ),
            // 梱包箱の輪郭（上面 + 側面の分割線）。
            el(
                "path",
                vec![
                    ("d", "M50 60 L80 44 L110 60 L110 106 L80 122 L50 106 Z"),
                    ("fill", "none"),
                    ("stroke", PALETTE_ACCENT),
                    ("stroke-width", "4"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M50 60 L80 76 L110 60 M80 76 L80 122"),
                    ("fill", "none"),
                    ("stroke", PALETTE_ACCENT),
                    ("stroke-width", "4"),
                    ("stroke-linejoin", "round"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    );
    render(&svg)
}

/// 人物アバターのプレースホルダー画像（円形キャンバスに頭部の円 + 肩の弧）。
#[must_use]
pub(crate) fn avatar_svg() -> String {
    let svg = el(
        "svg",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("viewBox", "0 0 96 96"),
            ("role", "img"),
            ("aria-label", "Placeholder avatar illustration"),
        ],
        vec![
            el(
                "circle",
                vec![
                    ("cx", "48"),
                    ("cy", "48"),
                    ("r", "46"),
                    ("fill", PALETTE_BG),
                ],
                vec![],
            ),
            // 頭部。
            el(
                "circle",
                vec![
                    ("cx", "48"),
                    ("cy", "38"),
                    ("r", "16"),
                    ("fill", PALETTE_ACCENT),
                ],
                vec![],
            ),
            // 肩（円の外へはみ出す部分は親 `<svg>` のクリップに任せる）。
            el(
                "path",
                vec![("d", "M16 92 Q48 62 80 92 Z"), ("fill", PALETTE_ACCENT)],
                vec![],
            ),
        ],
    );
    render(&svg)
}

/// 会社ロゴのプレースホルダー画像（六角形 + 中心円の抽象バッジ）。
#[must_use]
pub(crate) fn logo_svg() -> String {
    let svg = el(
        "svg",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("viewBox", "0 0 96 96"),
            ("role", "img"),
            ("aria-label", "Placeholder logo illustration"),
        ],
        vec![
            el(
                "polygon",
                vec![
                    ("points", "48,4 86,26 86,70 48,92 10,70 10,26"),
                    ("fill", PALETTE_BG),
                    ("stroke", PALETTE_OUTLINE),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "48"),
                    ("cy", "48"),
                    ("r", "20"),
                    ("fill", PALETTE_ACCENT),
                ],
                vec![],
            ),
        ],
    );
    render(&svg)
}

/// スクリーンショット枠のプレースホルダー画像（ブラウザ枠 + 本文プレース
/// ホルダー横線数本 + 画像枠 1 個）。
#[must_use]
pub(crate) fn screenshot_svg() -> String {
    let svg = el(
        "svg",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("viewBox", "0 0 200 140"),
            ("role", "img"),
            ("aria-label", "Placeholder screenshot illustration"),
        ],
        vec![
            // ブラウザ外枠。
            el(
                "rect",
                vec![
                    ("x", "2"),
                    ("y", "2"),
                    ("width", "196"),
                    ("height", "136"),
                    ("rx", "6"),
                    ("fill", "#ffffff"),
                    ("stroke", PALETTE_OUTLINE),
                    ("stroke-width", "2"),
                ],
                vec![],
            ),
            // 上部バー。
            el(
                "rect",
                vec![
                    ("x", "2"),
                    ("y", "2"),
                    ("width", "196"),
                    ("height", "18"),
                    ("rx", "6"),
                    ("fill", PALETTE_BG),
                ],
                vec![],
            ),
            // 信号灯（3 つの丸）。
            el(
                "circle",
                vec![
                    ("cx", "14"),
                    ("cy", "11"),
                    ("r", "3"),
                    ("fill", PALETTE_OUTLINE),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "26"),
                    ("cy", "11"),
                    ("r", "3"),
                    ("fill", PALETTE_OUTLINE),
                ],
                vec![],
            ),
            el(
                "circle",
                vec![
                    ("cx", "38"),
                    ("cy", "11"),
                    ("r", "3"),
                    ("fill", PALETTE_OUTLINE),
                ],
                vec![],
            ),
            // 本文プレースホルダー画像枠。
            el(
                "rect",
                vec![
                    ("x", "16"),
                    ("y", "30"),
                    ("width", "56"),
                    ("height", "40"),
                    ("rx", "4"),
                    ("fill", PALETTE_BG),
                    ("stroke", PALETTE_OUTLINE),
                    ("stroke-width", "1.5"),
                ],
                vec![],
            ),
            // 本文プレースホルダー横線（複数行）。
            el(
                "rect",
                vec![
                    ("x", "84"),
                    ("y", "32"),
                    ("width", "100"),
                    ("height", "8"),
                    ("rx", "3"),
                    ("fill", PALETTE_OUTLINE),
                ],
                vec![],
            ),
            el(
                "rect",
                vec![
                    ("x", "84"),
                    ("y", "48"),
                    ("width", "84"),
                    ("height", "8"),
                    ("rx", "3"),
                    ("fill", PALETTE_OUTLINE),
                ],
                vec![],
            ),
            el(
                "rect",
                vec![
                    ("x", "84"),
                    ("y", "64"),
                    ("width", "92"),
                    ("height", "8"),
                    ("rx", "3"),
                    ("fill", PALETTE_OUTLINE),
                ],
                vec![],
            ),
            el(
                "rect",
                vec![
                    ("x", "16"),
                    ("y", "84"),
                    ("width", "168"),
                    ("height", "8"),
                    ("rx", "3"),
                    ("fill", PALETTE_OUTLINE),
                ],
                vec![],
            ),
            el(
                "rect",
                vec![
                    ("x", "16"),
                    ("y", "100"),
                    ("width", "140"),
                    ("height", "8"),
                    ("rx", "3"),
                    ("fill", PALETTE_OUTLINE),
                ],
                vec![],
            ),
        ],
    );
    render(&svg)
}

/// 汎用背景タイル（320×180、ドットパターンをモノトーンで敷く）。
#[must_use]
pub(crate) fn background_svg() -> String {
    let mut dots = Vec::new();
    let mut y = 10;
    while y < 180 {
        let mut x = 10;
        while x < 320 {
            // 座標は実行時に組み立てる文字列のため `el_owned`（所有権付き
            // 属性値）を使う。`el`（`&'static str` 属性のみ）では表現できない。
            dots.push(el_owned(
                "circle",
                vec![
                    ("cx".to_string(), x.to_string()),
                    ("cy".to_string(), y.to_string()),
                    ("r".to_string(), "2".to_string()),
                    ("fill".to_string(), PALETTE_OUTLINE.to_string()),
                ],
                vec![],
            ));
            x += 20;
        }
        y += 20;
    }

    let mut children = vec![el(
        "rect",
        vec![("width", "320"), ("height", "180"), ("fill", PALETTE_BG)],
        vec![],
    )];
    children.extend(dots);

    let svg = el(
        "svg",
        vec![
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("viewBox", "0 0 320 180"),
            ("role", "img"),
            ("aria-label", "Placeholder background tile"),
        ],
        children,
    );
    render(&svg)
}

/// 架空フルネームのセット（人物系デモ用、実在人物とは無関係）。
pub(crate) const PERSON_NAMES: &[&str] = &[
    "Haruto Fujimaki",
    "Elena Vasquez",
    "Kwame Boateng",
    "Mei Lindqvist",
    "Noor Al-Sayed",
    "Ola Bergström",
    "Priya Chandran",
    "Théo Marchetti",
];

/// [`PERSON_NAMES`] と対にして使う架空の役職名。
pub(crate) const JOB_TITLES: &[&str] = &[
    "Product Designer",
    "Engineering Lead",
    "Customer Success Manager",
    "Data Analyst",
    "Marketing Strategist",
    "Operations Coordinator",
];

/// 架空社名のセット（既知の実データセット由来名称、実在企業名を避ける）。
pub(crate) const COMPANY_NAMES: &[&str] = &[
    "Lumenbridge Systems",
    "Verdant Foundry",
    "Quill & Meridian",
    "Trellisworks Co.",
    "Aurelia Dynamics",
    "Northshelf Logistics",
];

/// 一言レビュー（実企業・実サービス名を含まない）のサンプルセット。
pub(crate) const TESTIMONIAL_QUOTES: &[&str] = &[
    "導入からわずか数週間で、チーム全体の作業が驚くほど整理されました。",
    "細部まで作り込まれた操作感で、初めて触ったメンバーもすぐに馴染めました。",
    "サポートの対応が丁寧で、困ったときにいつも助けてもらっています。",
    "他のツールと比べて圧倒的にシンプルで、迷わず使い続けられています。",
];

/// 価格帯サンプル（ティア名, 価格表示）。
pub(crate) const SAMPLE_PRICE_TIERS: &[(&str, &str)] =
    &[("Starter", "$9"), ("Growth", "$29"), ("Scale", "$79")];

/// グラフ用サンプルカテゴリ（横軸ラベル）。
pub(crate) const SAMPLE_CHART_CATEGORIES: &[&str] = &["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

/// グラフ用サンプル系列 A（[`SAMPLE_CHART_CATEGORIES`] と同じ長さ）。
pub(crate) const SAMPLE_CHART_SERIES_A: &[f64] = &[12.0, 18.0, 15.0, 24.0, 21.0, 30.0];

/// グラフ用サンプル系列 B（[`SAMPLE_CHART_CATEGORIES`] と同じ長さ）。
pub(crate) const SAMPLE_CHART_SERIES_B: &[f64] = &[8.0, 10.0, 14.0, 13.0, 19.0, 22.0];

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_safe_svg(svg: &str) {
        assert!(svg.contains("<svg"), "svg root element must be present");
        assert!(!svg.contains("data:"), "svg must not embed a data: URI");
        assert!(!svg.contains("<script"), "svg must not contain <script");
        // `on\w+=` 相当のイベントハンドラ属性が含まれないことを確認する
        // （`onload=`/`onclick=` 等、固定図形のみのため本来出現し得ないが
        // 回帰として固定する）。
        assert!(!svg.contains("onload="));
        assert!(!svg.contains("onclick="));
        assert!(!svg.contains("onerror="));
    }

    #[test]
    fn product_svg_is_safe() {
        assert_safe_svg(&product_svg());
    }

    #[test]
    fn avatar_svg_is_safe() {
        assert_safe_svg(&avatar_svg());
    }

    #[test]
    fn logo_svg_is_safe() {
        assert_safe_svg(&logo_svg());
    }

    #[test]
    fn screenshot_svg_is_safe() {
        assert_safe_svg(&screenshot_svg());
    }

    #[test]
    fn background_svg_is_safe() {
        assert_safe_svg(&background_svg());
    }

    #[test]
    fn image_assets_dispatch_table_matches_generators() {
        // `IMAGE_ASSETS` の走査だけで 5 種すべてが非空の SVG を生成できる
        // ことを固定する（`crate::build::build_site` 側の単一ループ実装が
        // 前提とする契約）。
        assert_eq!(IMAGE_ASSETS.len(), 5);
        for (rel_path, generator) in IMAGE_ASSETS {
            assert!(rel_path.starts_with("assets/blocks-demo-"));
            let svg = generator();
            assert_safe_svg(&svg);
        }
    }

    #[test]
    fn text_constants_do_not_contain_markup_characters() {
        // 文言セットは `text()` 経由でのみ出力される想定だが、念のため
        // `<`/`>` を含まないことを固定する（既定エスケープの迂回にはならない
        // が、ダミーデータ自体に構造化文字を混入させない回帰）。
        for name in PERSON_NAMES {
            assert!(!name.contains('<') && !name.contains('>'));
        }
        for title in JOB_TITLES {
            assert!(!title.contains('<') && !title.contains('>'));
        }
        for company in COMPANY_NAMES {
            assert!(!company.contains('<') && !company.contains('>'));
        }
        for quote in TESTIMONIAL_QUOTES {
            assert!(!quote.contains('<') && !quote.contains('>'));
        }
        for (tier, price) in SAMPLE_PRICE_TIERS {
            assert!(!tier.contains('<') && !tier.contains('>'));
            assert!(!price.contains('<') && !price.contains('>'));
        }
        for category in SAMPLE_CHART_CATEGORIES {
            assert!(!category.contains('<') && !category.contains('>'));
        }
    }

    #[test]
    fn chart_sample_series_length_matches_categories() {
        assert_eq!(SAMPLE_CHART_CATEGORIES.len(), SAMPLE_CHART_SERIES_A.len());
        assert_eq!(SAMPLE_CHART_CATEGORIES.len(), SAMPLE_CHART_SERIES_B.len());
    }
}
