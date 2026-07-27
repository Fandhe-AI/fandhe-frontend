//! headless-ui（`fandhe-frontend-headless-ui`）dispatch の DOM イベント配線基盤
//! （イシュー #580）。
//!
//! # 背景・呼び出し文脈
//!
//! headless-ui の状態機械（`state::Disclosure`/`state::SingleSelect` および
//! それらを埋め込む `Collapsible`/`Dialog`/`Popover`/`Tooltip`/`Menu`/
//! `RadioGroup`/`Select` 等）は `fandhe_frontend_interactive::dispatch`
//! （文字列アクション `"open"`/`"close"`/`"toggle"`/`"select"` 等）で駆動
//! できるが、headless-ui 自身は DOM イベントから dispatch へ接続する配線を
//! 持たない（headless-ui 側の各コンポーネント doc が「クリック/キーボード
//! 操作の実挙動は wasm 層の責務」と明記している設計上の分離）。本モジュールは
//! その配線を提供する。
//!
//! 既存の [`crate::events::wire_events`] は `data-action`/`data-payload`
//! 属性ベースの委譲であり、headless-ui のマークアップは `data-action` を
//! 出力しない（`data-scope`/`data-part` の anatomy セレクタが正）。そのため
//! 本モジュールは (`data-scope`, `data-part`) から文字列アクションへの
//! **静的マッピング表**（[`action_for_part`]）を持つ、`events.rs` とは独立
//! した配線層として実装する。
//!
//! # 設計（2 層構成、`events.rs` と同型）
//!
//! - 純粋ロジック層（[`PartRef`]/[`action_for_part`]/[`action_from_parts`]）は
//!   web-sys に依存せず、native の `cargo test` で検証できる。
//! - 配線層（[`wire_headless_events`]/[`wire_headless_component`]）のみ
//!   `#[cfg(target_arch = "wasm32")]` でゲートし、native ビルドへ web-sys
//!   依存を混入させない。
//!
//! # fail-closed 契約（受け入れ条件 3）
//!
//! - マッピング表にない (scope, part) の組は `None`（no-op）。
//! - select 系 part（`data-value` を要求する行）で `data-value` が欠落して
//!   いる場合は `None`（改ざん・欠損入力を dispatch へ流さない）。
//! - part 要素（または祖先の part）に `data-disabled` が付与されている
//!   場合は `None`。[`action_from_parts`] はクリック位置から根方向へ並べた
//!   part 列（[`PartRef`]）**全体**を見て判定する。祖先 part（例:
//!   `radio-group`/`collapsible` の root）が `disabled` の場合、その内側に
//!   ある enabled な子 part（`item`/`trigger` 等）へのマッチが
//!   `find_map`（内側優先）で先に成立してしまわないよう、列内のいずれか 1
//!   要素でも `disabled` なら全体を `None` とする（fail-closed、イシュー
//!   #580 PR #611 Bugbot 指摘の修正）。
//! - 未知アクション名は `fandhe_frontend_interactive::dispatch`/
//!   `Component::decode_action` 側の既存契約（不変条件 4）により no-op と
//!   なる（本モジュールの fail-closed と合わせた二重の安全網）。
//!
//! # payload の扱い（REQ-1 との関係）
//!
//! `data-value` はクライアント側で改ざんされうる入力であり、本モジュールは
//! これを HTML/セレクタとして一切解釈せず、文字列のまま
//! [`crate::events::ActionRef::payload`] へ渡す。再描画時のエスケープは
//! 呼び出し側が経由する `fandhe_frontend_core::render`（既定エスケープ）が
//! 担う（`crate::events` の既存契約と同一）。

use crate::events::ActionRef;

/// (scope, part) → 文字列アクションの静的マッピング表 1 行。
struct MappingRow {
    scope: &'static str,
    part: &'static str,
    action: &'static str,
    /// `true` のとき `PartRef::value`（`data-value`）を payload として使う。
    /// `data-value` が欠落している場合は fail-closed で `None`（マッピング
    /// 不成立）とする。`false` の行は payload を常に空文字列とする。
    requires_value: bool,
}

