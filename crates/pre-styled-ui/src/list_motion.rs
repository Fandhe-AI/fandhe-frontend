//! `list` の並べ替え・追加・削除の opt-in 遷移 CSS（イシュー #2544、親
//! #2530 Phase 7、祖父トラッキング #2476）。
//!
//! 並べ替え（Move）は `fandhe-frontend-wasm-full` の `layout-animation`
//! feature（FLIP、イシュー #2518）が既に配線済みであり、本モジュールに
//! 追加実装はない。本モジュールが担うのは、追加行（enter）・削除行
//! （exit、`data-state="exiting"`）の `@keyframes` アニメーション CSS
//! のみである。追加行の配線は `fandhe-frontend-wasm-full` の `presence`
//! feature（`list_presence.rs`）、stagger 遅延の配線は既存 `stagger`
//! feature（`stagger_index.rs`）が担う——本モジュールは CSS を持つのみ
//! （3 層構成、`docs/design/motion-reference-adoption-policy.md` §6）。
//!
//! # `motion` feature 配下に置く理由
//!
//! [`crate`]（lib.rs）冒頭 doc「Cargo feature `motion`」節の gating 契約
//! に従い、本モジュールは `#[cfg(feature = "motion")]` 配下にのみ存在
//! する。[`crate::list`] 自体は変更しない（[`crate::forms_motion`]・
//! [`crate::border_beam`] と同型のパターン: 既存 `recipe()`/`css()` は
//! 一切変更せず、別モジュールが追加の静的 CSS を生成し、呼び出し側が
//! [`crate::theme::Theme::to_css_with_list_motion`] で明示的に連結する）。
//!
//! # `transition` ではなく `@keyframes` アニメーションを使う理由
//!
//! 1. stagger（[`crate::recipe::stagger_delay_declaration`]、イシュー
//!    #2384/#2397）は `animation-delay` ベースであり、`transition` では
//!    stagger と結合できない。
//! 2. `fandhe-frontend-wasm-full` の FLIP（`layout_flip.rs`）は
//!    `transform` を毎フレーム inline `!important` で書き込む。行に
//!    `transition: transform` があると各フレームの書き込みが transition
//!    化され追従が鈍るため、[`ENTER_KEYFRAMES_NAME`]/
//!    [`EXIT_KEYFRAMES_NAME`] はいずれも `opacity`/`scale` のみで構成し
//!    `transform` プロパティには一切触れない（FLIP 非干渉契約）。
//! 3. 挿入要素は CSS animation を挿入時に自動再生するため
//!    `@starting-style` を要しない。退場ゴースト
//!    （`fandhe-frontend-wasm-full::list_presence::play_exit_after`）も
//!    `data-state="exiting"` 付きで挿入するだけで再生が始まる。
//!
//! # `crate::recipe::SlotRecipe::presence_transition` を使わない理由
//!
//! [`crate::recipe::SlotRecipe::presence_transition`]（イシュー #2497）は
//! `[hidden]` 状態遷移前提の transition ベース preset であり、list の行
//! は `hidden` 属性を経由しない（`Insert`/`Remove` そのものが状態変化の
//! 表現）。加えて上記「`transition` ではなく `@keyframes`」節の 2 理由
//! （stagger との結合・FLIP 非干渉）にも合わない。本モジュールは
//! `presence_transition` を list へ読み替えて適用せず、独立の
//! `@keyframes` CSS を新設する。
//!
//! # `prefers-reduced-motion: reduce`
//!
//! [`ENTER_CSS`]/[`EXIT_CSS`] はいずれも `--fandhe-motion-duration-*`
//! トークン経由の `animation-duration`/`animation-delay` のみを使うため、
//! [`crate::theme::Theme::to_css`] が一括生成する reduced-motion ブロック
//! （`duration-*` トークンを `0ms` へ上書き）の対象に自然に含まれる
//! （個別の `@media` ブロックは不要）。無限反復・scroll-driven ではない
//! ため `animation-fill-mode: both`（enter）/`forwards`（exit）により
//! `0ms` でも最終状態で静止する。
//!
//! # 既知の挙動（意図的、本イシューのスコープ外）
//!
//! - SSR 初回描画時、行に既に [`PRESENCE_AUTO_ATTR`] が付いていると enter
//!   アニメーションが初回表示でも 1 回再生される（CSS `animation` は
//!   「属性が変化した瞬間」を検知できない、[`crate::forms_motion`]
//!   `SHAKE_CSS` と同じ既知のトレードオフ）。
//!
//! # styled 部品の公開 CSS 関数を持たない
//!
//! [`crate::forms_motion`]/[`crate::motion`] と同じ理由（各モジュール doc
//! 「styled 部品の公開 CSS 関数を持たない」節参照）により、本モジュールは
//! `crates/pre-styled-ui/src/stylesheet.rs` の
//! `all_styled_component_css_covers_every_component_module` が要求する
//! `pub fn css`/`stylesheet`（空引数）を持たない
//! （[`list_motion_css`] という別名を使う）。

