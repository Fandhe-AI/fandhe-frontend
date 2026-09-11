//! Attachment（shadcn/ui `Attachment` 相当）headless コンポーネント（イシュー
//! #2111、親 #2110、参照軸 #2001。2026-09-07 ユーザー判断（#2153）で確定した
//! shadcn/ui 主基準 3 者体制のもと `docs/design/component-coverage-map.md`
//! の会話系部品として実装対象になった）。
//!
//! AI チャット UI の「添付ファイル 1 件の表示」を表現する [`root`] /
//! [`media`] / [`content`] / [`name`] / [`meta`] / [`progress`] /
//! [`actions`] / [`action`] の 8 anatomy パーツを提供する。既存
//! [`crate::file_upload`](mod@crate::file_upload) が「選択・ドロップ」の入力側部品であるのに
//! 対し、本モジュールは「表示側」の別部品である。[`crate::message`](mod@crate::message)/
//! [`crate::bubble`](mod@crate::bubble) と同型で状態機械（[`crate::state`]）を持たない
//! 静的部品であり、`fandhe-frontend-wasm-full` の配線は不要（削除等の操作は
//! すべて呼び出し側が [`action`] の click ハンドラとして配線する）。
//!
//! # イシュータイトルとの差分（`action` パーツの追加）
//!
//! イシュータイトルは anatomy として `root / media / content / name / meta /
//! progress / actions` の 7 パーツを挙げるが、イシュー本文は「削除
//! アクションは `button` + `aria-label` を出力する」ことを要求している。
//! [`actions`]（アクション群のコンテナ）だけではこの要求を満たせないため、
//! 8 番目のパーツとして [`action`] を追加する（`crate::docs_site::nav`
//! の `header_nav` が差分記録を rustdoc に残す流儀を踏襲、根拠は
//! `crates/docs-site/src/nav.rs` の `header_nav` rustdoc「イシュータイトルと
//! の差分」節を参照）。
//!
//! # 会話系 4 部品の共通語彙への不追随（意図的）
//!
//! 会話系 4 部品（message（#2105）/ bubble（#2108）/ attachment（本
//! モジュール）/ marker（#2114））の共通語彙の正は [`crate::message`](mod@crate::message)
//! モジュール doc「会話系 4 部品の共通語彙」である。本モジュールは
//! `data-role`/`data-align` のいずれも持たない: 親イシュー（#2110）が
//! 列挙する表示状態は `data-variant`/`data-state`/`data-disabled` のみで
//! あり、添付ファイルは常に外側の [`crate::message`](mod@crate::message)/
//! [`crate::bubble`](mod@crate::bubble) の [`crate::message::content`]/[`crate::bubble::content`]
//! スロット内に置かれ、整列（`data-align`）はその親から継承する設計と
//! するため、新語彙を割らない。
//!
//! # `data-variant`（`file` | `image`）と `media` スロット
//!
//! [`AttachmentVariant`] は shadcn/ui `Attachment` の表示形態（画像
//! プレビューかファイルアイコンか）を表す。[`root`] にのみ付与し、
//! [`media`] パーツ自体は独自の `data-variant` を持たない（
//! `fandhe-frontend-pre-styled-ui`/利用者側 CSS は
//! `[data-variant="image"] [data-part="media"]` の子孫セレクタで分岐する
//! 設計とする）。この点は [`crate::item`](mod@crate::item) の media パーツが独自
//! variant を持つ設計とは意図的に異なる（親イシュー #2110 が root 側の
//! `file`/`image` を要求しているため）。[`media`] は画像プレビュー
//! （`img`）または種別アイコンを children として受けるだけのスロットで
//! あり、画像読み込み失敗時のフォールバック等は行わない
//! （[`crate::avatar`](mod@crate::avatar) の `Avatar` 状態機械は内包しない）。
//!
//! # `data-state`（`idle` | `uploading` | `error`）
//!
//! [`AttachmentState`] は親イシューが列挙する 3 値に固定する。shadcn/ui
//! 実装が持つ `processing`/`done` は意図的に不採用とする（`done` は
//! `idle` に包含できるため。将来必要になれば非破壊的に列挙子を追加できる）。
//! アップロード進捗の判定・エラー分類はアプリケーションロジックであり
//! 本モジュールは内包しない（`.claude/rules/coding-rust.md` §3.25 規則 1）。
//!
//! # `name`/`meta` は整形済み文字列を受け取るだけのスロット
//!
//! [`name`]/[`meta`] はファイル名・種別・サイズ・進捗率等の**整形済み
//! 文字列**を children（[`fandhe_frontend_core::text`]）として受け取る
//! だけであり、byte → KB 変換等の数値・単位整形は行わない
//! （`docs/policy/intentional-non-adoption.md` §3.23「数値・日時整形は
//! UI コンポーネント層の責務外」の系、`.claude/rules/coding-rust.md`
//! §3.25 規則 1 の一般化）。
//!
//! # `progress` は attachment scope のスロット（[`crate::progress::Progress`] を委譲しない）
//!
//! [`crate::anatomy::Anatomy::part`] は scope を固定するため、
//! [`crate::progress::Progress`] のパーツメソッドへ直接委譲すると
//! `data-scope` が `"progress"` へ切り替わり、docs-site の Anatomy/
//! `data-*` 表（`data-scope="attachment"` からの機械導出）から `progress`
//! パートが消えてしまう（[`crate::bubble`](mod@crate::bubble) が折りたたみパーツで
//! 直面したのと同じ罠）。そのため [`progress`] は attachment scope の
//! 単純な `div` スロットとし、呼び出し側が中身へ
//! [`crate::progress::Progress`] のパーツ群（`root`/`track`/`range` 等、
//! `data-scope="progress"`）を children として入れ子にする契約とする。
//! 両 scope は独立して残る（`tests/attachment.rs` の入れ子テストで固定）。
//!
//! # `actions`/`action`（削除等の個別アクション）
//!
//! [`actions`] はアクションボタン群を束ねるコンテナ（`div`）。[`action`]
//! （`button`）は shadcn `AttachmentAction` 相当で、`type="button"`
//! （フォーム内配置時の意図しない submit を防ぐ、A05 セキュリティ設定
//! ミス対策）・`label` が空文字列でないときのみ `aria-label`・`disabled`
//! 引数によるネイティブ `disabled` + `data-disabled` を出力する
//! （[`crate::file_upload::item_delete_trigger`] と同型のパターン）。
//! アイコンのみのボタン（children にテキストを持たない構成）では必ず
//! `label` を渡す契約とする。削除処理そのものはアプリケーションロジックで
//! あり本モジュールは内包しない（§3.25 規則 1）。
//!
//! # shadcn/ui 実 API との意図的差分
//!
//! shadcn/ui `Attachment` は `state: idle|uploading|processing|error|done`・
//! `size`・`orientation`・`AttachmentTitle`/`AttachmentDescription`・
//! `AttachmentTrigger`（カード全面のクリックオーバーレイ）・
//! `AttachmentGroup`（横スクロールコンテナ）を持つ。本モジュールは
//! `data-state` を親イシューの 3 値へ限定し、`size`/`orientation` は
//! 装飾・レイアウト計測の関心のため headless へ持ち込まない
//! （`.claude/rules/coding-rust.md` §3.25 規則 2、必要なら
//! `fandhe-frontend-pre-styled-ui` 側で扱う）。`trigger`/`group` は本
//! イシューのスコープ外とする（下記「スコープ外」参照）。
//!
//! # 呼び出し文脈
//!
//! 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・
//! [`crate::data_attrs`] へ薄く委譲するのみ。`fandhe-frontend-pre-styled-ui`
//! が本モジュールを呼んでスタイル済み Attachment（recipe・golden）を
//! 組み立てる想定（#2112、本イシューのスコープ外）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`type`/`disabled`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない。
//! - 動的値（呼び出し側 `attrs`/`children`/[`action`] の `label`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する
//!   （REQ-1）。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//!   `name`/`meta` は不透明な text スロットであり、部品内で整形ロジック
//!   （`format!` による HTML 近傍の文字列組み立て）を持たない。
//! - **呼び出し側による予約キーのなりすまし除去**: `drop_reserved`
//!   （ASCII 大文字小文字無視の完全一致）が呼び出し側 `attrs` から本
//!   モジュールが固定付与する属性名を除去してから固定値を合成する
//!   （[`crate::message`]/[`crate::bubble`] と同型のパターン）。
//!   `data-scope`/`data-part` の偽装は
//!   [`crate::anatomy::Anatomy::part`] が別途除去する。
//! - 状態機械・hydration 属性を持たない静的部品のため、クライアント改ざん
//!   入力の復元経路を新設しない（`data-hydrate-` 非出力をテストで固定。
//!   [`progress`] スロットへ [`crate::progress::Progress`] のパーツ
//!   メソッド（`root`/`track`/`range` 等）を直接呼んで入れ子にする限り、
//!   `Progress` 自体も hydration 属性を出力しない。`Progress` が
//!   `Component::render_for_hydration` 経由で呼ばれ
//!   `Hydrate::hydration_attrs` が付与されるのは、呼び出し側がその
//!   統合様式を明示的に選んだ場合のみである）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-pre-styled-ui` 側の recipe・golden テスト・
//!   `site/themes/attachment.md`・Themes ページ・
//!   `docs/design/component-coverage-map.md` の「実装済み」化は #2112。
//! - wasm-full 側の配線: 不要（静的部品、状態機械なし。[`action`] の
//!   click 配線はアプリ責務）。
//! - 兄弟部品 marker（#2114）への語彙追随。
//! - shadcn `AttachmentTrigger`（カード全面クリックオーバーレイ）/
//!   `AttachmentGroup`（横スクロールコンテナ）/ `processing`・`done` 状態 /
//!   `size`・`orientation`（上記「shadcn/ui 実 API との意図的差分」参照）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_label;
use crate::data_attrs::{data_disabled, data_state};
use fandhe_frontend_core::Node;

