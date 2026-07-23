//! Toast（一時的な通知の queue 表示、イシュー #760、親トラッキング #520）。
//!
//! ark-ui の Toast（`.claude/skills/ark-ui/references/components/overlays/toast.md`）
//! を参考に、group（live region）/ root（通知 1 件）/ title / description /
//! action-trigger / close-trigger の 6 anatomy パーツと、複数通知を有界な
//! キューとして管理する状態機械 [`Toaster`] を提供する。
//!
//! # 状態機械について（[`crate::state`] を使わない理由）
//!
//! Toast の状態は「複数エントリからなるキュー」であり、[`crate::state::Disclosure`]
//! （開閉 2 値）にも [`crate::state::SingleSelect`]（選択インデックス）にも
//! 意味的に写像できない。[`crate::avatar::Avatar`] と同様、本モジュールは
//! [`fandhe_frontend_interactive::Component`]/[`fandhe_frontend_interactive::Hydrate`]
//! を直接実装し、Phase 1 が確立した dispatch 契約（未知アクション no-op）・
//! fail-closed hydration という**統合様式**にのみ準拠する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`group`]/[`root`]/[`title`]/[`description`]/
//! [`action_trigger`]/[`close_trigger`]、純粋関数で完結）を直接呼んで組み立てる。
//! CSR/hydration は [`Toaster`] を経由し、dispatch（`"dismiss"`/`"clear"`）で
//! 状態遷移する。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! スタイル済み Toast（placement variant・status 配色）を組み立てる想定である。
//!
//! # `aria-live`（イシュー本文が指定する挙動）
//!
//! [`root`] の `aria-live` は [`ToastStatus`] から決定的に導出する（呼び出し側
//! 文字列を直接流し込まない）。[`ToastStatus::Error`] のみ `"assertive"`（緊急度の
//! 高い通知として即座に割り込ませる）、他は `"polite"`。`aria-atomic="true"` を
//! 併用し、通知全体を単位として読み上げさせる。
//!
//! # スコープ外
//!
//! - タイマーによる自動 dismiss の実配線（duration 管理・`"dismiss"` dispatch
//!   の実発火）は `fandhe-frontend-wasm-full` の後続イシューのスコープ。
//! - `"push"` の文字列 dispatch（payload の構造化エンコードが必要）は本モジュール
//!   では提供しない。[`Toaster::push`] を SSR/サーバー側から直接呼ぶ、または
//!   クライアント側の型付き API 経由で行う想定（`decode_action` は `"push"` を
//!   受理しない、本モジュール冒頭「未知アクション no-op」契約参照）。
//! - `ActionTrigger` の実際の動作配線・promise/loading 対応（ark-ui 固有機能）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`role`/`aria-*`）はすべて `&'static str` リテラルまたは
//!   固定スロットであり、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::data_attrs`]/[`crate::aria`] の既存不変条件を
//!   継承する）。
//! - 動的値（`id`/`title`/`description`/呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - hydration 属性（`data-hydrate-*`）はクライアント側で改ざんされうる入力
//!   として扱う。[`Toaster`] の [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   panic せず `HydrateError` を返す（4 リストの長さ不一致・未知語彙・`max`
//!   パース失敗・`id` 重複・`max` 超過はすべて fail-closed で拒否する）。
//! - キューは `max` で有界（[`Toaster::push`] が超過時に最古を押し出す。
//!   hydration 復元時も `max` 超過は拒否し、無制限のエントリ復元を許さない）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_atomic, aria_label, aria_live, role, AriaLive};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::codec::{decode_list, encode_list};
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Toast の anatomy（`data-scope="toast"`）。
const ANATOMY: Anatomy = anatomy("toast");

/// [`Toaster::default`] が使う既定の最大表示件数（ark-ui の例に倣う）。
pub const DEFAULT_MAX: usize = 24;

