//! Toast の stack 表示（積層・hover 展開、opt-in、イシュー #2543）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §9（参照は可・転写は
//! 禁止）に従い、Motion+ の toast stack 相当の意匠から着想した見た目を
//! CSS のみで独自に再実装する（コード・スクリーンショットの転写はしない）。
//!
//! # 何をするか
//!
//! [`STACK_ATTR`] を `crate::toast::group` の呼び出し側 `attrs` へ渡すと、
//! 通知が「後ろのカードほど縮小・オフセット」で積層表示になり、
//! `:hover`/`:focus-within` で通常の縦並び表示へ展開する
//! （[`TOAST_STACK_CSS`] 参照）。`crate::toast::group`/`crate::toast::root`
//! 本体・`crate::toast::stylesheet` は一切変更しない、既存部品への opt-in
//! 装飾層（`crate::button_motion`/`crate::forms_motion` と同型）。
//!
//! # DOM 順の契約（重要）
//!
//! CSS は兄弟の総数を知れないため「末尾を前面にする」計算はできない。
//! **積層の前面（index 0）は group の最初の子 root**という前提で
//! [`TOAST_STACK_CSS`] を書いているため、呼び出し側は新しい通知ほど
//! **先頭**に来る順で root を並べること
//! （[`fandhe_frontend_headless_ui::toast::Toaster::push`] は末尾追加の
//! ため、`entries.iter().rev()` で渡す）。
//!
//! # 自動配線の入口が [`stack_group_keyed`] である理由
//!
//! 積層オフセット・z-index は各 root の `--fandhe-motion-stagger-index`
//! （[`crate::recipe::STAGGER_INDEX_VAR`]）から算出する。動的な追加・削除
//! でこの値を書き戻すのは `fandhe-frontend-wasm-full` の stagger 書き戻し
//! （`stagger_index.rs`、#2511）の役目で、これは keyed list の親要素に
//! [`STAGGER_AUTO_FIRST_ATTR`] が付いているときのみ動く契約
//! （`fandhe-frontend-pre-styled-ui` は `wasm-full` に依存しないため型共有は
//! できず、本モジュールはリテラル値の写しを [`STAGGER_AUTO_FIRST_ATTR`]/
//! [`FLIP_AUTO_ATTR`] として保持する。ドリフトは
//! `crates/pre-styled-ui/tests/motion_toast_css.rs` が wasm-full 側ソース
//! を読んで fail-closed に検知する、`button_motion_attr_drift.rs` と同型）。
//! 同様に挿入・削除・並べ替え時の滑らかな移動は layout FLIP（#2518）が
//! [`FLIP_AUTO_ATTR`] を見て担う。[`stack_group_keyed`] は
//! `crate::toast::group` の出力へこの 3 属性を付けたうえで
//! `fandhe_frontend_core::keyed::keyed_list` を呼ぶ薄い合成関数であり、
//! headless の group 属性生成・variant class 付与を再実装しない。
//!
//! 静的（SSR のみ・`Toaster` 非使用）な呼び出し側は
//! `toast::group(placement, label, vec![(STACK_ATTR, "")], roots)` と各
//! root の `attrs` へ `("style", &recipe::stagger_index_style(i))` を足す
//! だけで足り、専用ヘルパは設けない（stagger/FLIP の自動追従は動的更新
//! 時のみ意味を持つため）。
//!
//! # `to_css()` 本体を変更しない理由
//!
//! [`crate::motion`] モジュール doc・[`crate::button_motion`] モジュール
//! doc と同じ契約: [`crate::theme::Theme::to_css`] 本体・走査ループは
//! 一切変更せず、[`Theme::to_css_with_toast_motion`] を別 impl ブロックと
//! して追加し、`to_css()` の出力へ [`TOAST_STACK_CSS`] を追記するだけの
//! opt-in メソッドにする（pure append）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! [`crate::button_motion`] モジュール doc と同じ理由により、本モジュール
//! は空引数の CSS 取得関数（`crate::toast` が持つのと同名の 2 関数）を
//! 持たない（`crates/pre-styled-ui/src/stylesheet.rs` の文字列走査が
//! 「styled 部品」と誤認し `motion` feature off でビルド不能になるのを
//! 避けるため。doc コメント中にもそのシグネチャ文字列を書かない）。
//!
//! # 既知の制約
//!
//! 展開は `display` の切替（grid → flex）を伴うため、積層状態と展開状態の
//! 間に位置の補間（アニメーション）はない（`translate`/`scale`/`opacity`
//! の戻り遷移のみが滑らかに動く）。通知の高さが不揃いでも破綻しない
//! `grid-area` 重ね合わせ方式を優先した結果であり、位置補間が必要になった
//! 時点で JS 計測を要する `fandhe-frontend-animation` 側の課題とする
//! （実装計画 §7.2）。
//!
//! # reduced-motion（WCAG 2.3.3）
//!
//! [`TOAST_STACK_CSS`] の `transition-duration`/`transition-timing-function`
//! は [`crate::theme::Theme::to_css`] が `prefers-reduced-motion: reduce`
//! 下で自動的に `0ms` へ縮退させるテーマトークン
//! （`--fandhe-motion-duration-normal`）を参照するため、本ファイルは
//! 個別の `@media (prefers-reduced-motion: reduce)` ブロックを持たない
//! （[`crate::forms_motion`] の underline-grow と同じ判断）。無限反復の
//! `@keyframes`・scroll-driven なアニメーションは使わない。

