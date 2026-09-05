//! TagsInput（タグ配列入力）headless コンポーネント（イシュー #744、親 #736/#726）。
//!
//! ark-ui の TagsInput
//!（`.claude/skills/ark-ui/references/components/form/tags-input.md`）を
//! 参考に、Root / Label / Control / Input / Item / ItemPreview / ItemText /
//! ItemInput / ItemDeleteTrigger / ClearTrigger / HiddenInput / LiveRegion
//! の 12 anatomy パーツと、[`fandhe_frontend_interactive::Component`]/
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
//! [`TagsInput::hidden_input`]/[`TagsInput::live_region`]）を呼んで組み立てる。
//! CSR/hydration は [`TagsInput`] を経由し、dispatch（`"add"`/`"remove"`/
//! `"clear"`/`"edit-start"`/`"edit-submit"`/`"edit-cancel"`/`"highlight-prev"`/
//! `"highlight-next"`/`"highlight-clear"`/`"delete-highlighted"`/
//! `"backspace"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`（#744〜）が
//! 本モジュールを呼んでスタイル済み TagsInput を組み立てる想定である。
//!
//! # 参照突合（イシュー #1623）
//!
//! Zag.js `tags-input.connect.ts`（ark-ui 基盤）・ark-ui 公式 Data
//! Attributes / Keyboard Support 表と突合し、以下を是正した:
//!
//! - [`TagsInputProps`]（`disabled`/`readonly`/`invalid`/`required`）を
//!   新設し、[`root`]/[`label`]/[`control`]/[`input`]/[`clear_trigger`]/
//!   [`hidden_input`] へ `data-disabled`/`data-invalid`/`data-readonly`
//!   （[`label`] にはさらに `data-required`）を反映した（旧実装は
//!   `data-disabled` のみ）。[`input`] はネイティブ `readonly` も追加し、
//!   `autocomplete="off"`/`autocorrect="off"`/`autocapitalize="none"`/
//!   `enterkeyhint="done"` を固定付与した（[`crate::number_input`]（#1613）
//!   と同種の固定属性）。
//! - タグ 1 個分の状態束 [`TagItem`]（`value`/`disabled`/`editing`/
//!   `highlighted`）を新設し、[`item`]/[`item_preview`]/[`item_text`]/
//!   [`item_input`]/[`item_delete_trigger`] の引数を統一した。
//!   [`item`] に `data-value`（タグ文字列）を追加し、[`item_preview`] に
//!   `value`/`disabled`（旧実装は `highlighted` のみ）を追加し、
//!   [`item_text`] に `disabled`/`highlighted` を追加し、[`item_input`] に
//!   `hidden`（非編集時）/`disabled` を追加し、[`item_delete_trigger`] に
//!   `highlighted` を追加して `aria-label` を zag 既定訳
//!   `"Delete tag {value}"` へ揃えた（旧実装は `"Delete {tag}"`）。
//!   当初 [`item_input`]/[`item_delete_trigger`] へ ark-ui 公式表どおりの
//!   `tabindex="-1"` も固定付与したが、対応する矢印キー操作・編集開始時の
//!   フォーカス配線（DOM 配線、下記スコープ外節）が未実装のまま固定すると
//!   従来 Tab で到達できていた削除ボタン・編集欄がキーボード操作不能に
//!   なる回帰であったため撤回した（codex-review 指摘、#1623）。DOM 配線が
//!   実装されるまでは `tabindex` を固定付与せず既定の Tab 到達性を
//!   維持する。
//! - **`role="listbox"`/`aria-orientation`（[`control`]）と
//!   `role="option"`/`aria-selected`（[`item_preview`]）を撤去した**（元は
//!   #744 本文の指定で付与していたが、zag/ark の control・item-preview は
//!   いずれも `role` を持たない。`role="listbox"` の許容子は
//!   `option`/`group` のみだが `control` は `<input type="text">` を
//!   内包しており content model 違反であり、`item_preview` へ
//!   `aria-selected="true"` を固定付与するのも「選択」意味論の誤用
//!   だった）。アクセシブルネームは [`label`] の `for`（呼び出し側 `attrs`）
//!   → [`input`] の関連付けで担う（ark の `htmlFor` と同型）。これに伴い
//!   [`control`] は `aria_label` 引数を廃した。
//! - 状態機械へ `highlighted: Option<usize>`（ephemeral、hydration では
//!   運ばない）を追加し、[`TagsInputAction::HighlightPrev`]/
//!   [`TagsInputAction::HighlightNext`]/[`TagsInputAction::ClearHighlight`]/
//!   [`TagsInputAction::DeleteHighlighted`]/[`TagsInputAction::Backspace`]
//!   （dispatch トークン `"highlight-prev"`/`"highlight-next"`/
//!   `"highlight-clear"`/`"delete-highlighted"`/`"backspace"`）を新設した
//!   （ark-ui Keyboard Support 表の ArrowLeft/ArrowRight/Escape/Delete/
//!   Backspace 相当）。**DOM 配線（keydown ハンドラ・caret 位置判定）は
//!   `fandhe-frontend-wasm-full` の別イシュー（下記スコープ外節）であり、
//!   本モジュールは dispatch を受けた際の状態遷移のみを提供する契約**
//!   （キー入力を検出して自動的に dispatch する配線は未実装）。
//!
//! 意図的に合わせなかった点（`docs/policy/intentional-non-adoption.md`
//! §3.25 規則 2: 装飾・レイアウト計測を headless へ持ち込まない、または
//! リポ内一貫性を優先した判断）:
//!
//! - `data-focus`（root/control、DOM フォーカスという ephemeral な状態）は
//!   SSR では決定できないため不採用。
//! - `data-empty`（root/input の「タグ 0 件」）は、自由関数 `root` がタグ
//!   列を保持しないため実装せず先送りする（[`TagsInput::root`] 経由でのみ
//!   将来追加可能。現時点では未実装、別 issue 候補）。
//! - `readonly` の DOM 表現は zag（`control` の `tabindex="0"` 維持 +
//!   `input` へネイティブ `disabled`）と異なり、`input` へネイティブ
//!   `readonly` を採用する（`crate::pin_input`/`crate::password_input`/
//!   `crate::file_upload` の直近精査（#1615/#1614/#1609）と同じ規約への
//!   リポ内一貫性を優先した判断）。
//! - `maxLength`（タグ文字数上限）・`validate` コールバック・`delimiter`/
//!   `addOnPaste`/`blurBehavior`/`allowOverflow`/`autoFocus`/`dir` は
//!   アプリロジック・クライアント配線の関心（下記スコープ外節、
//!   `docs/policy/intentional-non-adoption.md` §3.25）。
//! - [`clear_trigger`] の `aria-label` 固定付与・空時 `hidden` は行わない
//!   （children を持たない場合の既定文言をフレームワーク側で決め打ちしない
//!   最小主義、「空なら描画しない」判断は呼び出し側に委ねる）。
//!
//! # スコープ外（イシュー #744/#1623 が明示）
//!
//! - `fandhe-frontend-wasm-full` での実 DOM 配線（ArrowLeft/ArrowRight/
//!   Backspace/Delete/Enter/Escape/Ctrl+V の keydown/paste ハンドラ、
//!   delimiter によるペースト分割、blur 時の挙動、フォーカス管理）は
//!   別 issue。本モジュールは dispatch を受けた際の状態遷移のみを担う。
//! - `validate` コールバック相当の拡張検証・`maxLength`（タグ文字数上限）。
//!
//! # LiveRegion パーツと配置制約（イシュー #1069・#1623 で緩和）
//!
//! [`live_region`] はタグ追加・削除というタグ数の変化を支援技術へ通知する
//! ための live region（`role="status"` + `aria-live="polite"` +
//! `aria-atomic="true"` 固定、[`crate::toast::root`] と同じ 3 点セット）。
//! 緊急度は常に `polite` 固定で引数を取らない（安全側の判断、
//! [`crate::combobox::live_region`] と同じ設計）。配置は [`root`] 直下を
//! 推奨する（当初は [`control`] の `role="listbox"` 配下との子ロール衝突を
//! 避けるため「[`control`] の兄弟」に限定していたが、#1623 で
//! `role="listbox"` を撤去したためこの制約は解消した）。
//! [`crate::visually_hidden::root`] への委譲はせず、視覚的に隠す CSS は
//! 呼び出し側または `fandhe-frontend-pre-styled-ui` の責務とする。通知
//! 文言は `children` として呼び出し側が渡し、タグ数整形ヘルパは提供しない
//! （`docs/policy/intentional-non-adoption.md` §3.23/§3.25）。テキスト更新の
//! 実配線（DOM 書き換え）は `fandhe-frontend-wasm-full` の後続責務
//! （#1071 系）であり、本モジュールは SSR 静的マークアップと初期文言の
//! 描画のみを提供する。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`name`/`value`/`disabled`/
//!   `readonly`/`tabindex`/`hidden`/`autocomplete`/`autocorrect`/
//!   `autocapitalize`/`enterkeyhint`）はすべて `&'static str` リテラルで
//!   固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存
//!   不変条件をそのまま継承する）。
//! - 動的値（各タグ文字列/`format!` で組み立てる `aria-label`/呼び出し側
//!   `attrs`/children テキスト/hidden-input の連結値/`data-value`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - **タグ文字列はユーザー入力そのものである（REQ-1 の重点対象）**:
//!   `item_text` のテキストノード・`item`/`item_preview` の `data-value`
//!   属性値・`aria-label` 属性値・`hidden_input` の value のいずれも
//!   `render()` の既定エスケープを経由する経路以外を持たない
//!   （[`crate::xss_escape`] 相当の回帰テストで固定する）。
//! - **呼び出し側 `attrs` によるフレームワーク固定キーの偽装は
//!   [`drop_reserved`] が fail-closed に除外する**（`data-disabled` 等を
//!   なりすまし付与できない）。
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
//!   `tags.len() > max` をすべて拒否する）。`editing`/`highlighted`
//!   （編集中インデックス・キーボード強調インデックスという ephemeral な
//!   DOM 状態）はいずれも運ばない（[`crate::pin_input::PinInput`] の
//!   `focused` と同じ判断）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::aria_label;
use crate::data_attrs::{
    data_disabled, data_highlighted, data_invalid, data_readonly, data_required,
};
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

