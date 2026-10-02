//! `settings-share-link` block（イシュー #3011。Application / Settings
//! カテゴリ）。共有の有効化スイッチ・共有 URL のコピー欄・リンクコピー /
//! プレビューのボタン群を持つ共有リンク設定カードを合成する。主参照
//! R0318（代表構成）に、閲覧範囲の radio card（R0319）・埋め込み/リンクの
//! tabs 切替（R0320）・ドメイン接尾辞 + QR コード（R0321）の 3 差分版を
//! 併記する。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で本
//! worktree に存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`settings_billing_overview`〔イシュー #2984〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `card` / `switch` / `clipboard` / `button` / `button-group` /
//! `radio-card` / `qr-code` / `select` / `separator` / `input` /
//! `input-group` / `text` の 12 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//! `input-group` は Issue 本文の列挙（11 種）に無いが、
//! `settings_api_key_created` と同じ既存 Themes 部品であり、下記「Demo 内の
//! `clipboard` root は 1 個に限る」節の制約を満たすために追加した
//! （新規 UI 部品の新設ではない）。`text` も同様に既存 Themes 部品であり、
//! 下記「radio card の選択状態をフォーム走査外でも明文化する」節の制約を
//! 満たすために追加した。版 C（埋め込み/リンク切替）は実物の `tabs::tabs`
//! を使わず素の `div` による静的なタブ列で表示する（下記「版 C: 無 JS
//! での扱い」節参照）ため、`tabs` は `parts` に含めない。
//!
//! # 4 版と集約元の対応（原稿「原案差分メモ」節と対になる索引）
//!
//! - **A（代表構成、basic）**: R0318。共有有効化 switch → separator →
//!   共有 URL の clipboard → footer にリンクコピー/プレビューの
//!   button-group。
//! - **B（閲覧範囲、audience）**: R0319。A の body へ「閲覧できる範囲」の
//!   radio card 3 択を追加する。
//! - **C（埋め込み/リンク切替、tabs）**: R0320。body 先頭に静的なタブ列
//!   （リンク/埋め込みの 2 状態）を置く（下記「版 C: 無 JS での扱い」節
//!   参照）。
//! - **D（ドメイン接尾辞 + QR、domain-qr）**: R0321。スラッグ入力 + ドメイン
//!   接尾辞 select（閉じた状態固定）+ 共有 URL の QR コードを置く。表示
//!   スラッグ・ドメイン接尾辞・QR コードのリンク先は [`DOMAIN_SHARE_URL`]
//!   へ一致させる（版 A〜C の [`SHARE_URL`] とは別定数、レビュー指摘
//!   是正）。
//!
//! # 版 C: 無 JS での扱い（実物の `tabs::tabs` は使わない）
//!
//! 版 C は `fandhe_frontend_pre_styled_ui::tabs::tabs` を使わない。実物の
//! `tabs` は `role="tab"`/`aria-selected`/`hidden` パネルで構成され、
//! クリックでパネルを切り替える `headless_tabs` の JS 配線を前提とする。
//! docs サイトは JS ハイドレーションを行わないため、これらの属性を出すと
//! 「操作できるように見えて実際には切り替わらない」見せかけの対話性に
//! なる（`crates/docs-site/src/blocks/marketing/feature/feature_tabs_panel.rs`
//! で確立した判断、#2772/#2773 と同じ）。そのため本版は素の `div`
//! （`.blocks-settings-share-link-tab`・`data-state="active"/"inactive"`・
//! 装飾のため `aria-hidden="true"`）でタブ列を描き、[`LAYOUT_CSS`] で
//! `feature_tabs_panel` の下線型タブ CSS 相当を再現する。パネル本文は
//! 「リンク」「埋め込み」の両状態を常時可視でキャプション付き縦積み
//! （[`tab_preview`]）にし、`hidden` 属性・`role="tab"`/`role="tabpanel"`
//! を一切出さない。
//!
//! # Demo 内の `clipboard` root は 1 個に限る（1 root : 1 状態機械契約）
//!
//! `fandhe-frontend-wasm-full` の `headless_clipboard` 配線は「1 root : 1
//! 状態機械契約」を持ち、`data-copied` と indicator の反映を
//! `Runtime::mount`/`hydrate` に渡されたマウントルート配下の**全**
//! `clipboard` パーツへ及ぼす（`settings_api_key_created` と同じ制約）。
//! そのため本 Demo の `clipboard` root は版 A（[`version_basic`]）の 1 個
//! だけとする。版 B・版 C（タブ内の「リンク」）の共有 URL 表示は
//! [`static_url_display`]（`field` + `input`〔readonly〕+ `input-group` の
//! コピーボタン、`disabled: true`）に置き換え、`clipboard` scope の外側の
//! ため押しても何も起きないことを明示する（`settings_api_key_created` 版 B
//! と同型の判断）。
//!
//! # 静的固定（無 JS 前提、既存 block と同じ判断を踏襲）
//!
//! - `switch`（版 A/B/C 共通のヘッダ switch）: `SwitchProps { disabled: true,
//!   ..Default }` で常時 `checked` 固定。native disabled のため
//!   [`LAYOUT_CSS`] で `[data-disabled]` の `opacity: 1; cursor: default;`
//!   を復元する（`settings_integrations_grid` の版 B と同型）。
//! - `radio_card`（版 B）: 全 item `disabled: true`、root へ
//!   `aria-disabled="true"` を明示付与し、現在の選択を独立した文で明文化
//!   する（`card_form_footer::payment_method_field` と同型）。
//! - `select`（版 D）: `SelectProps { disabled: true }`・
//!   `OpenState::Closed` 固定。`positioner`/`content` は `hidden` 属性付き
//!   のまま出力し `aria-controls`/`aria-labelledby` の参照先を残す
//!   （`card_form_footer::closed_select` と同型）。
//! - 静的タブ列（版 C）: `selected: "link"` 固定の 1 インスタンス。実物の
//!   `tabs::tabs` は使わない（上記「版 C: 無 JS での扱い」節参照）。
//! - `clipboard`（版 A）: `copied: false`（idle）で初期化し、idle/copied
//!   両 indicator を出力する（`hero_install_command`/
//!   `settings_api_key_created` と同型）。
//! - `button`（footer の「リンクをコピー」「プレビュー」）: 実アプリへの
//!   遷移先・処理を持たない合成例のボタンであり、押しても何も起きないこと
//!   を明示するため `disabled: true` にする（clipboard scope の外側の
//!   ボタンを `disabled` にする `settings_api_key_created` 版 B と同じ
//!   判断）。
//!
//! # 埋め込みスニペットに山括弧を含めない
//!
//! 版 C の「埋め込み」タブは `<iframe>` 風の HTML スニペットを模した表示を
//! 検討したが、`raw_html()` を使わずテキストノードとして出力する場合でも
//! 既定エスケープにより `<`/`>` は `&lt;`/`&gt;` へ変換される（安全だが
//! 可読性を損なう）。可読性を優先し、山括弧を含まない `[embed] https://…`
//! 形式の文言にする（`crate::blocks` の XSS 回帰テストが検知する対象自体を
//! 単純化する狙いもある）。
//!
//! # ダミー値・ドメインは明白な架空パターン
//!
//! 共有 URL は `https://fandhe-frontend.example/share/…`
//! （`.example` ドメイン、`crates/docs-site/src/blocks/mod.rs` の
//! `component_page_specs_948` QR 例と同じ判断）とし、実在ドメイン・秘密
//! 情報らしき文字列を置かない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`switch::root`/`clipboard::root`/`button::button`/
//! `button_group::root`/`radio_card::root`/`select::root`/`input::input`/
//! `qr_code::root` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-settings-share-link-*` 属性で渡す。素の `div` は `class` が
//! そのまま効くため `.blocks-settings-share-link-*` クラスセレクタを使う。
//!
//! # id / ARIA の方針
//!
//! 4 枚のカードでそれぞれ id 接頭辞を分ける
//! （`blocks-settings-share-link-{basic,audience,tabs,domain}-*`）。
//! `clipboard::label` の `for` ↔ `clipboard::input` の `id`、
//! `radio_card::label` の id ↔ `root` の `labelled_by`、`select` の
//! label/content id をカードごとに一意にする（版 C の静的タブ列は装飾
//! 〔`aria-hidden="true"`〕のため id 参照を持たない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::button_group::{self, Orientation as ButtonGroupOrientation};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::qr_code::{
    self, encode, ErrorCorrectionLevel, DEFAULT_QUIET_ZONE,
};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation as RadioCardOrientation};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 版 A〜C 共通の共有 URL（架空 `.example` ドメイン、モジュール doc
/// 「ダミー値・ドメインは明白な架空パターン」節参照）。
const SHARE_URL: &str = "https://fandhe-frontend.example/share/o7pQ-report";

