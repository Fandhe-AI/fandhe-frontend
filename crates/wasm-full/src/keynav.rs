//! Tabs / Accordion / Menu / Select / RadioGroup のキーボード操作
//! （イシュー #582・#583、親 #581）。
//!
//! PR #560（Tabs）/#561（Accordion）は `fandhe-frontend-headless-ui` 側の SSR
//! 静的マークアップ（roving tabindex・`data-state`/`aria-selected`/`hidden`）
//! のみを実装し、Arrow/Home/End によるフォーカス移動・`activationMode`
//! （`crates/headless-ui/src/tabs.rs` の `ActivationMode`、イシュー #582）・
//! `loopFocus` の実挙動を本クレート（`fandhe-frontend-wasm-full`）へ委ねていた。
//! 同様に PR #566（Menu）/#568（Select）/#558（RadioGroup）も、Arrow 移動・
//! Home/End・Enter/Space 決定・highlight 追随の実挙動を本クレートへ申し送って
//! いた（各モジュール冒頭 doc の out-of-scope 節）。Select の highlight SSR
//! 表現（`data-highlighted`/`aria-activedescendant`/item `id`）は PR #617 で
//! 整備済みであり、本イシュー（#583）はその前提の上で Menu/Select/RadioGroup の
//! キーボード操作配線を実装する。本モジュールはその実装であり、[`events`] と
//! 同じ「純粋ロジック層（native `cargo test` 可）+
//! `#[cfg(target_arch = "wasm32")]` 配線層」の 2 層構成を踏襲する。
//!
//! # 設計: DOM 属性を単一情報源とするステートレス配線
//!
//! - 状態（roving tabindex・選択状態・orientation・activationMode・
//!   loopFocus・disabled・highlight・radio チェック状態）はすべて DOM 属性
//!   （`tabindex`/`data-state`/`aria-selected`/`hidden`/`data-orientation`/
//!   `data-activation-mode`/`data-loop-focus`/`disabled`/`data-disabled`/
//!   `data-highlighted`/`aria-activedescendant`/`checked`）から都度読み取り、
//!   DOM 属性へのみ書き戻す。`fandhe_frontend_interactive::Component`/
//!   `SingleSelect` のような複製状態を新設しない（hydration 状態を介さず
//!   SSR 出力とクライアント操作後 DOM の一貫性が構造的に保たれる）。
//! - Menu/Select の「決定」（Enter/Space/クリックによる項目選択）は、DOM を
//!   直接書き換えるのではなく highlight 中の項目要素へ `HtmlElement::click()`
//!   を合成することで、既存の click → `data-action` → dispatch 経路
//!   （[`events::wire_events`]）へ委譲する。マウスクリックとキーボード決定の
//!   経路を完全に一致させ、アプリ状態（開閉・選択値）へ波及する分岐処理を
//!   二重実装しない。
//! - `Closure::forget` はマウント時に keydown/click/change の 3 回のみ
//!   （[`wire_keynav`]。RadioGroup のネイティブ `<input type="radio">` の
//!   `change` 委譲を追加したため Tabs/Accordion 時代の 2 回から 1 回増える）。
//!   [`events::wire_events`] と合わせても定数個であり、無制限リークを構造的に
//!   回避する（A04 対策、events.rs と同方針）。
//! - 純粋層（[`tabs_next_index`]/[`accordion_next_index`]/
//!   [`highlight_next_index`]/[`radio_next_index`]）は web-sys に依存しない
//!   `&str`/`&[bool]` ベースの関数として切り出し、native の `cargo test`
//!   （`tests/keynav_native.rs`）で網羅的に検証する。
//!
//! # Tabs のキーボード仕様（WAI-ARIA APG Tabs パターン準拠）
//!
//! - horizontal: ArrowRight/ArrowLeft。vertical: ArrowDown/ArrowUp
//!   （`data-orientation` で分岐、他方向のキーは no-op）。
//! - Home/End で最初/最後の非 disabled trigger へ移動。disabled trigger は
//!   探索でスキップする。
//! - `data-loop-focus`（`crates/headless-ui/src/tabs.rs` が出力）が
//!   `"false"` の場合のみ端で no-op、それ以外（欠落含む）は循環する
//!   （ark-ui 既定の `true` に合わせる）。
//! - `data-activation-mode` が `"manual"` の場合はフォーカス移動のみを行い、
//!   タブの活性化（`aria-selected`/`data-state`/`hidden` の更新）は行わない。
//!   `"automatic"`（既定、欠落時も含む）はフォーカス移動と同時に活性化する。
//! - 活性化処理は `[data-part="trigger"]` への click 委譲（マウスクリック・
//!   ネイティブ `<button>` の Enter/Space が発火する click イベントの双方を
//!   カバーする）と共通の [`activate_tab`] を使う。disabled trigger の
//!   活性化要求は no-op（fail-closed）。
//! - ハンドリングしたキーのみ `prevent_default()`（ページスクロール抑止）。
//!   修飾キー（Ctrl/Alt/Meta）付き・未知キー・root 外要素（`contains` 検査、
//!   [`events`] と同じ封じ込め）は安全側 no-op。
//!
//! # Accordion のキーボード仕様（WAI-ARIA APG Accordion パターン準拠）
//!
//! - ArrowDown/ArrowUp で次/前の非 disabled item-trigger へフォーカス移動、
//!   Home/End で先頭/末尾へ。**循環はしない**（APG では循環はオプションであり、
//!   決定的挙動として本実装は非循環を選ぶ）。
//! - 開閉（Enter/Space）はネイティブ `<button>` の click 挙動と
//!   `fandhe_frontend_interactive::dispatch`/`Accordion` 状態機械の責務であり、
//!   本モジュールでは配線しない（フォーカス移動のみ）。roving tabindex も
//!   accordion には適用しない（全 trigger が tabbable という APG 仕様のまま、
//!   `crates/headless-ui/src/accordion.rs` の SSR 出力を変更しない）。
//!
//! # Menu / Select のキーボード仕様（WAI-ARIA APG Menu Button / Listbox 準拠）
//!
//! Menu（`crates/headless-ui/src/menu.rs`）・Select（`crates/headless-ui/src/select.rs`）
//! はいずれも「trigger（`button`）にフォーカスが留まり続け、`data-highlighted`/
//! `aria-activedescendant` で仮想的な項目フォーカスを表現する」パターン
//! （ネイティブ `<select>` に近い combobox 系の設計）を採る。ark-ui の Menu
//! 自体は項目へ実 DOM フォーカスを移す設計だが、本リポジトリの headless-ui は
//! Menu Item も Select Item も `div` ベースで `tabindex` を持たない（SSR 側の
//! 既存契約を変更しない）ため、本モジュールは trigger 上の keydown のみを
//! 監視し、フォーカス移動を伴わない highlight 更新に統一する。
//!
//! - **closed のとき**: ArrowDown/ArrowUp/Enter/Space で trigger へ `click()`
//!   を合成して開く（`prevent_default`）。Enter/Space もネイティブ
//!   `<button>` の既定 click 発火に任せず明示的に合成する。ネイティブ発火に
//!   任せると本ハンドラが戻った後で非同期に click が発火し、初期 highlight を
//!   設定する機会がないまま open してしまうため（Bugbot 指摘、イシュー
//!   #583）。開いた直後、可能であれば初期 highlight（先頭または末尾の非
//!   disabled 項目）を設定する。
//! - **open のとき**: ArrowDown/ArrowUp/Home/End で [`highlight_next_index`]
//!   により次の highlight 対象を求め、`data-highlighted` の付け替えと
//!   content の `aria-activedescendant` を highlight 対象の `id` へ更新する
//!   （`id` 欠落時は属性を除去する fail-safe）。**循環は既定なし**
//!   （zag/ark-ui の menu `loopFocus: false` 既定に合わせる。content の
//!   `data-loop-focus="true"` が明示されたときのみ循環する fail-closed
//!   パース、[`menu_loop_focus_from_attr`]。Tabs の
//!   [`loop_focus_from_attr`]（既定 true）とは逆既定のため専用関数とする）。
//!   Enter/Space は highlight 中の項目へ `click()` を合成し、選択・開閉制御
//!   （`closeOnSelect` 等）は click 経路の dispatch と再描画へ委譲する。
//!   highlight 対象が disabled・不在なら no-op（fail-closed）。
//! - content の解決は trigger の `aria-controls` を優先し、欠落時は
//!   `closest("[data-part=\"root\"]")` 配下の content パーツへフォール
//!   バックする。
//! - Escape による**閉鎖**（`hidden`/`data-state` の更新）は [`overlay`]
//!   モジュール（イシュー #585/#610、#580 統合層）の既存責務のため本
//!   モジュールでは扱わない。ただし highlight（`data-highlighted`/
//!   `aria-activedescendant`）は本モジュール自身が書き込む状態であり
//!   overlay 側は関知しないため、open のまま Escape を受けた時点で
//!   [`set_highlight`] の逆操作（highlight のクリア）のみを行う
//!   （実装は [`handle_menu_or_select_trigger_keydown`] の `"Escape"` 腕・
//!   [`clear_highlight`]）。これにより、閉鎖経路（クリックによる再オープン・
//!   将来の #580 統合層による Escape/outside click 閉鎖のいずれも）を問わず
//!   reopen 後の最初の Arrow キーが古い highlight から続くのを防ぐ
//!   （Bugbot 指摘、イシュー #583）。**outside click（overlay の
//!   document 単位 pointerdown 委譲）による閉鎖時の highlight 後始末は、
//!   本モジュールが root スコープの委譲リスナーしか持たず outside click を
//!   観測できないため対象外**（#580 統合層側で対応する）。
//!
//! ## サブメニュー（`trigger-item`）の ArrowRight/ArrowLeft 開閉ナビゲーション
//! （イシュー #662）
//!
//! `trigger-item`（`crates/headless-ui/src/menu.rs::trigger_item`。
//! `role="menuitem"`・`aria-haspopup="menu"`・`aria-expanded`/`data-state` の
//! sub_state・`aria-controls` を持つ、サブメニューを開く menu item）に対する
//! ArrowRight（展開）/ArrowLeft（閉鎖 + 親への復帰）を実装する。#583（keynav
//! 初期実装）・#641（typeahead）ではいずれもスコープ外として申し送られていた
//! 残要素（モジュール doc 旧版・`crates/wasm-full` 変更履歴参照）。
//!
//! - **アクティブ content の解決**（[`wiring::resolve_active_content`]）:
//!   「highlight 中の項目が `trigger-item` ∧ そのサブメニュー content が
//!   解決でき（[`wiring::resolve_submenu_content`]、`aria-controls` →
//!   `trigger-item` 子孫方向 `[data-part="content"]` → `trigger-item` 兄弟
//!   方向 `[data-part="content"]` の 3 段フォールバック。`headless-ui` の
//!   「子 `positioner`/`content` は `trigger-item` の兄弟として親 content
//!   直下に並ぶ」契約に対応するのは兄弟方向であり、子孫方向は旧実装からの
//!   後方互換）∧ root 内 ∧ open（`hidden` なし）」の間、階層を降下して
//!   アクティブ content を求める。
//!   ArrowDown/Up/Home/End・Enter/Space・typeahead・Escape の各既存キー
//!   処理は、このアクティブ content を対象にする（トップレベル Menu/Select
//!   ではアクティブ content は常に trigger 直下の content と一致し、
//!   単層時の挙動は #583/#641 時点と完全に不変）。降下回数は
//!   [`MAX_SUBMENU_DEPTH`] で上限を設け、改ざん DOM の `aria-controls`
//!   循環参照による無限ループを構造的に遮断する（A04 対策）。
//! - **開閉は click 合成で dispatch 経路へ委譲**（本モジュールの既存原則を
//!   継承）: keynav は `hidden`/`data-state`/`aria-expanded` を一切書かない。
//!   ArrowRight はアクティブ content の highlight 中項目（非 disabled・
//!   `trigger-item` かつサブメニュー解決可）へ `HtmlElement::click()` を
//!   合成し、閉鎖経路と同一の click → `crate::headless`（`data-scope`/
//!   `data-part` の静的マッピング表、`menu`/`trigger-item` → `"toggle"`）→
//!   `dispatch("toggle")` → 再描画へ委譲する（headless-ui は `data-action`
//!   を出力しないため、`events::wire_events` ではなく `headless` モジュール
//!   経由である点に注意）。展開後、サブメニュー content を再解決して先頭の
//!   非 disabled 項目へ highlight を設定する（click 経由の再描画で content
//!   が差し替わりうるため再解決するパターンは closed→open 時の既存実装
//!   （[`wiring::handle_menu_or_select_trigger_keydown`]）と同型）。
//!   ArrowLeft は親 trigger-item へ `click()` を合成して閉鎖し、アクティブ
//!   content の highlight をクリアした上で親 trigger-item へ highlight を
//!   復帰させる（親 content の `aria-activedescendant` も追随）。
//! - **fail-closed な no-op 条件**: highlight 中項目が `trigger-item` でない・
//!   disabled・サブメニュー解決失敗のときの ArrowRight、チェーン深さ 0
//!   （サブメニュー内でない）のときの ArrowLeft はいずれも `prevent_default`
//!   せず no-op とし、ページの既定キー動作を奪わない（受け入れ条件 2）。
//!   Select（`data-scope="select"`）は `trigger-item` が存在せずセレクタ
//!   不一致となるため、これらの腕は自然に no-op のまま働かない。
//! - typeahead バッファ（[`wiring::TypeaheadState`]）はアクティブ content
//!   基準で有効性判定し、ArrowRight/ArrowLeft はいずれもバッファをリセット
//!   する（展開/閉鎖後の再入力は新規バッファから始まる）。
//!
//! ## typeahead（文字キー入力による項目ジャンプ、イシュー #641）
//!
//! WAI-ARIA APG Menu Button / Listbox / Select-Only Combobox パターン準拠の
//! typeahead を Menu/Select 共通で実装する（純粋層 [`is_typeahead_key`]/
//! [`typeahead_push`]/[`typeahead_next_index`]、配線層
//! [`wiring::TypeaheadState`]）。
//!
//! - **対象キー**: 修飾キー（Ctrl/Alt/Meta）なしの単一 printable 文字
//!   （制御文字を除く）。Space はバッファが**タイムアウト内・非空のときのみ**
//!   typeahead 対象とし、バッファ無効時は従来通り決定キー（closed なら
//!   open、open なら highlight 項目への click 合成）として扱う。
//! - **バッファ**: 直前入力から [`TYPEAHEAD_TIMEOUT_MS`]（350ms、zag/ark-ui
//!   既定に整合）以内なら追記し、超過なら新規バッファとして開始する。
//!   最大 [`TYPEAHEAD_MAX_BUFFER_LEN`] 文字でそれ以上追記しない（キー長押し
//!   連打による無制限成長防止、A04 対策）。バッファは DOM から導出できない
//!   一時入力状態のため [`wire_keynav`] の keydown [`Closure`] が
//!   [`wiring::TypeaheadState`] として所有し、対象 content が変わったときの
//!   混線・タイムアウト超過は同状態が自動でリセットする（`data-*` 属性へは
//!   一切書き出さない。ユーザー打鍵文字列を DOM へ露出させる新規面を作らない
//!   ため）。
//! - **マッチング**: 各項目のラベル（[`wiring::item_label`]、Select は
//!   `[data-part="item-text"]` 子を優先し item-indicator の混入を避ける）
//!   先頭一致・大文字小文字非区別。disabled 項目はスキップし、探索は常に
//!   wrap する。バッファが同一文字の繰り返しのときは現在 highlight の
//!   **次**から、複数の異なる文字を含むバッファのときは現在 highlight
//!   **自身を含む**位置から探索する（詳細は [`typeahead_next_index`] doc
//!   参照）。マッチなし・全 disabled・空 items は no-op（fail-closed）。
//! - **closed のとき**: printable 文字キーで trigger へ `click()` を合成して
//!   開き、開いた直後にマッチ項目（無ければ先頭の非 disabled 項目）を初期
//!   highlight にする。
//! - **Escape**: 既存の highlight クリアに加えて typeahead バッファも
//!   リセットする（再入力は新規バッファから始まる）。typeahead は選択・
//!   開閉を行わず highlight 更新のみ（決定は従来通り Enter/Space の click
//!   合成経路）。
//!
//! # RadioGroup のキーボード仕様（WAI-ARIA APG Radio Group パターン準拠）
//!
//! [`crate::keynav`] が監視するのはネイティブ `<input type="radio">`
//! （`crates/headless-ui/src/radio_group.rs::item_hidden_input`）上の
//! keydown/change であり、Menu/Select と異なり実 DOM フォーカスがそのまま
//! 各項目を移動する（ネイティブ input のフォーカス可能性をそのまま使う）。
//!
//! - ArrowRight/ArrowDown で次、ArrowLeft/ArrowUp で前の非 disabled 項目へ
//!   移動する（[`radio_next_index`]）。root の `data-orientation` が
//!   `"horizontal"` のとき左右キーのみ、`"vertical"` のとき上下キーのみを
//!   受理し、欠落時は両軸を受理する（ark-ui 準拠）。**循環あり**（APG Radio
//!   Group パターンは端で反対側へ回り込む）。Home/End で先頭/末尾の非
//!   disabled 項目へ移動する（APG のオプション挙動として採用）。
//! - 移動先の input へ `focus()` + ネイティブ `checked = true` を設定し
//!   （APG「移動と同時に選択」）、グループ内全項目の `data-state`
//!   （`item`/`item-control`/`item-text`/`item-hidden-input` の
//!   `"checked"`/`"unchecked"`）を同期する。ブラウザの `name` 属性ベースの
//!   自動排他選択に頼らず、`HtmlInputElement::set_checked` で明示的に全項目
//!   へ反映することで `name` 欠落時も決定的に動作する。
//! - Space によるネイティブな決定（チェック + `change` 発火）はブラウザに
//!   委ね、`wire_keynav` が追加する `change` 委譲リスナーが `data-state`
//!   群を同期する（マウスクリックによる選択変更も同じ経路で追随する）。
//!   Enter は APG Radio パターンの対象外のため no-op。
//!
//! # Combobox のキーボード仕様（ARIA 1.2 Combobox パターン準拠、イシュー #1071）
//!
//! `crates/headless-ui/src/combobox.rs`（イシュー #749）は Combobox の SSR
//! 出力と状態機械（`Combobox` = `Disclosure` + `SingleSelect` + `TextInput`）
//! のみを提供し、キーボードナビゲーションの実挙動を本モジュールへ申し送って
//! いた。Combobox は Menu/Select と異なり `input`（`role="combobox"`）が実
//! DOM フォーカスを保持し続けるテキストフィールドであるため、
//! [`handle_menu_or_select_trigger_keydown`](wiring::handle_menu_or_select_trigger_keydown)
//! を流用せず専用ハンドラ
//! （[`handle_combobox_input_keydown`](wiring::handle_combobox_input_keydown)）
//! を新設する。
//!
//! - **typeahead を実装しない**: input 自身がテキスト入力欄であり、
//!   printable 文字キーはフィルタ入力としてブラウザの既定動作（キャレット
//!   位置への文字挿入）へそのまま委ねる。Menu/Select 用の
//!   [`is_typeahead_key`]/[`TypeaheadState`](wiring::TypeaheadState) は
//!   Combobox の keydown ハンドラから一切呼ばない。
//! - **ArrowLeft/ArrowRight/Tab を claim しない**: テキストフィールドでは
//!   キャレット移動・フォーカス移動の既定動作であり、[`submenu_nav`] も
//!   呼ばない（サブメニュー概念は Combobox に存在しない）。
//! - **Home/End は open のときのみ** highlight 移動として claim する。
//!   closed のときはキャレット移動（行頭/行末への移動）の既定動作を奪わない
//!   （fail-closed、[`combobox_key_action`] 判定表参照）。
//! - **Escape は open のときのみ** claim する。closed で claim すると
//!   trigger への `click()` 合成（toggle）により誤って open してしまう
//!   fail-open 回帰になるため、closed の Escape は必ず no-op とする
//!   （[`combobox_key_action`] doc 参照）。
//! - **Enter は open のときのみ** claim する。closed の Enter を奪うと
//!   フォーム内 Combobox の既定 submit 挙動を壊すため no-op のままにする。
//! - **`aria-activedescendant` は input 側へ書く**（`crates/headless-ui/src/combobox.rs`
//!   の「input 側に配線する」契約、Menu/Select の content 側配線とは逆）。
//!   [`wiring::set_highlight_on_host`]/[`wiring::clear_highlight_on_host`]
//!   （[`wiring::set_highlight`]/[`wiring::clear_highlight`] の薄いラッパー化
//!   後の実体）が `activedescendant_host` 引数でホスト要素を選べるようにし、
//!   Combobox は input を、Menu/Select は引き続き content を渡す。
//! - **`aria-expanded`/`hidden`/`data-state` は一切書かない**（本モジュールの
//!   既存不変条件、モジュール doc §設計参照）。開閉は `trigger`
//!   （`tabindex="-1"` 固定でフォーカスを受けない専用トリガー、
//!   `combobox::trigger` doc 参照）への `HtmlElement::click()` 合成 →
//!   `crate::headless` の静的マッピング表（`combobox`/`trigger` →
//!   `"toggle"`、本イシューで追加）→ `dispatch("toggle")` → 再描画で
//!   `combobox::input`/`combobox::trigger` が `state` から `aria-expanded`
//!   を再出力する経路へ委譲する（Menu/Select の trigger 開閉と同型、#662
//!   `menu`/`trigger-item` 整備と同種の対応）。確定（Enter）は highlight 中の
//!   `combobox`/`item` への `click()` 合成 → `crate::headless` の
//!   `combobox`/`item` → `"select"`（`data-value` 必須）→
//!   `ComboboxAction::Select` へ委譲する（ark-ui の `closeOnSelect` 既定に
//!   準拠し選択と同時に close、`crates/headless-ui/src/combobox.rs` の
//!   `update` 実装参照）。
//! - **選択後の入力値（label）反映は行わない**（#749 が明示した out-of-scope
//!   のまま、`ComboboxAction::Select` は value のみを知り label を知らない）。
//!
//! # セキュリティ不変条件
//!
//! - DOM 書き込みは `set_attribute`/`remove_attribute`/`focus()`/`click()`/
//!   `HtmlInputElement::set_checked` のみで、属性名は `&'static str`
//!   リテラル固定・属性値は固定語彙（`"0"`/`"-1"`/`"true"`/`"false"`/
//!   `"active"`/`"inactive"`/`"checked"`/`"unchecked"`/`""`）のみ。
//!   `aria-activedescendant` の値のみ DOM 上の既存 `id` 属性の転記であり、
//!   新規の注入面を作らない。`set_inner_html`・HTML 文字列組み立ては
//!   一切行わない（REQ-1 不変条件）。
//! - `data-activation-mode`/`data-loop-focus`/`data-orientation` の欠落・
//!   未知値は文書化された既定（automatic / menu は loop false・tabs は
//!   loop true / 両軸許容）へ決定的にフォールバックし、panic しない。
//! - highlight・radio 決定はいずれも disabled 項目に対して no-op
//!   （fail-closed）。
//! - サブメニューチェーン探索（[`wiring::resolve_active_content`]）は深さ上限
//!   （[`MAX_SUBMENU_DEPTH`]）+ root 封じ込め検査（[`wiring::resolve_submenu_content`]
//!   の `root.contains`）で、改ざん DOM の `aria-controls` 循環参照・
//!   root 外要素への越境をいずれも fail-closed に遮断する（イシュー #662、
//!   A01/A04 対策）。DOM 書き込み面（属性名リテラル固定・値語彙固定）は
//!   本イシューでも不変。

