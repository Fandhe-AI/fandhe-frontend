//! Bubble（shadcn/ui `Bubble` 相当）headless コンポーネント（イシュー
//! #2108、親 #2107、参照軸 #2001。2026-09-07 ユーザー判断（#2153）で確定した
//! shadcn/ui 主基準 3 者体制のもと `docs/design/component-coverage-map.md`
//! の会話系部品として実装対象になった）。
//!
//! チャット吹き出し 1 個を表現する [`root`] / [`content`] / [`reactions`] /
//! [`reaction`] / [`collapse_trigger`] / [`collapse_content`] の 6 anatomy
//! パーツを提供する。[`crate::message`](mod@crate::message) と同型で状態機械
//! （[`crate::state`]）を持たない静的部品であり、`fandhe-frontend-wasm-full`
//! の配線は現時点で未整備（下記「wasm-full 未配線」参照）。
//!
//! # 会話系 4 部品の共通語彙への追随（正は [`crate::message`](mod@crate::message)）
//!
//! `data-align`（[`crate::message::MessageAlign`] を再利用）は会話系 4 部品
//! （message（#2105）/ bubble（本モジュール）/ attachment（#2111）/
//! marker（#2114））が共有する語彙であり、正は [`crate::message`](mod@crate::message)
//! モジュール doc「会話系 4 部品の共通語彙」である。本モジュールは第 2 の
//! align 列挙型を作らず [`crate::message::MessageAlign`] をそのまま
//! [`BubbleRootProps::align`] に採用する。
//!
//! # `data-variant`（shadcn の 7 色調を 3 形態へ縮約）
//!
//! shadcn/ui `Bubble` は `default`/`secondary`/`muted`/`tinted`/`outline`/
//! `ghost`/`destructive` の 7 色調バリアントを持つが、本モジュールは
//! 「塗り・枠線・無装飾」という**見た目の形態**を表す
//! [`BubbleVariant`]（`solid`/`outline`/`plain`）の 3 値へ縮約する。実際の
//! 色調選択（アクセント・破壊的操作色等）は `fandhe-frontend-pre-styled-ui`
//! の `ColorPalette` 軸（イシュー #1678）の責務とする
//! （`.claude/rules/coding-rust.md` §3.25 規則 2「装飾・色調の関心は上層へ」
//! の一般化）。headless 層は「枠線があるか」「塗りがあるか」という構造的な
//! 区別のみを持つ。
//!
//! # `data-group-position`（算出は利用者責務）
//!
//! [`BubbleGroupPosition`]（`single`/`first`/`middle`/`last`）は連続発言の
//! 角丸連結（例: 同じ発言者の吹き出しが連続するとき隣接辺の角丸を
//! 潰す）を CSS 側が判定できるようにする**表示状態のみ**を表す。
//! 「リスト中の何番目か」「前後の発言者が同じか」を突き合わせて
//! `single`/`first`/`middle`/`last` を選ぶ計算はアプリケーションロジックで
//! あり本モジュールは内包しない（`.claude/rules/coding-rust.md` §3.25
//! 規則 1）。本モジュールはリスト全体を受け取らず [`root`] 単体の
//! 引数として 1 値を受け取るのみである。
//!
//! # `reactions`/`reaction`（`role="group"` を採用、`role="img"` は不採用）
//!
//! shadcn/ui の実装は `role="img"` + `aria-label` によるリアクション集計の
//! 表現を推奨するが、`img` ロールは children を presentational
//! （読み上げ対象外）にしてしまい、個々の [`reaction`] が持つ選択状態
//! （`data-selected`）を伝えられなくなる。本モジュールは
//! [`crate::message::group`]/[`crate::item::group`] と同型の
//! `role="group"` + 任意 `aria-label` を [`reactions`] に採用し、
//! [`reaction`] 自体は非インタラクティブな `span` に
//! `data-selected`（存在属性）のみを持たせる。リアクションの押下・集計・
//! トグルはアプリケーションロジック（§3.25 規則 1）であり内包しない。
//! shadcn `BubbleReactions` の `side`/`align`（配置）prop はレイアウト
//! 計測の関心のため headless へ持ち込まない（§3.25 規則 2）。
//!
//! # 折りたたみは bubble scope のまま [`crate::collapsible`] の属性契約を再利用
//!
//! [`crate::anatomy::Anatomy::part`] は scope を固定するため、
//! [`crate::collapsible::trigger`]/[`crate::collapsible::content`] へ
//! そのまま委譲すると `data-scope` が `"collapsible"` へ切り替わり、
//! docs-site の Anatomy/`data-*` 表（`data-scope="bubble"` からの機械導出）
//! から折りたたみパーツが消えてしまう。そのため本モジュールは
//! [`crate::state::OpenState`]・[`crate::data_attrs::data_state`]・
//! [`crate::aria::aria_expanded`]/[`crate::aria::aria_controls`]・
//! `type="button"` 固定・`id`/`hidden` という**属性の組み立て方のみ**を
//! [`crate::collapsible`] から再利用し、[`collapse_trigger`]/
//! [`collapse_content`] は bubble scope のパーツとして実装する。
//! [`crate::collapsible::trigger`] と異なり `disabled` は持たない
//! （イシュー要件外の最小構成）。
//!
//! # wasm-full 未配線（スコープ外）
//!
//! `fandhe-frontend-wasm-full` の `MAPPING_TABLE` に
//! `(bubble, collapse-trigger)` → `"toggle"` dispatch 行が無いため、
//! ハイドレーション後の折りたたみクリックは現時点で inert である。CSR での
//! 状態駆動は呼び出し側が [`crate::collapsible::Collapsible`] または
//! [`crate::state::Disclosure`] を保持し、その `state()` を
//! [`collapse_trigger`]/[`collapse_content`] へ注入する契約とする。
//! wasm-full 側の配線は後続イシューのスコープとする。
//!
//! # 呼び出し文脈
//!
//! 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・
//! [`crate::data_attrs`]・[`crate::state`] へ薄く委譲するのみ。
//! `fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! Bubble（recipe・golden）を組み立てる想定（#2109、本イシューのスコープ
//! 外）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`id`/`hidden`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない。
//! - 動的値（呼び出し側 `attrs`/`children`/[`reactions`] の
//!   `label`/[`collapse_trigger`] の `controls`/[`collapse_content`] の
//!   `id`）は [`fandhe_frontend_core::render`] の既定エスケープを必ず
//!   経由する（REQ-1）。`raw_html()` は使用せず、HTML 文字列を直接
//!   組み立てない。
//! - **呼び出し側による予約キーのなりすまし除去**: `drop_reserved`
//!   （ASCII 大文字小文字無視の完全一致）が呼び出し側 `attrs` から本
//!   モジュールが固定付与する属性名を除去してから固定値を合成する
//!   （[`crate::message`]/[`crate::collapsible`] と同型のパターン）。
//!   `data-scope`/`data-part` の偽装は
//!   [`crate::anatomy::Anatomy::part`] が別途除去する。
//! - 状態機械・hydration 属性を持たない静的部品のため、クライアント改ざん
//!   入力の復元経路を新設しない（`data-hydrate-` 非出力をテストで固定）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-pre-styled-ui` 側の recipe・golden テスト・
//!   `site/themes/bubble.md`・Themes ページ・
//!   `docs/design/component-coverage-map.md` の「実装済み」化は #2109。
//! - wasm-full 側の `MAPPING_TABLE` への `(bubble, collapse-trigger)` →
//!   `"toggle"` 行の追加は本モジュールのスコープ外（上記「wasm-full
//!   未配線」参照）。
//! - 兄弟部品 attachment（#2111）/ marker（#2114）への語彙追随。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_controls, aria_expanded, aria_label, role};
use crate::data_attrs::data_state;
use crate::message::MessageAlign;
use crate::state::OpenState;
use fandhe_frontend_core::Node;