/// Toast のステータス（既定 `Info`）。
///
/// 値語彙は `fandhe-frontend-pre-styled-ui` の `AlertStatus` と同一
/// （`"info"`/`"success"`/`"warning"`/`"error"`）とし、status 別配色の
/// 整合を取る（`docs/api/pre-styled-ui-api.md` 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastStatus {
    /// 情報提供（既定）。
    #[default]
    Info,
    /// 成功。
    Success,
    /// 警告。
    Warning,
    /// エラー。
    Error,
}

impl ToastStatus {
    /// `data-type` 属性値文字列（[`root`] が付与する ark-ui 準拠の status フック）。
    #[must_use]
    pub const fn as_data_status(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// [`ToastStatus::as_data_status`] の逆変換。未知の値は `None`
    /// （安全側、呼び出し元が `HydrateError::InvalidValue` へ変換する）。
    #[must_use]
    pub fn from_data_status(s: &str) -> Option<Self> {
        match s {
            "info" => Some(Self::Info),
            "success" => Some(Self::Success),
            "warning" => Some(Self::Warning),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// この status に対応する `aria-live` 緊急度。
    ///
    /// [`ToastStatus::Error`] のみ `Assertive`（即座に割り込ませる）、他は
    /// `Polite`（本モジュール冒頭の rustdoc「`aria-live`」節参照）。
    #[must_use]
    pub const fn aria_live_urgency(self) -> AriaLive {
        match self {
            Self::Error => AriaLive::Assertive,
            Self::Info | Self::Success | Self::Warning => AriaLive::Polite,
        }
    }
}

/// Toast の表示位置（ark-ui 準拠の 6 語彙、既定 `BottomEnd`）。
///
/// [`crate::positioning::Placement`]（anchor positioning の 12 語彙、
/// Popover/Tooltip/Menu/Select が要素同士の相対位置を表すために使う）とは
/// 意味論が異なる独立 enum である。Toast の `placement` はビューポート角への
/// 固定配置（`position: fixed` 前提）を表し、アンカー要素は持たない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPlacement {
    /// 上端・始端（LTR で左上）。
    TopStart,
    /// 上端・中央。
    Top,
    /// 上端・終端（LTR で右上）。
    TopEnd,
    /// 下端・始端（LTR で左下）。
    BottomStart,
    /// 下端・中央。
    Bottom,
    /// 下端・終端（LTR で右下、既定値。ark-ui の例に合わせる）。
    #[default]
    BottomEnd,
}

impl ToastPlacement {
    /// `data-placement` 属性値文字列。
    #[must_use]
    pub const fn as_data_placement(self) -> &'static str {
        match self {
            Self::TopStart => "top-start",
            Self::Top => "top",
            Self::TopEnd => "top-end",
            Self::BottomStart => "bottom-start",
            Self::Bottom => "bottom",
            Self::BottomEnd => "bottom-end",
        }
    }

    /// [`ToastPlacement::as_data_placement`] の逆変換。未知の値は `None`。
    #[must_use]
    pub fn from_data_placement(s: &str) -> Option<Self> {
        match s {
            "top-start" => Some(Self::TopStart),
            "top" => Some(Self::Top),
            "top-end" => Some(Self::TopEnd),
            "bottom-start" => Some(Self::BottomStart),
            "bottom" => Some(Self::Bottom),
            "bottom-end" => Some(Self::BottomEnd),
            _ => None,
        }
    }
}

/// 通知 1 件（キューの要素）。
///
/// `id` は呼び出し側が供給する一意識別子（本モジュールは時刻・乱数を使わない
/// 決定的なキュー操作のみを行う）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastEntry {
    /// 一意識別子（[`ToastAction::Dismiss`] が参照する）。
    pub id: String,
    /// ステータス（`aria-live`/`data-type` を決める）。
    pub status: ToastStatus,
    /// タイトル文字列。
    pub title: String,
    /// 説明文字列。
    pub description: String,
}

