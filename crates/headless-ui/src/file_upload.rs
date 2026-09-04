//! FileUpload（ファイルアップロード）headless コンポーネント（イシュー #840、
//! 親トラッキング #520）。
//!
//! ark-ui の FileUpload
//!（`.claude/skills/ark-ui/references/components/form/file-upload.md`）を
//! 参考に、Root / Label / Dropzone / Trigger / ItemGroup / Item / ItemName /
//! ItemSizeText / ItemDeleteTrigger / ClearTrigger / HiddenInput の 11
//! anatomy パーツと、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装する値状態機械
//! [`FileUpload`] を提供する。
//!
//! # `docs/policy/intentional-non-adoption.md` §7 の保留解除（本イシューの核心）
//!
//! FileUpload 相当のコンポーネントは、`File` API 依存部分を
//! `fandhe-frontend-wasm-full` 側の限定配線に閉じ込め、本クレート
//! （headless-ui）を純粋関数のまま保てる設計が未確立であることを理由に
//! 保留とされていた。本モジュールはその再評価トリガーを次のとおり充足する:
//!
//! - **本モジュール（headless 層）はファイルメタデータ（[`FileUploadItem`]:
//!   name / size_bytes / mime_type の明示的構造体）のリストのみを保持し、
//!   `File` オブジェクト自体・`FileReader`・object URL の類は一切持たない
//!   （外部依存ゼロ・`#![forbid(unsafe_code)]` を維持したまま）。
//! - accept / max-files / max-size / min-size の検証・拒否理由の決定は
//!   すべて決定的な純粋関数（[`accept_matches`]/[`validate_incoming`]）で
//!   行う。
//! - 実 `File` オブジェクトへの接触（`input[type=file]`/ドロップゾーンからの
//!   読み取り）は `fandhe-frontend-wasm-full` の `headless_file_upload.rs`
//!   （#840）が 2 層構成（純粋ロジック層 + `#[cfg(target_arch = "wasm32")]`
//!   配線層）で隔離する。本クレートはそちらから型付きメタデータのみを
//!   受け取る。
//!
//! # 参照突合（イシュー #1609、zag.js `file-upload` machine /
//! ark-ui FileUpload デモとの anatomy・`data-*`・キーボード操作の突合）
//!
//! - **是正**: [`FileUploadProps`]（disabled/readonly/invalid/required）を
//!   root/label/dropzone/trigger/item 系（item-group/item/item-name/
//!   item-size-text）/item-delete-trigger/clear-trigger/hidden-input へ
//!   一律付与する（`data-disabled`/`data-readonly`/`data-invalid`/
//!   `data-required`）。[`dropzone`] は disabled または readonly のとき
//!   `tabindex="-1"` + `aria-disabled="true"` にし（[`crate::angle_slider`]
//!   の thumb と同じ規約）、呼び出し側 `attrs` に `aria-label`/
//!   `aria-labelledby` が無い場合のみ既定 `aria-label="dropzone"`
//!   （zag の既定訳と同値）を付与してアクセシブルネームを fail-closed に
//!   保証する。[`trigger`]/[`item_delete_trigger`]/[`clear_trigger`]/
//!   [`hidden_input`] は readonly でもネイティブ `disabled` を付与する
//!   （zag `disabled: disabled || readOnly` と同値）。[`hidden_input`] には
//!   `tabindex="-1"`・`aria-hidden="true"`・`aria-required`（`props.required`。
//!   ネイティブ `required` ではない。理由は [`hidden_input`] rustdoc 参照）を
//!   追加する。item-group/item/item-name/item-size-text/
//!   item-delete-trigger には [`ItemType`] 固定語彙による `data-type`
//!   （`"accepted"`/`"rejected"`）を付与する。[`clear_trigger`] は
//!   `hidden: bool` 引数を追加し受理 0 件で `hidden=""` を出力する。
//!   [`root`] には `data-readonly`/`data-dragging` を追加する。
//! - **意図的に追随しない**（`docs/policy/intentional-non-adoption.md`
//!   §3.25 規則 2 と同じ判断軸）: zag の `role="application"`
//!   （`disableClick` 時、本モジュールにクリック抑止 prop を持たない）・
//!   chakra 独自合成部品（`DropzoneContent`/`List`/`Items`/`FileText` 等、
//!   ark の primitives に存在しない）・zag のフォーカス・ポインタ系
//!   `data-*`（`data-focus`/`data-highlighted` 等）。item への
//!   `data-invalid` は参照側（zag/ark）も出力しないため予約しない
//!   （`fandhe-frontend-pre-styled-ui`/docs-site の Themes 側は引き続き
//!   `attrs` 経由で個別に付与できる）。
//! - **スコープ外（`out-of-scope-tracking.md` に従い Issue 化を提案）**:
//!   `fandhe-frontend-wasm-full` の dropzone Enter/Space keydown 配線
//!   （zag はこのキー操作でファイル選択ダイアログを起動する）と
//!   [`root`] の `data-dragging` の DOM ローカルトグル（REQ-11 バンドル
//!   予算）。キーボード専用利用者は [`trigger`]（ネイティブ `<button>`）で
//!   操作できるため a11y 上のブロッカーではない。
//!
//! # 独自状態機械にした理由（[`crate::state`] の既存型を使わない理由）
//!
//! [`crate::tags_input::TagsInput`] と同じ判断（同モジュール rustdoc 参照）で、
//! [`crate::state`] の既存型は「メタデータ付きファイル一覧 + 拒否理由」という
//! 語彙を表現できない。本モジュールも [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装し、Phase 1 が確立した
//! dispatch 契約（未知アクション no-op）・fail-closed hydration という
//! **統合様式**にのみ準拠する。
//!
//! # dispatch 契約（型付き API 限定、Cursor Bugbot 指摘済みの他コンポーネント
//! と同型の判断）
//!
//! ファイル追加（[`FileUploadAction::AddFiles`]）はメタデータ列
//! （`Vec<`[`FileUploadItem`]`>`）を伴うため、文字列 dispatch
//! （[`FileUpload::decode_action`]）では受けず、型付き
//! [`fandhe_frontend_interactive::Component::update`] 呼び出し（
//! `update(FileUploadAction::AddFiles(items))`）のみで受理する
//! （[`crate::radio_group::RadioGroup`] の型付き `Deselect` 限定の先例と同型。
//! クライアント文字列へファイルメタデータを載せない）。文字列 dispatch は
//! `"remove"`（`accepted` インデックス指定）/`"remove-rejected"`（`rejected`
//! インデックス指定、イシュー #1609 codex-review 再指摘）/`"clear"` の
//! 単純アクションのみ受理する。
//!
//! # スコープ外（イシュー #840 本文が明示、`out-of-scope-tracking.md` に従い
//! Issue 化を提案する）
//!
//! - ItemPreview / ItemPreviewImage（object URL による画像プレビュー）:
//!   プレビューは `File` オブジェクト・object URL の保持を要求し、本モジュール
//!   の「`File` オブジェクト非保持」設計と両立しない。
//! - `directory` / `capture` / `transformFiles` 相当の拡張 props。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`name`/`value`/`disabled`/
//!   `accept`/`multiple`/`tabindex`）はすべて `&'static str`
//!   リテラルで固定しており、動的値が属性名スロットへ混入する経路はない。
//!   [`ItemType::as_str`] が返す `data-type` の値語彙（`"accepted"`/
//!   `"rejected"`）・[`dropzone`] の既定 `aria-label="dropzone"` も同様に
//!   固定リテラルである。
//! - **ファイル名・MIME 文字列は攻撃者が完全に制御可能な入力そのものである
//!   （REQ-1 の重点対象）**: [`item_name`] のテキストノード・
//!   [`item_delete_trigger`] の `aria-label`・[`hidden_input`] の value の
//!   いずれも [`fandhe_frontend_core::render`] の既定エスケープを経由する
//!   経路以外を持たない（`raw_html()` は使用せず、HTML 文字列を直接組み立て
//!   ない。`tests/xss_escape.rs` の回帰テストで固定する）。
//! - **ファイル内容は一切読まない**: 本モジュールはメタデータ（name /
//!   size_bytes / mime_type）のみを扱い、`FileReader` 相当の内容読み取りは
//!   設計上存在しない（DoS 面のメモリ膨張経路がない）。
//! - accept / max_files / max_file_size / min_file_size の検証は fail-closed
//!   （違反は [`FileRejectionReason`] 付きで不受理、状態不変条件
//!   「`accepted.len() <= max_files`（設定時）」を破る入力は適用しない）。
//! - hydration 属性（`data-hydrate-*`）はクライアント側で改ざんされうる入力
//!   として扱う。[`FileUpload`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は panic せず `HydrateError` を返す（パース不能な数値・件数不整合・
//!   `accepted.len() > max_files` をすべて拒否する）。拒否リスト
//!   （`rejected`、ephemeral な直近拒否履歴）は運ばない
//!   （[`crate::tags_input::TagsInput`] の `editing` と同じ判断）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_disabled, aria_hidden, aria_label, aria_required};
use crate::data_attrs::{data_disabled, data_dragging, data_invalid, data_readonly, data_required};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{codec, Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// FileUpload の anatomy（`data-scope="file-upload"`）。
const ANATOMY: Anatomy = anatomy("file-upload");

/// FileUpload の disabled/readonly/invalid/required 状態束。
/// root/label/dropzone/trigger/item 系/item-delete-trigger/clear-trigger/
/// hidden-input へ一律付与するために使う（[`crate::color_picker::ColorPickerProps`]
/// と同型のパターン、モジュール doc「参照突合」節参照）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FileUploadProps {
    /// 無効化状態。`true` で `data-disabled` を各パーツへ付与し、
    /// ネイティブ操作可能パーツ（trigger/item-delete-trigger/clear-trigger/
    /// hidden-input）には `disabled` を、[`dropzone`] には
    /// `tabindex="-1"` + `aria-disabled="true"` を付与する。
    pub disabled: bool,
    /// 読み取り専用状態。`true` で `data-readonly` を各パーツへ付与する。
    /// zag `disabled: disabled || readOnly` と同値の判断により、
    /// trigger/item-delete-trigger/clear-trigger/hidden-input は
    /// readonly でもネイティブ `disabled` を付与し、[`dropzone`] も
    /// `tabindex="-1"` + `aria-disabled="true"` にする（新規ファイルの
    /// 追加操作を抑止する意図）。
    pub readonly: bool,
    /// 入力検証エラー状態。`true` で root/label/dropzone/trigger に
    /// `data-invalid` を付与する（item 系/item-delete-trigger/clear-trigger
    /// へは参照側〔zag/ark〕も出さないため付与しない、モジュール doc
    /// 「意図的に追随しない」節参照）。
    pub invalid: bool,
    /// 入力必須状態。`true` で [`label`] に `data-required` を付与する。
    pub required: bool,
}