/// Bubble の anatomy（`data-scope="bubble"`）。
const ANATOMY: Anatomy = anatomy("bubble");

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは状態属性の
/// なりすましを許してしまう（[`crate::message::drop_reserved`] と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// [`root`] の見た目の形態（`data-variant`）。モジュール doc
/// 「`data-variant`（shadcn の 7 色調を 3 形態へ縮約）」参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleVariant {
    /// 塗りつぶし（既定）。
    Solid,
    /// 枠線のみ。
    Outline,
    /// 無装飾（背景・枠線なし）。
    Plain,
}

impl BubbleVariant {
    /// `data-variant` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Outline => "outline",
            Self::Plain => "plain",
        }
    }
}

impl Default for BubbleVariant {
    /// 既定は `Solid`（shadcn/ui `Bubble` の既定色調 `default` に対応する
    /// 「最も単純な塗りつぶし表示」を選ぶ判断軸に従った）。
    fn default() -> Self {
        Self::Solid
    }
}

/// 連続発言中の [`root`] の位置（`data-group-position`）。モジュール doc
/// 「`data-group-position`（算出は利用者責務）」参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleGroupPosition {
    /// 単独（前後に連続する同一発言者の発言がない）。
    Single,
    /// 連続発言の先頭。
    First,
    /// 連続発言の中間。
    Middle,
    /// 連続発言の末尾。
    Last,
}

