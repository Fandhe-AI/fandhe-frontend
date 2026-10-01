//! `settings-share-members` block（イシュー #3013/#3014。親 #3012「Blocks に
//! settings-share-members を追加する」。#3013 で骨格・主要領域（代表構成
//! 1 版）を実装し、本イシュー（#3014）で QR コード版の並記・状態違いの
//! 並記・原稿の差分メモ仕上げを追加した）。`_/blocks-intake/` の対応
//! ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・本
//! コメントには対応表 ID のみを記す（`profile-detail-datalist`〔#2937〕・
//! `settings-org-switcher`〔#2999〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `card` / `select` / `input-group` / `input` / `text` / `avatar` /
//! `clipboard` / `separator` / `field` / `heading` / `qr-code` の 11 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。`heading`/`qr-code` は本イシュー
//! （#3014）で追加した（下記「2 版の並記と見出し階層」「QR コードの
//! 組み立て」節参照）。新しい UI 部品は追加しない。
//!
//! # 2 版と集約元 ID の対応
//!
//! - **版 A（[`version_invited`]）**: R0323（主参照・代表構成、#3013）に
//!   対応。共有範囲は「招待したメンバーのみ」を選択済みで、共有リンク
//!   領域は `clipboard` のみの単純な行（[`share_link_section`]）。
//! - **版 B（[`version_link_qr`]）**: R0322（QR コード + メール招待）と
//!   R0031（読み取りリンク + メンバー権限）を 1 版へまとめて対応させる
//!   （両方とも「リンクを知っている全員」へ共有範囲を広げた状態でしか
//!   意味を持たないため、3 版並記は規模過剰と判断した）。共有範囲は
//!   「リンクを知っている全員」を選択済みで、共有リンク領域を QR コード
//!   （左）+ リンク・コピー操作 + 「リンクの権限」select（右）の 2 列へ
//!   拡張する（[`share_link_qr_section`]）。
//!
//! メール招待・メンバー一覧（アクセス権選択付き）の 2 section は両版で
//! 共有する（[`invite_section`]・[`members_section`]）。「状態違いの
//! 並記」は共有範囲 select の初期選択値の違い（招待制 / リンク公開）で
//! 表し、`clipboard` の `copied: true` 状態は並記しない
//! （`hero_install_command`〔#2786 Codex 指摘〕で「コピー操作前から
//! Copied 表示」を是正した先例と同じ判断: 実際に押していない静的表示へ
//! コピー完了状態を出すと誤解を招くため）。
//!
//! # 2 版の並記と見出し階層
//!
//! [`caption`]（`heading` の `h2`）→ `card::title`（`h3`）の 2 階層で
//! 構造化する（`settings_integration_detail.rs::caption` と同型の判断。
//! `heading` は `drop_class_attr` により `class` を除去するため、CSS
//! フックには `data-blocks-settings-share-members-caption` を使う）。
//!
//! # `id`/ARIA の一意性（版接頭辞方式）
//!
//! 版 A/B は同じ section 関数（[`share_scope_section`]・
//! [`invite_section`]・[`member_row`]/[`members_section`]・
//! [`clipboard_block`]・[`link_permission_select`]）を再利用するため、
//! 各関数は `version: &str`（`"a"`/`"b"`）を受け取り、`id` を
//! `blocks-settings-share-members-{version}-...` へ接頭辞化する
//! （`hero_install_command` の instance A/C 分離と同型の判断。実アプリへ
//! 組み込む際に版ごとへ個別に `mount`/`hydrate` できる）。
//!
//! # QR コードの組み立て
//!
//! [`qr_frame`] が
//! [`fandhe_frontend_pre_styled_ui::qr_code::encode`]（固定の共有リンク
//! 文字列を符号化、`component_page_specs_948.rs::qr_code_example` と同じ
//! 組み立て）を呼び、`Result` を `match` で処理する
//! （本 block の Demo 関数群は `Result` を返さない純関数契約のため）。
//! 値は固定 const のため実際には失敗しないが、`Err` になった場合は QR
//! コードの代わりに muted テキストを表示し、黙って要素が欠落しないように
//! する（ユニットテスト `qr_version_renders_qr_frame_with_label_and_no_copied_state`
//! が `data-scope="qr-code"` の存在を固定）。`frame` には
//! `aria-label="共有リンクの QR コード"` を付与する。
//!
//! # select を閉じた状態の固定表示で置く理由
//!
//! `select` の開閉は `fandhe-frontend-wasm-full` の JS 配線が担う
//! （headless `select` doc 参照）。docs サイトは JS ハイドレーションを
//! 行わないため、共有範囲・メンバーごとの権限選択・リンクの権限選択は
//! いずれも `OpenState::Closed` で固定した静的表示に留め、`SelectProps {
//! disabled: true, .. }` で trigger へネイティブ `disabled` 属性を付与する
//! （`card_form_footer::closed_select` と同型の判断、操作しても開閉が
//! 追従できない `<button>` を操作可能に見せないための構造的禁止）。
//! `positioner`/`content` は `hidden` 付きのまま出力し、`aria-controls`/
//! `aria-labelledby` の参照先が宙に浮かないようにする
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 対策）。
//!
//! # 招待ボタン・コピー配線の範囲
//!
//! メール招待の「招待」ボタンは `input-group` の addon ボタンであり
//! `clipboard` scope の外側にあるため送信処理を持たない（`disabled: true`
//! で押下不能を明示、`hero_install_command::instance_b` と同型の判断）。
//! 共有リンクの `clipboard::root`/`control`/`input`/`trigger` は、この
//! block では押しても無 JS のため無反応だが、ネイティブ `disabled`
//! 属性は付与しない（実アプリへ組み込めば `fandhe-frontend-wasm-full` の
//! `headless_clipboard` 配線が `mount`/`hydrate` 時にコピー操作を機能
//! させるため。ネイティブ `disabled` はブラウザが `click` イベント自体を
//! 発火させなくする属性であり、付与すると実アプリに組み込んでも
//! クリックできなくなり本節の前提と矛盾する。`settings_api_key_created`/
//! `settings_integrations_list`/`hero_install_command` の `clipboard::trigger`
//! と同じ判断、PR #3455 コードレビュー是正）。
//!
//! # `class` と `data-*` の使い分け（`drop_class_attr` の契約）
//!
//! `card::root`/`select::root`/`avatar::root`/`input_group::root`/
//! `clipboard::root`/`field::root`/`button::button`/`heading`/
//! `qr_code::root` はいずれも `drop_class_attr` により呼び出し側 `attrs`
//! の `class` を黙って除去する契約を持つため、CSS フックは
//! `data-blocks-settings-share-members-*` 属性で渡す。素の `div` には
//! `class="blocks-settings-share-members-*"` を使う。
//!
//! # 狭幅ではレイアウトを縦積みへ回す（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため `@container`（コンテナ
//! クエリ）で判定する（`profile_detail_datalist` と同型のパターン）。
//! [`LAYOUT_CSS`] のラッパー `.blocks-settings-share-members-stack` へ
//! `container-type: inline-size` を宣言し、コンテナ幅が `36rem` 未満の
//! とき、(1) メンバー行のグリッド列を 3 列（アバター・氏名・権限）から
//! 2 列へ変え権限 select を氏名の下へ回し、(2) 版 B の共有リンク領域
//! （QR + リンクの 2 列グリッド）を 1 列へ変えて QR の下にリンクを回す。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。ボタンは
//! `button::button`/`input_group::button` の既定 `type="button"` のまま
//! 用いる。両版とも静的固定表示（無 JS）で、状態の切り替え自体はできない。
//!
//! # ダミー素材・PII について
//!
//! 氏名は [`dummy_assets::PERSON_NAMES`]、アバターは
//! [`dummy_assets::AVATAR_SRC`]（同梱 SVG）を使う。メールアドレスは
//! `example.com` ドメイン、共有リンク・QR コードの値は RFC 2606 予約
//! ドメイン `share.example.com` を使い、実在の人物・組織・URL・トークン風
//! 文字列は含めない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::clipboard;
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::qr_code;
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 共有リンク・QR コードの値（RFC 2606 予約ドメイン、両版で共有）。
const SHARE_LINK: &str = "https://share.example.com/d/9f3a1c";