/// Attachment の anatomy（`data-scope="attachment"`）。
const ANATOMY: Anatomy = anatomy("attachment");

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

/// [`root`] の表示形態（`data-variant`）。モジュール doc
/// 「`data-variant`（`file` | `image`）と `media` スロット」参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentVariant {
    /// 種別アイコン表示（既定）。
    File,
    /// 画像プレビュー表示。
    Image,
}

impl AttachmentVariant {
    /// `data-variant` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Image => "image",
        }
    }
}

impl Default for AttachmentVariant {
    /// 既定は `File`（画像プレビューが取得できない添付が最も単純な
    /// 構成であるため。他 props 既定と同じ判断軸）。
    fn default() -> Self {
        Self::File
    }
}

/// [`root`] のアップロード状態（`data-state`）。モジュール doc
/// 「`data-state`（`idle` | `uploading` | `error`）」参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentState {
    /// 通常表示（アップロード完了・待機中を含む、既定）。
    Idle,
    /// アップロード進行中。[`progress`] スロットと併用する想定。
    Uploading,
    /// アップロード失敗。
    Error,
}

impl AttachmentState {
    /// `data-state` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Uploading => "uploading",
            Self::Error => "error",
        }
    }
}

impl Default for AttachmentState {
    /// 既定は `Idle`（アップロード動作を伴わない最も単純な表示）。
    fn default() -> Self {
        Self::Idle
    }
}

