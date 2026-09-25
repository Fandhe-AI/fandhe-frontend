//! `feature-split-list-image` block（イシュー #2769。親トラッキング
//! #2730/#2738「Blocks 目的別パーツ拡充ツリー、Phase 1 マーケティング A」
//! 配下）。左列に見出し・説明 + feature 一覧、右列に画像（1 枚または
//! 複数枚の組）を置く 2 列 feature セクションの合成例（`marketing/feature`
//! の 6 件目）。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （対応表 ID は R0486（主参照）・R0105・R0102・R0484・R0477・R0489・
//! R0487・R1155・R1157・R1161 のみを記す）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `icon` / `card` / `image` / `button` /
//! `data-list` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新規 UI 部品は追加しない。
//!
//! # 10 参照を 4 インスタンスへ統合する
//!
//! イシューが挙げる参照群を、`feature_image_cards`/`careers_card_grid` と
//! 同じく **4 インスタンスを縦に並べる**構成へ統合する。各インスタンスの
//! 直前に、集約元の差分を短く説明する注記（[`TextSize::Xs`] +
//! [`TextVariant::Muted`]）を置く（`feature_image_cards` の `note_a`/
//! `note_b` と同じ形）。
//!
//! - **インスタンス A**（R0486 を基準形とし R0105 を統合）: badge（eyebrow）
//!   と H3 見出し + リード文の下へ、`card::root` 4 枚を 2 列グリッドで並べる
//!   feature 一覧。右列は縦長画像 1 枚。
//! - **インスタンス B**（R0102・R0484・R0477 を統合）: H3 見出し + リード文
//!   の下へ、アイコン付き行を 3 項目縦に並べる feature 一覧。右列は正方形
//!   画像 1 枚。DOM 順を「画像 → 一覧」にする唯一のインスタンス（本節末尾
//!   「DOM 順と視覚順」参照）。
//! - **インスタンス C**（R0489・R0487 を統合）: H3 見出し + リード文の下へ、
//!   数値を主役にしたカードを 2 枚並べ、その下にボタン行を置く feature
//!   一覧。右列は正方形画像 1 枚。
//! - **インスタンス D**（R1155・R1157・R1161 を統合）: H3 見出し + リード文
//!   の下へ、`data_list::root` の定義リスト 4 項目。右列は正方形画像 4 枚を
//!   2×2 グリッドで組む。
//!
//! # レスポンシブ（64rem をブレークポイントとする理由）
//!
//! `< 64rem`（lg 未満）は一覧・画像の 2 列を 1 列へ畳む。`>= 64rem` で
//! `grid-template-columns: repeat(2, minmax(0, 1fr))` の 2 列へ切り替える。
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できないため
//! （CSS custom property は宣言側でのみ有効）、
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint`] の `Lg`（1024px =
//! 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする
//! （`feature_image_cards`/`careers_card_grid` と同じ判断）。インスタンス A
//! の一覧内カードグリッドは md（48rem）以上で 2 列になる（`feature_image_cards`
//! と同じ 2 段ブレークポイント）。
//!
//! # DOM 順と視覚順（インスタンス B のみ画像を先に置く）
//!
//! インスタンス A/C/D は DOM 順「一覧 → 画像」のまま、`@media (min-width:
//! 64rem)` で 2 列グリッドに切り替わると画像が右列へ配置される（グリッド
//! アイテムの並び順は DOM 順を保つため、`order` プロパティは使わない）。
//! インスタンス B のみ R0105 の「狭幅で画像が上・lg で画像が左」という要件
//! を満たすため、**DOM 順自体を「画像 → 一覧」に入れ替える**（CSS の
//! `order` で視覚順だけを操作すると、キーボード操作・スクリーンリーダーの
//! 読み上げ順が視覚順と食い違うため、DOM 順の入れ替えで対処する）。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由・`card::title` を使わない理由
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! [`HeadingLevel::H3`] にする（`feature_image_cards` と同じ判断）。
//! feature 一覧内の短い見出しはそれより 1 段下げて [`HeadingLevel::H4`]
//! にし、[`card::title`]（`<h3>` 固定）は使わない（セクション見出しと同じ
//! 見出しレベルが 1 ページに重複するのを避けるため、`careers_card_grid`
//! と同じ判断）。
//!
//! # インスタンス D の `<dt>` に heading 要素を入れない理由
//!
//! HTML の内容モデル上 `dt` は見出し要素を子孫に持てないため、定義リストの
//! 短い見出しは [`heading::heading`] ではなく、太字の [`styled_text::text`]
//! （[`TextWeight::Bold`]）で表す。
//!
//! # 自作幾何アイコン
//!
//! lucide 等の既存アイコンセットの path を複製しないため、
//! `feature_expand::geo_icon`/`careers_card_grid::geo_icon` と同型の自作
//! ヘルパ [`geo_icon`] で描く。`path` へ `fill="none"` +
//! `stroke="currentColor"` を明示し、`icon::icon` の `<svg>` 側が固定で
//! 持つ `fill="currentColor"`（塗り面）を上書きして線画として描画する。
//! いずれも隣に可視テキストがあるため装飾扱い（`IconProps::label` は
//! `None` のまま、`aria-hidden="true"`）とする。
//!
//! # alt を空文字列にする理由
//!
//! いずれの画像も隣接する見出し・説明で内容が伝わる装飾用途のため、
//! `alt=""` にする（`feature_alternating_rows`/`feature_image_cards` の前例
//! と同じ判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `styled_text::text` /
//! `icon::icon` / `card::root` / `image::image` / `button::button` /
//! `data_list::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-feature-split-list-image-*` 属性で渡す。素の
//! `div`・`card::header`/`card::body`（variant を持たず `attrs` をそのまま
//! 連結する）・`data_list::item`/`item_label`/`item_value` には `class`
//! （`.blocks-feature-split-list-image-*`）がそのまま効く。ルート要素の
//! class（`blocks-feature-split-list-image-layout`）は [`Block::demo_class`]
//! （`blocks-feature-split-list-image`）とは意図的に別名にする
//! （`feature_image_cards`/`careers_card_grid` と同じ Bugbot 教訓の回避）。
//!
//! # 詳細度の罠（`image`/`text` recipe への勝ち方）
//!
//! `image::image`・`styled_text::text` の recipe（詳細度 (0,2,0)）に確実に
//! 勝つため、上書きは `[data-scope="image"][data-part="root"][data-blocks-
//! feature-split-list-image-*]` / `[data-scope="text"][data-part="root"]
//! [data-blocks-feature-split-list-image-*]` の 3 セレクタ構成（詳細度
//! (0,3,0)）で行う（`feature_image_cards` と同型の判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text` と
//! `fandhe_frontend_core::text` が同名のため、styled 側を `styled_text`
//! として取り込む（`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・`id` を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。インスタンス C のボタン行は [`ButtonProps::default`]（
//! `type="button"` 固定）のまま送信先を持たない見た目のみの導線であり、
//! フォーム送信・XHR は一切行わない。文言はすべて架空のもの（実企業名・
//! 実クレデンシャル・PII を含まない）。画像は
//! [`crate::blocks::dummy_assets`] の各定数（いずれもビルド時生成の
//! プレースホルダー SVG）を使い分け、`alt=""` で出力する。`id` 属性は一切
//! 使わない（重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`careers_card_grid::geo_icon` と同型の判断）。
fn geo_icon(size: Size, path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size,
            ..IconProps::default()
        },
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

/// 円の幾何アイコン（インスタンス A のカード見出しに添える）。
fn circle_icon() -> Node {
    geo_icon(Size::Md, "M12 3a9 9 0 100 18 9 9 0 000-18z")
}

/// 四角の幾何アイコン（インスタンス A のカード見出しに添える）。
fn square_icon() -> Node {
    geo_icon(Size::Md, "M4 4h16v16H4z")
}

/// 折れ線の幾何アイコン（インスタンス A のカード見出しに添える）。
fn zigzag_icon() -> Node {
    geo_icon(Size::Md, "M4 18l5-9 4 6 4-8 3 5")
}

/// 三角の幾何アイコン（インスタンス A のカード見出しに添える）。
fn triangle_icon() -> Node {
    geo_icon(Size::Md, "M12 4l8 16H4z")
}

/// 盾の幾何アイコン（インスタンス B のアイコン付き行に添える）。
fn shield_icon() -> Node {
    geo_icon(Size::Sm, "M12 3l7 3v5c0 5-3.5 8.5-7 10-3.5-1.5-7-5-7-10V6z")
}

/// 稲妻の幾何アイコン（インスタンス B のアイコン付き行に添える）。
fn bolt_icon() -> Node {
    geo_icon(Size::Sm, "M13 3L5 14h5l-1 7 8-11h-5z")
}

/// 歯車の幾何アイコン（インスタンス B のアイコン付き行に添える）。
fn gear_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 8a4 4 0 100 8 4 4 0 000-8z M12 2v3 M12 19v3 M4.2 4.2l2.1 2.1 M17.7 17.7l2.1 2.1 M2 12h3 M19 12h3 M4.2 19.8l2.1-2.1 M17.7 6.3l2.1-2.1",
    )
}