/// headless-ui 全コンポーネント共通の (scope, part) → アクションマッピング表。
///
/// `docs/design/wasm-full-architecture.md` の headless 配線節（イシュー #580）
/// が正本の対応表であり、本配列はその実装である。表にない組は
/// [`action_for_part`] が `None` を返す（fail-closed）。
const MAPPING_TABLE: &[MappingRow] = &[
    // Disclosure 系（Collapsible/Dialog/Popover/Tooltip/Menu）の trigger は
    // すべて "toggle"（Disclosure/DisclosureAction の共通語彙）。
    MappingRow {
        scope: "collapsible",
        part: "trigger",
        action: "toggle",
        requires_value: false,
    },
    MappingRow {
        scope: "dialog",
        part: "trigger",
        action: "toggle",
        requires_value: false,
    },
    MappingRow {
        scope: "popover",
        part: "trigger",
        action: "toggle",
        requires_value: false,
    },
    MappingRow {
        scope: "tooltip",
        part: "trigger",
        action: "toggle",
        requires_value: false,
    },
    MappingRow {
        scope: "menu",
        part: "trigger",
        action: "toggle",
        requires_value: false,
    },
    // trigger-item（サブメニューを開く menu item、`Menu::trigger_item`）も
    // Disclosure の "toggle" を dispatch する。サブメニューは「子 Menu
    // インスタンス由来の trigger-item/positioner/content を親 content 内に
    // 入れ子配置する」契約（`crates/headless-ui/src/menu.rs` モジュール doc）
    // であり、trigger-item も `data-scope="menu"` を持つため、この行を欠くと
    // マウスクリックでの開閉が no-op になる（キーボード操作 ArrowRight/
    // ArrowLeft が合成する `click()` もこの経路を辿るため、同じく no-op に
    // なっていた。イシュー #662 PR #674 Bugbot 指摘の修正）。
    MappingRow {
        scope: "menu",
        part: "trigger-item",
        action: "toggle",
        requires_value: false,
    },
    // close-trigger（Dialog/Popover）は "close"。
    MappingRow {
        scope: "dialog",
        part: "close-trigger",
        action: "close",
        requires_value: false,
    },
    MappingRow {
        scope: "popover",
        part: "close-trigger",
        action: "close",
        requires_value: false,
    },
    // select 系（Tabs/RadioGroup/Select）の項目クリックは "select"（value 必須）。
    MappingRow {
        scope: "tabs",
        part: "trigger",
        action: "select",
        requires_value: true,
    },
    MappingRow {
        scope: "radio-group",
        part: "item",
        action: "select",
        requires_value: true,
    },
    MappingRow {
        scope: "select",
        part: "item",
        action: "select",
        requires_value: true,
    },
    // Select 固有: trigger は listbox 開閉トグル、clear-trigger は選択解除。
    MappingRow {
        scope: "select",
        part: "trigger",
        action: "toggle",
        requires_value: false,
    },
    MappingRow {
        scope: "select",
        part: "clear-trigger",
        action: "deselect",
        requires_value: false,
    },
    // SignaturePad（イシュー #843）: ClearTrigger は全ストローク削除。
    // ポインタ座標収集による描画（"add-stroke"）は本汎用マッピングでは
    // 扱わず `crate::headless_signature_pad::wire_stroke_collector` が
    // 専用配線する（クリックではないため）。
    MappingRow {
        scope: "signature-pad",
        part: "clear-trigger",
        action: "clear",
        requires_value: false,
    },
    // Combobox（イシュー #1071）: trigger は listbox 開閉トグル
    // （`ComboboxAction::Toggle`）。マウスクリックと `crate::keynav`
    // が合成する `HtmlElement::click()`（Arrow キーによる open/close）の
    // 双方がこの行を経由する。#662 の `menu`/`trigger-item` 欠落是正と
    // 同型の整備であり、この行を欠くと keynav の click 合成が no-op に
    // なる。
    MappingRow {
        scope: "combobox",
        part: "trigger",
        action: "toggle",
        requires_value: false,
    },
    // item クリック（マウス・keynav の Enter/highlight click 合成の両方）は
    // "select"（value 必須、`ComboboxAction::Select`。ark-ui の
    // `closeOnSelect` 既定に準拠し選択と同時に listbox を閉じる、
    // `crates/headless-ui/src/combobox.rs::Combobox::update` 参照）。
    MappingRow {
        scope: "combobox",
        part: "item",
        action: "select",
        requires_value: true,
    },
    // clear-trigger は選択解除（`ComboboxAction::Clear`。`Select` と異なり
    // `combobox::clear_trigger` は SSR 出力済みだが keynav・マウスクリック
    // いずれの経路もこの行が無いと no-op のままだった、Select の
    // `clear-trigger`→`"deselect"` 整備と同種）。
    MappingRow {
        scope: "combobox",
        part: "clear-trigger",
        action: "clear",
        requires_value: false,
    },
    // ToggleGroup（イシュー #1075）: item クリック（マウス・ネイティブ
    // Enter/Space の双方。`crate::keynav` はフォーカス移動のみを行い決定は
    // claim しない）は "toggle"（value 必須、`ToggleGroup`/
    // `MultiToggleGroup` の `decode_action` はいずれも "toggle" のみを
    // 受理する。`toggle_group::item` は `data-value` を常時出力する）。
    // この行を欠くとマウス・キーボードいずれの押下も no-op のままになる
    // （#662 の `menu`/`trigger-item` 欠落是正と同型）。
    //
    // NavigationMenu の `trigger` 行は本イシューでは追加しない:
    // `navigation_menu::trigger` は `data-value` を出力せず、
    // `NavigationMenu::decode_action`（`SingleSelect` へ全委譲）は payload に
    // 項目値を要求するため、`requires_value: true` 行は常に fail-closed
    // （`None`）になり、`requires_value: false` 行は
    // `SingleSelectAction::Toggle("")` という誤った値をトグルしてしまう
    // （`crates/wasm-full/src/keynav.rs` モジュール doc §NavigationMenu
    // 「既知のギャップ」参照。headless-ui 側の SSR 出力追加が前提となる別
    // イシュー）。
    MappingRow {
        scope: "toggle-group",
        part: "item",
        action: "toggle",
        requires_value: true,
    },
    // TreeView（イシュー #1072、`crates/headless-ui/src/tree_view.rs`）:
    // ブランチのクリック対象は要約行（`branch-control`、自身に `data-value`
    // を持たない）だが、マッピング表には無いためクリック解決は
    // `action_from_parts` の内側優先探索により祖先の `branch` へ falls
    // through する（`crate::keynav::wiring::synthesize_tree_click` が
    // `branch-control` を優先クリック先にする設計とセット）。この結果
    // ブランチノードは「選択」できず、Enter/Space は展開トグルとして働く
    // （`crate::keynav` モジュール doc §TreeView §帰結、意図的な仕様）。
    MappingRow {
        scope: "tree-view",
        part: "branch",
        action: "toggle",
        requires_value: true,
    },
    // 葉ノード（`item`）は展開状態を持たないため "select"（`SingleSelect`
    // 相当）。ブランチの `branch-control` クリックが上記 `branch` 行へ
    // フォールスルーするのと異なり、葉ノードは `branch-control` を持たない
    // ため `crate::keynav::wiring::synthesize_tree_click` は葉ノード自身へ
    // 直接 `click()` を合成する。
    MappingRow {
        scope: "tree-view",
        part: "item",
        action: "select",
        requires_value: true,
    },
    // Calendar（イシュー #1074）: prev-trigger/next-trigger は月移動
    // （`CalendarAction::PrevMonth`/`NextMonth`）。`crate::keynav` の
    // PageUp/PageDown が合成する `HtmlElement::click()`（モジュール doc
    // §Calendar 参照）はこの 2 行を経由して初めて dispatch へ到達する。
    // `day-trigger`→`"select"` の行は意図的に追加しない: `day_trigger` は
    // `data-value`（ISO 日付）を出力しないため、追加しても
    // `requires_value: true` により常に fail-closed で `None` になる
    // （`.claude/rules/out-of-scope-tracking.md` 対応の申し送り事項、
    // `crates/wasm-full/src/keynav.rs` モジュール doc §Calendar 参照）。
    MappingRow {
        scope: "calendar",
        part: "prev-trigger",
        action: "prev-month",
        requires_value: false,
    },
    MappingRow {
        scope: "calendar",
        part: "next-trigger",
        action: "next-month",
        requires_value: false,
    },
];

