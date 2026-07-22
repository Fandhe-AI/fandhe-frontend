//! `SlotRecipe`（イシュー #548）の決定性テスト（受け入れ条件 3）。
//!
//! ランタイム CSS-in-JS を採用せずビルド時に静的 CSS を生成する方針
//! （#545 概要）の前提として、「同一入力から常に同一 CSS が出る」ことを
//! 固定する。内部ストレージが `Vec` のみで `HashMap`/`HashSet` を使わない
//! という設計（`src/recipe.rs` の順序規約コメント参照）が壊れていないかの
//! 回帰検知でもある。

use fandhe_frontend_pre_styled_ui::decl;
use fandhe_frontend_pre_styled_ui::recipe::{when, Size, SlotRecipe, VariantValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tone {
    Solid,
    Outline,
}

impl VariantValue for Tone {
    fn axis(self) -> &'static str {
        "tone"
    }

    fn value(self) -> &'static str {
        match self {
            Tone::Solid => "solid",
            Tone::Outline => "outline",
        }
    }
}

fn build_recipe() -> SlotRecipe {
    SlotRecipe::new("button", &["root"])
        .base("root", vec![decl("display", "inline-flex")])
        .variant(Size::Sm, "root", vec![decl("padding", "2px 6px")])
        .variant(Size::Md, "root", vec![decl("padding", "4px 10px")])
        .variant(Tone::Solid, "root", vec![decl("background", "blue")])
        .variant(
            Tone::Outline,
            "root",
            vec![decl("background", "transparent")],
        )
        .default_variant(Size::Md)
        .default_variant(Tone::Solid)
        // イシュー #604: compound variant も他の規則と同じ決定性保証（byte
        // 一致・繰り返し呼び出し安定性）の対象であることを回帰検知する。
        .compound_variant(
            vec![when(Size::Sm), when(Tone::Outline)],
            "root",
            vec![decl("border-width", "1px")],
        )
}

#[test]
fn css_is_byte_identical_across_independently_built_instances() {
    let a = build_recipe();
    let b = build_recipe();
    assert_eq!(a.css(), b.css());
}

#[test]
fn css_is_stable_across_repeated_calls_on_same_instance() {
    let recipe = build_recipe();
    let first = recipe.css();
    for _ in 0..5 {
        assert_eq!(recipe.css(), first);
    }
}

#[test]
fn variant_classes_is_stable_across_independently_built_instances_and_repeated_calls() {
    let a = build_recipe();
    let b = build_recipe();
    let selection: &[(&str, &str)] = &[("size", "lg")];

    assert_eq!(a.variant_classes(selection), b.variant_classes(selection));
    assert_eq!(a.variant_classes(&[]), b.variant_classes(&[]));

    let first = a.variant_classes(selection);
    for _ in 0..5 {
        assert_eq!(a.variant_classes(selection), first);
    }
}
