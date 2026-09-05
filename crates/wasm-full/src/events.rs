//! イベント委譲によるクリック/入力処理（TASK-11.2b、イシュー #75）。
//!
//! `fandhe-frontend-wasm-full` は REQ-11（WASM 完全方式）の既定実装であり、クライアントの
//! イベント処理・DOM 更新を JS グルーへ漏らさず safe Rust の範囲に閉じ込める
//! ことが目的である。本モジュールはその「イベント処理」区画を担当し、
//! DOM 更新（TASK-11.2c、#76）とは責務を分離する。`mount()`/`hydrate()` の
//! 既定実装化（TASK-11.2d、#77）は `wasm-full/src/lib.rs` の `Runtime` が
//! [`wire_events`] を呼び出す形で統合する（本モジュール自体は `Runtime` に
//! 依存しない）。
//!
//! # 設計（PoC-5 `wasm-runtime-split/wasm-full/src/lib.rs` の一般化）
//!
//! - ルート要素へ `click` / `input` / `change` リスナーを **マウント時に 1 回だけ**
//!   委譲登録する（[`wire_events`]）。再描画で子要素が入れ替わってもルートの
//!   リスナーは保持されるため、再描画のたびにリスナーを張り直す必要がない。
//! - リスナー登録は [`wasm_bindgen::closure::Closure::forget`] を click / input /
//!   change の 3 回に限定する（イシュー #1120 で change リスナーを追加、旧 2 回
//!   から改訂）。`forget` は safe API であり `unsafe` ブロックを要しないが、
//!   登録回数を定数個に抑えることで無制限リーク（メモリ枯渇 DoS）を構造的に
//!   回避する（A04: 安全でない設計への対策）。
//! - 属性からのアクション判定ロジック（[`action_from_click`] / [`action_from_input`]）
//!   は web-sys に依存しない純粋関数として切り出し、native の `cargo test` で
//!   検証できるようにする（配線層のみ `#[cfg(target_arch = "wasm32")]` でゲート
//!   し、native ビルドへ web-sys 依存を混入させない）。
//!
//! # 他クレート・他モジュールとの契約
//!
//! - [`ActionRef`] の `action` / `payload` は `fandhe_frontend_interactive::dispatch` の
//!   `name` / `payload` 引数仕様と一致する（`data-action` / `data-payload` 属性、
//!   `interactive/src/lib.rs` の `render_with_root_attrs` が出力する DOM 契約）。
//! - [`wire_events`] は状態更新（`dispatch`）・再描画（DOM 更新、#76 のスコープ）を
//!   直接呼ばず、すべて `on_action` コールバックへ委譲する。これにより本モジュールは
//!   `fandhe-frontend-interactive` の具象状態にも DOM 更新実装にも結合しない。
//! - 再描画出力は呼び出し側（#76/#77）が `fandhe_frontend_core::render()`（既定エスケープ）を
//!   経由させる前提であり、本モジュールは HTML 文字列を一切組み立てない
//!   （REQ-1 不変条件、`.claude/rules/coding-rust.md`）。

/// クリック/入力イベントから判定した「dispatch すべきアクション」への参照。
///
/// `action` は `data-action` 属性値、`payload` は `data-payload`（クリック時）
/// または入力値そのもの（input 時）に対応する。`fandhe_frontend_interactive::dispatch`
/// （`interactive/src/lib.rs`）の `(name, payload)` 引数へそのまま渡せる形。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRef {
    /// `data-action` 属性値（`fandhe_frontend_interactive::Component::decode_action` の `name`）。
    pub action: String,
    /// dispatch へ渡す payload（`fandhe_frontend_interactive::Component::decode_action` の `payload`）。
    pub payload: String,
}

/// イベントターゲット（祖先方向の探索結果を含む）の属性読み取り抽象。
///
/// `web_sys::Element` とテストダブルの双方が実装できるようにし、
/// [`action_from_click`] を native の `cargo test` で検証可能にする。
pub trait AttrSource {
    /// 指定名の属性値を読む。属性が存在しなければ `None`。
    fn attr(&self, name: &str) -> Option<String>;
}

/// click イベントのターゲット属性から dispatch すべきアクションを判定する。
///
/// `data-action` 属性が無ければフレームワーク管轄外のクリックとして `None`
/// を返す（安全側 no-op）。`data-payload` が無い場合は空文字列を payload
/// とする（`fandhe_frontend_interactive::Component::decode_action` 側が未知/不正な
/// payload を no-op として扱う契約に委ねる、不変条件 4）。
///
/// 配線層（[`wire_events`]）は `target.closest("[data-action]")`
/// （`web_sys::Element::closest`）で得た祖先要素を `target` として渡す想定
/// であり、本関数自体は「渡された 1 要素の属性を読む」責務のみを持つ
/// （祖先探索は DOM API 依存のため配線層の責務とし、ここでは扱わない）。
pub fn action_from_click<T: AttrSource>(target: &T) -> Option<ActionRef> {
    let action = target.attr("data-action")?;
    let payload = target.attr("data-payload").unwrap_or_default();
    Some(ActionRef { action, payload })
}

/// input イベントから draft 更新アクションを判定する。
///
/// 対象は `id="draft-input"` の入力欄のみ（`interactive/src/lib.rs` の
/// `render_with_root_attrs` が出力するフォーム入力欄の id 契約に合わせる）。
/// 他 id の input イベントはフレームワーク管轄外として `None` を返す。
///
/// イシュー #345 より前は `should_repaint: false` を返し、`set_inner_html`
/// 全置換によるフォーカス・キャレット破壊を避けるため input イベント後の
/// 再描画自体をスキップしていた（PoC-5 由来の対症療法）。#345 でイベント後
/// 更新が束縛点更新（`set_text_content`/`set_attribute`、変更フィールド数に
/// 比例する冪等な最小更新）へ置き換わったため、この特別扱いは不要になり
/// `should_repaint` フィールド自体を撤去した（`docs/design/dom-binding-update-design.md`
/// #345 実装確定節 §6.1）。キャレット位置の保持は `wasm-client::binding_dom`
/// の value プロパティ等値ガード（現在値と等しければ `set_value` を呼ばない）
/// が担う。
///
/// # レガシー経路（イシュー #1120）
///
/// `id` ハードコードは PoC-5 由来のデモ専用経路であり、`<select>`/`<textarea>`
/// への一般化ができない・利用者アプリが `draft-input` という id を偶然
/// 共有しない限り再利用できないという課題があった（イシュー #1120 の
/// フィードバック 2）。新規アプリは [`ACTION_INPUT_ATTR`]（`data-action-input`
/// 属性契約）を使う [`action_from_form_control`] を使用すべきであり、本関数は
/// 既存の `interactive::AppState` デモ・ブラウザテスト・
/// `docs/api/interactive-api.md` の id 契約との後方互換のためにのみ残す
/// （[`wiring::wire_events`] が `data-action-input` 属性がない場合のみ本関数へ
/// フォールバックする）。
pub fn action_from_input(id: &str, value: &str) -> Option<ActionRef> {
    if id != "draft-input" {
        return None;
    }
    Some(ActionRef {
        action: "set_draft".to_string(),
        payload: value.to_string(),
    })
}

/// input イベント配線で `data-action-input` 属性値をアクション名として使う
/// ことを示す属性契約（イシュー #1120）。
///
/// [`action_from_input`]（`id="draft-input"` ハードコード）の一般化。値
/// フォーム要素（`<input>`/`<textarea>`/`<select>`）に `data-action-input`
/// を付けるだけで input イベントを dispatch へ配線できる。
pub const ACTION_INPUT_ATTR: &str = "data-action-input";