/// [`FileUploadProps`] から disabled/invalid/readonly の状態属性列を
/// 組み立てる非公開ヘルパ（[`crate::color_picker::state_attrs`] と同型）。
fn state_attrs(props: &FileUploadProps) -> Vec<(&'static str, &'static str)> {
    let mut attrs: Vec<(&'static str, &'static str)> = Vec::new();
    attrs.extend(data_disabled(props.disabled));
    attrs.extend(data_invalid(props.invalid));
    attrs.extend(data_readonly(props.readonly));
    attrs
}

/// item 系パーツ（item-group/item/item-name/item-size-text/
/// item-delete-trigger）の `data-type` 固定語彙（zag `DEFAULT_ITEM_TYPE`
/// の既定値 `"accepted"` に対応）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ItemType {
    /// 検証を通過し受理された一覧に属するファイル（既定）。
    #[default]
    Accepted,
    /// [`FileUploadAction::AddFiles`] の検証で拒否されたファイル。
    Rejected,
}

impl ItemType {
    /// `data-type` 属性値の固定リテラル文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }
}

/// [`FileUploadProps`] が全パーツへ一律付与する属性キー一覧。呼び出し側
/// `attrs` にこれらと同名キーが含まれていても fail-closed で除去する対象
/// （[`crate::color_picker::STATE_RESERVED`] と同型のパターン）。
const STATE_RESERVED: &[&str] = &["data-disabled", "data-invalid", "data-readonly"];

/// [`root`] が固定付与するキー一覧（[`STATE_RESERVED`] に `data-dragging`
/// を加えたもの）。
const ROOT_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-dragging",
];

/// [`label`] が固定付与するキー一覧（[`STATE_RESERVED`] に `data-required`
/// を加えたもの）。
const LABEL_RESERVED: &[&str] = &[
    "data-disabled",
    "data-invalid",
    "data-readonly",
    "data-required",
];

/// [`dropzone`] が固定付与するキー一覧。`aria-label` は呼び出し側指定が
/// 無い場合のみ既定値を付与する条件付き属性のため予約対象に含めない
/// （呼び出し側が明示指定した場合はそちらを優先させる意図）。
const DROPZONE_RESERVED: &[&str] = &[
    "role",
    "tabindex",
    "aria-disabled",
    "data-invalid",
    "data-disabled",
    "data-readonly",
    "data-dragging",
];

/// [`trigger`] が固定付与するキー一覧（[`STATE_RESERVED`]、`disabled`は
/// ネイティブ boolean 属性のため別枠で処理する）。
const TRIGGER_RESERVED: &[&str] = STATE_RESERVED;

/// item-group/item/item-name/item-size-text が固定付与するキー一覧。
const ITEM_RESERVED: &[&str] = &["data-disabled", "data-readonly", "data-type"];

/// [`item_delete_trigger`] が固定付与するキー一覧。
const ITEM_DELETE_TRIGGER_RESERVED: &[&str] = &["data-disabled", "data-readonly", "data-type"];

/// [`clear_trigger`] が固定付与するキー一覧。
const CLEAR_TRIGGER_RESERVED: &[&str] = &["data-disabled", "data-readonly"];

/// [`hidden_input`] が固定付与するキー一覧。
const HIDDEN_INPUT_RESERVED: &[&str] = &[
    "type",
    "tabindex",
    "aria-hidden",
    "aria-required",
    "accept",
    "multiple",
    "disabled",
    "data-disabled",
    "data-readonly",
    "data-required",
];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（[`crate::color_picker::drop_reserved`]/
/// [`crate::angle_slider::drop_reserved`] と同型の重複実装。モジュール間の
/// 相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// `attrs` に呼び出し側が既に `aria-label`/`aria-labelledby`
/// のいずれかを指定しているかを判定する（[`dropzone`] の既定
/// `aria-label` を上書きしないための判定、ASCII 大文字小文字無視）。
fn has_accessible_name(attrs: &[(&str, &str)]) -> bool {
    attrs.iter().any(|(k, _)| {
        k.eq_ignore_ascii_case("aria-label") || k.eq_ignore_ascii_case("aria-labelledby")
    })
}

/// 受理済み・拒否済みファイル 1 個分のメタデータ。
///
/// `File` オブジェクトへの参照・内容は一切保持しない（モジュール doc
/// 「保留解除」節参照）。`fandhe-frontend-wasm-full` の配線層が
/// `File::name()`/`File::size()`/`File::type_()` から変換して生成する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileUploadItem {
    /// ファイル名（攻撃者制御可能な入力そのもの、REQ-1 の重点対象）。
    pub name: String,
    /// バイト単位のサイズ。
    pub size_bytes: u64,
    /// MIME タイプ文字列（ブラウザ判定に依存し空文字列もありうる）。
    pub mime_type: String,
}

impl FileUploadItem {
    /// メタデータを指定して生成する。
    #[must_use]
    pub fn new(name: impl Into<String>, size_bytes: u64, mime_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size_bytes,
            mime_type: mime_type.into(),
        }
    }
}

/// ファイル拒否理由（ark-ui の検証エラー語彙に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRejectionReason {
    /// `max_files` 上限に到達しているため受理できない。
    TooManyFiles,
    /// `accept` に一致しない MIME タイプ/拡張子。
    FileInvalidType,
    /// `max_file_size` を超過している。
    FileTooLarge,
    /// `min_file_size` を下回っている。
    FileTooSmall,
    /// 同名・同サイズのファイルが既に受理済み一覧に存在する。
    FileExists,
}

/// `accept` 文字列（カンマ区切り、ark-ui/HTML `<input accept>` と同じ語彙）が
/// 与えられたファイルに一致するかを判定する（決定的純粋関数）。
///
/// 3 形式をサポートする: 完全一致 MIME（例 `"application/pdf"`）・
/// ワイルドカード（例 `"image/*"`）・拡張子（例 `".pdf"`、ファイル名の
/// 大文字小文字を無視して末尾一致）。`accept` が空文字列の場合は無制限
/// （常に一致）として扱う。
#[must_use]
pub fn accept_matches(mime_type: &str, file_name: &str, accept: &str) -> bool {
    if accept.trim().is_empty() {
        return true;
    }
    accept
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .any(|pattern| {
            if let Some(prefix) = pattern.strip_suffix("/*") {
                mime_type
                    .split('/')
                    .next()
                    .is_some_and(|top| top.eq_ignore_ascii_case(prefix))
            } else if let Some(ext) = pattern.strip_prefix('.') {
                file_name
                    .to_ascii_lowercase()
                    .ends_with(&format!(".{}", ext.to_ascii_lowercase()))
            } else {
                mime_type.eq_ignore_ascii_case(pattern)
            }
        })
}