use crate::fandhe_frontend_core::keyed::{keyed_list, KeyedListError};
use crate::fandhe_frontend_core::Node;
use crate::toast::ToastPlacement;

/// opt-in（呼び出し側が group の `attrs` へ渡す、値は空文字列）: 積層
/// 表示を有効化するマーカー属性。
pub const STACK_ATTR: &str = "data-fandhe-toast-stack";

/// `fandhe_frontend_wasm_full::stagger_index::STAGGER_AUTO_FIRST_ATTR` の
/// リテラルの写し（モジュール冒頭「自動配線の入口が `stack_group_keyed`
/// である理由」節参照）。
pub const STAGGER_AUTO_FIRST_ATTR: &str = "data-fandhe-stagger-auto-first";

/// `fandhe_frontend_wasm_full::layout_flip::FLIP_AUTO_ATTR` のリテラルの
/// 写し（同上）。
pub const FLIP_AUTO_ATTR: &str = "data-fandhe-flip-auto";

/// 積層表示の CSS 全文（末尾改行付き）。[`STACK_ATTR`] が付与された
/// group とその直下の root にのみ作用し、opt-in していない toast へは
/// 一切影響しない（すべてのセレクタが `[data-fandhe-toast-stack]` の
/// 子孫スコープに閉じている）。
///
/// # 前面 3 枚までを可視にする理由
///
/// `:nth-child(n+4)` の root を `opacity: 0` にするが、`display: none` に
/// はしない（`aria-live` region 内の DOM は維持し通知の読み上げ・
/// フォーカス到達性を損なわない、モジュール冒頭「既知の制約」節と対を
/// なす設計判断）。
pub const TOAST_STACK_CSS: &str = concat!(
    // group: 全 root を同一グリッドセルへ重ねる（積層時のみ）。
    "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack] {\n",
    "  display: grid;\n",
    "  grid-template-areas: \"stack\";\n",
    "  --fandhe-toast-stack-offset: var(--fandhe-space-3);\n",
    "  --fandhe-toast-stack-scale-step: 0.05;\n",
    "}\n",
    "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack][data-placement^=\"bottom\"] {\n",
    "  align-items: end;\n",
    "}\n",
    "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack][data-placement^=\"top\"] {\n",
    "  align-items: start;\n",
    "}\n",
    // root: 積層時のみ index に応じて縮小・オフセット（既定 bottom 系:
    // 前面ほど手前 = translate 上方向、index が大きいほど奥へ小さく）。
    "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack] > [data-scope=\"toast\"][data-part=\"root\"] {\n",
    "  grid-area: stack;\n",
    "  --fandhe-toast-stack-index: var(--fandhe-motion-stagger-index, 0);\n",
    "  z-index: calc(100 - var(--fandhe-toast-stack-index));\n",
    "  translate: 0 calc(var(--fandhe-toast-stack-index) * -1 * var(--fandhe-toast-stack-offset));\n",
    "  scale: calc(1 - var(--fandhe-toast-stack-index) * var(--fandhe-toast-stack-scale-step));\n",
    "  transform-origin: center bottom;\n",
    "  transition-property: translate, scale, opacity;\n",
    "  transition-duration: var(--fandhe-motion-duration-normal);\n",
    "  transition-timing-function: var(--fandhe-motion-easing-standard);\n",
    "}\n",
    // top 系は translate の符号・origin を反転する。
    "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack][data-placement^=\"top\"] > [data-scope=\"toast\"][data-part=\"root\"] {\n",
    "  translate: 0 calc(var(--fandhe-toast-stack-index) * var(--fandhe-toast-stack-offset));\n",
    "  transform-origin: center top;\n",
    "}\n",
    // 前面 3 枚のみ可視（4 枚目以降は透明化、DOM・読み上げは維持）。
    "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack] > [data-scope=\"toast\"][data-part=\"root\"]:nth-child(n+4) {\n",
    "  opacity: 0;\n",
    "  pointer-events: none;\n",
    "}\n",
    // hover/focus-within で展開（display を戻すのみ。flex-direction は
    // base/variant 規則が既に宣言済みのため再指定不要 — display が
    // flex/inline-flex に戻った時点で自動的に有効化される）。
    "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack]:is(:hover, :focus-within) {\n",
    "  display: flex;\n",
    "}\n",
    "[data-scope=\"toast\"][data-part=\"group\"][data-fandhe-toast-stack]:is(:hover, :focus-within) > [data-scope=\"toast\"][data-part=\"root\"] {\n",
    "  translate: none;\n",
    "  scale: none;\n",
    "  opacity: 1;\n",
    "  pointer-events: auto;\n",
    "}\n",
);

