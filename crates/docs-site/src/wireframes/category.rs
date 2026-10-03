//! Wireframes（`/wireframes/`）の部品カテゴリ型（イシュー #3618）。
//!
//! # 役割・呼び出し文脈
//!
//! [`super::Wireframe`] は本モジュールの [`WireframeCategory`] を必須フィールドと
//! して持ち、[`crate::category_index::render_wireframes`]（`/wireframes/` 索引の
//! カード生成）が「カテゴリ → 部品リンク」の階層を組む際の唯一の分類源に使う。
//! 分類は `docs/design/wireframe-ui-architecture.md` §8 の Phase 表を基にし、
//! 読み手向けに Forms A/B を 1 つへ統合した 7 種とする。
//!
//! # 全域性
//!
//! [`WireframeCategory::label`] は `_ =>` を使わない全 variant 明示の `match`
//! で、variant 追加時の更新漏れはコンパイルエラーになる（fail-closed）。
//! [`WireframeCategory::ALL`] は索引での表示順を兼ねる。

/// Wireframes 部品の分類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireframeCategory {
    Layout,
    Text,
    Forms,
    Navigation,
    OverlayFeedback,
    DataDisplay,
    Media,
}

impl WireframeCategory {
    /// 索引ページでの表示順。
    pub const ALL: [WireframeCategory; 7] = [
        WireframeCategory::Layout,
        WireframeCategory::Text,
        WireframeCategory::Forms,
        WireframeCategory::Navigation,
        WireframeCategory::OverlayFeedback,
        WireframeCategory::DataDisplay,
        WireframeCategory::Media,
    ];

    /// 索引ページの `h3` 見出しに使う表示ラベル。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            WireframeCategory::Layout => "Layout",
            WireframeCategory::Text => "Text",
            WireframeCategory::Forms => "Forms",
            WireframeCategory::Navigation => "Navigation",
            WireframeCategory::OverlayFeedback => "Overlay & Feedback",
            WireframeCategory::DataDisplay => "Data Display",
            WireframeCategory::Media => "Media",
        }
    }
}
