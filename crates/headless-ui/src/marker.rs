//! Marker（shadcn/ui `Marker` 相当）headless コンポーネント（イシュー
//! #2114、親 #2113、参照軸 #2001。2026-09-07 ユーザー判断（#2153）で確定した
//! shadcn/ui 主基準 3 者体制のもと `docs/design/component-coverage-map.md`
//! の会話系部品として実装対象になった）。
//!
//! 会話スレッド内のインライン注記行（システム注記・日付等の区切り・
//! ラベル付きセパレータ）を表す表示専用の静的部品。[`root`] / [`icon`] /
//! [`content`] の 3 anatomy パーツを提供する。[`mod@crate::message`]/
//! [`mod@crate::bubble`]/[`mod@crate::attachment`] と同型で状態機械
//! （[`crate::state`]）を持たず、`fandhe-frontend-wasm-full` の配線は
//! 不要である。
//!
//! # 区切り線は headless で描かない（`separator` パーツ再利用は上位層の責務）
//!
//! `fandhe-frontend-headless-ui` に `separator` モジュールは存在しない
//! （`fandhe-frontend-pre-styled-ui` の `separator.rs` が
//! `anatomy("separator")` を直接組み立てている）。`Divider`/`Label`
//! variant の区切り線描画（水平線・左右の線とラベル）は
//! `fandhe-frontend-pre-styled-ui` 側で `separator` パーツまたは
//! `[data-variant]` 条件の CSS を用いて行う契約とし、本モジュールは線要素
//! （`hr` / `role="separator"`）を一切出力しない。[`root`] は `div` の
//! 単純なコンテナ、[`icon`]/[`content`] は `span` の単純なスロットに限定
//! する（`.claude/rules/coding-rust.md` §3.25 規則 1 の装飾非内包の帰結。
//! 区切り線を `root` へ内包すると、Primitives showcase（headless 関数
//! のみで供給、デモ執筆規約 1「対象部品の `data-scope` を最外殻」）へ
//! 別 scope の装飾が混入してしまう）。
//!
//! # `data-variant`（`note` | `divider` | `label`）
//!
//! [`MarkerVariant`] は shadcn/ui `Marker` の `variant`（`default` |
//! `border` | `separator`）に対応する: `Note` ↔ `default`（インライン
//! 注記）、`Divider` ↔ `border`（行の下に境界線）、`Label` ↔
//! `separator`（中央ラベル + 左右の線）。shadcn/ui の変種名をそのまま
//! 採らず親イシューの語彙（`note`/`divider`/`label`）へ正規化する。
//!
//! # `data-tone`（`neutral` | `info` | `warning` | `danger`）
//!
//! [`MarkerTone`] の値語彙は `fandhe-frontend-pre-styled-ui`
//! `recipe::ColorPalette` の `neutral`/`info`/`warning`/`danger`（`callout`
//! が color-palette 軸として使う語）の部分集合であり、新語（`error` 等）
//! を作らない（`crates/pre-styled-ui/tests/data_attr_vocabulary.rs` に
//! 新しい値語彙を持ち込まない、というイシュー要件の実装）。`alert` が使う
//! `error` は採らず `callout` 系の `danger` を採る。
//!
//! # 会話系 4 部品の共通語彙への不追随（意図的）
//!
//! 会話系 4 部品（message（#2105）/ bubble（#2108）/ attachment（#2111）/
//! marker（本モジュール））の共通語彙の正は [`mod@crate::message`]
//! モジュール doc「会話系 4 部品の共通語彙」である。本モジュールは
//! `data-role`/`data-align` のいずれも持たない: 親イシュー（#2113）が
//! 列挙する表示状態は `data-variant`/`data-tone` のみであり、注記行は
//! ブロックの整列を持たない設計とするため、新語彙を割らない
//! （[`mod@crate::attachment`] モジュール doc「会話系 4 部品の共通語彙
//! への不追随」と同型の判断）。
//!
//! # アクセシビリティ
//!
//! - [`icon`] は装飾スロットとして `aria-hidden="true"` を固定付与する
//!   （shadcn `MarkerIcon` と同じ。headless 内の先例:
//!   [`mod@crate::breadcrumb`] の separator/ellipsis、
//!   [`mod@crate::accordion`] の item-indicator）。呼び出し側の
//!   `aria-hidden="false"` 偽装は予約キー除去で無効化する。
//! - [`root`] に `role` を固定付与しない（`fandhe-frontend-pre-styled-ui`
//!   の `callout` が `alert` ロールを持たないのと同じ判断: 静的注記に
//!   割り込み通知は不要）。ストリーミング中の注記へ `role="status"` +
//!   スピナーを付ける
//!   shadcn の案内は、呼び出し側が `attrs` で `("role", "status")` を渡す
//!   運用とする（「応答待ちの判定」はアプリケーションロジック、
//!   `.claude/rules/coding-rust.md` §3.25 規則 1）。`role` は予約キーに
//!   しない。
//! - [`content`] は固定属性を持たない素の `span`。
//!
//! # shadcn/ui 実 API との意図的差分
//!
//! shadcn/ui `Marker` の polymorphic root（`render`/`asChild`）・
//! `shimmer` ユーティリティ・`Spinner` 連携は不採用とする（装飾・
//! アニメーションは headless へ持ち込まない、
//! `.claude/rules/coding-rust.md` §3.25 規則 2）。`a`/`button` として
//! 描く要件はない。
//!
//! # 呼び出し文脈
//!
//! 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`] へ薄く委譲する
//! のみ。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル
//! 済み Marker（recipe・golden、`separator` パーツの再利用を含む）を
//! 組み立てる想定（#2115、本イシューのスコープ外）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-variant`/`data-tone`/`aria-hidden`/`data-scope`/
//!   `data-part`）はすべて `&'static str` リテラルで固定しており、動的
//!   値が属性名スロットへ混入する経路はない。
//! - 動的値（呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する
//!   （REQ-1）。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - **呼び出し側による予約キーのなりすまし除去**: `drop_reserved`
//!   （ASCII 大文字小文字無視の完全一致）が呼び出し側 `attrs` から本
//!   モジュールが固定付与する属性名を除去してから固定値を合成する
//!   （[`crate::attachment`]/[`crate::message`]/[`crate::bubble`] と同型
//!   のパターン）。`data-scope`/`data-part` の偽装は
//!   [`crate::anatomy::Anatomy::part`] が別途除去する。
//! - 状態機械・hydration 属性を持たない静的部品のため、クライアント改ざん
//!   入力の復元経路を新設しない（`data-hydrate-` 非出力をテストで固定）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-pre-styled-ui` 側の recipe・golden テスト・
//!   `site/themes/marker.md`・Themes ページ・
//!   `docs/design/component-coverage-map.md` の「実装済み」化は #2115。
//! - `divider`/`label` の区切り線描画における `separator::separator`/
//!   `group`/`label` の再利用は #2115（本モジュールは線要素を出力しない、
//!   上記「区切り線は headless で描かない」参照）。
//! - wasm-full 側の配線: 不要（静的部品、状態機械なし）。
//! - shadcn `render`（polymorphic root）/`shimmer`/`Spinner` 連携/
//!   `role="status"` の固定付与: 意図的非採用（呼び出し側 `attrs` で
//!   `role="status"` を付与可能）。
//! - 会話系共通語彙（`data-role`/`data-align`）への追随: 意図的不追随。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_hidden;
use fandhe_frontend_core::Node;