/// クリックされた要素（またはその祖先方向の 1 要素）の anatomy 属性を表す
/// 純粋データ型。`web_sys::Element` から独立しているため native の
/// `cargo test` で [`action_for_part`]/[`action_from_parts`] を検証できる
/// （配線層のみが `web_sys::Element` からこの型を組み立てる）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartRef {
    /// `data-scope` 属性値。
    pub scope: String,
    /// `data-part` 属性値。
    pub part: String,
    /// `data-value` 属性値（存在する場合）。改ざんされうるクライアント
    /// 入力としてそのまま保持し、HTML/セレクタとして解釈しない。
    pub value: Option<String>,
    /// この part 要素（または祖先の part 要素）に `data-disabled` 属性が
    /// 付与されているかどうか。`true` の場合は [`action_for_part`] が
    /// fail-closed で `None` を返す。
    pub disabled: bool,
}

/// (scope, part) の 1 段判定。マッピング表にない組・`data-value` 欠落・
/// `disabled` はいずれも `None`（fail-closed、受け入れ条件 3）。
#[must_use]
pub fn action_for_part(part: &PartRef) -> Option<ActionRef> {
    if part.disabled {
        return None;
    }
    let row = MAPPING_TABLE
        .iter()
        .find(|row| row.scope == part.scope && row.part == part.part)?;
    let payload = if row.requires_value {
        part.value.clone()?
    } else {
        String::new()
    };
    Some(ActionRef {
        action: row.action.to_string(),
        payload,
    })
}

