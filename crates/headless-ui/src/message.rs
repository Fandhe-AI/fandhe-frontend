//! Message（shadcn/ui `Message` 相当）headless コンポーネント（イシュー
//! #2105、親 #2104、参照軸 #2001。2026-09-07 ユーザー判断（#2153）で確定した
//! shadcn/ui 主基準 3 者体制のもと `docs/design/component-coverage-map.md`
//! §12.1 の会話系部品として実装対象になった）。
//!
//! AI チャット UI の「会話 1 発言」を表現する [`root`] / [`avatar`] /
//! [`header`] / [`content`] / [`footer`] / [`group`] の 6 anatomy パーツを
//! 提供する。[`crate::item`](mod@crate::item)/[`crate::button_group`](mod@crate::button_group) と同型で状態
//! 機械（[`crate::state`]）を持たない静的部品であり、`fandhe-frontend-
//! wasm-full` の配線は不要（応答待ち・送信失敗・ストリーミング更新はすべて
//! 呼び出し側が `bool`/子ノードとして渡す）。
//!
//! # 会話系 4 部品の共通語彙（本モジュールが最初に確定する正）
//!
//! 本イシューは会話系 4 部品（message / bubble（#2108） / attachment（#2111）
//! / marker（#2114））のうち最初に着手されるものであり、以降の 3 部品が従う
//! 共通語彙をここで確定する。後続はこの rustdoc と
//! `crates/pre-styled-ui/tests/data_attr_vocabulary.rs` の契約テストを正
//! として同じ語彙を再利用する。
//!
//! - **`data-role`**（[`MessageRole`]）: `user` / `assistant` / `system` の
//!   3 値。shadcn/ui `Message` の `from` prop に対応する「発言者の役割」を
//!   表す。
//! - **`data-align`**（[`MessageAlign`]）: `start` / `end` の 2 値。[`root`]
//!   の水平整列を表す**独立した軸**であり、`data-role` から自動導出しない
//!   （shadcn は user を右寄せにする慣習を持つが、それは
//!   `fandhe-frontend-pre-styled-ui` または利用者が `role` と `align` を
//!   組み合わせて決める判断であり、本モジュールが決め打ちしない）。
//!   [`crate::positioning::Align`]（`start`/`center`/`end`、`data-side` と
//!   対で positioner に付く語彙）とは名前が近いが別物であり、`center` を
//!   持たず positioner 語彙でもないため独立した [`MessageAlign`] 列挙型と
//!   して定義する。
//! - **`data-loading`**/**`data-error`**: [`crate::data_attrs::data_disabled`]
//!   と同じ「存在で真を表す」存在属性（`bool` から `then_some` で生成）。
//!   応答待ち・送信失敗の**表示のみ**を担い、判定・再送はアプリ責務
//!   （`.claude/rules/coding-rust.md` §3.25）。[`crate::tree_view`](mod@crate::tree_view) の
//!   `data-loading` + `aria-busy` 対とは意図的に異なり、**`aria-busy` は
//!   付けない**（下記「`aria-live`/`aria-busy` を付けない理由」参照）。
//!
//! # `role="listitem"`/`role="list"`（`group` は正規化用のリストコンテナ）
//!
//! [`root`] は `role="listitem"` を固定付与する。ARIA の `listitem` は
//! `list`（またはそれと同等の `role="list"` コンテナ）を required context
//! として要求するため、[`group`] は `role="list"` + 任意 `aria-label`
//! （[`crate::item::group`] と同じ「空文字列なら省略」引数形）を固定付与
//! する。会話全体は「発言者ターンごとの [`group`]（list）の並び」として
//! スクリーンリーダーへ読み上げられる想定である。[`group`] を介さず
//! [`root`] 単体で使う場合は、呼び出し側が `ul`/`role="list"` コンテナ
//! （例: 会話全体のスクロールコンテナ、message-scroller #2121 のスコープ）
//! へ置く契約とする。
//!
//! 不採用案（検討記録）: `group` を `role="presentation"`（純 CSS フック）
//! にする案は `aria-label` を持てず group 単独で意味を持てないため不採用。
//! `article`/`feed` パターンはイシューが明記する「listitem 相当」という
//! 要件から外れるため不採用。
//!
//! # `aria-live`/`aria-busy` を付けない理由
//!
//! ストリーミング応答の通知（`aria-live` によるライブリージョン化）・
//! 応答待ちの読み上げ（`aria-busy`）はどちらも「いつ・どの頻度で読み上げる
//! か」というアプリケーション固有の UX 判断を要求する。本モジュールは
//! anatomy・アクセシビリティ・表示状態（`data-*`）までを責務とし
//! （`.claude/rules/coding-rust.md` §3.25）、通知系の判断は内包しない。
//! `data-loading`/`data-error` は見た目・スタイルフックとしてのみ機能し、
//! 通知が必要な利用者は自前で `aria-live` リージョンを合成する。
//!
//! # `avatar` はスロット（既存 [`crate::avatar`](mod@crate::avatar) を内包しない）
//!
//! [`avatar`] パーツは `div` のスロットであり、[`crate::avatar`](mod@crate::avatar) の
//! `Avatar` 状態機械や anatomy を内部で呼び出さない（[`crate::sidebar`](mod@crate::sidebar)
//! が他 scope を内包しない規則の踏襲）。呼び出し側が中身へ
//! `avatar::root`/`avatar::image`/`avatar::fallback` 等を自由に組み込む。
//!
//! # `group` は連続発言のまとめ（先頭以外の avatar 省略は CSS 側の責務）
//!
//! 同じ発言者が連続する場合に [`root`] を束ねるコンテナが [`group`]
//! である。「先頭の [`root`] のみ [`avatar`] を表示し以降は省略する」見た目
//! は `fandhe-frontend-pre-styled-ui` または利用者の CSS
//! （`:not(:first-child) [data-part="avatar"]` 等）が担い、本モジュールは
//! 構造のみを提供する。
//!
//! # `content` は中身を持たない（Markdown レンダリング結果等は利用者責務）
//!
//! [`content`] パーツはスロットであり、Markdown → HTML 変換・コードハイ
//! ライト等は行わない（本クレートの責務外、`.claude/rules/coding-rust.md`
//! §3.25 の「数値・日時整形は UI コンポーネント層の責務外」と同じ判断軸）。
//!
//! # 呼び出し文脈
//!
//! 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`] へ薄く委譲するのみ。
//! `fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み
//! Message（recipe・golden）を組み立てる想定（#2106、本イシューのスコープ
//! 外）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`role`/`data-role`/`data-align`/`data-loading`/`data-error`/
//!   `aria-label`）はすべて `&'static str` リテラルで固定しており、動的値
//!   が属性名スロットへ混入する経路はない。
//! - 動的値（呼び出し側 `attrs`/`children`/[`group`] の `label`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する
//!   （REQ-1）。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - **呼び出し側による予約キーのなりすまし除去**: `drop_reserved`
//!   （ASCII 大文字小文字無視の完全一致）が呼び出し側 `attrs` から本
//!   モジュールが固定付与する属性名を除去してから固定値を合成する
//!   （[`crate::item::root`] と同型のパターン）。`data-scope`/`data-part`
//!   の偽装は [`crate::anatomy::Anatomy::part`] が別途除去する。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-pre-styled-ui` 側の recipe・golden テスト・
//!   `site/themes/message.md`・Themes ページ・
//!   `docs/design/component-coverage-map.md` の「実装済み」化は #2106。
//! - wasm-full 側の配線: 不要（静的部品、状態機械なし）。
//! - 兄弟部品 bubble / attachment / marker の語彙追随は #2108 / #2111 /
//!   #2114（本モジュールの rustdoc を正として参照する）。
//! - 会話全体のスクロール・`aria-posinset`/`aria-setsize` 等は
//!   message-scroller #2121 のスコープで検討する。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, role};
use fandhe_frontend_core::Node;

