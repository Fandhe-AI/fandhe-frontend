//! TreeView（階層構造の展開・折りたたみ・選択）headless コンポーネント
//! （イシュー #753、親トラッキング #748/#520）。
//!
//! ark-ui の TreeView
//!（`.claude/skills/ark-ui/references/components/collections/tree-view.md`）を
//! 参考に、Root / Label / Tree / Branch / BranchControl / BranchIndicator /
//! BranchText / BranchContent / BranchIndentGuide / Item / ItemText /
//! ItemIndicator の 12 anatomy パーツと、[`crate::state::MultiSelect`]
//! （展開中のブランチ値の集合）+ [`crate::state::SingleSelect`]（選択中の
//! ノード値）を合成した状態機械 [`TreeView`] を提供する。
//!
//! ツリーデータは ark-ui の `createTreeCollection` 相当を、決定的な静的構造体
//! [`TreeNode`] で表現する（ランタイムでの動的なコレクション操作 API は持たず、
//! 呼び出し側が構築した木をそのまま描画する）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`label`]/[`tree`]/[`branch`]/
//! [`branch_control`]/[`branch_indicator`]/[`branch_text`]/[`branch_content`]/
//! [`branch_indent_guide`]/[`item`]/[`item_text`]/[`item_indicator`]、いずれも
//! 純粋関数で完結）を直接呼んで組み立てるか、[`TreeView::render_nodes`] に
//! [`TreeNode`] 列を渡して深さ・`aria-posinset`/`aria-setsize` を再帰的に
//! 計算させる。CSR/hydration は [`TreeView`]
//! （[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] 実装）を経由し、dispatch
//! （`"expand"`/`"collapse"`/`"toggle"`/`"select"`/`"deselect"`）で展開集合・
//! 選択値の状態遷移をする。`fandhe-frontend-pre-styled-ui`（イシュー #753）が
//! 本モジュールを呼んでスタイル済み TreeView を組み立てる想定である。
//!
//! # anatomy 12 パーツ（ark-ui との対応）
//!
//! ark-ui の `NodeProvider` は React context 相当でありマークアップを持たない
//! ため、本モジュールの anatomy 対象外とする（`docs/design`
//! （該当があれば）参照は不要な純粋な設計判断）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`hidden`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`mod@crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の既存
//!   不変条件をそのまま継承する）。
//! - 動的値（ノード値/ラベル/`aria-level`・`aria-posinset`・`aria-setsize`・
//!   `data-depth` の数値文字列/呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に一元化し、本モジュールで独自の値を作らない。
//! - hydration 属性はクライアント側で改ざんされうる入力として扱う。
//!   [`TreeView`] の [`fandhe_frontend_interactive::Hydrate`] 実装は展開集合
//!   フィールド名の衝突回避（下記 §hydration フィールド名）を除き
//!   [`crate::state::MultiSelect`]/[`crate::state::SingleSelect`] の既存
//!   dedupe/型検証へ委譲し、panic せず `HydrateError` を返す既存保証を
//!   継承する。
//! - dispatch payload（ブランチ値・ノード値）は改ざんされうるクライアント
//!   入力として扱い、HTML として解釈せず値として保持する（[`crate::state`]
//!   の既存契約を継承）。
//!
//! # hydration フィールド名（[`MultiSelect`]/[`SingleSelect`] 併用時の衝突回避）
//!
//! [`crate::state::MultiSelect::FIELD_SELECTED`]（フィールド部分 `"selected"`）
//! と [`crate::state::SingleSelect::FIELD_SELECTED`]（同じく `"selected"`）は
//! 単独利用時は型が異なるため衝突しないが（`state` モジュール doc 参照）、
//! 本モジュールのように両方を 1 コンポーネントへ埋め込む場合、そのまま
//! 併記すると `data-hydrate-selected` 属性名が重複してしまう。そのため
//! [`TreeView`] は展開集合側のみ [`TreeView::FIELD_EXPANDED`]
//! （`"expanded"`）へ書き換えて運び、選択値側は [`SingleSelect`] の
//! `"selected"` をそのまま使う。[`MultiSelect::hydration_attrs`]/
//! [`MultiSelect::from_hydration_attrs`] が持つ重複値拒否等の検証ロジックは
//! 属性名を書き換えた一時 `Vec` を経由して呼び出すことで再実装しない
//! （[`Hydrate`] 実装内のコメント参照）。
//!
//! # out-of-scope（本イシュー #753 のスコープ外）
//!
//! - **キーボードナビゲーション・typeahead**: ARIA APG の Tree パターンが
//!   要求する矢印キー操作・文字入力によるジャンプは、SSR 静的マークアップに
//!   寄与しない CSR 挙動層（`fandhe-frontend-wasm-full` 後続イシュー）の
//!   責務のため未提供。
//! - **checkbox モード（`checkedValue`）**: ark-ui の checkbox 選択（複数
//!   チェック可能なツリー）は採用しない。本モジュールは高々 1 個の選択
//!   （[`crate::state::SingleSelect`]）のみを提供する。
//! - **複数選択（multiple selection）**: 上記と同じ理由でスコープ外。
//! - **lazy loading（`loadChildren`）**: [`TreeNode`] は決定的な静的コレクション
//!   のみを表現し、非同期の子ノード読み込みは持たない。
//! - **inline renaming・virtualization**: いずれも CSR 挙動層の責務であり
//!   本イシューのスコープ外。
//! - **wasm-full の実 DOM 配線（クリック→dispatch）**: `PositionedKind`
//!   相当の位置決めは TreeView には不要だが、クリックイベント→dispatch
//!   ペイロード組み立ての実配線は後続イシューのスコープ。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label as aria_label_attr, aria_labelledby, aria_selected, role};
use crate::data_attrs::{data_disabled, data_state};
use crate::state::{MultiSelect, MultiSelectAction, OpenState, SingleSelect, SingleSelectAction};
use fandhe_frontend_core::{text, Node};
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError};