/// 星の幾何アイコン（インスタンス D の定義リストに添える）。
fn star_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 3l2.6 5.6 6.1.6-4.6 4.1 1.3 6-5.4-3.2-5.4 3.2 1.3-6-4.6-4.1 6.1-.6z",
    )
}

/// 波の幾何アイコン（インスタンス D の定義リストに添える）。
fn wave_icon() -> Node {
    geo_icon(Size::Sm, "M3 12c2-3 4-3 6 0s4 3 6 0 4-3 6 0")
}

/// 錠前の幾何アイコン（インスタンス D の定義リストに添える）。
fn lock_icon() -> Node {
    geo_icon(Size::Sm, "M6 11V8a6 6 0 1112 0v3 M5 11h14v9H5z")
}

/// 地球の幾何アイコン（インスタンス D の定義リストに添える）。
fn globe_icon() -> Node {
    geo_icon(
        Size::Sm,
        "M12 3a9 9 0 100 18 9 9 0 000-18z M3 12h18 M12 3c2.5 2.5 2.5 15.5 0 18 M12 3c-2.5 2.5-2.5 15.5 0 18",
    )
}

/// feature 一覧カード 1 件分の架空データ（インスタンス A）。
struct FeatureItem {
    icon_node: fn() -> Node,
    title: &'static str,
    body: &'static str,
}