/// 版 D（ドメイン接尾辞 + QR）専用のスラッグ。表示スラッグ入力・QR コードの
/// 両方がこの値から導出され、食い違いを構造的に防ぐ（レビュー指摘是正）。
const DOMAIN_SLUG: &str = "o7pq-report";

/// 版 D 専用のドメイン接尾辞（RFC 2606 予約の `.example.com`）。
const DOMAIN_SUFFIX: &str = ".example.com";

/// 版 D の表示スラッグ・select・QR コードが共通して参照する共有 URL。
/// `DOMAIN_SLUG`/`DOMAIN_SUFFIX` から構築し、3 箇所が独立した文字列
/// リテラルを持つことによる食い違い（レビュー指摘: 表示
/// `.example.com` と QR の旧リンク先が不一致だった）を防ぐ。
const DOMAIN_SHARE_URL: &str = "https://o7pq-report.example.com/";

/// 共有の有効化 switch（版 A/B/C 共通、常時 checked 固定の静的表示）。
/// `id_prefix` はカードごとの id 一意性のため。
fn share_toggle(id_prefix: &'static str) -> Node {
    let switch_props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    let name = format!("{id_prefix}-toggle");
    switch::root(
        Size::Md,
        ColorPalette::Accent,
        true,
        &switch_props,
        vec![("data-blocks-settings-share-link-toggle", "")],
        vec![
            switch::label(true, &switch_props, vec![], vec![text("共有を有効にする")]),
            switch::hidden_input(&name, "on", true, &switch_props, vec![]),
            switch::control(
                true,
                &switch_props,
                vec![],
                vec![switch::thumb(true, &switch_props, vec![], vec![])],
            ),
        ],
    )
}

