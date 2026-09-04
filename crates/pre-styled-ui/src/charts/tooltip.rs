//! チャートツールチップ（イシュー #847、chakra-ui `charts/tooltip.md`
//! 相当。[`crate::tooltip`]（汎用 headless Tooltip、hover/focus で JS が
//! 表示制御する）とは別物であり、モジュールパス `charts::tooltip` で区別する
//! （chakra-ui 側も「chart 専用であり general-purpose Tooltip とは別」と
//! 明記しており対応が一致する）。
//!
//! # SSR ツールチップ方式（JS を使わない設計）
//!
//! マウス追従型のリッチツールチップ（recharts `<Tooltip>` の cursor 追従）は
//! JS ランタイムが必須のためスコープ外とする（`crates/pre-styled-ui/src/charts/mod.rs`
//! のスコープ外節参照）。代わりに、データ点要素（`datum` slot）へ:
//!
//! 1. 子 `<title>` 要素（ブラウザネイティブな hover 表示。SVG 標準機能で
//!    JS 不要）
//! 2. `aria-label` 属性（スクリーンリーダー向け、`<title>` と同一文字列）
//! 3. [`crate::recipe::StateCondition::Hover`] による `:hover` 時の視覚的
//!    強調（`stroke`/`stroke-width` 変更、CSS のみ。base で背景色ハロー
//!    （`stroke: var(--fandhe-color-bg)`）を敷いてから hover で前景色
//!    （`--fandhe-color-fg`）へ切り替える方式とし、SVG の既定 `stroke: none`
//!    に依存しない。イシュー #1425 で当該規則は
//!    `@media (hover: hover)` 内・セレクタ末尾 `:not([data-disabled])` 付き
//!    で出力される形へ変更された。タッチ端末での hover 貼り付き回避と
//!    disabled 規則との勝敗を記述順に依存させないための共通対策であり、
//!    イシュー #1425 時点では本モジュールのコード自体は無変更だった。
//!    その後 #1593 で base に背景色ハロー（上記 `stroke: var(--fandhe-color-bg)`）
//!    と transition を追加している）
//!
//! を組み合わせて埋め込み、JS なしで「ホバーで詳細が分かる」体験を実現する。
//!
//! `stroke`/`stroke-width` は本モジュールの CSS が専有する。呼び出し側
//! （[`datum`]）は `fill` 等の見た目属性のみを渡す契約とし、CSS 側で
//! `fill` を宣言しない（SVG ではプレゼンテーション属性より CSS の
//! `fill`/`stroke` が優先されるため、CSS で `fill` を書くと呼び出し側の
//! 系列色指定が潰れる）。
//!
//! # 参考サイト基準への調整（イシュー #1593）
//!
//! 参照 4 サイト（ark-ui / chakra-ui / Radix Primitives / Radix Themes）に
//! 対応部品が無いため、評価軸は内部整合のみとした。
//!
//! | 観点 | 判定 |
//! |---|---|
//! | サイズ / バリアント / colorPalette | 非該当（表示専用・参照軸なし） |
//! | 色 | 是正（hover ストローク色を `--fandhe-color-accent-emphasized` から `--fandhe-color-fg` へ） |
//! | 状態・`data-*` | 変更なし |
//! | ダーク | 是正（`accent-emphasized` の dark 値 `#63b3ed` が `chart-1` の dark 値と同一で hover 強調が消える問題を解消） |
//! | フォーカス | 非該当（`tabindex` 付与は anatomy 変更のため別判断） |
//! | 余白・角丸・影 | 非該当 |
//! | hover / disabled / transition | 是正（`transition_declarations` 未導入だったため追加） |
//! | 内部整合 | 是正（隣接・重なった異系列データ点の識別のため base に背景色ハローを追加） |
//!
//! ## 是正した点
//!
//! - **hover ストローク色**: `--fandhe-color-accent-emphasized` は dark で
//!   `#63b3ed` を返し、6 系列パレットの `chart-1`（dark `#63b3ed`）と
//!   完全一致するため、ダークテーマで系列 1 のデータ点は hover 強調が
//!   視覚的に消えていた。`--fandhe-color-fg` は light/dark とも背景との
//!   コントラスト比が高く（約 15:1）、6 系列色のいずれとも一致しない
//!   ため採用した（`--fandhe-color-focus-ring`（focus-ring 色トークン）は dark で同じ `#63b3ed`
//!   衝突を起こすため候補から外した）。
//! - **transition**: hover 状態遷移を持つ唯一のインタラクティブ slot
//!   だったが `transition_declarations` が未導入だった
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §5）。
//!   `prefers-reduced-motion: reduce` は [`crate::theme::Theme::to_css`]
//!   の `--fandhe-motion-duration-*: 0ms` 一括上書きで自動対応するため、
//!   本モジュールに `@media` を追加する必要はない。
//! - **背景色ハロー**: 隣接・重なった異系列のデータ点の輪郭を識別できる
//!   よう、base に `stroke: var(--fandhe-color-bg)` / `stroke-width: 1`
//!   を追加した（[`crate::area_chart`] の `point` slot と同型）。
//!
//! ## 意図的に合わせなかった点
//!
//! - 6 系列パレット（`chart-1`〜`chart-6`、[`crate::theme`]）自体の見直し
//!   は本イシューのスコープ外とした（`theme.rs` の変更は docs-site の
//!   契約テストへ波及し他部品にも影響するため）。
//! - `vector-effect: non-scaling-stroke` 等のスケーリング対策はスコープ
//!   外とした（兄弟部品との線幅の見え方の乖離を避けるため）。