/// Marker の anatomy（`data-scope="marker"`）。
const ANATOMY: Anatomy = anatomy("marker");

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは状態属性の
/// なりすましを許してしまう（[`crate::attachment::drop_reserved`] と
/// 同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// [`root`] の表示形態（`data-variant`）。モジュール doc「`data-variant`
/// （`note` | `divider` | `label`）」参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerVariant {
    /// インライン注記表示（既定）。shadcn `default` 相当。
    Note,
    /// 行の下に境界線を伴う表示。shadcn `border` 相当（線自体は
    /// 上位層が描く、モジュール doc参照）。
    Divider,
    /// 中央ラベル + 左右の線を伴う表示。shadcn `separator` 相当（線自体は
    /// 上位層が描く、モジュール doc参照）。
    Label,
}

impl MarkerVariant {
    /// `data-variant` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Divider => "divider",
            Self::Label => "label",
        }
    }
}

impl Default for MarkerVariant {
    /// 既定は `Note`（shadcn の既定＝インライン注記と一致し、区切り線を
    /// 伴わない最も単純な構成であるため）。
    fn default() -> Self {
        Self::Note
    }
}

/// [`root`] の色調（`data-tone`）。モジュール doc「`data-tone`
/// （`neutral` | `info` | `warning` | `danger`）」参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerTone {
    /// 中立色（既定）。
    Neutral,
    /// 情報提供色。
    Info,
    /// 警告色。
    Warning,
    /// 危険・エラー色。
    Danger,
}

