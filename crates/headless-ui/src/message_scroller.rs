//! Message Scroller（shadcn/ui `Message Scroller` 相当）headless コンポーネント
//! （イシュー #2121、親 #2120、参照軸 #2001）。
//!
//! AI チャット UI の会話ログを収めるスクロールコンテナの anatomy（[`root`] /
//! [`viewport`] / [`content`] / [`anchor`] / [`jump_to_latest`] /
//! [`load_more`]）と表示状態 `data-*` のみを提供する。
//! [`mod@crate::message`] と同じく状態機械（[`crate::state`]）を持たない
//! 静的部品であり、最下部追従・新着検知・スクロール位置の計測・復元は
//! 一切内包しない（`.claude/rules/coding-rust.md` §3.25 規則 2
//! 「参照元が primitives 層へ持ち込んでいる装飾・アニメーション・レイアウト
//! 計測の関心は headless-ui へ持ち込まない」の適用例。
//! `docs/design/component-coverage-map.md` §12.1 参照）。
//!
//! # `data-stuck`/`data-has-new`（[`root`] の表示状態）
//!
//! - **`data-stuck`**（[`MessageScrollerStuck`]）: `bottom`/`free` の 2 値。
//!   利用者が最下部に張り付いているか、手動スクロールで離脱したかを表す。
//!   SSR は常に「最下部に居る」初期状態（既定 `Bottom`）を決定的に描画する
//!   （実行時のスクロール位置計測は `fandhe-frontend-wasm-full` 側、#2122
//!   のスコープ）。
//! - **`data-has-new`**: 存在属性（[`crate::data_attrs::data_disabled`] と
//!   同型の「存在で真を表す」語彙）。新着メッセージの有無を表す表示状態の
//!   みで、検知ロジックは内包しない。
//!
//! jump-to-latest の実際の可視判定（`stuck=free` かつ `has_new` 等の組み
//! 合わせ）は呼び出し側または #2122 が行い、本モジュールは [`jump_to_latest`]
//! に `visible: bool` を渡すだけの薄い契約とする。
//!
//! # `viewport` は `message-scroller` scope 自身のパーツ（`scroll_area` へ
//! 委譲しない）
//!
//! イシュータイトルの「scroll-area のパートを再利用」は**属性契約の再利用**
//! （[`crate::scroll_area::viewport`] と同じ `tabindex="0"` 固定 + 予約）と
//! 解釈し、[`crate::scroll_area::viewport`] を内部で呼び出すことはしない
//! （[`mod@crate::message`] が [`mod@crate::avatar`] を内包しない規則と同型）。
//! 理由は (1) イシュータイトルの 6 パーツに `viewport` が含まれ
//! `crates/docs-site/tests/primitive_showcase.rs` が本モジュールの
//! `.part("viewport", …)` を走査して Demo と突合する fail-closed 契約が
//! あること、(2) 他 scope を内包しない既存規則の踏襲、の 2 点。カスタム
//! スクロールバーが必要な利用者は [`crate::scroll_area::scrollbar`]/
//! [`crate::scroll_area::thumb`] を [`viewport`] 内へ入れ子にできる（両
//! scope は独立して共存できる）。
//!
//! # `content` に `role="log"` を固定付与しない
//!
//! shadcn は `role="log"`（暗黙 `aria-live="polite"` を伴う）+
//! `aria-relevant="additions"` を付けるが、これは
//! [`mod@crate::message`] が「ストリーミング通知はアプリ固有の UX 判断で
//! あり内包しない」と確定した方針そのものに当たる（本モジュール doc
//! 「`aria-live`/`aria-busy` を付けない理由」参照）。[`content`] は純スロット
//! とし `role` を予約しないため、必要な利用者は `attrs` 経由で
//! `role="log"` を渡せる。
//!
//! # `aria-live`/`aria-busy` を付けない理由
//!
//! [`mod@crate::message`] と同じ判断軸: 通知の頻度・タイミングはアプリ
//! 固有の UX 判断であり、本モジュールは anatomy・アクセシビリティ・表示
//! 状態（`data-*`）までを責務とする（`.claude/rules/coding-rust.md`
//! §3.25）。`data-stuck`/`data-has-new`/[`jump_to_latest`] の
//! `data-visible`/[`load_more`] の `data-loading` は見た目・スタイル
//! フックとしてのみ機能する。
//!
//! # `aria-posinset`/`aria-setsize` を持たない理由
//!
//! 総数はアプリのデータ（履歴の総件数）であり、無限履歴では SSR 時点で
//! 確定できない。必要な利用者は [`crate::message::root`] の `attrs` へ
//! 付与できる。
//!
//! # 意図的非採用（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - shadcn の `data-autoscrolling`/`data-pending-scroll`/`data-scrollable`
//!   （いずれも計測・遷移の実行時状態）は固定値で出力すると実態と乖離
//!   するため出力しない（[`crate::scroll_area`] と同じ論法）。
//! - `MessageScrollerItem`（行境界は [`crate::message::root`] が担う）。
//! - `scrollPreviousItemPeek`/`defaultScrollPosition`/
//!   `preserveScrollOnPrepend`（配線層の設定、#2122 のスコープ）。
//!
//! # 呼び出し文脈
//!
//! 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・[`crate::data_attrs`]
//! へ薄く委譲するのみ。`fandhe-frontend-wasm-full` が [`root`]/[`anchor`]
//! を観測してスクロール計測・`data-stuck`/`data-has-new`/`data-visible` の
//! 実行時更新を行う想定（#2122、本イシューのスコープ外）。
//! `fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! Message Scroller（recipe・golden）を組み立てる想定（#2123、同スコープ外）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`tabindex`/`role`/`aria-label`/`aria-hidden`/`type`/`hidden`/
//!   `disabled`/`data-*`）はすべて `&'static str` リテラルで固定しており、
//!   動的値が属性名スロットへ混入する経路はない。
//! - 動的値（呼び出し側 `attrs`/`children`/[`viewport`] と
//!   [`jump_to_latest`] の `label`）は [`fandhe_frontend_core::render`] の
//!   既定エスケープを必ず経由する（REQ-1）。`raw_html()` は使用せず、HTML
//!   文字列を直接組み立てない。
//! - **呼び出し側による予約キーのなりすまし除去**: `drop_reserved`
//!   （ASCII 大文字小文字無視の完全一致）が呼び出し側 `attrs` から本
//!   モジュールが固定付与する属性名を除去してから固定値を合成する
//!   （[`crate::message::root`] と同型のパターン）。`data-scope`/
//!   `data-part` の偽装は [`crate::anatomy::Anatomy::part`] が別途除去する。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `docs/design/component-coverage-map.md` の「実装済み」化・
//!   `fandhe-frontend-pre-styled-ui` 側の recipe・golden テスト・
//!   `site/themes/message-scroller.md`・Themes ページは #2123。
//! - wasm-full 側の配線（`MAPPING_TABLE` 登録・最下部追従・新着検知・
//!   履歴読み込み時の位置維持・`data-stuck`/`data-has-new`/`data-visible`
//!   の実行時更新）は #2122。
//! - [`crate::data_attrs`] への `data-stuck` 共有ヘルパ追加は行わない
//!   （本部品固有語彙であり、存在属性は [`mod@crate::message`] と同様に
//!   モジュール内でインライン生成する）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_hidden, aria_label, role};
use fandhe_frontend_core::Node;