/// クリック位置から根方向へ並べた part 列（内側優先）で最初に解決できた
/// アクションを返す。
///
/// `item-text`（Menu/Select の item 内側テキスト）等「マッピング表にない
/// 内側 part」をクリックしても、祖先の `item`/`trigger` で解決できるように
/// するための抽象。配線層（[`wire_headless_events`]）は event.target から
/// root 方向へ祖先を辿りながら `data-scope`/`data-part` を持つ要素ごとに
/// [`PartRef`] を構築し、本関数へ内側優先の順で渡す。
///
/// fail-closed（受け入れ条件 3）: 列内のいずれかの part（クリックされた
/// part 自身、または祖先方向の part）が `disabled` の場合、列全体を
/// `None` とする。[`action_for_part`] 単体は要素自身の `disabled` しか
/// 見ないため、`find_map` で内側から順に呼ぶだけでは「無効化された
/// root（例: `radio-group`/`collapsible` の root）の配下にある enabled な
/// 子 part（`item`/`trigger`）」がすり抜けてしまう（イシュー #580 PR #611
/// Bugbot 指摘）。祖先の disabled 伝播はここで一括判定し、
/// [`action_for_part`] 側の判定に依存しない。
///
/// `content` 境界を越えて祖先方向の `trigger`/`trigger-item` へ誤って解決
/// しない（イシュー #662 PR #674 Bugbot 指摘の修正）: サブメニューの
/// `trigger-item` は、`keynav.rs::wiring::resolve_submenu_content` の
/// フォールバック経路（`aria-controls` 欠落時に子孫 `[data-part="content"]`
/// を辿る）が示すとおり、自身の子孫として子 `Menu` インスタンスの
/// `content`（さらにその子孫の `item` 等）を持ちうる。この配置では
/// クリックされた `item`（`menu`/`item` はマッピング表に無く常に `None`）
/// から根方向へ辿る途中で `content` を通過し、その外側の祖先である
/// `trigger-item`（`menu`/`trigger-item` → `"toggle"`）に達してしまう。
/// `content` はマッピング表に存在せず単体では絶対にマッチしないため、
/// 「`content` 部分に達するまでに一致が見つからなければ、その `content`
/// を含む子 `Menu` インスタンスの外側（親 `trigger-item`/`trigger` 等）を
/// 誤ってこの click のアクションとして解決しない」よう、`content` を
/// 探索の境界として扱い列挙を打ち切る。これにより、直接 `trigger-item`
/// 自身をクリックした場合（`content` に達する前の最初の要素で即座に
/// マッチする）の挙動は変えず、`content` 配下の子孫クリックが親
/// `trigger-item` の `toggle` を奪う（かつ [`crate::headless::wiring`]
/// 側で `stop_propagation` されアイテム自身のクリック処理が握り潰される）
/// 事態のみを防ぐ（`stop_propagation` の呼び出し箇所は
/// [`wire_headless_events`] 参照）。
#[must_use]
pub fn action_from_parts(parts: &[PartRef]) -> Option<ActionRef> {
    if parts.iter().any(|part| part.disabled) {
        return None;
    }
    for part in parts {
        if let Some(action) = action_for_part(part) {
            return Some(action);
        }
        if part.part == "content" {
            return None;
        }
    }
    None
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`crate::events` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{action_from_parts, ActionRef, PartRef};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event};

    /// `data-scope`/`data-part` を持つ要素 1 個を [`PartRef`] へ変換する。
    /// `data-scope`/`data-part` のいずれかが無い要素は呼び出し元
    /// （[`collect_part_refs`]）が事前にフィルタする前提のため、本関数は
    /// 呼ばない（`data-scope`/`data-part` 双方の存在は上位で保証される）。
    fn to_part_ref(element: &Element) -> Option<PartRef> {
        let scope = element.get_attribute("data-scope")?;
        let part = element.get_attribute("data-part")?;
        let value = element.get_attribute("data-value");
        let disabled = element.has_attribute("data-disabled");
        Some(PartRef {
            scope,
            part,
            value,
            disabled,
        })
    }

    /// `target` から `root`（含む）まで祖先方向へ辿り、`data-scope`/
    /// `data-part` を両方持つ要素ごとに [`PartRef`] を構築する（内側優先の
    /// 順で返す）。`root` の子孫でない要素（`root` より外側へ抜けた場合）は
    /// 採用しない。
    ///
    /// `data-disabled` は「クリックされた part 自身」だけでなく祖先の part
    /// にも付与されうる（例: 無効化された `radio-group`/`collapsible` の
    /// root）。`to_part_ref` は要素自身の `has_attribute("data-disabled")`
    /// しか見ないため、本関数は祖先方向の各 part を個別の [`PartRef`] とし
    /// て列に積むだけに留める（祖先の disabled を子へ伝播させる集約判定は
    /// 行わない）。列全体を見て「いずれかの part が disabled なら全体を
    /// `None`」とする fail-closed 判定は呼び出し元の
    /// [`action_from_parts`] が担う（イシュー #580 PR #611 Bugbot 指摘の
    /// 修正。本関数側で祖先 disabled を子 `PartRef` に書き戻す実装ではない
    /// ことに注意）。
    fn collect_part_refs(root: &Element, target: &Element) -> Vec<PartRef> {
        let mut refs = Vec::new();
        let mut current = Some(target.clone());
        while let Some(element) = current {
            if !root.contains(Some(&element)) {
                break;
            }
            if let Some(part_ref) = to_part_ref(&element) {
                refs.push(part_ref);
            }
            if element == *root {
                break;
            }
            current = element.parent_element();
        }
        refs
    }

    /// ルート要素へ click 委譲リスナーを 1 回だけ登録する
    /// （`crate::events::wire_events` と同じ「マウント時 1 回」契約）。
    ///
    /// `event.target()` がテキストノード（headless-ui の trigger/item 内の
    /// テキスト等）の場合は `Node::parent_element()` で直近の親要素まで
    /// 遡ってから祖先探索を始める（`events.rs::wiring::wire_events` と同じ
    /// 対策）。祖先探索で得た part 列（内側優先）を [`action_from_parts`]
    /// へ渡し、解決できたアクションのみ `on_action` を呼ぶ。
    ///
    /// `Closure::forget` は本関数呼び出しにつき 1 回のみに限定する
    /// （`events.rs` と同じ判断。無制限リークによるメモリ枯渇 DoS を構造的
    /// に回避、A04 対策）。
    ///
    /// # ネストした headless root 間のクロスディスパッチ防止（イシュー #580
    /// PR #611 Bugbot 指摘の修正）
    ///
    /// 本関数はルート要素ごとに click リスナーを 1 個登録する委譲方式のため、
    /// 例えば `Select` を内包する `Dialog` のように headless root が入れ子に
    /// なっている場合、内側 root（Select）と外側 root（Dialog）の双方に
    /// リスナーが登録される。DOM の bubble 順は「内側 root → 外側 root」で
    /// あり、内側で `action_from_parts` がアクションを解決できた時点で
    /// `Event::stop_propagation` を呼ばないと、同一 click イベントが外側の
    /// リスナーまで届いてしまう。Disclosure（`toggle`/`close`）・
    /// SingleSelect（`select`）は語彙が全 headless コンポーネントで共有
    /// されているため、外側の `find_map` が内側の part を誤って自分の
    /// アクションとして解決し、Dialog/Select/Collapsible の組み合わせで
    /// 意図しない外側コンポーネントへの二重ディスパッチが発生しうる。
    ///
    /// 解決できた場合のみ `stop_propagation` を呼ぶことで、この二重
    /// ディスパッチを防ぐ。解決できなかった場合（fail-closed で `None`）は
    /// 伝播を止めない。`disabled` 起因で内側が `None` を返すケースでも
    /// 安全性は保たれる: 内側で disabled と判定された part は外側の
    /// `collect_part_refs` が辿る祖先列にも同じ要素として含まれるため、
    /// 外側の [`action_from_parts`] も同じ disabled 判定で `None` を返す
    /// （`crate::events::wire_events` とは異なり、本関数は
    /// `stop_immediate_propagation` ではなく `stop_propagation` を使う。
    /// 同一要素上の他リスナーは阻害せず、祖先の別 root リスナーへの伝播
    /// のみを止めるため）。
    pub fn wire_headless_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let on_action = std::rc::Rc::new(std::cell::RefCell::new(on_action));
        let click_root = root.clone();

        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let target_element: Element = match target.dyn_ref::<Element>() {
                Some(element) => element.clone(),
                None => {
                    let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                        return;
                    };
                    let Some(parent) = node.parent_element() else {
                        return;
                    };
                    parent
                }
            };
            let parts = collect_part_refs(&click_root, &target_element);
            if let Some(action_ref) = action_from_parts(&parts) {
                // ネストした外側 root（例: Dialog の外側リスナー）へ同一
                // click イベントが bubble して二重解決されるのを防ぐ
                // （上記関数 doc 参照）。
                event.stop_propagation();
                (on_action.borrow_mut())(action_ref);
            }
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_headless_events;

