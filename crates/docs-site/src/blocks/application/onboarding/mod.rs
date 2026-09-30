//! Application / Onboarding カテゴリの block 登録点（イシュー #2734 雛形、
//! イシュー #2978 で最初の block（`onboarding-centered-steps`）を追加し
//! ディレクトリ化した。以降の block 追加は本ディレクトリ内で完結し、
//! `super`（`application`）側の宣言・集約コードは変更不要（`pub(super) fn
//! blocks()` のシグネチャを維持するため、`docs/design/docs-site-blocks-
//! section.md` §18 参照）。

mod onboarding_centered_steps;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![onboarding_centered_steps::BLOCK]
}
