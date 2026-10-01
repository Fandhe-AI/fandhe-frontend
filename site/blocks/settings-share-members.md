# settings-share-members

共有範囲の選択・メール招待・アクセス権選択付きメンバー一覧・共有リンクの
コピー操作を 1 枚のカードにまとめた共有設定ブロックです。`card` /
`select` / `input-group` / `input` / `text` / `avatar` / `clipboard` /
`separator` / `field` / `heading` / `qr-code` の 11 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

版 A「招待したメンバーのみ」（対応表 ID R0323、主参照）と版 B「リンクを
知っている全員（QR コード付き）」（R0322 + R0031）の 2 版を並記します。
版 B は共有範囲を「リンクを知っている全員」へ広げ、共有リンク領域を
QR コード + リンクのコピー操作 + 「リンクの権限」select の 2 列へ拡張
します。メール招待・メンバー一覧（アクセス権選択付き）は両版で共有し、
状態違いは共有範囲 select の初期選択値（招待制 / リンク公開）で表します。
氏名・メールアドレス・共有リンク・QR コードの値はすべて架空のデータで
あり、実在の人物・組織・URL は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。共有範囲・
メンバーごとの権限選択・リンクの権限選択はいずれも `disabled: true` で
固定した閉じた `select` です（操作しても開閉が追従できないため）。共有
リンクのコピー操作自体は `fandhe-frontend-wasm-full` の配線を持つため、
実アプリへ組み込めば機能します。`clipboard` の「コピー済み」状態は、
実際に押していない静的表示へ出すと誤解を招くため並記しません
（`hero_install_command`〔#2786〕の是正と同じ判断）。

コンテナ幅が 36rem 未満になると、メンバー一覧の権限選択が氏名の下へ
折り返され、版 B の共有リンク領域も QR コードの下にリンクが回る 1 列
表示に切り替わります。

## Rust コード

```rust
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
```

## 原案差分メモ

- **R0323 → 版 A（招待したメンバーのみ）**: 共有範囲の `select`（初期値
  「招待したメンバーのみ」）・メール招待の `input-group`・アクセス権
  選択付きメンバー一覧 3 件・共有リンクの `clipboard` を、`separator` で
  区切って縦積みにした基本形です。
- **R0322 → 版 B の QR コード**: 共有範囲を「リンクを知っている全員」へ
  広げた状態で、共有リンク領域へ QR コード（`qr-code` 部品）を左列として
  追加します。
- **R0031 → 版 B のリンク権限 + 両版共通のメンバー権限**: 読み取り
  リンクの権限（「閲覧のみ」/「編集可」）を版 B の共有リンク領域右列へ
  「リンクの権限」select として追加します。メンバーごとのアクセス権
  選択は R0323 から引き継ぎ両版で共有します。
- **状態違いの並記**: 共有範囲 select の初期選択値（招待制 / リンク
  公開）を版 A/B で切り替えることで表します。`clipboard` の「コピー
  済み」状態（操作結果）は並記の対象にしません。

関連情報: [Card](../themes/card.md) / [Select](../themes/select.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Text](../themes/text.md) / [Avatar](../themes/avatar.md) /
[Clipboard](../themes/clipboard.md) / [Separator](../themes/separator.md) /
[Field](../themes/field.md) / [Heading](../themes/heading.md) /
[QR Code](../themes/qr-code.md)