/// Message Scroller の anatomy（`data-scope="message-scroller"`）。
const ANATOMY: Anatomy = anatomy("message-scroller");

/// [`root`] が固定付与する予約キー。
const ROOT_RESERVED: &[&str] = &["data-stuck", "data-has-new"];

/// [`viewport`] が固定付与する予約キー（`tabindex` は常時固定、`role`/
/// `aria-label` は `label` が非空のときのみ出力するが、なりすまし防止の
/// ため常に予約する）。
const VIEWPORT_RESERVED: &[&str] = &["tabindex", "role", "aria-label"];

/// [`content`] は固定属性を持たない純スロット（[`mod@crate::message`]
/// `NO_RESERVED` と同型）。
const NO_RESERVED: &[&str] = &[];

/// [`anchor`] が固定付与する予約キー。
const ANCHOR_RESERVED: &[&str] = &["aria-hidden"];

/// [`jump_to_latest`] が固定付与する予約キー。
const JUMP_TO_LATEST_RESERVED: &[&str] = &["type", "hidden", "data-visible", "aria-label"];

/// [`load_more`] が固定付与する予約キー。
const LOAD_MORE_RESERVED: &[&str] = &["type", "disabled", "data-disabled", "data-loading"];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致、[`crate::message::drop_reserved`]
/// と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// [`root`] の `data-stuck` 値（モジュール doc「`data-stuck`/`data-has-new`」
/// 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageScrollerStuck {
    /// 利用者が最下部に張り付いている（新着到着時は自動追従する想定）。
    Bottom,
    /// 利用者が手動スクロールで最下部から離脱した。
    Free,
}