/// TagsInput の disabled/invalid/readonly/required 状態束（ark-ui/zag
/// Data Attributes 表との突合、イシュー #1623）。root/label/control/input/
/// clear-trigger/hidden-input の全パーツへ [`data_disabled`]/
/// [`data_invalid`]/[`data_readonly`] を一律付与し、[`label`] にのみ
/// [`data_required`] を追加で付与するために使う
/// （[`crate::pin_input::PinInputProps`] と同型のパターン）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TagsInputProps {
    /// 無効化状態。`true` で `data-disabled` を各パーツへ付与し、
    /// [`input`]/[`clear_trigger`]/[`hidden_input`] にはネイティブ
    /// `disabled` も付与する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly` を各パーツへ、
    /// [`input`] にはネイティブ `readonly` を付与する（`type="hidden"` の
    /// [`hidden_input`] には効果がないため付けない、
    /// [`crate::pin_input::hidden_input`] と同じ判断。モジュール doc
    /// 「意図的に合わせなかった点」節参照）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で `data-invalid` を各パーツへ、
    /// [`input`] には追加で `aria-invalid="true"` を付与する（valid の
    /// ときは `aria-invalid` 属性自体を省略する、[`crate::field`] と同型）。
    pub invalid: bool,
    /// 入力必須状態。`true` で [`label`] に `data-required` を付与する
    /// （`type="hidden"` の [`hidden_input`] は制約検証対象外のため
    /// `required` ネイティブ属性は付けない）。
    pub required: bool,
}