use crate::recipe::{stagger_delay_declaration, MotionDuration};

/// keyed list の親要素（`list::root`）に付け、[`list_motion_css`] の
/// 対象・`fandhe-frontend-wasm-full` の `layout_flip::FLIP_AUTO_ATTR`/
/// `list_presence::PRESENCE_AUTO_ATTR`/`stagger_index::
/// STAGGER_AUTO_FIRST_ATTR` の対象であることを明示するオプトイン属性名。
/// 値は不問（存在のみを見る）。
///
/// `fandhe-frontend-wasm-full` 側の `list_presence::PRESENCE_AUTO_ATTR`
/// と同一リテラルを保つ契約（ドリフト検知は `tests/presence_auto_attr_drift.rs`）。
pub const PRESENCE_AUTO_ATTR: &str = "data-fandhe-presence-auto";

/// 追加行（enter）の `@keyframes` 名。
pub const ENTER_KEYFRAMES_NAME: &str = "fd-list-motion-enter";
/// 削除行（exit、`data-state="exiting"`）の `@keyframes` 名。
pub const EXIT_KEYFRAMES_NAME: &str = "fd-list-motion-exit";

/// 追加行の CSS 全文（末尾改行付き）。
///
/// [`PRESENCE_AUTO_ATTR`] を持つ `list::root` 直下の `list::item` へ、
/// stagger 遅延付きの enter アニメーションを適用する。`animation-delay`
/// は [`crate::recipe::stagger_delay_declaration`]（`--fandhe-motion-
/// stagger-index` を参照する `calc()`、既定 0 なので stagger 配線
/// （`stagger` feature）が無効でも遅延なしで動く）をそのまま使う。
pub fn enter_css() -> String {
    format!(
        concat!(
            "@keyframes ",
            "{enter_name}",
            " {{\n",
            "  from {{\n    opacity: 0;\n    scale: 0.95;\n  }}\n",
            "  to {{\n    opacity: 1;\n    scale: 1;\n  }}\n",
            "}}\n",
            "[data-scope=\"list\"][data-part=\"root\"][{presence_attr}] > [data-scope=\"list\"][data-part=\"item\"] {{\n",
            "  animation-name: {enter_name};\n",
            "  animation-duration: var(--fandhe-motion-duration-normal);\n",
            "  animation-timing-function: var(--fandhe-motion-easing-standard);\n",
            "  animation-fill-mode: both;\n",
            "  {stagger_delay};\n",
            "}}\n",
        ),
        enter_name = ENTER_KEYFRAMES_NAME,
        presence_attr = PRESENCE_AUTO_ATTR,
        stagger_delay = stagger_delay_declaration(MotionDuration::Fast).value(),
    )
}