/// 1 件の新規ファイルを現行の受理済み一覧・設定に照らして検証する
/// （決定的純粋関数）。`Ok(())` なら受理可能、`Err` なら拒否理由を返す。
#[must_use = "拒否理由を無視すると受理判定が失われる"]
pub fn validate_incoming(
    item: &FileUploadItem,
    accepted: &[FileUploadItem],
    accept: &str,
    max_files: Option<usize>,
    max_file_size: Option<u64>,
    min_file_size: Option<u64>,
) -> Result<(), FileRejectionReason> {
    if max_files.is_some_and(|m| accepted.len() >= m) {
        return Err(FileRejectionReason::TooManyFiles);
    }
    if !accept_matches(&item.mime_type, &item.name, accept) {
        return Err(FileRejectionReason::FileInvalidType);
    }
    if let Some(max) = max_file_size {
        if item.size_bytes > max {
            return Err(FileRejectionReason::FileTooLarge);
        }
    }
    if let Some(min) = min_file_size {
        if item.size_bytes < min {
            return Err(FileRejectionReason::FileTooSmall);
        }
    }
    if accepted
        .iter()
        .any(|existing| existing.name == item.name && existing.size_bytes == item.size_bytes)
    {
        return Err(FileRejectionReason::FileExists);
    }
    Ok(())
}

/// バイト数を決定的な表示用文字列へ変換する（純粋関数。丸め規則:
/// 1024 未満は整数バイト表示、以降は 1024 単位で KB/MB/GB へ切り替え、
/// 小数第 1 位まで表示する）。
#[must_use]
pub fn item_size_text(size_bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let bytes = size_bytes as f64;
    if size_bytes < 1024 {
        format!("{size_bytes} B")
    } else if bytes < MB {
        format!("{:.1} KB", bytes / KB)
    } else if bytes < GB {
        format!("{:.1} MB", bytes / MB)
    } else {
        format!("{:.1} GB", bytes / GB)
    }
}

/// Root パーツ（`div`）。[`FileUploadProps`] の状態束 + `data-dragging` を
/// 反映する。`dragging` は `fandhe-frontend-wasm-full` の配線層が
/// `dragenter`/`dragleave` に応じてトグルする DOM ローカル状態
/// （hydration では運ばない）。
#[must_use]
pub fn root<'a>(
    props: &FileUploadProps,
    dragging: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(data_dragging(dragging));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。意味論的な関連付けは呼び出し側が `attrs` 経由で
/// `for`/`id` を配線する（装飾用パーツ、他コンポーネントと同じ最小主義）。
/// [`FileUploadProps`] の状態束 + `data-required` を付与する。
#[must_use]
pub fn label<'a>(
    props: &FileUploadProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, LABEL_RESERVED);
    let mut merged = state_attrs(props);
    merged.extend(data_required(props.required));
    merged.extend(attrs);
    ANATOMY.part("label", "label", merged, children)
}

/// Dropzone パーツ（`div`）。ドラッグ&ドロップ領域。`role="button"` を
/// 常に付与し、disabled または readonly のときは `tabindex="-1"` +
/// `aria-disabled="true"`（zag の readOnly 時ドロップ無視・クリック抑止と
/// 対応、`fandhe-frontend-wasm-full` 側のガードは別途必要）、それ以外は
/// `tabindex="0"` にする（[`crate::angle_slider`] の thumb と同じ規約）。
/// 呼び出し側 `attrs` に `aria-label`/`aria-labelledby` が無ければ既定
/// `aria-label="dropzone"`（zag の既定訳と同値）を付与し、アクセシブル
/// ネームを fail-closed に保証する。`dragging` は [`root`] と同じ DOM
/// ローカル状態。
#[must_use]
pub fn dropzone<'a>(
    props: &FileUploadProps,
    dragging: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, DROPZONE_RESERVED);
    let needs_default_label = !has_accessible_name(&attrs);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("role", "button")];
    if props.disabled || props.readonly {
        merged.push(("tabindex", "-1"));
        merged.push(aria_disabled(true));
    } else {
        merged.push(("tabindex", "0"));
    }
    merged.extend(state_attrs(props));
    merged.extend(data_dragging(dragging));
    if needs_default_label {
        merged.push(aria_label("dropzone"));
    }
    merged.extend(attrs);
    ANATOMY.part("dropzone", "div", merged, children)
}

/// Trigger パーツ（`button`）。クリックでファイル選択ダイアログを開く操作
/// （実際の [`hidden_input`] への `click()` 転送は wasm-full 配線層が担う）。
/// zag `disabled: disabled || readOnly` と同値の判断で readonly でも
/// ネイティブ `disabled` を付与する。
#[must_use]
pub fn trigger<'a>(
    props: &FileUploadProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, TRIGGER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if props.disabled || props.readonly {
        merged.push(("disabled", ""));
    }
    merged.extend(state_attrs(props));
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// ItemGroup パーツ（`ul`）。受理済み・拒否済み [`item`] 群のコンテナ。
/// `item_type` は配下の [`item`] 群が共通して持つ [`ItemType`]（呼び出し側が
/// 受理済み一覧・拒否済み一覧を別々の `item_group` で描画する構成を前提とする）。
/// [`FileUploadProps`] の `disabled`/`readonly` を `data-disabled`/
/// `data-readonly` として伝播する。
#[must_use]
pub fn item_group<'a>(
    item_type: ItemType,
    props: &FileUploadProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.push(("data-type", item_type.as_str()));
    merged.extend(attrs);
    ANATOMY.part("item-group", "ul", merged, children)
}

/// Item パーツ（`li`）。ファイル 1 個分のコンテナ（[`item_name`]/
/// [`item_size_text_node`]/[`item_delete_trigger`] を子に持つ）。
/// [`ItemType`] による `data-type` を付与し、[`FileUploadProps`] の
/// `disabled`/`readonly` を `data-disabled`/`data-readonly` として伝播する。
#[must_use]
pub fn item<'a>(
    item_type: ItemType,
    props: &FileUploadProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.push(("data-type", item_type.as_str()));
    merged.extend(attrs);
    ANATOMY.part("item", "li", merged, children)
}

/// ItemName パーツ（`div`）。ファイル名を表示するテキストノードのコンテナ。
/// ファイル名は children として渡され `render()` の既定エスケープを経由する
/// （REQ-1 の重点対象、モジュール doc「セキュリティ不変条件」参照）。
/// [`ItemType`] による `data-type` を付与し、[`FileUploadProps`] の
/// `disabled`/`readonly` を `data-disabled`/`data-readonly` として伝播する。
#[must_use]
pub fn item_name<'a>(
    item_type: ItemType,
    props: &FileUploadProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.push(("data-type", item_type.as_str()));
    merged.extend(attrs);
    ANATOMY.part("item-name", "div", merged, children)
}

/// ItemSizeText パーツ（`div`）。[`item_size_text`] が生成した表示用文字列を
/// 描画するテキストノードのコンテナ。[`ItemType`] による `data-type` を
/// 付与し、[`FileUploadProps`] の `disabled`/`readonly` を
/// `data-disabled`/`data-readonly` として伝播する。
#[must_use]
pub fn item_size_text_node<'a>(
    item_type: ItemType,
    props: &FileUploadProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.push(("data-type", item_type.as_str()));
    merged.extend(attrs);
    ANATOMY.part("item-size-text", "div", merged, children)
}

/// ItemDeleteTrigger パーツ（`button`）。当該ファイルを削除する操作。
/// `aria-label` は `format!` で組み立てた「Delete {name}」（動的値だが
/// `render()` の既定エスケープを経由するため注入経路にはならない、
/// [`crate::tags_input::item_delete_trigger`] と同型の判断）。zag
/// `disabled: disabled || readOnly` と同値の判断で readonly でもネイティブ
/// `disabled` を付与し、[`ItemType`] による `data-type` を付与する。
#[must_use]
pub fn item_delete_trigger<'a>(
    name: &str,
    item_type: ItemType,
    props: &FileUploadProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ITEM_DELETE_TRIGGER_RESERVED);
    let label_str = format!("Delete {name}");
    let mut merged: Vec<(&str, &str)> = vec![("type", "button"), aria_label(label_str.as_str())];
    if props.disabled || props.readonly {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.push(("data-type", item_type.as_str()));
    merged.extend(attrs);
    ANATOMY.part("item-delete-trigger", "button", merged, children)
}