/// opt-in API。[`crate::theme::Theme::to_css`] の出力へ
/// [`TOAST_STACK_CSS`] を追記して返す（pure append、
/// [`crate::button_motion::Theme::to_css_with_button_motion`] と同型）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に積層表示用 CSS
    /// （[`TOAST_STACK_CSS`]）を追記して返す。
    #[must_use]
    pub fn to_css_with_toast_motion(&self) -> String {
        let mut out = self.to_css();
        out.push_str(TOAST_STACK_CSS);
        out
    }
}

/// stack 対応 group を keyed list として組み立てる（モジュール冒頭「自動
/// 配線の入口が `stack_group_keyed` である理由」節参照）。動的な追加・
/// 削除・並べ替えで `wasm-full` の stagger 書き戻し（#2511）+ layout FLIP
/// （#2518）が自動適用される唯一の入口。
///
/// `attrs` に予約属性（`data-bind-list`/`data-key`）が含まれる場合や
/// `items` がキー制約（非空・一意・上限）を満たさない場合は
/// [`fandhe_frontend_core::keyed::keyed_list`] と同じ `Err` を返す。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::toast::{self, ToastPlacement, ToastStatus};
/// use fandhe_frontend_pre_styled_ui::toast_motion::stack_group_keyed;
///
/// let root = toast::root(ToastStatus::Info, vec![], vec![]);
/// let node = stack_group_keyed(
///     ToastPlacement::BottomEnd,
///     "Notifications",
///     vec![],
///     "toasts",
///     vec![("t-1".to_string(), root)],
/// )
/// .expect("有効な items のため Ok");
/// let html = render(&node);
/// assert!(html.contains("data-fandhe-toast-stack"));
/// assert!(html.contains(r#"data-bind-list="toasts""#));
/// ```
///
/// # Errors
///
/// `attrs`/`items` の制約違反は
/// [`fandhe_frontend_core::keyed::KeyedListError`] を返す（`unwrap`
/// しないこと、`crate::recipe`/`keyed.rs` 側の既存規約と同じ）。
pub fn stack_group_keyed<'a>(
    placement: ToastPlacement,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    field: &'static str,
    items: Vec<(String, Node)>,
) -> Result<Node, KeyedListError> {
    let mut merged = attrs;
    merged.push((STACK_ATTR, ""));
    merged.push((STAGGER_AUTO_FIRST_ATTR, ""));
    merged.push((FLIP_AUTO_ATTR, ""));
    let group_node = crate::toast::group(placement, label, merged, vec![]);

    // `crate::toast::group` は headless `ANATOMY.part("group", "div", ...)`
    // へ委譲するため常に `Node::Element` を返す（headless-ui 側 doc
    // 参照）。ここが `Element` にならないのは本クレートが自身で呼んだ
    // 直後の構造上の不変条件（利用者入力に由来しない）であり、キー付き
    // リストを構築できない致命的状態のため `unreachable!` で明示する。
    let Node::Element { tag, attrs, .. } = &group_node else {
        unreachable!("crate::toast::group は常に Node::Element を返す")
    };
    let group_attrs: Vec<(&str, &str)> = attrs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    keyed_list(tag, group_attrs, field, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toast::{root, ToastStatus};
    use fandhe_frontend_core::keyed::{BIND_LIST_ATTR, KEY_ATTR};
    use fandhe_frontend_core::render;

    #[test]
    fn stack_attr_is_transparent_through_group() {
        let html = render(&crate::toast::group(
            ToastPlacement::BottomEnd,
            "Notifications",
            vec![(STACK_ATTR, "")],
            vec![],
        ));
        assert!(html.contains(STACK_ATTR));
    }

    #[test]
    fn stack_group_keyed_outputs_bind_list_and_opt_in_attrs() {
        let items = vec![
            ("a".to_string(), root(ToastStatus::Info, vec![], vec![])),
            ("b".to_string(), root(ToastStatus::Success, vec![], vec![])),
        ];
        let node = stack_group_keyed(
            ToastPlacement::BottomEnd,
            "Notifications",
            vec![],
            "toasts",
            items,
        )
        .expect("有効な items のため Ok");
        let html = render(&node);
        assert!(html.contains(r#"data-scope="toast""#));
        assert!(html.contains(r#"data-part="group""#));
        assert!(html.contains("fd-toast--placement-bottom-end"));
        assert!(html.contains(STACK_ATTR));
        assert!(html.contains(STAGGER_AUTO_FIRST_ATTR));
        assert!(html.contains(FLIP_AUTO_ATTR));
        assert!(html.contains(BIND_LIST_ATTR));
        assert!(html.contains(&format!("{KEY_ATTR}=\"a\"")));
        assert!(html.contains(&format!("{KEY_ATTR}=\"b\"")));
    }

    #[test]
    fn stack_group_keyed_rejects_reserved_bind_list_attr() {
        let err = stack_group_keyed(
            ToastPlacement::BottomEnd,
            "Notifications",
            vec![(BIND_LIST_ATTR, "evil")],
            "toasts",
            vec![],
        )
        .expect_err("data-bind-list を呼び出し側から偽装できないこと");
        assert_eq!(
            err,
            KeyedListError::ReservedAttr {
                attr: BIND_LIST_ATTR
            }
        );
    }

    #[test]
    fn to_css_with_toast_motion_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let with_motion = theme.to_css_with_toast_motion();
        assert_eq!(with_motion, format!("{base}{TOAST_STACK_CSS}"));
    }
}