impl MarkerTone {
    /// `data-tone` の属性値文字列を返す。
    /// `fandhe-frontend-pre-styled-ui` `recipe::ColorPalette` の同名 4 値
    /// （`neutral`/`info`/`warning`/`danger`）と文字列一致させる
    /// （モジュール doc「`data-tone`」参照）。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }
}

impl Default for MarkerTone {
    /// 既定は `Neutral`（親イシュー列挙の先頭であり、システム注記の色調
    /// として最も単純であるため）。
    fn default() -> Self {
        Self::Neutral
    }
}

/// [`root`] の描画引数。将来の非破壊的拡張に備えた props 構造体
/// （[`crate::attachment::AttachmentRootProps`] と同型）。
#[derive(Debug, Clone, Copy, Default)]
pub struct MarkerRootProps {
    /// 表示形態。
    pub variant: MarkerVariant,
    /// 色調。
    pub tone: MarkerTone,
}

/// [`root`] が固定付与する予約キー。
const ROOT_RESERVED: &[&str] = &["data-variant", "data-tone"];

/// `root` パーツ（`div`）。`data-variant`/`data-tone` を出力する
/// （モジュール doc参照）。区切り線は出力しない（上記「区切り線は
/// headless で描かない」参照）。
#[must_use]
pub fn root<'a>(
    props: MarkerRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("data-variant", props.variant.as_str()),
        ("data-tone", props.tone.as_str()),
    ];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// [`icon`] が固定付与する予約キー。
const ICON_RESERVED: &[&str] = &["aria-hidden"];

/// `content` パーツが固定属性を持たない場合の空の予約キー定数
/// （[`crate::attachment::NO_RESERVED`] と同型）。
const NO_RESERVED: &[&str] = &[];

/// `icon` パーツ（`span`）。装飾スロットとして `aria-hidden="true"` を
/// 固定付与する（モジュール doc「アクセシビリティ」参照）。
#[must_use]
pub fn icon<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, ICON_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("icon", "span", merged, children)
}