/// ClearTrigger パーツ（`button`）。全ファイルを一括削除する操作。zag の
/// 「受理済みファイルが 0 件のとき `hidden`」を `hidden` 引数として受け取る
/// （状態機械から導出する利便メソッドは [`FileUpload::clear_trigger`]）。
/// readonly でもネイティブ `disabled` を付与する。
#[must_use]
pub fn clear_trigger<'a>(
    props: &FileUploadProps,
    hidden: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, CLEAR_TRIGGER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if props.disabled || props.readonly {
        merged.push(("disabled", ""));
    }
    if hidden {
        merged.push(("hidden", ""));
    }
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.extend(attrs);
    ANATOMY.part("clear-trigger", "button", merged, children)
}

/// HiddenInput パーツ（`input type="file"`）。実ファイル選択のネイティブ入力欄
/// （`fandhe-frontend-wasm-full` の配線層が `change`/`drop` を委譲で受ける
/// ターゲット）。`accept`/`multiple` はネイティブ属性として反映し、視覚的には
/// 非表示にする想定（呼び出し側が `attrs` でスタイルクラス等を与える）。
/// `tabindex="-1"` + `aria-hidden="true"`（zag と同値、フォーカス・
/// スクリーンリーダー走査の対象外にする）を付与し、readonly でもネイティブ
/// `disabled` を付与する。`data-readonly` も [`FileUploadProps`] から伝播する。
///
/// **`required` はネイティブ `required` 属性ではなく `aria-required` +
/// `data-required`（[`aria_required`]/[`data_required`]）として表現する
/// （codex-review 再指摘、イシュー #1609）。本パーツは実 `FileList` を
/// 保持しない設計（`wasm-full` の配線層が `change` 直後に値をクリアする）
/// であり、ネイティブ `required` を付けると受理済みファイルが存在しても
/// ネイティブフォーム送信が constraint validation により常にブロックされて
/// しまう。UI コンポーネント層はバリデーション・送信処理を内包しない
/// （`.claude/rules/coding-rust.md` §UI 部品の責務境界）ため、要求有無を
/// `aria-required`/`data-required` で提示するに留め、実際の必須検証は
/// [`FileUpload::accepted`] を読んだ呼び出し側アプリケーションコードに委ねる。
#[must_use]
pub fn hidden_input<'a>(
    accept: &'a str,
    multiple: bool,
    props: &FileUploadProps,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let attrs = drop_reserved(attrs, HIDDEN_INPUT_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "file"),
        ("tabindex", "-1"),
        aria_hidden(true),
        aria_required(props.required),
    ];
    if !accept.is_empty() {
        merged.push(("accept", accept));
    }
    if multiple {
        merged.push(("multiple", ""));
    }
    if props.disabled || props.readonly {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(props.disabled));
    merged.extend(data_readonly(props.readonly));
    merged.extend(data_required(props.required));
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// [`FileUpload`] に対する型付きアクション（WASM 境界からの接続は
/// [`fandhe_frontend_interactive::Component::update`] を型付きで直接呼ぶ
/// （[`FileUploadAction::AddFiles`]）か、文字列 dispatch
/// （[`FileUpload::decode_action`]、`"remove"`/`"remove-rejected"`/`"clear"`
/// のみ）を経由する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileUploadAction {
    /// 新規ファイル群を検証したうえで追加する（型付き API 限定、モジュール
    /// doc「dispatch 契約」節参照）。各ファイルは [`validate_incoming`] の
    /// 判定に従い受理または [`FileRejectionReason`] 付きで拒否される。
    AddFiles(Vec<FileUploadItem>),
    /// 指定インデックスの受理済みファイルを削除する（範囲外は no-op）。
    Remove(usize),
    /// 指定インデックスの拒否済みファイル（`rejected` 一覧）を削除する
    /// （範囲外は no-op、イシュー #1609 codex-review 再指摘）。[`item_delete_trigger`]
    /// は `ItemType::Rejected` にも削除ボタンを提供するため、`accepted` 専用の
    /// [`Self::Remove`] とは別に `rejected` を対象とするインデックス操作を
    /// 用意する（`rejected` は ephemeral な UI 状態であり削除しても
    /// hydration/`AddFiles` 契約は変わらない）。
    RemoveRejected(usize),
    /// 受理済み・拒否済みの全ファイルをクリアする。
    Clear,
}

/// FileUpload の値状態機械。
///
/// `accepted` は受理済みファイルの表示順一覧（不変条件:
/// `max_files.is_some()` の場合 `len() <= max_files`。[`Self::update`]/
/// [`Self::from_hydration_attrs`] のいずれの経路でも破られない）。
/// `rejected` は直近の [`FileUploadAction::AddFiles`] 呼び出しで拒否された
/// ファイルとその理由の一覧という ephemeral な UI 状態であり、hydration
/// では運ばない（モジュール doc 参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileUpload {
    accepted: Vec<FileUploadItem>,
    rejected: Vec<(FileUploadItem, FileRejectionReason)>,
    accept: String,
    max_files: Option<usize>,
    max_file_size: Option<u64>,
    min_file_size: Option<u64>,
}

impl Default for FileUpload {
    /// 既定は空一覧・無制限（`accept` 無指定・上限なし）。
    fn default() -> Self {
        Self::new(String::new(), None, None, None)
    }
}

impl FileUpload {
    /// `data-hydrate-file-names` 属性名のフィールド部分。
    pub const FIELD_NAMES: &'static str = "file-names";
    /// `data-hydrate-file-sizes` 属性名のフィールド部分。
    pub const FIELD_SIZES: &'static str = "file-sizes";
    /// `data-hydrate-file-mimes` 属性名のフィールド部分。
    pub const FIELD_MIMES: &'static str = "file-mimes";
    /// `data-hydrate-accept` 属性名のフィールド部分。
    pub const FIELD_ACCEPT: &'static str = "accept";
    /// `data-hydrate-max-files` 属性名のフィールド部分。
    pub const FIELD_MAX_FILES: &'static str = "max-files";
    /// `data-hydrate-max-size` 属性名のフィールド部分。
    pub const FIELD_MAX_SIZE: &'static str = "max-size";
    /// `data-hydrate-min-size` 属性名のフィールド部分。
    pub const FIELD_MIN_SIZE: &'static str = "min-size";

    /// `accept` 文字列・`max_files`・`max_file_size`・`min_file_size`
    /// （すべて `None` = 無制限）を指定して空の [`FileUpload`] を生成する。
    #[must_use]
    pub fn new(
        accept: impl Into<String>,
        max_files: Option<usize>,
        max_file_size: Option<u64>,
        min_file_size: Option<u64>,
    ) -> Self {
        Self {
            accepted: Vec::new(),
            rejected: Vec::new(),
            accept: accept.into(),
            max_files,
            max_file_size,
            min_file_size,
        }
    }

    /// 現在の受理済みファイル一覧（表示順）。
    #[must_use]
    pub fn accepted(&self) -> &[FileUploadItem] {
        &self.accepted
    }

    /// 直近の追加試行で拒否されたファイルとその理由の一覧。
    #[must_use]
    pub fn rejected(&self) -> &[(FileUploadItem, FileRejectionReason)] {
        &self.rejected
    }