/// change イベント配線で `data-action-change` 属性値をアクション名として
/// 使うことを示す属性契約（イシュー #1120）。
///
/// `<select>`/`<input type="checkbox">`/`<input type="radio">`/`<input
/// type="date">` 等、input イベントではなく change イベントで確定する
/// フォーム要素を dispatch へ配線するための契約（イシュー #1120 の
/// フィードバック 2「select / date / radio / checkbox の change を dispatch
/// に載せる公式経路がない」の解消）。
pub const ACTION_CHANGE_ATTR: &str = "data-action-change";

/// フォーム要素（`input`/`change` の対象）から `attr` 属性値をアクション名、
/// `value` を payload として [`ActionRef`] を組み立てる純粋関数
/// （イシュー #1120）。
///
/// `attr` は [`ACTION_INPUT_ATTR`] または [`ACTION_CHANGE_ATTR`] を渡す想定。
/// 属性が付いていない要素（フレームワーク管轄外の input/change）は `None`
/// を返す（安全側 no-op、[`action_from_click`] と同じ方針）。`value` の
/// 抽出（`checked`/`value` のどちらを使うか）は配線層
/// （[`wiring::extract_form_value`]）の責務であり、本関数は文字列化済みの
/// `value` を受け取るだけの薄いロジックに留める。
pub fn action_from_form_control<T: AttrSource>(
    target: &T,
    attr: &str,
    value: &str,
) -> Option<ActionRef> {
    let action = target.attr(attr)?;
    Some(ActionRef {
        action,
        payload: value.to_string(),
    })
}

/// クリック伝播境界における要素の分類（イシュー #1616 Bugbot/codex-review
/// 再指摘の是正で汎用化。`crates/wasm-full/src/keynav.rs` の RadioGroup
/// readonly クリック抑止・本モジュールの [`wiring::wire_events`] `data-action`
/// 解決の双方から共有する純粋ロジック。web-sys 非依存のため native
/// `cargo test` で検証できる（配線層のみが `web_sys::Element` からこの型を
/// 組み立てる））。
///
/// # 背景（HTML の label activation behavior）
///
/// HTML 仕様の `<label>` activation behavior は、click イベントの
/// `target` が「interactive content」
/// （<https://html.spec.whatwg.org/multipage/dom.html#interactive-content>
/// の一覧: `a[href]`/`audio[controls]`/`button`/`details`/`embed`/
/// `iframe`/`img[usemap]`/`input`〔`type=hidden` を除く〕/`label`/
/// `object[usemap]`/`select`/`summary`/`textarea`/`video[controls]`）の
/// ときは発火しない
/// （<https://html.spec.whatwg.org/multipage/forms.html#the-label-element>
/// の `run pre-click activation steps` は non-interactive-content 判定を
/// 前提とする）。一方、ARIA ロールや `tabindex`・`contenteditable` の
/// 付与は HTML 仕様上のこの判定に一切影響しない（ブラウザは role 属性を
/// 見て activation behavior の可否を変えない）ため、これらの「独自
/// ウィジェット」を click target にしても label の activation behavior
/// 自体は止まらず、呼び出し側が明示的に `preventDefault` する必要がある。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveBoundaryClass {
    /// (A) HTML interactive content。label の activation behavior
    /// 自体が発火しないため、呼び出し側は `preventDefault`/
    /// `stop_propagation` のいずれも行わず、ネイティブ動作・子要素の
    /// イベントハンドラを一切妨げてはならない（Bugbot 指摘: この分類を
    /// スキップして一律に抑止すると、`<a href>` のようなネイティブ要素の
    /// クリックまで祖先パーツの選択操作として抑止してしまう）。
    Html,
    /// (B) ARIA ロール（[`ARIA_INTERACTIVE_ROLES`]）・`tabindex`・
    /// `[contenteditable]`（`"false"` を除く）を持つ要素とその子孫
    /// （別コンポーネントの独自ウィジェット境界）。HTML 上は非
    /// interactive content のため label の activation behavior 自体は
    /// 止まらず、呼び出し側が `preventDefault` で明示的に阻止する必要が
    /// ある。ただし要素自身のイベントハンドラへイベントを届ける必要が
    /// あるため `stop_propagation` は行ってはならない（codex-review
    /// 指摘: capture フェーズで `stop_propagation` すると、
    /// `role="checkbox"` 等の子要素自身の click ハンドラへイベントが
    /// 到達できなくなる）。**`data-scope` が異なることだけを理由には
    /// 境界としない**（イシュー #1616 codex-review P1 再指摘: 装飾用の
    /// 別 scope 子孫、例えば `pre-styled-ui::button::close_button` 内の
    /// `data-scope="icon"` の `<svg>` は対話ロール・`tabindex`・
    /// `contenteditable` のいずれも持たないため `Ordinary` のまま）。
    Aria,
    /// (C) 上記以外（パーツ自身・装飾用の子孫・対話性を持たない別
    /// `data-scope` の子孫等）。選択操作・`data-action` 解決の一部として
    /// 扱ってよく、呼び出し側は `preventDefault`/`stop_propagation` の
    /// 双方で抑止してよい。
    Ordinary,
}

/// HTML 仕様上 label の activation behavior を止める「interactive
/// content」（<https://html.spec.whatwg.org/multipage/dom.html#interactive-content>）
/// のうち、属性条件なしで常に該当するタグ名（小文字比較）。
///
/// `a`（`has_href` 条件）・`input`（`type=hidden` 除外）・`audio`/`video`
/// （`controls` 条件）・`img`/`object`（`usemap` 条件）は別途
/// [`classify_interactive_boundary`] 内で個別に判定する（イシュー #1616
/// codex-review P1 再指摘の是正: 旧実装は `a[href]`/`button`/`input`/
/// `select`/`textarea`/`summary`/`details` のみを対象にしており、
/// `audio[controls]`/`video[controls]`/`img[usemap]`/`object[usemap]`/
/// `embed`/`iframe`/`label` が interactive content 一覧から漏れていた）。
/// `label` は「別コンポーネントの `<label>`」を Html 境界として扱うための
/// 追加（radio-group の `item` 自身が `<label>` として実装されるため、
/// 呼び出し側は item 自身・item 内の自パーツを本関数へ渡す前に除外する
/// 契約を維持する。除外しないと `item` 自身が毎回 Html 境界化し、
/// readonly クリック抑止の [`FullSuppression`](
/// crate::keynav::RadioGroupReadonlyClickOutcome::FullSuppression) 経路が
/// 消えてしまう）。
const HTML_UNCONDITIONAL_INTERACTIVE_TAGS: &[&str] = &[
    "button", "select", "textarea", "summary", "details", "label", "embed", "iframe",
];

/// ARIA ロールのうち、独自の対話ウィジェットとして扱うロール一覧。
/// `button`/`link` は native HTML 要素ではなく ARIA ロールとしての付与
/// （`role="button"`/`role="link"`）を想定しており、HTML interactive
/// content の判定（[`HTML_UNCONDITIONAL_INTERACTIVE_TAGS`] 等）とは独立に
/// 扱う（Bugbot 指摘: `role="button"`/`role="link"` は native
/// `<button>`/`<a href>` と異なり label activation behavior を止める
/// HTML 仕様上の根拠が無いため、`preventDefault` を要する (B) 分類に置く）。
const ARIA_INTERACTIVE_ROLES: &[&str] = &[
    "button",
    "link",
    "checkbox",
    "switch",
    "radio",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "tab",
    "slider",
    "spinbutton",
    "textbox",
    "searchbox",
    "combobox",
    "listbox",
    "treeitem",
];

