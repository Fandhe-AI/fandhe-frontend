//! SkipNav（`fandhe_frontend_headless_ui`/`fandhe_frontend_pre_styled_ui`
//! の `skip_nav` モジュール、イシュー #776）が使う専用 CSS の配線。
//!
//! # 役割・呼び出し文脈
//!
//! [`crate::layout::docs_page_with_assets`] は全ページの `<body>` 先頭へ
//! `fandhe_frontend_pre_styled_ui::skip_nav::link`（「Skip to content」
//! リンク）を、`main` 内の本文直前へ `fandhe_frontend_pre_styled_ui::skip_nav::content`
//! （スキップ先ターゲット）を常時挿入する（WCAG 2.1 SC 2.4.1 Bypass Blocks
//! 対応、docs-site は全ページがこのレイアウト骨格を持つため
//! [`crate::admonition`]/[`crate::showcase`] のような「使われているページ
//! だけ」の条件判定を必要としない — **常時**書き出し・常時 `<link>` 付与
//! する点が両モジュールとの違い）。
//!
//! [`crate::build::build_site`] は [`stylesheet`] が返す CSS 本体を
//! [`STYLESHEET_REL_PATH`] へ**全ビルドで無条件に**書き出す。
//!
//! サイト骨格スタイル（[`crate::site_theme`] によるビルド時生成、出力先
//! `assets/site.css`）は一切変更しない
//! （[`crate::admonition`] と同じ #715 の分離 CSS 不変条件）。
//!
//! # 純 CSS の focus 時表示（docs-site は hydration を持たない）
//!
//! docs-site はテーマトグル用の素の JS（`crate::script`、イシュー #951）を
//! 導入したが、`data-hydrate`/`data-bind-*` 束縛点による hydration は
//! 引き続き持たない。そのため `fandhe_frontend_pre_styled_ui::skip_nav` が
//! [`fandhe_frontend_pre_styled_ui::recipe::StateCondition::FocusVisible`]
//! （`:focus-visible` 疑似クラス）のみで表現する focus 時表示規則は、本
//! モジュール経由で純 CSS のまま docs-site へ適用される（`data-focus-visible`
//! 配線は不要、`fandhe-frontend-pre-styled-ui::skip_nav` モジュール doc 参照）。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! CSS は [`StyleSheet`]（検証済み CSS のみを保持し `<`・不正な制御文字を
//! 拒否する型）経由でのみ組み立てる（[`crate::admonition::stylesheet`] と
//! 同じ方針）。

use fandhe_frontend_pre_styled_ui::theme::Theme;
use fandhe_frontend_pre_styled_ui::{skip_nav, StyleSheet, StylesheetError};

/// SkipNav 専用 CSS の出力先（`out_dir` 起点の相対パス）。
/// `crate::build::build_site` が [`stylesheet`] の内容をこのパスへ書き出し、
/// `crate::layout::docs_page_with_assets` の追加 `<link>` が全ページで
/// これを参照する。
pub const STYLESHEET_REL_PATH: &str = "assets/skip-nav.css";

/// SkipNav が参照する CSS 全量を組み立てる。
///
/// 内訳: テーマトークン（`Theme::default`、SkipNav が参照する
/// `--fandhe-color-bg`/`-fg`/`-bg-muted`/`--fandhe-radius-md`/
/// `--fandhe-space-6`/`-2-5`/`--fandhe-z-index-skip-nav`/
/// `--fandhe-focus-ring-*`/`--fandhe-font-font-*`/`--fandhe-motion-*`
/// 等を定義する。イシュー #1586 でタイポグラフィ・focus ring・z-index・
/// hover・transition の各トークンを追加参照するよう改修した）→
/// `skip_nav::stylesheet()`（clip 手法 + `:focus-visible` 表示規則）の順で
/// 決定的に連結する（[`crate::admonition::stylesheet`] と同型の組み立て
/// 順）。
///
/// # Errors
///
/// [`StyleSheet::push_css`] の検証（`<`・制御文字の拒否）に落ちた場合
/// [`StylesheetError`] を返す。pre-styled-ui 側の生成 CSS は構造上 `<` を
/// 含み得ないため通常は到達しないが、黙って欠けた CSS を公開しない
/// fail-closed 方針で伝播させる。
pub fn stylesheet() -> Result<StyleSheet, StylesheetError> {
    let mut sheet = StyleSheet::new();
    sheet.push_theme(&Theme::default());
    sheet.push_css(&skip_nav::stylesheet())?;
    Ok(sheet)
}