/// 閉じた状態の styled select 1 件（モジュール doc「select を閉じた状態の
/// 固定表示で置く理由」節。`card_form_footer::closed_select` と同型）。
/// `options` は `(value, label, selected)` の組。
fn closed_select(
    label_id: &str,
    content_id: &str,
    options: &[(&'static str, &'static str, bool)],
) -> Node {
    let props = SelectProps {
        disabled: true,
        ..SelectProps::default()
    };
    let selected_label = options
        .iter()
        .find(|(_, _, selected)| *selected)
        .map(|(_, label, _)| *label)
        .unwrap_or_default();
    let items: Vec<Node> = options
        .iter()
        .map(|(value, label, selected)| {
            let state = if *selected {
                OpenState::Open
            } else {
                OpenState::Closed
            };
            select::item(
                state,
                &props,
                false,
                false,
                value,
                None,
                vec![],
                vec![select::item_text(
                    state,
                    &props,
                    false,
                    false,
                    None,
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    select::root(
        Size::Md,
        OpenState::Closed,
        &props,
        vec![],
        vec![
            select::control(
                OpenState::Closed,
                &props,
                vec![],
                vec![select::trigger(
                    OpenState::Closed,
                    &props,
                    false,
                    Some(content_id),
                    Some(label_id),
                    vec![("data-blocks-settings-share-members-select", "")],
                    vec![
                        select::value_text(false, &props, vec![], vec![text(selected_label)]),
                        select::indicator(OpenState::Closed, &props, vec![], vec![]),
                    ],
                )],
            ),
            select::positioner(
                OpenState::Closed,
                vec![],
                vec![select::content(
                    OpenState::Closed,
                    Some(content_id),
                    Some(label_id),
                    None,
                    vec![],
                    items,
                )],
            ),
        ],
    )
}

/// 共有範囲の選択領域（`field` ラベル + 閉じた `select`）。`link_open` が
/// `true` のとき「リンクを知っている全員」（版 B）、`false` のとき
/// 「招待したメンバーのみ」（版 A）を初期選択にする（モジュール doc
/// 「2 版と集約元 ID の対応」節参照）。`version` は `id` の一意化に使う
/// （モジュール doc「`id`/ARIA の一意性」節）。
fn share_scope_section(version: &str, link_open: bool) -> Node {
    let label_id = format!("blocks-settings-share-members-{version}-scope-label");
    let content_id = format!("blocks-settings-share-members-{version}-scope-content");
    div(
        vec![("class", "blocks-settings-share-members-section")],
        vec![
            select::label(
                &SelectProps {
                    disabled: true,
                    ..SelectProps::default()
                },
                Some(label_id.as_str()),
                vec![],
                vec![text("共有範囲")],
            ),
            closed_select(
                &label_id,
                &content_id,
                &[
                    ("anyone", "リンクを知っている全員", link_open),
                    ("org", "組織内のメンバー", false),
                    ("invited", "招待したメンバーのみ", !link_open),
                ],
            ),
        ],
    )
}

/// メール招待の入力欄（`field` ラベル、`input-group`（`input type="email"`
/// と末尾 addon の「招待」ボタン）で構成する）。addon ボタンは送信先を
/// 持たないため `disabled: true`（モジュール doc「招待ボタン・コピー配線の
/// 範囲」節）。両版で共有するため `version` で `id` を一意化する。
fn invite_section(version: &str) -> Node {
    let field_id = format!("blocks-settings-share-members-{version}-invite-input");
    let field_props = FieldProps {
        id: field_id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    div(
        vec![("class", "blocks-settings-share-members-section")],
        vec![field::root(
            &FieldRootProps::default(),
            &field_props,
            vec![],
            vec![
                field::label(&field_props, vec![], vec![text("メールで招待")]),
                input_group::root(
                    &group_props,
                    vec![],
                    vec![
                        input::input(
                            &InputProps::default(),
                            &field_props,
                            vec![("type", "email"), ("placeholder", "you@example.com")],
                        ),
                        input_group::addon(
                            InputGroupAlign::InlineEnd,
                            &group_props,
                            vec![],
                            vec![input_group::button(
                                &InputGroupProps {
                                    disabled: true,
                                    ..group_props
                                },
                                vec![],
                                vec![text("招待")],
                            )],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// メンバー 1 行（アバター + 氏名・メール + 権限 `select`）。`version` +
/// `index` で `id` を一意化する（モジュール doc「`id`/ARIA の一意性」
/// 節）。
fn member_row(
    version: &str,
    index: usize,
    name: &'static str,
    email: String,
    perm_selected: usize,
) -> Node {
    let label_id = format!("blocks-settings-share-members-{version}-perm-{index}-label");
    let content_id = format!("blocks-settings-share-members-{version}-perm-{index}-content");
    let perms = [
        ("editor", "編集可"),
        ("viewer", "閲覧のみ"),
        ("owner", "オーナー"),
    ];
    let options: Vec<(&str, &str, bool)> = perms
        .iter()
        .enumerate()
        .map(|(i, (value, label))| (*value, *label, i == perm_selected))
        .collect();
    div(
        vec![("class", "blocks-settings-share-members-member")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Md,
                    ..AvatarProps::default()
                },
                vec![("data-blocks-settings-share-members-avatar", "")],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-settings-share-members-identity")],
                vec![
                    fandhe_frontend_pre_styled_ui::text::text(
                        &fandhe_frontend_pre_styled_ui::text::TextProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    fandhe_frontend_pre_styled_ui::text::text(
                        &fandhe_frontend_pre_styled_ui::text::TextProps {
                            variant: fandhe_frontend_pre_styled_ui::text::TextVariant::Muted,
                            size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
                            ..fandhe_frontend_pre_styled_ui::text::TextProps::default()
                        },
                        vec![],
                        vec![text(email)],
                    ),
                ],
            ),
            div(
                vec![("data-blocks-settings-share-members-perm", "")],
                vec![
                    fandhe_frontend_pre_styled_ui::visually_hidden::root(
                        vec![],
                        vec![select::label(
                            &SelectProps {
                                disabled: true,
                                ..SelectProps::default()
                            },
                            Some(label_id.as_str()),
                            vec![],
                            vec![text("権限")],
                        )],
                    ),
                    closed_select(&label_id, &content_id, &options),
                ],
            ),
        ],
    )
}

/// メンバー一覧領域（[`member_row`] を 3 件並べる）。両版で共有する。
fn members_section(version: &str) -> Node {
    let members: Vec<Node> = dummy_assets::PERSON_NAMES[..3]
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let email = format!(
                "{}@example.com",
                name.to_lowercase()
                    .replace(' ', ".")
                    .replace(['\'', '-'], "")
            );
            member_row(version, i, name, email, if i == 0 { 2 } else { 0 })
        })
        .collect();
    div(
        vec![("class", "blocks-settings-share-members-section")],
        std::iter::once(fandhe_frontend_pre_styled_ui::text::text(
            &fandhe_frontend_pre_styled_ui::text::TextProps {
                size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
                variant: fandhe_frontend_pre_styled_ui::text::TextVariant::Muted,
                ..fandhe_frontend_pre_styled_ui::text::TextProps::default()
            },
            vec![],
            vec![text("メンバー")],
        ))
        .chain(members)
        .collect(),
    )
}

/// 共有リンクの `clipboard` 本体（版 A の単純な共有リンク領域
/// [`share_link_section`]、版 B の QR 版共有リンク領域
/// [`share_link_qr_section`] の双方から呼ばれる共通部分。モジュール doc
/// 「招待ボタン・コピー配線の範囲」節参照。実アプリへ組み込めばコピー
/// 操作は機能する）。
fn clipboard_block(version: &str) -> Node {
    let input_id = format!("blocks-settings-share-members-{version}-link-input");
    clipboard::root(
        SHARE_LINK,
        false,
        vec![],
        vec![
            clipboard::label(
                false,
                Some(input_id.as_str()),
                vec![],
                vec![text("共有リンク")],
            ),
            clipboard::control(
                false,
                vec![],
                vec![
                    clipboard::input(SHARE_LINK, false, vec![("id", input_id.as_str())]),
                    // ネイティブ `disabled` は付与しない（モジュール doc「招待ボタン・
                    // コピー配線の範囲」節参照。実アプリへ組み込んだ際にクリック
                    // イベント自体が発火しなくなるのを避けるため）。
                    clipboard::trigger(
                        false,
                        vec![],
                        vec![
                            clipboard::indicator(false, false, vec![], vec![text("コピー")]),
                            clipboard::indicator(true, false, vec![], vec![text("コピー済み")]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 共有リンクとコピー操作の領域（版 A、単純な `clipboard` のみ）。
fn share_link_section(version: &str) -> Node {
    div(
        vec![("class", "blocks-settings-share-members-section")],
        vec![clipboard_block(version)],
    )
}

/// 「リンクの権限」select（版 B のみ、R0031 差分）。可視ラベル付きの
/// 閉じた select 1 件（初期選択「閲覧のみ」）。
fn link_permission_select(version: &str) -> Node {
    let label_id = format!("blocks-settings-share-members-{version}-link-perm-label");
    let content_id = format!("blocks-settings-share-members-{version}-link-perm-content");
    div(
        vec![],
        vec![
            select::label(
                &SelectProps {
                    disabled: true,
                    ..SelectProps::default()
                },
                Some(label_id.as_str()),
                vec![],
                vec![text("リンクの権限")],
            ),
            closed_select(
                &label_id,
                &content_id,
                &[("viewer", "閲覧のみ", true), ("editor", "編集可", false)],
            ),
        ],
    )
}

/// 共有リンクの QR コード（モジュール doc「QR コードの組み立て」節参照）。
/// `encode` が失敗した場合は muted テキストへ差し替え、黙って要素を
/// 欠落させない。
fn qr_frame() -> Node {
    match qr_code::encode(SHARE_LINK, qr_code::ErrorCorrectionLevel::M) {
        Ok(matrix) => qr_code::root(
            Size::Md,
            vec![("data-blocks-settings-share-members-qr", "")],
            vec![qr_code::frame(
                &matrix,
                qr_code::DEFAULT_QUIET_ZONE,
                Some("共有リンクの QR コード"),
                vec![],
                vec![qr_code::pattern(
                    &matrix,
                    qr_code::DEFAULT_QUIET_ZONE,
                    vec![],
                )],
            )],
        ),
        Err(_) => fandhe_frontend_pre_styled_ui::text::text(
            &fandhe_frontend_pre_styled_ui::text::TextProps {
                variant: fandhe_frontend_pre_styled_ui::text::TextVariant::Muted,
                size: fandhe_frontend_pre_styled_ui::text::TextSize::Sm,
                ..fandhe_frontend_pre_styled_ui::text::TextProps::default()
            },
            vec![],
            vec![text("QR コードを生成できませんでした。")],
        ),
    }
}

/// 共有リンクとコピー操作の領域（版 B、QR コード + `clipboard` + リンクの
/// 権限 select の 2 列。モジュール doc「2 版と集約元 ID の対応」節参照）。
fn share_link_qr_section(version: &str) -> Node {
    div(
        vec![("class", "blocks-settings-share-members-section")],
        vec![div(
            vec![("class", "blocks-settings-share-members-link-row")],
            vec![
                qr_frame(),
                div(
                    vec![("class", "blocks-settings-share-members-link-fields")],
                    vec![clipboard_block(version), link_permission_select(version)],
                ),
            ],
        )],
    )
}

/// 版のキャプション（見出し）。`h2`（カード表題 `h3` の親階層）として
/// 構造化する（`settings_integration_detail.rs::caption` と同型のパターン、
/// モジュール doc「2 版の並記と見出し階層」節参照）。`heading` は `class`
/// を `drop_class_attr` 経由で除去するため、スタイルフックには
/// `data-blocks-settings-share-members-caption` を使う。
fn caption(label: &'static str) -> Node {
    heading(
        HeadingLevel::H2,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![("data-blocks-settings-share-members-caption", "")],
        vec![text(label)],
    )
}

/// 版 A/B 共通のカード骨格（ヘッダー + 4 領域の縦積み、`separator` 区切り）。
/// `link_section` に版ごとの共有リンク領域（[`share_link_section`] または
/// [`share_link_qr_section`]）を渡す。
fn share_card(version: &str, link_open: bool, link_section: Node) -> Node {
    card::root(
        CardProps::from(CardVariant::Outline),
        vec![("data-blocks-settings-share-members-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("共有設定")]),
                    card::description(
                        vec![],
                        vec![text("このファイルを共有する範囲とメンバーを管理します。")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    share_scope_section(version, link_open),
                    separator(&SeparatorProps::default(), vec![]),
                    invite_section(version),
                    separator(&SeparatorProps::default(), vec![]),
                    members_section(version),
                    separator(&SeparatorProps::default(), vec![]),
                    link_section,
                ],
            ),
        ],
    )
}

/// 版 A（招待制、R0323 主参照）。
fn version_invited() -> Node {
    div(
        vec![("class", "blocks-settings-share-members-version")],
        vec![
            caption("招待したメンバーのみ"),
            share_card("a", false, share_link_section("a")),
        ],
    )
}

/// 版 B（リンク公開制、R0322 の QR コード + R0031 の読み取りリンク権限）。
fn version_link_qr() -> Node {
    div(
        vec![("class", "blocks-settings-share-members-version")],
        vec![
            caption("リンクを知っている全員（QR コード付き）"),
            share_card("b", true, share_link_qr_section("b")),
        ],
    )
}

/// `settings-share-members` の Demo 本体（版 A/B を並記。呼び出しごとに
/// 同一の `Node` を返す純関数、モジュール doc「2 版と集約元 ID の対応」
/// 節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-share-members-stack")],
        vec![version_invited(), version_link_qr()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-share-members/",
    title: "settings-share-members",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_share_members.rs",
    demo_class: "blocks-settings-share-members",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Select",
            path: "/themes/select/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Clipboard",
            path: "/themes/clipboard/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "QR Code",
            path: "/themes/qr-code/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_share_members` 固有のレイアウト規則（`--fandhe-*` トークンの
/// み使用）。
const LAYOUT_CSS: &str = "\
.blocks-settings-share-members-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n  container-type: inline-size;\n  container-name: blocks-settings-share-members;\n}\n\
.blocks-settings-share-members-version {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-share-members-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-share-members-member {\n  display: grid;\n  grid-template-columns: auto 1fr auto;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-share-members-identity {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n  min-width: 0;\n}\n\
.blocks-settings-share-members-link-row {\n  display: grid;\n  grid-template-columns: auto 1fr;\n  align-items: start;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-share-members-link-fields {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
[data-scope=\"heading\"][data-blocks-settings-share-members-caption] {\n  border-top: none;\n  padding-top: 0;\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"select\"][data-part=\"trigger\"][data-blocks-settings-share-members-select][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@container blocks-settings-share-members (max-width: 36rem) {\n  \
.blocks-settings-share-members-member {\n    grid-template-columns: auto 1fr;\n  }\n  \
[data-blocks-settings-share-members-perm] {\n    grid-column: 2;\n  }\n  \
.blocks-settings-share-members-link-row {\n    grid-template-columns: 1fr;\n  }\n\
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
            "data-scope=\"select\"",
            "data-scope=\"input-group\"",
            "data-scope=\"avatar\"",
            "data-scope=\"clipboard\"",
            "data-scope=\"separator\"",
            "data-scope=\"field\"",
            "data-scope=\"text\"",
            "data-scope=\"heading\"",
            "data-scope=\"qr-code\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(html.contains("data-part=\"input\""));
        // メンバー行 3 件 x 版 2 件分の権限 select フック。
        assert_eq!(
            html.matches("data-blocks-settings-share-members-perm=")
                .count(),
            6
        );
    }

    #[test]
    fn demo_has_two_versions_with_distinct_ids() {
        let html = demo_html();
        assert!(html.contains(r#"id="blocks-settings-share-members-a-scope-label""#));
        assert!(html.contains(r#"id="blocks-settings-share-members-b-scope-label""#));
        assert!(html.contains(r#"id="blocks-settings-share-members-a-link-input""#));
        assert!(html.contains(r#"id="blocks-settings-share-members-b-link-input""#));
    }

    #[test]
    fn qr_version_renders_qr_frame_with_label_and_no_copied_state() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"qr-code\" data-part=\"frame\""));
        assert_eq!(
            html.matches("aria-label=\"共有リンクの QR コード\"")
                .count(),
            1
        );
        assert_eq!(html.matches("data-copied").count(), 0);
    }

    #[test]
    fn link_qr_version_selects_anyone_and_viewer_link_permission() {
        let html = demo_html();
        assert!(html.contains("リンクを知っている全員"));
        assert!(html.contains("リンクの権限"));
        assert!(html.contains("閲覧のみ"));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(html.contains("../../assets/blocks-demo-avatar.svg"));
    }

    #[test]
    fn layout_css_is_safe_and_stacks_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-share-members (max-width: 36rem)"));
        assert!(LAYOUT_CSS.contains(
            ".blocks-settings-share-members-link-row {\n    grid-template-columns: 1fr;"
        ));
    }

    #[test]
    fn selects_are_closed_and_disabled() {
        let html = demo_html();
        // 版 A: 共有範囲 1 + 権限 3 = 4 個。版 B: 共有範囲 1 + 権限 3 +
        // リンクの権限 1 = 5 個。合計 9 個の select trigger がすべて disabled。
        assert_eq!(
            html.matches("data-blocks-settings-share-members-select")
                .count(),
            9
        );
        assert!(html.contains(r#"aria-expanded="false""#));
    }

    #[test]
    fn share_link_uses_reserved_example_domain() {
        let html = demo_html();
        assert!(html.contains("share.example.com"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-settings-share-members-stack\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-settings-share-members-stack"
        );
    }
}