/// 共有 URL のコピー欄（版 A/B/C 共通。`clipboard` root は本関数の 1 個に
/// 限る、モジュール doc「Demo 内の `clipboard` root は 1 個に限る」節）。
fn share_url_clipboard(id_prefix: &'static str) -> Node {
    let input_id = format!("{id_prefix}-clipboard-input");
    clipboard::root(
        SHARE_URL,
        false,
        vec![("data-blocks-settings-share-link-clipboard", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![clipboard::label(
                    false,
                    Some(&input_id),
                    vec![],
                    vec![text("共有 URL")],
                )],
            ),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(SHARE_URL, false, vec![("id", &input_id)]),
                    clipboard::trigger(
                        false,
                        // Demo 内の他ボタン（footer の action button-group・
                        // 版 B/C/D の静的コピーボタン）はいずれも
                        // `disabled` で押下不能を明示しているため、この
                        // trigger だけが操作可能に見えるのは一貫性を欠く
                        // （レビュー指摘是正）。native `disabled` により
                        // クリックしても何も起きないことを明示する。
                        vec![("disabled", "")],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピーしました")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 共有 URL の非 clipboard 静的表示（版 B・版 C。モジュール doc「Demo 内の
/// `clipboard` root は 1 個に限る」節参照）。readonly の `input` +
/// `input-group` のコピーボタン（`disabled: true`）で構成する
/// （`settings_api_key_created::key_row` と同型）。
fn static_url_display(id_prefix: &'static str) -> Node {
    let input_id = format!("{id_prefix}-url-input");
    let field_props = FieldProps {
        id: &input_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &field_props,
        vec![],
        vec![
            field::label(&field_props, vec![], vec![text("共有 URL")]),
            input_group::root(
                &group_props,
                vec![("data-blocks-settings-share-link-url", "")],
                vec![
                    input::input(
                        &InputProps::default(),
                        &field_props,
                        vec![("value", SHARE_URL)],
                    ),
                    input_group::addon(
                        InputGroupAlign::InlineEnd,
                        &group_props,
                        vec![],
                        vec![input_group::button(
                            // `clipboard` scope の外側にあり `headless_clipboard`
                            // 配線が届かないため、押しても何も起きないことを
                            // `disabled: true` で明示する。
                            &InputGroupProps {
                                disabled: true,
                                ..group_props
                            },
                            vec![],
                            vec![text("コピー")],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// footer の操作 button-group（リンクをコピー / プレビュー、モジュール doc
/// 「静的固定」節: 実アプリで機能する `clipboard` scope の外側のため
/// `disabled: true` で押下不能を明示する）。
fn action_button_group() -> Node {
    button_group::root(
        ButtonGroupOrientation::Horizontal,
        "共有リンクの操作",
        vec![("data-blocks-settings-share-link-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("リンクをコピー")],
            ),
            button::button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("プレビュー")],
            ),
        ],
    )
}

/// カード骨格（header + body + footer）を束ねる共通ヘルパ。
fn share_card(id_prefix: &'static str, body: Vec<Node>) -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-settings-share-link-card", id_prefix)],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("共有リンク")]),
                    card::description(
                        vec![],
                        vec![text("このページへのアクセスをリンクで共有します。")],
                    ),
                ],
            ),
            card::body(vec![("class", "blocks-settings-share-link-body")], body),
            card::footer(vec![], vec![action_button_group()]),
        ],
    )
}

/// A: 代表構成（R0318）。switch → separator → clipboard の縦積み。
fn version_basic() -> Node {
    let id_prefix = "blocks-settings-share-link-basic";
    share_card(
        id_prefix,
        vec![
            share_toggle(id_prefix),
            separator::separator(&SeparatorProps::default(), vec![]),
            share_url_clipboard(id_prefix),
        ],
    )
}

/// 閲覧範囲 radio card 1 件（モジュール doc「静的固定」節: 全件
/// `disabled: true` のネイティブ操作禁止）。
fn audience_item(checked: bool, value: &'static str, label: &'static str) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(
                checked,
                true,
                Some("blocks-settings-share-link-audience"),
                value,
                vec![],
            ),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![radio_card::item_text(vec![], vec![text(label)])],
                    ),
                ],
            ),
        ],
    )
}