/// TreeView の anatomy（`data-scope="tree-view"`）。
const ANATOMY: Anatomy = anatomy("tree-view");

/// `data-selected` 存在属性。[`crate::data_attrs::data_disabled`] と同じ
/// 「存在で真を表す」規約に従う。選択有無は `aria-selected`（明示 2 値）と
/// 併記し、CSS セレクタで `[data-selected]` の有無だけを見たい呼び出し側の
/// 利便性のために提供する（`crate::data_attrs` へは汎化せず本モジュール内に
/// 留める。TreeView 固有の "選択" 意味論であり、他コンポーネントの
/// `data-checked`/`data-pressed` とは別軸のため）。
fn data_selected(selected: bool) -> Option<(&'static str, &'static str)> {
    selected.then_some(("data-selected", ""))
}

/// Root パーツ（`div`）。状態非依存。
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "div", attrs, children)
}

/// Label パーツ（`span`）。ツリー全体の見出しテキスト（装飾用パーツ、
/// [`crate::slider::label`] と同型の最小主義）。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "span", attrs, children)
}

/// Tree パーツ（`div[role="tree"]`）。WAI-ARIA APG の Tree パターンに従い
/// `role="tree"` を固定付与する。`aria_label`/`aria_labelledby` はどちらか
/// 一方が `Some` の場合のみ出力する（アクセシブルな名前を持たない `tree`
/// は支援技術上の識別が困難なため、呼び出し側にいずれかの指定を促す設計だが、
/// 両方 `None` でも fail-closed に属性省略で描画は継続する）。
#[must_use]
pub fn tree<'a>(
    aria_label_text: Option<&'a str>,
    aria_labelledby_id: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("tree")];
    if let Some(text) = aria_label_text {
        merged.push(aria_label_attr(text));
    }
    if let Some(id) = aria_labelledby_id {
        merged.push(aria_labelledby(id));
    }
    merged.extend(attrs);
    ANATOMY.part("tree", "div", merged, children)
}

/// Branch パーツ（`div[role="treeitem"]`）。子を持つノード 1 個を表す。
///
/// `level`/`posinset`/`setsize`/`depth` は呼び出し側（[`TreeView::render_nodes`]
/// 等）が事前に `usize` から文字列化した値を渡す（[`crate::slider::thumb`]
/// が `min`/`max`/`now` を `&str` で受ける方針と同型。数値 ARIA 属性値は
/// 所有 `String` を要求し `(&str, &str)` 型のヘルパ体系と噛み合わないため、
/// `crate::aria` へは追加せず呼び出し側が `format!` した文字列をそのまま渡す）。
/// `level` は 1 起点（WAI-ARIA `aria-level` の仕様どおり）、`depth` は
/// `data-depth` 用に 0 起点（ark-ui 準拠、CSS のインデント計算に使う想定）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn branch<'a>(
    state: OpenState,
    value: &'a str,
    selected: bool,
    disabled: bool,
    level: &'a str,
    posinset: &'a str,
    setsize: &'a str,
    depth: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("treeitem"),
        (
            "aria-expanded",
            if state.is_open() { "true" } else { "false" },
        ),
        aria_selected(selected),
        ("aria-level", level),
        ("aria-posinset", posinset),
        ("aria-setsize", setsize),
        data_state(state.as_data_state()),
        ("data-value", value),
        ("data-depth", depth),
    ];
    merged.extend(data_selected(selected));
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("branch", "div", merged, children)
}

/// BranchControl パーツ（`div`）。クリック対象の要約行（ark-ui 準拠で
/// `role`/`<button>` は持たない。実際の treeitem ロールは [`branch`] 側が
/// 担う。クリック→dispatch の実 DOM 配線は wasm 層の後続責務、モジュール
/// doc §out-of-scope 参照）。`selected` は [`branch`] と同じ選択値を渡す
/// （`data-selected` を要約行自身にも反映し、styled 層が
/// `[data-part="branch-control"][data-selected]` セレクタで選択強調を
/// 適用できるようにする。`branch` のみに付与すると要約行が視覚的に
/// 選択されないため、Cursor Bugbot 指摘 #798 で追加）。
#[must_use]
pub fn branch_control<'a>(
    state: OpenState,
    selected: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(data_selected(selected));
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("branch-control", "div", merged, children)
}

