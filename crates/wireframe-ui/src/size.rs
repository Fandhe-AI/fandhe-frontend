//! サイズ段階 `Size` を提供する。
//!
//! `fandhe-frontend-pre-styled-ui` の `recipe::Size`（`crates/pre-styled-ui/src/recipe.rs`）
//! と段階名（`xs`/`sm`/`md`/`lg`/`xl`）を一致させるが、pre-styled-ui への
//! 依存は追加しない（`docs/design/wireframe-ui-architecture.md` §4）。
//! 両者の variant 名パリティは `crates/xtask/tests/wireframe_ui_size_parity.rs`
//! がソース走査で機械固定する。

/// wireframe-ui 部品が共通で使うサイズ段階。既定は `Md`。
///
/// pre-styled-ui `recipe::Size` と段階名は同じだが、こちらは
/// `SlotRecipe`/`VariantValue` トレイトを実装しない独立した型である
/// （headless-ui/pre-styled-ui への依存を持たない wireframe-ui の
/// 責務境界、`docs/design/wireframe-ui-architecture.md` §1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Size {
    /// 極小サイズ。
    Xs,
    /// 小サイズ。
    Sm,
    /// 中サイズ（既定値）。
    #[default]
    Md,
    /// 大サイズ。
    Lg,
    /// 極大サイズ。
    Xl,
}

impl Size {
    /// 全段階を宣言順（`Xs`〜`Xl`）で列挙する。テスト・CSS 生成が走査に使う。
    pub const ALL: [Size; 5] = [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl];

    /// 段階名の文字列表現（`"xs"`〜`"xl"`）。pre-styled-ui `recipe::Size::value()`
    /// と同じリテラル値を返す（依存はしない、文字列の一致のみ）。
    pub const fn as_str(self) -> &'static str {
        match self {
            Size::Xs => "xs",
            Size::Sm => "sm",
            Size::Md => "md",
            Size::Lg => "lg",
            Size::Xl => "xl",
        }
    }

    /// この段階に対応する class 名（`fw-wire-size-<段階>`）。
    pub const fn class(self) -> &'static str {
        match self {
            Size::Xs => "fw-wire-size-xs",
            Size::Sm => "fw-wire-size-sm",
            Size::Md => "fw-wire-size-md",
            Size::Lg => "fw-wire-size-lg",
            Size::Xl => "fw-wire-size-xl",
        }
    }
}

/// `Size` 段階ごとのスコープ付きカスタムプロパティ値表。
///
/// 各部品 CSS はここで定義される `--fw-wire-font-size`/`--fw-wire-control-size`
/// を参照することで、部品ごとに 5 段の宣言を書かずに済む（設計判断は
/// イシュー #2605 実装計画 §3.1）。
///
/// クレート内限定で公開する（`pub(crate)`）。[`crate::frame::frame_padding_css`]
/// が padding 値（`control_size` を使う `calc()` 式）を導出するために
/// 直接走査する唯一の消費経路であり、`control_size` の値を他所へ
/// 書き写さず常にこの配列を単一の正として参照させる（コードレビュー
/// 指摘、イシュー #2609/PR #2679）。`.find()`/`.expect()` を要する
/// ルックアップ関数は設けない（ライブラリコードでの `expect()` を避ける
/// 規約、`.claude/rules/coding-rust.md`）。全 5 段の直接走査で足りる
/// 呼び出し側のみが本配列を参照する設計とする。
pub(crate) const SCALE: [(Size, &str, &str); 5] = [
    (Size::Xs, "0.75rem", "1.5rem"),
    (Size::Sm, "0.875rem", "1.75rem"),
    (Size::Md, "1rem", "2rem"),
    (Size::Lg, "1.125rem", "2.5rem"),
    (Size::Xl, "1.25rem", "3rem"),
];

/// `Size` 5 段の CSS 宣言（`.fw-wire-size-<段階> { --fw-wire-font-size: ...; --fw-wire-control-size: ...; }`）
/// を生成する。[`crate::css::wireframe_css`] から呼ばれる。
pub fn css() -> String {
    let mut out = String::new();
    for (size, font_size, control_size) in SCALE {
        out.push('.');
        out.push_str(size.class());
        out.push_str(" { --fw-wire-font-size: ");
        out.push_str(font_size);
        out.push_str("; --fw-wire-control-size: ");
        out.push_str(control_size);
        out.push_str("; }\n");
    }
    out
}