/// Message の anatomy（`data-scope="message"`）。
const ANATOMY: Anatomy = anatomy("message");

/// [`root`] が固定付与する予約キー。
const ROOT_RESERVED: &[&str] = &[
    "role",
    "data-role",
    "data-align",
    "data-loading",
    "data-error",
];

/// [`avatar`]/[`header`]/[`content`]/[`footer`] は固定属性を持たないが、
/// 将来の追加に備え対称性のため空の予約キー定数を用意する
/// （`item::NO_RESERVED` と同型）。
const NO_RESERVED: &[&str] = &[];

/// [`group`] が固定付与する予約キー。
const GROUP_RESERVED: &[&str] = &["role", "aria-label"];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは状態属性の
/// なりすましを許してしまう（`item::drop_reserved` と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// 発言者の役割（`data-role`。shadcn/ui `Message` の `from` prop に対応）。
/// モジュール doc「会話系 4 部品の共通語彙」参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    /// 利用者本人の発言。
    User,
    /// AI アシスタントの発言。
    Assistant,
    /// システムメッセージ（エラー通知・操作案内等）。
    System,
}

impl MessageRole {
    /// `data-role` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }
}

impl Default for MessageRole {
    /// 既定は `User`（1 発言を単体で描画する最小構成のとき、送信主体で
    /// ある利用者を既定とみなす。shadcn/ui 自体は既定値を規定しないが、
    /// 本クレートの他 props 既定同様「最も単純な構成」を選ぶ判断軸に
    /// 従った）。
    fn default() -> Self {
        Self::User
    }
}