/// BranchIndicator パーツ（`span`）。開閉状態のみを `data-state` へ反映する
/// 最小主義な装飾用パーツ（アイコン等は呼び出し側の `attrs`/`children` が
/// 担う。[`crate::accordion::item_indicator`] と同型）。
#[must_use]
pub fn branch_indicator<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(state.as_data_state())];
    merged.extend(attrs);
    ANATOMY.part("branch-indicator", "span", merged, children)
}

/// BranchText パーツ（`span`）。ブランチのラベルテキストを表示する。
#[must_use]
pub fn branch_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("branch-text", "span", attrs, children)
}

/// BranchContent パーツ（`div[role="group"]`）。ブランチの子ノード列を
/// 包むコンテナ。WAI-ARIA APG の Tree パターンに従いネストされた
/// `role="group"` を固定付与する。closed のとき `hidden` 存在属性を付与し、
/// JS なしの SSR でも閉状態を表現する（[`crate::accordion::item_content`]
/// と同型、アニメーション対応はスコープ外）。
#[must_use]
pub fn branch_content<'a>(
    state: OpenState,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![role("group"), data_state(state.as_data_state())];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("branch-content", "div", merged, children)
}

/// BranchIndentGuide パーツ（`div`）。深さに応じた縦インデントガイド線を
/// 描く装飾用パーツ（styled 層が CSS custom property でインデント幅を
/// 制御する想定、headless 側は状態を持たない）。
#[must_use]
pub fn branch_indent_guide<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("branch-indent-guide", "div", attrs, children)
}

/// Item パーツ（`div[role="treeitem"]`）。子を持たない葉ノード 1 個を表す。
/// 引数の意味は [`branch`] と同じ（`aria-expanded`/`data-state` は持たない
/// 点のみ異なる。葉ノードは開閉状態を持たないため）。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn item<'a>(
    value: &'a str,
    selected: bool,
    disabled: bool,
    level: &'a str,
    posinset: &'a str,
    setsize: &'a str,
    depth: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("treeitem"),
        aria_selected(selected),
        ("aria-level", level),
        ("aria-posinset", posinset),
        ("aria-setsize", setsize),
        ("data-value", value),
        ("data-depth", depth),
    ];
    merged.extend(data_selected(selected));
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemText パーツ（`span`）。葉ノードのラベルテキストを表示する。
#[must_use]
pub fn item_text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-text", "span", attrs, children)
}

/// ItemIndicator パーツ（`span`）。選択状態のみを `data-selected` へ反映する
/// 最小主義な装飾用パーツ（チェックマーク等のアイコンは呼び出し側の
/// `attrs`/`children` が担う）。
#[must_use]
pub fn item_indicator<'a>(
    selected: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_selected(selected));
    merged.extend(attrs);
    ANATOMY.part("item-indicator", "span", merged, children)
}

/// ツリーの 1 ノード（ark-ui の `createTreeCollection` 相当を決定的な静的
/// 構造体で表現する）。
///
/// `children` が空ならば葉ノード（[`item`] で描画）、1 個以上ならばブランチ
/// （[`branch`]/[`branch_content`] で描画）として扱う（[`Self::is_branch`]）。
/// ランタイムでの動的な追加・削除・並べ替え API は持たない（呼び出し側が
/// 構築し終えた木をそのまま [`TreeView::render_nodes`] へ渡す想定）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    value: String,
    label: String,
    children: Vec<TreeNode>,
    disabled: bool,
}

impl TreeNode {
    /// 指定した値・ラベルで葉ノード（`children` 空）を作る。
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: Vec::new(),
            disabled: false,
        }
    }

    /// 子ノード列を設定する（ビルダメソッド）。空の `Vec` を渡すと葉ノード
    /// のまま変わらない。
    #[must_use]
    pub fn with_children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }

    /// disabled 状態を設定する（ビルダメソッド）。
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// このノードの値（dispatch payload・`data-value` に使う識別子）。
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// このノードの表示ラベル。
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// 子ノード列（葉ノードなら空スライス）。
    #[must_use]
    pub fn children(&self) -> &[TreeNode] {
        &self.children
    }

    /// disabled 状態かどうか。
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// 子を 1 個以上持つブランチかどうか（`false` なら葉ノード）。
    #[must_use]
    pub fn is_branch(&self) -> bool {
        !self.children.is_empty()
    }
}