impl MessageScrollerStuck {
    /// `data-stuck` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bottom => "bottom",
            Self::Free => "free",
        }
    }
}

impl Default for MessageScrollerStuck {
    /// 既定は `Bottom`（SSR は「最下部に居る」初期状態を決定的に描画する
    /// 要件、モジュール doc参照）。
    fn default() -> Self {
        Self::Bottom
    }
}

/// [`root`] の描画引数（[`crate::message::MessageRootProps`] と同型）。
#[derive(Debug, Clone, Copy, Default)]
pub struct MessageScrollerRootProps {
    /// 最下部追従の状態。
    pub stuck: MessageScrollerStuck,
    /// `true` なら `data-has-new` 存在属性を付与する（新着メッセージの
    /// 表示のみ、検知ロジックは内包しない）。
    pub has_new: bool,
}

/// `root` パーツ（`div`）。`data-stuck`/`data-has-new` を出力する
/// （モジュール doc「`data-stuck`/`data-has-new`」参照）。
#[must_use]
pub fn root<'a>(
    props: MessageScrollerRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-stuck", props.stuck.as_str())];
    if props.has_new {
        merged.push(("data-has-new", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// `viewport` パーツ（`div`）。`tabindex="0"` を固定付与する
/// （[`crate::scroll_area::viewport`] と同じ属性契約、モジュール doc
/// 「`viewport` は `message-scroller` scope 自身のパーツ」参照）。`label`
/// が非空のときのみ `role="region"` + `aria-label` を出力する（名前の
/// ない region を作らない）。`label` は動的値であり
/// [`fandhe_frontend_core::render`] の既定エスケープを経由する。
#[must_use]
pub fn viewport<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, VIEWPORT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("tabindex", "0")];
    if !label.is_empty() {
        merged.push(role("region"));
        merged.push(aria_label(label));
    }
    merged.extend(attrs);
    ANATOMY.part("viewport", "div", merged, children)
}

/// `content` パーツ（`div`）。純スロット（モジュール doc「`content` に
/// `role="log"` を固定付与しない」参照。`role` を予約しないため、必要な
/// 利用者は `attrs` 経由で `role="log"` 等を渡せる）。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("content", "div", attrs, children)
}

/// `anchor` パーツ（`div`）。[`viewport`] 末尾に置く最下部センチネル
/// （#2122 が観測してスクロール計測を行う対象）。`aria-hidden="true"` を
/// 固定付与し、children を持たない（純粋な計測用マーカーのため）。
#[must_use]
pub fn anchor<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    let attrs = drop_reserved(attrs, ANCHOR_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("anchor", "div", merged, vec![])
}

