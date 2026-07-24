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
//! `"remove"`（インデックス指定）/`"clear"` の単純アクションのみ受理する。
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
//!   `accept`/`multiple`）はすべて `&'static str` リテラルで固定しており、
//!   動的値が属性名スロットへ混入する経路はない。
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
use crate::aria::aria_label;
use crate::data_attrs::{data_disabled, data_dragging};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{codec, Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// FileUpload の anatomy（`data-scope="file-upload"`）。
const ANATOMY: Anatomy = anatomy("file-upload");

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

/// Root パーツ（`div`）。`data-disabled` を反映する。
#[must_use]
pub fn root<'a>(disabled: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Label パーツ（`label`）。意味論的な関連付けは呼び出し側が `attrs` 経由で
/// `for`/`id` を配線する（装飾用パーツ、他コンポーネントと同じ最小主義）。
#[must_use]
pub fn label<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("label", "label", attrs, children)
}

/// Dropzone パーツ（`div`）。ドラッグ&ドロップ領域。`role="button"` +
/// `tabindex="0"` でフォーカス可能にし、呼び出し側が `aria-label`（`attrs`
/// 経由）を与える。`dragging` は `fandhe-frontend-wasm-full` の配線層が
/// `dragenter`/`dragleave` に応じてトグルする `data-dragging`（DOM ローカル
/// 状態、hydration では運ばない）。
#[must_use]
pub fn dropzone<'a>(
    disabled: bool,
    dragging: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("role", "button"), ("tabindex", "0")];
    merged.extend(data_disabled(disabled));
    merged.extend(data_dragging(dragging));
    merged.extend(attrs);
    ANATOMY.part("dropzone", "div", merged, children)
}

/// Trigger パーツ（`button`）。クリックでファイル選択ダイアログを開く操作
/// （実際の [`hidden_input`] への `click()` 転送は wasm-full 配線層が担う）。
#[must_use]
pub fn trigger<'a>(disabled: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// ItemGroup パーツ（`ul`）。受理済みファイル [`item`] 群のコンテナ。
#[must_use]
pub fn item_group<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-group", "ul", attrs, children)
}

/// Item パーツ（`li`）。ファイル 1 個分のコンテナ（[`item_name`]/
/// [`item_size_text_node`]/[`item_delete_trigger`] を子に持つ）。
#[must_use]
pub fn item<'a>(disabled: bool, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item", "li", merged, children)
}

/// ItemName パーツ（`div`）。ファイル名を表示するテキストノードのコンテナ。
/// ファイル名は children として渡され `render()` の既定エスケープを経由する
/// （REQ-1 の重点対象、モジュール doc「セキュリティ不変条件」参照）。
#[must_use]
pub fn item_name<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-name", "div", attrs, children)
}

/// ItemSizeText パーツ（`div`）。[`item_size_text`] が生成した表示用文字列を
/// 描画するテキストノードのコンテナ。
#[must_use]
pub fn item_size_text_node<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("item-size-text", "div", attrs, children)
}

/// ItemDeleteTrigger パーツ（`button`）。当該ファイルを削除する操作。
/// `aria-label` は `format!` で組み立てた「Delete {name}」（動的値だが
/// `render()` の既定エスケープを経由するため注入経路にはならない、
/// [`crate::tags_input::item_delete_trigger`] と同型の判断）。
#[must_use]
pub fn item_delete_trigger<'a>(
    name: &str,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let label_str = format!("Delete {name}");
    let mut merged: Vec<(&str, &str)> = vec![("type", "button"), aria_label(label_str.as_str())];
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("item-delete-trigger", "button", merged, children)
}

/// ClearTrigger パーツ（`button`）。全ファイルを一括削除する操作。
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

/// HiddenInput パーツ（`input type="file"`）。実ファイル選択のネイティブ入力欄
/// （`fandhe-frontend-wasm-full` の配線層が `change`/`drop` を委譲で受ける
/// ターゲット）。`accept`/`multiple` はネイティブ属性として反映し、視覚的には
/// 非表示にする想定（呼び出し側が `attrs` でスタイルクラス等を与える）。
#[must_use]
pub fn hidden_input<'a>(
    accept: &'a str,
    multiple: bool,
    disabled: bool,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "file")];
    if !accept.is_empty() {
        merged.push(("accept", accept));
    }
    if multiple {
        merged.push(("multiple", ""));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(data_disabled(disabled));
    merged.extend(attrs);
    ANATOMY.part("hidden-input", "input", merged, Vec::new())
}