    /// 受理済みファイル数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.accepted.len()
    }

    /// 受理済みファイルが 1 個もないか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.accepted.is_empty()
    }

    /// `accept` 文字列。
    #[must_use]
    pub fn accept(&self) -> &str {
        &self.accept
    }

    /// `max_files`（`None` = 無制限）。
    #[must_use]
    pub fn max_files(&self) -> Option<usize> {
        self.max_files
    }

    /// 上限に到達しているか（`max_files` が `None` の場合は常に `false`）。
    #[must_use]
    pub fn is_at_max(&self) -> bool {
        self.max_files.is_some_and(|m| self.accepted.len() >= m)
    }

    /// フォーム送信値（受理済みファイル名をカンマ結合。[`Self::hidden_input`]
    /// が使う簡易表現。実ファイル本体の送信は本モジュールの責務外）。
    #[must_use]
    pub fn value(&self) -> String {
        self.accepted
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// [`root`] へ委譲する利便メソッド（状態を持たないため素通し、`props`/
    /// `dragging` は呼び出し側が与える）。
    #[must_use]
    pub fn root<'a>(
        &self,
        props: &FileUploadProps,
        dragging: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        root(props, dragging, attrs, children)
    }

    /// [`label`] へ委譲する利便メソッド（状態を持たないため素通し）。
    #[must_use]
    pub fn label<'a>(
        &self,
        props: &FileUploadProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        label(props, attrs, children)
    }

    /// [`dropzone`] へ委譲する利便メソッド。`dragging` は DOM ローカル状態の
    /// ため呼び出し側から与える。
    #[must_use]
    pub fn dropzone<'a>(
        &self,
        props: &FileUploadProps,
        dragging: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        dropzone(props, dragging, attrs, children)
    }

    /// [`trigger`] へ委譲する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        props: &FileUploadProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(props, attrs, children)
    }

    /// [`hidden_input`] へ現在の `accept` を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        multiple: bool,
        props: &FileUploadProps,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        hidden_input(&self.accept, multiple, props, attrs)
    }

    /// [`clear_trigger`] へ現在の状態（`is_empty()`）から導出した `hidden`
    /// を注入する利便メソッド。
    #[must_use]
    pub fn clear_trigger<'a>(
        &self,
        props: &FileUploadProps,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        clear_trigger(props, self.is_empty(), attrs, children)
    }
}

impl Component for FileUpload {
    type Action = FileUploadAction;

    fn update(&mut self, action: FileUploadAction) {
        match action {
            FileUploadAction::AddFiles(items) => {
                // 直近の拒否履歴を今回の試行内容で置き換える（ephemeral な
                // UI 状態、モジュール doc「rejected」節参照）。
                self.rejected.clear();
                for item in items {
                    match validate_incoming(
                        &item,
                        &self.accepted,
                        &self.accept,
                        self.max_files,
                        self.max_file_size,
                        self.min_file_size,
                    ) {
                        Ok(()) => self.accepted.push(item),
                        Err(reason) => self.rejected.push((item, reason)),
                    }
                }
            }
            FileUploadAction::Remove(idx) => {
                if idx < self.accepted.len() {
                    self.accepted.remove(idx);
                }
            }
            FileUploadAction::RemoveRejected(idx) => {
                if idx < self.rejected.len() {
                    self.rejected.remove(idx);
                }
            }
            FileUploadAction::Clear => {
                self.accepted.clear();
                self.rejected.clear();
            }
        }
    }

    /// 共通契約（hydration ルート）のみを表す最小正準ビュー
    /// （root > label + dropzone(trigger + hidden-input) +
    /// item-group > item × len + clear-trigger）。公開 UI としての利用は
    /// 想定しない（実際の UI 構築は各パーツメソッドを呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        let props = FileUploadProps::default();
        let items: Vec<Node> = self
            .accepted
            .iter()
            .map(|f| {
                let size_text = item_size_text(f.size_bytes);
                item(
                    ItemType::Accepted,
                    &props,
                    Vec::new(),
                    vec![
                        item_name(
                            ItemType::Accepted,
                            &props,
                            Vec::new(),
                            vec![fandhe_frontend_core::text(&f.name)],
                        ),
                        item_size_text_node(
                            ItemType::Accepted,
                            &props,
                            Vec::new(),
                            vec![fandhe_frontend_core::text(&size_text)],
                        ),
                        item_delete_trigger(
                            &f.name,
                            ItemType::Accepted,
                            &props,
                            Vec::new(),
                            Vec::new(),
                        ),
                    ],
                )
            })
            .collect();
        self.root(
            &props,
            false,
            Vec::new(),
            vec![
                self.label(
                    &props,
                    Vec::new(),
                    vec![fandhe_frontend_core::text("Files")],
                ),
                self.dropzone(
                    &props,
                    false,
                    Vec::new(),
                    vec![
                        self.trigger(
                            &props,
                            Vec::new(),
                            vec![fandhe_frontend_core::text("Browse")],
                        ),
                        self.hidden_input(true, &props, Vec::new()),
                    ],
                ),
                item_group(ItemType::Accepted, &props, Vec::new(), items),
                self.clear_trigger(
                    &props,
                    Vec::new(),
                    vec![fandhe_frontend_core::text("Clear")],
                ),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<FileUploadAction> {
        match name {
            "remove" => payload.parse::<usize>().ok().map(FileUploadAction::Remove),
            "remove-rejected" => payload
                .parse::<usize>()
                .ok()
                .map(FileUploadAction::RemoveRejected),
            "clear" => Some(FileUploadAction::Clear),
            _ => None,
        }
    }
}

/// `Option<u64>` を hydration 属性値へエンコードする（`"none"` または
/// 非負整数文字列、[`crate::tags_input::TagsInput`] の `max` エンコードと
/// 同型）。
fn encode_optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |v| v.to_string())
}

/// `Option<usize>` を hydration 属性値へエンコードする。
fn encode_optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_string(), |v| v.to_string())
}

/// hydration 属性値から `Option<u64>` を復元する（改ざん耐性: パース不能な
/// 値は `HydrateError::InvalidValue` を返す）。
fn decode_optional_u64(attr: &str, raw: &str) -> Result<Option<u64>, HydrateError> {
    if raw == "none" {
        Ok(None)
    } else {
        raw.parse::<u64>()
            .map(Some)
            .map_err(|_| HydrateError::InvalidValue {
                attr: attr.to_string(),
                reason: "expected \"none\" or a non-negative integer".to_string(),
            })
    }
}

/// hydration 属性値から `Option<usize>` を復元する。
fn decode_optional_usize(attr: &str, raw: &str) -> Result<Option<usize>, HydrateError> {
    if raw == "none" {
        Ok(None)
    } else {
        raw.parse::<usize>()
            .map(Some)
            .map_err(|_| HydrateError::InvalidValue {
                attr: attr.to_string(),
                reason: "expected \"none\" or a non-negative integer".to_string(),
            })
    }
}

