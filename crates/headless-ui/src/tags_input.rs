//! TagsInput（タグ配列入力）headless コンポーネント（イシュー #744、親 #736/#726）。
//!
//! ark-ui の TagsInput
//!（`.claude/skills/ark-ui/references/components/form/tags-input.md`）を
//! 参考に、Root / Label / Control / Input / Item / ItemPreview / ItemText /
//! ItemInput / ItemDeleteTrigger / ClearTrigger / HiddenInput の 11 anatomy
//! パーツと、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する値状態機械
//! [`TagsInput`] を提供する。
//!
//! # 独自状態機械にした理由（[`crate::state`] の既存型を使わない理由）
//!
//! [`crate::state::SingleSelect`]/[`crate::state::MultiSelect`] は選択肢集合
//! からの選択（既存の候補一覧からの取捨選択）を表す語彙であり、TagsInput が
//! 持つ「タグ文字列の可変長リスト + 重複拒否 + 上限 + 編集中インデックス」と
//! いう自由入力ベースの状態を表現できない。[`crate::pin_input::PinInput`]/
//! [`crate::number_input::NumberInput`] と同じ判断（両モジュールの rustdoc
//! 参照）で、本モジュールも [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装し、Phase 1 が確立した
//! dispatch 契約（未知アクション no-op）・fail-closed hydration という
//! **統合様式**にのみ準拠する。
//!
//! # 呼び出し文脈
//!
//! SSR は [`TagsInput::new`] でタグ列・上限を指定してから各パーツメソッド
//! （[`TagsInput::root`]/[`TagsInput::label`]/[`TagsInput::control`]/
//! [`TagsInput::input`]/[`TagsInput::item`]/[`TagsInput::item_preview`]/
//! [`TagsInput::item_text`]/[`TagsInput::item_input`]/
//! [`TagsInput::item_delete_trigger`]/[`TagsInput::clear_trigger`]/
//! [`TagsInput::hidden_input`]）を呼んで組み立てる。CSR/hydration は
//! [`TagsInput`] を経由し、dispatch（`"add"`/`"remove"`/`"clear"`/
//! `"edit-start"`/`"edit-submit"`/`"edit-cancel"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui`（#744〜）が本モジュールを呼んでスタイル済み
//! TagsInput を組み立てる想定である。
//!
//! # スコープ外（イシュー #744 本文が明示）
//!
//! - `fandhe-frontend-wasm-full` での実 DOM 配線（Enter/Backspace/矢印キーの
//!   キーボード操作、delimiter によるペースト分割、blur 時の挙動、フォーカス
//!   管理）は別 issue。本モジュールは dispatch を受けた際の状態遷移のみを担う。
//! - `validate` コールバック相当の拡張検証・`maxLength`（タグ文字数上限）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`name`/`value`/`disabled`）は
//!   すべて `&'static str` リテラルで固定しており、動的値が属性名スロットへ
//!   混入する経路はない（[`crate::anatomy`]/[`crate::aria`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 動的値（各タグ文字列/`format!` で組み立てる `aria-label`/呼び出し側
//!   `attrs`/children テキスト/hidden-input の連結値）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - **タグ文字列はユーザー入力そのものである（REQ-1 の重点対象）**:
//!   `item_text` のテキストノード・`aria-label` 属性値・`hidden_input` の
//!   value のいずれも `render()` の既定エスケープを経由する経路以外を持たない
//!   （[`crate::xss_escape`] 相当の回帰テストで固定する）。
//! - **不変条件「重複タグなし・`len() <= max`・カンマを含まない・空文字列を
//!   含まない」を破る入力は一切適用しない**（fail-closed。
//!   [`TagsInputAction::Add`] の空文字列・重複（完全一致）・カンマ含有・
//!   max 到達は no-op、[`TagsInputAction::EditSubmit`] の空文字列・他タグ
//!   との重複・カンマ含有は編集を確定せず元値を維持する）。**カンマ禁止の
//!   理由（Cursor Bugbot 指摘 #744 review comment 3639762375）**:
//!   [`TagsInput::value`]/[`TagsInput::hidden_input`] はフォーム送信値として
//!   タグ列を単純にカンマ結合する（区切り文字自体をエスケープしない）。
//!   タグ文字列がカンマを含むことを許すと `["foo,bar"]` と `["foo", "bar"]`
//!   が同一のフォーム送信値（`"foo,bar"`）に縮退し、受信側で復元が一意に
//!   定まらない。この曖昧さを構造的に防ぐため、カンマを含むタグそのものを
//!   [`TagsInput::new`]/[`TagsInputAction::Add`]/[`TagsInputAction::EditSubmit`]/
//!   [`Hydrate::from_hydration_attrs`] のすべての入口で拒否する（hydration の
//!   内部搬送自体は [`fandhe_frontend_interactive::codec::encode_list`] が
//!   カンマと無関係の区切り文字で安全に行うが、復元後の値がカンマ結合値へ
//!   還元される契約のため入口で一貫して拒否する）。**空タグ拒否の理由
//!   （Cursor Bugbot 指摘 #744 review comment、BUGBOT_BUG_ID:
//!   83d9064b-d1f4-4f7c-9b06-26f3dcc21235）**: [`TagsInputAction::Add`]/
//!   [`TagsInputAction::EditSubmit`] は空文字列タグを拒否するが、
//!   [`TagsInput::new`]/[`Hydrate::from_hydration_attrs`] だけがこれを許すと
//!   空タグ列（`vec![]`）と単一の空文字列タグ（`vec![""]`）が
//!   [`TagsInput::value`]/[`TagsInput::hidden_input`] を通じて同一の `""`
//!   へ縮退し、フォーム送信値のラウンドトリップが曖昧になる（カンマ禁止と
//!   同種の衝突クラス）。この曖昧さを構造的に防ぐため、空文字列タグも
//!   [`TagsInput::new`]/[`TagsInputAction::Add`]/[`TagsInputAction::EditSubmit`]/
//!   [`Hydrate::from_hydration_attrs`] のすべての入口で一貫して拒否する。
//! - hydration 属性（`data-hydrate-tags`/`data-hydrate-max`）はクライアント
//!   側で改ざんされうる入力として扱う。[`TagsInput`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す（パース不能な `max`・復元タグ列中の重複・
//!   `tags.len() > max` をすべて拒否する）。`editing`（編集中インデックスと
//!   いう ephemeral な DOM 状態）は運ばない（[`crate::pin_input::PinInput`]
//!   の `focused` と同じ判断）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, aria_orientation, aria_selected};
use crate::data_attrs::{data_disabled, data_invalid, Orientation};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{codec, Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// TagsInput の anatomy（`data-scope="tags-input"`）。
const ANATOMY: Anatomy = anatomy("tags-input");

/// `data-editing` 存在属性。編集中のタグ item にのみ付与する
/// （TagsInput 固有の語彙であるため、ここに閉じて一元管理する。
/// [`crate::data_attrs::data_highlighted`] と同じ「存在で真を表す」規約）。
fn data_editing(editing: bool) -> Option<(&'static str, &'static str)> {
    editing.then_some(("data-editing", ""))
}

/// Root パーツ（`div`）。`data-disabled` を反映する。
#[must_use]
pub fn root<'a>(disabled: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。意味論的なラベル関連付けは呼び出し側が
/// `attrs` 経由で `for`/`id`（または labelledby）を配線する（装飾用パーツ、
/// [`crate::pin_input::label`] と同じ最小主義）。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "label", attrs, children)
}