/// [`FileUpload`] に対する型付きアクション（WASM 境界からの接続は
/// [`fandhe_frontend_interactive::Component::update`] を型付きで直接呼ぶ
/// （[`FileUploadAction::AddFiles`]）か、文字列 dispatch
/// （[`FileUpload::decode_action`]、`"remove"`/`"clear"` のみ）を経由する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileUploadAction {
    /// 新規ファイル群を検証したうえで追加する（型付き API 限定、モジュール
    /// doc「dispatch 契約」節参照）。各ファイルは [`validate_incoming`] の
    /// 判定に従い受理または [`FileRejectionReason`] 付きで拒否される。
    AddFiles(Vec<FileUploadItem>),
    /// 指定インデックスの受理済みファイルを削除する（範囲外は no-op）。
    Remove(usize),
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

    /// [`dropzone`] へ現在の状態を注入する利便メソッド。`dragging` は DOM
    /// ローカル状態のため呼び出し側から与える。
    #[must_use]
    pub fn dropzone<'a>(
        &self,
        disabled: bool,
        dragging: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        dropzone(disabled, dragging, attrs, children)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(disabled, attrs, children)
    }

    /// [`hidden_input`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn hidden_input<'a>(
        &self,
        multiple: bool,
        disabled: bool,
        attrs: Vec<(&'a str, &'a str)>,
    ) -> Node {
        hidden_input(&self.accept, multiple, disabled, attrs)
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
        let items: Vec<Node> = self
            .accepted
            .iter()
            .map(|f| {
                let size_text = item_size_text(f.size_bytes);
                item(
                    false,
                    Vec::new(),
                    vec![
                        item_name(Vec::new(), vec![fandhe_frontend_core::text(&f.name)]),
                        item_size_text_node(
                            Vec::new(),
                            vec![fandhe_frontend_core::text(&size_text)],
                        ),
                        item_delete_trigger(&f.name, false, Vec::new(), Vec::new()),
                    ],
                )
            })
            .collect();
        self.root(
            false,
            Vec::new(),
            vec![
                self.label(Vec::new(), vec![fandhe_frontend_core::text("Files")]),
                self.dropzone(
                    false,
                    false,
                    Vec::new(),
                    vec![
                        self.trigger(
                            false,
                            Vec::new(),
                            vec![fandhe_frontend_core::text("Browse")],
                        ),
                        self.hidden_input(true, false, Vec::new()),
                    ],
                ),
                item_group(Vec::new(), items),
                clear_trigger(false, Vec::new(), vec![fandhe_frontend_core::text("Clear")]),
            ],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<FileUploadAction> {
        match name {
            "remove" => payload.parse::<usize>().ok().map(FileUploadAction::Remove),
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

    // --- 各パーツの data-scope/data-part/data-disabled 出力 ---

    #[test]
    fn root_outputs_scope_part_and_no_state_when_enabled() {
        let html = render(&root(false, vec![], vec![]));
        assert!(html.contains(r#"data-scope="file-upload""#));
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
        let html = render(&label(vec![], vec![text("Files")]));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains("Files"));
    }

    #[test]
    fn dropzone_outputs_role_button_and_tabindex() {
        let html = render(&dropzone(false, false, vec![], vec![]));
        assert!(html.contains(r#"data-part="dropzone""#));
        assert!(html.contains(r#"role="button""#));
        assert!(html.contains(r#"tabindex="0""#));
        assert!(!html.contains("data-dragging"));
    }

    #[test]
    fn dropzone_dragging_outputs_data_dragging() {
        let html = render(&dropzone(false, true, vec![], vec![]));
        assert!(html.contains(r#"data-dragging="""#));
    }

    #[test]
    fn trigger_outputs_type_button() {
        let html = render(&trigger(false, vec![], vec![text("Browse")]));
        assert!(html.contains(r#"data-part="trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn trigger_disabled_outputs_native_disabled() {
        let html = render(&trigger(true, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn item_group_outputs_scope_and_part() {
        let html = render(&item_group(vec![], vec![]));
        assert!(html.contains(r#"data-part="item-group""#));
    }

    #[test]
    fn item_outputs_scope_and_part() {
        let html = render(&item(false, vec![], vec![]));
        assert!(html.contains(r#"data-part="item""#));
    }

    #[test]
    fn item_name_outputs_scope_and_part_with_text() {
        let html = render(&item_name(vec![], vec![text("report.pdf")]));
        assert!(html.contains(r#"data-part="item-name""#));
        assert!(html.contains("report.pdf"));
    }

    #[test]
    fn item_size_text_node_outputs_scope_and_part() {
        let html = render(&item_size_text_node(vec![], vec![text("1.0 KB")]));
        assert!(html.contains(r#"data-part="item-size-text""#));
        assert!(html.contains("1.0 KB"));
    }

    #[test]
    fn item_delete_trigger_outputs_type_button_and_aria_label() {
        let html = render(&item_delete_trigger("report.pdf", false, vec![], vec![]));
        assert!(html.contains(r#"data-part="item-delete-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-label="Delete report.pdf""#));
    }

    #[test]
    fn clear_trigger_outputs_type_button() {
        let html = render(&clear_trigger(false, vec![], vec![text("Clear")]));
        assert!(html.contains(r#"data-part="clear-trigger""#));
        assert!(html.contains(r#"type="button""#));
    }

    #[test]
    fn hidden_input_outputs_type_file_accept_and_multiple() {
        let html = render(&hidden_input("image/*,.pdf", true, false, vec![]));
        assert!(html.contains(r#"data-part="hidden-input""#));
        assert!(html.contains(r#"type="file""#));
        assert!(html.contains(r#"accept="image/*,.pdf""#));
        assert!(html.contains(r#"multiple="""#));
    }

    #[test]
    fn hidden_input_without_accept_omits_accept_attr() {
        let html = render(&hidden_input("", false, false, vec![]));
        assert!(!html.contains("accept"));
    }

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            false,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="file-upload""#));
        assert!(html.contains(r#"data-part="root""#));
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
        let html = render(&item_name(vec![], vec![text(SCRIPT_PAYLOAD)]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn item_delete_trigger_aria_label_payload_is_escaped_on_render() {
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
    fn add_files_with_script_payload_name_then_render_escapes() {
        let mut f = FileUpload::default();
        f.update(FileUploadAction::AddFiles(vec![item_of(
            SCRIPT_PAYLOAD,
            10,
            "text/plain",
        )]));
        let name = &f.accepted()[0].name;
        let html = render(&item_name(vec![], vec![text(name)]));
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
        let html = render(&item_name(vec![], vec![text(&restored.accepted()[0].name)]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
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
}