/// インスタンス A のカードデータ（4 件）。
const FEATURES_A: [FeatureItem; 4] = [
    FeatureItem {
        icon_node: circle_icon,
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成します。",
    },
    FeatureItem {
        icon_node: square_icon,
        title: "型で表現する構造",
        body: "スロットと props は Rust の型で表現されます。",
    },
    FeatureItem {
        icon_node: zigzag_icon,
        title: "外部依存ゼロ",
        body: "描画コアは外部クレートに依存しません。",
    },
    FeatureItem {
        icon_node: triangle_icon,
        title: "単一実行ファイル配布",
        body: "SSR/SSG のいずれも単一バイナリへまとめられます。",
    },
];

/// インスタンス B のアイコン付き行データ（3 件）。
const FEATURES_B: [FeatureItem; 3] = [
    FeatureItem {
        icon_node: shield_icon,
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされます。",
    },
    FeatureItem {
        icon_node: bolt_icon,
        title: "無 JS の静的表示",
        body: "JS ハイドレーションを行わない決定的な表示です。",
    },
    FeatureItem {
        icon_node: gear_icon,
        title: "機械検証可能な構成",
        body: "構造マニフェストが依存関係を機械検証します。",
    },
];

/// インスタンス C の数値カードデータ（2 件）。
struct StatItem {
    value: &'static str,
    title: &'static str,
    body: &'static str,
}

const STATS_C: [StatItem; 2] = [
    StatItem {
        value: "9",
        title: "公開クレート",
        body: "描画コアから CLI までを個別クレートへ分割しています。",
    },
    StatItem {
        value: "60",
        title: "依存上限",
        body: "標準構成の依存パッケージ数は 60 件以内に収めます。",
    },
];