/// Control パーツ（`div`）。タグ [`item`] 群 + [`input`] を並べるコンテナ。
/// `role="listbox"` + `aria-orientation="horizontal"` を持つ（イシュー本文が
/// 指定する listbox 相当の ARIA、`aria_label` は呼び出し側が与える）。
#[must_use]
pub fn control<'a>(
    disabled: bool,
    invalid: bool,
    label_text: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("role", "listbox"),
        aria_orientation(Orientation::Horizontal),
        aria_label(label_text),
    ];
    merged.extend(data_disabled(disabled));
    merged.extend(data_invalid(invalid));
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// Item パーツ（`div`）。タグ 1 個分のコンテナ（[`item_preview`] または
/// 編集モード時の [`item_input`] を子に持つ）。
#[must_use]
pub fn item<'a>(
    disabled: bool,
    editing: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(data_editing(editing));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemPreview パーツ（`div`）。表示モードのタグチップ本体。
/// `role="option"`（listbox 相当の ARIA、イシュー本文が指定）。[`control`] の
/// `role="listbox"` 配下に描画される item-preview はいずれも「既に追加され
/// 現に確定しているタグ」を表す（[`select::item`] 等の他 listbox 実装と同じ
/// 規約で `role="option"` には必ず `aria-selected` を対で付与する。本パーツは
/// 常に選択済みタグを表示するため `aria-selected="true"` 固定であり、
/// `highlighted`（キーボード操作上の強調・別軸）とは独立した意味論である）。
#[must_use]
pub fn item_preview<'a>(
    highlighted: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("role", "option"), aria_selected(true)];
    merged.extend(crate::data_attrs::data_highlighted(highlighted));
    merged.extend(attrs);
    ANATOMY.part("item-preview", "div", merged, children)
}

/// ItemText パーツ（`div`）。タグ文字列を表示するテキストノードのコンテナ。
/// タグ文字列は children として渡され `render()` の既定エスケープを経由する。
#[must_use]
pub fn item_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-text", "div", attrs, children)
}

/// ItemInput パーツ（`input`）。タグ編集モード時のネイティブ入力欄。
/// `value` は編集中の暫定値（動的だが `render()` の既定エスケープ経由）。
#[must_use]
pub fn item_input<'a>(value: &'a str, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "text"), ("value", value)];
    merged.extend(attrs);
    ANATOMY.part("item-input", "input", merged, Vec::new())
}