impl BubbleGroupPosition {
    /// `data-group-position` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::First => "first",
            Self::Middle => "middle",
            Self::Last => "last",
        }
    }
}

impl Default for BubbleGroupPosition {
    /// 既定は `Single`（1 個の吹き出しを単体で描画する最小構成のとき、
    /// 連続発言の判定を必要としない状態を既定とみなす）。
    fn default() -> Self {
        Self::Single
    }
}

/// [`root`] の描画引数。将来の非破壊的拡張に備えた props 構造体
/// （[`crate::message::MessageRootProps`] と同型）。
#[derive(Debug, Clone, Copy, Default)]
pub struct BubbleRootProps {
    /// 見た目の形態。
    pub variant: BubbleVariant,
    /// 水平整列（[`crate::message::MessageAlign`] を再利用。モジュール doc
    /// 「会話系 4 部品の共通語彙への追随」参照）。
    pub align: MessageAlign,
    /// 連続発言中の位置。
    pub group_position: BubbleGroupPosition,
}

/// [`root`] が固定付与する予約キー。
const ROOT_RESERVED: &[&str] = &["data-variant", "data-align", "data-group-position"];

/// `root` パーツ（`div`）。`data-variant`/`data-align`/`data-group-position`
/// を出力する（モジュール doc参照）。
#[must_use]
pub fn root<'a>(
    props: BubbleRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("data-variant", props.variant.as_str()),
        ("data-align", props.align.as_str()),
        ("data-group-position", props.group_position.as_str()),
    ];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// パーツが固定属性を持たない場合の空の予約キー定数
/// （[`crate::message::NO_RESERVED`] と同型）。
const NO_RESERVED: &[&str] = &[];

/// `content` パーツ（`div`）。本文スロット（Markdown レンダリング結果等は
/// 利用者責務、[`crate::message::content`] と同じ判断軸）。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("content", "div", attrs, children)
}

/// [`reactions`] が固定付与する予約キー。
const REACTIONS_RESERVED: &[&str] = &["role", "aria-label"];

/// `reactions` パーツ（`div`）。`role="group"` を固定出力し、`label` は
/// 動的値であり [`fandhe_frontend_core::render`] の既定エスケープを経由して
/// `aria-label` へ出力する（空文字列のときは省略する。モジュール doc
/// 「`reactions`/`reaction`」参照）。
#[must_use]
pub fn reactions<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, REACTIONS_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("group")];
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    merged.extend(attrs);
    ANATOMY.part("reactions", "div", merged, children)
}

/// [`reaction`] が固定付与する予約キー。
const REACTION_RESERVED: &[&str] = &["data-selected"];