/// [`root`] の水平整列（`data-align`）。モジュール doc「会話系 4 部品の
/// 共通語彙」参照。`data-role` から独立した軸であり、`center` を持たない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAlign {
    /// 左寄せ（LTR 既定）。
    Start,
    /// 右寄せ（LTR 既定）。
    End,
}

impl MessageAlign {
    /// `data-align` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

impl Default for MessageAlign {
    /// 既定は `Start`（[`crate::positioning::Align`] の既定と揃える）。
    fn default() -> Self {
        Self::Start
    }
}

/// [`root`] の描画引数。将来の非破壊的拡張に備えた props 構造体
/// （[`crate::item::ItemRootProps`] と同型）。
#[derive(Debug, Clone, Copy, Default)]
pub struct MessageRootProps {
    /// 発言者の役割。
    pub role: MessageRole,
    /// 水平整列。
    pub align: MessageAlign,
    /// `true` なら `data-loading` 存在属性を付与する（応答待ちの表示）。
    pub loading: bool,
    /// `true` なら `data-error` 存在属性を付与する（送信失敗の表示）。
    pub error: bool,
}

/// `root` パーツ（`div`）。`role="listitem"` を固定付与し、`data-role`/
/// `data-align`/`data-loading`/`data-error` を出力する（モジュール doc
/// 「会話系 4 部品の共通語彙」「`role="listitem"`/`role="list"`」参照）。
#[must_use]
pub fn root<'a>(
    props: MessageRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("listitem"),
        ("data-role", props.role.as_str()),
        ("data-align", props.align.as_str()),
    ];
    if props.loading {
        merged.push(("data-loading", ""));
    }
    if props.error {
        merged.push(("data-error", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// `avatar` パーツ（`div`）。既存 [`crate::avatar`](mod@crate::avatar) を内包しないスロット
/// （モジュール doc「`avatar` はスロット」参照）。
#[must_use]
pub fn avatar<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("avatar", "div", attrs, children)
}

/// `header` パーツ（`div`）。発言者名・時刻等の上部スロット。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("header", "div", attrs, children)
}

/// `content` パーツ（`div`）。本文スロット（モジュール doc「`content` は
/// 中身を持たない」参照。Markdown レンダリング結果等は利用者責務）。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("content", "div", attrs, children)
}

/// `footer` パーツ（`div`）。アクション行（コピー・再生成等）の下部スロット。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("footer", "div", attrs, children)
}