/// group パーツ（`div`。live region 全体を束ねる、ark-ui にはない補助パーツ）。
///
/// `role="region"` + `aria-label`（`label` は必須引数。[`crate::avatar::image`]
/// の `alt` 必須化と同じアクセシビリティ担保方針）+ `data-placement` を付与する。
#[must_use]
pub fn group<'a>(
    placement: ToastPlacement,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("region"),
        aria_label(label),
        ("data-placement", placement.as_data_placement()),
    ];
    merged.extend(attrs);
    ANATOMY.part("group", "div", merged, children)
}

/// root パーツ（`div`。通知 1 件）。
///
/// `role="status"` + `aria-atomic="true"` + `aria-live`（[`ToastStatus::aria_live_urgency`]）
/// + `data-type`（ark-ui 準拠の status フック）を付与する。
#[must_use]
pub fn root<'a>(status: ToastStatus, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("status"),
        aria_atomic(true),
        aria_live(status.aria_live_urgency()),
        ("data-type", status.as_data_status()),
    ];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// title パーツ（`div`）。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("title", "div", attrs, children)
}

/// description パーツ（`div`）。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("description", "div", attrs, children)
}

/// action-trigger パーツ（`button type="button"`。呼び出し側が定義するアクションボタン）。
#[must_use]
pub fn action_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("action-trigger", "button", merged, children)
}

/// close-trigger パーツ（`button type="button"`）。
#[must_use]
pub fn close_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("close-trigger", "button", merged, children)
}

/// Toaster のアクション（WASM 境界の文字列 dispatch と
/// [`Toaster::decode_action`] で接続する）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastAction {
    /// 通知を追加する（同一 `id` の既存エントリは更新扱いで置き換える）。
    /// 文字列 dispatch からは到達しない（本モジュール冒頭「スコープ外」参照）。
    Push(ToastEntry),
    /// 指定した `id` の通知を除去する。不一致は no-op。
    Dismiss {
        /// 除去対象の `id`。
        id: String,
    },
    /// 全通知を除去する。
    Clear,
}

/// Toast のキュー状態機械。
///
/// `entries` は先頭が最古（追加順）。[`Toaster::max`] で有界。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toaster {
    entries: Vec<ToastEntry>,
    max: usize,
    placement: ToastPlacement,
}

impl Default for Toaster {
    fn default() -> Self {
        Self::new(DEFAULT_MAX, ToastPlacement::default())
    }
}

impl Toaster {
    /// `data-hydrate-ids` 属性名のフィールド部分。
    pub const FIELD_IDS: &'static str = "ids";
    /// `data-hydrate-statuses` 属性名のフィールド部分。
    pub const FIELD_STATUSES: &'static str = "statuses";
    /// `data-hydrate-titles` 属性名のフィールド部分。
    pub const FIELD_TITLES: &'static str = "titles";
    /// `data-hydrate-descriptions` 属性名のフィールド部分。
    pub const FIELD_DESCRIPTIONS: &'static str = "descriptions";
    /// `data-hydrate-max` 属性名のフィールド部分。
    pub const FIELD_MAX: &'static str = "max";
    /// `data-hydrate-placement` 属性名のフィールド部分。
    pub const FIELD_PLACEMENT: &'static str = "placement";

    /// 最大表示件数 `max`・表示位置 `placement` を指定して空の Toaster を作る。
    #[must_use]
    pub fn new(max: usize, placement: ToastPlacement) -> Self {
        Self {
            entries: Vec::new(),
            max,
            placement,
        }
    }

    /// 現在のキュー内容（先頭が最古）。
    #[must_use]
    pub fn entries(&self) -> &[ToastEntry] {
        &self.entries
    }

    /// 表示位置。
    #[must_use]
    pub fn placement(&self) -> ToastPlacement {
        self.placement
    }

    /// 最大表示件数。
    #[must_use]
    pub fn max(&self) -> usize {
        self.max
    }