/// `reaction` パーツ（`span`）。非インタラクティブな表示用パーツであり、
/// 押下・集計・トグルは内包しない（モジュール doc「`reactions`/
/// `reaction`」参照）。`selected` は「存在で真を表す」存在属性として
/// `data-selected` へ反映する。
#[must_use]
pub fn reaction<'a>(selected: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, REACTION_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![];
    if selected {
        merged.push(("data-selected", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("reaction", "span", merged, children)
}

/// [`collapse_trigger`] が固定付与する予約キー。
const COLLAPSE_TRIGGER_RESERVED: &[&str] =
    &["type", "aria-expanded", "aria-controls", "data-state"];

/// `collapse-trigger` パーツ（`button`）。フォーム内配置時の意図しない
/// submit を防ぐため `type="button"` を固定で付与する（A05 セキュリティ
/// 設定ミス対策、[`crate::collapsible::trigger`] と同型）。`controls` が
/// `Some` のとき `aria-controls` で [`collapse_content`] と関連付ける。
/// `disabled` は持たない（モジュール doc「折りたたみは bubble scope の
/// まま `collapsible` の属性契約を再利用」参照）。
#[must_use]
pub fn collapse_trigger<'a>(
    state: OpenState,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, COLLAPSE_TRIGGER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    merged.extend(attrs);
    ANATOMY.part("collapse-trigger", "button", merged, children)
}

/// [`collapse_content`] が固定付与する予約キー。
const COLLAPSE_CONTENT_RESERVED: &[&str] = &["data-state", "id", "hidden"];

/// `collapse-content` パーツ（`div`）。closed のとき `hidden` 存在属性を
/// 付与し、JS なしの SSR でも閉状態を表現する（[`crate::collapsible::content`]
/// と同型）。`id` が `Some` のとき [`collapse_trigger`] の `controls` と対で
/// `aria-controls` 関連付けを成立させる。
#[must_use]
pub fn collapse_content<'a>(
    state: OpenState,
    id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, COLLAPSE_CONTENT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("collapse-content", "div", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_defaults_are_solid_start_single() {
        let node = root(BubbleRootProps::default(), vec![], vec![]);
        let html = render(&node);
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="bubble""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-variant="solid""#));
        assert!(html.contains(r#"data-align="start""#));
        assert!(html.contains(r#"data-group-position="single""#));
    }

    #[test]
    fn root_variant_vocabulary_is_fixed() {
        for (variant, expected) in [
            (BubbleVariant::Solid, "solid"),
            (BubbleVariant::Outline, "outline"),
            (BubbleVariant::Plain, "plain"),
        ] {
            let props = BubbleRootProps {
                variant,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-variant="{expected}""#)));
        }
    }

    #[test]
    fn root_align_vocabulary_is_fixed() {
        for (align, expected) in [(MessageAlign::Start, "start"), (MessageAlign::End, "end")] {
            let props = BubbleRootProps {
                align,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-align="{expected}""#)));
        }
    }

    #[test]
    fn root_group_position_vocabulary_is_fixed() {
        for (position, expected) in [
            (BubbleGroupPosition::Single, "single"),
            (BubbleGroupPosition::First, "first"),
            (BubbleGroupPosition::Middle, "middle"),
            (BubbleGroupPosition::Last, "last"),
        ] {
            let props = BubbleRootProps {
                group_position: position,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-group-position="{expected}""#)));
        }
    }

    #[test]
    fn root_align_is_independent_from_variant_and_group_position() {
        // data-align は variant/group-position から自動導出しない独立軸で
        // あることを固定する（モジュール doc「会話系 4 部品の共通語彙への
        // 追随」参照）。
        let props = BubbleRootProps {
            variant: BubbleVariant::Outline,
            align: MessageAlign::End,
            group_position: BubbleGroupPosition::Last,
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains(r#"data-variant="outline""#));
        assert!(html.contains(r#"data-align="end""#));
        assert!(html.contains(r#"data-group-position="last""#));
    }

    #[test]
    fn root_drops_reserved_attrs_case_insensitively() {
        let node = root(
            BubbleRootProps::default(),
            vec![
                ("Data-Variant", "spoofed"),
                ("DATA-ALIGN", "end"),
                ("data-group-position", "spoofed"),
                ("data-testid", "kept"),
            ],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-variant="solid""#));
        assert!(html.contains(r#"data-align="start""#));
        assert!(html.contains(r#"data-group-position="single""#));
        assert!(!html.contains("spoofed"));
        assert!(html.contains(r#"data-testid="kept""#));
    }

    #[test]
    fn content_is_a_plain_slot() {
        let html = render(&content(vec![], vec![text("hello")]));
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="bubble""#));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains("hello"));
    }

    #[test]
    fn reactions_has_role_group_and_optional_aria_label() {
        let without_label = render(&reactions("", vec![], vec![]));
        assert!(without_label.contains(r#"role="group""#));
        assert!(!without_label.contains("aria-label"));

        let with_label = render(&reactions("3 reactions", vec![], vec![]));
        assert!(with_label.contains(r#"role="group""#));
        assert!(with_label.contains(r#"aria-label="3 reactions""#));
    }

    #[test]
    fn reactions_drops_reserved_attrs_case_insensitively() {
        let html = render(&reactions(
            "Reactions",
            vec![("Role", "presentation"), ("ARIA-LABEL", "spoofed")],
            vec![],
        ));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-label="Reactions""#));
        assert!(!html.contains("spoofed"));
    }

    #[test]
    fn reaction_selected_is_a_presence_attribute() {
        let selected = render(&reaction(true, vec![], vec![]));
        assert!(selected.contains(r#"data-selected="""#));

        let unselected = render(&reaction(false, vec![], vec![]));
        assert!(!unselected.contains("data-selected"));
    }

    #[test]
    fn reaction_drops_reserved_attrs_case_insensitively() {
        let html = render(&reaction(false, vec![("Data-Selected", "spoofed")], vec![]));
        assert!(!html.contains("data-selected"));
        assert!(!html.contains("spoofed"));
    }

    #[test]
    fn collapse_trigger_reflects_open_state_and_optional_controls() {
        let open = render(&collapse_trigger(
            OpenState::Open,
            Some("bubble-collapse-1"),
            vec![],
            vec![],
        ));
        assert!(open.contains(r#"type="button""#));
        assert!(open.contains(r#"aria-expanded="true""#));
        assert!(open.contains(r#"data-state="open""#));
        assert!(open.contains(r#"aria-controls="bubble-collapse-1""#));

        let closed = render(&collapse_trigger(OpenState::Closed, None, vec![], vec![]));
        assert!(closed.contains(r#"aria-expanded="false""#));
        assert!(closed.contains(r#"data-state="closed""#));
        assert!(!closed.contains("aria-controls"));
    }

    #[test]
    fn collapse_trigger_drops_reserved_attrs_case_insensitively() {
        let html = render(&collapse_trigger(
            OpenState::Closed,
            None,
            vec![
                ("Type", "submit"),
                ("ARIA-EXPANDED", "true"),
                ("Data-State", "open"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains(r#"type="submit""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn collapse_content_hides_when_closed_and_exposes_id() {
        let open = render(&collapse_content(
            OpenState::Open,
            Some("bubble-collapse-1"),
            vec![],
            vec![text("detail")],
        ));
        assert!(open.contains(r#"data-state="open""#));
        assert!(open.contains(r#"id="bubble-collapse-1""#));
        assert!(!open.contains("hidden"));
        assert!(open.contains("detail"));

        let closed = render(&collapse_content(OpenState::Closed, None, vec![], vec![]));
        assert!(closed.contains(r#"data-state="closed""#));
        assert!(closed.contains("hidden"));
        assert!(!closed.contains("id="));
    }

    #[test]
    fn collapse_content_drops_reserved_attrs_case_insensitively() {
        let html = render(&collapse_content(
            OpenState::Closed,
            None,
            vec![("Data-State", "open"), ("Hidden", "spoofed"), ("ID", "x")],
            vec![],
        ));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains("hidden"));
        assert!(!html.contains(r#"id="x""#));
    }

    #[test]
    fn no_part_emits_hydration_attributes() {
        let html = render(&root(
            BubbleRootProps::default(),
            vec![],
            vec![
                content(vec![], vec![]),
                reactions("", vec![], vec![reaction(true, vec![], vec![])]),
                collapse_trigger(OpenState::Closed, Some("c1"), vec![], vec![]),
                collapse_content(OpenState::Closed, Some("c1"), vec![], vec![]),
            ],
        ));
        assert!(!html.contains("data-hydrate-"));
    }

    #[test]
    fn nesting_message_content_does_not_leak_bubble_scope() {
        // message::content スロット内へ bubble を入れ子にしても、それぞれの
        // scope が独立して固定されることを固定する（メッセージ内に複数
        // 吹き出しを並べる想定の回帰）。
        let node = crate::message::content(
            vec![],
            vec![root(BubbleRootProps::default(), vec![], vec![])],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-scope="message""#));
        assert!(html.contains(r#"data-scope="bubble""#));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains(r#"data-part="root""#));
    }
}