/// ItemDeleteTrigger パーツ（`button`）。当該タグを削除する操作。
/// `aria-label` は `format!` で組み立てた「Delete {tag}」（動的値だが
/// `render()` の既定エスケープを経由するため注入経路にはならない）。
/// [`clear_trigger`] 等の他 trigger パーツと同様に `children` を受け取り、
/// 呼び出し側が × アイコン・視覚ラベルを描画できる（Cursor Bugbot 指摘
/// #744 review comment 3639762362: 従来固定 `Vec::new()` だったため呼び出し側
/// が視覚的内容を一切描画できなかった）。`aria-label` は children の有無に
/// 関わらず常に付与するため、視覚的に空（アイコンフォントが読み込めない等）
/// でもスクリーンリーダーには「Delete {tag}」が伝わる。
#[must_use]
pub fn item_delete_trigger<'a>(
    tag: &str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    // aria_label は呼び出し時にのみ必要な一時 String であり、el() が
    // 即座に owned String へコピーするため関数スコープを超えて借用が
    // 残ることはない（crates/headless-ui/src/pin_input.rs の input() 参照）。
    let label_str = format!("Delete {tag}");
    let mut merged: Vec<(&str, &str)> = vec![("type", "button"), aria_label(label_str.as_str())];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-delete-trigger", "button", merged, children)
}

/// ClearTrigger パーツ（`button`）。全タグを一括削除する操作。
#[must_use]
pub fn clear_trigger<'a>(
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// Input パーツ（`input`）。新規タグ入力用のネイティブ入力欄。
/// max 到達時は `data-invalid`/`aria-invalid` を出力する。
#[must_use]
pub fn input<'a>(
    value: &'a str,
    disabled: bool,
    at_max: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("type", "text"), ("value", value)];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(data_invalid(at_max));
    if at_max {
        merged.push(("aria-invalid", "true"));
    }
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信時に全タグの
/// カンマ結合値を 1 個の値として運ぶ。
#[must_use]
pub fn hidden_input<'a>(
    name: &'a str,
    value: &'a str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "hidden"), ("name", name), ("value", value)];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// [`TagsInput`] に対する型付きアクション（WASM 境界の文字列 dispatch と
/// [`TagsInput::decode_action`] で接続する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagsInputAction {
    /// タグを追加する（空文字列・重複・max 到達は no-op）。
    Add(String),
    /// 指定インデックスのタグを削除する（範囲外は no-op）。
    Remove(usize),
    /// 全タグを削除する。
    Clear,
    /// 指定インデックスのタグを編集モードにする（範囲外は no-op）。
    EditStart(usize),
    /// 編集中のタグを新文字列で確定する（編集中でない・空文字列・他タグと
    /// 重複なら編集を破棄し元値を維持する）。
    EditSubmit(String),
    /// 編集を破棄して元値を維持したまま編集モードを終了する。
    EditCancel,
}

/// TagsInput の値状態機械。
///
/// `tags` は表示順のタグ列（不変条件: 重複なし・`len() <= max`。この
/// 不変条件は [`Self::update`]/[`Self::from_hydration_attrs`] のいずれの
/// 経路でも破られない）。`editing` は編集中インデックスという ephemeral な
/// DOM 状態であり、hydration では運ばない（モジュール doc 参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagsInput {
    tags: Vec<String>,
    max: Option<usize>,
    editing: Option<usize>,
}

impl Default for TagsInput {
    /// 既定は空タグ列・上限なし。
    fn default() -> Self {
        Self::new(Vec::new(), None)
    }
}

impl TagsInput {
    /// `data-hydrate-tags` 属性名のフィールド部分。
    pub const FIELD_TAGS: &'static str = "tags";
    /// `data-hydrate-max` 属性名のフィールド部分。
    pub const FIELD_MAX: &'static str = "max";

    /// 初期タグ列・上限（`None` = 無制限）を指定して [`TagsInput`] を生成する。
    /// 呼び出し時点で空文字列・カンマを含むタグは除外し（[`TagsInputAction::Add`]/
    /// [`TagsInputAction::EditSubmit`] と同じ拒否基準。モジュール doc
    /// 「カンマ禁止の理由」節参照。空タグを許すと空リストと単一の空文字列
    /// タグが [`Self::value`]/[`Self::hidden_input`] を通じて同一の `""` へ
    /// 縮退し、フォーム送信値が曖昧になるため一貫して拒否する）、残った列の
    /// 重複タグは先頭から見て初出のみを残し後続の重複を落とす（不変条件を
    /// 最初から保証する、panic しない）。
    #[must_use]
    pub fn new(tags: Vec<String>, max: Option<usize>) -> Self {
        let mut deduped: Vec<String> = Vec::with_capacity(tags.len());
        for t in tags {
            if !t.is_empty() && !t.contains(',') && !deduped.contains(&t) {
                deduped.push(t);
            }
        }
        if let Some(m) = max {
            deduped.truncate(m);
        }
        Self {
            tags: deduped,
            max,
            editing: None,
        }
    }