/// WAI-ARIA のロール解決規則
/// (<https://www.w3.org/TR/wai-aria-1.2/#host_general_role>) における
/// 「認識できる非 abstract ロール」のうち、[`ARIA_INTERACTIVE_ROLES`] に
/// 含まれない代表的な非対話ロール一覧（イシュー #1616 codex-review P1
/// 再指摘・Bugbot Medium 指摘の是正）。`role` 属性はスペース区切りの
/// フォールバック候補列であり、UA は左から順に「認識できる」トークンを
/// 探し、そのトークンだけで対話性を確定する（未知トークンは読み飛ばして
/// 次の候補を見る）。本一覧に載っているトークンで確定した場合は
/// 「認識できたが非対話」を意味し、それ以降の候補は見ない
/// （例: `role="presentation button"` は `presentation` で確定し
/// `button` は見ない。網羅は必須ではないが、少なくとも UI コンポーネント
/// 層が実際に使う `presentation`/`none` 等の主要な非対話ロールは含める）。
const ARIA_KNOWN_NON_INTERACTIVE_ROLES: &[&str] = &[
    "alert",
    "alertdialog",
    "application",
    "article",
    "banner",
    "blockquote",
    "caption",
    "cell",
    "columnheader",
    "complementary",
    "contentinfo",
    "definition",
    "dialog",
    "directory",
    "document",
    "feed",
    "figure",
    "form",
    "generic",
    "grid",
    "group",
    "heading",
    "img",
    "list",
    "listitem",
    "log",
    "main",
    "marquee",
    "math",
    "meter",
    "navigation",
    "none",
    "note",
    "paragraph",
    "presentation",
    "progressbar",
    "radiogroup",
    "region",
    "row",
    "rowgroup",
    "rowheader",
    "scrollbar",
    "search",
    "separator",
    "status",
    "table",
    "tabpanel",
    "term",
    "timer",
    "toolbar",
    "tooltip",
    "tree",
    "treegrid",
];

/// `role` 属性値を WAI-ARIA のロール解決規則
/// (<https://www.w3.org/TR/wai-aria-1.2/#host_general_role>) に従って
/// 解決する（イシュー #1616 codex-review P1 再指摘・Bugbot Medium 指摘の
/// 是正: 旧実装は `role` 属性値全体を単一トークンとして
/// `ARIA_INTERACTIVE_ROLES.contains` していたため、複数ロールのフォール
/// バック列（`role="switch checkbox"`）や大文字小文字混在
/// （`role="Button"`）、前後の空白（`role=" button "`）を正しく解釈
/// できなかった）。
///
/// ASCII 空白区切りでトークン化し、各トークンを ASCII 小文字化した上で
/// 左から順に走査する。最初に [`ARIA_INTERACTIVE_ROLES`] に一致した
/// トークンがあれば `Some(true)`、[`ARIA_KNOWN_NON_INTERACTIVE_ROLES`] に
/// 一致したトークンがあれば `Some(false)` を返す。いずれのトークンも
/// 認識できなければ `None`（呼び出し側は `role` 自体が無いのと同様に
/// 扱う）。
fn resolve_role_interactive(role: &str) -> Option<bool> {
    for token in role.split_ascii_whitespace() {
        let lower = token.to_ascii_lowercase();
        if ARIA_INTERACTIVE_ROLES.contains(&lower.as_str()) {
            return Some(true);
        }
        if ARIA_KNOWN_NON_INTERACTIVE_ROLES.contains(&lower.as_str()) {
            return Some(false);
        }
    }
    None
}

/// [`classify_interactive_boundary`] への入力を束ねる構造体（イシュー
/// #1616 codex-review P1 再指摘の是正で導入）。wasm32 側の配線層
/// （`events::wiring`/`keynav::wiring`）が `web_sys::Element` から属性を
/// 一度だけ抽出して組み立て、native の `cargo test` からは手組みの値で
/// 直接構築できる（web-sys 非依存を維持するための橋渡し）。
///
/// フィールドの多くは `Option<&str>` だが、`has_*`/`is_*` 系は真偽値の
/// 属性有無（例: `controls`/`usemap`/`tabindex` は値を問わない属性の
/// 存在自体が意味を持つブーリアン属性）。
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundaryProbe<'a> {
    /// 対象要素のタグ名（大文字小文字は問わない）。
    pub tag: &'a str,
    /// `<a>` が `href` 属性を持つか（`href` の無い `<a>` は interactive
    /// content ではない）。
    pub has_href: bool,
    /// `<audio>`/`<video>` が `controls` 属性を持つか（ブーリアン属性、
    /// 存在自体が interactive content の条件）。
    pub has_controls: bool,
    /// `<img>`/`<object>` が `usemap` 属性を持つか。
    pub has_usemap: bool,
    /// `<input>` の `type` 属性値（無ければ既定の `text` 相当として
    /// 対話性ありと扱う）。`type="hidden"`（大文字小文字を問わない）は
    /// interactive content ではない。
    pub input_type: Option<&'a str>,
    /// `role` 属性値（無ければ `None`）。空白区切りの複数トークンを
    /// 許容し、[`resolve_role_interactive`] で解決する。
    pub role: Option<&'a str>,
    /// `tabindex` 属性の有無（値は問わない。値の妥当性検証は呼び出し側の
    /// 関心事ではない）。
    pub has_tabindex_attr: bool,
    /// `contenteditable` 属性値（無ければ `None`）。`"false"`（大文字
    /// 小文字を問わない）は「編集不可」を意味し無指定と同義に扱う
    /// （HTML 仕様の contenteditable 属性値契約）。
    pub contenteditable: Option<&'a str>,
    /// 対象要素の `data-scope` 属性値（無ければ `None`）。
    pub element_scope: Option<&'a str>,
    /// 保持者（`closest("[data-action]")` 等で解決した基準要素）の
    /// `data-scope` 属性値（無ければ `None`）。
    ///
    /// **`element_scope`/`holder_scope` は単独では境界条件にならない**
    /// （イシュー #1616 codex-review P1 再指摘の是正: 別 `data-scope`
    /// であること自体を無条件に境界とすると、`pre-styled-ui::
    /// button::close_button` 内の装飾用アイコン（`data-scope="icon"` の
    /// `<svg>`）のような、対話性を持たない装飾パーツまで境界化して
    /// しまい `data-action` の dispatch を止めてしまう）。現状この 2
    /// フィールドは分類結果を左右しないが、将来「同一 scope 内のみ通過を
    /// 許す」等の scope 依存判定を追加する余地を残すため
    /// （`classify_interactive_boundary` 1 箇所に判定を集約する設計）
    /// ごと維持する。
    pub holder_scope: Option<&'a str>,
}

impl<'a> BoundaryProbe<'a> {
    /// `tag` のみを指定し、他は「対話性の手がかりなし」で初期化する
    /// テスト・単純呼び出し向けの入口。
    #[must_use]
    pub fn new(tag: &'a str) -> Self {
        Self {
            tag,
            ..Default::default()
        }
    }

    /// `has_href` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn href(mut self, has_href: bool) -> Self {
        self.has_href = has_href;
        self
    }

    /// `has_controls` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn controls(mut self, has_controls: bool) -> Self {
        self.has_controls = has_controls;
        self
    }

    /// `has_usemap` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn usemap(mut self, has_usemap: bool) -> Self {
        self.has_usemap = has_usemap;
        self
    }

    /// `input_type` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn input_type(mut self, input_type: &'a str) -> Self {
        self.input_type = Some(input_type);
        self
    }

    /// `role` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn role(mut self, role: &'a str) -> Self {
        self.role = Some(role);
        self
    }

    /// `has_tabindex_attr` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn tabindex(mut self, has_tabindex_attr: bool) -> Self {
        self.has_tabindex_attr = has_tabindex_attr;
        self
    }

    /// `contenteditable` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn contenteditable(mut self, contenteditable: &'a str) -> Self {
        self.contenteditable = Some(contenteditable);
        self
    }

    /// `element_scope` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn element_scope(mut self, element_scope: &'a str) -> Self {
        self.element_scope = Some(element_scope);
        self
    }

    /// `holder_scope` を設定する（ビルダー、テスト向け）。
    #[must_use]
    pub fn holder_scope(mut self, holder_scope: &'a str) -> Self {
        self.holder_scope = Some(holder_scope);
        self
    }
}