/// [`TreeView`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは [`TreeView::decode_action`]
/// で接続する。[`crate::state::MultiSelect`] の `"select"`/`"deselect"`/
/// `"toggle"`（展開集合の追加・除去・反転）と [`crate::state::SingleSelect`]
/// の `"select"`/`"deselect"`（選択値の設定・解除）は dispatch 名が衝突する
/// ため（両者とも `"select"`/`"toggle"` 相当の語彙を持つ）、いずれの埋め込み
/// 状態機械へも生の名前を転送せず、本モジュール独自の語彙
/// （`"expand"`/`"collapse"`/`"toggle"` はブランチの展開制御、
/// `"select"`/`"deselect"` はノードの選択制御）へ一度デコードしてから
/// 内部で使い分ける（[`crate::combobox::ComboboxAction`] が `"toggle"` を
/// 独自デコードするのと同型の判断）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeViewAction {
    /// 指定したブランチ値を展開する（既に展開中なら no-op）。
    Expand(String),
    /// 指定したブランチ値を折りたたむ（未展開なら no-op）。
    Collapse(String),
    /// 指定したブランチ値の展開/折りたたみをトグルする。
    ToggleBranch(String),
    /// 指定したノード値を選択する（他の選択は解除される）。
    Select(String),
    /// 選択を解除する。
    Deselect,
}

/// [`MultiSelect`]（展開中のブランチ値の集合）+ [`SingleSelect`]（選択中の
/// ノード値）を埋め込んだ TreeView の状態機械。
///
/// 状態を取る各パーツ関数（[`branch`]/[`branch_control`]/
/// [`branch_indicator`]/[`branch_content`]/[`item`]/[`item_indicator`]）へ
/// 現在状態を注入する主な入口は [`Self::render_nodes`]（[`TreeNode`] 列から
/// 深さ・`aria-posinset`/`aria-setsize` を再帰的に計算しつつ全体を組み立てる）
/// である。SSR での自由関数直接利用（本型を経由しない構成）も引き続き可能。
/// `Default` は全ブランチ折りたたみ・未選択（SSR の状態なし初期描画に対応
/// する既定値）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TreeView {
    expanded: MultiSelect,
    selected: SingleSelect,
}

impl TreeView {
    /// `data-hydrate-expanded` 属性名のフィールド部分。[`MultiSelect`] の
    /// `"selected"` をそのまま使うと [`SingleSelect`] の
    /// `data-hydrate-selected` と衝突するため、TreeView 固有の名前を持つ
    /// （モジュール doc §hydration フィールド名 参照）。
    pub const FIELD_EXPANDED: &'static str = "expanded";

    /// 指定したブランチ値が展開中かどうか。
    #[must_use]
    pub fn is_expanded(&self, value: &str) -> bool {
        self.expanded.is_selected(value)
    }

    /// ブランチ `value` の現在の [`OpenState`]。
    #[must_use]
    pub fn branch_state(&self, value: &str) -> OpenState {
        if self.is_expanded(value) {
            OpenState::Open
        } else {
            OpenState::Closed
        }
    }

    /// 現在選択中のノード値（未選択なら `None`）。
    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selected.selected()
    }

    /// 指定したノード値が選択中かどうか。
    #[must_use]
    pub fn is_selected(&self, value: &str) -> bool {
        self.selected.is_selected(value)
    }

    /// [`TreeNode`] 列を現在の展開・選択状態で再帰的に描画する。
    ///
    /// 各階層で `aria-posinset`（1 起点の兄弟内位置）・`aria-setsize`
    /// （兄弟数）・`aria-level`（1 起点の深さ）・`data-depth`（0 起点の深さ）
    /// を決定的に計算し、[`branch`]/[`branch_control`]/[`branch_indicator`]/
    /// [`branch_text`]/[`branch_content`]/[`item`]/[`item_text`]/
    /// [`item_indicator`] を組み合わせて完全なマークアップを組み立てる
    /// （[`crate::accordion::MultiAccordion`] の状態注入利便メソッド群と同じ
    /// 位置付けの、木構造向け再帰版）。
    #[must_use]
    pub fn render_nodes(&self, nodes: &[TreeNode]) -> Vec<Node> {
        Self::render_level(&self.expanded, &self.selected, nodes, 0)
    }

    fn render_level(
        expanded: &MultiSelect,
        selected: &SingleSelect,
        nodes: &[TreeNode],
        depth: usize,
    ) -> Vec<Node> {
        let setsize = nodes.len();
        let setsize_s = setsize.to_string();
        let level_s = (depth + 1).to_string();
        let depth_s = depth.to_string();

        nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let posinset_s = (index + 1).to_string();
                let is_selected = selected.is_selected(node.value());

                if node.is_branch() {
                    let state = if expanded.is_selected(node.value()) {
                        OpenState::Open
                    } else {
                        OpenState::Closed
                    };
                    let child_nodes =
                        Self::render_level(expanded, selected, node.children(), depth + 1);
                    branch(
                        state,
                        node.value(),
                        is_selected,
                        node.is_disabled(),
                        &level_s,
                        &posinset_s,
                        &setsize_s,
                        &depth_s,
                        Vec::new(),
                        vec![
                            branch_control(
                                state,
                                is_selected,
                                node.is_disabled(),
                                Vec::new(),
                                vec![
                                    branch_indicator(state, Vec::new(), Vec::new()),
                                    branch_text(Vec::new(), vec![text(node.label())]),
                                ],
                            ),
                            branch_content(
                                state,
                                Vec::new(),
                                vec![
                                    branch_indent_guide(Vec::new(), Vec::new()),
                                    root(Vec::new(), child_nodes),
                                ],
                            ),
                        ],
                    )
                } else {
                    item(
                        node.value(),
                        is_selected,
                        node.is_disabled(),
                        &level_s,
                        &posinset_s,
                        &setsize_s,
                        &depth_s,
                        Vec::new(),
                        vec![
                            item_indicator(is_selected, Vec::new(), Vec::new()),
                            item_text(Vec::new(), vec![text(node.label())]),
                        ],
                    )
                }
            })
            .collect()
    }
}