/// [`root`] の描画引数。将来の非破壊的拡張に備えた props 構造体
/// （[`crate::bubble::BubbleRootProps`] と同型）。
#[derive(Debug, Clone, Copy, Default)]
pub struct AttachmentRootProps {
    /// 表示形態。
    pub variant: AttachmentVariant,
    /// アップロード状態。
    pub state: AttachmentState,
    /// `true` なら `data-disabled` 存在属性を付与する。
    pub disabled: bool,
}

/// [`root`] が固定付与する予約キー。
const ROOT_RESERVED: &[&str] = &["data-variant", "data-state", "data-disabled"];

/// `root` パーツ（`div`）。`data-variant`/`data-state`/`data-disabled` を
/// 出力する（モジュール doc参照）。
#[must_use]
pub fn root<'a>(
    props: AttachmentRootProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("data-variant", props.variant.as_str()),
        data_state(props.state.as_str()),
    ];
    merged.extend(data_disabled(props.disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// パーツが固定属性を持たない場合の空の予約キー定数
/// （[`crate::message::NO_RESERVED`] と同型）。
const NO_RESERVED: &[&str] = &[];

/// `media` パーツ（`div`）。画像プレビュー（`img`）または種別アイコンを
/// children として受けるスロット（モジュール doc「`data-variant`
/// （`file` | `image`）と `media` スロット」参照）。
#[must_use]
pub fn media<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("media", "div", attrs, children)
}

/// `content` パーツ（`div`）。[`name`]/[`meta`] を束ねるスロット。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("content", "div", attrs, children)
}