/// `content` パーツ（`span`）。注記の本文テキストを children
/// （[`fandhe_frontend_core::text`]）で受け取る固定属性なしのスロット。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("content", "span", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_defaults_are_note_neutral() {
        let node = root(MarkerRootProps::default(), vec![], vec![]);
        let html = render(&node);
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="marker""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-variant="note""#));
        assert!(html.contains(r#"data-tone="neutral""#));
    }

    #[test]
    fn root_variant_vocabulary_is_fixed() {
        for (variant, expected) in [
            (MarkerVariant::Note, "note"),
            (MarkerVariant::Divider, "divider"),
            (MarkerVariant::Label, "label"),
        ] {
            let props = MarkerRootProps {
                variant,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-variant="{expected}""#)));
        }
    }

    #[test]
    fn root_tone_vocabulary_is_fixed() {
        for (tone, expected) in [
            (MarkerTone::Neutral, "neutral"),
            (MarkerTone::Info, "info"),
            (MarkerTone::Warning, "warning"),
            (MarkerTone::Danger, "danger"),
        ] {
            let props = MarkerRootProps {
                tone,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-tone="{expected}""#)));
        }
    }

    #[test]
    fn variant_and_tone_are_independent_axes() {
        let html = render(&root(
            MarkerRootProps {
                variant: MarkerVariant::Label,
                tone: MarkerTone::Danger,
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-variant="label""#));
        assert!(html.contains(r#"data-tone="danger""#));
    }

    #[test]
    fn root_drops_reserved_attrs_case_insensitively() {
        let node = root(
            MarkerRootProps::default(),
            vec![
                ("Data-Variant", "spoofed"),
                ("DATA-TONE", "spoofed"),
                ("data-testid", "kept"),
            ],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-variant="note""#));
        assert!(html.contains(r#"data-tone="neutral""#));
        assert!(!html.contains("spoofed"));
        assert!(html.contains(r#"data-testid="kept""#));
    }

    #[test]
    fn root_does_not_fix_a_role() {
        // 静的注記に割り込み通知は不要（モジュール doc「アクセシビリティ」
        // 参照）。呼び出し側が `attrs` で `role="status"` を渡すのは通る。
        let without_role = render(&root(MarkerRootProps::default(), vec![], vec![]));
        assert!(!without_role.contains("role="));

        let with_role = render(&root(
            MarkerRootProps::default(),
            vec![("role", "status")],
            vec![],
        ));
        assert!(with_role.contains(r#"role="status""#));
    }

    #[test]
    fn icon_is_aria_hidden_and_spoofing_is_dropped() {
        let html = render(&icon(vec![], vec![]));
        assert!(html.starts_with("<span"));
        assert!(html.contains(r#"data-part="icon""#));
        assert!(html.contains(r#"aria-hidden="true""#));

        let spoofed = render(&icon(vec![("aria-hidden", "false")], vec![]));
        assert_eq!(spoofed.matches("aria-hidden").count(), 1);
        assert!(spoofed.contains(r#"aria-hidden="true""#));
        assert!(!spoofed.contains(r#"aria-hidden="false""#));
    }

    #[test]
    fn content_is_a_plain_slot() {
        let html = render(&content(vec![], vec![text("Today")]));
        assert!(html.starts_with("<span"));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains("Today"));
    }

    #[test]
    fn no_part_emits_hydration_attributes() {
        let html = render(&root(
            MarkerRootProps::default(),
            vec![],
            vec![icon(vec![], vec![]), content(vec![], vec![text("Today")])],
        ));
        assert!(!html.contains("data-hydrate-"));
    }

    #[test]
    fn root_does_not_emit_a_line_element() {
        // headless 層は divider/label variant でも線要素（hr /
        // role="separator"）を出力しない（モジュール doc「区切り線は
        // headless で描かない」参照）。
        let html = render(&root(
            MarkerRootProps {
                variant: MarkerVariant::Divider,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(!html.contains("<hr"));
        assert!(!html.contains(r#"role="separator""#));
    }

    #[test]
    fn nesting_message_content_does_not_leak_marker_scope() {
        // message::content スロット内へ marker を入れ子にしても、
        // それぞれの scope が独立して固定されることを固定する（会話 1
        // 発言内にシステム注記を並べる想定の回帰）。
        let node = crate::message::content(
            vec![],
            vec![root(MarkerRootProps::default(), vec![], vec![])],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-scope="message""#));
        assert!(html.contains(r#"data-scope="marker""#));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains(r#"data-part="root""#));
    }
}