impl Hydrate for FileUpload {
    /// 受理済みファイルのメタデータは 3 本の並行リスト
    /// （`file-names`/`file-sizes`/`file-mimes`、いずれも
    /// [`codec::encode_list`] でエンコード）として運ぶ。`accept`/`max-files`/
    /// `max-size`/`min-size` という設定値も併せて運び、復元時の再検証に使う。
    /// `rejected`（ephemeral な直近拒否履歴）は運ばない（モジュール doc
    /// 参照）。
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let names: Vec<String> = self.accepted.iter().map(|f| f.name.clone()).collect();
        let sizes: Vec<String> = self
            .accepted
            .iter()
            .map(|f| f.size_bytes.to_string())
            .collect();
        let mimes: Vec<String> = self.accepted.iter().map(|f| f.mime_type.clone()).collect();
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_NAMES),
                codec::encode_list(&names),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SIZES),
                codec::encode_list(&sizes),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIMES),
                codec::encode_list(&mimes),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ACCEPT),
                self.accept.clone(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX_FILES),
                encode_optional_usize(self.max_files),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX_SIZE),
                encode_optional_u64(self.max_file_size),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIN_SIZE),
                encode_optional_u64(self.min_file_size),
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

        let names_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_NAMES);
        let sizes_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_SIZES);
        let max_files_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX_FILES);

        let names = codec::decode_list(find(Self::FIELD_NAMES)?);
        let sizes_raw = codec::decode_list(find(Self::FIELD_SIZES)?);
        let mimes = codec::decode_list(find(Self::FIELD_MIMES)?);
        let accept = find(Self::FIELD_ACCEPT)?.to_string();
        let max_files = decode_optional_usize(&max_files_attr, find(Self::FIELD_MAX_FILES)?)?;
        let max_file_size = decode_optional_u64(
            &format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX_SIZE),
            find(Self::FIELD_MAX_SIZE)?,
        )?;
        let min_file_size = decode_optional_u64(
            &format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MIN_SIZE),
            find(Self::FIELD_MIN_SIZE)?,
        )?;

        // 3 本の並行リストの件数が一致しない改ざん入力は復元しない
        // （fail-closed。TagsInput の重複/長さ検証と同型の判断）。
        if names.len() != sizes_raw.len() || names.len() != mimes.len() {
            return Err(HydrateError::InvalidValue {
                attr: names_attr,
                reason: "file-names/file-sizes/file-mimes length mismatch".to_string(),
            });
        }

        let mut accepted = Vec::with_capacity(names.len());
        for ((name, size_raw), mime) in names.into_iter().zip(sizes_raw).zip(mimes) {
            let size_bytes: u64 = size_raw.parse().map_err(|_| HydrateError::InvalidValue {
                attr: sizes_attr.clone(),
                reason: "expected a non-negative integer byte size".to_string(),
            })?;
            accepted.push(FileUploadItem::new(name, size_bytes, mime));
        }

        // 復元一覧が「max_files 設定時は len() <= max_files」という不変条件を
        // 満たすことを検証する。改ざんされた data-* によって不変条件を破った
        // 状態を復元しない（fail-closed、TagsInput 相当の判断）。
        if let Some(m) = max_files {
            if accepted.len() > m {
                return Err(HydrateError::InvalidValue {
                    attr: names_attr,
                    reason: "accepted files length exceeds max_files".to_string(),
                });
            }
        }

        Ok(Self {
            accepted,
            // 直近拒否履歴は ephemeral な UI 状態のため運ばない
            // （モジュール doc 参照）。復元直後は常に空。
            rejected: Vec::new(),
            accept,
            max_files,
            max_file_size,
            min_file_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn item_of(name: &str, size: u64, mime: &str) -> FileUploadItem {
        FileUploadItem::new(name, size, mime)
    }

    fn enabled() -> FileUploadProps {
        FileUploadProps::default()
    }

    fn disabled() -> FileUploadProps {
        FileUploadProps {
            disabled: true,
            ..Default::default()
        }
    }

    fn readonly() -> FileUploadProps {
        FileUploadProps {
            readonly: true,
            ..Default::default()
        }
    }

    // --- 各パーツの data-scope/data-part/状態属性出力 ---

    #[test]
    fn root_outputs_scope_part_and_no_state_when_enabled() {
        let html = render(&root(&enabled(), false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="file-upload""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains("data-readonly"));
    }

    #[test]
    fn root_disabled_true_outputs_data_disabled() {
        let html = render(&root(&disabled(), false, vec![], vec![]));
        assert!(html.contains(r#"data-disabled="""#));
    }

    #[test]
    fn root_readonly_true_outputs_data_readonly() {
        let html = render(&root(&readonly(), false, vec![], vec![]));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn label_outputs_scope_and_part() {
        let html = render(&label(&enabled(), vec![], vec![text("Files")]));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains("Files"));
    }

    #[test]
    fn label_required_true_outputs_data_required() {
        let props = FileUploadProps {
            required: true,
            ..Default::default()
        };
        let html = render(&label(&props, vec![], vec![]));
        assert!(html.contains(r#"data-required="""#));
    }

    #[test]
    fn dropzone_outputs_role_button_tabindex_and_default_aria_label() {
        let html = render(&dropzone(&enabled(), false, vec![], vec![]));
        assert!(html.contains(r#"data-part="dropzone""#));
        assert!(html.contains(r#"role="button""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(html.contains(r#"aria-label="dropzone""#));
        assert!(!html.contains("data-dragging"));
        assert!(!html.contains("aria-disabled"));
    }

    #[test]
    fn dropzone_dragging_outputs_data_dragging() {
        let html = render(&dropzone(&enabled(), true, vec![], vec![]));
        assert!(html.contains(r#"data-dragging="""#));
    }

    #[test]
    fn dropzone_disabled_outputs_tabindex_negative_one_and_aria_disabled() {
        let html = render(&dropzone(&disabled(), false, vec![], vec![]));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn dropzone_readonly_outputs_tabindex_negative_one_and_aria_disabled() {
        let html = render(&dropzone(&readonly(), false, vec![], vec![]));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-disabled="true""#));
    }

    #[test]
    fn dropzone_caller_aria_label_overrides_default() {
        let html = render(&dropzone(
            &enabled(),
            false,
            vec![("aria-label", "Upload files here")],
            vec![],
        ));
        assert!(html.contains(r#"aria-label="Upload files here""#));
        assert!(!html.contains(r#"aria-label="dropzone""#));
    }

    #[test]
    fn dropzone_caller_aria_labelledby_suppresses_default_label() {
        let html = render(&dropzone(
            &enabled(),
            false,
            vec![("aria-labelledby", "external-label")],
            vec![],
        ));
        assert!(html.contains(r#"aria-labelledby="external-label""#));
        assert!(!html.contains("aria-label=\"dropzone\""));
    }

    #[test]
    fn trigger_outputs_type_button() {
        let html = render(&trigger(&enabled(), vec![], vec![text("Browse")]));
        assert!(html.contains(r#"data-part="trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn trigger_disabled_outputs_native_disabled() {
        let html = render(&trigger(&disabled(), vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn trigger_readonly_outputs_native_disabled() {
        let html = render(&trigger(&readonly(), vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn item_group_outputs_scope_part_and_data_type() {
        let html = render(&item_group(ItemType::Accepted, &enabled(), vec![], vec![]));
        assert!(html.contains(r#"data-part="item-group""#));
        assert!(html.contains(r#"data-type="accepted""#));
    }

    #[test]
    fn item_group_rejected_outputs_data_type_rejected() {
        let html = render(&item_group(ItemType::Rejected, &enabled(), vec![], vec![]));
        assert!(html.contains(r#"data-type="rejected""#));
    }

    #[test]
    fn item_outputs_scope_part_and_data_type() {
        let html = render(&item(ItemType::Accepted, &enabled(), vec![], vec![]));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"data-type="accepted""#));
    }

    #[test]
    fn item_caller_data_invalid_is_kept() {
        // item への data-invalid は参照側（zag/ark）も出さないため予約せず、
        // 呼び出し側 attrs 経由の付与を許可する（モジュール doc「意図的に
        // 追随しない」節参照）。
        let html = render(&item(
            ItemType::Accepted,
            &enabled(),
            vec![("data-invalid", "")],
            vec![],
        ));
        assert!(html.contains(r#"data-invalid="""#));
    }

    #[test]
    fn item_name_outputs_scope_part_and_data_type_with_text() {
        let html = render(&item_name(
            ItemType::Accepted,
            &enabled(),
            vec![],
            vec![text("report.pdf")],
        ));
        assert!(html.contains(r#"data-part="item-name""#));
        assert!(html.contains(r#"data-type="accepted""#));
        assert!(html.contains("report.pdf"));
    }

    #[test]
    fn item_size_text_node_outputs_scope_part_and_data_type() {
        let html = render(&item_size_text_node(
            ItemType::Accepted,
            &enabled(),
            vec![],
            vec![text("1.0 KB")],
        ));
        assert!(html.contains(r#"data-part="item-size-text""#));
        assert!(html.contains(r#"data-type="accepted""#));
        assert!(html.contains("1.0 KB"));
    }

    #[test]
    fn item_delete_trigger_outputs_type_button_aria_label_and_data_type() {
        let html = render(&item_delete_trigger(
            "report.pdf",
            ItemType::Accepted,
            &enabled(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-part="item-delete-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Delete report.pdf""#));
        assert!(html.contains(r#"data-type="accepted""#));
    }

    #[test]
    fn item_delete_trigger_readonly_outputs_native_disabled() {
        let html = render(&item_delete_trigger(
            "a",
            ItemType::Accepted,
            &readonly(),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn clear_trigger_outputs_type_button() {
        let html = render(&clear_trigger(
            &enabled(),
            false,
            vec![],
            vec![text("Clear")],
        ));
        assert!(html.contains(r#"data-part="clear-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn clear_trigger_hidden_true_outputs_hidden_attr() {
        let html = render(&clear_trigger(&enabled(), true, vec![], vec![]));
        assert!(html.contains(r#"hidden="""#));
    }

    #[test]
    fn clear_trigger_readonly_outputs_native_disabled() {
        let html = render(&clear_trigger(&readonly(), false, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
        assert!(html.contains(r#"data-readonly="""#));
    }

    #[test]
    fn hidden_input_outputs_type_file_tabindex_aria_hidden_accept_and_multiple() {
        let html = render(&hidden_input("image/*,.pdf", true, &enabled(), vec![]));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="file""#));
        assert!(html.contains(r#"tabindex="-1""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"accept="image/*,.pdf""#));
        assert!(html.contains(r#"multiple="""#));
        assert!(html.contains(r#"aria-required="false""#));
        assert!(!html.contains(r#"required="""#));
    }

    #[test]
    fn hidden_input_without_accept_omits_accept_attr() {
        let html = render(&hidden_input("", false, &enabled(), vec![]));
        assert!(!html.contains("accept"));
    }

    /// 手順 2 の是正対象（codex-review 再指摘、イシュー #1609）: `required: true`
    /// でもネイティブ `required` 属性は出力しない（実 `FileList` を保持しない
    /// 設計と衝突し、ネイティブフォーム送信を常にブロックしてしまうため）。
    /// 表現は `aria-required`/`data-required` に限定する。
    #[test]
    fn hidden_input_required_true_outputs_aria_and_data_required_not_native() {
        let props = FileUploadProps {
            required: true,
            ..Default::default()
        };
        let html = render(&hidden_input("", false, &props, vec![]));
        assert!(html.contains(r#"aria-required="true""#));
        assert!(html.contains(r#"data-required="""#));
        assert!(!html.contains(r#" required="""#));
    }

    #[test]
    fn hidden_input_readonly_outputs_native_disabled() {
        let html = render(&hidden_input("", false, &readonly(), vec![]));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            &enabled(),
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="file-upload""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn caller_supplied_reserved_keys_are_dropped_from_dropzone() {
        // role/tabindex/aria-disabled/data-* はフレームワーク固定のため、
        // 呼び出し側 attrs の同名キーは無効化される（fail-closed、
        // モジュール doc「セキュリティ不変条件」参照）。
        let html = render(&dropzone(
            &enabled(),
            false,
            vec![("role", "attacker"), ("tabindex", "99")],
            vec![],
        ));
        assert!(html.contains(r#"role="button""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("attacker"));
        assert!(!html.contains(r#"tabindex="99""#));
    }

    #[test]
    fn caller_supplied_data_type_is_dropped_from_item() {
        let html = render(&item(
            ItemType::Accepted,
            &enabled(),
            vec![("data-type", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-type="accepted""#));
        assert!(!html.contains("attacker"));
    }

    // --- accept_matches ---

    #[test]
    fn accept_matches_empty_pattern_matches_anything() {
        assert!(accept_matches("application/pdf", "x.pdf", ""));
    }

    #[test]
    fn accept_matches_exact_mime() {
        assert!(accept_matches(
            "application/pdf",
            "x.pdf",
            "application/pdf"
        ));
        assert!(!accept_matches("text/plain", "x.txt", "application/pdf"));
    }

    #[test]
    fn accept_matches_wildcard_mime_top_level() {
        assert!(accept_matches("image/png", "x.png", "image/*"));
        assert!(!accept_matches("application/pdf", "x.pdf", "image/*"));
    }

    #[test]
    fn accept_matches_extension_case_insensitive() {
        assert!(accept_matches("application/pdf", "REPORT.PDF", ".pdf"));
        assert!(!accept_matches("text/plain", "x.txt", ".pdf"));
    }

    #[test]
    fn accept_matches_multiple_comma_separated_patterns() {
        assert!(accept_matches("image/png", "x.png", "image/*,.pdf"));
        assert!(accept_matches("application/pdf", "x.pdf", "image/*,.pdf"));
        assert!(!accept_matches("text/plain", "x.txt", "image/*,.pdf"));
    }

    // --- item_size_text ---

    #[test]
    fn item_size_text_bytes() {
        assert_eq!(item_size_text(512), "512 B");
        assert_eq!(item_size_text(0), "0 B");
        assert_eq!(item_size_text(1023), "1023 B");
    }

    #[test]
    fn item_size_text_kilobytes() {
        assert_eq!(item_size_text(1024), "1.0 KB");
        assert_eq!(item_size_text(2048), "2.0 KB");
    }

    #[test]
    fn item_size_text_megabytes() {
        assert_eq!(item_size_text(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn item_size_text_gigabytes() {
        assert_eq!(item_size_text(1024 * 1024 * 1024 * 2), "2.0 GB");
    }

    // --- FileUpload: 状態機械 ---

    #[test]
    fn default_is_empty_and_unlimited() {
        let f = FileUpload::default();
        assert_eq!(f.len(), 0);
        assert!(f.is_empty());
        assert_eq!(f.max_files(), None);
        assert!(!f.is_at_max());
    }

    #[test]
    fn add_files_appends_valid_items() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![
            item_of("a.txt", 10, "text/plain"),
            item_of("b.txt", 20, "text/plain"),
        ]));
        assert_eq!(f.len(), 2);
        assert!(f.rejected().is_empty());
    }

    #[test]
    fn add_files_rejects_invalid_type() {
        let mut f = FileUpload::new("image/*", None, None, None);
        f.update(FileUploadAction::AddFiles(vec![item_of(
            "a.txt",
            10,
            "text/plain",
        )]));
        assert!(f.is_empty());
        assert_eq!(f.rejected().len(), 1);
        assert_eq!(f.rejected()[0].1, FileRejectionReason::FileInvalidType);
    }

    #[test]
    fn add_files_rejects_too_large() {
        let mut f = FileUpload::new("", None, Some(100), None);
        f.update(FileUploadAction::AddFiles(vec![item_of(
            "a.bin",
            200,
            "application/octet-stream",
        )]));
        assert!(f.is_empty());
        assert_eq!(f.rejected()[0].1, FileRejectionReason::FileTooLarge);
    }

    #[test]
    fn add_files_rejects_too_small() {
        let mut f = FileUpload::new("", None, None, Some(100));
        f.update(FileUploadAction::AddFiles(vec![item_of(
            "a.bin",
            10,
            "application/octet-stream",
        )]));
        assert!(f.is_empty());
        assert_eq!(f.rejected()[0].1, FileRejectionReason::FileTooSmall);
    }

    #[test]
    fn add_files_rejects_when_at_max() {
        let mut f = FileUpload::new("", Some(1), None, None);
        f.update(FileUploadAction::AddFiles(vec![item_of("a", 1, "")]));
        assert!(f.is_at_max());
        f.update(FileUploadAction::AddFiles(vec![item_of("b", 1, "")]));
        assert_eq!(f.len(), 1);
        assert_eq!(f.rejected()[0].1, FileRejectionReason::TooManyFiles);
    }

    #[test]
    fn add_files_rejects_duplicate_name_and_size() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![item_of("a", 1, "")]));
        f.update(FileUploadAction::AddFiles(vec![item_of("a", 1, "")]));
        assert_eq!(f.len(), 1);
        assert_eq!(f.rejected()[0].1, FileRejectionReason::FileExists);
    }

    #[test]
    fn add_files_clears_previous_rejections_on_new_attempt() {
        let mut f = FileUpload::new("image/*", None, None, None);
        f.update(FileUploadAction::AddFiles(vec![item_of(
            "a.txt",
            1,
            "text/plain",
        )]));
        assert_eq!(f.rejected().len(), 1);
        f.update(FileUploadAction::AddFiles(vec![item_of(
            "b.png",
            1,
            "image/png",
        )]));
        assert!(f.rejected().is_empty());
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn remove_action_removes_by_index() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![
            item_of("a", 1, ""),
            item_of("b", 1, ""),
        ]));
        assert!(dispatch(&mut f, "remove", "0"));
        assert_eq!(f.accepted()[0].name, "b");
    }

    #[test]
    fn remove_action_out_of_range_is_no_op() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![item_of("a", 1, "")]));
        assert!(dispatch(&mut f, "remove", "5"));
        assert_eq!(f.len(), 1);
    }

    /// codex-review 再指摘（イシュー #1609）の回帰テスト:
    /// `"remove-rejected"` 文字列 dispatch が `rejected` 一覧を対象に
    /// インデックス削除する（`accepted` 専用の `"remove"` とは独立）。
    #[test]
    fn remove_rejected_action_removes_by_index() {
        let mut f = FileUpload::new(String::new(), None, Some(0), None);
        f.update(FileUploadAction::AddFiles(vec![
            item_of("a", 1, ""),
            item_of("b", 1, ""),
        ]));
        assert_eq!(f.rejected().len(), 2);
        assert!(dispatch(&mut f, "remove-rejected", "0"));
        assert_eq!(f.rejected().len(), 1);
        assert_eq!(f.rejected()[0].0.name, "b");
    }

    #[test]
    fn remove_rejected_action_out_of_range_is_no_op() {
        let mut f = FileUpload::new(String::new(), None, Some(0), None);
        f.update(FileUploadAction::AddFiles(vec![item_of("a", 1, "")]));
        assert_eq!(f.rejected().len(), 1);
        assert!(dispatch(&mut f, "remove-rejected", "5"));
        assert_eq!(f.rejected().len(), 1);
    }

    #[test]
    fn clear_action_removes_all_files() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![item_of("a", 1, "")]));
        assert!(dispatch(&mut f, "clear", ""));
        assert!(f.is_empty());
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut f = FileUpload::default();
        assert!(!dispatch(&mut f, "no_such_action", "x"));
        assert!(f.is_empty());
    }

    // --- FileUpload: SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&FileUpload::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- FileUpload: hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let mut f = FileUpload::new("image/*", Some(5), None, None);
        f.update(FileUploadAction::AddFiles(vec![item_of(
            "a.png",
            100,
            "image/png",
        )]));
        let rendered = render(&render_for_hydration(&f));
        assert!(rendered.contains(r#"data-hydrate-max-files="5""#));

        let restored = FileUpload::from_hydration_attrs(&f.hydration_attrs()).unwrap();
        assert_eq!(restored.accepted(), f.accepted());
        assert_eq!(restored.max_files(), f.max_files());
        assert!(restored.rejected().is_empty());
    }

    #[test]
    fn hydration_round_trip_unlimited_encodes_none() {
        let f = FileUpload::default();
        let attrs = f.hydration_attrs();
        assert!(attrs
            .iter()
            .any(|(k, v)| k.ends_with("max-files") && v == "none"));
        let restored = FileUpload::from_hydration_attrs(&attrs).unwrap();
        assert_eq!(restored.max_files(), None);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = FileUpload::from_hydration_attrs(&[]).unwrap_err();
        assert!(matches!(err, HydrateError::MissingAttr(_)));
    }

    #[test]
    fn from_hydration_attrs_invalid_max_files_does_not_panic() {
        let f = FileUpload::default();
        let mut attrs = f.hydration_attrs();
        for (k, v) in attrs.iter_mut() {
            if k.ends_with("max-files") {
                *v = "not-a-number".to_string();
            }
        }
        let err = FileUpload::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_length_mismatch_does_not_panic() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![item_of(
            "a",
            1,
            "text/plain",
        )]));
        let mut attrs = f.hydration_attrs();
        for (k, v) in attrs.iter_mut() {
            if k.ends_with("file-sizes") {
                *v = codec::encode_list(&["1".to_string(), "2".to_string()]);
            }
        }
        let err = FileUpload::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_exceeds_max_files_does_not_panic() {
        let mut f = FileUpload::new("", Some(5), None, None);
        f.update(FileUploadAction::AddFiles(vec![
            item_of("a", 1, ""),
            item_of("b", 1, ""),
        ]));
        let mut attrs = f.hydration_attrs();
        for (k, v) in attrs.iter_mut() {
            if k.ends_with("max-files") {
                *v = "1".to_string();
            }
        }
        let err = FileUpload::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn hydration_does_not_carry_rejected_state() {
        let mut f = FileUpload::new("image/*", None, None, None);
        f.update(FileUploadAction::AddFiles(vec![item_of(
            "a.txt",
            1,
            "text/plain",
        )]));
        assert_eq!(f.rejected().len(), 1);
        let restored = FileUpload::from_hydration_attrs(&f.hydration_attrs()).unwrap();
        assert!(restored.rejected().is_empty());
    }

    // --- XSS 回帰: ファイル名/MIME にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
    const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

    #[test]
    fn item_name_payload_is_escaped_on_render() {
        let html = render(&item_name(
            ItemType::Accepted,
            &enabled(),
            vec![],
            vec![text(SCRIPT_PAYLOAD)],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn item_delete_trigger_aria_label_payload_is_escaped_on_render() {
        let html = render(&item_delete_trigger(
            ATTR_BREAK_PAYLOAD,
            ItemType::Accepted,
            &enabled(),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn add_files_with_script_payload_name_then_render_escapes() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![item_of(
            SCRIPT_PAYLOAD,
            10,
            "text/plain",
        )]));
        let name = &f.accepted()[0].name;
        let html = render(&item_name(
            ItemType::Accepted,
            &enabled(),
            vec![],
            vec![text(name)],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_tampered_name_with_script_payload_round_trips_but_escapes_on_render() {
        let attrs = vec![
            (
                "data-hydrate-file-names".to_string(),
                codec::encode_list(&[SCRIPT_PAYLOAD.to_string()]),
            ),
            (
                "data-hydrate-file-sizes".to_string(),
                codec::encode_list(&["10".to_string()]),
            ),
            (
                "data-hydrate-file-mimes".to_string(),
                codec::encode_list(&["text/plain".to_string()]),
            ),
            ("data-hydrate-accept".to_string(), String::new()),
            ("data-hydrate-max-files".to_string(), "none".to_string()),
            ("data-hydrate-max-size".to_string(), "none".to_string()),
            ("data-hydrate-min-size".to_string(), "none".to_string()),
        ];
        let restored = FileUpload::from_hydration_attrs(&attrs).unwrap();
        let html = render(&item_name(
            ItemType::Accepted,
            &enabled(),
            vec![],
            vec![text(&restored.accepted()[0].name)],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            &enabled(),
            false,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    // --- 参照突合契約（イシュー #1609）: 11 パート + FileUploadProps 伝播 ---

    #[test]
    fn reference_anatomy_part_names_match_reference() {
        // ark-ui/zag の FileUpload anatomy 11 パーツが本モジュールの
        // data-part 語彙と一致することを固定する（ItemPreview/
        // ItemPreviewImage はスコープ外、モジュール doc 参照）。
        const EXPECTED_PARTS: &[&str] = &[
            "root",
            "label",
            "dropzone",
            "trigger",
            "item-group",
            "item",
            "item-name",
            "item-size-text",
            "item-delete-trigger",
            "clear-trigger",
            "hidden-input",
        ];
        let props = enabled();
        let html = render(&root(
            &props,
            false,
            vec![],
            vec![
                label(&props, vec![], vec![]),
                dropzone(
                    &props,
                    false,
                    vec![],
                    vec![
                        trigger(&props, vec![], vec![]),
                        hidden_input("", false, &props, vec![]),
                    ],
                ),
                item_group(
                    ItemType::Accepted,
                    &props,
                    vec![],
                    vec![item(
                        ItemType::Accepted,
                        &props,
                        vec![],
                        vec![
                            item_name(ItemType::Accepted, &props, vec![], vec![]),
                            item_size_text_node(ItemType::Accepted, &props, vec![], vec![]),
                            item_delete_trigger("a", ItemType::Accepted, &props, vec![], vec![]),
                        ],
                    )],
                ),
                clear_trigger(&props, false, vec![], vec![]),
            ],
        ));
        for part in EXPECTED_PARTS {
            assert!(
                html.contains(&format!(r#"data-part="{part}""#)),
                "missing data-part={part}"
            );
        }
    }

    #[test]
    fn invalid_true_propagates_to_root_label_dropzone_trigger_item_family() {
        let props = FileUploadProps {
            invalid: true,
            ..Default::default()
        };
        assert!(render(&root(&props, false, vec![], vec![])).contains("data-invalid"));
        assert!(render(&label(&props, vec![], vec![])).contains("data-invalid"));
        assert!(render(&dropzone(&props, false, vec![], vec![])).contains("data-invalid"));
        assert!(render(&trigger(&props, vec![], vec![])).contains("data-invalid"));
    }

    #[test]
    fn invalid_true_does_not_propagate_to_item_hidden_input_or_clear_trigger() {
        // item・hidden-input・item-delete-trigger・clear-trigger は参照側
        // （zag/ark）も data-invalid を出さないため予約しない（モジュール
        // doc「意図的に追随しない」節参照）。
        let props = FileUploadProps {
            invalid: true,
            ..Default::default()
        };
        assert!(!render(&item(ItemType::Accepted, &props, vec![], vec![])).contains("data-invalid"));
        assert!(!render(&hidden_input("", false, &props, vec![])).contains("data-invalid"));
        assert!(!render(&item_delete_trigger(
            "a",
            ItemType::Accepted,
            &props,
            vec![],
            vec![]
        ))
        .contains("data-invalid"));
        assert!(!render(&clear_trigger(&props, false, vec![], vec![])).contains("data-invalid"));
    }

    #[test]
    fn file_upload_clear_trigger_convenience_derives_hidden_from_is_empty() {
        let mut f = FileUpload::default();
        let props = FileUploadProps::default();
        let html_empty = render(&f.clear_trigger(&props, vec![], vec![]));
        assert!(html_empty.contains(r#"hidden="""#));

        f.update(FileUploadAction::AddFiles(vec![item_of("a", 1, "")]));
        let html_nonempty = render(&f.clear_trigger(&props, vec![], vec![]));
        assert!(!html_nonempty.contains("hidden"));
    }
}