/// `jump-to-latest` パーツ（`button`）。`type="button"` を固定付与する。
/// `visible=false` のときは `hidden=""` を出力し `data-visible` を省略する
/// （JS 無効時に「押しても何も起きないボタン」を SSR で見せないため）。
/// `visible=true` のときは `data-visible=""` を出力し `hidden` を省略する。
/// `label` が非空のときのみ `aria-label` を出力する。`label` は動的値で
/// あり [`fandhe_frontend_core::render`] の既定エスケープを経由する。
#[must_use]
pub fn jump_to_latest<'a>(
    label: &'a str,
    visible: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, JUMP_TO_LATEST_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if visible {
        merged.push(("data-visible", ""));
    } else {
        merged.push(("hidden", ""));
    }
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    merged.extend(attrs);
    ANATOMY.part("jump-to-latest", "button", merged, children)
}

/// `load-more` パーツ（`button`）。`type="button"` を固定付与する。
/// `loading` は `data-loading=""` 存在属性、`disabled` はネイティブ
/// `disabled=""` + `data-disabled=""` を出力する。`loading`/`disabled` は
/// 自動連動させない（`loading=true, disabled=false` でもネイティブ
/// `disabled` は出力しない）。`aria-busy` は付けない（モジュール doc
/// 「`aria-live`/`aria-busy` を付けない理由」参照）。
#[must_use]
pub fn load_more<'a>(
    loading: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LOAD_MORE_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if disabled {
        merged.push(("disabled", ""));
        merged.push(("data-disabled", ""));
    }
    if loading {
        merged.push(("data-loading", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("load-more", "button", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_defaults_are_bottom_and_no_has_new() {
        let node = root(MessageScrollerRootProps::default(), vec![], vec![]);
        let html = render(&node);
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="message-scroller""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-stuck="bottom""#));
        assert!(!html.contains("data-has-new"));
    }

    #[test]
    fn root_stuck_vocabulary_is_fixed() {
        for (stuck, expected) in [
            (MessageScrollerStuck::Bottom, "bottom"),
            (MessageScrollerStuck::Free, "free"),
        ] {
            let props = MessageScrollerRootProps {
                stuck,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-stuck="{expected}""#)));
        }
    }

    #[test]
    fn root_has_new_is_presence_attribute() {
        let html = render(&root(
            MessageScrollerRootProps {
                has_new: true,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-has-new="""#));
    }

    #[test]
    fn root_drops_reserved_attrs_case_insensitively() {
        let node = root(
            MessageScrollerRootProps::default(),
            vec![
                ("DATA-STUCK", "free"),
                ("Data-Has-New", "spoofed"),
                ("data-testid", "kept"),
            ],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-stuck="bottom""#));
        assert!(!html.contains("spoofed"));
        assert!(html.contains(r#"data-testid="kept""#));
    }

    #[test]
    fn viewport_has_tabindex_zero_and_no_region_by_default() {
        let html = render(&viewport("", vec![], vec![]));
        assert!(html.contains(r#"data-part="viewport""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("role=\"region\""));
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn viewport_has_region_role_and_aria_label_when_labeled() {
        let html = render(&viewport("Conversation", vec![], vec![]));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"role="region""#));
        assert!(html.contains(r#"aria-label="Conversation""#));
    }

    #[test]
    fn viewport_drops_reserved_attrs_case_insensitively() {
        let html = render(&viewport(
            "Conversation",
            vec![
                ("TABINDEX", "-1"),
                ("Role", "presentation"),
                ("ARIA-LABEL", "spoofed"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"role="region""#));
        assert!(html.contains(r#"aria-label="Conversation""#));
        assert!(!html.contains("spoofed"));
    }

    #[test]
    fn content_is_a_plain_slot_without_role_log() {
        let html = render(&content(vec![], vec![text("body")]));
        assert!(html.contains(r#"data-scope="message-scroller""#));
        assert!(html.contains(r#"data-part="content""#));
        assert!(!html.contains("role"));
        assert!(!html.contains("aria-live"));
    }

    #[test]
    fn anchor_has_aria_hidden_and_no_children() {
        let html = render(&anchor(vec![]));
        assert!(html.contains(r#"data-part="anchor""#));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn anchor_drops_reserved_attrs_case_insensitively() {
        let html = render(&anchor(vec![("ARIA-HIDDEN", "false")]));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains(r#"aria-hidden="false""#));
    }

    #[test]
    fn jump_to_latest_hidden_when_not_visible() {
        let html = render(&jump_to_latest("", false, vec![], vec![]));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"hidden="""#));
        assert!(!html.contains("data-visible"));
    }

    #[test]
    fn jump_to_latest_visible_shows_data_visible_and_no_hidden() {
        let html = render(&jump_to_latest("Jump to latest", true, vec![], vec![]));
        assert!(html.contains(r#"data-visible="""#));
        assert!(!html.contains("hidden"));
        assert!(html.contains(r#"aria-label="Jump to latest""#));
    }

    #[test]
    fn jump_to_latest_omits_aria_label_when_empty() {
        let html = render(&jump_to_latest("", true, vec![], vec![]));
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn jump_to_latest_drops_reserved_attrs_case_insensitively() {
        let html = render(&jump_to_latest(
            "Jump",
            true,
            vec![
                ("TYPE", "submit"),
                ("Data-Visible", "spoofed"),
                ("HIDDEN", ""),
            ],
            vec![],
        ));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains(r#"type="submit""#));
        assert!(html.contains(r#"data-visible="""#));
        assert!(!html.contains("spoofed"));
    }

    #[test]
    fn load_more_loading_and_disabled_do_not_auto_link() {
        let loading_only = render(&load_more(true, false, vec![], vec![]));
        assert!(loading_only.contains(r#"data-loading="""#));
        assert!(!loading_only.contains("disabled"));

        let disabled_only = render(&load_more(false, true, vec![], vec![]));
        assert!(disabled_only.contains(r#"disabled="""#));
        assert!(disabled_only.contains(r#"data-disabled="""#));
        assert!(!disabled_only.contains("data-loading"));

        let both = render(&load_more(true, true, vec![], vec![]));
        assert!(both.contains(r#"data-loading="""#));
        assert!(both.contains(r#"disabled="""#));
        assert!(both.contains(r#"data-disabled="""#));
    }

    #[test]
    fn load_more_does_not_emit_aria_busy() {
        let html = render(&load_more(true, true, vec![], vec![]));
        assert!(!html.contains("aria-busy"));
    }

    #[test]
    fn load_more_drops_reserved_attrs_case_insensitively() {
        let html = render(&load_more(
            false,
            false,
            vec![
                ("TYPE", "submit"),
                ("Disabled", ""),
                ("Data-Loading", "spoofed"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains(r#"type="submit""#));
        assert!(!html.contains("disabled"));
        assert!(!html.contains("spoofed"));
    }

    #[test]
    fn no_part_emits_hydration_attributes_or_measurement_state() {
        let html = render(&root(
            MessageScrollerRootProps::default(),
            vec![],
            vec![
                viewport("", vec![], vec![content(vec![], vec![]), anchor(vec![])]),
                jump_to_latest("", false, vec![], vec![]),
                load_more(false, false, vec![], vec![]),
            ],
        ));
        assert!(!html.contains("data-hydrate-"));
        assert!(!html.contains("data-autoscrolling"));
        assert!(!html.contains("data-pending-scroll"));
        assert!(!html.contains("data-scrollable"));
        assert!(!html.contains("aria-live"));
        assert!(!html.contains("aria-busy"));
        assert!(!html.contains("aria-posinset"));
        assert!(!html.contains("aria-setsize"));
    }
}