impl Component for TreeView {
    type Action = TreeViewAction;

    fn update(&mut self, action: TreeViewAction) {
        match action {
            TreeViewAction::Expand(value) => {
                self.expanded.update(MultiSelectAction::Select(value));
            }
            TreeViewAction::Collapse(value) => {
                self.expanded.update(MultiSelectAction::Deselect(value));
            }
            TreeViewAction::ToggleBranch(value) => {
                self.expanded.update(MultiSelectAction::Toggle(value));
            }
            TreeViewAction::Select(value) => {
                self.selected.update(SingleSelectAction::Select(value));
            }
            TreeViewAction::Deselect => {
                self.selected.update(SingleSelectAction::Deselect);
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー（root、children
    /// 空）。[`crate::accordion::Accordion::view`] と同じ位置付けであり、
    /// 公開 UI としての利用は想定しない（実際の UI 構築は
    /// [`Self::render_nodes`] を呼び出し側が使う）。
    fn view(&self) -> Node {
        root(Vec::new(), Vec::new())
    }

    fn decode_action(name: &str, payload: &str) -> Option<TreeViewAction> {
        match name {
            "expand" => Some(TreeViewAction::Expand(payload.to_string())),
            "collapse" => Some(TreeViewAction::Collapse(payload.to_string())),
            "toggle" => Some(TreeViewAction::ToggleBranch(payload.to_string())),
            "select" => Some(TreeViewAction::Select(payload.to_string())),
            "deselect" => Some(TreeViewAction::Deselect),
            _ => None,
        }
    }
}

impl Hydrate for TreeView {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        // MultiSelect::hydration_attrs() はフィールド名 "selected" で
        // "data-hydrate-selected" を返す。SingleSelect の同名属性と衝突する
        // ため、展開集合側のみ Self::FIELD_EXPANDED（"expanded"）へ書き換える
        // （モジュール doc §hydration フィールド名 参照。MultiSelect 自体の
        // エンコード処理（codec::encode_list）は再実装しない）。
        let expanded_field = format!(
            "{}selected",
            fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX
        );
        let expanded_renamed_field = format!(
            "{}{}",
            fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX,
            Self::FIELD_EXPANDED
        );
        let mut attrs: Vec<(String, String)> = self
            .expanded
            .hydration_attrs()
            .into_iter()
            .map(|(k, v)| {
                if k == expanded_field {
                    (expanded_renamed_field.clone(), v)
                } else {
                    (k, v)
                }
            })
            .collect();
        attrs.extend(self.selected.hydration_attrs());
        attrs
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let expanded_renamed_field = format!(
            "{}{}",
            fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX,
            Self::FIELD_EXPANDED
        );
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == expanded_renamed_field)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| HydrateError::MissingAttr(expanded_renamed_field.clone()))?;

        // MultiSelect::from_hydration_attrs の既存検証（重複値の fail-closed
        // 拒否）を再実装せず再利用するため、フィールド名を "selected" へ
        // 戻した一時 Vec を経由して呼び出す（上記 hydration_attrs のリネームの
        // 逆写像）。
        let renamed_back = vec![(
            format!(
                "{}selected",
                fandhe_frontend_interactive::HYDRATE_ATTR_PREFIX
            ),
            raw,
        )];
        let expanded = MultiSelect::from_hydration_attrs(&renamed_back)?;
        let selected = SingleSelect::from_hydration_attrs(attrs)?;

        Ok(Self { expanded, selected })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::{codec, dispatch};

    // --- 各パーツの data-scope/data-part/ARIA 出力 ---

    #[test]
    fn root_outputs_scope_and_part_only() {
        let html = render(&root(vec![], vec![]));
        assert_eq!(
            html,
            r#"<div data-scope="tree-view" data-part="root"></div>"#
        );
    }

    #[test]
    fn tree_outputs_role_tree() {
        let html = render(&tree(None, None, vec![], vec![]));
        assert!(html.contains(r#"role="tree""#));
        assert!(!html.contains("aria-label"));
        assert!(!html.contains("aria-labelledby"));
    }

    #[test]
    fn tree_aria_label_and_labelledby_are_optional() {
        let html = render(&tree(Some("File tree"), None, vec![], vec![]));
        assert!(html.contains(r#"aria-label="File tree""#));

        let html2 = render(&tree(None, Some("tree-label"), vec![], vec![]));
        assert!(html2.contains(r#"aria-labelledby="tree-label""#));
    }

    #[test]
    fn branch_outputs_role_treeitem_and_aria_attrs() {
        let html = render(&branch(
            OpenState::Closed,
            "src",
            false,
            false,
            "1",
            "1",
            "2",
            "0",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="treeitem""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(html.contains(r#"aria-selected="false""#));
        assert!(html.contains(r#"aria-level="1""#));
        assert!(html.contains(r#"aria-posinset="1""#));
        assert!(html.contains(r#"aria-setsize="2""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-value="src""#));
        assert!(html.contains(r#"data-depth="0""#));
        assert!(!html.contains("data-selected"));
        assert!(!html.contains("data-disabled"));
    }

    #[test]
    fn branch_open_and_selected_adds_expected_attrs() {
        let html = render(&branch(
            OpenState::Open,
            "src",
            true,
            true,
            "2",
            "1",
            "1",
            "1",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-expanded="true""#));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-selected="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn branch_control_outputs_scope_part_and_state() {
        let html = render(&branch_control(
            OpenState::Open,
            false,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="branch-control""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(!html.contains("data-selected"));
        assert!(!html.contains("role="));
        assert!(!html.contains("<button"));
    }

    #[test]
    fn branch_control_reflects_selected_state() {
        // #798 Cursor Bugbot 指摘: 選択強調 CSS
        // （`[data-part="branch-control"][data-selected]`）が反応するには
        // branch-control 自身にも data-selected を反映する必要がある。
        let html = render(&branch_control(
            OpenState::Open,
            true,
            false,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="branch-control""#));
        assert!(html.contains(r#"data-selected="""#));
    }

    #[test]
    fn branch_indicator_outputs_scope_part_and_state() {
        let html = render(&branch_indicator(OpenState::Closed, vec![], vec![]));
        assert!(html.contains(r#"data-part="branch-indicator""#));
        assert!(html.contains(r#"data-state="closed""#));
    }

    #[test]
    fn branch_text_outputs_scope_and_part_only() {
        let html = render(&branch_text(vec![], vec![text("src")]));
        assert!(html.contains(r#"data-part="branch-text""#));
        assert!(html.contains("src"));
    }

    #[test]
    fn branch_content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&branch_content(OpenState::Closed, vec![], vec![]));
        assert!(closed.contains(r#"role="group""#));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&branch_content(OpenState::Open, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn branch_indent_guide_outputs_scope_and_part_only() {
        let html = render(&branch_indent_guide(vec![], vec![]));
        assert_eq!(
            html,
            r#"<div data-scope="tree-view" data-part="branch-indent-guide"></div>"#
        );
    }

    #[test]
    fn item_outputs_role_treeitem_without_aria_expanded() {
        let html = render(&item(
            "file.txt",
            false,
            false,
            "2",
            "1",
            "1",
            "1",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="treeitem""#));
        assert!(!html.contains("aria-expanded"));
        assert!(html.contains(r#"aria-selected="false""#));
        assert!(html.contains(r#"aria-level="2""#));
        assert!(html.contains(r#"aria-posinset="1""#));
        assert!(html.contains(r#"aria-setsize="1""#));
        assert!(html.contains(r#"data-value="file.txt""#));
        assert!(html.contains(r#"data-depth="1""#));
        assert!(!html.contains("data-selected"));
        assert!(!html.contains("data-state"));
    }

    #[test]
    fn item_selected_and_disabled_adds_expected_attrs() {
        let html = render(&item(
            "file.txt",
            true,
            true,
            "1",
            "1",
            "1",
            "0",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-selected="true""#));
        assert!(html.contains(r#"data-selected="""#));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn item_text_outputs_scope_and_part_only() {
        let html = render(&item_text(vec![], vec![text("file.txt")]));
        assert!(html.contains(r#"data-part="item-text""#));
        assert!(html.contains("file.txt"));
    }

    #[test]
    fn item_indicator_reflects_selection() {
        let unselected = render(&item_indicator(false, vec![], vec![]));
        assert!(!unselected.contains("data-selected"));

        let selected = render(&item_indicator(true, vec![], vec![]));
        assert!(selected.contains(r#"data-selected="""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&item(
            "a",
            false,
            false,
            "1",
            "1",
            "1",
            "0",
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tree-view""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(!html.contains("attacker"));
    }

    // --- TreeNode: ビルダ API ---

    #[test]
    fn tree_node_default_is_leaf() {
        let node = TreeNode::new("a", "A");
        assert_eq!(node.value(), "a");
        assert_eq!(node.label(), "A");
        assert!(node.children().is_empty());
        assert!(!node.is_branch());
        assert!(!node.is_disabled());
    }

    #[test]
    fn tree_node_with_children_becomes_branch() {
        let node = TreeNode::new("src", "src").with_children(vec![TreeNode::new("a.rs", "a.rs")]);
        assert!(node.is_branch());
        assert_eq!(node.children().len(), 1);
    }

    #[test]
    fn tree_node_disabled_builder() {
        let node = TreeNode::new("a", "A").disabled(true);
        assert!(node.is_disabled());
    }

    // --- TreeView: dispatch 統合 ---

    #[test]
    fn tree_view_default_is_all_collapsed_and_unselected() {
        let t = TreeView::default();
        assert!(!t.is_expanded("src"));
        assert_eq!(t.selected(), None);
    }

    #[test]
    fn tree_view_dispatch_expand_collapse_toggle() {
        let mut t = TreeView::default();

        assert!(dispatch(&mut t, "expand", "src"));
        assert!(t.is_expanded("src"));

        assert!(dispatch(&mut t, "collapse", "src"));
        assert!(!t.is_expanded("src"));

        assert!(dispatch(&mut t, "toggle", "src"));
        assert!(t.is_expanded("src"));
        assert!(dispatch(&mut t, "toggle", "src"));
        assert!(!t.is_expanded("src"));
    }

    #[test]
    fn tree_view_dispatch_expand_multiple_branches_simultaneously() {
        let mut t = TreeView::default();
        dispatch(&mut t, "expand", "src");
        dispatch(&mut t, "expand", "docs");
        assert!(t.is_expanded("src"));
        assert!(t.is_expanded("docs"));
    }

    #[test]
    fn tree_view_dispatch_select_deselect() {
        let mut t = TreeView::default();

        assert!(dispatch(&mut t, "select", "a.rs"));
        assert_eq!(t.selected(), Some("a.rs"));

        assert!(dispatch(&mut t, "select", "b.rs"));
        assert_eq!(t.selected(), Some("b.rs"));

        assert!(dispatch(&mut t, "deselect", ""));
        assert_eq!(t.selected(), None);
    }

    #[test]
    fn tree_view_dispatch_expand_and_select_are_independent() {
        // 展開集合と選択値は別々の状態機械であり、片方の操作がもう片方へ
        // 波及しないことを固定する（合成の正しさの回帰）。
        let mut t = TreeView::default();
        dispatch(&mut t, "expand", "src");
        dispatch(&mut t, "select", "a.rs");

        assert!(t.is_expanded("src"));
        assert_eq!(t.selected(), Some("a.rs"));

        dispatch(&mut t, "collapse", "src");
        assert!(!t.is_expanded("src"));
        assert_eq!(t.selected(), Some("a.rs"));
    }

    #[test]
    fn tree_view_dispatch_ignores_unknown_action() {
        let mut t = TreeView::default();
        dispatch(&mut t, "expand", "src");
        assert!(!dispatch(&mut t, "no_such_action", "src"));
        assert!(t.is_expanded("src"));
    }

    // --- TreeView: SSR 状態なし初期描画 ---

    #[test]
    fn tree_view_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&TreeView::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- TreeView: hydration 経路 ---

    #[test]
    fn tree_view_hydration_round_trip_expanded_and_selected() {
        use fandhe_frontend_interactive::render_for_hydration;

        let mut t = TreeView::default();
        dispatch(&mut t, "expand", "src");
        dispatch(&mut t, "expand", "docs");
        dispatch(&mut t, "select", "a.rs");

        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains("data-hydrate-expanded="));
        assert!(rendered.contains("data-hydrate-selected="));
        assert!(rendered.contains("src"));
        assert!(rendered.contains("docs"));
        assert!(rendered.contains("a.rs"));

        let restored = TreeView::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn tree_view_hydration_round_trip_default() {
        let t = TreeView::default();
        let restored = TreeView::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn tree_view_from_hydration_attrs_missing_expanded_attr() {
        // "data-hydrate-selected"（SingleSelect 側）のみを与え、
        // "data-hydrate-expanded" を欠落させた改ざん入力は MissingAttr で
        // 拒否される（panic しない）。
        let attrs = vec![("data-hydrate-selected".to_string(), codec::encode_list(&[]))];
        let err = TreeView::from_hydration_attrs(&attrs).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-expanded".to_string())
        );
    }

    #[test]
    fn tree_view_from_hydration_attrs_missing_selected_attr() {
        let attrs = vec![("data-hydrate-expanded".to_string(), codec::encode_list(&[]))];
        let err = TreeView::from_hydration_attrs(&attrs).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-selected".to_string())
        );
    }

    #[test]
    fn tree_view_from_hydration_attrs_rejects_duplicate_expanded_values() {
        // MultiSelect::from_hydration_attrs の重複値拒否ロジックが
        // フィールド名リネーム越しでも機能することを固定する。
        let bogus = codec::encode_list(&["src".to_string(), "src".to_string()]);
        let attrs = vec![
            ("data-hydrate-expanded".to_string(), bogus),
            ("data-hydrate-selected".to_string(), codec::encode_list(&[])),
        ];
        let err = TreeView::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn tree_view_from_hydration_attrs_rejects_multiple_selected_values() {
        let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
        let attrs = vec![
            ("data-hydrate-expanded".to_string(), codec::encode_list(&[])),
            ("data-hydrate-selected".to_string(), bogus),
        ];
        let err = TreeView::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn tree_view_view_root_is_element_for_render_for_hydration() {
        let node = TreeView::default().view();
        assert!(matches!(node, Node::Element { .. }));
    }

    // --- TreeView::render_nodes: ネスト深さの ARIA 出力 ---

    fn sample_tree() -> Vec<TreeNode> {
        vec![
            TreeNode::new("src", "src").with_children(vec![
                TreeNode::new("a.rs", "a.rs"),
                TreeNode::new("nested", "nested")
                    .with_children(vec![TreeNode::new("b.rs", "b.rs")]),
            ]),
            TreeNode::new("readme.md", "readme.md"),
        ]
    }

    #[test]
    fn render_nodes_top_level_posinset_and_setsize() {
        let t = TreeView::default();
        let nodes = t.render_nodes(&sample_tree());
        assert_eq!(nodes.len(), 2);

        let html0 = render(&nodes[0]);
        assert!(html0.contains(r#"aria-level="1""#));
        assert!(html0.contains(r#"aria-posinset="1""#));
        assert!(html0.contains(r#"aria-setsize="2""#));
        assert!(html0.contains(r#"data-depth="0""#));

        let html1 = render(&nodes[1]);
        assert!(html1.contains(r#"aria-level="1""#));
        assert!(html1.contains(r#"aria-posinset="2""#));
        assert!(html1.contains(r#"aria-setsize="2""#));
    }

    #[test]
    fn render_nodes_nested_branch_has_incremented_level_and_depth() {
        let t = TreeView::default();
        let nodes = t.render_nodes(&sample_tree());
        let html0 = render(&nodes[0]);

        // "src" ブランチの子（a.rs, nested）は aria-level="2"/data-depth="1"、
        // 兄弟数 2 の aria-setsize="2" を持つ。
        assert!(html0.contains(r#"aria-level="2""#));
        assert!(html0.contains(r#"data-depth="1""#));

        // さらに 1 段深い "nested" の子（b.rs）は aria-level="3"/data-depth="2"、
        // 兄弟数 1 の aria-setsize="1" を持つ。
        assert!(html0.contains(r#"aria-level="3""#));
        assert!(html0.contains(r#"data-depth="2""#));
    }

    #[test]
    fn render_nodes_branch_reflects_expanded_state() {
        // ネストした未展開ブランチ（"nested"）の hidden と、対象ブランチ
        // （"src"）自身の hidden を混同しないよう、子を持たない単純な木で
        // 検証する（サンプル木は他テストで深さ 3 段のネスト検証に使う）。
        let mut t = TreeView::default();
        dispatch(&mut t, "expand", "src");
        let nodes = t.render_nodes(&[
            TreeNode::new("src", "src").with_children(vec![TreeNode::new("a.rs", "a.rs")])
        ]);
        let html0 = render(&nodes[0]);
        assert!(html0.contains(r#"aria-expanded="true""#));
        assert!(html0.contains(r#"data-state="open""#));
        // 展開中は branch-content に hidden 属性が付かない。
        assert!(!html0.contains(r#"hidden="""#));
    }

    #[test]
    fn render_nodes_branch_collapsed_by_default_hides_content() {
        let t = TreeView::default();
        let nodes = t.render_nodes(&sample_tree());
        let html0 = render(&nodes[0]);
        assert!(html0.contains(r#"aria-expanded="false""#));
        assert!(html0.contains(r#"hidden="""#));
    }

    #[test]
    fn render_nodes_leaf_has_no_aria_expanded() {
        let t = TreeView::default();
        let nodes = t.render_nodes(&sample_tree());
        let html1 = render(&nodes[1]);
        assert!(!html1.contains("aria-expanded"));
        assert!(html1.contains(r#"data-value="readme.md""#));
    }

    #[test]
    fn render_nodes_reflects_selected_state() {
        let mut t = TreeView::default();
        dispatch(&mut t, "select", "readme.md");
        let nodes = t.render_nodes(&sample_tree());
        let html1 = render(&nodes[1]);
        assert!(html1.contains(r#"aria-selected="true""#));
        assert!(html1.contains(r#"data-selected="""#));
    }

    #[test]
    fn render_nodes_label_text_is_included() {
        let t = TreeView::default();
        let nodes = t.render_nodes(&sample_tree());
        let html0 = render(&nodes[0]);
        assert!(html0.contains("src"));
        assert!(html0.contains("a.rs"));
        assert!(html0.contains("nested"));
        assert!(html0.contains("b.rs"));
    }

    #[test]
    fn render_nodes_disabled_branch_and_leaf() {
        let nodes = vec![TreeNode::new("src", "src")
            .disabled(true)
            .with_children(vec![TreeNode::new("a.rs", "a.rs").disabled(true)])];
        let t = TreeView::default();
        let rendered = t.render_nodes(&nodes);
        let html = render(&rendered[0]);
        assert!(html.contains(r#"data-disabled="""#));
    }
}