/// [`InteractiveBoundaryClass`] を判定する純粋関数。
///
/// 引数は [`BoundaryProbe`] を参照。呼び出し側（wasm32 配線層）は
/// `web_sys::Element` から一度だけ属性を抽出して [`BoundaryProbe`] を
/// 組み立てる契約。native の `cargo test` からは手組みの値で直接検証
/// できる（web-sys 非依存を維持する設計、`crates/wasm-full/src/
/// keynav.rs::radio_group_readonly_click_outcome` の doc 参照）。
#[must_use]
pub fn classify_interactive_boundary(probe: &BoundaryProbe<'_>) -> InteractiveBoundaryClass {
    let tag_lower = probe.tag.to_ascii_lowercase();
    let is_html_interactive = match tag_lower.as_str() {
        "a" => probe.has_href,
        "input" => !probe
            .input_type
            .is_some_and(|t| t.eq_ignore_ascii_case("hidden")),
        "audio" | "video" => probe.has_controls,
        "img" | "object" => probe.has_usemap,
        other => HTML_UNCONDITIONAL_INTERACTIVE_TAGS.contains(&other),
    };
    if is_html_interactive {
        return InteractiveBoundaryClass::Html;
    }
    if probe
        .role
        .is_some_and(|r| resolve_role_interactive(r) == Some(true))
    {
        return InteractiveBoundaryClass::Aria;
    }
    if probe.has_tabindex_attr {
        return InteractiveBoundaryClass::Aria;
    }
    if probe
        .contenteditable
        .is_some_and(|v| !v.eq_ignore_ascii_case("false"))
    {
        return InteractiveBoundaryClass::Aria;
    }
    InteractiveBoundaryClass::Ordinary
}

/// readonly が「値を変更する操作」を抑止する対象パーツの allowlist
/// （`(data-scope, data-part)` の組、イシュー #1616 codex-review P1 再指摘
/// の是正、PR #1886）。
///
/// [`wiring::holder_instance_is_readonly`] は `data-readonly` を持つ祖先の
/// 有無だけで dispatch 抑止を決めると、readonly を「値変更操作の抑止」
/// ではなく「そのインスタンス配下の全 `data-action` クリックの抑止」へ
/// 拡大解釈してしまう。これは `crates/headless-ui/src/password_input.rs`
/// の `visibility_trigger`（`data-readonly` を出力するが、表示切替は値を
/// 変更しないため readonly でも操作可能という既存の公開契約、同モジュール
/// rustdoc 281〜289 行・432〜433 行参照）を壊し得る。本 allowlist は
/// `holder`（`closest("[data-action]")` の解決結果）自身の
/// `(data-scope, data-part)` がここに列挙された「値を変更する操作パーツ」
/// である場合に限り、readonly 抑止の対象とすることを明示する契約である。
///
/// 新たに readonly 対応するコンポーネントを配線する場合は、値を変更する
/// 操作パーツ（例: 選択・入力確定操作）だけをここへ追加登録する。表示
/// 切替・独立した子操作（例: ヘルプボタン）などの値を変更しない操作は
/// 含めない。
pub const READONLY_VALUE_CHANGING_PARTS: &[(&str, &str)] = &[
    ("radio-group", "item"),
    ("radio-group", "item-hidden-input"),
    ("radio-group", "item-control"),
];