/// インスタンス D の定義リストデータ（4 件）。
struct DefinitionItem {
    icon_node: fn() -> Node,
    term: &'static str,
    detail: &'static str,
}

const DEFINITIONS_D: [DefinitionItem; 4] = [
    DefinitionItem {
        icon_node: star_icon,
        term: "既定エスケープ",
        detail: "迂回経路は明示的なオプトイン API に限られます。",
    },
    DefinitionItem {
        icon_node: wave_icon,
        term: "決定的な出力",
        detail: "同じ入力からは常に同じ静的ファイルを生成します。",
    },
    DefinitionItem {
        icon_node: lock_icon,
        term: "unsafe 境界の限定",
        detail: "描画コア・状態管理コアでは unsafe を使用しません。",
    },
    DefinitionItem {
        icon_node: globe_icon,
        term: "単一バイナリ配布",
        detail: "SSR/SSG を単一実行ファイルへまとめて配布できます。",
    },
];

/// セクション見出し（badge + H3 + リード文）を組み立てる。
fn section_header(eyebrow: &str, title: &str, lead: &str) -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-split-list-image-eyebrow", "")],
                vec![core_text(eyebrow)],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-split-list-image-lead", "")],
                vec![core_text(lead)],
            ),
        ],
    )
}

/// インスタンス A の feature カード 1 枚（アイコン + H4 見出し + 説明）。
fn feature_card(item: &FeatureItem) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-split-list-image-card", "")],
        vec![card::body(
            vec![("class", "blocks-feature-split-list-image-card-body")],
            vec![
                (item.icon_node)(),
                heading::heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Sm,
                        weight: HeadingWeight::Semibold,
                    },
                    vec![],
                    vec![core_text(item.title)],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-feature-split-list-image-card-desc", "")],
                    vec![core_text(item.body)],
                ),
            ],
        )],
    )
}

/// インスタンス B のアイコン付き行 1 件（アイコン列 + (H4 + 説明)）。
fn feature_row(item: &FeatureItem) -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-row")],
        vec![
            div(
                vec![("data-blocks-feature-split-list-image-row-icon", "")],
                vec![(item.icon_node)()],
            ),
            div(
                vec![("class", "blocks-feature-split-list-image-row-text")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Sm,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![core_text(item.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(item.body)],
                    ),
                ],
            ),
        ],
    )
}

/// インスタンス C の数値カード 1 枚（大きな数値 + H4 見出し + 説明）。
fn stat_card(item: &StatItem) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-split-list-image-stat-card", "")],
        vec![card::body(
            vec![("class", "blocks-feature-split-list-image-card-body")],
            vec![
                styled_text::text(
                    &TextProps {
                        size: TextSize::Xl3,
                        weight: TextWeight::Bold,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-feature-split-list-image-stat-value", "")],
                    vec![core_text(item.value)],
                ),
                heading::heading(
                    HeadingLevel::H4,
                    &HeadingProps {
                        size: HeadingSize::Sm,
                        weight: HeadingWeight::Semibold,
                    },
                    vec![],
                    vec![core_text(item.title)],
                ),
                styled_text::text(
                    &TextProps {
                        size: TextSize::Sm,
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![core_text(item.body)],
                ),
            ],
        )],
    )
}

/// インスタンス C 末尾のボタン行（送信先を持たない見た目のみの導線）。
fn stat_button_row() -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-button-row")],
        vec![
            button::button(
                &ButtonProps::default(),
                vec![],
                vec![core_text("詳しく見る")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![core_text("資料をダウンロード")],
            ),
        ],
    )
}

/// インスタンス D の定義リスト 1 項目（アイコン + 短い見出し / 説明）。
fn definition_item(item: &DefinitionItem) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(
                vec![("class", "blocks-feature-split-list-image-dt")],
                vec![
                    (item.icon_node)(),
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Bold,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(item.term)],
                    ),
                ],
            ),
            data_list::item_value(
                vec![],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![core_text(item.detail)],
                )],
            ),
        ],
    )
}