/// [`TagsInputProps`] から共通の状態属性列を組み立てる非公開ヘルパ
/// （disabled/invalid/readonly の 3 属性）。
fn state_attrs(props: &TagsInputProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// [`root`]/[`control`] が固定付与するキー一覧
/// （[`crate::pin_input::ROOT_RESERVED`] と同型のパターン）。
const ROOT_RESERVED: &[&str] = &["data-disabled", "data-invalid", "data-readonly"];

/// [`label`] が固定付与するキー一覧（[`ROOT_RESERVED`] に `data-required`
/// を加えたもの）。
const LABEL_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-required",
];

/// [`input`] が固定付与するキー一覧（[`ROOT_RESERVED`] に `data-empty`/
/// `aria-invalid` を加えたもの）。
const INPUT_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-empty",
    "aria-invalid",
];

/// [`item`]/[`item_preview`]/[`item_text`]/[`item_input`]/
/// [`item_delete_trigger`] が固定付与するキー一覧。
const ITEM_RESERVED: &[&str] = &[
    "data-disabled",
    "data-editing",
    "data-value",
    "data-highlighted",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::pin_input::drop_reserved`] と同型の重複実装。
/// モジュール間の相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Root パーツ（`div`）。
#[must_use]
pub fn root<'a>(
    props: &TagsInputProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。意味論的なラベル関連付けは呼び出し側が
/// `attrs` 経由で `for`/`id`（または labelledby）を配線する（装飾用パーツ、
/// [`crate::pin_input::label`] と同じ最小主義。`role="listbox"` 撤去に伴い
/// [`control`]/[`input`] のアクセシブルネームはこの関連付けが担う）。
#[must_use]
pub fn label<'a>(
    props: &TagsInputProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LABEL_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = state_attrs(props);
    merged.extend(data_required(props.required));
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Control パーツ（`div`）。タグ [`item`] 群 + [`input`] を並べるコンテナ。
/// `role` は持たない（zag/ark 準拠、モジュール doc「参照突合」節参照）。
#[must_use]
pub fn control<'a>(
    props: &TagsInputProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = state_attrs(props);
    merged.extend(attrs);
    ANATOMY.part("control", "div", merged, children)
}

/// タグ 1 個分の状態束（[`item`]/[`item_preview`]/[`item_text`]/
/// [`item_input`]/[`item_delete_trigger`] が共有する）。独立した `bool`
/// 引数のままだと clippy `too_many_arguments` を超えやすいため、
/// [`crate::rating_group::RatingItemFlags`] と同型の薄い構造体としてまとめる
/// （イシュー #1623）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagItem<'a> {
    /// タグ文字列そのもの（ユーザー入力、REQ-1 の重点対象）。
    pub value: &'a str,
    /// 無効化状態。
    pub disabled: bool,
    /// 編集モード中かどうか。
    pub editing: bool,
    /// キーボード操作上の強調（highlight）状態かどうか
    /// （確定選択とは独立の軸、[`crate::rating_group::RatingItemFlags`]
    /// rustdoc と同じ区別）。
    pub highlighted: bool,
}

/// Item パーツ（`div`）。タグ 1 個分のコンテナ（[`item_preview`] または
/// 編集モード時の [`item_input`] を子に持つ）。
#[must_use]
pub fn item<'a>(item: &TagItem<'a>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-value", item.value)];
    merged.extend(data_disabled(item.disabled));
    merged.extend(data_editing(item.editing));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemPreview パーツ（`div`）。表示モードのタグチップ本体。編集中は
/// `hidden` 存在属性を出力する（zag と同じく、編集中は [`item_input`] を
/// 表示し本パーツを隠す）。`role` は持たない（モジュール doc「参照突合」
/// 節参照。旧実装の `role="option"` + `aria-selected="true"` 固定は撤去）。
#[must_use]
pub fn item_preview<'a>(
    item: &TagItem<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-value", item.value)];
    merged.extend(data_disabled(item.disabled));
    merged.extend(data_highlighted(item.highlighted));
    if item.editing {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("item-preview", "div", merged, children)
}

/// ItemText パーツ（`div`）。タグ文字列を表示するテキストノードのコンテナ。
/// タグ文字列は children として渡され `render()` の既定エスケープを経由する。
#[must_use]
pub fn item_text<'a>(
    item: &TagItem<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(item.disabled));
    merged.extend(data_highlighted(item.highlighted));
    merged.extend(attrs);
    ANATOMY.part("item-text", "div", merged, children)
}

/// ItemInput パーツ（`input`）。タグ編集モード時のネイティブ入力欄。
/// `value` は編集中の暫定値（動的だが `render()` の既定エスケープ経由）。
/// 非編集時は `hidden` 存在属性を出力する。`tabindex` は固定付与しない
/// （codex-review 指摘: 矢印キー操作・編集開始時のフォーカス配線が
/// 未実装の状態で `tabindex="-1"` を固定すると、従来 Tab で到達できていた
/// 編集欄がキーボード操作不能になる。DOM 配線を実装するまでは既定の
/// Tab 到達性を維持する）。`item.disabled` はネイティブ `disabled` +
/// `data-disabled` へ反映する。
#[must_use]
pub fn item_input<'a>(item: &TagItem<'a>, value: &'a str, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "text"), ("value", value)];
    if !item.editing {
        merged.push(("hidden", ""));
    }
    if item.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(item.disabled));
    merged.extend(attrs);
    ANATOMY.part("item-input", "input", merged, Vec::new())
}

/// ItemDeleteTrigger パーツ（`button`）。当該タグを削除する操作。
/// `aria-label` は `format!` で組み立てた「Delete tag {value}」（zag 既定訳、
/// 動的値だが `render()` の既定エスケープを経由するため注入経路には
/// ならない）。`tabindex` は固定付与しない（codex-review 指摘: ark-ui
/// 公式 Data Attributes/Keyboard Support 表は `tabindex="-1"` + 矢印キー
/// 配線の契約だが、矢印キー操作の DOM 配線が未実装の状態で固定すると
/// 従来 Tab で到達できていた削除ボタンがキーボード操作不能になる。DOM
/// 配線を実装するまでは既定の Tab 到達性を維持する）。[`clear_trigger`]
/// 等の他 trigger パーツと同様に `children` を受け取り、呼び出し側が ×
/// アイコン・視覚ラベルを描画できる（Cursor Bugbot 指摘 #744 review
/// comment 3639762362）。`aria-label` は children の有無に関わらず常に
/// 付与する。
#[must_use]
pub fn item_delete_trigger<'a>(
    item: &TagItem<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    // aria_label は呼び出し時にのみ必要な一時 String であり、el() が
    // 即座に owned String へコピーするため関数スコープを超えて借用が
    // 残ることはない（crates/headless-ui/src/pin_input.rs の input() 参照）。
    let label_str = format!("Delete tag {}", item.value);
    let mut merged: Vec<(&str, &str)> = vec![("type", "button"), aria_label(label_str.as_str())];
    if item.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(item.disabled));
    merged.extend(data_highlighted(item.highlighted));
    merged.extend(attrs);
    ANATOMY.part("item-delete-trigger", "button", merged, children)
}

/// ClearTrigger パーツ（`button`）。全タグを一括削除する操作。
#[must_use]
pub fn clear_trigger<'a>(
    props: &TagsInputProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// Input パーツ（`input`）。新規タグ入力用のネイティブ入力欄。
/// max 到達時は `data-invalid`/`aria-invalid` を出力する。`autocomplete`/
/// `autocorrect`/`autocapitalize`/`enterkeyhint` は zag 固定属性
/// （[`crate::number_input`]（#1613）と同種の判断、モジュール doc
/// 「参照突合」節参照）。
#[must_use]
pub fn input<'a>(
    props: &TagsInputProps,
    value: &'a str,
    at_max: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(attrs, INPUT_RESERVED);
    let mut merged: Vec<(&str, &str)> = vec![
        ("type", "text"),
        ("value", value),
        ("autocomplete", "off"),
        ("autocorrect", "off"),
        ("autocapitalize", "none"),
        ("enterkeyhint", "done"),
    ];
    if props.disabled {
        merged.push(("disabled", ""));
    }
    if props.readonly {
        merged.push(("readonly", ""));
    }
    // data_invalid は `props.invalid || at_max` を一度だけ反映する
    // （`state_attrs(props)` の `data-invalid` と重複出力しないため、
    // ここでは state_attrs を使わず disabled/readonly/invalid を個別に
    // 組み立てる）。
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.extend(data_invalid(props.invalid || at_max));
    if at_max || props.invalid {
        merged.push(("aria-invalid", "true"));
    }
    merged.extend(value.is_empty().then_some(("data-empty", "")));
    merged.extend(attrs);
    ANATOMY.part("input", "input", merged, Vec::new())
}

/// HiddenInput パーツ（`input type="hidden"`）。フォーム送信時に全タグの
/// カンマ結合値を 1 個の値として運ぶ。`props.readonly`/`props.required` は
/// `data-*` のみへ反映し、ネイティブ `readonly`/`required` は付けない
/// （`type="hidden"` では意味を持たない、[`crate::pin_input::hidden_input`]
/// と同じ判断）。
#[must_use]
pub fn hidden_input<'a>(
    props: &TagsInputProps,
    name: &'a str,
    value: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(attrs, LABEL_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "hidden"), ("name", name), ("value", value)];
    if props.disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(state_attrs(props));
    merged.extend(data_required(props.required));
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// LiveRegion パーツ（`div`）。タグ数の変化という視覚的にしか伝わらない
/// 動的更新を支援技術へ通知するための live region（イシュー #1069）。
///
/// `role="status"` + `aria-live="polite"` + `aria-atomic="true"` を固定
/// 付与する（[`crate::toast::root`] と同じ 3 点セット。緊急度は `polite`
/// 固定で引数を取らない）。配置制約・wasm-full との責務境界はモジュール
/// doc「LiveRegion パーツと配置制約」節を参照。通知文言は `children` として
/// 呼び出し側が渡し、`render()` の既定エスケープを経由する。
#[must_use]
pub fn live_region<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        crate::aria::role("status"),
        crate::aria::aria_live(crate::aria::AriaLive::Polite),
        crate::aria::aria_atomic(true),
    ];
    merged.extend(attrs);
    ANATOMY.part("live-region", "div", merged, children)
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
    /// 指定インデックスのタグを編集モードにする（範囲外は no-op、
    /// 強調〔highlight〕状態を解除する）。
    EditStart(usize),
    /// 編集中のタグを新文字列で確定する（編集中でない・空文字列・他タグと
    /// 重複なら編集を破棄し元値を維持する）。
    EditSubmit(String),
    /// 編集を破棄して元値を維持したまま編集モードを終了する。
    EditCancel,
    /// キーボード強調（highlight）を 1 つ前へ移動する（ArrowLeft 相当）。
    /// 強調なしなら末尾タグへ、先頭タグで saturating（それ以上前進しない）。
    /// タグが 0 件なら no-op。
    HighlightPrev,
    /// キーボード強調（highlight）を 1 つ後ろへ移動する（ArrowRight
    /// 相当）。末尾タグを強調中なら強調解除（入力欄へ戻る）。強調なしは
    /// no-op。
    HighlightNext,
    /// キーボード強調（highlight）を解除する（Escape 相当）。
    ClearHighlight,
    /// 強調中のタグを削除する（Delete 相当。強調なしは no-op）。削除後の
    /// 強調は 1 つ前へ移動する（先頭を削除した場合は解除）。
    DeleteHighlighted,
    /// 強調中なら [`Self::DeleteHighlighted`] と同じ、強調なしなら末尾
    /// タグを削除する（Backspace 相当。タグが 0 件なら no-op）。
    Backspace,
}

/// TagsInput の値状態機械。
///
/// `tags` は表示順のタグ列（不変条件: 重複なし・`len() <= max`。この
/// 不変条件は [`Self::update`]/[`Self::from_hydration_attrs`] のいずれの
/// 経路でも破られない）。`editing`/`highlighted` は編集中インデックス・
/// キーボード強調インデックスという ephemeral な DOM 状態であり、
/// hydration では運ばない（モジュール doc参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagsInput {
    tags: Vec<String>,
    max: Option<usize>,
    editing: Option<usize>,
    highlighted: Option<usize>,
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
            highlighted: None,
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

    /// 指定インデックスが現在キーボード強調（highlight）中か。
    #[must_use]
    pub fn is_highlighted(&self, index: usize) -> bool {
        self.highlighted == Some(index)
    }

    /// 現在強調中のインデックス（未強調なら `None`）。
    #[must_use]
    pub fn highlighted_index(&self) -> Option<usize> {
        self.highlighted
    }

    /// 全タグをカンマ結合した値（フォーム送信・[`Self::hidden_input`] が使う）。
    #[must_use]
    pub fn value(&self) -> String {
        self.tags.join(",")
    }

    /// 指定インデックスのタグから [`TagItem`] を組み立てる利便メソッド
    /// （`editing`/`highlighted` を状態機械から導出する）。範囲外は
    /// `None`。
    #[must_use]
    pub fn item_state(&self, index: usize, disabled: bool) -> Option<TagItem<'_>> {
        self.tags.get(index).map(|value| TagItem {
            value,
            disabled,
            editing: self.is_editing(index),
            highlighted: self.is_highlighted(index),
        })
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(
        &self,
        props: &TagsInputProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(props, attrs, children)
    }

    /// [`label`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn label<'a>(
        &self,
        props: &TagsInputProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(props, attrs, children)
    }

    /// [`control`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn control<'a>(
        &self,
        props: &TagsInputProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        // max 到達時は input と同じく control にも data-invalid を反映する
        // （codex-review 指摘: 上限到達時に input だけが invalid になり
        // control の枠線表示等に反映されなかった、Cursor Bugbot 同一指摘
        // "Control omits at-max invalid state"）。TagsInputProps は Copy
        // のため呼び出し元の props を書き換えず、invalid のみ合成した
        // ローカルコピーを free function control() へ渡す。
        let effective_props = TagsInputProps {
            invalid: props.invalid || self.is_at_max(),
            ..*props
        };
        control(&effective_props, attrs, children)
    }

    /// [`item`] へ現在の状態を注入する利便メソッド。範囲外インデックスは
    /// 空の `div` を描画する（[`Self::item_state`] が `None` を返す場合、
    /// disabled/editing/highlighted すべて偽の [`TagItem`] を使う）。
    #[must_use]
    pub fn item<'a>(
        &'a self,
        index: usize,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let fallback = TagItem {
            value: "",
            disabled,
            editing: false,
            highlighted: false,
        };
        let state = self.item_state(index, disabled).unwrap_or(fallback);
        item(&state, attrs, children)
    }

    /// [`item_preview`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn item_preview<'a>(
        &'a self,
        index: usize,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let fallback = TagItem {
            value: "",
            disabled,
            editing: false,
            highlighted: false,
        };
        let state = self.item_state(index, disabled).unwrap_or(fallback);
        item_preview(&state, attrs, children)
    }

    /// [`item_text`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn item_text<'a>(
        &'a self,
        index: usize,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let fallback = TagItem {
            value: "",
            disabled,
            editing: false,
            highlighted: false,
        };
        let state = self.item_state(index, disabled).unwrap_or(fallback);
        item_text(&state, attrs, children)
    }

    /// [`item_input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn item_input<'a>(
        &'a self,
        index: usize,
        disabled: bool,
        value: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let fallback = TagItem {
            value: "",
            disabled,
            editing: false,
            highlighted: false,
        };
        let state = self.item_state(index, disabled).unwrap_or(fallback);
        item_input(&state, value, attrs)
    }

    /// [`item_delete_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn item_delete_trigger<'a>(
        &'a self,
        index: usize,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let fallback = TagItem {
            value: "",
            disabled,
            editing: false,
            highlighted: false,
        };
        let state = self.item_state(index, disabled).unwrap_or(fallback);
        item_delete_trigger(&state, attrs, children)
    }

    /// [`clear_trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn clear_trigger<'a>(
        &self,
        props: &TagsInputProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        clear_trigger(props, attrs, children)
    }

    /// [`input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn input<'a>(
        &self,
        props: &TagsInputProps,
        value: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        input(props, value, self.is_at_max(), attrs)
    }

    /// [`hidden_input`] へ現在の連結値を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        props: &TagsInputProps,
        name: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        let value = self.value();
        hidden_input(props, name, &value, attrs)
    }

    /// [`live_region`] へ委譲する利便メソッド（状態を持たないため素通し、
    /// [`TagsInput::label`] と同じ規約）。
    #[must_use]
    pub fn live_region<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        live_region(attrs, children)
    }

    /// 削除により編集中/強調中インデックスの対象がずれる場合を再調整する
    /// 非公開ヘルパ（[`TagsInputAction::Remove`]/
    /// [`TagsInputAction::DeleteHighlighted`]/[`TagsInputAction::Backspace`]
    /// が共有する。存在しない/別タグを指す状態を残さない）。
    fn reindex_after_remove(&mut self, idx: usize) {
        if self.editing == Some(idx) {
            self.editing = None;
        } else if let Some(e) = self.editing {
            if e > idx {
                self.editing = Some(e - 1);
            }
        }
        if self.highlighted == Some(idx) {
            self.highlighted = if idx == 0 { None } else { Some(idx - 1) };
        } else if let Some(h) = self.highlighted {
            if h > idx {
                self.highlighted = Some(h - 1);
            }
        }
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
                self.reindex_after_remove(idx);
            }
            TagsInputAction::Clear => {
                self.tags.clear();
                self.editing = None;
                self.highlighted = None;
            }
            TagsInputAction::EditStart(idx) => {
                if idx < self.tags.len() {
                    self.editing = Some(idx);
                    self.highlighted = None;
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
            TagsInputAction::HighlightPrev => {
                if self.tags.is_empty() {
                    return;
                }
                self.highlighted = match self.highlighted {
                    None => Some(self.tags.len() - 1),
                    Some(i) => Some(i.saturating_sub(1)),
                };
            }
            TagsInputAction::HighlightNext => {
                let Some(i) = self.highlighted else { return };
                self.highlighted = if i + 1 >= self.tags.len() {
                    None
                } else {
                    Some(i + 1)
                };
            }
            TagsInputAction::ClearHighlight => {
                self.highlighted = None;
            }
            TagsInputAction::DeleteHighlighted => {
                let Some(idx) = self.highlighted else { return };
                if idx >= self.tags.len() {
                    return;
                }
                self.tags.remove(idx);
                self.reindex_after_remove(idx);
            }
            TagsInputAction::Backspace => {
                if let Some(idx) = self.highlighted {
                    if idx < self.tags.len() {
                        self.tags.remove(idx);
                        self.reindex_after_remove(idx);
                    }
                    return;
                }
                if let Some(last) = self.tags.len().checked_sub(1) {
                    self.tags.remove(last);
                    self.reindex_after_remove(last);
                }
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー
    /// （root > control > (item > item-preview > item-text) × len + input）。
    /// 公開 UI としての利用は想定しない（実際の UI 構築は各パーツメソッドを
    /// 呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let props = TagsInputProps::default();
        let items: Vec<Node> = self
            .tags
            .iter()
            .map(|tag| {
                let state = TagItem {
                    value: tag,
                    disabled: false,
                    editing: false,
                    highlighted: false,
                };
                let preview = item_preview(
                    &state,
                    Vec::new(),
                    vec![item_text(
                        &state,
                        Vec::new(),
                        vec![fandhe_frontend_core::text(tag)],
                    )],
                );
                item(&state, Vec::new(), vec![preview])
            })
            .collect();
        let mut control_children = items;
        control_children.push(self.input(&props, "", Vec::new()));
        self.root(
            &props,
            Vec::new(),
            vec![self.control(&props, Vec::new(), control_children)],
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
            "highlight-prev" => Some(TagsInputAction::HighlightPrev),
            "highlight-next" => Some(TagsInputAction::HighlightNext),
            "highlight-clear" => Some(TagsInputAction::ClearHighlight),
            "delete-highlighted" => Some(TagsInputAction::DeleteHighlighted),
            "backspace" => Some(TagsInputAction::Backspace),
            _ => None,
        }
    }
}

impl Hydrate for TagsInput {
    /// [`codec::encode_list`] でタグ列を運ぶ（[`crate::pin_input::PinInput`]
    /// の `values` と同型）。`max` は `"none"` または非負整数文字列。
    /// `editing`/`highlighted` は ephemeral のため運ばない（モジュール doc
    /// 参照）。
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
            // 編集中/強調中インデックスはいずれも ephemeral な DOM 状態の
            // ため運ばない（モジュール doc 参照）。復元直後は常に未設定。
            editing: None,
            highlighted: None,
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

    fn tag_item(value: &str) -> TagItem<'_> {
        TagItem {
            value,
            disabled: false,
            editing: false,
            highlighted: false,
        }
    }

    // --- 各パーツの data-scope/data-part/data-disabled 出力 ---

    #[test]
    fn root_outputs_scope_part_and_no_state_when_enabled() {
        let html = render(&root(&TagsInputProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn root_props_reflect_disabled_invalid_readonly() {
        let props = TagsInputProps {
            disabled: true,
            readonly: true,
            invalid: true,
            required: false,
        };
        let html = render(&root(&props, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn label_required_outputs_data_required() {
        let props = TagsInputProps {
            required: true,
            ..Default::default()
        };
        let html = render(&label(&props, vec![], vec![text("Tags")]));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"data-required="""#));
        assert!(html.contains("Tags"));
    }

    #[test]
    fn control_no_longer_outputs_listbox_role() {
        // #1623: zag/ark の control は role を持たない
        // （モジュール doc「参照突合」節参照）。
        let html = render(&control(&TagsInputProps::default(), vec![], vec![]));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="control""#));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-orientation"));
        assert!(!html.contains("data-invalid"));
    }

    #[test]
    fn control_invalid_true_outputs_data_invalid() {
        let props = TagsInputProps {
            invalid: true,
            ..Default::default()
        };
        let html = render(&control(&props, vec![], vec![]));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn item_outputs_scope_part_value_and_editing_state() {
        let mut state = tag_item("rust");
        state.editing = true;
        let html = render(&item(&state, vec![], vec![]));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"data-value="rust""#));
        assert!(html.contains(r#"data-editing="""#));
    }

    #[test]
    fn item_not_editing_omits_data_editing() {
        let html = render(&item(&tag_item("rust"), vec![], vec![]));
        assert!(!html.contains("data-editing"));
    }

    #[test]
    fn item_preview_no_longer_outputs_option_role_or_aria_selected() {
        // #1623: 旧実装の role="option" + aria-selected="true" 固定は撤去
        // （モジュール doc「参照突合」節参照）。
        let html = render(&item_preview(&tag_item("rust"), vec![], vec![]));
        assert!(html.contains(r#"data-part="item-preview""#));
        assert!(html.contains(r#"data-value="rust""#));
        assert!(!html.contains("role="));
        assert!(!html.contains("aria-selected"));
    }

    #[test]
    fn item_preview_editing_outputs_hidden() {
        let mut state = tag_item("rust");
        state.editing = true;
        let html = render(&item_preview(&state, vec![], vec![]));
        assert!(html.contains("hidden"));
    }

    #[test]
    fn item_preview_highlighted_outputs_data_highlighted() {
        let mut state = tag_item("rust");
        state.highlighted = true;
        let html = render(&item_preview(&state, vec![], vec![]));
        assert!(html.contains(r#"data-highlighted="""#));
    }

    #[test]
    fn item_text_outputs_scope_and_part_with_tag_text() {
        let html = render(&item_text(&tag_item("rust"), vec![], vec![text("rust")]));
        assert!(html.contains(r#"data-part="item-text""#));
        assert!(html.contains("rust"));
    }

    #[test]
    fn item_input_editing_outputs_type_text_value_and_no_hidden() {
        let mut state = tag_item("rust");
        state.editing = true;
        let html = render(&item_input(&state, "editing-value", vec![]));
        assert!(html.contains(r#"data-part="item-input""#));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"value="editing-value""#));
        // #1623 codex-review: 矢印キー配線・編集開始時フォーカス配線が
        // 未実装のため tabindex="-1" は固定付与しない（既定 Tab 到達性を
        // 維持する）。
        assert!(!html.contains("tabindex"));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn item_input_not_editing_outputs_hidden() {
        let html = render(&item_input(&tag_item("rust"), "", vec![]));
        assert!(html.contains("hidden"));
    }

    #[test]
    fn item_delete_trigger_outputs_type_button_and_zag_aria_label() {
        let html = render(&item_delete_trigger(&tag_item("rust"), vec![], vec![]));
        assert!(html.contains(r#"data-part="item-delete-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Delete tag rust""#));
        // #1623 codex-review: 矢印キー配線が未実装のため tabindex="-1" は
        // 固定付与しない（既定 Tab 到達性を維持する）。
        assert!(!html.contains("tabindex"));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn item_delete_trigger_disabled_outputs_native_disabled() {
        let mut state = tag_item("rust");
        state.disabled = true;
        let html = render(&item_delete_trigger(&state, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn clear_trigger_outputs_type_button() {
        let html = render(&clear_trigger(
            &TagsInputProps::default(),
            vec![],
            vec![text("Clear")],
        ));
        assert!(html.contains(r#"data-part="clear-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains("Clear"));
    }

    #[test]
    fn clear_trigger_readonly_outputs_data_readonly() {
        let props = TagsInputProps {
            readonly: true,
            ..Default::default()
        };
        let html = render(&clear_trigger(&props, vec![], vec![]));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn input_outputs_type_text_value_and_zag_fixed_attrs() {
        let html = render(&input(&TagsInputProps::default(), "draft", false, vec![]));
        assert!(html.contains(r#"data-part="input""#));
        assert!(html.contains(r#"type="text""#));
        assert!(html.contains(r#"value="draft""#));
        assert!(html.contains(r#"autocomplete="off""#));
        assert!(html.contains(r#"autocorrect="off""#));
        assert!(html.contains(r#"autocapitalize="none""#));
        assert!(html.contains(r#"enterkeyhint="done""#));
        assert!(!html.contains("data-invalid"));
        assert!(!html.contains("aria-invalid"));
        assert!(!html.contains("data-empty"));
    }

    #[test]
    fn input_empty_value_outputs_data_empty() {
        let html = render(&input(&TagsInputProps::default(), "", false, vec![]));
        assert!(html.contains(r#"data-empty="""#));
    }

    #[test]
    fn input_at_max_outputs_data_invalid_and_aria_invalid() {
        let html = render(&input(&TagsInputProps::default(), "", true, vec![]));
        assert!(html.contains(r#"data-invalid="""#));
        assert!(html.contains(r#"aria-invalid="true""#));
    }

    #[test]
    fn input_props_invalid_and_at_max_does_not_duplicate_data_invalid() {
        // props.invalid と at_max が両方 true でも data-invalid は 1 回だけ
        // 出力する（state_attrs 経由と at_max 経由の二重出力回帰防止）。
        let props = TagsInputProps {
            invalid: true,
            ..Default::default()
        };
        let html = render(&input(&props, "", true, vec![]));
        assert_eq!(html.matches("data-invalid").count(), 1);
    }

    #[test]
    fn input_readonly_outputs_native_readonly() {
        let props = TagsInputProps {
            readonly: true,
            ..Default::default()
        };
        let html = render(&input(&props, "", false, vec![]));
        assert!(html.contains(r#"readonly="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn hidden_input_outputs_type_hidden_name_and_value() {
        let html = render(&hidden_input(
            &TagsInputProps::default(),
            "tags",
            "rust,wasm",
            vec![],
        ));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="hidden""#));
        assert!(html.contains(r#"name="tags""#));
        assert!(html.contains(r#"value="rust,wasm""#));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn hidden_input_disabled_outputs_native_disabled_and_data_disabled() {
        let props = TagsInputProps {
            disabled: true,
            ..Default::default()
        };
        let html = render(&hidden_input(&props, "tags", "", vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn hidden_input_required_does_not_output_native_required() {
        // type="hidden" では制約検証対象外のためネイティブ required は
        // 付けない（モジュール doc「参照突合」節参照）。
        let props = TagsInputProps {
            required: true,
            ..Default::default()
        };
        let html = render(&hidden_input(&props, "tags", "", vec![]));
        assert!(html.contains(r#"data-required="""#));
        assert!(!html.contains(r#" required"#));
    }

    #[test]
    fn live_region_has_role_status_polite_and_atomic() {
        let html = render(&live_region(vec![], vec![text("1 tag")]));
        assert!(html.contains(r#"data-part="live-region""#));
        assert!(html.contains(r#"role="status""#));
        assert!(html.contains(r#"aria-live="polite""#));
        assert!(html.contains(r#"aria-atomic="true""#));
        assert!(html.contains("1 tag"));
    }

    // --- Anatomy::part fail-closed 回帰 ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            &TagsInputProps::default(),
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_supplied_data_disabled_on_root_is_dropped() {
        // drop_reserved による fail-closed（呼び出し側がなりすまし付与
        // できない、モジュール doc「セキュリティ不変条件」節参照）。
        let html = render(&root(
            &TagsInputProps::default(),
            vec![("data-disabled", "")],
            vec![],
        ));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn live_region_caller_supplied_scope_and_part_are_dropped() {
        let html = render(&live_region(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tags-input""#));
        assert!(html.contains(r#"data-part="live-region""#));
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
        assert_eq!(t.highlighted_index(), None);
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
    fn clear_action_clears_highlight() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "highlight-prev", "");
        assert!(dispatch(&mut t, "clear", ""));
        assert_eq!(t.highlighted_index(), None);
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
    fn edit_start_clears_highlight() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "highlight-prev", "");
        assert_eq!(t.highlighted_index(), Some(1));
        assert!(dispatch(&mut t, "edit-start", "0"));
        assert_eq!(t.highlighted_index(), None);
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

    // --- TagsInput: highlight（キーボード強調）遷移 ---

    #[test]
    fn highlight_prev_from_none_selects_last_tag() {
        let mut t = TagsInput::new(tags(&["a", "b", "c"]), None);
        assert!(dispatch(&mut t, "highlight-prev", ""));
        assert_eq!(t.highlighted_index(), Some(2));
    }

    #[test]
    fn highlight_prev_saturates_at_first_tag() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "highlight-prev", "");
        dispatch(&mut t, "highlight-prev", "");
        assert!(dispatch(&mut t, "highlight-prev", ""));
        assert_eq!(t.highlighted_index(), Some(0));
    }

    #[test]
    fn highlight_prev_on_empty_tags_is_no_op() {
        let mut t = TagsInput::default();
        assert!(dispatch(&mut t, "highlight-prev", ""));
        assert_eq!(t.highlighted_index(), None);
    }

    #[test]
    fn highlight_next_from_last_tag_returns_to_input() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "highlight-prev", "");
        assert_eq!(t.highlighted_index(), Some(1));
        assert!(dispatch(&mut t, "highlight-next", ""));
        assert_eq!(t.highlighted_index(), None);
    }

    #[test]
    fn highlight_next_without_highlight_is_no_op() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        assert!(dispatch(&mut t, "highlight-next", ""));
        assert_eq!(t.highlighted_index(), None);
    }

    #[test]
    fn highlight_clear_resets_highlight() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "highlight-prev", "");
        assert!(dispatch(&mut t, "highlight-clear", ""));
        assert_eq!(t.highlighted_index(), None);
    }

    #[test]
    fn delete_highlighted_removes_tag_and_moves_highlight_back() {
        let mut t = TagsInput::new(tags(&["a", "b", "c"]), None);
        dispatch(&mut t, "highlight-prev", ""); // highlight index 2 ("c")
        dispatch(&mut t, "highlight-prev", ""); // highlight index 1 ("b")
        assert!(dispatch(&mut t, "delete-highlighted", ""));
        assert_eq!(t.tags(), &tags(&["a", "c"]));
        assert_eq!(t.highlighted_index(), Some(0));
    }

    #[test]
    fn delete_highlighted_first_tag_clears_highlight() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "highlight-prev", "");
        dispatch(&mut t, "highlight-prev", "");
        assert_eq!(t.highlighted_index(), Some(0));
        assert!(dispatch(&mut t, "delete-highlighted", ""));
        assert_eq!(t.tags(), &tags(&["b"]));
        assert_eq!(t.highlighted_index(), None);
    }

    #[test]
    fn delete_highlighted_without_highlight_is_no_op() {
        let mut t = TagsInput::new(tags(&["a"]), None);
        assert!(dispatch(&mut t, "delete-highlighted", ""));
        assert_eq!(t.tags(), &tags(&["a"]));
    }

    #[test]
    fn backspace_without_highlight_removes_last_tag() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        assert!(dispatch(&mut t, "backspace", ""));
        assert_eq!(t.tags(), &tags(&["a"]));
    }

    #[test]
    fn backspace_with_highlight_removes_highlighted_tag() {
        let mut t = TagsInput::new(tags(&["a", "b", "c"]), None);
        dispatch(&mut t, "highlight-prev", "");
        dispatch(&mut t, "highlight-prev", ""); // highlight index 1 ("b")
        assert!(dispatch(&mut t, "backspace", ""));
        assert_eq!(t.tags(), &tags(&["a", "c"]));
    }

    #[test]
    fn backspace_on_empty_tags_is_no_op() {
        let mut t = TagsInput::default();
        assert!(dispatch(&mut t, "backspace", ""));
        assert!(t.is_empty());
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

    #[test]
    fn hydration_does_not_carry_highlighted_state() {
        let mut t = TagsInput::new(tags(&["a", "b"]), None);
        dispatch(&mut t, "highlight-prev", "");
        assert_eq!(t.highlighted_index(), Some(1));
        let restored = TagsInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.highlighted_index(), None);
    }

    // --- XSS 回帰: タグ文字列/name/value/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
    const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

    #[test]
    fn item_text_tag_payload_is_escaped_on_render() {
        let html = render(&item_text(
            &tag_item(SCRIPT_PAYLOAD),
            vec![],
            vec![text(SCRIPT_PAYLOAD)],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn item_data_value_payload_is_escaped_on_render() {
        let html = render(&item(&tag_item(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_preview_data_value_payload_is_escaped_on_render() {
        let html = render(&item_preview(&tag_item(ATTR_BREAK_PAYLOAD), vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn item_delete_trigger_aria_label_tag_payload_is_escaped_on_render() {
        let html = render(&item_delete_trigger(
            &tag_item(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn live_region_children_and_attrs_payload_is_escaped_on_render() {
        let html = render(&live_region(
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![text(SCRIPT_PAYLOAD)],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hidden_input_value_payload_is_escaped_on_render() {
        let html = render(&hidden_input(
            &TagsInputProps::default(),
            "tags",
            ATTR_BREAK_PAYLOAD,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            &TagsInputProps::default(),
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
        let html = render(&item_text(&tag_item(tag), vec![], vec![text(tag)]));
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
        let tag = &restored.tags()[0];
        let html = render(&item_text(&tag_item(tag), vec![], vec![text(tag)]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