/// dispatch への橋渡し便宜 API（`#[cfg(target_arch = "wasm32")]`）。
///
/// [`wire_headless_events`] が解決したアクションを
/// `fandhe_frontend_interactive::dispatch` へ渡し、成功時のみ
/// `on_update(&C, &Element)` を呼ぶ。DOM への `data-state` 反映（再描画）は
/// 呼び出し側の責務であり、本関数は関知しない
/// （`crate::Runtime::wire` と同じ「配線は状態更新・再描画に結合しない」
/// 方針。本イシューは配線基盤の提供のみがスコープであり、束縛点更新との
/// 統合はスコープ外、モジュール doc 参照）。
///
/// `try_borrow_mut` が失敗する場合（イベントハンドラ内からの再入等）は
/// 状態変更・`on_update` 呼び出しのいずれも行わず no-op とする
/// （panic 回避、`.claude/rules/coding-rust.md`）。
#[cfg(target_arch = "wasm32")]
pub fn wire_headless_component<C: fandhe_frontend_interactive::Component + 'static>(
    root: web_sys::Element,
    component: std::rc::Rc<std::cell::RefCell<C>>,
    on_update: impl FnMut(&C, &web_sys::Element) + 'static,
) -> Result<(), wasm_bindgen::JsValue> {
    let on_update = std::rc::Rc::new(std::cell::RefCell::new(on_update));
    let wired_root = root.clone();

    wire_headless_events(root, move |action_ref: ActionRef| {
        let Ok(mut state) = component.try_borrow_mut() else {
            return;
        };
        let dispatched = fandhe_frontend_interactive::dispatch(
            &mut *state,
            &action_ref.action,
            &action_ref.payload,
        );
        if !dispatched {
            return;
        }
        (on_update.borrow_mut())(&state, &wired_root);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(scope: &str, part: &str, value: Option<&str>, disabled: bool) -> PartRef {
        PartRef {
            scope: scope.to_string(),
            part: part.to_string(),
            value: value.map(str::to_string),
            disabled,
        }
    }

    // --- Disclosure 系: trigger → "toggle" ---

    #[test]
    fn disclosure_trigger_scopes_map_to_toggle() {
        for scope in ["collapsible", "dialog", "popover", "tooltip", "menu"] {
            let action_ref = action_for_part(&part(scope, "trigger", None, false))
                .unwrap_or_else(|| panic!("{scope} trigger should map to an action"));
            assert_eq!(action_ref.action, "toggle");
            assert_eq!(action_ref.payload, "");
        }
    }

    // --- close-trigger（Dialog/Popover）→ "close" ---

    #[test]
    fn close_trigger_scopes_map_to_close() {
        for scope in ["dialog", "popover"] {
            let action_ref = action_for_part(&part(scope, "close-trigger", None, false))
                .unwrap_or_else(|| panic!("{scope} close-trigger should map to an action"));
            assert_eq!(action_ref.action, "close");
            assert_eq!(action_ref.payload, "");
        }
    }

    // --- select 系（Tabs/RadioGroup/Select item）→ "select"（value 必須） ---

    #[test]
    fn tabs_trigger_with_value_maps_to_select_with_payload() {
        let action_ref = action_for_part(&part("tabs", "trigger", Some("tab-1"), false)).unwrap();
        assert_eq!(action_ref.action, "select");
        assert_eq!(action_ref.payload, "tab-1");
    }

    #[test]
    fn radio_group_item_with_value_maps_to_select_with_payload() {
        let action_ref = action_for_part(&part("radio-group", "item", Some("red"), false)).unwrap();
        assert_eq!(action_ref.action, "select");
        assert_eq!(action_ref.payload, "red");
    }

    #[test]
    fn select_item_with_value_maps_to_select_with_payload() {
        let action_ref = action_for_part(&part("select", "item", Some("opt-1"), false)).unwrap();
        assert_eq!(action_ref.action, "select");
        assert_eq!(action_ref.payload, "opt-1");
    }

    #[test]
    fn select_trigger_maps_to_toggle() {
        let action_ref = action_for_part(&part("select", "trigger", None, false)).unwrap();
        assert_eq!(action_ref.action, "toggle");
        assert_eq!(action_ref.payload, "");
    }

    #[test]
    fn select_clear_trigger_maps_to_deselect() {
        let action_ref = action_for_part(&part("select", "clear-trigger", None, false)).unwrap();
        assert_eq!(action_ref.action, "deselect");
        assert_eq!(action_ref.payload, "");
    }

    // --- fail-closed: 表外・value 欠落・disabled は None ---

    #[test]
    fn unknown_scope_or_part_is_none() {
        assert_eq!(
            action_for_part(&part("unknown-scope", "trigger", None, false)),
            None
        );
        assert_eq!(
            action_for_part(&part("dialog", "unknown-part", None, false)),
            None
        );
        assert_eq!(action_for_part(&part("menu", "item", None, false)), None);
    }

    #[test]
    fn select_requiring_value_but_missing_is_none() {
        assert_eq!(action_for_part(&part("tabs", "trigger", None, false)), None);
        assert_eq!(
            action_for_part(&part("radio-group", "item", None, false)),
            None
        );
        assert_eq!(action_for_part(&part("select", "item", None, false)), None);
    }

    #[test]
    fn disabled_part_is_none_even_if_otherwise_valid() {
        assert_eq!(
            action_for_part(&part("collapsible", "trigger", None, true)),
            None
        );
        assert_eq!(
            action_for_part(&part("tabs", "trigger", Some("tab-1"), true)),
            None
        );
    }

    #[test]
    fn empty_value_is_distinct_from_missing_value_and_still_maps() {
        // 空文字列 value は「欠落」ではない（`Some("")`）。fail-closed の
        // 対象はあくまで `data-value` 属性自体の不在（`None`）である。
        let action_ref = action_for_part(&part("tabs", "trigger", Some(""), false)).unwrap();
        assert_eq!(action_ref.action, "select");
        assert_eq!(action_ref.payload, "");
    }

    // --- action_from_parts: 内側優先で祖先解決（item-text 等の表外パーツ対策） ---

    #[test]
    fn action_from_parts_resolves_via_ancestor_when_inner_part_is_unmapped() {
        // 内側（menu の item-text、表にない part）→ 外側（menu の item、表外）
        // → さらに外側（menu の trigger、表内）という祖先列を模す。
        let parts = vec![
            part("menu", "item-text", None, false),
            part("menu", "item", Some("item-1"), false),
            part("menu", "trigger", None, false),
        ];
        let action_ref = action_from_parts(&parts).unwrap();
        assert_eq!(action_ref.action, "toggle");
    }

    #[test]
    fn action_from_parts_picks_innermost_match_first() {
        let parts = vec![
            part("tabs", "trigger", Some("tab-1"), false),
            part("collapsible", "trigger", None, false),
        ];
        let action_ref = action_from_parts(&parts).unwrap();
        assert_eq!(action_ref.action, "select");
        assert_eq!(action_ref.payload, "tab-1");
    }

    #[test]
    fn action_from_parts_empty_list_is_none() {
        assert_eq!(action_from_parts(&[]), None);
    }

    #[test]
    fn action_from_parts_all_unmapped_is_none() {
        let parts = vec![
            part("unknown", "a", None, false),
            part("unknown", "b", None, false),
        ];
        assert_eq!(action_from_parts(&parts), None);
    }

    // --- fail-closed 回帰: 祖先 part が disabled の場合、enabled な内側の
    // 子 part（item/trigger）へのマッチは成立してはならない（イシュー #580
    // PR #611 Bugbot 指摘: root disabled でも find_map が内側の enabled な
    // マッチを先に見つけて素通りしていた）。

    #[test]
    fn action_from_parts_is_none_when_ancestor_root_is_disabled_radio_group() {
        // radio-group の root が disabled、内側の item 自体は enabled。
        let parts = vec![
            part("radio-group", "item", Some("red"), false),
            part("radio-group", "root", None, true),
        ];
        assert_eq!(action_from_parts(&parts), None);
    }

    #[test]
    fn action_from_parts_is_none_when_ancestor_root_is_disabled_collapsible() {
        // collapsible の root が disabled、内側の trigger 自体は enabled。
        let parts = vec![
            part("collapsible", "trigger", None, false),
            part("collapsible", "root", None, true),
        ];
        assert_eq!(action_from_parts(&parts), None);
    }

    #[test]
    fn action_from_parts_is_none_when_ancestor_root_is_disabled_select() {
        // select の root が disabled、内側の item（value あり）は enabled。
        let parts = vec![
            part("select", "item", Some("opt-1"), false),
            part("select", "root", None, true),
        ];
        assert_eq!(action_from_parts(&parts), None);
    }

    // --- REQ-1 経路一貫性回帰: マッピング結果の payload は HTML 解釈されず
    // dispatch → render の既定エスケープをそのまま経由する ---

    #[test]
    fn select_action_payload_with_xss_payload_is_escaped_on_render() {
        use fandhe_frontend_headless_ui::state::SingleSelect;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration};

        let payload = "\"><script>alert(1)</script>";
        let action_ref = action_for_part(&part("tabs", "trigger", Some(payload), false)).unwrap();

        let mut state = SingleSelect::default();
        assert!(dispatch(
            &mut state,
            &action_ref.action,
            &action_ref.payload
        ));

        let html = fandhe_frontend_core::render(&render_for_hydration(&state));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