    /// 現在のタグ列（表示順）。
    #[must_use]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// タグ数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    /// タグが 1 個もないか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// 上限（`None` = 無制限）。
    #[must_use]
    pub fn max(&self) -> Option<usize> {
        self.max
    }

    /// 上限に到達しているか（`max` が `None` の場合は常に `false`）。
    #[must_use]
    pub fn is_at_max(&self) -> bool {
        self.max.is_some_and(|m| self.tags.len() >= m)
    }

    /// 指定文字列を含むか（完全一致）。
    #[must_use]
    pub fn contains(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// 指定インデックスが現在編集中か。
    #[must_use]
    pub fn is_editing(&self, index: usize) -> bool {
        self.editing == Some(index)
    }

    /// 現在編集中のインデックス（未編集なら `None`）。
    #[must_use]
    pub fn editing_index(&self) -> Option<usize> {
        self.editing
    }

    /// 全タグをカンマ結合した値（フォーム送信・[`Self::hidden_input`] が使う）。
    #[must_use]
    pub fn value(&self) -> String {
        self.tags.join(",")
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(disabled, attrs, children)
    }

    /// [`label`] へ委譲する利便メソッド（状態を持たないため素通し）。
    #[must_use]
    pub fn label<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        label(attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        disabled: bool,
        label_text: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        control(disabled, self.is_at_max(), label_text, attrs, children)
    }

    /// [`item`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn item<'a>(
        &self,
        index: usize,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item(disabled, self.is_editing(index), attrs, children)
    }

    /// [`item_preview`] へ委譲する利便メソッド。
    #[must_use]
    pub fn item_preview<'a>(
        &self,
        highlighted: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_preview(highlighted, attrs, children)
    }

    /// [`item_text`] へ委譲する利便メソッド。
    #[must_use]
    pub fn item_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        item_text(attrs, children)
    }

    /// [`item_input`] へ委譲する利便メソッド。
    #[must_use]
    pub fn item_input<'a>(&self, value: &'a str, attrs: Vec<(&'a str, &'a str)>) -> Node {
        item_input(value, attrs)
    }

    /// [`item_delete_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn item_delete_trigger<'a>(
        &self,
        tag: &str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        item_delete_trigger(tag, disabled, attrs, children)
    }

    /// [`clear_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn clear_trigger<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        clear_trigger(disabled, attrs, children)
    }

    /// [`input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn input<'a>(
        &self,
        value: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        input(value, disabled, self.is_at_max(), attrs)
    }

    /// [`hidden_input`] へ現在の連結値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        name: &'a str,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let value = self.value();
        hidden_input(name, &value, disabled, attrs)
    }
}

impl Component for TagsInput {
    type Action = TagsInputAction;