use super::svg::fmt_coord;
use crate::css::decl;
use crate::recipe::{transition_declarations, MotionDuration, SlotRecipe, StateCondition};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

/// 本モジュールの anatomy scope（[`super::axis`]/[`super::grid`] と共有）。
const SCOPE: &str = "chart";

/// [`recipe`] に渡す slot 一覧。
const SLOTS: &[&str] = &["datum"];

/// [`datum`] が固定する属性名（呼び出し側 `attrs` からの偽装を fail-closed
/// で除去する対象。`crates/pre-styled-ui/src/table.rs` の `COLUMN_HEADER_RESERVED`
/// と同型の判断）。
const DATUM_RESERVED: &[&str] = &["data-scope", "data-part", "cx", "cy", "r", "aria-label"];

/// Tooltip（データ点強調表示）の recipe（scope `"chart"`、[`SLOTS`] の
/// 1 パーツ）。
fn recipe() -> SlotRecipe {
    let mut base = vec![
        decl("cursor", "default"),
        decl("stroke", "var(--fandhe-color-bg)"),
        decl("stroke-width", "1"),
    ];
    base.extend(transition_declarations(
        "stroke, stroke-width",
        MotionDuration::Fast,
    ));
    SlotRecipe::new(SCOPE, SLOTS).base("datum", base).state(
        "datum",
        StateCondition::Hover,
        vec![
            decl("stroke", "var(--fandhe-color-fg)"),
            decl("stroke-width", "2"),
        ],
    )
}

/// Tooltip の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// カテゴリ・系列名・値からツールチップ本文（`<title>`/`aria-label` 共用）の
/// 決定的文字列を組み立てる。値の文字列化は [`super::svg::fmt_coord`] のみを
/// 経由する（数値の決定的文字列化の一元化、`crates/pre-styled-ui/src/charts/mod.rs`
/// 冒頭 doc 不変条件 2）。
#[must_use]
pub fn datum_label(category: &str, series: &str, value: f64) -> String {
    format!("{category} · {series}: {}", fmt_coord(value))
}