/// 画像 1 枚を組み立てる（`alt=""`、装飾扱い）。
fn split_image(src: &str, aspect_ratio: AspectRatio) -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio,
            ..ImageProps::new(src, "")
        },
        vec![("data-blocks-feature-split-list-image-image", "")],
    )
}

/// インスタンス A・C・D 共通の「一覧 → 画像」2 列レイアウト。
fn split_list_then_image(list: Node, image_col: Node) -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-split")],
        vec![list, image_col],
    )
}

/// インスタンス B 専用の「画像 → 一覧」2 列レイアウト（本モジュール doc
/// 「DOM 順と視覚順」節）。
fn split_image_then_list(image_col: Node, list: Node) -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-split")],
        vec![image_col, list],
    )
}

/// インスタンス A（カード一覧 + 縦長画像 1 枚）。
fn instance_a() -> Node {
    let header = section_header(
        "特長",
        "カード一覧と画像で特長を紹介する",
        "各カードはアイコン・見出し・説明の順に積み、画面幅に応じて 1〜2 列へ切り替わります。",
    );
    let grid = div(
        vec![("class", "blocks-feature-split-list-image-card-grid")],
        FEATURES_A.iter().map(feature_card).collect(),
    );
    let list = div(
        vec![("class", "blocks-feature-split-list-image-list")],
        vec![header, grid],
    );
    let image_col = split_image(dummy_assets::PRODUCT_SRC, AspectRatio::Portrait);
    div(
        vec![("class", "blocks-feature-split-list-image-instance")],
        vec![split_list_then_image(list, image_col)],
    )
}

/// インスタンス B（アイコン付き行 3 件 + 正方形画像 1 枚、DOM 順は画像が先）。
fn instance_b() -> Node {
    let header = section_header(
        "できること",
        "アイコン付きの一覧で要点を伝える",
        "各行はアイコン・短い見出し・説明の 3 要素で揃えています。",
    );
    let rows = div(
        vec![("class", "blocks-feature-split-list-image-row-list")],
        FEATURES_B.iter().map(feature_row).collect(),
    );
    let list = div(
        vec![("class", "blocks-feature-split-list-image-list")],
        vec![header, rows],
    );
    let image_col = split_image(dummy_assets::SCREENSHOT_SRC, AspectRatio::Square);
    div(
        vec![("class", "blocks-feature-split-list-image-instance")],
        vec![split_image_then_list(image_col, list)],
    )
}

/// インスタンス C（数値カード 2 枚 + ボタン行 + 正方形画像 1 枚）。
fn instance_c() -> Node {
    let header = section_header(
        "実績",
        "数値で語る特長とボタン導線",
        "数値カードの下に、資料へのボタン導線を並べています。",
    );
    let grid = div(
        vec![("class", "blocks-feature-split-list-image-card-grid")],
        STATS_C.iter().map(stat_card).collect(),
    );
    let list = div(
        vec![("class", "blocks-feature-split-list-image-list")],
        vec![header, grid, stat_button_row()],
    );
    let image_col = split_image(dummy_assets::BACKGROUND_SRC, AspectRatio::Square);
    div(
        vec![("class", "blocks-feature-split-list-image-instance")],
        vec![split_list_then_image(list, image_col)],
    )
}

/// インスタンス D（定義リスト 4 項目 + 画像 4 枚の 2×2 グリッド）。
fn instance_d() -> Node {
    let header = section_header(
        "詳細",
        "定義リストと画像の組で詳細を示す",
        "用語ごとの短い説明を、画像 4 枚の組と並べています。",
    );
    let items: Vec<Node> = DEFINITIONS_D.iter().map(definition_item).collect();
    let dl = data_list::root(
        DataListProps {
            orientation: DataListOrientation::Vertical,
            ..DataListProps::default()
        },
        vec![],
        items,
    );
    let list = div(
        vec![("class", "blocks-feature-split-list-image-list")],
        vec![header, dl],
    );
    let image_grid = div(
        vec![("class", "blocks-feature-split-list-image-image-grid")],
        vec![
            split_image(dummy_assets::PRODUCT_SRC, AspectRatio::Square),
            split_image(dummy_assets::SCREENSHOT_SRC, AspectRatio::Square),
            split_image(dummy_assets::BACKGROUND_SRC, AspectRatio::Square),
            split_image(dummy_assets::LOGO_SRC, AspectRatio::Square),
        ],
    );
    div(
        vec![("class", "blocks-feature-split-list-image-instance")],
        vec![split_list_then_image(list, image_grid)],
    )
}