    fn update(&mut self, action: TagsInputAction) {
        match action {
            TagsInputAction::Add(tag) => {
                // 空文字列・カンマ含有・重複（完全一致）・max 到達は no-op
                // （fail-closed、モジュール doc「カンマ禁止の理由」節参照）。
                if tag.is_empty() || tag.contains(',') || self.contains(&tag) || self.is_at_max() {
                    return;
                }
                self.tags.push(tag);
            }
            TagsInputAction::Remove(idx) => {
                if idx >= self.tags.len() {
                    return;
                }
                self.tags.remove(idx);
                // 削除により編集中インデックスの対象がずれる場合は編集を
                // 終了する（存在しない/別タグを指す編集状態を残さない）。
                if self.editing == Some(idx) {
                    self.editing = None;
                } else if let Some(e) = self.editing {
                    if e > idx {
                        self.editing = Some(e - 1);
                    }
                }
            }
            TagsInputAction::Clear => {
                self.tags.clear();
                self.editing = None;
            }
            TagsInputAction::EditStart(idx) => {
                if idx < self.tags.len() {
                    self.editing = Some(idx);
                }
            }
            TagsInputAction::EditSubmit(new_value) => {
                let Some(idx) = self.editing else { return };
                if idx >= self.tags.len() {
                    self.editing = None;
                    return;
                }
                // 空文字列・カンマ含有・他タグとの重複は編集を破棄し元値を
                // 維持する（fail-closed、部分適用しない。モジュール doc
                // 「カンマ禁止の理由」節参照）。
                let duplicates_other = self
                    .tags
                    .iter()
                    .enumerate()
                    .any(|(i, t)| i != idx && t == &new_value);
                if !new_value.is_empty() && !new_value.contains(',') && !duplicates_other {
                    self.tags[idx] = new_value;
                }
                self.editing = None;
            }
            TagsInputAction::EditCancel => {
                self.editing = None;
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー
    /// （root > control > (item > item-preview > item-text) × len + input）。
    /// 公開 UI としての利用は想定しない（実際の UI 構築は各パーツメソッドを
    /// 呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let items: Vec<Node> = self
            .tags
            .iter()
            .enumerate()
            .map(|(i, tag)| {
                let preview = self.item_preview(
                    false,
                    Vec::new(),
                    vec![self.item_text(Vec::new(), vec![fandhe_frontend_core::text(tag)])],
                );
                self.item(i, false, Vec::new(), vec![preview])
            })
            .collect();
        let mut control_children = items;
        control_children.push(self.input("", false, Vec::new()));
        self.root(
            false,
            Vec::new(),
            vec![self.control(false, "Tags", Vec::new(), control_children)],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<TagsInputAction> {
        match name {
            "add" => Some(TagsInputAction::Add(payload.to_string())),
            "remove" => payload.parse::<usize>().ok().map(TagsInputAction::Remove),
            "clear" => Some(TagsInputAction::Clear),
            "edit-start" => payload
                .parse::<usize>()
                .ok()
                .map(TagsInputAction::EditStart),
            "edit-submit" => Some(TagsInputAction::EditSubmit(payload.to_string())),
            "edit-cancel" => Some(TagsInputAction::EditCancel),
            _ => None,
        }
    }
}

impl Hydrate for TagsInput {
    /// [`codec::encode_list`] でタグ列を運ぶ（[`crate::pin_input::PinInput`]
    /// の `values` と同型）。`max` は `"none"` または非負整数文字列。`editing`
    /// は ephemeral のため運ばない（モジュール doc 参照）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_TAGS),
                codec::encode_list(&self.tags),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX),
                self.max
                    .map_or_else(|| "none".to_string(), |m| m.to_string()),
            ),
        ]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name.clone()))
        };

        let max_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX);
        let max_raw = find(Self::FIELD_MAX)?;
        let max: Option<usize> = if max_raw == "none" {
            None
        } else {
            Some(max_raw.parse().map_err(|_| HydrateError::InvalidValue {
                attr: max_attr.clone(),
                reason: "expected \"none\" or a non-negative integer".to_string(),
            })?)
        };

        let tags_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_TAGS);
        let tags = codec::decode_list(find(Self::FIELD_TAGS)?);

        // 復元タグ列が不変条件（重複なし・len <= max・カンマを含まない・
        // 空文字列を含まない）を満たすことを検証する。改ざんされた data-*
        // によって不変条件を破った状態を復元しない（fail-closed、モジュール
        // doc「カンマ禁止の理由」節参照。空タグの拒否は [`Self::new`]/
        // [`TagsInputAction::Add`]/[`TagsInputAction::EditSubmit`] と同じ
        // 基準であり、空リストと単一の空文字列タグが `value()`/
        // `hidden_input` を通じて同一の `""` へ縮退する曖昧さをこの経路でも
        // 一貫して排除する）。
        for (i, t) in tags.iter().enumerate() {
            if t.is_empty() {
                return Err(HydrateError::InvalidValue {
                    attr: tags_attr,
                    reason: "tags must not contain an empty string".to_string(),
                });
            }
            if t.contains(',') {
                return Err(HydrateError::InvalidValue {
                    attr: tags_attr,
                    reason: "tags must not contain a comma".to_string(),
                });
            }
            if tags[..i].contains(t) {
                return Err(HydrateError::InvalidValue {
                    attr: tags_attr,
                    reason: "tags must not contain duplicates".to_string(),
                });
            }
        }
        if let Some(m) = max {
            if tags.len() > m {
                return Err(HydrateError::InvalidValue {
                    attr: tags_attr,
                    reason: "tags length exceeds max".to_string(),
                });
            }
        }

        Ok(Self {
            tags,
            max,
            // 編集中インデックスは ephemeral な DOM 状態のため運ばない
            // （モジュール doc 参照）。復元直後は常に未設定。
            editing: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn tags(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    // --- 各パーツの data-scope/data-part/data-disabled 出力 ---

    #[test]
    fn root_outputs_scope_part_and_no_state_when_enabled() {
        let html = render(&root(false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_disabled_true_outputs_data_disabled() {
        let html = render(&root(true, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn label_outputs_scope_and_part() {
        let html = render(&label(vec![], vec![text("Tags")]));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains("Tags"));
    }

    #[test]
    fn control_outputs_listbox_role_and_orientation() {
        let html = render(&control(false, false, "Selected tags", vec![], vec![]));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
        assert!(html.contains(r#"aria-label="Selected tags""#));
        assert!(!html.contains("data-invalid"));
    }

    #[test]
    fn control_invalid_true_outputs_data_invalid() {
        let html = render(&control(false, true, "Tags", vec![], vec![]));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn item_outputs_scope_part_and_editing_state() {
        let html = render(&item(false, true, vec![], vec![]));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"data-editing="""#));
    }

    #[test]
    fn item_not_editing_omits_data_editing() {
        let html = render(&item(false, false, vec![], vec![]));
        assert!(!html.contains("data-editing"));
    }

    #[test]
    fn item_preview_outputs_option_role() {
        let html = render(&item_preview(false, vec![], vec![]));
        assert!(html.contains(r#"data-part="item-preview""#));
        assert!(html.contains(r#"role="option""#));
    }

    #[test]
    fn item_preview_always_outputs_aria_selected_true() {
        // ItemPreview は listbox 配下の「既に追加されたタグ」を表すため
        // `highlighted` の真偽に関わらず常に aria-selected="true" を伴う
        // （`role="option"` と対で必須、Cursor Bugbot 指摘 #744 review comment
        // 3639870269）。
        let unhighlighted = render(&item_preview(false, vec![], vec![]));
        assert!(unhighlighted.contains(r#"aria-selected="true""#));
        let highlighted = render(&item_preview(true, vec![], vec![]));
        assert!(highlighted.contains(r#"aria-selected="true""#));
    }

    #[test]
    fn item_preview_highlighted_outputs_data_highlighted() {
        let html = render(&item_preview(true, vec![], vec![]));
        assert!(html.contains(r#"data-highlighted="""#));
    }

    #[test]
    fn item_text_outputs_scope_and_part_with_tag_text() {
        let html = render(&item_text(vec![], vec![text("rust")]));
        assert!(html.contains(r#"data-part="item-text""#));
        assert!(html.contains("rust"));
    }

    #[test]
    fn item_input_outputs_type_text_and_value() {
        let html = render(&item_input("editing-value", vec![]));
        assert!(html.contains(r#"data-part="item-input""#));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"value="editing-value""#));
    }

    #[test]
    fn item_delete_trigger_outputs_type_button_and_aria_label() {
        let html = render(&item_delete_trigger("rust", false, vec![], vec![]));
        assert!(html.contains(r#"data-part="item-delete-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Delete rust""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn item_delete_trigger_disabled_outputs_native_disabled() {
        let html = render(&item_delete_trigger("rust", true, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn clear_trigger_outputs_type_button() {
        let html = render(&clear_trigger(false, vec![], vec![text("Clear")]));
        assert!(html.contains(r#"data-part="clear-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains("Clear"));
    }

    #[test]
    fn input_outputs_type_text_and_value() {
        let html = render(&input("draft", false, false, vec![]));
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"value="draft""#));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("aria-invalid"));
    }

    #[test]
    fn input_at_max_outputs_data_invalid_and_aria_invalid() {
        let html = render(&input("", false, true, vec![]));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"aria-invalid="true""#));
    }

    #[test]
    fn hidden_input_outputs_type_hidden_name_and_value() {
        let html = render(&hidden_input("tags", "rust,wasm", false, vec![]));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="tags""#));
        assert!(html.contains(r#"value="rust,wasm""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn hidden_input_disabled_outputs_native_disabled_and_data_disabled() {
        let html = render(&hidden_input("tags", "", true, vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- TagsInput: 状態機械 ---

    #[test]
    fn default_is_empty_and_unlimited() {
        let t = TagsInput::default();
        assert_eq!(t.len(), 0);
        assert!(t.is_empty());
        assert_eq!(t.max(), None);
        assert!(!t.is_at_max());
        assert_eq!(t.value(), "");
    }

    #[test]
    fn new_deduplicates_initial_tags_keeping_first_occurrence() {
        let t = TagsInput::new(tags(&["a", "b", "a"]), None);
        assert_eq!(t.tags(), &tags(&["a", "b"]));
    }

    #[test]
    fn new_truncates_initial_tags_to_max() {
        let t = TagsInput::new(tags(&["a", "b", "c"]), Some(2));
        assert_eq!(t.tags(), &tags(&["a", "b"]));
    }

    #[test]
    fn new_drops_initial_tags_containing_comma() {
        // カンマを含むタグは value()/hidden_input のカンマ結合値を曖昧にする
        // ため、構築時点で除外する（モジュール doc「カンマ禁止の理由」節、
        // Cursor Bugbot 指摘 #744 review comment 3639762375）。
        let t = TagsInput::new(tags(&["a", "b,c", "d"]), None);
        assert_eq!(t.tags(), &tags(&["a", "d"]));
    }

    #[test]
    fn new_drops_empty_string_initial_tags() {
        // 空文字列タグを許すと空タグ列と単一の空文字列タグが value()/
        // hidden_input を通じて同一の "" へ縮退し、フォーム送信値の
        // ラウンドトリップが曖昧になる（モジュール doc「空タグ拒否の理由」
        // 節、Cursor Bugbot 指摘 #744、BUGBOT_BUG_ID:
        // 83d9064b-d1f4-4f7c-9b06-26f3dcc21235）。Add/EditSubmit と同じ基準を
        // コンストラクタでも一貫適用することを固定する。
        let t = TagsInput::new(tags(&["a", "", "d"]), None);
        assert_eq!(t.tags(), &tags(&["a", "d"]));
    }

    #[test]
    fn new_with_only_empty_string_tag_does_not_serialize_as_ambiguous_empty_value() {
        // 単一の空文字列タグだけを渡した場合、空タグ列と value() が区別
        // できなくなる縮退を防ぐ（BUGBOT_BUG_ID:
        // 83d9064b-d1f4-4f7c-9b06-26f3dcc21235）。
        let t = TagsInput::new(tags(&[""]), None);
        assert!(t.is_empty());
        assert_eq!(t.value(), "");
    }

    #[test]
    fn add_action_appends_new_tag() {
        let mut t = TagsInput::default();
        assert!(dispatch(&mut t, "add", "rust"));
        assert_eq!(t.tags(), &tags(&["rust"]));
        assert!(dispatch(&mut t, "add", "wasm"));
        assert_eq!(t.tags(), &tags(&["rust", "wasm"]));
    }

    #[test]
    fn add_action_rejects_empty_string_as_no_op() {
        let mut t = TagsInput::default();
        assert!(dispatch(&mut t, "add", ""));
        assert!(t.is_empty());
    }

    #[test]
    fn add_action_rejects_duplicate_as_no_op() {
        let mut t = TagsInput::new(tags(&["rust"]), None);
        assert!(dispatch(&mut t, "add", "rust"));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn add_action_rejects_when_at_max() {
        let mut t = TagsInput::new(tags(&["a", "b"]), Some(2));
        assert!(t.is_at_max());
        assert!(dispatch(&mut t, "add", "c"));
        assert_eq!(t.tags(), &tags(&["a", "b"]));
    }

    #[test]
    fn add_action_rejects_tag_containing_comma_as_no_op() {
        // カンマを含むタグを許すと value()/hidden_input のカンマ結合値が
        // 曖昧になる（モジュール doc「カンマ禁止の理由」節、Cursor Bugbot
        // 指摘 #744 review comment 3639762375）。
        let mut t = TagsInput::default();
        assert!(dispatch(&mut t, "add", "foo,bar"));
        assert!(t.is_empty());
    }

    #[test]
    fn remove_action_removes_by_index() {
        let mut t = TagsInput::new(tags(&["a", "b", "c"]), None);
        assert!(dispatch(&mut t, "remove", "1"));
        assert_eq!(t.tags(), &tags(&["a", "c"]));
    }

    #[test]
    fn remove_action_out_of_range_is_no_op() {
        let mut t = TagsInput::new(tags(&["a"]), None);
        assert!(dispatch(&mut t, "remove", "5"));
        assert_eq!(t.tags(), &tags(&["a"]));
    }

    #[test]
    fn clear_action_removes_all_tags() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        assert!(dispatch(&mut t, "clear", ""));
        assert!(t.is_empty());
    }

    #[test]
    fn edit_start_then_submit_updates_tag() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        assert!(dispatch(&mut t, "edit-start", "0"));
        assert!(t.is_editing(0));
        assert!(dispatch(&mut t, "edit-submit", "z"));
        assert_eq!(t.tags(), &tags(&["z", "b"]));
        assert_eq!(t.editing_index(), None);
    }

    #[test]
    fn edit_start_then_cancel_keeps_original_value() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        assert!(dispatch(&mut t, "edit-start", "0"));
        assert!(dispatch(&mut t, "edit-cancel", ""));
        assert_eq!(t.tags(), &tags(&["a", "b"]));
        assert_eq!(t.editing_index(), None);
    }

    #[test]
    fn edit_submit_with_duplicate_discards_edit_and_keeps_original() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        assert!(dispatch(&mut t, "edit-start", "0"));
        assert!(dispatch(&mut t, "edit-submit", "b"));
        assert_eq!(t.tags(), &tags(&["a", "b"]));
        assert_eq!(t.editing_index(), None);
    }

    #[test]
    fn edit_submit_with_empty_string_discards_edit_and_keeps_original() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        assert!(dispatch(&mut t, "edit-start", "0"));
        assert!(dispatch(&mut t, "edit-submit", ""));
        assert_eq!(t.tags(), &tags(&["a", "b"]));
    }

    #[test]
    fn edit_submit_with_comma_discards_edit_and_keeps_original() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        assert!(dispatch(&mut t, "edit-start", "0"));
        assert!(dispatch(&mut t, "edit-submit", "x,y"));
        assert_eq!(t.tags(), &tags(&["a", "b"]));
        assert_eq!(t.editing_index(), None);
    }

    #[test]
    fn edit_start_out_of_range_is_no_op() {
        let mut t = TagsInput::new(tags(&["a"]), None);
        assert!(dispatch(&mut t, "edit-start", "9"));
        assert_eq!(t.editing_index(), None);
    }

    #[test]
    fn edit_submit_without_edit_start_is_no_op() {
        let mut t = TagsInput::new(tags(&["a"]), None);
        assert!(dispatch(&mut t, "edit-submit", "z"));
        assert_eq!(t.tags(), &tags(&["a"]));
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut t = TagsInput::new(tags(&["a"]), None);
        assert!(!dispatch(&mut t, "no_such_action", "x"));
        assert_eq!(t.tags(), &tags(&["a"]));
    }

    #[test]
    fn remove_shifts_editing_index_when_removing_earlier_tag() {
        let mut t = TagsInput::new(tags(&["a", "b", "c"]), None);
        dispatch(&mut t, "edit-start", "2");
        assert!(dispatch(&mut t, "remove", "0"));
        assert_eq!(t.editing_index(), Some(1));
    }

    #[test]
    fn remove_clears_editing_index_when_removing_edited_tag() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "edit-start", "0");
        assert!(dispatch(&mut t, "remove", "0"));
        assert_eq!(t.editing_index(), None);
    }

    // --- TagsInput: SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&TagsInput::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- TagsInput: hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let mut t = TagsInput::new(Vec::new(), Some(5));
        dispatch(&mut t, "add", "rust");
        dispatch(&mut t, "add", "wasm");
        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains(r#"data-hydrate-max="5""#));

        let restored = TagsInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.tags(), t.tags());
        assert_eq!(restored.max(), t.max());
        assert_eq!(restored.editing_index(), None);
    }

    #[test]
    fn hydration_round_trip_unlimited_max_encodes_none() {
        let t = TagsInput::new(tags(&["a"]), None);
        let attrs = t.hydration_attrs();
        assert!(attrs.iter().any(|(k, v)| k.ends_with("max") && v == "none"));
        let restored = TagsInput::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored.max(), None);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = TagsInput::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-max".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_invalid_max_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-max".to_string(), "not-a-number".to_string()),
            ("data-hydrate-tags".to_string(), String::new()),
        ];
        let err = TagsInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_duplicate_tags_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-max".to_string(), "none".to_string()),
            (
                "data-hydrate-tags".to_string(),
                codec::encode_list(&tags(&["a", "a"])),
            ),
        ];
        let err = TagsInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_tag_containing_comma_does_not_panic() {
        // 改ざんされた data-hydrate-tags がカンマを含むタグを運んできても、
        // value()/hidden_input のカンマ結合値が曖昧になる復元を許さない
        // （fail-closed、モジュール doc「カンマ禁止の理由」節、Cursor Bugbot
        // 指摘 #744 review comment 3639762375）。
        let attrs = vec![
            ("data-hydrate-max".to_string(), "none".to_string()),
            (
                "data-hydrate-tags".to_string(),
                codec::encode_list(&tags(&["a,b"])),
            ),
        ];
        let err = TagsInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_empty_string_tag_does_not_panic() {
        // 改ざんされた data-hydrate-tags が空文字列タグを運んできても、
        // value()/hidden_input を通じて空タグ列と縮退して区別できなくなる
        // 復元を許さない（fail-closed、モジュール doc「空タグ拒否の理由」節、
        // Cursor Bugbot 指摘 #744、BUGBOT_BUG_ID:
        // 83d9064b-d1f4-4f7c-9b06-26f3dcc21235）。
        let attrs = vec![
            ("data-hydrate-max".to_string(), "none".to_string()),
            (
                "data-hydrate-tags".to_string(),
                codec::encode_list(&tags(&["a", ""])),
            ),
        ];
        let err = TagsInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_length_exceeds_max_does_not_panic() {
        let attrs = vec![
            ("data-hydrate-max".to_string(), "1".to_string()),
            (
                "data-hydrate-tags".to_string(),
                codec::encode_list(&tags(&["a", "b"])),
            ),
        ];
        let err = TagsInput::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn hydration_does_not_carry_editing_state() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "edit-start", "1");
        let restored = TagsInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.editing_index(), None);
    }

    // --- XSS 回帰: タグ文字列/name/value/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
    const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

    #[test]
    fn item_text_tag_payload_is_escaped_on_render() {
        let html = render(&item_text(vec![], vec![text(SCRIPT_PAYLOAD)]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn item_delete_trigger_aria_label_tag_payload_is_escaped_on_render() {
        let html = render(&item_delete_trigger(
            ATTR_BREAK_PAYLOAD,
            false,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn hidden_input_value_payload_is_escaped_on_render() {
        let html = render(&hidden_input("tags", ATTR_BREAK_PAYLOAD, false, vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            false,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn add_action_with_script_payload_then_render_escapes_tag() {
        let mut t = TagsInput::default();
        dispatch(&mut t, "add", SCRIPT_PAYLOAD);
        let tag = &t.tags()[0];
        let html = render(&item_text(vec![], vec![text(tag)]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_tampered_tags_with_script_payload_round_trips_but_escapes_on_render() {
        // hydration 自体はタグの内容を検証しない（重複・長さ超過のみ検証）
        // ため、スクリプト断片を含むタグは復元されるが、描画経路で必ず
        // エスケープされることを固定する（REQ-1 は render() 側の責務）。
        let attrs = vec![
            ("data-hydrate-max".to_string(), "none".to_string()),
            (
                "data-hydrate-tags".to_string(),
                codec::encode_list(&[SCRIPT_PAYLOAD.to_string()]),
            ),
        ];
        let restored = TagsInput::from_hydration_attrs(&attrs).unwrap();
        let html = render(&item_text(vec![], vec![text(&restored.tags()[0])]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