    /// 通知を末尾へ追加する（決定的な状態遷移、時刻・乱数を使わない）。
    ///
    /// 同一 `id` の既存エントリがあれば先に除去してから追加する（更新扱い）。
    /// 追加後に `entries.len() > max` となる場合は先頭（最古）から押し出す。
    pub fn push(&mut self, entry: ToastEntry) {
        self.entries.retain(|e| e.id != entry.id);
        self.entries.push(entry);
        while self.entries.len() > self.max {
            self.entries.remove(0);
        }
    }

    /// 指定した `id` の通知を除去する。不一致は no-op。
    pub fn dismiss(&mut self, id: &str) {
        self.entries.retain(|e| e.id != id);
    }

    /// 全通知を除去する。
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// [`group`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn group<'a>(
        &self,
        label: &'a str,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        group(self.placement, label, attrs, children)
    }
}

impl Component for Toaster {
    type Action = ToastAction;

    fn update(&mut self, action: ToastAction) {
        match action {
            ToastAction::Push(entry) => self.push(entry),
            ToastAction::Dismiss { id } => self.dismiss(&id),
            ToastAction::Clear => self.clear(),
        }
    }

    /// 共通契約（`data-scope`/`data-part`・hydration ルート）のみを表す最小
    /// 正準ビュー（group > 各エントリの root > title + description +
    /// close_trigger）。[`Avatar::view`](crate::avatar::Avatar::view) と同じ
    /// 位置付けであり、実際の UI 構築は §パーツ関数群を呼び出し側が組み合わせる。
    fn view(&self) -> Node {
        let children = self
            .entries
            .iter()
            .map(|entry| {
                root(
                    entry.status,
                    Vec::new(),
                    vec![
                        title(Vec::new(), vec![fandhe_frontend_core::text(&entry.title)]),
                        description(
                            Vec::new(),
                            vec![fandhe_frontend_core::text(&entry.description)],
                        ),
                        close_trigger(Vec::new(), Vec::new()),
                    ],
                )
            })
            .collect();
        self.group("", Vec::new(), children)
    }

    /// クライアント由来の文字列 dispatch は `"dismiss"`（payload = `id`）・
    /// `"clear"` のみを受理する（本モジュール冒頭「スコープ外」節参照。`"push"`
    /// は payload の構造化エンコードを要するため文字列 dispatch からは到達しない）。
    fn decode_action(name: &str, payload: &str) -> Option<ToastAction> {
        match name {
            "dismiss" => Some(ToastAction::Dismiss {
                id: payload.to_string(),
            }),
            "clear" => Some(ToastAction::Clear),
            _ => None,
        }
    }
}