/// 各インスタンスに添える差分注記（`feature_image_cards` の `note_a`/
/// `note_b` と同じ形）。
fn note(body: &str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(body)],
    )
}

/// `feature-split-list-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feature-split-list-image-layout")],
        vec![
            note("カード一覧（4 件・2 列）+ 縦長画像 1 枚。一覧 → 画像の順。"),
            instance_a(),
            note("アイコン付き行（3 件）+ 正方形画像 1 枚。画像 → 一覧の順（狭幅で画像が上、lg で画像が左）。"),
            instance_b(),
            note("数値カード（2 件）+ ボタン行 + 正方形画像 1 枚。一覧 → 画像の順。"),
            instance_c(),
            note("定義リスト（4 件）+ 画像 4 枚の 2×2 グリッド。一覧 → 画像の順。"),
            instance_d(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feature-split-list-image/",
    title: "feature-split-list-image",
    category: BlockCategory::Feature,
    rust_source: "crates/docs-site/src/blocks/marketing/feature/feature_split_list_image.rs",
    demo_class: "blocks-feature-split-list-image",
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
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feature_split_list_image` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-feature-split-list-image-*` と
/// `[data-blocks-feature-split-list-image-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`feature_image_cards`/`careers_card_grid` と
/// 同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-feature-split-list-image-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-feature-split-list-image-instance {\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-feature-split-list-image-split {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n  align-items: start;\n}\n\
.blocks-feature-split-list-image-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-feature-split-list-image-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-feature-split-list-image-eyebrow] {\n  align-self: flex-start;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-split-list-image-lead] {\n  margin: 0;\n}\n\
.blocks-feature-split-list-image-card-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-feature-split-list-image-card-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-feature-split-list-image-card-desc] {\n  margin: 0;\n}\n\
[data-blocks-feature-split-list-image-stat-value] {\n  color: var(--fandhe-color-accent);\n  line-height: 1;\n}\n\
.blocks-feature-split-list-image-row-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-feature-split-list-image-row {\n  display: flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-feature-split-list-image-row-icon] {\n  flex-shrink: 0;\n}\n\
.blocks-feature-split-list-image-row-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-feature-split-list-image-button-row {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feature-split-list-image-dt {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feature-split-list-image-image-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-feature-split-list-image-image] {\n  display: block;\n  width: 100%;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-feature-split-list-image-card-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-feature-split-list-image-split {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
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
            "data-scope=\"icon\"",
            "data-scope=\"card\"",
            "data-scope=\"image\"",
            "data-scope=\"button\"",
            "data-scope=\"data-list\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // A(1) + B(1) + C(1) + D(4) = 7 枚。
        assert_eq!(html.matches("<img").count(), 7);
        assert!(html.contains("<dl"));
        assert!(html.contains("<dt"));
        assert!(html.contains("<dd"));
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 画像の src がすべて `dummy_assets` のプレースホルダーであること。
    #[test]
    fn demo_uses_dummy_asset_images_only() {
        let html = render(&demo());
        assert!(html.contains(dummy_assets::PRODUCT_SRC));
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        assert!(html.contains(dummy_assets::LOGO_SRC));
    }

    /// [`LAYOUT_CSS`] が想定する lg ブレークポイント・2 列 grid を持つこと。
    #[test]
    fn layout_css_declares_lg_split_grid_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「CSS フックの選び方」節の Bugbot 教訓の
    /// 固定、`feature_image_cards`/`careers_card_grid` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feature-split-list-image-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feature-split-list-image-layout"
        );
    }
}