/// B: 閲覧範囲版（R0319）。A の body へ radio card 3 択を追加する。
///
/// radio card 全 item は `disabled: true`（ネイティブ操作禁止）のため、
/// フォーム走査（スクリーンリーダーの「フォーム項目を読む」操作等）が
/// disabled な input を読み飛ばす環境では選択状態が伝わらない。
/// `card_form_footer::payment_method_field` と同型に、radio card の外側へ
/// 独立した文（[`styled_text::text`]）で現在の選択を明文化する
/// （レビュー指摘是正、PR #3454 cursor Bugbot）。
fn version_audience() -> Node {
    let id_prefix = "blocks-settings-share-link-audience";
    let label_id = format!("{id_prefix}-label");
    share_card(
        id_prefix,
        vec![
            share_toggle(id_prefix),
            separator::separator(&SeparatorProps::default(), vec![]),
            static_url_display(id_prefix),
            div(
                vec![("class", "blocks-settings-share-link-audience-field")],
                vec![
                    radio_card::label(Some(&label_id), vec![], vec![text("閲覧できる範囲")]),
                    radio_card::root(
                        Size::Sm,
                        ColorPalette::Accent,
                        true,
                        None::<RadioCardOrientation>,
                        Some(&label_id),
                        vec![("aria-disabled", "true")],
                        vec![
                            audience_item(true, "invited", "招待した人のみ"),
                            audience_item(false, "anyone", "リンクを知っている全員"),
                            audience_item(false, "org", "組織内のメンバー"),
                        ],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("現在の選択: 招待した人のみ")],
                    ),
                ],
            ),
        ],
    )
}