impl Hydrate for Toaster {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let ids: Vec<String> = self.entries.iter().map(|e| e.id.clone()).collect();
        let statuses: Vec<String> = self
            .entries
            .iter()
            .map(|e| e.status.as_data_status().to_string())
            .collect();
        let titles: Vec<String> = self.entries.iter().map(|e| e.title.clone()).collect();
        let descriptions: Vec<String> =
            self.entries.iter().map(|e| e.description.clone()).collect();

        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IDS),
                encode_list(&ids),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATUSES),
                encode_list(&statuses),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_TITLES),
                encode_list(&titles),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_DESCRIPTIONS),
                encode_list(&descriptions),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX),
                self.max.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PLACEMENT),
                self.placement.as_data_placement().to_string(),
            ),
        ]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let attr_name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == attr_name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(attr_name))
        };

        let ids = decode_list(find(Self::FIELD_IDS)?);
        let statuses_raw = decode_list(find(Self::FIELD_STATUSES)?);
        let titles = decode_list(find(Self::FIELD_TITLES)?);
        let descriptions = decode_list(find(Self::FIELD_DESCRIPTIONS)?);

        let len = ids.len();
        if statuses_raw.len() != len || titles.len() != len || descriptions.len() != len {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IDS),
                reason: "ids/statuses/titles/descriptions must have equal length".to_string(),
            });
        }

        // id の重複はキュー不変条件（Toaster::push が同一 id を単一エントリへ
        // 収束させる契約）に反するため、改ざん入力として fail-closed に拒否する。
        for (i, id) in ids.iter().enumerate() {
            if ids[..i].contains(id) {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IDS),
                    reason: "duplicate id in queue".to_string(),
                });
            }
        }

        let mut statuses = Vec::with_capacity(len);
        for raw in &statuses_raw {
            statuses.push(ToastStatus::from_data_status(raw).ok_or_else(|| {
                HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATUSES),
                    reason: "expected \"info\", \"success\", \"warning\", or \"error\"".to_string(),
                }
            })?);
        }

        let max_raw = find(Self::FIELD_MAX)?;
        let max: usize = max_raw.parse().map_err(|_| HydrateError::InvalidValue {
            attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_MAX),
            reason: "expected a non-negative decimal integer".to_string(),
        })?;

        if len > max {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IDS),
                reason: "entry count exceeds max".to_string(),
            });
        }

        let placement_raw = find(Self::FIELD_PLACEMENT)?;
        let placement = ToastPlacement::from_data_placement(placement_raw).ok_or_else(|| {
            HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PLACEMENT),
                reason: "unknown placement value".to_string(),
            }
        })?;

        let entries = ids
            .into_iter()
            .zip(statuses)
            .zip(titles)
            .zip(descriptions)
            .map(|(((id, status), title), description)| ToastEntry {
                id,
                status,
                title,
                description,
            })
            .collect();

        Ok(Self {
            entries,
            max,
            placement,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn entry(id: &str, status: ToastStatus, title: &str, description: &str) -> ToastEntry {
        ToastEntry {
            id: id.to_string(),
            status,
            title: title.to_string(),
            description: description.to_string(),
        }
    }

    // --- anatomy パーツの data-scope/data-part 出力 ---

    #[test]
    fn group_outputs_region_role_label_and_placement() {
        let html = render(&group(
            ToastPlacement::TopStart,
            "Notifications",
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toast""#));
        assert!(html.contains(r#"data-part="group""#));
        assert!(html.contains(r#"role="region""#));
        assert!(html.contains(r#"aria-label="Notifications""#));
        assert!(html.contains(r#"data-placement="top-start""#));
    }

    #[test]
    fn root_outputs_status_role_atomic_and_type() {
        let html = render(&root(ToastStatus::Success, vec![], vec![]));
        assert!(html.contains(r#"data-scope="toast""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"role="status""#));
        assert!(html.contains(r#"aria-atomic="true""#));
        assert!(html.contains(r#"aria-live="polite""#));
        assert!(html.contains(r#"data-type="success""#));
    }

    #[test]
    fn root_error_status_uses_assertive_aria_live() {
        let html = render(&root(ToastStatus::Error, vec![], vec![]));
        assert!(html.contains(r#"aria-live="assertive""#));
        assert!(html.contains(r#"data-type="error""#));
    }

    #[test]
    fn root_non_error_statuses_use_polite_aria_live() {
        for status in [
            ToastStatus::Info,
            ToastStatus::Success,
            ToastStatus::Warning,
        ] {
            let html = render(&root(status, vec![], vec![]));
            assert!(html.contains(r#"aria-live="polite""#));
        }
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&title(vec![], vec![]))
            .starts_with(r#"<div data-scope="toast" data-part="title""#));
        assert!(render(&description(vec![], vec![]))
            .starts_with(r#"<div data-scope="toast" data-part="description""#));
        assert!(render(&action_trigger(vec![], vec![]))
            .starts_with(r#"<button data-scope="toast" data-part="action-trigger" type="button""#));
        assert!(render(&close_trigger(vec![], vec![]))
            .starts_with(r#"<button data-scope="toast" data-part="close-trigger" type="button""#));
    }

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            ToastStatus::Info,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="toast""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- ToastStatus/ToastPlacement 変換 ---

    #[test]
    fn status_round_trips_for_known_values() {
        for status in [
            ToastStatus::Info,
            ToastStatus::Success,
            ToastStatus::Warning,
            ToastStatus::Error,
        ] {
            assert_eq!(
                ToastStatus::from_data_status(status.as_data_status()),
                Some(status)
            );
        }
    }

    #[test]
    fn status_rejects_unknown_values() {
        for bogus in ["INFO", "", "<script>alert(1)</script>"] {
            assert_eq!(ToastStatus::from_data_status(bogus), None);
        }
    }

    #[test]
    fn placement_round_trips_for_known_values() {
        for placement in [
            ToastPlacement::TopStart,
            ToastPlacement::Top,
            ToastPlacement::TopEnd,
            ToastPlacement::BottomStart,
            ToastPlacement::Bottom,
            ToastPlacement::BottomEnd,
        ] {
            assert_eq!(
                ToastPlacement::from_data_placement(placement.as_data_placement()),
                Some(placement)
            );
        }
    }

    #[test]
    fn placement_rejects_unknown_values() {
        for bogus in ["TOP", "", "<script>"] {
            assert_eq!(ToastPlacement::from_data_placement(bogus), None);
        }
    }

    #[test]
    fn default_status_is_info_and_placement_is_bottom_end() {
        assert_eq!(ToastStatus::default(), ToastStatus::Info);
        assert_eq!(ToastPlacement::default(), ToastPlacement::BottomEnd);
    }

    // --- Toaster: キュー決定性 ---

    #[test]
    fn toaster_default_uses_default_max_and_placement() {
        let t = Toaster::default();
        assert_eq!(t.max(), DEFAULT_MAX);
        assert_eq!(t.placement(), ToastPlacement::BottomEnd);
        assert!(t.entries().is_empty());
    }

    #[test]
    fn push_preserves_order() {
        let mut t = Toaster::new(10, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A", ""));
        t.push(entry("b", ToastStatus::Info, "B", ""));
        t.push(entry("c", ToastStatus::Info, "C", ""));
        let ids: Vec<&str> = t.entries().iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn push_beyond_max_evicts_oldest() {
        let mut t = Toaster::new(2, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A", ""));
        t.push(entry("b", ToastStatus::Info, "B", ""));
        t.push(entry("c", ToastStatus::Info, "C", ""));
        let ids: Vec<&str> = t.entries().iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["b", "c"]);
    }

    #[test]
    fn push_same_id_updates_in_place_moving_to_end() {
        let mut t = Toaster::new(10, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A1", ""));
        t.push(entry("b", ToastStatus::Info, "B", ""));
        t.push(entry("a", ToastStatus::Error, "A2", "updated"));
        let ids: Vec<&str> = t.entries().iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["b", "a"]);
        assert_eq!(t.entries()[1].status, ToastStatus::Error);
        assert_eq!(t.entries()[1].title, "A2");
    }

    #[test]
    fn dismiss_removes_matching_id_only() {
        let mut t = Toaster::new(10, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A", ""));
        t.push(entry("b", ToastStatus::Info, "B", ""));
        t.dismiss("a");
        let ids: Vec<&str> = t.entries().iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["b"]);
    }

    #[test]
    fn dismiss_unknown_id_is_no_op() {
        let mut t = Toaster::new(10, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A", ""));
        t.dismiss("no-such-id");
        assert_eq!(t.entries().len(), 1);
    }

    #[test]
    fn clear_removes_all_entries() {
        let mut t = Toaster::new(10, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A", ""));
        t.push(entry("b", ToastStatus::Info, "B", ""));
        t.clear();
        assert!(t.entries().is_empty());
    }

    #[test]
    fn same_operation_sequence_yields_same_state() {
        let mut t1 = Toaster::new(3, ToastPlacement::Top);
        let mut t2 = Toaster::new(3, ToastPlacement::Top);
        for t in [&mut t1, &mut t2] {
            t.push(entry("a", ToastStatus::Info, "A", ""));
            t.push(entry("b", ToastStatus::Warning, "B", "b-desc"));
            t.dismiss("a");
            t.push(entry("c", ToastStatus::Error, "C", ""));
        }
        assert_eq!(t1, t2);
    }

    // --- Toaster: dispatch 契約 ---

    #[test]
    fn dispatch_dismiss_removes_entry_by_payload_id() {
        let mut t = Toaster::new(10, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A", ""));
        assert!(dispatch(&mut t, "dismiss", "a"));
        assert!(t.entries().is_empty());
    }

    #[test]
    fn dispatch_clear_removes_all_entries() {
        let mut t = Toaster::new(10, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A", ""));
        t.push(entry("b", ToastStatus::Info, "B", ""));
        assert!(dispatch(&mut t, "clear", ""));
        assert!(t.entries().is_empty());
    }

    #[test]
    fn dispatch_ignores_unknown_action_including_push() {
        let mut t = Toaster::new(10, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "A", ""));
        assert!(!dispatch(&mut t, "push", "anything"));
        assert!(!dispatch(&mut t, "no_such_action", "x"));
        assert_eq!(t.entries().len(), 1);
    }

    // --- Toaster: SSR 状態なし初期描画 ---

    #[test]
    fn toaster_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Toaster::default().view());
        assert!(!rendered.contains("data-hydrate-"));
        assert!(rendered.contains(r#"data-scope="toast""#));
        assert!(rendered.contains(r#"data-part="group""#));
    }

    #[test]
    fn toaster_view_root_is_element_node() {
        assert!(matches!(Toaster::default().view(), Node::Element { .. }));
    }

    // --- Toaster: hydration 経路 ---

    #[test]
    fn hydration_round_trips_with_multiple_entries() {
        let mut t = Toaster::new(5, ToastPlacement::TopEnd);
        t.push(entry("a", ToastStatus::Info, "Title A", "Desc A"));
        t.push(entry("b", ToastStatus::Error, "Title B", "Desc B"));

        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains("data-hydrate-ids="));
        assert!(rendered.contains(r#"data-hydrate-max="5""#));
        assert!(rendered.contains(r#"data-hydrate-placement="top-end""#));

        let restored = Toaster::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn hydration_round_trips_with_empty_queue() {
        let t = Toaster::new(5, ToastPlacement::Bottom);
        let restored = Toaster::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn hydration_round_trips_survive_control_chars_in_title() {
        let mut t = Toaster::new(5, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "sep:\u{1f}here\\back", ""));
        let restored = Toaster::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn hydration_round_trips_survive_empty_title_and_description() {
        let mut t = Toaster::new(5, ToastPlacement::Bottom);
        t.push(entry("a", ToastStatus::Info, "", ""));
        let restored = Toaster::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Toaster::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-ids".to_string())
        );
    }

    #[test]
    fn from_hydration_attrs_rejects_length_mismatch() {
        let attrs = vec![
            (
                "data-hydrate-ids".to_string(),
                encode_list(&["a".to_string()]),
            ),
            ("data-hydrate-statuses".to_string(), encode_list(&[])),
            (
                "data-hydrate-titles".to_string(),
                encode_list(&["A".to_string()]),
            ),
            (
                "data-hydrate-descriptions".to_string(),
                encode_list(&["".to_string()]),
            ),
            ("data-hydrate-max".to_string(), "5".to_string()),
            ("data-hydrate-placement".to_string(), "bottom".to_string()),
        ];
        let err = Toaster::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_unknown_status() {
        let attrs = vec![
            (
                "data-hydrate-ids".to_string(),
                encode_list(&["a".to_string()]),
            ),
            (
                "data-hydrate-statuses".to_string(),
                encode_list(&["bogus".to_string()]),
            ),
            (
                "data-hydrate-titles".to_string(),
                encode_list(&["A".to_string()]),
            ),
            (
                "data-hydrate-descriptions".to_string(),
                encode_list(&["".to_string()]),
            ),
            ("data-hydrate-max".to_string(), "5".to_string()),
            ("data-hydrate-placement".to_string(), "bottom".to_string()),
        ];
        let err = Toaster::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_unknown_placement() {
        let attrs = vec![
            ("data-hydrate-ids".to_string(), encode_list(&[])),
            ("data-hydrate-statuses".to_string(), encode_list(&[])),
            ("data-hydrate-titles".to_string(), encode_list(&[])),
            ("data-hydrate-descriptions".to_string(), encode_list(&[])),
            ("data-hydrate-max".to_string(), "5".to_string()),
            ("data-hydrate-placement".to_string(), "bogus".to_string()),
        ];
        let err = Toaster::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_invalid_max() {
        let attrs = vec![
            ("data-hydrate-ids".to_string(), encode_list(&[])),
            ("data-hydrate-statuses".to_string(), encode_list(&[])),
            ("data-hydrate-titles".to_string(), encode_list(&[])),
            ("data-hydrate-descriptions".to_string(), encode_list(&[])),
            ("data-hydrate-max".to_string(), "not-a-number".to_string()),
            ("data-hydrate-placement".to_string(), "bottom".to_string()),
        ];
        let err = Toaster::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_duplicate_id() {
        let attrs = vec![
            (
                "data-hydrate-ids".to_string(),
                encode_list(&["a".to_string(), "a".to_string()]),
            ),
            (
                "data-hydrate-statuses".to_string(),
                encode_list(&["info".to_string(), "info".to_string()]),
            ),
            (
                "data-hydrate-titles".to_string(),
                encode_list(&["A".to_string(), "A2".to_string()]),
            ),
            (
                "data-hydrate-descriptions".to_string(),
                encode_list(&["".to_string(), "".to_string()]),
            ),
            ("data-hydrate-max".to_string(), "5".to_string()),
            ("data-hydrate-placement".to_string(), "bottom".to_string()),
        ];
        let err = Toaster::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn from_hydration_attrs_rejects_entry_count_exceeding_max() {
        let attrs = vec![
            (
                "data-hydrate-ids".to_string(),
                encode_list(&["a".to_string(), "b".to_string()]),
            ),
            (
                "data-hydrate-statuses".to_string(),
                encode_list(&["info".to_string(), "info".to_string()]),
            ),
            (
                "data-hydrate-titles".to_string(),
                encode_list(&["A".to_string(), "B".to_string()]),
            ),
            (
                "data-hydrate-descriptions".to_string(),
                encode_list(&["".to_string(), "".to_string()]),
            ),
            ("data-hydrate-max".to_string(), "1".to_string()),
            ("data-hydrate-placement".to_string(), "bottom".to_string()),
        ];
        let err = Toaster::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: title/description/id/呼び出し側 attrs にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn group_label_payload_is_escaped_on_render() {
        let html = render(&group(
            ToastPlacement::Bottom,
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            ToastStatus::Info,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn title_children_text_is_escaped_on_render() {
        let html = render(&title(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn toaster_view_escapes_entry_title_and_description() {
        let mut t = Toaster::new(5, ToastPlacement::Bottom);
        t.push(entry(
            "a",
            ToastStatus::Info,
            "<script>alert(1)</script>",
            "<img src=x onerror=alert(1)>",
        ));
        let html = render(&t.view());
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("<img src=x onerror"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn hydration_xss_payload_in_status_is_rejected_not_rendered() {
        // data-hydrate-statuses はサーバーが as_data_status() から生成する固定
        // 語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が未知値を
        // 拒否することを固定する。
        let attrs = vec![
            (
                "data-hydrate-ids".to_string(),
                encode_list(&["a".to_string()]),
            ),
            (
                "data-hydrate-statuses".to_string(),
                encode_list(&["<script>alert(1)</script>".to_string()]),
            ),
            (
                "data-hydrate-titles".to_string(),
                encode_list(&["A".to_string()]),
            ),
            (
                "data-hydrate-descriptions".to_string(),
                encode_list(&["".to_string()]),
            ),
            ("data-hydrate-max".to_string(), "5".to_string()),
            ("data-hydrate-placement".to_string(), "bottom".to_string()),
        ];
        let err = Toaster::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