/// `(scope, part)` が [`READONLY_VALUE_CHANGING_PARTS`] allowlist に
/// 含まれるかどうかを判定する純粋関数（native の `cargo test` で検証
/// 可能にするため `wiring` の外に置く）。
#[must_use]
pub fn is_readonly_value_changing_part(scope: &str, part: &str) -> bool {
    READONLY_VALUE_CHANGING_PARTS.contains(&(scope, part))
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        action_from_click, action_from_form_control, action_from_input,
        classify_interactive_boundary, is_readonly_value_changing_part, ActionRef, AttrSource,
        BoundaryProbe, InteractiveBoundaryClass, ACTION_CHANGE_ATTR, ACTION_INPUT_ATTR,
    };
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};

    /// `web_sys::Element` を [`AttrSource`] に橋渡しする薄いラッパー。
    ///
    /// 配線層（本モジュール）専用のアダプタであり、純粋ロジック層
    /// （[`action_from_click`]）を web-sys の具象型から独立させたまま保つ。
    struct ElementAttrSource<'a>(&'a Element);

    impl AttrSource for ElementAttrSource<'_> {
        fn attr(&self, name: &str) -> Option<String> {
            self.0.get_attribute(name)
        }
    }

    /// input/change イベントターゲットからフォーム値を文字列として抽出する
    /// （イシュー #1120）。
    ///
    /// `HtmlInputElement`（`type="checkbox"`/`type="radio"` は `checked` を
    /// `"true"`/`"false"` に文字列化、それ以外は `value`）→
    /// `HtmlSelectElement`（`value`）→ `HtmlTextAreaElement`（`value`）の順に
    /// キャストを試み、いずれにも該当しない要素（フレームワークが関知しない
    /// カスタム要素等）は `None` を返す（安全側 no-op）。
    fn extract_form_value(target: &web_sys::EventTarget) -> Option<String> {
        if let Some(input) = target.dyn_ref::<HtmlInputElement>() {
            let input_type = input.type_();
            return Some(if input_type == "checkbox" || input_type == "radio" {
                input.checked().to_string()
            } else {
                input.value()
            });
        }
        if let Some(select) = target.dyn_ref::<HtmlSelectElement>() {
            return Some(select.value());
        }
        if let Some(textarea) = target.dyn_ref::<HtmlTextAreaElement>() {
            return Some(textarea.value());
        }
        None
    }

    /// `target`（クリックイベントの解決起点）から `holder`（`closest(
    /// "[data-action]")` で解決した要素、`target` 自身を含む）まで祖先方向へ
    /// 辿り、途中に「`holder` の `data-action` とは無関係な独立対話要素」
    /// （[`classify_interactive_boundary`] が `Html`/`Aria` と判定する要素）
    /// が挟まっているかどうかを判定する（イシュー #1616 codex-review P1
    /// 再指摘: `item[data-readonly][data-action="select"]` 内のリンクを
    /// クリックすると、リンク自体は独立要素として readonly 抑止をすり抜ける
    /// 一方で、`closest("[data-action]")` が親 `item` を拾ってしまい
    /// readonly を確認せず `select` を dispatch していた）。
    ///
    /// `holder` 自身に到達する前に境界要素へ遭遇した場合は `true`
    /// （呼び出し側は dispatch を行わない）。境界に遭遇せず `holder` へ
    /// 到達できた場合は `false`。境界判定は
    /// [`classify_interactive_boundary`] に委譲しており、`data-scope` が
    /// `holder` と異なることだけを理由には境界としない（イシュー #1616
    /// codex-review P1 再指摘の是正: `pre-styled-ui::button::close_button`
    /// のような `data-action` 付き要素が、装飾用の別 `data-scope` 子孫
    /// （`data-scope="icon"` の `<svg>` 等）を内包していても、その子孫を
    /// クリックしたときに dispatch が止まってはならない）。
    ///
    /// # 例外: holder 自身が持つ labelable control
    ///
    /// `holder` が `<label>` であり、`el` がその子孫の `input`/`select`/
    /// `textarea`（[`InteractiveBoundaryClass::Html`] に分類される native
    /// フォームコントロール）の場合は境界とみなさない。これらは
    /// `holder`（`<label data-action="select">` 相当）自身が意味的に
    /// 保持する制御であり、キーボード Space 決定等でこの input 自身へ
    /// 合成 click が飛んだ場合に `data-action` の解決を止めてしまうと
    /// 正常な選択操作まで壊れる（`crates/headless-ui/src/radio_group.rs`
    /// の `item` は native `<label>` として実装され、内包する
    /// `item_hidden_input` への操作を label 経由で受ける契約）。
    fn foreign_action_boundary_between(target: &Element, holder: &Element) -> bool {
        let holder_is_label = holder.tag_name().eq_ignore_ascii_case("label");
        let holder_scope = holder.get_attribute("data-scope");
        let mut current = Some(target.clone());
        while let Some(el) = current {
            if el.is_same_node(Some(holder)) {
                return false;
            }
            let tag_name = el.tag_name();
            let is_holder_own_control = holder_is_label
                && matches!(
                    tag_name.to_ascii_lowercase().as_str(),
                    "input" | "select" | "textarea"
                );
            if !is_holder_own_control {
                let has_href = el.has_attribute("href");
                let has_controls = el.has_attribute("controls");
                let has_usemap = el.has_attribute("usemap");
                let input_type = el.get_attribute("type");
                let role = el.get_attribute("role");
                let has_tabindex_attr = el.has_attribute("tabindex");
                let contenteditable = el.get_attribute("contenteditable");
                let element_scope = el.get_attribute("data-scope");
                let probe = BoundaryProbe {
                    tag: &tag_name,
                    has_href,
                    has_controls,
                    has_usemap,
                    input_type: input_type.as_deref(),
                    role: role.as_deref(),
                    has_tabindex_attr,
                    contenteditable: contenteditable.as_deref(),
                    element_scope: element_scope.as_deref(),
                    holder_scope: holder_scope.as_deref(),
                };
                let class = classify_interactive_boundary(&probe);
                if !matches!(class, InteractiveBoundaryClass::Ordinary) {
                    return true;
                }
            }
            current = el.parent_element();
        }
        false
    }

    /// `holder`（`closest("[data-action]")` で解決した要素）自身が
    /// readonly な headless-ui インスタンスの「値を変更する操作パーツ」で
    /// あるかどうかを DOM 上で判定する（イシュー #1616 codex-review P1
    /// 再指摘の是正。さらに PR #1886 codex-review P1 再指摘で
    /// [`is_readonly_value_changing_part`] allowlist ゲートを追加）。
    ///
    /// `crates/wasm-full/src/headless.rs::instance_is_readonly` と同じ
    /// 判定方針を DOM 直接走査で再現するが、**探索の起点は `holder` 自身に
    /// 限定し、`holder` に `data-scope` が無い場合は祖先を遡って探しに
    /// 行かない**（イシュー #1616 codex-review P1 再指摘: readonly item
    /// 内に独立した `button[data-action="show_help"]` を置くと、
    /// `closest("[data-action]")` は `button` 自身を `holder` として解決
    /// する。ここで祖先方向へ `data-scope` を探しに行ってしまうと、無関係の
    /// `button` が祖先 `item` の readonly を継承してしまい、`show_help` の
    /// ような独立した子操作まで抑止してしまう。`button` はそのコンポーネント
    /// 自身の選択操作パーツではないため、readonly 判定の対象外とする）。
    ///
    /// **`holder` 自身の `(data-scope, data-part)` が
    /// [`READONLY_VALUE_CHANGING_PARTS`] allowlist に含まれない場合は、
    /// `data-readonly` の有無にかかわらず一切抑止しない**（PR #1886
    /// codex-review P1 再指摘: `data-readonly` の有無だけで判定すると、
    /// readonly を「値変更操作の抑止」ではなく「そのインスタンス配下の
    /// 全 `data-action` クリックの抑止」へ拡大解釈してしまい、
    /// `crates/headless-ui/src/password_input.rs` の `visibility_trigger`
    /// （`data-readonly` を出力するが表示切替は値を変更しないため readonly
    /// でも操作可能という既存の公開契約）を壊し得る。allowlist ゲートは
    /// このすり替えを構造的に防ぐ）。
    ///
    /// 判定手順:
    /// 1. `holder` 自身に `data-scope`/`data-part` が無い、または
    ///    その組が allowlist に無ければ直ちに `false`（headless-ui の
    ///    anatomy と無関係な独立要素、既存の非 headless-ui アプリの
    ///    `data-action` 経路、または値を変更しない操作パーツ。readonly
    ///    判定の対象外）。
    /// 2. `holder` 自身が `data-readonly` を持てば直ちに `true`。
    /// 3. `holder` を起点に祖先方向へ同じ `data-scope` の要素だけを見ながら
    ///    `data-readonly` の有無を確認し、`data-part="root"` に到達したら
    ///    打ち切る（異なる `data-scope` の要素は無関係な別コンポーネント
    ///    としてスキップして継続し、`root` を越えて別インスタンスの
    ///    readonly が越境伝播しないようにする、PR #1879 codex-review P1
    ///    再指摘と同じ設計）。これは `holder` がコンポーネント自身の
    ///    選択操作パーツ（例: `[data-scope="radio-group"][data-part="item"]`）
    ///    である場合に、同一インスタンスの readonly を正しく反映するための
    ///    経路である。
    ///
    /// `click_root` は探索範囲を配線対象の root 内へ限定する（`root` より
    /// 外側の祖先まで走査しない）。
    fn holder_instance_is_readonly(holder: &Element, click_root: &Element) -> bool {
        let Some(scope) = holder.get_attribute("data-scope") else {
            return false;
        };
        let Some(part) = holder.get_attribute("data-part") else {
            return false;
        };
        if !is_readonly_value_changing_part(&scope, &part) {
            return false;
        }
        if holder.has_attribute("data-readonly") {
            return true;
        }

        let mut current = Some(holder.clone());
        while let Some(el) = current {
            if el.get_attribute("data-scope").as_deref() == Some(scope.as_str()) {
                if el.has_attribute("data-readonly") {
                    return true;
                }
                if el.get_attribute("data-part").as_deref() == Some("root") {
                    break;
                }
            }
            if el.is_same_node(Some(click_root)) {
                break;
            }
            current = el.parent_element();
        }
        false
    }

    /// input イベントの `target` が属性契約 [`ACTION_INPUT_ATTR`] を持つ
    /// 祖先要素に一致する場合のみ [`ActionRef`] を組み立てる（イシュー
    /// #1120）。一致しない・`root` の子孫でない・フォーム値が抽出できない
    /// ・属性値が付いていないのいずれかであれば `None`（呼び出し側は
    /// レガシー経路へフォールバックする、`wire_events` doc 参照）。
    ///
    /// `selector`（`"[data-action-input]"`）は [`wire_events`] がマウント時に
    /// 1 回だけ組み立てて渡す（毎イベントで `format!` を呼ぶアロケーションを
    /// 避けるため）。
    fn attribute_input_action(
        root: &Element,
        target: &web_sys::EventTarget,
        selector: &str,
    ) -> Option<ActionRef> {
        let element = target.dyn_ref::<Element>()?;
        let matched = element.closest(selector).ok().flatten()?;
        if !root.contains(Some(&matched)) {
            return None;
        }
        let value = extract_form_value(target)?;
        let source = ElementAttrSource(&matched);
        action_from_form_control(&source, ACTION_INPUT_ATTR, &value)
    }

    /// ルート要素へ `click` / `input` の委譲リスナーをマウント時に 1 回だけ登録する。
    ///
    /// - `click`: `event.target()` から `closest("[data-action]")` で祖先方向に
    ///   `data-action` 属性を持つ要素を探索する（ボタン内の子要素クリックを
    ///   取りこぼさないための対策。PoC 版は `target()` 直接参照のため子要素
    ///   クリックを取りこぼしていた）。`event.target()` がテキストノード
    ///   （`fandhe_frontend_core::text` が生成するボタン文言等）の場合は `Element` への
    ///   キャストが失敗するため、`Node::parent_element()` で直近の親要素まで
    ///   遡ってから `closest` を呼ぶ（テキストノードクリックの取りこぼし対策、
    ///   PR #200 Cursor Bugbot 指摘）。`Element::closest` は呼び出し要素自身
    ///   から祖先方向へ辿るのみで文書全体は走査しないが、`root` より外側の
    ///   祖先に `data-action` 要素があれば理論上そこまで一致し得るため、本関数は
    ///   `contains` で「ヒットした要素が root の子孫（root 自身を含む）であること」
    ///   を確認してから採用する。
    /// - `input`: `event.target()` から `closest("[data-action-input]")`（click と
    ///   同型の祖先探索。値要素自身が属性を持つ通常のケースでは 1 ステップで
    ///   一致する）で属性契約 [`ACTION_INPUT_ATTR`] 一致を試み、
    ///   [`action_from_form_control`] へ渡す。一致しない場合は
    ///   `event.target()` を `HtmlInputElement` へキャストできた場合のみ
    ///   レガシー経路 [`action_from_input`]（`id="draft-input"` ハードコード）
    ///   へフォールバックする（イシュー #1120。既存デモ・回帰テストの
    ///   非退行）。
    /// - `change`: `event.target()` から `closest("[data-action-change]")` で
    ///   属性契約 [`ACTION_CHANGE_ATTR`] 一致を試みる（`<select>`/checkbox/
    ///   radio/date 等、input イベントでは確定しないフォーム要素向け。
    ///   イシュー #1120 で新規追加）。
    ///
    /// アクション判定に成功した場合のみ `on_action` を呼ぶ（状態更新・再描画は
    /// 呼び出し側の責務。本関数は関知しない）。
    ///
    /// `Closure::forget` は click / input / change の 3 回のみに限定する
    /// （イシュー #1120 で change 分を追加）。マウントはアプリ生存期間に
    /// 1 度だけの前提であり、リーク数は定数個に収まる（`forget` は safe API
    /// であり `unsafe` を要しない）。
    pub fn wire_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let click_root = root.clone();
        let on_action_click = std::rc::Rc::new(std::cell::RefCell::new(on_action));
        let on_action_input = on_action_click.clone();
        let on_action_change = on_action_click.clone();
        let input_root = root.clone();
        let change_root = root.clone();
        // `closest` へ渡すセレクタ文字列はマウント時に 1 回だけ組み立てる
        // （毎イベントで `format!` を呼ぶアロケーションを避けるため、
        // イシュー #1120）。
        let input_selector = format!("[{ACTION_INPUT_ATTR}]");
        let change_selector = format!("[{ACTION_CHANGE_ATTR}]");

        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            // `event.target()` はクリックされた最も深いノードを指し、テキスト
            // ノード（`fandhe_frontend_core::text` が生成する `data-action` ボタン内の文言
            // 等）であることがある。テキストノードは `Element` ではないため
            // `dyn_ref::<Element>()` は `None` を返すが、これは「フレームワーク
            // 管轄外のクリック」ではなく「祖先探索の起点を要素まで遡る必要が
            // ある」ケースである。`Node::parent_element()` で直近の親要素へ
            // 遡ってから `closest` を呼ぶことで、テキストノードクリックでも
            // `data-action` 祖先探索を取りこぼさないようにする（Cursor Bugbot
            // 指摘、PR #200 review 4719004004）。
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
            // data-action を持つ祖先要素を探索する。探索失敗（None）・
            // クエリ不正（Err）はいずれもフレームワーク管轄外のクリックとして
            // 無視する（安全側 no-op）。
            let Ok(Some(matched)) = target_element.closest("[data-action]") else {
                return;
            };
            // click_root の子孫でない要素（closest が別ツリーへ抜けた場合）は
            // 採用しない。`contains` は自分自身も含むため matched == root の
            // ケースも許容する。
            if !click_root.contains(Some(&matched)) {
                return;
            }
            // イシュー #1616 codex-review P1 再指摘: `target_element` から
            // `matched`（`data-action` 解決結果）までの間に独立対話要素の
            // 境界があれば、その境界より外側の `data-action` は解決しない
            // （[`foreign_action_boundary_between`] doc 参照）。
            if foreign_action_boundary_between(&target_element, &matched) {
                return;
            }
            // 解決した `matched` が readonly な headless-ui パーツの
            // インスタンス内にある場合は dispatch しない
            // （[`holder_instance_is_readonly`] doc 参照）。
            if holder_instance_is_readonly(&matched, &click_root) {
                return;
            }
            let source = ElementAttrSource(&matched);
            if let Some(action_ref) = action_from_click(&source) {
                (on_action_click.borrow_mut())(action_ref);
            }
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        let input_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            // 属性契約 `data-action-input` を優先する（イシュー #1120）。値
            // 要素自身（`event.target()`）が対象のため click と異なりテキスト
            // ノード遡りは不要だが、`closest` は呼び出し要素自身も含めて
            // 祖先方向へ辿るため、値要素自身に属性が付いている通常の構成では
            // そのまま一致する。属性契約に一致しなかった場合のみレガシー
            // 経路（`id="draft-input"` ハードコード）へフォールバックする
            // （`action_from_input` doc 参照、既存アプリの非退行）。
            if let Some(action_ref) = attribute_input_action(&input_root, &target, &input_selector)
            {
                (on_action_input.borrow_mut())(action_ref);
                return;
            }
            let Some(input) = target.dyn_ref::<HtmlInputElement>() else {
                return;
            };
            if let Some(action_ref) = action_from_input(&input.id(), &input.value()) {
                (on_action_input.borrow_mut())(action_ref);
            }
        });
        root.add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())?;
        input_closure.forget();

        let change_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(element) = target.dyn_ref::<Element>() else {
                return;
            };
            let Ok(Some(matched)) = element.closest(&change_selector) else {
                return;
            };
            if !change_root.contains(Some(&matched)) {
                return;
            }
            let Some(value) = extract_form_value(&target) else {
                return;
            };
            let source = ElementAttrSource(&matched);
            if let Some(action_ref) = action_from_form_control(&source, ACTION_CHANGE_ATTR, &value)
            {
                (on_action_change.borrow_mut())(action_ref);
            }
        });
        root.add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
        change_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_events;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// native `cargo test` 用のテストダブル。web-sys 非依存で
    /// [`action_from_click`] の判定ロジックを検証する。
    struct FakeElement {
        attrs: HashMap<&'static str, &'static str>,
    }

    impl AttrSource for FakeElement {
        fn attr(&self, name: &str) -> Option<String> {
            self.attrs.get(name).map(|v| v.to_string())
        }
    }

    fn element(attrs: &[(&'static str, &'static str)]) -> FakeElement {
        FakeElement {
            attrs: attrs.iter().copied().collect(),
        }
    }

    #[test]
    fn click_with_action_and_payload_dispatches() {
        let target = element(&[("data-action", "remove_item"), ("data-payload", "2")]);
        let action_ref = action_from_click(&target).expect("data-action present");
        assert_eq!(action_ref.action, "remove_item");
        assert_eq!(action_ref.payload, "2");
    }

    #[test]
    fn click_without_data_action_is_ignored() {
        let target = element(&[("data-testid", "some-div")]);
        assert_eq!(action_from_click(&target), None);
    }

    #[test]
    fn click_with_action_but_no_payload_uses_empty_payload() {
        let target = element(&[("data-action", "increment")]);
        let action_ref = action_from_click(&target).expect("data-action present");
        assert_eq!(action_ref.action, "increment");
        assert_eq!(action_ref.payload, "");
    }

    #[test]
    fn click_with_unknown_action_still_produces_action_ref() {
        // 未知アクション名の判定自体は本モジュールの責務ではない。
        // no-op 化は fandhe_frontend_interactive::dispatch/decode_action 側の契約
        // （不変条件 4）に委ねる。
        let target = element(&[("data-action", "no_such_action")]);
        let action_ref = action_from_click(&target).expect("data-action present");
        assert_eq!(action_ref.action, "no_such_action");
    }

    #[test]
    fn input_on_draft_input_dispatches_set_draft() {
        let action_ref = action_from_input("draft-input", "hello").expect("draft-input matches");
        assert_eq!(action_ref.action, "set_draft");
        assert_eq!(action_ref.payload, "hello");
    }

    #[test]
    fn input_on_other_id_is_ignored() {
        assert_eq!(action_from_input("other-input", "hello"), None);
    }

    /// REQ-1（既定エスケープ）の経路一貫性回帰テスト:
    /// イベント判定 → dispatch → `fandhe_frontend_core::render` の一連の経路を通しても
    /// 生タグが出力に現れないこと（`docs/spec/04-requirements.md` の
    /// 「イベント処理・DOM 更新経由の出力にも同一のエスケープ保証」対応）。
    #[test]
    fn event_to_dispatch_to_render_roundtrip_escapes_script_payload() {
        use fandhe_frontend_interactive::{dispatch, AppState, Component};

        let target = element(&[
            ("data-action", "set_draft"),
            ("data-payload", "<script>alert(1)</script>"),
        ]);
        let action_ref = action_from_click(&target).expect("data-action present");

        let mut state = AppState::new();
        assert!(dispatch(
            &mut state,
            &action_ref.action,
            &action_ref.payload
        ));
        // set_draft だけでは items へ反映されないため、描画確認用に add_item も
        // dispatch して draft の内容を items へ確定させる。
        assert!(dispatch(&mut state, "add_item", ""));

        let html = fandhe_frontend_core::render(&state.view());
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert"));
    }

    /// `data-idx`/payload の数値パース失敗が panic しないこと
    /// （`remove_item` は `fandhe_frontend_interactive::AppState::decode_action` 側で
    /// `parse::<usize>()` の失敗を no-op とする契約、境界外・非数値入力）。
    #[test]
    fn remove_item_with_non_numeric_payload_is_noop_not_panic() {
        use fandhe_frontend_interactive::{dispatch, AppState};

        let target = element(&[
            ("data-action", "remove_item"),
            ("data-payload", "not-a-number"),
        ]);
        let action_ref = action_from_click(&target).expect("data-action present");

        let mut state = AppState::new();
        let before = state.clone();
        let dispatched = dispatch(&mut state, &action_ref.action, &action_ref.payload);
        assert!(!dispatched);
        assert_eq!(state, before);
    }

    // -----------------------------------------------------------------
    // イシュー #1120: `data-action-input`/`data-action-change` 属性契約
    // （`action_from_form_control`）の native テスト。
    // -----------------------------------------------------------------

    #[test]
    fn form_control_with_matching_attr_dispatches() {
        let target = element(&[(ACTION_INPUT_ATTR, "select_status")]);
        let action_ref = action_from_form_control(&target, ACTION_INPUT_ATTR, "shipped")
            .expect("data-action-input present");
        assert_eq!(action_ref.action, "select_status");
        assert_eq!(action_ref.payload, "shipped");
    }

    #[test]
    fn form_control_without_matching_attr_is_ignored() {
        let target = element(&[("data-testid", "some-select")]);
        assert_eq!(
            action_from_form_control(&target, ACTION_INPUT_ATTR, "shipped"),
            None
        );
    }

    #[test]
    fn form_control_with_empty_value_uses_empty_payload() {
        let target = element(&[(ACTION_CHANGE_ATTR, "select_status")]);
        let action_ref = action_from_form_control(&target, ACTION_CHANGE_ATTR, "")
            .expect("data-action-change present");
        assert_eq!(action_ref.action, "select_status");
        assert_eq!(action_ref.payload, "");
    }

    /// REQ-1（既定エスケープ）の経路一貫性回帰テスト（属性契約経路版）:
    /// `data-action-input`/`data-action-change` 経由でも XSS ペイロードが
    /// エスケープされること（`event_to_dispatch_to_render_roundtrip_escapes_script_payload`
    /// と同型、イシュー #1120）。
    #[test]
    fn form_control_to_dispatch_to_render_roundtrip_escapes_script_payload() {
        use fandhe_frontend_interactive::{dispatch, AppState, Component};

        let target = element(&[(ACTION_INPUT_ATTR, "set_draft")]);
        let action_ref =
            action_from_form_control(&target, ACTION_INPUT_ATTR, "<script>alert(1)</script>")
                .expect("data-action-input present");

        let mut state = AppState::new();
        assert!(dispatch(
            &mut state,
            &action_ref.action,
            &action_ref.payload
        ));
        assert!(dispatch(&mut state, "add_item", ""));

        let html = fandhe_frontend_core::render(&state.view());
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;alert"));
    }

    // --- classify_interactive_boundary（イシュー #1616 Bugbot/codex-review
    // 再指摘。RadioGroup readonly クリック抑止と wire_events の
    // data-action 解決の双方から共有する分類ロジックの表テスト） ---

    #[test]
    fn classify_anchor_with_href_is_html() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("a").href(true)),
            InteractiveBoundaryClass::Html
        );
    }

    #[test]
    fn classify_anchor_without_href_is_not_html() {
        assert_ne!(
            classify_interactive_boundary(&BoundaryProbe::new("a")),
            InteractiveBoundaryClass::Html
        );
    }

    #[test]
    fn classify_native_form_and_disclosure_controls_are_html() {
        for tag in [
            "button", "input", "select", "textarea", "summary", "details",
        ] {
            assert_eq!(
                classify_interactive_boundary(&BoundaryProbe::new(tag)),
                InteractiveBoundaryClass::Html,
                "tag={tag} は HTML interactive content として扱うべき"
            );
        }
    }

    #[test]
    fn classify_audio_video_with_controls_is_html() {
        // HTML 標準の interactive content
        // (https://html.spec.whatwg.org/multipage/dom.html#interactive-content)
        // には `audio[controls]`/`video[controls]` が含まれる（イシュー
        // #1616 codex-review P1 再指摘の是正）。
        for tag in ["audio", "video"] {
            assert_eq!(
                classify_interactive_boundary(&BoundaryProbe::new(tag).controls(true)),
                InteractiveBoundaryClass::Html,
                "tag={tag}[controls] は HTML interactive content として扱うべき"
            );
        }
    }

    #[test]
    fn classify_audio_without_controls_is_ordinary() {
        // `controls` 属性が無い `<audio>`/`<video>` は interactive content
        // ではない。
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("audio")),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_img_and_object_with_usemap_is_html() {
        for tag in ["img", "object"] {
            assert_eq!(
                classify_interactive_boundary(&BoundaryProbe::new(tag).usemap(true)),
                InteractiveBoundaryClass::Html,
                "tag={tag}[usemap] は HTML interactive content として扱うべき"
            );
        }
    }

    #[test]
    fn classify_img_without_usemap_is_ordinary() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("img")),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_embed_and_iframe_are_html() {
        for tag in ["embed", "iframe"] {
            assert_eq!(
                classify_interactive_boundary(&BoundaryProbe::new(tag)),
                InteractiveBoundaryClass::Html,
                "tag={tag} は HTML interactive content として扱うべき"
            );
        }
    }

    #[test]
    fn classify_input_hidden_is_ordinary() {
        // `input[type=hidden]` は HTML 標準の interactive content 一覧から
        // 明示的に除外される（イシュー #1616 codex-review P1 再指摘）。
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("input").input_type("hidden")),
            InteractiveBoundaryClass::Ordinary
        );
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("input").input_type("HIDDEN")),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_input_without_type_or_non_hidden_type_is_html() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("input")),
            InteractiveBoundaryClass::Html
        );
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("input").input_type("text")),
            InteractiveBoundaryClass::Html
        );
    }

    #[test]
    fn classify_label_of_other_component_is_html() {
        // `label` は HTML 標準の interactive content に含まれる
        // （イシュー #1616 codex-review P1 再指摘）。radio-group の `item`
        // 自身（同じく `<label>`）は呼び出し側が自パーツ判定で本関数へ
        // 渡す前に除外する契約であり、本テストは「別コンポーネントの
        // label」が Html 境界になることを固定する。
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("label")),
            InteractiveBoundaryClass::Html
        );
    }

    #[test]
    fn classify_aria_button_and_link_roles_are_aria_not_html() {
        // Bugbot 指摘: role="button"/"link" は native <button>/<a href> と
        // 異なり label activation behavior を止める HTML 仕様上の根拠が
        // 無いため、独立要素として preventDefault を要する Aria 分類に置く
        // （旧実装は誤って抑止を完全スキップしていた）。
        for role in ["button", "link"] {
            assert_eq!(
                classify_interactive_boundary(&BoundaryProbe::new("span").role(role)),
                InteractiveBoundaryClass::Aria
            );
        }
    }

    #[test]
    fn classify_aria_widget_roles_are_aria() {
        // codex-review 指摘: role="checkbox"/"switch" 等の独自ウィジェットは
        // 旧実装では分類対象外（Ordinary 扱い）だったため、祖先の
        // stop_propagation が子要素自身のクリックハンドラへの到達を阻んで
        // いた。
        for role in ["checkbox", "switch", "tab", "menuitem", "option"] {
            assert_eq!(
                classify_interactive_boundary(&BoundaryProbe::new("span").role(role)),
                InteractiveBoundaryClass::Aria
            );
        }
    }

    #[test]
    fn classify_unknown_role_is_ordinary() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("span").role("presentation")),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_role_token_list_uses_first_recognized_token() {
        // WAI-ARIA のロール解決規則
        // (https://www.w3.org/TR/wai-aria-1.2/#host_general_role)。
        // 空白区切りの複数トークンは左から順に走査し、最初に認識できる
        // 非 abstract ロールで確定する（イシュー #1616 codex-review P1
        // 再指摘・Bugbot Medium 指摘の是正）。
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("span").role("switch checkbox")),
            InteractiveBoundaryClass::Aria,
            "role=\"switch checkbox\" は最初のトークン switch で対話ロールに確定するべき"
        );
    }

    #[test]
    fn classify_role_is_ascii_case_insensitive() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("span").role("CHECKBOX")),
            InteractiveBoundaryClass::Aria
        );
    }

    #[test]
    fn classify_role_trims_surrounding_whitespace() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("span").role(" button ")),
            InteractiveBoundaryClass::Aria
        );
    }

    #[test]
    fn classify_role_skips_unknown_tokens_before_recognized_one() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("span").role("unknown button")),
            InteractiveBoundaryClass::Aria
        );
    }

    #[test]
    fn classify_role_presentation_is_ordinary() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("span").role("presentation")),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_tabindex_attribute_is_aria() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("span").tabindex(true)),
            InteractiveBoundaryClass::Aria
        );
    }

    #[test]
    fn classify_contenteditable_true_and_empty_are_aria() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("div").contenteditable("")),
            InteractiveBoundaryClass::Aria
        );
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("div").contenteditable("true")),
            InteractiveBoundaryClass::Aria
        );
    }

    #[test]
    fn classify_contenteditable_false_is_not_aria() {
        // advisor 指摘: contenteditable="false" は「編集不可」を意味し
        // 無指定と同義に扱う（HTML 仕様の contenteditable 属性値契約）。
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("div").contenteditable("false")),
            InteractiveBoundaryClass::Ordinary
        );
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("div").contenteditable("FALSE")),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_foreign_data_scope_alone_is_not_boundary() {
        // イシュー #1616 codex-review P1 再指摘: 別 `data-scope` である
        // ことだけを理由に境界化すると、対話性のない別コンポーネントの
        // 装飾パーツ（例: 別 scope の `<div>`）まで境界にしてしまう。
        // ARIA ロール・`tabindex`・`contenteditable` のいずれも無ければ
        // `Ordinary` のままであるべき。
        assert_eq!(
            classify_interactive_boundary(
                &BoundaryProbe::new("div")
                    .element_scope("combobox")
                    .holder_scope("radio-group")
            ),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_same_data_scope_is_not_boundary() {
        assert_eq!(
            classify_interactive_boundary(
                &BoundaryProbe::new("div")
                    .element_scope("radio-group")
                    .holder_scope("radio-group")
            ),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_decorative_icon_scope_in_close_button_is_ordinary() {
        // `pre-styled-ui::button::close_button`（`data-action` 付き）内の
        // 装飾用アイコン（`data-scope="icon"` の `<svg>`）は、別 `data-scope`
        // であっても対話ロール・`tabindex`・`contenteditable` を持たない
        // ため境界にならず、クリックが `data-action` へ dispatch される
        // （イシュー #1616 codex-review P1 再指摘の回帰固定）。
        assert_eq!(
            classify_interactive_boundary(
                &BoundaryProbe::new("svg")
                    .element_scope("icon")
                    .holder_scope("button")
            ),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_non_interactive_role_in_foreign_scope_is_ordinary() {
        // `role="dialog"` は対話ウィジェットロールではない
        // （[`ARIA_INTERACTIVE_ROLES`] 非該当）ため、別 `data-scope` と
        // 組み合わさっても `Ordinary` のままとする（対話ロール一覧に
        // よってのみ境界を判定する）。
        assert_eq!(
            classify_interactive_boundary(
                &BoundaryProbe::new("div")
                    .role("dialog")
                    .element_scope("dialog")
                    .holder_scope("radio-group")
            ),
            InteractiveBoundaryClass::Ordinary
        );
    }

    #[test]
    fn classify_interactive_role_in_foreign_scope_is_still_aria() {
        // 別 `data-scope` であっても、対話ウィジェットロール
        // （`role="checkbox"` 等）自体は引き続き (B) Aria に分類される
        // （scope の異同はもはや判定に寄与しないが、ロール単体の判定は
        // 維持されることの確認）。
        assert_eq!(
            classify_interactive_boundary(
                &BoundaryProbe::new("span")
                    .role("checkbox")
                    .element_scope("checkbox")
                    .holder_scope("radio-group")
            ),
            InteractiveBoundaryClass::Aria
        );
    }

    #[test]
    fn classify_plain_span_is_ordinary() {
        assert_eq!(
            classify_interactive_boundary(&BoundaryProbe::new("span")),
            InteractiveBoundaryClass::Ordinary
        );
    }

    // --- is_readonly_value_changing_part（PR #1886 codex-review P1
    // 再指摘の是正: readonly 抑止を値変更パーツの allowlist へ限定） ---

    #[test]
    fn radio_group_item_parts_are_readonly_value_changing() {
        for part in ["item", "item-hidden-input", "item-control"] {
            assert!(
                is_readonly_value_changing_part("radio-group", part),
                "radio-group の {part} は値を変更する操作パーツとして \
                 allowlist に含まれるべき"
            );
        }
    }

    #[test]
    fn radio_group_non_operation_parts_are_not_readonly_value_changing() {
        // root/label/item-text は選択操作そのものではないため、たとえ
        // `data-action` が付いていても readonly 抑止の対象にしない。
        for part in ["root", "label", "item-text"] {
            assert!(!is_readonly_value_changing_part("radio-group", part));
        }
    }

    #[test]
    fn password_input_visibility_trigger_is_not_readonly_value_changing() {
        // `crates/headless-ui/src/password_input.rs` の `visibility_trigger`
        // は `data-readonly` を出力するが、表示切替は値を変更しないため
        // readonly でも操作可能という既存の公開契約（同モジュール
        // rustdoc 281〜289 行・432〜433 行）を allowlist ゲートで保持する。
        assert!(!is_readonly_value_changing_part(
            "password-input",
            "visibility-trigger"
        ));
    }

    #[test]
    fn unknown_scope_is_not_readonly_value_changing() {
        assert!(!is_readonly_value_changing_part("unknown-scope", "item"));
    }
}