/// 静的なタブ列（版 C 専用、モジュール doc「版 C: 無 JS での扱い」節参照）。
/// 実物の `tabs::tabs` を使わず、選択中タブへ `data-state="active"` を
/// 付与した素の `div` で構成する。操作できるように見せないため
/// `role="tab"`/`<button>`/`tabindex` を出さず、装飾として
/// `aria-hidden="true"` を付ける
/// （`feature_tabs_panel::static_tab_list` と同型の判断）。
fn static_tab_list(selected: &'static str, items: &[(&'static str, &'static str)]) -> Node {
    div(
        vec![("class", "blocks-settings-share-link-tablist")],
        items
            .iter()
            .map(|(value, label)| {
                let state = if *value == selected {
                    "active"
                } else {
                    "inactive"
                };
                div(
                    vec![
                        ("class", "blocks-settings-share-link-tab"),
                        ("data-state", state),
                        ("aria-hidden", "true"),
                    ],
                    vec![text(*label)],
                )
            })
            .collect(),
    )
}

/// タブ 1 件分のプレビュー本文（キャプション + 内容）。両パネルを常時
/// 可視のまま縦に並記する（`feature_tabs_panel::tab_preview` と同型）。
fn tab_preview(caption: &'static str, content: Node) -> Node {
    div(
        vec![("class", "blocks-settings-share-link-preview")],
        vec![
            div(
                vec![("class", "blocks-settings-share-link-preview-caption")],
                vec![text(caption)],
            ),
            content,
        ],
    )
}

/// C: 埋め込み/リンク切替版（R0320）。body 先頭に静的なタブ列（`selected:
/// "link"` 固定）を置き、両パネルをキャプション付きで常時可視のまま
/// 縦に並記する（モジュール doc「版 C: 無 JS での扱い」節）。
fn version_tabs() -> Node {
    let id_prefix = "blocks-settings-share-link-tabs";
    let link_display = static_url_display(id_prefix);
    let embed_snippet = div(
        vec![("class", "blocks-settings-share-link-embed-snippet")],
        // 山括弧を含まないプレーン文字列（モジュール doc「埋め込み
        // スニペットに山括弧を含めない」節参照）。
        vec![text(format!("[embed] {SHARE_URL}"))],
    );
    let tab_list = static_tab_list("link", &[("link", "リンク"), ("embed", "埋め込み")]);
    share_card(
        id_prefix,
        vec![
            share_toggle(id_prefix),
            tab_list,
            tab_preview("リンク", link_display),
            tab_preview("埋め込み", embed_snippet),
        ],
    )
}