/// パーツの向き（`crates/headless-ui/src/data_attrs.rs::Orientation` の値語彙
/// と対応する、web-sys 非依存の純粋層専用の複製）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// 横方向（ArrowRight/ArrowLeft で移動）。
    Horizontal,
    /// 縦方向（ArrowDown/ArrowUp で移動）。
    Vertical,
}

impl Orientation {
    /// `data-orientation` 属性値文字列から解釈する。未知値・欠落は
    /// horizontal へ決定的にフォールバックする（fail-closed、panic しない）。
    #[must_use]
    pub fn from_attr(value: Option<&str>) -> Self {
        match value {
            Some("vertical") => Self::Vertical,
            _ => Self::Horizontal,
        }
    }

    /// `data-orientation` 属性値文字列から解釈する任意版（RadioGroup 専用）。
    ///
    /// [`Self::from_attr`] と異なり、欠落・未知値は `None`（軸制限なし＝両軸
    /// 受理）として扱う。RadioGroup の `data-orientation` は
    /// `crates/headless-ui/src/radio_group.rs::root` の `orientation: Option<Orientation>`
    /// 引数が `None` のとき出力されない任意属性であり、「horizontal 既定」に
    /// フォールバックする Tabs の [`Self::from_attr`] とは契約が異なる
    /// （ark-ui 準拠：orientation 未指定時は上下左右いずれのキーも受理する）。
    #[must_use]
    pub fn from_attr_optional(value: Option<&str>) -> Option<Self> {
        match value {
            Some("vertical") => Some(Self::Vertical),
            Some("horizontal") => Some(Self::Horizontal),
            _ => None,
        }
    }
}

/// `data-loop-focus` 属性値文字列から解釈する。`"false"` のときのみ
/// 非循環、それ以外（`"true"`・未知値・欠落）は循環する
/// （ark-ui 既定の `true` に合わせた fail-open ではなく、
/// 明示的な `"false"` のみを非循環の合図として扱う fail-closed 挙動）。
#[must_use]
pub fn loop_focus_from_attr(value: Option<&str>) -> bool {
    value != Some("false")
}

/// `disabled` インデックス列の中で、`start` から `delta`（+1/-1）方向へ
/// 1 マスずつ移動しながら最初に見つかった非 disabled インデックスを返す。
///
/// `loop_focus` が `true` のときは端を越えると反対端へ循環する。`false` の
/// ときは端で探索を打ち切り `None` を返す。`disabled` が空、または
/// 移動先を `disabled.len()` 回探しても見つからない（全 disabled または
/// 自分自身に戻ってきた）場合も `None`（fail-closed、panic しない）。
fn step_non_disabled(
    start: usize,
    delta: isize,
    disabled: &[bool],
    loop_focus: bool,
) -> Option<usize> {
    let len = disabled.len();
    if len == 0 {
        return None;
    }
    let mut idx = start as isize;
    for _ in 0..len {
        idx += delta;
        if idx < 0 {
            if !loop_focus {
                return None;
            }
            idx = len as isize - 1;
        } else if idx >= len as isize {
            if !loop_focus {
                return None;
            }
            idx = 0;
        }
        if idx as usize == start {
            // 全 disabled 等で 1 周して自分自身に戻った場合、移動先はない。
            return None;
        }
        if !disabled[idx as usize] {
            return Some(idx as usize);
        }
    }
    None
}

/// 最初の非 disabled インデックス（Home キー用）。全 disabled・空なら `None`。
///
/// `pub(crate)`: 配線層（`wiring::handle_menu_or_select_trigger_keydown`）が
/// Menu/Select を開いた直後の初期 highlight 計算に流用するため公開する。
pub(crate) fn first_non_disabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().position(|&d| !d)
}

/// 最後の非 disabled インデックス（End キー用）。全 disabled・空なら `None`。
/// `pub(crate)` の理由は [`first_non_disabled`] 参照。
pub(crate) fn last_non_disabled(disabled: &[bool]) -> Option<usize> {
    disabled.iter().rposition(|&d| !d)
}

/// キーボード修飾キー（Ctrl/Alt/Meta）が押されている場合は本モジュールの
/// ナビゲーション対象外とする（ブラウザ標準のショートカット・OS ショート
/// カットとの衝突を避ける安全側判断）。Shift は許容する（Shift+Tab 等は
/// そもそも本モジュールが処理する `key` 集合に含まれないため実害はないが、
/// 将来の拡張を妨げないよう明示的に許容側へ倒す）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    /// Ctrl キー押下。
    pub ctrl: bool,
    /// Alt キー押下。
    pub alt: bool,
    /// Meta（Cmd/Win）キー押下。
    pub meta: bool,
}

impl Modifiers {
    /// いずれかの対象修飾キーが押されているか。
    #[must_use]
    pub fn any(self) -> bool {
        self.ctrl || self.alt || self.meta
    }
}

/// Tabs の keydown に対する「次にフォーカスすべきインデックス」を計算する
/// 純粋関数（web-sys 非依存、native `cargo test` 可）。
///
/// `current` は現在フォーカス中の trigger のインデックス（keydown イベント
/// ターゲット、配線層が `NodeList` から解決する）。`orientation` に一致しない
/// 方向キー（例: horizontal で ArrowUp/ArrowDown）・未知キー・修飾キー付きは
/// `None`（no-op）。`disabled` は各 trigger の disabled フラグ列で、
/// `current` は `disabled.len()` の範囲内であることを呼び出し側が保証する
/// （範囲外の場合も panic せず `None` を返す）。
#[must_use]
pub fn tabs_next_index(
    current: usize,
    key: &str,
    orientation: Orientation,
    loop_focus: bool,
    modifiers: Modifiers,
    disabled: &[bool],
) -> Option<usize> {
    if modifiers.any() || current >= disabled.len() {
        return None;
    }
    match key {
        "Home" => first_non_disabled(disabled),
        "End" => last_non_disabled(disabled),
        "ArrowRight" if orientation == Orientation::Horizontal => {
            step_non_disabled(current, 1, disabled, loop_focus)
        }
        "ArrowLeft" if orientation == Orientation::Horizontal => {
            step_non_disabled(current, -1, disabled, loop_focus)
        }
        "ArrowDown" if orientation == Orientation::Vertical => {
            step_non_disabled(current, 1, disabled, loop_focus)
        }
        "ArrowUp" if orientation == Orientation::Vertical => {
            step_non_disabled(current, -1, disabled, loop_focus)
        }
        _ => None,
    }
}

/// Accordion の keydown に対する「次にフォーカスすべきインデックス」を計算
/// する純粋関数。[`tabs_next_index`] と異なり orientation を持たず
/// （ArrowDown/ArrowUp 固定）、**循環しない**（モジュール doc 参照、APG が
/// 循環をオプションとする中で本実装は非循環を選ぶ）。
#[must_use]
pub fn accordion_next_index(
    current: usize,
    key: &str,
    modifiers: Modifiers,
    disabled: &[bool],
) -> Option<usize> {
    if modifiers.any() || current >= disabled.len() {
        return None;
    }
    match key {
        "Home" => first_non_disabled(disabled),
        "End" => last_non_disabled(disabled),
        "ArrowDown" => step_non_disabled(current, 1, disabled, false),
        "ArrowUp" => step_non_disabled(current, -1, disabled, false),
        _ => None,
    }
}

/// Menu/Select の content `data-loop-focus` 属性値文字列から解釈する。
/// [`loop_focus_from_attr`]（Tabs、既定 true）とは**逆の既定**を持つ。
/// `"true"` のときのみ循環し、それ以外（欠落・`"false"`・未知値）は
/// 非循環（zag/ark-ui の menu `loopFocus: false` 既定に合わせる、
/// モジュール doc §Menu/Select 参照）。
#[must_use]
pub fn menu_loop_focus_from_attr(value: Option<&str>) -> bool {
    value == Some("true")
}

/// Menu/Select の keydown に対する「次に highlight すべきインデックス」を
/// 計算する純粋関数（web-sys 非依存、native `cargo test` 可）。
///
/// [`tabs_next_index`]/[`accordion_next_index`] と異なり、実 DOM フォーカスの
/// 現在位置ではなく `data-highlighted` の現在位置（`current`、`None` は
/// 「まだ何も highlight されていない」）を起点にする（モジュール doc
/// §Menu/Select 参照）。`current` が `Some` かつ範囲外の場合は「highlight
/// なし」と同じ扱いにフォールバックする（fail-closed、panic しない）。
///
/// - `Home`/`End`: 先頭/末尾の非 disabled 項目。
/// - `ArrowDown`: `current` があれば次の非 disabled 項目（`loop_focus` に
///   従う）、なければ先頭の非 disabled 項目。
/// - `ArrowUp`: `current` があれば前の非 disabled 項目、なければ末尾の非
///   disabled 項目。
/// - 修飾キー付き・未知キーは `None`（no-op）。
#[must_use]
pub fn highlight_next_index(
    current: Option<usize>,
    key: &str,
    loop_focus: bool,
    modifiers: Modifiers,
    disabled: &[bool],
) -> Option<usize> {
    if modifiers.any() {
        return None;
    }
    let current_in_range = current.filter(|&i| i < disabled.len());
    match key {
        "Home" => first_non_disabled(disabled),
        "End" => last_non_disabled(disabled),
        "ArrowDown" => match current_in_range {
            Some(i) => step_non_disabled(i, 1, disabled, loop_focus),
            None => first_non_disabled(disabled),
        },
        "ArrowUp" => match current_in_range {
            Some(i) => step_non_disabled(i, -1, disabled, loop_focus),
            None => last_non_disabled(disabled),
        },
        _ => None,
    }
}

/// RadioGroup の keydown に対する「次にチェック・移動すべきインデックス」を
/// 計算する純粋関数。APG Radio Group パターンに従い**常に循環する**
/// （固定 `loop_focus = true`、[`step_non_disabled`] へ委譲）。`orientation`
/// が `Some` のとき、その軸のキーのみを受理する（`Horizontal` なら
/// ArrowLeft/ArrowRight のみ、`Vertical` なら ArrowUp/ArrowDown のみ）。
/// `None`（`data-orientation` 欠落）のときは両軸のキーを受理する
/// （[`Orientation::from_attr_optional`] の契約、ark-ui 準拠）。Home/End は
/// orientation に関わらず先頭/末尾の非 disabled 項目へ移動する。
#[must_use]
pub fn radio_next_index(
    current: usize,
    key: &str,
    orientation: Option<Orientation>,
    modifiers: Modifiers,
    disabled: &[bool],
) -> Option<usize> {
    if modifiers.any() || current >= disabled.len() {
        return None;
    }
    match key {
        "Home" => first_non_disabled(disabled),
        "End" => last_non_disabled(disabled),
        "ArrowRight" if orientation != Some(Orientation::Vertical) => {
            step_non_disabled(current, 1, disabled, true)
        }
        "ArrowLeft" if orientation != Some(Orientation::Vertical) => {
            step_non_disabled(current, -1, disabled, true)
        }
        "ArrowDown" if orientation != Some(Orientation::Horizontal) => {
            step_non_disabled(current, 1, disabled, true)
        }
        "ArrowUp" if orientation != Some(Orientation::Horizontal) => {
            step_non_disabled(current, -1, disabled, true)
        }
        _ => None,
    }
}

/// typeahead バッファのタイムアウト（ms）。直前の入力からこの時間以内なら
/// 同一バッファへ追記し、超過したら新規バッファとして開始する
/// （zag・ark-ui の既定値に整合。モジュール doc §Menu/Select 参照）。
pub const TYPEAHEAD_TIMEOUT_MS: f64 = 350.0;

/// typeahead バッファの最大文字数。キー長押し連打による無制限成長を防ぐ
/// （A04 対策、モジュール doc §セキュリティ不変条件参照）。
pub const TYPEAHEAD_MAX_BUFFER_LEN: usize = 32;

/// keydown の `key` が typeahead（文字検索）対象かどうかを判定する純粋関数。
///
/// 修飾キー（Ctrl/Alt/Meta）付き・複数文字のキー名（`"Enter"`/`"ArrowDown"`/
/// `"F5"` 等、`key.chars().count() != 1`）は対象外。Space（`" "`）は
/// **`buffer_active`（typeahead バッファがタイムアウト内で非空）のときのみ**
/// 対象とし、バッファが無効なときは呼び出し側の既存の決定キー処理
/// （trigger の open・highlight 項目の click 合成）に譲る（zag と同挙動、
/// モジュール doc §Menu/Select 参照）。制御文字（Space 以外）も対象外。
#[must_use]
pub fn is_typeahead_key(key: &str, buffer_active: bool, modifiers: Modifiers) -> bool {
    if modifiers.any() {
        return false;
    }
    let mut chars = key.chars();
    let Some(c) = chars.next() else {
        return false;
    };
    if chars.next().is_some() {
        return false;
    }
    if c == ' ' {
        return buffer_active;
    }
    !c.is_control()
}

