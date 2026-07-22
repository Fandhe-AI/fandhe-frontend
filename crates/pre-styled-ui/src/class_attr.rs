//! `class` 属性のマージヘルパ（イシュー #550）。
//!
//! [`crate::recipe::SlotRecipe::variant_classes`] が生成するクラス文字列が
//! styled 部品の variant 付きパーツにおける `class` 属性の唯一の権威となる
//! よう、呼び出し側が渡した `attrs` に含まれる `class`（大文字小文字を無視）
//! を黙って除去してから recipe 生成クラスと合成する。
//!
//! これは `fandhe_frontend_headless_ui::anatomy::Anatomy::part` が
//! `data-scope`/`data-part` の呼び出し側偽装値を fail-closed で除外するのと
//! 同型の判断であり、重複 `class` 属性による無効な HTML 出力・後勝ちの
//! 非決定的なスタイル適用を防ぐ（`.claude/rules/security.md` 準拠）。

/// `attrs` から `class`（ASCII 大文字小文字を無視）を除いた列を返す。
///
/// [`crate::button::button`]・[`crate::badge::badge`]・[`crate::card::root`]・
/// [`crate::alert::root`] など、recipe が `class` 属性を自ら付与するパーツが
/// 呼び出し側 `attrs` を連結する前に必ず本関数を通す契約とする。
#[must_use]
pub(crate) fn drop_class_attr<'a>(attrs: Vec<(&'a str, &'a str)>) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case("class"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_class_case_insensitively_and_keeps_other_attrs() {
        let attrs = vec![("id", "x"), ("Class", "attacker"), ("data-foo", "bar")];
        assert_eq!(
            drop_class_attr(attrs),
            vec![("id", "x"), ("data-foo", "bar")]
        );
    }

    #[test]
    fn drops_uppercase_class_variant() {
        let attrs = vec![("CLASS", "attacker")];
        assert!(drop_class_attr(attrs).is_empty());
    }

    #[test]
    fn empty_attrs_stay_empty() {
        assert!(drop_class_attr(Vec::new()).is_empty());
    }
}