/// データ点（`<circle>`）を組み立てる。子 `<title>` 要素と `aria-label`
/// 属性の両方に `label` を埋め込む（モジュール doc「SSR ツールチップ方式」
/// 参照）。
///
/// `attrs` に本関数が固定するキー（[`DATUM_RESERVED`]）が含まれていても
/// 除去してから連結する（fail-closed。呼び出し側は `fill` 等の見た目属性
/// のみを追加する想定、後続チャート部品 #848〜#851 の消費経路）。
///
/// 座標は [`super::svg::fmt_coord`] のみを経由して文字列化する。`cx`/`cy`/`r`
/// が非有限の場合の出力は未規定（[`super::svg::fmt_coord`] の契約と同じく、
/// 呼び出し元は [`super::data::ChartData::new`] の検証を経由した有限値のみを
/// 渡す契約とする）。
#[must_use]
pub fn datum<'a>(cx: f64, cy: f64, r: f64, label: &str, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let (cx, cy, r) = (fmt_coord(cx), fmt_coord(cy), fmt_coord(r));
    let mut merged: Vec<(&str, &str)> = vec![
        ("data-scope", SCOPE),
        ("data-part", "datum"),
        ("cx", cx.as_str()),
        ("cy", cy.as_str()),
        ("r", r.as_str()),
        ("aria-label", label),
    ];
    merged.extend(
        attrs
            .into_iter()
            .filter(|(k, _)| !DATUM_RESERVED.iter().any(|r| k.eq_ignore_ascii_case(r))),
    );
    el(
        "circle",
        merged,
        vec![el("title", vec![], vec![text(label)])],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn datum_label_joins_category_series_and_formatted_value() {
        assert_eq!(datum_label("Jan", "visits", 12.5), "Jan · visits: 12.5");
    }

    #[test]
    fn datum_label_is_deterministic() {
        assert_eq!(datum_label("a", "b", 1.0), datum_label("a", "b", 1.0));
    }

    #[test]
    fn datum_renders_circle_with_title_and_aria_label() {
        let label = datum_label("Jan", "visits", 10.0);
        let html = render(&datum(1.0, 2.0, 4.0, &label, vec![("fill", "red")]));
        assert!(
            html.starts_with(r#"<circle data-scope="chart" data-part="datum" cx="1" cy="2" r="4""#)
        );
        assert!(html.contains(r#"aria-label="Jan · visits: 10""#));
        assert!(html.contains("<title>Jan · visits: 10</title>"));
        assert!(html.contains(r#"fill="red""#));
    }

    #[test]
    fn datum_drops_caller_supplied_reserved_attrs() {
        let html = render(&datum(
            0.0,
            0.0,
            1.0,
            "safe",
            vec![
                ("data-scope", "attacker"),
                ("data-part", "attacker"),
                ("cx", "999"),
                ("cy", "999"),
                ("r", "999"),
                ("aria-label", "attacker"),
                ("fill", "blue"),
            ],
        ));
        assert!(html.contains(r#"data-scope="chart""#));
        assert!(html.contains(r#"data-part="datum""#));
        assert!(html.contains(r#"cx="0""#));
        assert!(html.contains(r#"cy="0""#));
        assert!(html.contains(r#"r="1""#));
        assert!(html.contains(r#"aria-label="safe""#));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert_eq!(html.matches(" cx=").count(), 1);
        assert!(html.contains(r#"fill="blue""#));
    }

    #[test]
    fn xss_regression_label_is_escaped_in_title_and_aria_label() {
        let payload = "</title><script>alert(1)</script>";
        let html = render(&datum(0.0, 0.0, 1.0, payload, vec![]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_output_declares_hover_state_and_is_closed_charset() {
        let out = css();
        assert!(out.contains(":hover"));
        assert!(out.contains("stroke: var(--fandhe-color-fg)"));
        assert!(out.contains("stroke: var(--fandhe-color-bg)"));
        assert!(out.contains("transition-duration: var(--fandhe-motion-duration-fast)"));
        assert!(!out.contains('<'));
    }
}