/// `name` パーツ（`span`）。ファイル名を children
/// （[`fandhe_frontend_core::text`]）で受け取るスロット。
#[must_use]
pub fn name<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("name", "span", attrs, children)
}

/// `meta` パーツ（`span`）。種別・サイズ・進捗率等の**整形済み文字列**を
/// children で受け取るスロット（モジュール doc「`name`/`meta` は整形済み
/// 文字列を受け取るだけのスロット」参照）。
#[must_use]
pub fn meta<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("meta", "span", attrs, children)
}

/// `progress` パーツ（`div`）。attachment scope の単純なスロットであり、
/// 呼び出し側が中身へ [`crate::progress::Progress`] のパーツ群を入れ子に
/// する契約とする（モジュール doc「`progress` は attachment scope の
/// スロット」参照）。
#[must_use]
pub fn progress<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("progress", "div", attrs, children)
}

/// `actions` パーツ（`div`）。[`action`] 群を束ねるコンテナ。
#[must_use]
pub fn actions<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("actions", "div", attrs, children)
}

/// [`action`] が固定付与する予約キー。
const ACTION_RESERVED: &[&str] = &["type", "aria-label", "disabled", "data-disabled"];

/// `action` パーツ（`button`）。削除等の個別アクション（shadcn
/// `AttachmentAction` 相当、モジュール doc「`actions`/`action`」参照）。
/// `type="button"` を固定付与し、`label` が空文字列でないときのみ
/// `aria-label` を出力する。`disabled` はネイティブ `disabled` +
/// `data-disabled` の両方に反映する
/// （[`crate::file_upload::item_delete_trigger`] と同型のパターン）。
#[must_use]
pub fn action<'a>(
    label: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ACTION_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("action", "button", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_defaults_are_file_idle_enabled() {
        let node = root(AttachmentRootProps::default(), vec![], vec![]);
        let html = render(&node);
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="attachment""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-variant="file""#));
        assert!(html.contains(r#"data-state="idle""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_variant_vocabulary_is_fixed() {
        for (variant, expected) in [
            (AttachmentVariant::File, "file"),
            (AttachmentVariant::Image, "image"),
        ] {
            let props = AttachmentRootProps {
                variant,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-variant="{expected}""#)));
        }
    }

    #[test]
    fn root_state_vocabulary_is_fixed() {
        for (state, expected) in [
            (AttachmentState::Idle, "idle"),
            (AttachmentState::Uploading, "uploading"),
            (AttachmentState::Error, "error"),
        ] {
            let props = AttachmentRootProps {
                state,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-state="{expected}""#)));
        }
    }

    #[test]
    fn root_disabled_is_a_presence_attribute() {
        let disabled = render(&root(
            AttachmentRootProps {
                disabled: true,
                ..Default::default()
            },
            vec![],
            vec![],
        ));
        assert!(disabled.contains(r#"data-disabled="""#));

        let enabled = render(&root(AttachmentRootProps::default(), vec![], vec![]));
        assert!(!enabled.contains("data-disabled"));
    }

    #[test]
    fn root_drops_reserved_attrs_case_insensitively() {
        let node = root(
            AttachmentRootProps::default(),
            vec![
                ("Data-Variant", "spoofed"),
                ("DATA-STATE", "error"),
                ("data-disabled", "spoofed"),
                ("data-testid", "kept"),
            ],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-variant="file""#));
        assert!(html.contains(r#"data-state="idle""#));
        assert!(!html.contains("spoofed"));
        assert!(html.contains(r#"data-testid="kept""#));
    }

    #[test]
    fn media_content_name_meta_are_plain_slots() {
        let media_html = render(&media(vec![], vec![text("icon")]));
        assert!(media_html.contains(r#"data-part="media""#));
        assert!(media_html.contains("icon"));

        let content_html = render(&content(vec![], vec![text("wrap")]));
        assert!(content_html.contains(r#"data-part="content""#));

        let name_html = render(&name(vec![], vec![text("report.pdf")]));
        assert!(name_html.starts_with("<span"));
        assert!(name_html.contains(r#"data-part="name""#));
        assert!(name_html.contains("report.pdf"));

        let meta_html = render(&meta(vec![], vec![text("PDF · 128 KB")]));
        assert!(meta_html.starts_with("<span"));
        assert!(meta_html.contains(r#"data-part="meta""#));
        assert!(meta_html.contains("128 KB"));
    }

    #[test]
    fn progress_is_a_plain_slot() {
        let html = render(&progress(vec![], vec![text("64%")]));
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="attachment""#));
        assert!(html.contains(r#"data-part="progress""#));
        assert!(html.contains("64%"));
    }

    #[test]
    fn progress_nesting_crate_progress_keeps_scopes_independent() {
        // attachment::progress スロットへ crate::progress::Progress のパーツを
        // 入れ子にしても、それぞれの scope が独立して固定されることを固定
        // する（モジュール doc「`progress` は attachment scope のスロット」
        // 参照）。
        let inner = crate::progress::Progress::new(
            0.0,
            100.0,
            Some(64.0),
            crate::data_attrs::Orientation::Horizontal,
        );
        let node = progress(
            vec![],
            vec![inner.root(
                None,
                vec![],
                vec![inner.track(vec![], vec![inner.range(vec![], vec![])])],
            )],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-scope="attachment""#));
        assert!(html.contains(r#"data-part="progress""#));
        assert!(html.contains(r#"data-scope="progress""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-part="track""#));
        assert!(html.contains(r#"data-part="range""#));
    }

    #[test]
    fn actions_is_a_plain_container() {
        let html = render(&actions(
            vec![],
            vec![action("Delete", false, vec![], vec![])],
        ));
        assert!(html.contains(r#"data-part="actions""#));
        assert!(html.contains(r#"data-part="action""#));
    }

    #[test]
    fn action_has_type_button_and_optional_aria_label() {
        let without_label = render(&action("", false, vec![], vec![]));
        assert!(without_label.contains(r#"type="button""#));
        assert!(!without_label.contains("aria-label"));
        assert!(!without_label.contains("disabled"));

        let with_label = render(&action("Delete report.pdf", false, vec![], vec![]));
        assert!(with_label.contains(r#"aria-label="Delete report.pdf""#));
    }

    #[test]
    fn action_disabled_sets_native_and_data_attrs() {
        let html = render(&action("Delete", true, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));

        let enabled = render(&action("Delete", false, vec![], vec![]));
        assert!(!enabled.contains("disabled"));
    }

    #[test]
    fn action_drops_reserved_attrs_case_insensitively() {
        let html = render(&action(
            "Delete",
            false,
            vec![
                ("Type", "submit"),
                ("ARIA-LABEL", "spoofed"),
                ("Disabled", "true"),
                ("Data-Disabled", "true"),
            ],
            vec![],
        ));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains(r#"type="submit""#));
        assert!(html.contains(r#"aria-label="Delete""#));
        assert!(!html.contains("spoofed"));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn no_part_emits_hydration_attributes() {
        let html = render(&root(
            AttachmentRootProps::default(),
            vec![],
            vec![
                media(vec![], vec![]),
                content(vec![], vec![name(vec![], vec![]), meta(vec![], vec![])]),
                progress(vec![], vec![]),
                actions(vec![], vec![action("Delete", false, vec![], vec![])]),
            ],
        ));
        assert!(!html.contains("data-hydrate-"));
    }

    #[test]
    fn nesting_message_content_does_not_leak_attachment_scope() {
        // message::content スロット内へ attachment を入れ子にしても、
        // それぞれの scope が独立して固定されることを固定する（会話 1 発言
        // 内に添付ファイルを並べる想定の回帰）。
        let node = crate::message::content(
            vec![],
            vec![root(AttachmentRootProps::default(), vec![], vec![])],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-scope="message""#));
        assert!(html.contains(r#"data-scope="attachment""#));
        assert!(html.contains(r#"data-part="content""#));
        assert!(html.contains(r#"data-part="root""#));
    }
}