/// typeahead バッファへ 1 文字追記する純粋関数。`elapsed_ms`
/// （直前の入力からの経過時間、同一 content 上でのみ意味を持つ。呼び出し側
/// が対象 content の一致判定を行い、不一致・初回入力時は `f64::INFINITY`
/// 等の大きな値を渡すことで新規バッファ扱いにする）が
/// [`TYPEAHEAD_TIMEOUT_MS`] 以内ならバッファを継続し、超過なら空文字から
/// 開始する。バッファ長が [`TYPEAHEAD_MAX_BUFFER_LEN`] に達している場合は
/// それ以上追記しない（無制限成長防止、fail-closed）。
#[must_use]
pub fn typeahead_push(buffer: &str, key: &str, elapsed_ms: f64) -> String {
    let mut next = if elapsed_ms <= TYPEAHEAD_TIMEOUT_MS {
        buffer.to_string()
    } else {
        String::new()
    };
    if next.chars().count() < TYPEAHEAD_MAX_BUFFER_LEN {
        next.push_str(key);
    }
    next
}

/// typeahead バッファ（`buffer`）から次に highlight すべき項目インデックスを
/// 求める純粋関数（web-sys 非依存、native `cargo test` 可）。`labels` は各
/// 項目のラベル文字列（大文字小文字非区別で前方一致比較する）、`disabled` は
/// 各項目の disabled フラグ列で `labels` と同じ長さを前提とする（長さ不一致・
/// 空・`buffer` 空は `None`、fail-closed）。
///
/// - `buffer` が同一文字の繰り返し（例 `"aa"`、単一文字 `"a"` も含む）の
///   ときは、その 1 文字で始まる項目を `current` の**次**から循環探索する
///   （同一頭文字の項目を順に巡回）。
/// - それ以外（複数の異なる文字を含むバッファ、例 `"ab"`）は `current`
///   **自身を含む**位置から循環探索する（`"a"` → `"ab"` と絞り込む際に
///   現在項目へ留まれるようにする）。
/// - `current` が `None`、または範囲外（`labels.len()` 以上）の場合は
///   先頭（インデックス 0）から探索する（[`highlight_next_index`] の
///   「範囲外は highlight なしと同じ扱い」と同じ fail-closed 方針）。
/// - disabled 項目はスキップする。マッチする項目が無い場合は `None`
///   （no-op、fail-closed）。
#[must_use]
pub fn typeahead_next_index(
    current: Option<usize>,
    buffer: &str,
    labels: &[&str],
    disabled: &[bool],
) -> Option<usize> {
    let len = disabled.len();
    if len == 0 || len != labels.len() || buffer.is_empty() {
        return None;
    }
    let query = buffer.to_lowercase();
    // repeat 判定は元の `buffer`（打鍵そのもの）の文字単位で行う。
    // `to_lowercase()` は 'ß' 等、1 文字が複数文字へ展開されうる文字を
    // 含むため、展開後の文字列（`query`）の文字数で判定すると単一打鍵を
    // 誤って「複数文字の異なる入力」または逆に「同一文字の繰り返し」と
    // 誤認する（Bugbot 指摘: Casefold breaks repeat matching）。
    let is_repeat_of_single_char = {
        let mut chars = buffer.chars();
        let first = chars.next();
        match first {
            Some(f) => chars.all(|c| c.to_lowercase().eq(f.to_lowercase())),
            None => false,
        }
    };
    let current_in_range = current.filter(|&i| i < len);
    let start = current_in_range.unwrap_or(0);
    let skip_current = is_repeat_of_single_char && current_in_range.is_some();
    let match_query: String = if is_repeat_of_single_char {
        // 展開後の `query` から単純に先頭 1 文字を切り出すと、展開を
        // 伴う文字（例: 'ß' → "ss"）の場合に本来の対応関係が壊れる
        // ため、元の 1 文字目を改めて小文字化して使う。
        buffer
            .chars()
            .next()
            .map(|c| c.to_lowercase().collect())
            .unwrap_or_default()
    } else {
        query
    };

    for offset in 0..len {
        let idx = if skip_current {
            (start + 1 + offset) % len
        } else {
            (start + offset) % len
        };
        if disabled[idx] {
            continue;
        }
        if labels[idx].to_lowercase().starts_with(&match_query) {
            return Some(idx);
        }
    }
    None
}

/// サブメニュー（`trigger-item`）チェーン探索の深さ上限（イシュー #662）。
///
/// アクティブ content の解決（[`wiring::resolve_active_content`]）は
/// `aria-controls` を辿って子孫方向へ降下するが、改ざんされた DOM が
/// `aria-controls` を自身または祖先へ循環参照させた場合、封じ込め検査
/// （`root.contains`）だけでは無限ループを止められない
/// （`root.contains` は「root 配下か」を見るだけで「既に訪問済みか」を
/// 見ないため）。本定数はその降下回数の fail-closed な上限であり、
/// 通常の UI 構成（ネストは高々数段）を大きく超える値を確保しつつ、
/// 攻撃者制御 DOM による DoS（A04 対策）を構造的に遮断する。
pub const MAX_SUBMENU_DEPTH: usize = 16;

/// ArrowRight/ArrowLeft によるサブメニュー（`trigger-item`）開閉操作の種別
/// （イシュー #662、WAI-ARIA APG Menu パターン準拠）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmenuNav {
    /// ArrowRight: highlight 中の trigger-item のサブメニューを展開する。
    Open,
    /// ArrowLeft: 現在アクティブなサブメニューを閉じ、親へ戻る。
    Close,
}

/// keydown の `key`/`modifiers` からサブメニュー開閉操作を判定する純粋関数
/// （web-sys 非依存、native `cargo test` 可）。修飾キー付き・ArrowRight/
/// ArrowLeft 以外のキーは `None`（no-op）。実際に展開・閉鎖できるか
/// （trigger-item か・disabled か・サブメニューが解決できるか・チェーン
/// 深さ 0 で ArrowLeft を受けていないか等）は配線層
/// （[`wiring::handle_menu_or_select_trigger_keydown`]）が DOM 状態を見て
/// 判断する。本関数は「そもそもこのキーがサブメニュー操作の候補か」だけを
/// 決定的に返す（モジュール doc §Menu/Select §サブメニュー参照）。
#[must_use]
pub fn submenu_nav(key: &str, modifiers: Modifiers) -> Option<SubmenuNav> {
    if modifiers.any() {
        return None;
    }
    match key {
        "ArrowRight" => Some(SubmenuNav::Open),
        "ArrowLeft" => Some(SubmenuNav::Close),
        _ => None,
    }
}

/// Combobox の keydown が要求する操作種別（純粋層、web-sys 非依存、native
/// `cargo test` 可。イシュー #1071、モジュール doc §Combobox 参照）。
///
/// Menu/Select 用の [`highlight_next_index`] 判定と異なり、Combobox は
/// フォーカスを保持するのがテキスト `<input>` であるため、typeahead・
/// キャレット移動（Home/End/ArrowLeft/ArrowRight）・フォーム submit
/// （Enter）といったネイティブ input の既定動作と衝突しない範囲でのみ
/// キーを claim する（[`combobox_key_action`] の判定表参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboboxKeyAction {
    /// closed → open。`from_end` が `true` のとき初期 highlight は末尾の
    /// 非 disabled 項目（ArrowUp で開いた場合）、`false` のとき先頭
    /// （ArrowDown で開いた場合）。
    Open {
        /// 初期 highlight を末尾から始めるか（ArrowUp での open）。
        from_end: bool,
    },
    /// open 中の highlight 移動（[`highlight_next_index`] へ委譲）。
    MoveHighlight,
    /// open 中の確定（highlight 中項目への `click()` 合成）。
    Confirm,
    /// open 中の Escape（highlight クリア + close 委譲）。
    Close,
}