/// 削除行（`data-state="exiting"`）の CSS 全文（末尾改行付き）。
///
/// `fandhe-frontend-wasm-full::list_presence::play_exit_after` が挿入する
/// ゴースト（`data-state="exiting"`）へ exit アニメーションを適用する。
/// `animation-delay: 0s` で stagger を打ち消す（退場は一斉に行う）。
/// `pointer-events: none`・`margin: 0` はゴースト自身の絶対配置座標を
/// レイアウトへ影響させないための保険（`insert_exit_ghost` が inline で
/// も設定する値だが、CSS が未読み込みの場合にも同じ値へ倒す）。
///
/// セレクタは [`PRESENCE_AUTO_ATTR`] を持つ `list::root` の子である
/// ことまで含めて明記する（属性セレクタ 4 個 = 詳細度 `(0,4,0)`）。
/// ゴーストは削除前後で `list::item` のまま残る（`play_exit_after` が
/// `data-key` 等の一部属性のみ剥がし `data-scope`/`data-part` は残す）ため、
/// [`enter_css`] のセレクタ（詳細度 `(0,5,0)`）にも一致してしまう。CSS の
/// 詳細度はソース順に優先しないため、ここを [`enter_css`] 未満のまま
/// にすると `data-state="exiting"` の行に enter の `animation-name` が
/// 勝ってしまい退場アニメーションが再生されない。属性セレクタをもう 1 個
/// （`data-state="exiting"`）追加した本セレクタは詳細度 `(0,4,0)` に
/// とどまり [`enter_css`] の `(0,5,0)` に劣後するため、[`PRESENCE_AUTO_ATTR`]
/// の重複指定でさらに 1 個積み増し `(0,5,0)` へ揃えたうえで、退場側にのみ
/// 存在する `data-state="exiting"` 分をもう 1 個積んだ `(0,6,0)` として
/// 常に enter を上回るようにする。
pub fn exit_css() -> String {
    format!(
        concat!(
            "@keyframes ",
            "{exit_name}",
            " {{\n",
            "  from {{\n    opacity: 1;\n    scale: 1;\n  }}\n",
            "  to {{\n    opacity: 0;\n    scale: 0.95;\n  }}\n",
            "}}\n",
            "[data-scope=\"list\"][data-part=\"root\"][{presence_attr}] > [data-scope=\"list\"][data-part=\"item\"][data-state=\"exiting\"] {{\n",
            "  animation-name: {exit_name};\n",
            "  animation-duration: var(--fandhe-motion-duration-normal);\n",
            "  animation-timing-function: var(--fandhe-motion-easing-standard);\n",
            "  animation-delay: 0s;\n",
            "  animation-fill-mode: forwards;\n",
            "  pointer-events: none;\n",
            "  margin: 0;\n",
            "}}\n",
        ),
        exit_name = EXIT_KEYFRAMES_NAME,
        presence_attr = PRESENCE_AUTO_ATTR,
    )
}

/// [`enter_css`]・[`exit_css`] を決定的な順序で連結して返す（決定的:
/// 同一呼び出しは常にバイト単位で同一の文字列を返す）。
#[must_use]
pub fn list_motion_css() -> String {
    let mut out = String::new();
    out.push_str(&enter_css());
    out.push_str(&exit_css());
    out
}