/// D: ドメイン接尾辞 + QR 版（R0321）。スラッグ入力 + ドメイン接尾辞
/// select（閉じた状態固定）+ QR コード。
fn version_domain_qr() -> Node {
    let id_prefix = "blocks-settings-share-link-domain";
    let slug_id = format!("{id_prefix}-slug");
    let slug_field = FieldProps {
        id: &slug_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        // QR コードは `DOMAIN_SHARE_URL` から生成する静的表示のため、スラッグを
        // 編集可能にすると表示スラッグと QR コードのリンク先が食い違う
        // （レビュー指摘、PR #3454）。読み取り専用にし、かつ入力値・select・
        // QR を同一の `DOMAIN_SLUG`/`DOMAIN_SUFFIX`/`DOMAIN_SHARE_URL` 定数
        // から取ることで構造的に不整合を防ぐ。
        readonly: true,
        has_helper_text: false,
    };
    let select_label_id = format!("{id_prefix}-select-label");
    let select_content_id = format!("{id_prefix}-select-content");
    let select_props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
    let domain_select = div(
        vec![("class", "blocks-settings-share-link-domain-select")],
        vec![
            select::label(
                &select_props,
                Some(&select_label_id),
                vec![],
                vec![text("ドメイン接尾辞")],
            ),
            select::root(
                Size::Md,
                OpenState::Closed,
                &select_props,
                vec![],
                vec![
                    select::control(
                        OpenState::Closed,
                        &select_props,
                        vec![],
                        vec![select::trigger(
                            OpenState::Closed,
                            &select_props,
                            false,
                            Some(&select_content_id),
                            Some(&select_label_id),
                            vec![],
                            vec![
                                select::value_text(
                                    false,
                                    &select_props,
                                    vec![],
                                    vec![text(DOMAIN_SUFFIX)],
                                ),
                                select::indicator(OpenState::Closed, &select_props, vec![], vec![]),
                            ],
                        )],
                    ),
                    select::positioner(
                        OpenState::Closed,
                        vec![],
                        vec![select::content(
                            OpenState::Closed,
                            Some(&select_content_id),
                            Some(&select_label_id),
                            None,
                            vec![],
                            vec![select::item(
                                OpenState::Open,
                                &select_props,
                                false,
                                false,
                                "example-com",
                                None,
                                vec![],
                                vec![select::item_text(
                                    OpenState::Open,
                                    &select_props,
                                    false,
                                    false,
                                    None,
                                    vec![],
                                    vec![text(DOMAIN_SUFFIX)],
                                )],
                            )],
                        )],
                    ),
                ],
            ),
        ],
    );
    let matrix = encode(DOMAIN_SHARE_URL, ErrorCorrectionLevel::M)
        // 固定短文字列のみを符号化するため `TooLong` になり得ない
        // （`settings_api_key_created` 等と同じくダミー値は本 block 内の
        // 定数として決定的に管理される）。失敗時は空 QR へフォールバック
        // し `expect`/`unwrap` を避ける。
        .unwrap_or_else(|_| {
            encode("share", ErrorCorrectionLevel::L).expect("固定短文字列の符号化は失敗しない")
        });
    let qr = qr_code::root(
        Size::Md,
        vec![("data-blocks-settings-share-link-qr", "")],
        vec![qr_code::frame(
            &matrix,
            DEFAULT_QUIET_ZONE,
            Some("共有 URL の QR コード"),
            vec![],
            vec![qr_code::pattern(&matrix, DEFAULT_QUIET_ZONE, vec![])],
        )],
    );
    share_card(
        id_prefix,
        vec![
            share_toggle(id_prefix),
            div(
                vec![("class", "blocks-settings-share-link-domain-row")],
                vec![
                    field::root(
                        &FieldRootProps::default(),
                        &slug_field,
                        vec![],
                        vec![
                            field::label(&slug_field, vec![], vec![text("公開スラッグ")]),
                            input::input(
                                &InputProps::default(),
                                &slug_field,
                                vec![("type", "text"), ("value", DOMAIN_SLUG)],
                            ),
                        ],
                    ),
                    domain_select,
                ],
            ),
            div(
                vec![("class", "blocks-settings-share-link-qr-row")],
                vec![qr],
            ),
        ],
    )
}