/// Combobox の keydown（`key`/`modifiers`/`is_open`）から
/// [`ComboboxKeyAction`] を決定する純粋関数（web-sys 非依存、native
/// `cargo test` 可。イシュー #1071）。
///
/// `None` は no-op（`prevent_default` しない）。判定表（モジュール doc
/// §Combobox 参照）:
///
/// | key | closed | open |
/// |---|---|---|
/// | `ArrowDown` | `Open { from_end: false }` | `MoveHighlight` |
/// | `ArrowUp` | `Open { from_end: true }` | `MoveHighlight` |
/// | `Home`/`End` | `None`（キャレット移動を奪わない） | `MoveHighlight` |
/// | `Enter` | `None`（フォーム submit 等の既定を奪わない） | `Confirm` |
/// | `Escape` | `None`（**fail-closed**。closed で claim すると誤って open してしまう） | `Close` |
/// | それ以外（`ArrowLeft`/`ArrowRight`/`Tab`/printable 文字・未知キー） | `None` | `None` |
///
/// 修飾キー（Ctrl/Alt/Meta）付きは open/closed いずれも `None`
/// （[`Modifiers::any`]、既存モジュール方針を踏襲）。
#[must_use]
pub fn combobox_key_action(
    key: &str,
    modifiers: Modifiers,
    is_open: bool,
) -> Option<ComboboxKeyAction> {
    if modifiers.any() {
        return None;
    }
    if is_open {
        match key {
            "ArrowDown" | "ArrowUp" | "Home" | "End" => Some(ComboboxKeyAction::MoveHighlight),
            "Enter" => Some(ComboboxKeyAction::Confirm),
            "Escape" => Some(ComboboxKeyAction::Close),
            _ => None,
        }
    } else {
        match key {
            "ArrowDown" => Some(ComboboxKeyAction::Open { from_end: false }),
            "ArrowUp" => Some(ComboboxKeyAction::Open { from_end: true }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mods() -> Modifiers {
        Modifiers::default()
    }

    // --- Orientation/loop_focus のパース ---

    #[test]
    fn orientation_from_attr_defaults_to_horizontal_for_unknown_or_missing() {
        assert_eq!(
            Orientation::from_attr(Some("vertical")),
            Orientation::Vertical
        );
        assert_eq!(
            Orientation::from_attr(Some("horizontal")),
            Orientation::Horizontal
        );
        assert_eq!(
            Orientation::from_attr(Some("bogus")),
            Orientation::Horizontal
        );
        assert_eq!(Orientation::from_attr(None), Orientation::Horizontal);
    }

    #[test]
    fn loop_focus_from_attr_is_true_unless_explicitly_false() {
        assert!(loop_focus_from_attr(Some("true")));
        assert!(loop_focus_from_attr(None));
        assert!(loop_focus_from_attr(Some("bogus")));
        assert!(!loop_focus_from_attr(Some("false")));
    }

    // --- Tabs: horizontal ---

    #[test]
    fn horizontal_arrow_right_moves_to_next_enabled() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(1)
        );
    }

    #[test]
    fn horizontal_arrow_left_moves_to_previous_enabled() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                1,
                "ArrowLeft",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(0)
        );
    }

    #[test]
    fn horizontal_ignores_vertical_keys() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowDown",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowUp",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
    }

    // --- Tabs: vertical ---

    #[test]
    fn vertical_arrow_down_up_move_and_ignore_horizontal_keys() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowDown",
                Orientation::Vertical,
                true,
                mods(),
                &disabled
            ),
            Some(1)
        );
        assert_eq!(
            tabs_next_index(1, "ArrowUp", Orientation::Vertical, true, mods(), &disabled),
            Some(0)
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Vertical,
                true,
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowLeft",
                Orientation::Vertical,
                true,
                mods(),
                &disabled
            ),
            None
        );
    }

    // --- Home/End ---

    #[test]
    fn home_end_move_to_first_last_enabled_skipping_disabled() {
        let disabled = [true, false, false, true];
        assert_eq!(
            tabs_next_index(2, "Home", Orientation::Horizontal, true, mods(), &disabled),
            Some(1)
        );
        assert_eq!(
            tabs_next_index(1, "End", Orientation::Horizontal, true, mods(), &disabled),
            Some(2)
        );
    }

    // --- loopFocus ---

    #[test]
    fn loop_focus_true_wraps_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                2,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(0)
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowLeft",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(2)
        );
    }

    #[test]
    fn loop_focus_false_is_noop_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                2,
                "ArrowRight",
                Orientation::Horizontal,
                false,
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowLeft",
                Orientation::Horizontal,
                false,
                mods(),
                &disabled
            ),
            None
        );
    }

    // --- disabled スキップ ---

    #[test]
    fn disabled_items_are_skipped_when_stepping() {
        let disabled = [false, true, true, false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            Some(3)
        );
    }

    #[test]
    fn all_disabled_or_single_item_yields_none() {
        let disabled = [true, true, true];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(first_non_disabled(&disabled), None);
        assert_eq!(last_non_disabled(&disabled), None);

        let single = [false];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &single
            ),
            None
        );
    }

    #[test]
    fn empty_items_yields_none_without_panicking() {
        let empty: [bool; 0] = [];
        assert_eq!(
            tabs_next_index(
                0,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &empty
            ),
            None
        );
        assert_eq!(first_non_disabled(&empty), None);
        assert_eq!(last_non_disabled(&empty), None);
    }

    #[test]
    fn out_of_range_current_index_is_noop_not_panic() {
        let disabled = [false, false];
        assert_eq!(
            tabs_next_index(
                5,
                "ArrowRight",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
    }

    // --- 未知キー・修飾キー ---

    #[test]
    fn unknown_key_is_noop() {
        let disabled = [false, false, false];
        assert_eq!(
            tabs_next_index(
                0,
                "PageDown",
                Orientation::Horizontal,
                true,
                mods(),
                &disabled
            ),
            None
        );
    }

    #[test]
    fn modifier_keys_are_noop_even_for_known_keys() {
        let disabled = [false, false, false];
        for modifiers in [
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            Modifiers {
                alt: true,
                ..Modifiers::default()
            },
            Modifiers {
                meta: true,
                ..Modifiers::default()
            },
        ] {
            assert_eq!(
                tabs_next_index(
                    0,
                    "ArrowRight",
                    Orientation::Horizontal,
                    true,
                    modifiers,
                    &disabled
                ),
                None
            );
        }
    }

    // --- Accordion ---

    #[test]
    fn accordion_arrow_down_up_move_between_enabled_items() {
        let disabled = [false, false, false];
        assert_eq!(
            accordion_next_index(0, "ArrowDown", mods(), &disabled),
            Some(1)
        );
        assert_eq!(
            accordion_next_index(1, "ArrowUp", mods(), &disabled),
            Some(0)
        );
    }

    #[test]
    fn accordion_does_not_loop_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            accordion_next_index(2, "ArrowDown", mods(), &disabled),
            None
        );
        assert_eq!(accordion_next_index(0, "ArrowUp", mods(), &disabled), None);
    }

    #[test]
    fn accordion_home_end_skip_disabled() {
        let disabled = [true, false, false, true];
        assert_eq!(accordion_next_index(2, "Home", mods(), &disabled), Some(1));
        assert_eq!(accordion_next_index(1, "End", mods(), &disabled), Some(2));
    }

    #[test]
    fn accordion_unknown_key_and_modifiers_are_noop() {
        let disabled = [false, false];
        assert_eq!(accordion_next_index(0, "Home2", mods(), &disabled), None);
        assert_eq!(
            accordion_next_index(
                0,
                "ArrowDown",
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                &disabled
            ),
            None
        );
    }

    // --- Orientation::from_attr_optional ---

    #[test]
    fn orientation_from_attr_optional_none_for_unknown_or_missing() {
        assert_eq!(
            Orientation::from_attr_optional(Some("vertical")),
            Some(Orientation::Vertical)
        );
        assert_eq!(
            Orientation::from_attr_optional(Some("horizontal")),
            Some(Orientation::Horizontal)
        );
        assert_eq!(Orientation::from_attr_optional(Some("bogus")), None);
        assert_eq!(Orientation::from_attr_optional(None), None);
    }

    // --- menu_loop_focus_from_attr（既定 false、tabs の loop_focus_from_attr と逆） ---

    #[test]
    fn menu_loop_focus_from_attr_is_false_unless_explicitly_true() {
        assert!(!menu_loop_focus_from_attr(None));
        assert!(!menu_loop_focus_from_attr(Some("false")));
        assert!(!menu_loop_focus_from_attr(Some("bogus")));
        assert!(menu_loop_focus_from_attr(Some("true")));
    }

    // --- highlight_next_index（Menu/Select 共用） ---

    #[test]
    fn highlight_next_index_no_current_arrow_down_picks_first_enabled() {
        let disabled = [false, false, false];
        assert_eq!(
            highlight_next_index(None, "ArrowDown", false, mods(), &disabled),
            Some(0)
        );
    }

    #[test]
    fn highlight_next_index_no_current_arrow_up_picks_last_enabled() {
        let disabled = [false, false, false];
        assert_eq!(
            highlight_next_index(None, "ArrowUp", false, mods(), &disabled),
            Some(2)
        );
    }

    #[test]
    fn highlight_next_index_steps_from_current_and_skips_disabled() {
        let disabled = [false, true, false];
        assert_eq!(
            highlight_next_index(Some(0), "ArrowDown", false, mods(), &disabled),
            Some(2)
        );
        assert_eq!(
            highlight_next_index(Some(2), "ArrowUp", false, mods(), &disabled),
            Some(0)
        );
    }

    #[test]
    fn highlight_next_index_home_end_skip_disabled() {
        let disabled = [true, false, false, true];
        assert_eq!(
            highlight_next_index(Some(2), "Home", false, mods(), &disabled),
            Some(1)
        );
        assert_eq!(
            highlight_next_index(Some(1), "End", false, mods(), &disabled),
            Some(2)
        );
    }

    #[test]
    fn highlight_next_index_default_no_loop_is_noop_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            highlight_next_index(Some(2), "ArrowDown", false, mods(), &disabled),
            None
        );
        assert_eq!(
            highlight_next_index(Some(0), "ArrowUp", false, mods(), &disabled),
            None
        );
    }

    #[test]
    fn highlight_next_index_loop_focus_true_wraps_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            highlight_next_index(Some(2), "ArrowDown", true, mods(), &disabled),
            Some(0)
        );
        assert_eq!(
            highlight_next_index(Some(0), "ArrowUp", true, mods(), &disabled),
            Some(2)
        );
    }

    #[test]
    fn highlight_next_index_out_of_range_current_falls_back_to_no_current_behavior() {
        let disabled = [false, false, false];
        assert_eq!(
            highlight_next_index(Some(99), "ArrowDown", false, mods(), &disabled),
            Some(0)
        );
        assert_eq!(
            highlight_next_index(Some(99), "ArrowUp", false, mods(), &disabled),
            Some(2)
        );
    }

    #[test]
    fn highlight_next_index_unknown_key_and_modifiers_are_noop() {
        let disabled = [false, false, false];
        assert_eq!(
            highlight_next_index(None, "PageDown", false, mods(), &disabled),
            None
        );
        assert_eq!(
            highlight_next_index(
                None,
                "ArrowDown",
                false,
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                &disabled
            ),
            None
        );
    }

    #[test]
    fn highlight_next_index_all_disabled_or_empty_yields_none() {
        let all_disabled = [true, true];
        assert_eq!(
            highlight_next_index(None, "ArrowDown", false, mods(), &all_disabled),
            None
        );
        let empty: [bool; 0] = [];
        assert_eq!(
            highlight_next_index(None, "ArrowDown", false, mods(), &empty),
            None
        );
    }

    // --- radio_next_index（RadioGroup 専用、常に循環） ---

    #[test]
    fn radio_next_index_no_orientation_accepts_both_axes() {
        let disabled = [false, false, false];
        assert_eq!(
            radio_next_index(0, "ArrowRight", None, mods(), &disabled),
            Some(1)
        );
        assert_eq!(
            radio_next_index(0, "ArrowDown", None, mods(), &disabled),
            Some(1)
        );
        assert_eq!(
            radio_next_index(0, "ArrowLeft", None, mods(), &disabled),
            Some(2)
        );
        assert_eq!(
            radio_next_index(0, "ArrowUp", None, mods(), &disabled),
            Some(2)
        );
    }

    #[test]
    fn radio_next_index_horizontal_orientation_restricts_to_left_right() {
        let disabled = [false, false, false];
        assert_eq!(
            radio_next_index(
                0,
                "ArrowRight",
                Some(Orientation::Horizontal),
                mods(),
                &disabled
            ),
            Some(1)
        );
        assert_eq!(
            radio_next_index(
                0,
                "ArrowDown",
                Some(Orientation::Horizontal),
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(
            radio_next_index(
                0,
                "ArrowUp",
                Some(Orientation::Horizontal),
                mods(),
                &disabled
            ),
            None
        );
    }

    #[test]
    fn radio_next_index_vertical_orientation_restricts_to_up_down() {
        let disabled = [false, false, false];
        assert_eq!(
            radio_next_index(
                0,
                "ArrowDown",
                Some(Orientation::Vertical),
                mods(),
                &disabled
            ),
            Some(1)
        );
        assert_eq!(
            radio_next_index(
                0,
                "ArrowRight",
                Some(Orientation::Vertical),
                mods(),
                &disabled
            ),
            None
        );
        assert_eq!(
            radio_next_index(
                0,
                "ArrowLeft",
                Some(Orientation::Vertical),
                mods(),
                &disabled
            ),
            None
        );
    }

    #[test]
    fn radio_next_index_always_loops_at_ends() {
        let disabled = [false, false, false];
        assert_eq!(
            radio_next_index(2, "ArrowRight", None, mods(), &disabled),
            Some(0)
        );
        assert_eq!(
            radio_next_index(0, "ArrowLeft", None, mods(), &disabled),
            Some(2)
        );
    }

    #[test]
    fn radio_next_index_home_end_ignore_orientation_and_skip_disabled() {
        let disabled = [true, false, false, true];
        assert_eq!(
            radio_next_index(2, "Home", Some(Orientation::Vertical), mods(), &disabled),
            Some(1)
        );
        assert_eq!(
            radio_next_index(1, "End", Some(Orientation::Horizontal), mods(), &disabled),
            Some(2)
        );
    }

    #[test]
    fn radio_next_index_skips_disabled_items() {
        let disabled = [false, true, true, false];
        assert_eq!(
            radio_next_index(0, "ArrowRight", None, mods(), &disabled),
            Some(3)
        );
    }

    #[test]
    fn radio_next_index_out_of_range_or_modifiers_are_noop_not_panic() {
        let disabled = [false, false];
        assert_eq!(
            radio_next_index(99, "ArrowRight", None, mods(), &disabled),
            None
        );
        assert_eq!(
            radio_next_index(
                0,
                "ArrowRight",
                None,
                Modifiers {
                    alt: true,
                    ..Modifiers::default()
                },
                &disabled
            ),
            None
        );
    }

    #[test]
    fn radio_next_index_all_disabled_or_empty_yields_none() {
        let all_disabled = [true, true];
        assert_eq!(
            radio_next_index(0, "ArrowRight", None, mods(), &all_disabled),
            None
        );
        let empty: [bool; 0] = [];
        assert_eq!(
            radio_next_index(0, "ArrowRight", None, mods(), &empty),
            None
        );
    }

    // --- typeahead（Menu/Select 共用、イシュー #641） ---

    #[test]
    fn is_typeahead_key_accepts_single_printable_chars() {
        assert!(is_typeahead_key("a", false, mods()));
        assert!(is_typeahead_key("A", false, mods()));
        assert!(is_typeahead_key("1", false, mods()));
    }

    #[test]
    fn is_typeahead_key_rejects_multi_char_key_names() {
        assert!(!is_typeahead_key("Enter", false, mods()));
        assert!(!is_typeahead_key("ArrowDown", true, mods()));
        assert!(!is_typeahead_key("F5", true, mods()));
        assert!(!is_typeahead_key("Escape", true, mods()));
    }

    #[test]
    fn is_typeahead_key_rejects_modifier_keys_even_for_printable_chars() {
        assert!(!is_typeahead_key(
            "a",
            false,
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            }
        ));
        assert!(!is_typeahead_key(
            "a",
            true,
            Modifiers {
                alt: true,
                ..Modifiers::default()
            }
        ));
        assert!(!is_typeahead_key(
            "a",
            true,
            Modifiers {
                meta: true,
                ..Modifiers::default()
            }
        ));
    }

    #[test]
    fn is_typeahead_key_space_depends_on_buffer_active() {
        assert!(!is_typeahead_key(" ", false, mods()));
        assert!(is_typeahead_key(" ", true, mods()));
    }

    #[test]
    fn typeahead_push_appends_within_timeout_and_resets_after() {
        assert_eq!(typeahead_push("a", "b", 100.0), "ab");
        assert_eq!(typeahead_push("a", "b", TYPEAHEAD_TIMEOUT_MS), "ab");
        assert_eq!(typeahead_push("a", "b", TYPEAHEAD_TIMEOUT_MS + 1.0), "b");
        assert_eq!(typeahead_push("", "a", f64::INFINITY), "a");
    }

    #[test]
    fn typeahead_push_caps_at_max_buffer_len() {
        let long = "a".repeat(TYPEAHEAD_MAX_BUFFER_LEN);
        assert_eq!(typeahead_push(&long, "z", 0.0), long);
    }

    #[test]
    fn typeahead_next_index_single_char_steps_from_current_next_and_wraps() {
        let labels = ["Apple", "Banana", "Avocado"];
        assert_eq!(
            typeahead_next_index(Some(0), "a", &labels, &[false, false, false]),
            Some(2)
        );
        assert_eq!(
            typeahead_next_index(Some(2), "a", &labels, &[false, false, false]),
            Some(0)
        );
    }

    #[test]
    fn typeahead_next_index_repeated_same_char_cycles_through_matches() {
        let labels = ["Apple", "Banana", "Avocado"];
        assert_eq!(
            typeahead_next_index(Some(0), "aa", &labels, &[false, false, false]),
            Some(2)
        );
        assert_eq!(
            typeahead_next_index(Some(2), "aa", &labels, &[false, false, false]),
            Some(0)
        );
    }

    #[test]
    fn typeahead_next_index_multi_char_buffer_includes_current_position() {
        let labels = ["Apple", "Apricot", "Banana"];
        assert_eq!(
            typeahead_next_index(Some(0), "ap", &labels, &[false, false, false]),
            Some(0)
        );
        assert_eq!(
            typeahead_next_index(Some(0), "apr", &labels, &[false, false, false]),
            Some(1)
        );
    }

    #[test]
    fn typeahead_next_index_skips_disabled_items() {
        let labels = ["Apple", "Avocado", "Banana"];
        assert_eq!(
            typeahead_next_index(Some(0), "a", &labels, &[false, true, false]),
            Some(0)
        );
    }

    #[test]
    fn typeahead_next_index_case_insensitive_matching() {
        let labels = ["apple", "Banana"];
        assert_eq!(
            typeahead_next_index(None, "A", &labels, &[false, false]),
            Some(0)
        );
    }

    #[test]
    fn typeahead_next_index_no_match_all_disabled_or_empty_yields_none() {
        let labels = ["Apple", "Banana"];
        assert_eq!(
            typeahead_next_index(None, "z", &labels, &[false, false]),
            None
        );
        assert_eq!(
            typeahead_next_index(None, "a", &labels, &[true, true]),
            None
        );
        let empty_labels: [&str; 0] = [];
        let empty_disabled: [bool; 0] = [];
        assert_eq!(
            typeahead_next_index(None, "a", &empty_labels, &empty_disabled),
            None
        );
        assert_eq!(
            typeahead_next_index(None, "", &labels, &[false, false]),
            None
        );
    }

    #[test]
    fn typeahead_next_index_no_current_starts_from_beginning() {
        let labels = ["Banana", "Apple", "Avocado"];
        assert_eq!(
            typeahead_next_index(None, "a", &labels, &[false, false, false]),
            Some(1)
        );
    }

    #[test]
    fn typeahead_next_index_out_of_range_current_falls_back_to_no_current_behavior() {
        let labels = ["Apple", "Banana"];
        assert_eq!(
            typeahead_next_index(Some(99), "a", &labels, &[false, false]),
            Some(0)
        );
    }

    #[test]
    fn typeahead_next_index_label_disabled_length_mismatch_is_noop_not_panic() {
        let labels = ["Apple", "Banana"];
        assert_eq!(typeahead_next_index(None, "a", &labels, &[false]), None);
    }

    #[test]
    fn typeahead_next_index_repeat_detection_uses_raw_buffer_not_expanded_casefold() {
        // 'İ'（U+0130, LATIN CAPITAL LETTER I WITH DOT ABOVE）は
        // `char::to_lowercase()` で "i" + COMBINING DOT ABOVE (U+0307) の
        // 2 文字へ展開される。展開後の `query` の文字数で「単一文字の
        // 繰り返しか」を判定すると、2 回の同一キー打鍵（buffer 上は
        // 2 文字）が「異なる文字を含む複数文字バッファ」と誤認され、
        // 本来は current をスキップして次の一致へ循環すべきところが
        // マッチ不能になる（Bugbot 指摘: Casefold breaks repeat matching）。
        let labels = ["İstanbul", "İzmir"];
        assert_eq!(
            typeahead_next_index(Some(0), "İİ", &labels, &[false, false]),
            Some(1)
        );
    }

    #[test]
    fn submenu_nav_arrow_right_is_open() {
        assert_eq!(submenu_nav("ArrowRight", mods()), Some(SubmenuNav::Open));
    }

    #[test]
    fn submenu_nav_arrow_left_is_close() {
        assert_eq!(submenu_nav("ArrowLeft", mods()), Some(SubmenuNav::Close));
    }

    #[test]
    fn submenu_nav_rejects_modifier_keys() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        assert_eq!(submenu_nav("ArrowRight", ctrl), None);
        assert_eq!(submenu_nav("ArrowLeft", ctrl), None);
    }

    #[test]
    fn submenu_nav_rejects_unrelated_keys() {
        assert_eq!(submenu_nav("ArrowDown", mods()), None);
        assert_eq!(submenu_nav("Enter", mods()), None);
        assert_eq!(submenu_nav("Escape", mods()), None);
    }

    // --- combobox_key_action（イシュー #1071） ---

    #[test]
    fn combobox_key_action_closed_arrow_down_opens_from_start() {
        assert_eq!(
            combobox_key_action("ArrowDown", mods(), false),
            Some(ComboboxKeyAction::Open { from_end: false })
        );
    }

    #[test]
    fn combobox_key_action_closed_arrow_up_opens_from_end() {
        assert_eq!(
            combobox_key_action("ArrowUp", mods(), false),
            Some(ComboboxKeyAction::Open { from_end: true })
        );
    }

    #[test]
    fn combobox_key_action_closed_home_end_enter_are_noop() {
        // キャレット移動・フォーム submit の既定動作を奪わない
        // （受け入れ条件、モジュール doc §Combobox 参照）。
        assert_eq!(combobox_key_action("Home", mods(), false), None);
        assert_eq!(combobox_key_action("End", mods(), false), None);
        assert_eq!(combobox_key_action("Enter", mods(), false), None);
    }

    #[test]
    fn combobox_key_action_closed_escape_is_noop_fail_closed() {
        // closed で Escape を claim すると toggle で誤って open してしまう
        // fail-open 回帰（モジュール doc §Combobox 参照）。
        assert_eq!(combobox_key_action("Escape", mods(), false), None);
    }

    #[test]
    fn combobox_key_action_closed_typeahead_and_caret_keys_are_noop() {
        assert_eq!(combobox_key_action("a", mods(), false), None);
        assert_eq!(combobox_key_action("ArrowLeft", mods(), false), None);
        assert_eq!(combobox_key_action("ArrowRight", mods(), false), None);
        assert_eq!(combobox_key_action("Tab", mods(), false), None);
    }

    #[test]
    fn combobox_key_action_open_arrow_and_home_end_move_highlight() {
        assert_eq!(
            combobox_key_action("ArrowDown", mods(), true),
            Some(ComboboxKeyAction::MoveHighlight)
        );
        assert_eq!(
            combobox_key_action("ArrowUp", mods(), true),
            Some(ComboboxKeyAction::MoveHighlight)
        );
        assert_eq!(
            combobox_key_action("Home", mods(), true),
            Some(ComboboxKeyAction::MoveHighlight)
        );
        assert_eq!(
            combobox_key_action("End", mods(), true),
            Some(ComboboxKeyAction::MoveHighlight)
        );
    }

    #[test]
    fn combobox_key_action_open_enter_confirms() {
        assert_eq!(
            combobox_key_action("Enter", mods(), true),
            Some(ComboboxKeyAction::Confirm)
        );
    }

    #[test]
    fn combobox_key_action_open_escape_closes() {
        assert_eq!(
            combobox_key_action("Escape", mods(), true),
            Some(ComboboxKeyAction::Close)
        );
    }

    #[test]
    fn combobox_key_action_open_typeahead_and_caret_keys_are_noop() {
        // typeahead はユーザー打鍵文字列を DOM へ露出させないため keynav では
        // 実装しない（モジュール doc §Combobox・スコープ外事項参照）。
        assert_eq!(combobox_key_action("a", mods(), true), None);
        assert_eq!(combobox_key_action("ArrowLeft", mods(), true), None);
        assert_eq!(combobox_key_action("ArrowRight", mods(), true), None);
        assert_eq!(combobox_key_action("Tab", mods(), true), None);
    }

    #[test]
    fn combobox_key_action_rejects_modifier_keys_open_and_closed() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        for is_open in [false, true] {
            assert_eq!(combobox_key_action("ArrowDown", ctrl, is_open), None);
            assert_eq!(combobox_key_action("Enter", ctrl, is_open), None);
            assert_eq!(combobox_key_action("Escape", ctrl, is_open), None);
        }
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、native の
// `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （events.rs/hydration.rs/dom.rs と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        accordion_next_index, combobox_key_action, first_non_disabled, highlight_next_index,
        is_typeahead_key, last_non_disabled, loop_focus_from_attr, menu_loop_focus_from_attr,
        radio_next_index, submenu_nav, tabs_next_index, typeahead_next_index, typeahead_push,
        ComboboxKeyAction, Modifiers, Orientation, SubmenuNav, MAX_SUBMENU_DEPTH,
        TYPEAHEAD_TIMEOUT_MS,
    };
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlElement, HtmlInputElement, KeyboardEvent};

    /// `[data-scope="tabs"][data-part="trigger"]` セレクタ。
    const TABS_TRIGGER_SELECTOR: &str = "[data-scope=\"tabs\"][data-part=\"trigger\"]";
    /// `[data-scope="accordion"][data-part="item-trigger"]` セレクタ。
    const ACCORDION_TRIGGER_SELECTOR: &str =
        "[data-scope=\"accordion\"][data-part=\"item-trigger\"]";
    /// `[data-scope="menu"][data-part="trigger"]` セレクタ。
    const MENU_TRIGGER_SELECTOR: &str = "[data-scope=\"menu\"][data-part=\"trigger\"]";
    /// `[data-scope="menu"][data-part="content"]` セレクタ。
    const MENU_CONTENT_SELECTOR: &str = "[data-scope=\"menu\"][data-part=\"content\"]";
    /// `[data-scope="menu"][data-part="item"]`/`[data-scope="menu"][data-part="trigger-item"]`
    /// セレクタ（いずれも highlight 対象、モジュール doc §Menu/Select 参照）。
    const MENU_ITEM_SELECTOR: &str =
        "[data-scope=\"menu\"][data-part=\"item\"], [data-scope=\"menu\"][data-part=\"trigger-item\"]";
    /// `[data-scope="menu"][data-part="trigger-item"]` セレクタ（サブメニューを
    /// 開くための menu item、`crates/headless-ui/src/menu.rs::trigger_item` の
    /// SSR 出力。イシュー #662、アクティブ content チェーン解決・
    /// ArrowRight/ArrowLeft の対象判定に使う）。
    const TRIGGER_ITEM_SELECTOR: &str = "[data-scope=\"menu\"][data-part=\"trigger-item\"]";
    /// `[data-scope="select"][data-part="trigger"]` セレクタ。
    const SELECT_TRIGGER_SELECTOR: &str = "[data-scope=\"select\"][data-part=\"trigger\"]";
    /// `[data-scope="select"][data-part="content"]` セレクタ。
    const SELECT_CONTENT_SELECTOR: &str = "[data-scope=\"select\"][data-part=\"content\"]";
    /// `[data-scope="select"][data-part="item"]` セレクタ。
    const SELECT_ITEM_SELECTOR: &str = "[data-scope=\"select\"][data-part=\"item\"]";
    /// `[data-scope="radio-group"][data-part="root"]` セレクタ。
    const RADIO_GROUP_ROOT_SELECTOR: &str = "[data-scope=\"radio-group\"][data-part=\"root\"]";
    /// `[data-scope="radio-group"][data-part="item-hidden-input"]` セレクタ
    /// （ネイティブ `<input type="radio">`、キーボード操作・change 監視の対象）。
    const RADIO_GROUP_INPUT_SELECTOR: &str =
        "[data-scope=\"radio-group\"][data-part=\"item-hidden-input\"]";
    /// `[data-scope="radio-group"][data-part="item"]` セレクタ。
    const RADIO_GROUP_ITEM_SELECTOR: &str = "[data-scope=\"radio-group\"][data-part=\"item\"]";
    /// `[data-scope="radio-group"][data-part="item-control"]` セレクタ。
    const RADIO_GROUP_ITEM_CONTROL_SELECTOR: &str =
        "[data-scope=\"radio-group\"][data-part=\"item-control\"]";
    /// `[data-scope="radio-group"][data-part="item-text"]` セレクタ。
    const RADIO_GROUP_ITEM_TEXT_SELECTOR: &str =
        "[data-scope=\"radio-group\"][data-part=\"item-text\"]";
    /// `[data-scope="combobox"][data-part="input"]` セレクタ（イシュー #1071。
    /// `crates/headless-ui/src/combobox.rs::input`、`role="combobox"` を持つ
    /// テキストフィールド。keydown の監視対象かつ `aria-activedescendant` の
    /// 書き込み先）。
    const COMBOBOX_INPUT_SELECTOR: &str = "[data-scope=\"combobox\"][data-part=\"input\"]";
    /// `[data-scope="combobox"][data-part="trigger"]` セレクタ（イシュー
    /// #1071。`tabindex="-1"` 固定でフォーカスを受けないため
    /// [`matching_keydown_target`] には登録しない。開閉の `click()` 合成先
    /// としてのみ使う）。
    const COMBOBOX_TRIGGER_SELECTOR: &str = "[data-scope=\"combobox\"][data-part=\"trigger\"]";
    /// `[data-scope="combobox"][data-part="content"]` セレクタ（イシュー
    /// #1071）。
    const COMBOBOX_CONTENT_SELECTOR: &str = "[data-scope=\"combobox\"][data-part=\"content\"]";
    /// `[data-scope="combobox"][data-part="item"]` セレクタ（イシュー
    /// #1071）。
    const COMBOBOX_ITEM_SELECTOR: &str = "[data-scope=\"combobox\"][data-part=\"item\"]";

    /// `element.closest(selector)` の失敗（`Err`）・不一致（`None`）をまとめて
    /// `None` として扱う薄いヘルパ。DOM API のクエリ不正は本モジュールの
    /// 責務外の異常系であり、安全側 no-op とする。
    fn closest(element: &Element, selector: &str) -> Option<Element> {
        element.closest(selector).ok().flatten()
    }

    /// `list_or_root` 配下の `part_selector` に一致する要素を出現順に
    /// `Vec<Element>` として集める。`query_selector_all` の失敗は空 `Vec`
    /// として扱う（fail-closed、panic しない）。
    fn collect_parts(list_or_root: &Element, part_selector: &str) -> Vec<Element> {
        let Ok(node_list) = list_or_root.query_selector_all(part_selector) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(element) = node.dyn_into::<Element>() {
                    out.push(element);
                }
            }
        }
        out
    }

    /// 各要素の disabled 状態（ネイティブ `disabled` 属性または
    /// `data-disabled` 属性の存在）を列挙する。
    fn disabled_flags(elements: &[Element]) -> Vec<bool> {
        elements
            .iter()
            .map(|el| el.has_attribute("disabled") || el.has_attribute("data-disabled"))
            .collect()
    }

    /// `elements` 中で `target` と同一の要素のインデックスを探す
    /// （`Element::is_same_node` 相当を `Node::contains`/`==` ではなく
    /// `is_same_node` で判定し、テキストノード等の混入を避ける）。
    fn index_of(elements: &[Element], target: &Element) -> Option<usize> {
        elements.iter().position(|el| el.is_same_node(Some(target)))
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`）。本モジュールが書き込む属性
    /// （`tabindex`/`aria-selected`/`data-state`/`hidden`）はいずれも
    /// `&'static str` リテラルで固定された非 URL・非イベントハンドラ属性
    /// であり実害はないが、`fandhe_frontend_core::url` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が
    /// 動的な入力から組み立てられるよう変更された場合の防御としても
    /// 機能する（`wasm-client::binding_dom` の `set_attribute` 呼び出しと
    /// 同じガード方針）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) {
        if fandhe_frontend_core::is_event_handler_attr(name) {
            return;
        }
        if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
            return;
        }
        if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
            return;
        }
        let _ = element.set_attribute(name, value);
    }

    /// roving tabindex（`tabindex="0"`/`"-1"`）をフォーカス対象
    /// `active_index` に追従させる。書き込み失敗（`Err`）は個々の要素に
    /// 限定した安全側 no-op とし、他要素の更新は継続する。
    fn set_roving_tabindex(triggers: &[Element], active_index: usize) {
        for (i, trigger) in triggers.iter().enumerate() {
            let value = if i == active_index { "0" } else { "-1" };
            set_dom_attribute(trigger, "tabindex", value);
        }
    }

    /// Tabs の活性化（`aria-selected`/`data-state`/`hidden`）を
    /// `active_index` の trigger/content へ反映する。クリック委譲・
    /// automatic activationMode の keydown の双方から共通で呼ばれる
    /// （モジュール doc §Tabs 参照）。`aria-controls` から
    /// `document.get_element_by_id` で対応 content を解決できない場合、
    /// その trigger の content 更新のみ no-op とする（fail-closed）。
    fn activate_tab(document: &web_sys::Document, triggers: &[Element], active_index: usize) {
        for (i, trigger) in triggers.iter().enumerate() {
            let is_active = i == active_index;
            set_dom_attribute(
                trigger,
                "aria-selected",
                if is_active { "true" } else { "false" },
            );
            set_dom_attribute(
                trigger,
                "data-state",
                if is_active { "active" } else { "inactive" },
            );
            let Some(controls_id) = trigger.get_attribute("aria-controls") else {
                continue;
            };
            let Some(content) = document.get_element_by_id(&controls_id) else {
                continue;
            };
            set_dom_attribute(
                &content,
                "data-state",
                if is_active { "active" } else { "inactive" },
            );
            if is_active {
                let _ = content.remove_attribute("hidden");
            } else {
                set_dom_attribute(&content, "hidden", "");
            }
        }
    }

    /// `event` の修飾キー状態を [`Modifiers`] へ変換する薄いアダプタ。
    fn modifiers_of(event: &KeyboardEvent) -> Modifiers {
        Modifiers {
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        }
    }

    /// Tabs trigger 上の keydown を処理する。root 封じ込め検査
    /// （`root.contains`）・disabled 除外・純粋層（[`tabs_next_index`]）への
    /// 委譲・DOM 反映（roving tabindex・フォーカス移動・automatic activation）
    /// をこの 1 関数にまとめる。
    fn handle_tabs_keydown(root: &Element, target: &Element, event: &KeyboardEvent) {
        let Some(list) = closest(target, "[data-part=\"list\"]") else {
            return;
        };
        if !root.contains(Some(&list)) {
            return;
        }
        let triggers = collect_parts(&list, TABS_TRIGGER_SELECTOR);
        let Some(current) = index_of(&triggers, target) else {
            return;
        };
        let disabled = disabled_flags(&triggers);
        let orientation = Orientation::from_attr(list.get_attribute("data-orientation").as_deref());
        let loop_focus = loop_focus_from_attr(list.get_attribute("data-loop-focus").as_deref());
        let modifiers = modifiers_of(event);

        let Some(next_index) = tabs_next_index(
            current,
            &event.key(),
            orientation,
            loop_focus,
            modifiers,
            &disabled,
        ) else {
            return;
        };

        event.prevent_default();
        set_roving_tabindex(&triggers, next_index);
        if let Some(next_element) = triggers.get(next_index) {
            if let Ok(html_element) = next_element.clone().dyn_into::<HtmlElement>() {
                let _ = html_element.focus();
            }
        }

        let is_manual = list.get_attribute("data-activation-mode").as_deref() == Some("manual");
        if !is_manual {
            if let Some(document) = target.owner_document() {
                activate_tab(&document, &triggers, next_index);
            }
        }
    }

    /// Accordion item-trigger 上の keydown を処理する。root 封じ込め検査・
    /// disabled 除外・純粋層（[`accordion_next_index`]）への委譲・フォーカス
    /// 移動のみを行う（roving tabindex 更新・活性化は行わない、モジュール
    /// doc §Accordion 参照）。
    fn handle_accordion_keydown(root: &Element, target: &Element, event: &KeyboardEvent) {
        let Some(accordion_root) = closest(target, "[data-part=\"root\"]") else {
            return;
        };
        if !root.contains(Some(&accordion_root)) {
            return;
        }
        let triggers = collect_parts(&accordion_root, ACCORDION_TRIGGER_SELECTOR);
        let Some(current) = index_of(&triggers, target) else {
            return;
        };
        let disabled = disabled_flags(&triggers);
        let modifiers = modifiers_of(event);

        let Some(next_index) = accordion_next_index(current, &event.key(), modifiers, &disabled)
        else {
            return;
        };

        event.prevent_default();
        if let Some(next_element) = triggers.get(next_index) {
            if let Ok(html_element) = next_element.clone().dyn_into::<HtmlElement>() {
                let _ = html_element.focus();
            }
        }
    }

    /// Menu/Select の content 要素を解決する。`trigger` の `aria-controls` を
    /// 優先し、欠落・解決失敗時は `closest("[data-part=\"root\"]")` 配下の
    /// `content_selector` へフォールバックする（モジュール doc §Menu/Select
    /// 参照）。
    fn resolve_menu_select_content(trigger: &Element, content_selector: &str) -> Option<Element> {
        if let Some(controls_id) = trigger.get_attribute("aria-controls") {
            if let Some(document) = trigger.owner_document() {
                if let Some(content) = document.get_element_by_id(&controls_id) {
                    return Some(content);
                }
            }
        }
        let root_part = closest(trigger, "[data-part=\"root\"]")?;
        root_part.query_selector(content_selector).ok().flatten()
    }

    /// サブメニュー（`trigger_item`）が制御する子 Menu content を解決する
    /// （イシュー #662、Bugbot 指摘・PR #674 追補）。`crates/headless-ui/src/menu.rs`
    /// モジュール doc の「親 `Menu` インスタンスの `content` 内に子 `Menu`
    /// インスタンス由来の `trigger_item`/`positioner`/`content` を入れ子で
    /// 配置する」契約では、子の `positioner`/`content` は `trigger_item` の
    /// **兄弟**として親 content 直下に並ぶ配置が正当である（`trigger_item`
    /// の `aria-controls` も anatomy 上 optional）。3 段のフォールバックで
    /// この兄弟配置を第一級に解決する:
    /// 1. `aria-controls` による `document.get_element_by_id` 解決（最優先、
    ///    改ざん時のなりすまし対策は 2. の `root.contains` 封じ込めで担保）
    /// 2. `trigger_item` の子孫方向へ `MENU_CONTENT_SELECTOR` で解決
    ///    （旧実装からの後方互換フォールバック。子 content を `trigger_item`
    ///    自身の子孫へ配置する構成も引き続き許容する）
    /// 3. `trigger_item` の兄弟方向フォールバック（[`resolve_submenu_content_via_sibling`]、
    ///    新規）。1./2. がいずれも解決できない場合のみ試みる
    ///
    /// 解決結果は経路によらず必ず `root.contains` で封じ込め検査し、`root`
    /// 外を指す改ざん `aria-controls` は不採用として `None` を返す
    /// （fail-closed、A01 対策）。
    fn resolve_submenu_content(root: &Element, trigger_item: &Element) -> Option<Element> {
        let content = trigger_item
            .get_attribute("aria-controls")
            .and_then(|controls_id| {
                trigger_item
                    .owner_document()
                    .and_then(|document| document.get_element_by_id(&controls_id))
            })
            .or_else(|| {
                trigger_item
                    .query_selector(MENU_CONTENT_SELECTOR)
                    .ok()
                    .flatten()
            })
            .or_else(|| resolve_submenu_content_via_sibling(trigger_item))?;
        if root.contains(Some(&content)) {
            Some(content)
        } else {
            None
        }
    }

    /// [`resolve_submenu_content`] の第 3 フォールバック（兄弟方向、イシュー
    /// #662 Bugbot 指摘・PR #674 追補）。`trigger_item` の
    /// `next_element_sibling` チェーンを出現順に走査し、次の
    /// `[data-scope="menu"][data-part="trigger-item"]`（[`TRIGGER_ITEM_SELECTOR`]）
    /// に到達する**前**に現れる最初の子 Menu content を返す。
    ///
    /// - 候補要素自身が `MENU_CONTENT_SELECTOR` に一致すればそれを返す
    ///   （`content` を直接 `trigger_item` の兄弟に置く最小構成）。
    /// - 一致しない場合は候補要素の子孫方向へ `MENU_CONTENT_SELECTOR` を
    ///   探す（`positioner` ラッパー越しに `root`/`content` を包む構成、
    ///   `crates/headless-ui/src/menu.rs` の一般的な anatomy 配置）。
    /// - 次の trigger-item に到達したら**必ず打ち切る**。これを怠ると、
    ///   同一 content 直下に複数のサブメニューが並ぶ場合に、隣の
    ///   trigger-item のサブメニューを誤って自分のものとして解決して
    ///   しまう（`filter_own_scope_items` によるスコープ分離とは別に、
    ///   本関数自身が誤マッチを防ぐ必要がある）。
    ///
    /// 呼び出し元 [`resolve_submenu_content`] が結果を `root.contains` で
    /// 封じ込め検査するため、本関数自身は封じ込め判定を行わない。
    fn resolve_submenu_content_via_sibling(trigger_item: &Element) -> Option<Element> {
        let mut sibling = trigger_item.next_element_sibling();
        while let Some(current) = sibling {
            if current.matches(TRIGGER_ITEM_SELECTOR).unwrap_or(false) {
                break;
            }
            if current.matches(MENU_CONTENT_SELECTOR).unwrap_or(false) {
                return Some(current);
            }
            if let Some(descendant) = current.query_selector(MENU_CONTENT_SELECTOR).ok().flatten() {
                return Some(descendant);
            }
            sibling = current.next_element_sibling();
        }
        None
    }

    /// `items` 中で `data-highlighted` 属性を持つ要素のインデックスを探す
    /// （現在 highlight されている項目、モジュール doc §Menu/Select 参照）。
    fn find_highlighted_index(items: &[Element]) -> Option<usize> {
        items
            .iter()
            .position(|item| item.has_attribute("data-highlighted"))
    }

    /// `items` から、`content`（keydown を受けた Menu/Select 自身の content）に
    /// 属さない項目を除外する。`collect_parts` は `query_selector_all` で
    /// content 配下の subtree 全体を対象にするため、ネストしたサブメニュー
    /// （`trigger-item` が開く子 Menu の content）配下の item/trigger-item も
    /// 混入してしまう。各項目から `closest(content_selector)` で最も近い
    /// content 祖先を求め、それが `content` 自身と一致する項目のみを残すことで、
    /// 親の Arrow/Home/End・Enter/Space 操作がスコープ外のサブメニュー項目を
    /// 移動・アクティブ化するのを防ぐ（Bugbot 指摘、イシュー #583）。
    /// `content_selector` は呼び出し元（Menu/Select 共通実装）から渡される
    /// スコープ固有セレクタで、`MENU_CONTENT_SELECTOR` 固定にすると Select
    /// （`data-scope="select"`）側の項目がすべて誤って除外されてしまう。
    /// `closest` が失敗する（祖先に content が無い）場合も安全側で除外する。
    fn filter_own_scope_items(
        items: Vec<Element>,
        content: &Element,
        content_selector: &str,
    ) -> Vec<Element> {
        items
            .into_iter()
            .filter(|item| {
                closest(item, content_selector)
                    .is_some_and(|nearest| nearest.is_same_node(Some(content)))
            })
            .collect()
    }

    /// `top_content`（trigger 直下の Menu/Select content）から出発し、
    /// 「highlight 中の項目が `trigger-item` ∧ そのサブメニューが解決でき
    /// （[`resolve_submenu_content`]）∧ open（`hidden` なし）」の間、子孫方向へ
    /// 降下してアクティブ content を求める（イシュー #662、モジュール doc
    /// §サブメニュー参照）。ArrowDown/Up/Home/End・Enter/Space・typeahead・
    /// Escape の各既存キー処理は、サブメニューが開いている間その最深の
    /// content へルーティングされるべきであり、本関数はその「今どの階層が
    /// アクティブか」を DOM のみから都度導出する単一の入口である。
    ///
    /// 戻り値はアクティブ content と、深さ ≥ 1（サブメニュー内）のときのみ
    /// `Some` になる親 trigger-item（ArrowLeft の復帰先）。サブメニューが
    /// 一つも開いていない場合は `(top_content.clone(), None)` を返し、
    /// 既存の Menu/Select 単層キー処理と完全に同じ経路（`active_content`
    /// を素通しする）になる（既存の単層 Menu/Select の挙動を変えないための
    /// 不変条件）。
    ///
    /// 降下回数は [`MAX_SUBMENU_DEPTH`] で上限を設ける（改ざん DOM による
    /// `aria-controls` 循環参照からの無限ループ防止、A04 対策）。
    fn resolve_active_content(
        root: &Element,
        top_content: &Element,
        item_selector: &str,
        content_selector: &str,
    ) -> (Element, Option<Element>) {
        let mut active = top_content.clone();
        let mut parent_trigger_item: Option<Element> = None;
        for _ in 0..MAX_SUBMENU_DEPTH {
            let items = filter_own_scope_items(
                collect_parts(&active, item_selector),
                &active,
                content_selector,
            );
            let Some(highlighted_index) = find_highlighted_index(&items) else {
                break;
            };
            let highlighted = &items[highlighted_index];
            if !highlighted.matches(TRIGGER_ITEM_SELECTOR).unwrap_or(false) {
                break;
            }
            let Some(sub_content) = resolve_submenu_content(root, highlighted) else {
                break;
            };
            if sub_content.has_attribute("hidden") {
                break;
            }
            parent_trigger_item = Some(highlighted.clone());
            active = sub_content;
        }
        (active, parent_trigger_item)
    }

    /// `items[next_index]` のみへ `data-highlighted` を付与し、他項目からは
    /// 除去する。`activedescendant_host` の `aria-activedescendant` を
    /// highlight 対象の `id` へ更新し、`id` が欠落している場合は属性ごと
    /// 除去する（fail-safe、モジュール doc §Menu/Select 参照）。
    ///
    /// `activedescendant_host` は「`aria-activedescendant` を実際に読む側」
    /// （フォーカスを保持する要素）であり、Menu/Select では content 自身、
    /// Combobox では input（`crates/headless-ui/src/combobox.rs` の
    /// 「`aria-activedescendant` は input 側に配線する」契約、モジュール
    /// doc §Combobox 参照）と対象が異なるため引数として分離する
    /// （イシュー #1071 で [`set_highlight`] から抽出）。
    fn set_highlight_on_host(
        items: &[Element],
        next_index: usize,
        activedescendant_host: &Element,
    ) {
        for (i, item) in items.iter().enumerate() {
            if i == next_index {
                set_dom_attribute(item, "data-highlighted", "");
            } else {
                let _ = item.remove_attribute("data-highlighted");
            }
        }
        match items
            .get(next_index)
            .and_then(|item| item.get_attribute("id"))
        {
            Some(id) => set_dom_attribute(activedescendant_host, "aria-activedescendant", &id),
            None => {
                let _ = activedescendant_host.remove_attribute("aria-activedescendant");
            }
        }
    }

    /// [`set_highlight_on_host`] の薄いラッパー。Menu/Select は
    /// `aria-activedescendant` を content 自身へ書く既存契約のため、
    /// `activedescendant_host` に `content` をそのまま渡す（呼び出し側の
    /// シグネチャを変更しない、イシュー #1071 §並行実装との衝突対策）。
    fn set_highlight(items: &[Element], next_index: usize, content: &Element) {
        set_highlight_on_host(items, next_index, content)
    }

    /// `top_content` から [`resolve_active_content`] と同型の降下を行いながら、
    /// 通過するすべての階層（トップ content から最深のアクティブ content まで）
    /// の highlight を [`clear_highlight`] で消す（イシュー #662、Bugbot 指摘）。
    ///
    /// Escape は `active_content`（最深階層）のみを対象にすると、サブメニューが
    /// 開いている状態でその親 `trigger-item` の `data-highlighted` と親 content の
    /// `aria-activedescendant` が残留してしまい、#583 の「reopen 契約」
    /// （オーバーレイが閉じて再オープンした際、最初の Arrow キー操作は必ず
    /// 新規状態から始まる）を破る。本関数はチェーン上の全階層を一括で
    /// クリアすることでこれを保証する。降下回数は [`MAX_SUBMENU_DEPTH`] で
    /// 上限を設ける（`resolve_active_content` と同じ理由、A04 対策）。
    fn clear_active_chain_highlights(
        root: &Element,
        top_content: &Element,
        item_selector: &str,
        content_selector: &str,
    ) {
        let mut current = top_content.clone();
        for _ in 0..MAX_SUBMENU_DEPTH {
            let items = filter_own_scope_items(
                collect_parts(&current, item_selector),
                &current,
                content_selector,
            );
            let highlighted_index = find_highlighted_index(&items);
            clear_highlight(&items, &current);
            let Some(highlighted_index) = highlighted_index else {
                break;
            };
            let highlighted = &items[highlighted_index];
            if !highlighted.matches(TRIGGER_ITEM_SELECTOR).unwrap_or(false) {
                break;
            }
            let Some(sub_content) = resolve_submenu_content(root, highlighted) else {
                break;
            };
            if sub_content.has_attribute("hidden") {
                break;
            }
            current = sub_content;
        }
    }

    /// `items` すべてから `data-highlighted` を除去し、`content` の
    /// `aria-activedescendant` も除去する（[`set_highlight`] の逆操作）。
    ///
    /// Escape キーで highlight をクリアするために使う（モジュール doc
    /// §Menu/Select「Escape によるクローズ」節参照、Bugbot 指摘、イシュー
    /// #583）。**Menu/Select の実際の close（`hidden`/`data-state` の更新）は
    /// 依然として [`overlay`](crate::overlay) モジュール（#580 統合層）の責務であり、
    /// 本関数はキーボード配線層自身が書き込んだ highlight 表現の後始末のみを
    /// 行う（クローズそのものではない）**。呼び出し時点で content がまだ
    /// open のままでも副作用として問題はない（highlight 表示が一時的に消える
    /// だけで、fail-closed な no-op と同じ安全側の状態になる）。
    fn clear_highlight(items: &[Element], content: &Element) {
        clear_highlight_on_host(items, content)
    }

    /// [`clear_highlight`] の実体。`activedescendant_host` は
    /// [`set_highlight_on_host`] と同じ理由（Combobox は input、Menu/Select
    /// は content）で分離する（イシュー #1071）。
    fn clear_highlight_on_host(items: &[Element], activedescendant_host: &Element) {
        for item in items {
            let _ = item.remove_attribute("data-highlighted");
        }
        let _ = activedescendant_host.remove_attribute("aria-activedescendant");
    }

    /// item の表示ラベルを読み取り専用で解決する（typeahead のラベル比較
    /// 専用、イシュー #641）。`[data-part="item-text"]` 子（Select の
    /// item-indicator 混入を避けるため優先、`crates/headless-ui/src/select.rs`
    /// の anatomy 契約）があればその `text_content()` を使い、無ければ
    /// （Menu item のように子パーツを持たない場合）item 自身の
    /// `text_content()` へフォールバックする。DOM への書き戻しは行わない。
    ///
    /// `trigger-item` は子 `Menu` インスタンスの content（サブメニュー、
    /// [`MENU_CONTENT_SELECTOR`]）を自身の**子孫**として配置する構成
    /// （`resolve_submenu_content` 参照、後方互換フォールバック）も許容する
    /// ため、素朴に `text_content()` を使うとサブメニューが `hidden` でも
    /// 子孫アイテムのテキストまで拾ってしまい、親レベルの typeahead が
    /// `trigger-item` 自身のラベルではなく入れ子アイテムのテキストに誤マッチ
    /// してしまう（Bugbot 指摘、イシュー #662 PR #674）。
    /// `strip_nested_submenu_content` で（DOM への書き戻しを伴わない）
    /// クローン上からサブメニュー content を除去してから `text_content()`
    /// を読むことでこれを防ぐ（サブメニュー content が `trigger-item` の
    /// **兄弟**として配置される正当な構成では、そもそも子孫に含まれない
    /// ため本関数は no-op と同等に働く）。
    fn item_label(item: &Element) -> String {
        let text = item
            .query_selector("[data-part=\"item-text\"]")
            .ok()
            .flatten()
            .and_then(|el| el.text_content())
            .or_else(|| strip_nested_submenu_content(item).and_then(|el| el.text_content()))
            .unwrap_or_default();
        text.trim().to_string()
    }

    /// `item` の（DOM へ書き戻さない）ディープクローンを作り、その中に
    /// 含まれるサブメニュー content（[`MENU_CONTENT_SELECTOR`]、
    /// `trigger-item` が開く子 `Menu` インスタンスの content）をすべて
    /// 除去して返す（[`item_label`] 専用）。クローン操作自体が失敗した
    /// 場合は安全側として `None` を返し、呼び出し元は素の `text_content()`
    /// を使わず空文字列にフォールバックする（サブメニュー内容混入より
    /// ラベル取得失敗の方が安全、fail-closed）。
    fn strip_nested_submenu_content(item: &Element) -> Option<Element> {
        let clone: Element = item.clone_node_with_deep(true).ok()?.dyn_into().ok()?;
        if let Ok(nested) = clone.query_selector_all(MENU_CONTENT_SELECTOR) {
            for i in 0..nested.length() {
                if let Some(node) = nested.item(i) {
                    if let Ok(el) = node.dyn_into::<Element>() {
                        el.remove();
                    }
                }
            }
        }
        Some(clone)
    }

    /// Menu/Select の typeahead（文字キー入力による項目ジャンプ、イシュー
    /// #641）バッファを保持する状態。DOM から導出できない一時入力状態のため
    /// [`wire_keynav`] の keydown [`Closure`] が本構造体を所有する（モジュール
    /// doc §Menu/Select 参照。`data-*` 属性への書き出しは行わない例外——
    /// ユーザーの打鍵文字列そのものを DOM へ露出させる新規面を作らないため）。
    /// `content`（typeahead 対象の Menu/Select content 要素）を併せて保持し、
    /// 前回と異なる content 上での入力・タイムアウト超過時はバッファを
    /// 新規開始することで、同一 root 配下に複数の Menu/Select があっても
    /// 混線しない。
    struct TypeaheadState {
        buffer: String,
        last_time_stamp: f64,
        content: Option<Element>,
    }

    impl TypeaheadState {
        fn new() -> Self {
            Self {
                buffer: String::new(),
                last_time_stamp: 0.0,
                content: None,
            }
        }

        /// `content` 上で現時点（`now`、`KeyboardEvent::time_stamp()`）に
        /// typeahead バッファが有効（非空・タイムアウト内）かどうか。
        /// 対象 content が前回と異なる場合は無条件で無効（新規バッファ扱い）。
        fn is_active_for(&self, content: &Element, now: f64) -> bool {
            if self.buffer.is_empty() {
                return false;
            }
            if !self
                .content
                .as_ref()
                .is_some_and(|c| c.is_same_node(Some(content)))
            {
                return false;
            }
            (now - self.last_time_stamp) <= TYPEAHEAD_TIMEOUT_MS
        }

        /// バッファへ 1 文字追記し、更新後のバッファ文字列を返す。`content`
        /// が前回と異なる場合はタイムアウト超過と同じ扱い（[`typeahead_push`]
        /// へ `f64::INFINITY` を渡し新規バッファとして開始する）。
        fn push(&mut self, key: &str, now: f64, content: &Element) -> String {
            let same_content = self
                .content
                .as_ref()
                .is_some_and(|c| c.is_same_node(Some(content)));
            let elapsed = if same_content {
                now - self.last_time_stamp
            } else {
                f64::INFINITY
            };
            self.buffer = typeahead_push(&self.buffer, key, elapsed);
            self.last_time_stamp = now;
            self.content = Some(content.clone());
            self.buffer.clone()
        }

        /// バッファ・対象 content をリセットする（Escape・非 typeahead 経路の
        /// open 等、モジュール doc §Menu/Select 参照）。
        fn reset(&mut self) {
            self.buffer.clear();
            self.content = None;
        }

        /// closed-menu typeahead で `push` 後に click() 経由の再描画が
        /// 起きた場合、`content` を再解決後の要素へ同期する。これを怠ると
        /// 次の打鍵で [`is_active_for`](Self::is_active_for) が古い
        /// （破棄された）content と比較してしまい、タイムアウト内でも
        /// バッファが無効と誤判定されて新規クエリとして扱われてしまう
        /// （Bugbot 指摘: Stale content breaks typeahead buffer）。
        /// バッファ・タイムスタンプは変更しない。
        fn rebind_content(&mut self, content: &Element) {
            self.content = Some(content.clone());
        }
    }

    /// アクティブ content の highlight 中項目に対して Enter/Space の決定操作を
    /// 行う。項目が非 disabled・サブメニューが解決できる `trigger-item` の
    /// 場合は [`open_submenu_and_focus_first_item`] へ委譲してサブメニューを
    /// 展開したうえで新規 content の先頭項目へハイライトを移す（Bugbot 指摘
    /// "Enter opens submenu without entering"、イシュー #662）。それ以外は
    /// 従来通り highlight 中の項目へ `click()` のみを合成する。highlight
    /// 不在・disabled は no-op（fail-closed）。
    fn activate_or_open_submenu(
        root: &Element,
        active_content: &Element,
        item_selector: &str,
        content_selector: &str,
    ) {
        let items = filter_own_scope_items(
            collect_parts(active_content, item_selector),
            active_content,
            content_selector,
        );
        let Some(idx) = find_highlighted_index(&items) else {
            return;
        };
        let disabled = disabled_flags(&items);
        if disabled[idx] {
            return;
        }
        let item = &items[idx];
        if item.matches(TRIGGER_ITEM_SELECTOR).unwrap_or(false)
            && resolve_submenu_content(root, item).is_some()
        {
            open_submenu_and_focus_first_item(root, item, item_selector, content_selector);
        } else if let Ok(html_item) = item.clone().dyn_into::<HtmlElement>() {
            html_item.click();
        }
    }

    /// `trigger_item` へ `click()` を合成してサブメニューを開き、click 駆動の
    /// 再レンダー後に (1) `trigger_item` 自身の親チェーン highlight
    /// （`data-highlighted`/`aria-activedescendant`）を再付与し、(2) 展開した
    /// サブメニューの先頭非 disabled 項目へ highlight を設定する
    /// （ArrowRight・Enter/Space の submenu 展開経路から共通で呼ばれる、
    /// イシュー #662）。
    ///
    /// click() による再描画で `trigger_item` 自身を含む DOM ノードが差し替え
    /// られる可能性があるため、`id` を持つ場合は click 前に控えた `id` を使い
    /// `document.get_element_by_id` で"今の" trigger-item 要素を再解決してから
    /// 処理する（`ArrowLeft` の親 highlight 復帰と同型のパターン、モジュール
    /// doc 参照）。`resolve_active_content` は open chain 再構築にこの親
    /// highlight を必要とするため、これを怠ると `trigger_item` が置換された
    /// 場合に `ArrowLeft` で閉じられなくなり、以降のキー操作がトップレベル
    /// content にルーティングされたままサブメニューが開いた状態になる
    /// （Bugbot 指摘 "ArrowRight drops parent chain highlight"）。`headless-ui`
    /// は `trigger_item` の `id` を必須にしておらず、`id` が無い場合は click
    /// 前に保持していた `trigger_item` をそのまま使う（`id` 欠落を理由に
    /// highlight 移動自体を no-op にすると、サブメニューは開くのにハイライト
    /// が入らない不具合が再発する。Bugbot 指摘 "Missing id skips submenu
    /// entry"）。`id` 再解決の失敗・依然 closed はいずれも no-op
    /// （fail-closed）。
    fn open_submenu_and_focus_first_item(
        root: &Element,
        trigger_item: &Element,
        item_selector: &str,
        content_selector: &str,
    ) {
        let trigger_id = trigger_item.get_attribute("id");
        if let Ok(html_trigger_item) = trigger_item.clone().dyn_into::<HtmlElement>() {
            // 開閉は既存の click → `crate::headless`（`data-scope`/`data-part`
            // の静的マッピング表、`menu`/`trigger-item` → `"toggle"`）→
            // dispatch 経路（マウスクリックと同一経路）へ委譲する。
            html_trigger_item.click();
        }
        // click() 由来の再描画で `trigger_item` 自身を含む DOM ノードが
        // 差し替えられる可能性があるため、`id` を持つ場合は click 前に控えた
        // `id` で "今の" 要素を document.get_element_by_id 経由で再解決する
        // （ArrowLeft の親 highlight 復帰と同型のパターン、モジュール doc
        // 参照）。`headless-ui` は `trigger_item` の `id` を必須にしておらず
        // （anatomy 上 optional）、`id` が無い場合は再解決の手段が無いため、
        // click() 前に保持していた `trigger_item` をそのまま以降の解決に使う
        // （旧 ArrowRight 経路で `resolve_submenu_content` を元ノードへ直接
        // 適用していたのと同じ fallback）。ここで即 return してしまうと
        // サブメニューは開くのに highlight 移動が一切行われない不具合が
        // 再発する（Bugbot 指摘 "Missing id skips submenu entry"、イシュー
        // #662）。`id` 再解決に失敗した場合（要素が消失した等）のみ
        // no-op（fail-closed）とする。
        let resolved_trigger_item = match trigger_id.as_deref() {
            Some(id) => {
                let Some(document) = trigger_item.owner_document() else {
                    return;
                };
                let Some(fresh_trigger_item) = document.get_element_by_id(id) else {
                    return;
                };
                fresh_trigger_item
            }
            None => trigger_item.clone(),
        };
        // (1) 親チェーンの highlight を再付与する。
        if let Some(parent_content) = closest(&resolved_trigger_item, content_selector) {
            if root.contains(Some(&parent_content)) {
                let parent_items = filter_own_scope_items(
                    collect_parts(&parent_content, item_selector),
                    &parent_content,
                    content_selector,
                );
                if let Some(parent_index) =
                    parent_items
                        .iter()
                        .position(|item| match trigger_id.as_deref() {
                            Some(id) => item.get_attribute("id").as_deref() == Some(id),
                            None => item.is_same_node(Some(&resolved_trigger_item)),
                        })
                {
                    set_highlight(&parent_items, parent_index, &parent_content);
                }
            }
        }
        // (2) 展開後のサブメニュー先頭項目へ highlight を設定する。
        if let Some(sub_content_after) = resolve_submenu_content(root, &resolved_trigger_item) {
            if root.contains(Some(&sub_content_after)) && !sub_content_after.has_attribute("hidden")
            {
                let sub_items = filter_own_scope_items(
                    collect_parts(&sub_content_after, item_selector),
                    &sub_content_after,
                    content_selector,
                );
                let sub_disabled = disabled_flags(&sub_items);
                if let Some(idx) = first_non_disabled(&sub_disabled) {
                    set_highlight(&sub_items, idx, &sub_content_after);
                }
            }
        }
    }

    /// typeahead 1 手（1 文字追記 + マッチ項目への highlight 移動）を処理する
    /// 共通ヘルパ。closed 直後の初期 highlight（`current: None` 固定）・open
    /// 中の highlight 更新（`current` は現在の highlighted index）の双方から
    /// 呼ばれる。ラベルは [`item_label`] で読み取り専用に解決し、DOM への
    /// 書き戻しは行わない（XSS 対策、モジュール doc §セキュリティ不変条件
    /// 参照）。マッチが無い場合は `items`/`content` を変更しない。
    fn apply_typeahead_match(
        items: &[Element],
        content: &Element,
        current: Option<usize>,
        query: &str,
    ) {
        let labels: Vec<String> = items.iter().map(item_label).collect();
        let label_refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        let disabled = disabled_flags(items);
        if let Some(next_index) = typeahead_next_index(current, query, &label_refs, &disabled) {
            set_highlight(items, next_index, content);
        }
    }

    /// Menu/Select trigger 上の keydown を処理する（モジュール doc
    /// §Menu/Select 参照）。`content_selector`/`item_selector` で Menu/Select
    /// のいずれのスコープかを切り替える薄い共通実装。
    ///
    /// - content が closed のとき: ArrowDown/ArrowUp/Enter/Space で trigger へ
    ///   `click()` を合成して開く（`prevent_default`）。Enter/Space も
    ///   ArrowDown と同じく `prevent_default` した上で明示的に `click()` を
    ///   合成する（ネイティブ `<button>` の既定 click 発火に任せない）。
    ///   ネイティブ発火に任せると、本ハンドラが戻った後で非同期に click が
    ///   発火し初期 highlight を設定する機会がないまま open してしまい、
    ///   直後の Enter/Space が highlight 不在で no-op になる（Bugbot 指摘、
    ///   イシュー #583）。開いた直後、可能であれば content/items を再解決して
    ///   初期 highlight（ArrowDown・Enter・Space なら先頭、ArrowUp なら末尾の
    ///   非 disabled 項目）を設定する（再描画後の DOM 差し替えで解決に
    ///   失敗しても no-op、fail-closed）。
    /// - content が open のとき: ArrowDown/ArrowUp/Home/End で
    ///   [`highlight_next_index`] へ委譲し `data-highlighted`/
    ///   `aria-activedescendant` を更新する。Enter/Space（バッファ無効時）は
    ///   highlight 中の項目へ `click()` を合成する（disabled・highlight
    ///   不在は no-op）。ページスクロール抑止・trigger 自身の再クリック抑止の
    ///   ため、ハンドリング対象キーは常に `prevent_default` する。項目集合は
    ///   [`filter_own_scope_items`] でこの content 直下（ネストしたサブメニュー
    ///   content を除く）にスコープする。
    /// - typeahead（イシュー #641、[`is_typeahead_key`]/[`typeahead_next_index`]
    ///   参照）: closed 時は printable 文字キーで open + マッチ項目の初期
    ///   highlight、open 時はマッチ項目への highlight 移動を行う。バッファは
    ///   `typeahead`（[`TypeaheadState`]、[`wire_keynav`] が所有）に保持し、
    ///   Space はバッファが有効なときのみ typeahead 対象（無効時は従来通り
    ///   決定キー）。Escape でバッファをリセットする。
    fn handle_menu_or_select_trigger_keydown(
        root: &Element,
        trigger: &Element,
        event: &KeyboardEvent,
        content_selector: &str,
        item_selector: &str,
        typeahead: &mut TypeaheadState,
    ) {
        let Some(content) = resolve_menu_select_content(trigger, content_selector) else {
            return;
        };
        if !root.contains(Some(&content)) {
            return;
        }
        let modifiers = modifiers_of(event);
        if modifiers.any() {
            return;
        }
        let key = event.key();
        let is_open = !content.has_attribute("hidden");
        let now = event.time_stamp();
        let buffer_active = typeahead.is_active_for(&content, now);

        if !is_open {
            if is_typeahead_key(&key, buffer_active, modifiers) {
                event.prevent_default();
                if let Ok(html_trigger) = trigger.clone().dyn_into::<HtmlElement>() {
                    html_trigger.click();
                }
                let query = typeahead.push(&key, now, &content);
                // 開いた直後の初期 highlight。click() 経由の dispatch/再描画で
                // content/items が新しい要素に差し替わっている可能性があるため
                // 再解決する（解決失敗・依然 closed は no-op、fail-closed）。
                if let Some(content_after) = resolve_menu_select_content(trigger, content_selector)
                {
                    if root.contains(Some(&content_after)) && !content_after.has_attribute("hidden")
                    {
                        // `push` は再描画前の `content` を保持したため、次の
                        // 打鍵で [`TypeaheadState::is_active_for`] が新しい
                        // content と正しく照合できるよう同期する。
                        typeahead.rebind_content(&content_after);
                        let items = filter_own_scope_items(
                            collect_parts(&content_after, item_selector),
                            &content_after,
                            content_selector,
                        );
                        apply_typeahead_match(&items, &content_after, None, &query);
                        // マッチが無い場合は APG 既定（先頭の非 disabled 項目）へ
                        // フォールバックする。
                        if find_highlighted_index(&items).is_none() {
                            let disabled = disabled_flags(&items);
                            if let Some(idx) = first_non_disabled(&disabled) {
                                set_highlight(&items, idx, &content_after);
                            }
                        }
                    }
                }
                return;
            }

            // ArrowUp のみ末尾の非 disabled 項目を初期 highlight にする。
            // ArrowDown・Enter・Space はいずれも先頭から開始する
            // （WAI-ARIA APG Menu Button/Listbox パターン準拠）。
            let initial_from_end = key == "ArrowUp";
            let should_open =
                key == "ArrowDown" || key == "ArrowUp" || key == "Enter" || key == " ";
            if should_open {
                typeahead.reset();
                event.prevent_default();
                if let Ok(html_trigger) = trigger.clone().dyn_into::<HtmlElement>() {
                    html_trigger.click();
                }
                // 開いた直後の初期 highlight。click() 経由の dispatch/再描画で
                // content/items が新しい要素に差し替わっている可能性があるため
                // 再解決する（解決失敗・依然 closed は no-op、fail-closed）。
                if let Some(content_after) = resolve_menu_select_content(trigger, content_selector)
                {
                    if root.contains(Some(&content_after)) && !content_after.has_attribute("hidden")
                    {
                        let items = filter_own_scope_items(
                            collect_parts(&content_after, item_selector),
                            &content_after,
                            content_selector,
                        );
                        let disabled = disabled_flags(&items);
                        let initial = if initial_from_end {
                            last_non_disabled(&disabled)
                        } else {
                            first_non_disabled(&disabled)
                        };
                        if let Some(idx) = initial {
                            set_highlight(&items, idx, &content_after);
                        }
                    }
                }
            }
            return;
        }

        // ここから open。サブメニュー（`trigger-item`）が展開されている間は、
        // 以下すべてのキー操作を最深の open な content（アクティブ content）へ
        // ルーティングする（イシュー #662）。サブメニューが一つも開いて
        // いない単層 Menu/Select では `resolve_active_content` が常に
        // `(content, None)` を返すため、`active_content` は従来の `content`
        // と同一になり、以下の各腕は #583/#641 時点の挙動と完全に一致する
        // （モジュール doc §サブメニュー不変条件）。`buffer_active` も
        // トップレベルの `content` ではなく `active_content` を基準に
        // 再判定する（typeahead はアクティブな階層の項目を対象にすべきため）。
        let (active_content, parent_trigger_item) =
            resolve_active_content(root, &content, item_selector, content_selector);
        let buffer_active = typeahead.is_active_for(&active_content, now);

        match key.as_str() {
            "ArrowDown" | "ArrowUp" | "Home" | "End" => {
                // 非ループ既定動作で端に到達し highlight_next_index が None を
                // 返す場合でも、開いている間はこれらのキーを常にキャンセルする
                // （モジュール doc の契約）。preventDefault を怠るとページ
                // スクロールが素通りしてしまう（Bugbot 指摘、イシュー #583）。
                event.prevent_default();
                let items = filter_own_scope_items(
                    collect_parts(&active_content, item_selector),
                    &active_content,
                    content_selector,
                );
                let disabled = disabled_flags(&items);
                let current = find_highlighted_index(&items);
                let loop_focus = menu_loop_focus_from_attr(
                    active_content.get_attribute("data-loop-focus").as_deref(),
                );
                if let Some(next_index) =
                    highlight_next_index(current, &key, loop_focus, modifiers, &disabled)
                {
                    set_highlight(&items, next_index, &active_content);
                }
                // Arrow/Home/End によるナビゲーションは typeahead バッファを
                // 継続利用する状態から外れる操作のため、ここでバッファを
                // リセットする（Bugbot 指摘、イシュー #641）。リセットを
                // 怠ると `TYPEAHEAD_TIMEOUT_MS` 以内にナビゲーション後の
                // 文字入力が古いバッファへ追記され、誤ったクエリで検索が
                // 行われてしまう。
                typeahead.reset();
            }
            "Enter" => {
                event.prevent_default();
                activate_or_open_submenu(root, &active_content, item_selector, content_selector);
                // 選択確定後は typeahead バッファを継続する意味が無いため
                // リセットする（Bugbot 指摘、イシュー #641）。
                typeahead.reset();
            }
            " " if !buffer_active => {
                event.prevent_default();
                activate_or_open_submenu(root, &active_content, item_selector, content_selector);
                // Enter と同様、選択確定後はバッファをリセットする
                // （Bugbot 指摘、イシュー #641）。
                typeahead.reset();
            }
            "ArrowRight" if submenu_nav("ArrowRight", modifiers) == Some(SubmenuNav::Open) => {
                // アクティブ content の highlight 中項目が trigger-item であり、
                // 非 disabled・サブメニューが解決できるときのみ展開する
                // （それ以外は `prevent_default` せず no-op。Select は
                // trigger-item が存在せずセレクタ不一致で自然に no-op）。
                let items = filter_own_scope_items(
                    collect_parts(&active_content, item_selector),
                    &active_content,
                    content_selector,
                );
                let Some(highlighted_index) = find_highlighted_index(&items) else {
                    return;
                };
                let trigger_item = &items[highlighted_index];
                if !trigger_item.matches(TRIGGER_ITEM_SELECTOR).unwrap_or(false) {
                    return;
                }
                let disabled = disabled_flags(&items);
                if disabled[highlighted_index] {
                    return;
                }
                if resolve_submenu_content(root, trigger_item).is_none() {
                    return;
                }
                event.prevent_default();
                // 開閉は既存の click → `crate::headless`（`data-scope`/
                // `data-part` の静的マッピング表、`menu`/`trigger-item` →
                // `"toggle"`）→ dispatch 経路（マウスクリックと同一経路）へ
                // 委譲する（headless-ui は `data-action` を出力しないため
                // `events::wire_events` ではない）。keynav 自身は
                // `hidden`/`data-state`/`aria-expanded` を一切書かない
                // （モジュール doc §設計）。click 後の親チェーン highlight
                // 再付与・子メニュー先頭項目への highlight 設定は
                // [`open_submenu_and_focus_first_item`] に委譲する（Bugbot
                // 指摘 "ArrowRight drops parent chain highlight"、イシュー
                // #662）。
                open_submenu_and_focus_first_item(
                    root,
                    trigger_item,
                    item_selector,
                    content_selector,
                );
                typeahead.reset();
            }
            "ArrowLeft" if submenu_nav("ArrowLeft", modifiers) == Some(SubmenuNav::Close) => {
                // チェーン深さ 0（サブメニュー内ではない）は no-op。
                // `prevent_default` もしない（受け入れ条件 2、トップレベルの
                // ページ内既定動作を奪わない）。
                let Some(parent_trigger) = parent_trigger_item else {
                    return;
                };
                event.prevent_default();
                let items = filter_own_scope_items(
                    collect_parts(&active_content, item_selector),
                    &active_content,
                    content_selector,
                );
                clear_highlight(&items, &active_content);
                if let Ok(html_parent_trigger) = parent_trigger.clone().dyn_into::<HtmlElement>() {
                    // 開閉は click 合成で dispatch 経路へ委譲する（ArrowRight と
                    // 対称、モジュール doc §設計）。
                    html_parent_trigger.click();
                }
                // click() 後、親 content・親 trigger-item を再解決して
                // highlight を復帰させる。ArrowRight は再描画後に
                // `aria-controls` の id で子 content を再検索するのに対し、
                // ここで `closest`/`is_same_node` を click 前の `parent_trigger`
                // 要素へ適用すると、click → dispatch で親 content 側のノードが
                // 差し替わった場合に静かに失敗し親項目が再ハイライトされない
                // （Bugbot 指摘、イシュー #662）。そのため click 前に
                // `parent_trigger` の `id` 属性を控え、`id` がある場合は click
                // 後に `document.get_element_by_id` で改めて“今の” trigger-item
                // 要素を取得し、復帰先項目も `id` の一致で照合する
                // （ArrowRight の id ベース再解決と同型のパターン）。
                // `headless-ui` は `trigger_item` の `id` を必須にしておらず
                // （anatomy 上 optional）、`id` が無い場合は再解決の手段が
                // 無いため、click() 前に保持していた `parent_trigger` を
                // そのまま解決に使い、復帰先項目の照合も `is_same_node` で
                // 行う（`open_submenu_and_focus_first_item` の ArrowRight/
                // Enter 側で採用した fallback と同型）。ここで `id` 欠落を
                // 理由に highlight 復帰自体を no-op にすると、id なし
                // trigger-item を ArrowLeft で閉じた際に親項目が再ハイライト
                // されない不整合が残る（Bugbot 指摘 "ArrowLeft still requires
                // trigger id"、イシュー #662）。`id` 再解決（`id` がある場合
                // のみ）の失敗・親 content 未検出はいずれも no-op
                // （fail-closed）。
                let parent_trigger_id = parent_trigger.get_attribute("id");
                let resolved_parent_trigger = match parent_trigger_id.as_deref() {
                    Some(id) => parent_trigger
                        .owner_document()
                        .and_then(|document| document.get_element_by_id(id)),
                    None => Some(parent_trigger.clone()),
                };
                if let Some(fresh_parent_trigger) = resolved_parent_trigger {
                    if let Some(parent_content) = closest(&fresh_parent_trigger, content_selector) {
                        if root.contains(Some(&parent_content)) {
                            let parent_items = filter_own_scope_items(
                                collect_parts(&parent_content, item_selector),
                                &parent_content,
                                content_selector,
                            );
                            if let Some(parent_index) = parent_items.iter().position(|item| {
                                match parent_trigger_id.as_deref() {
                                    Some(id) => item.get_attribute("id").as_deref() == Some(id),
                                    None => item.is_same_node(Some(&fresh_parent_trigger)),
                                }
                            }) {
                                set_highlight(&parent_items, parent_index, &parent_content);
                            }
                        }
                    }
                }
                typeahead.reset();
            }
            "Escape" => {
                // Menu/Select 自体の close は [`overlay`](crate::overlay) モジュール
                // （#580 統合層）の責務のため、ここでは `hidden`/`data-state` を
                // 一切書き換えない。だが highlight（`data-highlighted`/
                // `aria-activedescendant`）は本モジュール自身が書き込む状態
                // であり、overlay 側は関知しないため、ここで放置すると
                // Escape → 再度マウス等で reopen → 最初の Arrow キーが古い
                // highlight から続いてしまう（Bugbot 指摘、イシュー #583）。
                // サブメニューが開いている場合、`active_content`（最深階層）
                // だけをクリアすると親 `trigger-item` の `data-highlighted` と
                // 親 content の `aria-activedescendant` が残留し、同じ #583 の
                // reopen 契約を破る（Bugbot 指摘、イシュー #662）。そのため
                // トップ content から `active_content` までのチェーン全体を
                // [`clear_active_chain_highlights`] で一括クリアする。
                // `prevent_default`/`stop_propagation` は呼ばない
                // （overlay.rs の document keydown リスナーが同じ Escape を
                // 引き続き観測して実際の close 判定を行える必要がある）。
                // typeahead バッファも合わせてリセットする（イシュー #641、
                // Escape 後の再入力は新規バッファから始まるべきため）。
                clear_active_chain_highlights(root, &content, item_selector, content_selector);
                typeahead.reset();
            }
            _ if is_typeahead_key(&key, buffer_active, modifiers) => {
                event.prevent_default();
                let query = typeahead.push(&key, now, &active_content);
                let items = filter_own_scope_items(
                    collect_parts(&active_content, item_selector),
                    &active_content,
                    content_selector,
                );
                let current = find_highlighted_index(&items);
                apply_typeahead_match(&items, &active_content, current, &query);
            }
            _ => {}
        }
    }

    /// Combobox root（`[data-part="root"]`）を `input` から解決する薄い
    /// ヘルパ（イシュー #1071）。click 駆動の再描画後に content/input/items
    /// を再解決する起点として使う（モジュール doc §Combobox、`input`
    /// ハンドル自体が再描画で差し替わりうるため、`aria-controls` を持つ
    /// `input` からではなく root からの再クエリに統一する）。
    fn resolve_combobox_root(input: &Element) -> Option<Element> {
        closest(input, "[data-part=\"root\"]")
    }

    /// `combobox_root` 配下の trigger（[`COMBOBOX_TRIGGER_SELECTOR`]）を
    /// 解決し、`root`（keynav がマウントされた封じ込め境界）内であることを
    /// 検査する（A01 対策、イシュー #1071）。`trigger` は `tabindex="-1"`
    /// 固定でフォーカスを受けないため [`matching_keydown_target`] には
    /// 登録されないが、開閉の `click()` 合成先として使う。
    fn resolve_combobox_trigger(root: &Element, combobox_root: &Element) -> Option<Element> {
        let trigger = combobox_root
            .query_selector(COMBOBOX_TRIGGER_SELECTOR)
            .ok()
            .flatten()?;
        if root.contains(Some(&trigger)) {
            Some(trigger)
        } else {
            None
        }
    }

    /// Combobox の `input`（`role="combobox"`）上の keydown を処理する
    /// （イシュー #1071、モジュール doc §Combobox 参照）。
    ///
    /// [`handle_menu_or_select_trigger_keydown`] を流用しない理由・typeahead
    /// を実装しない理由はモジュール doc §Combobox に記載。純粋層
    /// [`combobox_key_action`] へキー判定を委譲し、本関数は DOM 解決・
    /// 封じ込め検査・click 合成・highlight 反映のみを担う。
    fn handle_combobox_input_keydown(root: &Element, input: &Element, event: &KeyboardEvent) {
        if !root.contains(Some(input)) {
            return;
        }
        let Some(content) = resolve_menu_select_content(input, COMBOBOX_CONTENT_SELECTOR) else {
            return;
        };
        if !root.contains(Some(&content)) {
            return;
        }
        let modifiers = modifiers_of(event);
        let key = event.key();
        let is_open = !content.has_attribute("hidden");
        let Some(action) = combobox_key_action(&key, modifiers, is_open) else {
            return;
        };

        match action {
            ComboboxKeyAction::Open { from_end } => {
                let Some(combobox_root) = resolve_combobox_root(input) else {
                    return;
                };
                let Some(trigger) = resolve_combobox_trigger(root, &combobox_root) else {
                    return;
                };
                event.prevent_default();
                if let Ok(html_trigger) = trigger.clone().dyn_into::<HtmlElement>() {
                    html_trigger.click();
                }
                // click 駆動の再描画で content/input/items が新しい要素に
                // 差し替わりうるため、combobox root 配下から再解決する
                // （`input` は id 必須ではなく `aria-controls` 起点の
                // 再解決ができないため、モジュール doc §Combobox 参照）。
                // 再解決失敗・依然 closed はいずれも no-op（fail-closed）。
                let Some(content_after) = combobox_root
                    .query_selector(COMBOBOX_CONTENT_SELECTOR)
                    .ok()
                    .flatten()
                else {
                    return;
                };
                if !root.contains(Some(&content_after)) || content_after.has_attribute("hidden") {
                    return;
                }
                let Some(input_after) = combobox_root
                    .query_selector(COMBOBOX_INPUT_SELECTOR)
                    .ok()
                    .flatten()
                else {
                    return;
                };
                if !root.contains(Some(&input_after)) {
                    return;
                }
                let items = filter_own_scope_items(
                    collect_parts(&content_after, COMBOBOX_ITEM_SELECTOR),
                    &content_after,
                    COMBOBOX_CONTENT_SELECTOR,
                );
                let disabled = disabled_flags(&items);
                let initial = if from_end {
                    last_non_disabled(&disabled)
                } else {
                    first_non_disabled(&disabled)
                };
                if let Some(idx) = initial {
                    set_highlight_on_host(&items, idx, &input_after);
                }
            }
            ComboboxKeyAction::MoveHighlight => {
                event.prevent_default();
                let items = filter_own_scope_items(
                    collect_parts(&content, COMBOBOX_ITEM_SELECTOR),
                    &content,
                    COMBOBOX_CONTENT_SELECTOR,
                );
                let disabled = disabled_flags(&items);
                let current = find_highlighted_index(&items);
                let loop_focus =
                    menu_loop_focus_from_attr(content.get_attribute("data-loop-focus").as_deref());
                if let Some(next_index) =
                    highlight_next_index(current, &key, loop_focus, modifiers, &disabled)
                {
                    set_highlight_on_host(&items, next_index, input);
                }
            }
            ComboboxKeyAction::Confirm => {
                let items = filter_own_scope_items(
                    collect_parts(&content, COMBOBOX_ITEM_SELECTOR),
                    &content,
                    COMBOBOX_CONTENT_SELECTOR,
                );
                let Some(highlighted_index) = find_highlighted_index(&items) else {
                    return;
                };
                let disabled = disabled_flags(&items);
                if disabled[highlighted_index] {
                    return;
                }
                event.prevent_default();
                // 開閉と同様、確定も click → `crate::headless`（`combobox`/
                // `item` → `"select"`、本イシューで追加）→ dispatch 経路へ
                // 委譲する（モジュール doc §Combobox 参照）。
                if let Ok(html_item) = items[highlighted_index].clone().dyn_into::<HtmlElement>() {
                    html_item.click();
                }
            }
            ComboboxKeyAction::Close => {
                event.prevent_default();
                let items = filter_own_scope_items(
                    collect_parts(&content, COMBOBOX_ITEM_SELECTOR),
                    &content,
                    COMBOBOX_CONTENT_SELECTOR,
                );
                clear_highlight_on_host(&items, input);
                // trigger 解決失敗時は highlight クリアのみ行い close は
                // no-op（fail-closed、モジュール doc §Combobox 参照）。
                let Some(combobox_root) = resolve_combobox_root(input) else {
                    return;
                };
                let Some(trigger) = resolve_combobox_trigger(root, &combobox_root) else {
                    return;
                };
                if let Ok(html_trigger) = trigger.clone().dyn_into::<HtmlElement>() {
                    html_trigger.click();
                }
            }
        }
    }

    /// 項目 1 個分の RadioGroup `data-state`（`"checked"`/`"unchecked"`）を
    /// `item-hidden-input` 自身・祖先 `item`・その子孫 `item-control`/
    /// `item-text` へ同期する（`crates/headless-ui/src/radio_group.rs` の
    /// `DATA_STATE_CHECKED`/`DATA_STATE_UNCHECKED` 語彙と一致させる）。
    /// 祖先 `item`・子孫パーツが見つからない場合はその部分のみ no-op
    /// （fail-closed）。
    fn apply_radio_item_state(input: &Element, checked: bool) {
        let state_value = if checked { "checked" } else { "unchecked" };
        set_dom_attribute(input, "data-state", state_value);
        let Some(item) = closest(input, RADIO_GROUP_ITEM_SELECTOR) else {
            return;
        };
        set_dom_attribute(&item, "data-state", state_value);
        if let Ok(Some(control)) = item.query_selector(RADIO_GROUP_ITEM_CONTROL_SELECTOR) {
            set_dom_attribute(&control, "data-state", state_value);
        }
        if let Ok(Some(text)) = item.query_selector(RADIO_GROUP_ITEM_TEXT_SELECTOR) {
            set_dom_attribute(&text, "data-state", state_value);
        }
    }

    /// `inputs` 全体のネイティブ `checked` と `data-state` 群を
    /// `checked_index` のみが選択された状態に同期する
    /// （[`apply_radio_item_state`] を各項目へ適用）。
    fn sync_radio_group_states(inputs: &[Element], checked_index: usize) {
        for (i, input) in inputs.iter().enumerate() {
            let checked = i == checked_index;
            if let Ok(html_input) = input.clone().dyn_into::<HtmlInputElement>() {
                html_input.set_checked(checked);
            }
            apply_radio_item_state(input, checked);
        }
    }

    /// RadioGroup のネイティブ `<input type="radio">` 上の keydown を処理する
    /// （モジュール doc §RadioGroup 参照）。root 封じ込め検査・disabled 除外・
    /// 純粋層（[`radio_next_index`]）への委譲・フォーカス移動 + ネイティブ
    /// `checked` 設定 + `data-state` 群の同期をこの 1 関数にまとめる。
    fn handle_radio_keydown(root: &Element, input: &Element, event: &KeyboardEvent) {
        let Some(group_root) = closest(input, RADIO_GROUP_ROOT_SELECTOR) else {
            return;
        };
        if !root.contains(Some(&group_root)) {
            return;
        }
        let inputs = collect_parts(&group_root, RADIO_GROUP_INPUT_SELECTOR);
        let Some(current) = index_of(&inputs, input) else {
            return;
        };
        let disabled = disabled_flags(&inputs);
        let orientation = Orientation::from_attr_optional(
            group_root.get_attribute("data-orientation").as_deref(),
        );
        let modifiers = modifiers_of(event);
        let key = event.key();

        // ネイティブ radio グループ化（同一 name 属性）は data-orientation を
        // 知らず、4 方向いずれの矢印キーでもブラウザ既定でフォーカス・選択を
        // 移動させてしまう。orientation により radio_next_index が None を
        // 返す（却下される）場合も、このハンドラが対象とする Home/End/矢印
        // キーである限り常に prevent_default し、その既定動作を抑止する
        // （Bugbot 指摘、イシュー #583。修飾キー付きは対象外のまま）。
        let is_handled_key = matches!(
            key.as_str(),
            "Home" | "End" | "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown"
        );
        if !modifiers.any() && is_handled_key {
            event.prevent_default();
        }

        let Some(next_index) = radio_next_index(current, &key, orientation, modifiers, &disabled)
        else {
            return;
        };

        if let Some(next_input) = inputs.get(next_index) {
            if let Ok(html_input) = next_input.clone().dyn_into::<HtmlElement>() {
                let _ = html_input.focus();
            }
        }
        sync_radio_group_states(&inputs, next_index);
    }

    /// RadioGroup のネイティブ `change`（マウスクリック・ネイティブ Space 決定
    /// が発火する）を処理する。ブラウザが既に反映したネイティブ `checked` の
    /// 実態を読み取り、グループ全体の `data-state` 群を追随させるのみで
    /// `checked` 自体は変更しない（モジュール doc §RadioGroup 参照）。
    fn handle_radio_change(root: &Element, changed_input: &Element) {
        let Some(group_root) = closest(changed_input, RADIO_GROUP_ROOT_SELECTOR) else {
            return;
        };
        if !root.contains(Some(&group_root)) {
            return;
        }
        let inputs = collect_parts(&group_root, RADIO_GROUP_INPUT_SELECTOR);
        for input in &inputs {
            let checked = input
                .clone()
                .dyn_into::<HtmlInputElement>()
                .map(|html_input| html_input.checked())
                .unwrap_or(false);
            apply_radio_item_state(input, checked);
        }
    }

    /// Tabs trigger クリック（マウスクリック・ネイティブ button の
    /// Enter/Space が発火する click イベントの双方）による活性化を処理する。
    /// disabled trigger のクリックは no-op（fail-closed。ネイティブ
    /// `disabled` 属性がある場合、ブラウザは通常 click 自体を発火しないが、
    /// 念のため二重に防御する）。
    fn handle_trigger_click(root: &Element, target: &Element) {
        let Some(list) = closest(target, "[data-part=\"list\"]") else {
            return;
        };
        if !root.contains(Some(&list)) {
            return;
        }
        let triggers = collect_parts(&list, TABS_TRIGGER_SELECTOR);
        let Some(index) = index_of(&triggers, target) else {
            return;
        };
        if disabled_flags(&triggers)[index] {
            return;
        }
        set_roving_tabindex(&triggers, index);
        if let Some(document) = target.owner_document() {
            activate_tab(&document, &triggers, index);
        }
    }

    /// キーボードイベントのターゲットを、Tabs trigger / Accordion
    /// item-trigger / Menu trigger / Select trigger / RadioGroup ネイティブ
    /// `<input type="radio">` のいずれかに一致する要素として解決する
    /// （返り値の `&'static str` はスコープ識別子）。`matches` の失敗
    /// （不正セレクタ等）は不一致として扱う。Menu/Select/RadioGroup は
    /// いずれも keydown 時に実 DOM フォーカスがそのままターゲット
    /// （trigger button / radio input）上にあるため、Tabs/Accordion と同じく
    /// `closest` を介さず `target` 自身の一致判定のみで足りる
    /// （モジュール doc §Menu/Select/RadioGroup 参照）。
    fn matching_keydown_target(target: &Element) -> Option<(&'static str, Element)> {
        if target.matches(TABS_TRIGGER_SELECTOR).unwrap_or(false) {
            return Some(("tabs", target.clone()));
        }
        if target.matches(ACCORDION_TRIGGER_SELECTOR).unwrap_or(false) {
            return Some(("accordion", target.clone()));
        }
        if target.matches(MENU_TRIGGER_SELECTOR).unwrap_or(false) {
            return Some(("menu", target.clone()));
        }
        if target.matches(SELECT_TRIGGER_SELECTOR).unwrap_or(false) {
            return Some(("select", target.clone()));
        }
        if target.matches(RADIO_GROUP_INPUT_SELECTOR).unwrap_or(false) {
            return Some(("radio", target.clone()));
        }
        // Combobox は input（`role="combobox"`）が実 DOM フォーカスを保持する
        // （trigger は `tabindex="-1"` 固定でフォーカスを受けないため登録
        // しない、モジュール doc §Combobox 参照、イシュー #1071）。
        if target.matches(COMBOBOX_INPUT_SELECTOR).unwrap_or(false) {
            return Some(("combobox", target.clone()));
        }
        None
    }

    /// ルート要素へ `keydown` / `click` / `change` の委譲リスナーをマウント時に
    /// 1 回だけ登録する（`Closure::forget` は 3 回のみ、モジュール doc
    /// §設計参照。[`events::wire_events`] と合わせても定数個）。
    ///
    /// - `keydown`: イベントターゲットが Tabs trigger / Accordion
    ///   item-trigger / Menu trigger / Select trigger / RadioGroup ネイティブ
    ///   `<input type="radio">` / Combobox `input`（イシュー #1071）の
    ///   いずれかに一致する場合のみ処理する
    ///   （[`handle_tabs_keydown`]/[`handle_accordion_keydown`]/
    ///   [`handle_menu_or_select_trigger_keydown`]/[`handle_radio_keydown`]/
    ///   [`handle_combobox_input_keydown`]）。
    /// - `click`: Tabs trigger への委譲クリックで [`handle_trigger_click`]
    ///   を呼び、マウスクリック・manual activationMode 下の Enter/Space の
    ///   双方をカバーする（Menu/Select の決定はキーボード側で highlight 中
    ///   項目へ `click()` を合成する設計のため、本リスナーでの追加処理は
    ///   不要）。
    /// - `change`: RadioGroup のネイティブ `<input type="radio">` の
    ///   `change`（マウスクリック・ネイティブ Space 決定）を
    ///   [`handle_radio_change`] で `data-state` 群へ同期する。
    ///
    /// `root` より外側の要素にヒットした場合は採用しない（`contains` 検査、
    /// [`events::wire_events`] と同じ封じ込め）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` が失敗した場合に `Err` を返す。
    pub fn wire_keynav(root: Element) -> Result<(), JsValue> {
        let keydown_root = root.clone();
        // typeahead バッファ（イシュー #641）は DOM から導出できない一時入力
        // 状態のため、本 keydown [`Closure`]（`FnMut`）が所有する。root 配下の
        // 全 Menu/Select に対し 1 個を共有し、対象 content が変わったときの
        // 混線防止は [`TypeaheadState`] 自身が担う（`TypeaheadState` doc 参照）。
        let mut typeahead_state = TypeaheadState::new();
        let keydown_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Ok(keyboard_event) = event.clone().dyn_into::<KeyboardEvent>() else {
                return;
            };
            let Some(target) = event.target() else {
                return;
            };
            let Some(target_element) = target.dyn_ref::<Element>().cloned() else {
                return;
            };
            if !keydown_root.contains(Some(&target_element)) {
                return;
            }
            let Some((scope, matched)) = matching_keydown_target(&target_element) else {
                return;
            };
            match scope {
                "tabs" => handle_tabs_keydown(&keydown_root, &matched, &keyboard_event),
                "accordion" => handle_accordion_keydown(&keydown_root, &matched, &keyboard_event),
                "menu" => handle_menu_or_select_trigger_keydown(
                    &keydown_root,
                    &matched,
                    &keyboard_event,
                    MENU_CONTENT_SELECTOR,
                    MENU_ITEM_SELECTOR,
                    &mut typeahead_state,
                ),
                "select" => handle_menu_or_select_trigger_keydown(
                    &keydown_root,
                    &matched,
                    &keyboard_event,
                    SELECT_CONTENT_SELECTOR,
                    SELECT_ITEM_SELECTOR,
                    &mut typeahead_state,
                ),
                "radio" => handle_radio_keydown(&keydown_root, &matched, &keyboard_event),
                // typeahead 非適用（モジュール doc §Combobox 参照、イシュー
                // #1071）のため `TypeaheadState` を渡さない。
                "combobox" => {
                    handle_combobox_input_keydown(&keydown_root, &matched, &keyboard_event)
                }
                _ => {}
            }
        });
        root.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
        keydown_closure.forget();

        let click_root = root.clone();
        let click_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            // `event.target()` はクリックされた最も深いノードを指し、Tabs
            // trigger のテキストラベル（`fandhe_frontend_core::text` が生成する
            // テキストノード）であることがある。テキストノードは `Element` では
            // ないため `dyn_ref::<Element>()` は `None` を返すが、これは
            // 「フレームワーク管轄外のクリック」ではなく「祖先探索の起点を
            // 要素まで遡る必要がある」ケースである。`events::wire_events` と
            // 同方針で `Node::parent_element()` により直近の親要素へ遡ってから
            // `closest` を呼び、テキストラベルクリックでも trigger 祖先探索を
            // 取りこぼさないようにする（Cursor Bugbot 指摘、PR #612）。
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
            if !click_root.contains(Some(&target_element)) {
                return;
            }
            let Ok(Some(matched)) = target_element.closest(TABS_TRIGGER_SELECTOR) else {
                return;
            };
            if !click_root.contains(Some(&matched)) {
                return;
            }
            handle_trigger_click(&click_root, &matched);
        });
        root.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        // RadioGroup のネイティブ `<input type="radio">` の `change`
        // （マウスクリック・ネイティブ Space 決定の双方で発火、バブリングする
        // ため他モジュール（headless_avatar の load/error）と異なり capture
        // フェーズ不要）を委譲する。`data-state` 群の同期のみを行い、
        // `checked` 自体はブラウザのネイティブ挙動に委ねる
        // （[`handle_radio_change`] 参照）。
        let change_root = root.clone();
        let change_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Some(target_element) = target.dyn_ref::<Element>().cloned() else {
                return;
            };
            if !change_root.contains(Some(&target_element)) {
                return;
            }
            if !target_element
                .matches(RADIO_GROUP_INPUT_SELECTOR)
                .unwrap_or(false)
            {
                return;
            }
            handle_radio_change(&change_root, &target_element);
        });
        root.add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
        change_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_keynav;