/// opt-in API（イシュー #2544）。[`crate::theme::Theme::to_css`] 本体は
/// 変更せず、その出力へ [`list_motion_css`] を追記するだけの別 impl
/// ブロックとして追加する（[`crate::theme::Theme::to_css_with_forms_motion`]
/// と同型）。
impl crate::theme::Theme {
    /// [`crate::theme::Theme::to_css`] の出力に本モジュールの opt-in CSS
    /// （[`list_motion_css`]）を追記して返す。
    ///
    /// `StyleSheet` 経由で個別に取り込みたい場合は
    /// `sheet.push_css(&list_motion::list_motion_css())` を使う。
    #[must_use]
    pub fn to_css_with_list_motion(&self) -> String {
        let mut out = self.to_css();
        out.push_str(&list_motion_css());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_css_targets_presence_auto_attr_scoped_items() {
        let css = enter_css();
        assert!(css.contains(&format!(
            "[data-scope=\"list\"][data-part=\"root\"][{PRESENCE_AUTO_ATTR}] > [data-scope=\"list\"][data-part=\"item\"] {{"
        )));
        assert!(css.contains("@keyframes fd-list-motion-enter {"));
    }

    #[test]
    fn enter_css_delay_matches_stagger_fast_declaration() {
        let css = enter_css();
        let expected = stagger_delay_declaration(MotionDuration::Fast).value();
        assert!(css.contains(expected));
    }

    #[test]
    fn exit_css_targets_exiting_state() {
        let css = exit_css();
        assert!(css.contains(&format!(
            "[data-scope=\"list\"][data-part=\"root\"][{PRESENCE_AUTO_ATTR}] > [data-scope=\"list\"][data-part=\"item\"][data-state=\"exiting\"] {{"
        )));
        assert!(css.contains("@keyframes fd-list-motion-exit {"));
        assert!(css.contains("animation-delay: 0s;"));
    }

    #[test]
    fn exit_selector_has_higher_specificity_than_enter_selector() {
        // codex-review/Bugbot 指摘是正の回帰: exit セレクタの属性セレクタ
        // 個数が enter セレクタ以上であることを機械的に固定する
        // （詳細度が enter 未満に戻ると退場アニメーションが再び enter に
        // 上書きされる）。
        let enter_attr_count = enter_css()
            .lines()
            .find(|line| line.starts_with("[data-scope"))
            .expect("enter_css にセレクタ行がない")
            .matches('[')
            .count();
        let exit_attr_count = exit_css()
            .lines()
            .find(|line| line.starts_with("[data-scope"))
            .expect("exit_css にセレクタ行がない")
            .matches('[')
            .count();
        assert!(exit_attr_count >= enter_attr_count);
    }

    #[test]
    fn keyframes_do_not_touch_transform_property() {
        // FLIP 非干渉契約: `transform` プロパティには一切触れない。
        let css = list_motion_css();
        assert!(!css.contains("transform"));
    }

    #[test]
    fn keyframes_names_do_not_collide_with_other_modules() {
        assert_ne!(ENTER_KEYFRAMES_NAME, crate::motion::FADE_IN_KEYFRAMES_NAME);
        assert_ne!(EXIT_KEYFRAMES_NAME, crate::motion::FADE_OUT_KEYFRAMES_NAME);
    }

    #[test]
    fn list_motion_css_never_contains_style_breakout_sequences() {
        let out = list_motion_css();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn list_motion_css_is_deterministic() {
        assert_eq!(list_motion_css(), list_motion_css());
    }

    #[test]
    fn list_motion_css_concatenates_enter_then_exit() {
        let out = list_motion_css();
        let enter_idx = out.find(&enter_css()).expect("enter_css が見つからない");
        let exit_idx = out.find(&exit_css()).expect("exit_css が見つからない");
        assert!(enter_idx < exit_idx);
    }

    #[test]
    fn to_css_with_list_motion_is_pure_append() {
        let theme = crate::theme::Theme::default();
        let base = theme.to_css();
        let extended = theme.to_css_with_list_motion();
        let extra = list_motion_css();
        assert!(extended.starts_with(&base));
        assert_eq!(extended.len(), base.len() + extra.len());
        assert_eq!(&extended[base.len()..], extra);
    }

    #[test]
    fn list_motion_css_passes_stylesheet_push_css() {
        let mut sheet = crate::stylesheet::StyleSheet::new();
        assert!(sheet.push_css(&list_motion_css()).is_ok());
    }
}