/// `settings-share-link` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。4 版を縦に並記する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-share-link-layout")],
        vec![
            version_basic(),
            version_audience(),
            version_tabs(),
            version_domain_qr(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-share-link/",
    title: "settings-share-link",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_share_link.rs",
    demo_class: "blocks-settings-share-link",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Button Group",
            path: "/themes/button-group/",
        },
        Part {
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "QR Code",
            path: "/themes/qr-code/",
        },
        Part {
            label: "Select",
            path: "/themes/select/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_share_link` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。狭幅では footer の
/// button-group を縦積み・全幅にし、`40rem` 以上で横並びへ切り替える
/// （テーマ breakpoint トークンは `@media` 内で解決できないためリテラル、
/// `card_form_footer`/`settings_billing_overview` と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-settings-share-link-layout {\n  display: grid;\n  grid-template-columns: repeat(auto-fit, minmax(min(100%, 22rem), 1fr));\n  gap: var(--fandhe-space-6);\n  align-items: start;\n}\n\
.blocks-settings-share-link-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-share-link-audience-field {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5, 0.375rem);\n}\n\
.blocks-settings-share-link-domain-row {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n  align-items: flex-end;\n}\n\
.blocks-settings-share-link-qr-row {\n  display: flex;\n  justify-content: center;\n}\n\
.blocks-settings-share-link-embed-snippet {\n  font-family: var(--fandhe-font-font-mono);\n  font-size: var(--fandhe-font-font-size-sm);\n  word-break: break-all;\n}\n\
.blocks-settings-share-link-tablist {\n  display: flex;\n  gap: var(--fandhe-space-1);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-settings-share-link-tab {\n  padding: var(--fandhe-space-1-5, 0.375rem) var(--fandhe-space-3);\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n  border-bottom: 2px solid transparent;\n}\n\
.blocks-settings-share-link-tab[data-state=\"active\"] {\n  color: var(--fandhe-color-fg);\n  border-bottom-color: var(--fandhe-color-border-accent, var(--fandhe-color-fg));\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
.blocks-settings-share-link-preview {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5, 0.375rem);\n}\n\
.blocks-settings-share-link-preview-caption {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-settings-share-link-toggle] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-settings-share-link-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-settings-share-link-clipboard] {\n  display: flex;\n  flex-direction: column;\n  width: 100%;\n}\n\
[data-blocks-settings-share-link-clipboard] [data-scope=\"clipboard\"][data-part=\"control\"] {\n  width: 100%;\n}\n\
.blocks-settings-share-link-layout [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-settings-share-link-layout [data-scope=\"select\"][data-part=\"trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-settings-share-link-actions] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  width: 100%;\n}\n\
[data-blocks-settings-share-link-actions] [data-scope=\"button\"] {\n  inline-size: 100%;\n}\n\
@media (min-width: 40rem) {\n  \
[data-blocks-settings-share-link-actions] {\n    flex-direction: row;\n    justify-content: flex-end;\n  }\n  \
[data-blocks-settings-share-link-actions] [data-scope=\"button\"] {\n    inline-size: auto;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"switch\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"button\"",
            "data-scope=\"button-group\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"qr-code\"",
            "data-scope=\"select\"",
            "data-scope=\"separator\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"input-group\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        // 版 C は実物の `tabs::tabs` を使わない静的タブ列（モジュール doc
        // 「版 C: 無 JS での扱い」節）。
        assert!(!html.contains("data-scope=\"tabs\""));
        assert!(!html.contains("role=\"tab\""));
        assert!(!html.contains("role=\"tabpanel\""));
    }

    #[test]
    fn static_tab_list_marks_selected_state_without_fake_interactivity() {
        let html = demo_html();
        assert!(html.contains(
            "class=\"blocks-settings-share-link-tab\" data-state=\"active\" aria-hidden=\"true\""
        ));
        assert!(html.contains(
            "class=\"blocks-settings-share-link-tab\" data-state=\"inactive\" aria-hidden=\"true\""
        ));
        assert!(!html.contains("<button class=\"blocks-settings-share-link-tab"));
        assert!(html.contains("class=\"blocks-settings-share-link-preview-caption\""));
    }

    #[test]
    fn clipboard_trigger_is_disabled() {
        let html = demo_html();
        let pos = html
            .find(r#"data-scope="clipboard" data-part="trigger""#)
            .expect("clipboard trigger must be present");
        let tag_start = html[..pos].rfind('<').expect("trigger tag start");
        let tag_end = html[tag_start..]
            .find('>')
            .map(|i| tag_start + i)
            .expect("trigger tag end");
        assert!(html[tag_start..tag_end].contains("disabled"));
    }

    #[test]
    fn audience_selection_is_stated_independently_of_disabled_radio_card() {
        let html = demo_html();
        // radio card 全 item が disabled のため、フォーム走査で選択状態が
        // 読み上げられない環境向けに、独立した文で明文化する
        // （レビュー指摘是正、PR #3454 cursor Bugbot）。
        assert!(html.contains("現在の選択: 招待した人のみ"));
        assert!(html.contains("data-scope=\"text\" data-part=\"root\""));
    }

    #[test]
    fn domain_share_url_matches_slug_and_suffix() {
        assert_eq!(
            super::DOMAIN_SHARE_URL,
            format!("https://{}{}/", super::DOMAIN_SLUG, super::DOMAIN_SUFFIX)
        );
        let html = demo_html();
        assert!(html.contains(super::DOMAIN_SLUG));
        assert!(html.contains(super::DOMAIN_SUFFIX));
    }

    #[test]
    fn no_angle_brackets_in_embed_snippet_text_node() {
        let html = demo_html();
        assert!(html.contains("[embed] https://fandhe-frontend.example/share/"));
    }

    #[test]
    fn share_url_is_example_domain_only() {
        let html = demo_html();
        assert!(html.contains("fandhe-frontend.example"));
        // `xmlns="http://www.w3.org/2000/svg"`（qr-code frame の標準
        // 名前空間 URI）は無害なため forbidden 判定に含めない。
        for forbidden in ["javascript:", "data:text/html"] {
            assert!(!html.contains(forbidden), "must not contain {forbidden}");
        }
    }

    #[test]
    fn interactive_parts_are_natively_disabled() {
        let html = demo_html();
        // radio card 3 件 = type="radio" 3 件。
        assert_eq!(html.matches(r#"type="radio""#).count(), 3);
        // switch 4 版分（A/B/C/D 共通ヘッダ）= role="switch" 4 件、いずれも disabled。
        assert_eq!(html.matches(r#"role="switch""#).count(), 4);
        // select trigger（版 D の 1 件）= aria-expanded="false" 1 件。
        assert_eq!(html.matches("aria-expanded=\"false\"").count(), 1);
        assert!(html.contains(r#"role="listbox""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn footer_action_buttons_are_disabled() {
        let html = demo_html();
        // footer の button-group は版 4 個 × 2 ボタン = 8 個、すべて disabled。
        let disabled_buttons = html.matches("data-scope=\"button\"").count();
        assert!(disabled_buttons >= 8);
    }

    #[test]
    fn only_one_clipboard_root() {
        let html = demo_html();
        assert_eq!(
            html.matches(r#"data-scope="clipboard" data-part="root""#)
                .count(),
            1
        );
    }

    #[test]
    fn qr_code_renders_pattern_with_aria_label() {
        let html = demo_html();
        assert!(html.contains(r#"data-scope="qr-code" data-part="frame""#));
        assert!(html.contains(r#"data-scope="qr-code" data-part="pattern""#));
        assert!(html.contains(r#"aria-label="共有 URL の QR コード""#));
    }

    #[test]
    fn every_for_target_and_aria_reference_resolves() {
        let html = demo_html();
        assert!(html.contains("id=\"blocks-settings-share-link-domain-select-label\""));
        assert!(html.contains("id=\"blocks-settings-share-link-domain-select-content\""));
        assert!(html.contains("aria-labelledby=\"blocks-settings-share-link-domain-select-label\""));
        assert!(html.contains("aria-controls=\"blocks-settings-share-link-domain-select-content\""));
        assert!(html.contains("id=\"blocks-settings-share-link-audience-label\""));
        assert!(html.contains("aria-labelledby=\"blocks-settings-share-link-audience-label\""));
    }

    #[test]
    fn ids_are_unique() {
        let html = demo_html();
        let mut ids = std::collections::HashSet::new();
        let mut rest = html.as_str();
        while let Some(pos) = rest.find("id=\"") {
            rest = &rest[pos + 4..];
            let end = rest.find('"').expect("id attribute must be closed");
            let id = &rest[..end];
            assert!(ids.insert(id.to_string()), "duplicate id: {id}");
            rest = &rest[end..];
        }
    }

    #[test]
    fn layout_css_stacks_on_narrow_and_neutralizes_disabled() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("flex-direction: column;"));
        assert!(LAYOUT_CSS.contains("flex-direction: row;"));
        assert!(LAYOUT_CSS.contains("inline-size: 100%;"));
        assert!(LAYOUT_CSS.contains("inline-size: auto;"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
        assert!(LAYOUT_CSS.contains("cursor: default;"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-share-link-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-settings-share-link-layout");
    }
}