/// `group` パーツ（`div`）。複数 [`root`] の連続発言まとめコンテナ。
/// `role="list"` を固定出力し（モジュール doc「`role="listitem"`/
/// `role="list"`」参照）、`label` は動的値であり
/// [`fandhe_frontend_core::render`] の既定エスケープを経由して
/// `aria-label` へ出力する（空文字列のときは省略する）。
#[must_use]
pub fn group<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, GROUP_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("list")];
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    merged.extend(attrs);
    ANATOMY.part("group", "div", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_defaults_are_listitem_user_start_no_loading_no_error() {
        let node = root(MessageRootProps::default(), vec![], vec![]);
        let html = render(&node);
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="message""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="listitem""#));
        assert!(html.contains(r#"data-role="user""#));
        assert!(html.contains(r#"data-align="start""#));
        assert!(!html.contains("data-loading"));
        assert!(!html.contains("data-error"));
        assert!(!html.contains("aria-live"));
        assert!(!html.contains("aria-busy"));
    }

    #[test]
    fn root_role_vocabulary_is_fixed() {
        for (role_value, expected) in [
            (MessageRole::User, "user"),
            (MessageRole::Assistant, "assistant"),
            (MessageRole::System, "system"),
        ] {
            let props = MessageRootProps {
                role: role_value,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-role="{expected}""#)));
        }
    }

    #[test]
    fn root_align_vocabulary_is_fixed() {
        for (align, expected) in [(MessageAlign::Start, "start"), (MessageAlign::End, "end")] {
            let props = MessageRootProps {
                align,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-align="{expected}""#)));
        }
    }

    #[test]
    fn root_align_is_independent_from_role() {
        // data-align は data-role から自動導出しない独立軸であることを
        // 固定する（モジュール doc「会話系 4 部品の共通語彙」参照）。
        let props = MessageRootProps {
            role: MessageRole::Assistant,
            align: MessageAlign::End,
            ..Default::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains(r#"data-role="assistant""#));
        assert!(html.contains(r#"data-align="end""#));
    }

    #[test]
    fn root_loading_and_error_are_presence_attributes() {
        let loading = render(&root(
            MessageRootProps {
                loading: true,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(loading.contains(r#"data-loading="""#));
        assert!(!loading.contains("data-error"));

        let error = render(&root(
            MessageRootProps {
                error: true,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(error.contains(r#"data-error="""#));
        assert!(!error.contains("data-loading"));
    }

    #[test]
    fn root_drops_reserved_attrs_case_insensitively() {
        let node = root(
            MessageRootProps::default(),
            vec![
                ("role", "note"),
                ("Data-Role", "assistant"),
                ("DATA-ALIGN", "end"),
                ("data-loading", "spoofed"),
                ("data-error", "spoofed"),
                ("data-testid", "kept"),
            ],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"role="listitem""#));
        assert!(!html.contains("role=\"note\""));
        assert!(html.contains(r#"data-role="user""#));
        assert!(html.contains(r#"data-align="start""#));
        assert!(!html.contains("spoofed"));
        assert!(html.contains(r#"data-testid="kept""#));
    }

    #[test]
    fn avatar_header_content_footer_are_plain_slots() {
        for html in [
            render(&avatar(vec![], vec![text("a")])),
            render(&header(vec![], vec![text("h")])),
            render(&content(vec![], vec![text("c")])),
            render(&footer(vec![], vec![text("f")])),
        ] {
            assert!(html.starts_with("<div"));
            assert!(html.contains(r#"data-scope="message""#));
        }
        assert!(render(&avatar(vec![], vec![])).contains(r#"data-part="avatar""#));
        assert!(render(&header(vec![], vec![])).contains(r#"data-part="header""#));
        assert!(render(&content(vec![], vec![])).contains(r#"data-part="content""#));
        assert!(render(&footer(vec![], vec![])).contains(r#"data-part="footer""#));
    }

    #[test]
    fn group_has_role_list_and_optional_aria_label() {
        let without_label = render(&group("", vec![], vec![]));
        assert!(without_label.contains(r#"role="list""#));
        assert!(!without_label.contains("aria-label"));

        let with_label = render(&group("Conversation", vec![], vec![]));
        assert!(with_label.contains(r#"role="list""#));
        assert!(with_label.contains(r#"aria-label="Conversation""#));
    }

    #[test]
    fn group_drops_reserved_attrs_case_insensitively() {
        let html = render(&group(
            "Conversation",
            vec![("Role", "presentation"), ("ARIA-LABEL", "spoofed")],
            vec![],
        ));
        assert!(html.contains(r#"role="list""#));
        assert!(html.contains(r#"aria-label="Conversation""#));
        assert!(!html.contains("spoofed"));
    }

    #[test]
    fn no_part_emits_hydration_attributes() {
        let html = render(&root(
            MessageRootProps::default(),
            vec![],
            vec![avatar(vec![], vec![]), group("", vec![], vec![])],
        ));
        assert!(!html.contains("data-hydrate-"));
    }
}
