//! Avatar（プロフィール画像・フォールバック表示）headless コンポーネント
//! （イシュー #543、親 #542）。
//!
//! ark-ui の Avatar
//!（`.claude/skills/ark-ui/references/components/display/avatar.md`）を
//! 参考に、Root / Image / Fallback の 3 anatomy パーツと、画像読み込み
//! ステータス（`"loading"`/`"loaded"`/`"error"`）を管理する状態機械
//! [`Avatar`] を提供する。
//!
//! # `data-state` 語彙について（[`crate::state`] を使わない理由）
//!
//! Avatar の状態は画像読み込みステータス 3 値（[`ImageStatus`]）であり、
//! [`crate::state::Disclosure`]（開閉 2 値）にも [`crate::state::SingleSelect`]
//! （選択インデックス）にも意味的に写像できない。[`crate::switch::Switch`]
//! と同様、本モジュールは [`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] を直接実装し、Phase 1 が確立した
//! dispatch 契約（未知アクション no-op）・fail-closed hydration という
//! **統合様式**にのみ準拠する。
//!
//! なお ark-ui 準拠で `data-state`（`"visible"`/`"hidden"`）は [`image`]/
//! [`fallback`] のみに付与し、[`root`] には付与しない（ark-ui リファレンス
//! 「Notes」節を参照。`data-state` は「どちらが表示中か」を表す
//! Image/Fallback 固有の情報であり、Root の関心事ではないため）。
//! この `"visible"`/`"hidden"` 語彙は [`ImageStatus`] の
//! `"loading"`/`"loaded"`/`"error"`（hydration 語彙）とは別物であり、
//! 混同しない（[`ImageStatus::is_image_visible`] が両者を接続する唯一の
//! 変換点）。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`image`]/[`fallback`]、純粋関数で
//! 完結）を直接呼んで組み立てる。CSR/hydration は [`Avatar`] を経由し、
//! dispatch（`"loaded"`/`"error"`/`"reset"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui`（#546〜）が本モジュールを呼んでスタイル済み
//! Avatar を組み立てる想定である。
//!
//! # ARIA について
//!
//! Avatar は WAI-ARIA の専用パターンを持たない表示系コンポーネントであり、
//! ark-ui / Zag.js も追加の `role`/`aria-*` を付与しない。[`image`] の
//! `alt` 引数を必須にすることが実質的なアクセシビリティ担保である
//! （呼び出し側が空文字列を渡すことは可能だが、その場合は意図的な装飾画像
//! 扱いとして呼び出し側の責務とする）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`src`/`alt`/`hidden`）はすべて `&'static str` リテラル
//!   または固定スロットであり、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::data_attrs`] の既存不変条件をそのまま
//!   継承する）。
//! - 動的値（`src`/`alt`/呼び出し側 `attrs`/`children` テキスト）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `img` の `src` に対する URL スキーム検証（`javascript:` 等）は本
//!   headless 層では行わない（既定エスケープが属性破りを防ぎ、現代の
//!   ブラウザは `img src` の `javascript:` を実行しない）。URL の妥当性検証が
//!   必要な場合はアプリ側の責務とする。
//! - hydration 属性（`data-hydrate-status`）はクライアント側で改ざんされ
//!   うる入力として扱う。[`Avatar`] の
//!   [`fandhe_frontend_interactive::Hydrate`] 実装は panic せず
//!   `HydrateError` を返す。
//!
//! # スコープ外
//!
//! - クライアント側での `load`/`error` イベント検知と dispatch 発火の
//!   JS/wasm グルー（wasm 層の後続スコープ）
//! - `onStatusChange` コールバック・`asChild`・`ids` オプション（ark-ui 固有
//!   機能）
//! - Progress は別イシュー #544

use crate::anatomy::{anatomy, Anatomy};
use crate::data_attrs::data_state;
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Avatar の anatomy（`data-scope="avatar"`）。
const ANATOMY: Anatomy = anatomy("avatar");

/// [`image`]/[`fallback`] の `data-state` 属性値 "visible"。
const DATA_STATE_VISIBLE: &str = "visible";
/// [`image`]/[`fallback`] の `data-state` 属性値 "hidden"。
const DATA_STATE_HIDDEN: &str = "hidden";

/// 画像読み込みステータス（ark-ui 準拠の 3 値）。
///
/// [`image`]/[`fallback`] の `data-state`（`"visible"`/`"hidden"`）と
/// `data-hydrate-status` の両方の元になる、本モジュールで唯一の状態表現。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageStatus {
    /// 読み込み中（初期値。JS なし環境ではこの状態のまま留まる）。
    #[default]
    Loading,
    /// 読み込み成功。
    Loaded,
    /// 読み込み失敗。
    Error,
}

impl ImageStatus {
    /// `data-hydrate-status` 属性値文字列（`"loading"`/`"loaded"`/`"error"`）。
    #[must_use]
    pub const fn as_data_status(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Loaded => "loaded",
            Self::Error => "error",
        }
    }

    /// [`ImageStatus::as_data_status`] の逆変換。未知の値は `None`
    /// （安全側、呼び出し元が [`HydrateError::InvalidValue`] へ変換する）。
    #[must_use]
    pub fn from_data_status(s: &str) -> Option<Self> {
        match s {
            "loading" => Some(Self::Loading),
            "loaded" => Some(Self::Loaded),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// [`image`] が表示中（`data-state="visible"`）かどうか。
    ///
    /// `Loaded` のときのみ画像を表示し、`Loading`/`Error` は [`fallback`]
    /// を表示する安全側の既定（ark-ui の挙動と一致。JS なし環境では
    /// フォールバックが表示され続ける）。
    #[must_use]
    pub const fn is_image_visible(self) -> bool {
        matches!(self, Self::Loaded)
    }
}

/// `visible`/`hidden` から `data-state` 属性値文字列へ変換する内部ヘルパ。
const fn visibility_str(visible: bool) -> &'static str {
    if visible {
        DATA_STATE_VISIBLE
    } else {
        DATA_STATE_HIDDEN
    }
}

/// Root パーツ（`div`）。
///
/// ark-ui 準拠で `data-state` は付与しない（本モジュール冒頭の rustdoc
/// 「`data-state` 語彙について」参照。表示切り替えは [`image`]/[`fallback`]
/// 側の関心事）。
#[must_use]
pub fn root<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("root", "div", attrs, children)
}

/// Image パーツ（`img`）。
///
/// `src`/`alt` を必須引数とする（`alt` の必須化が実質的なアクセシビリティ
/// 担保、本モジュール冒頭の rustdoc「ARIA について」参照）。`status` に応じて
/// `data-state`（[`ImageStatus::is_image_visible`]）と、非表示時の `hidden`
/// 存在属性を出力し、JS なしの SSR でも表示制御を成立させる
/// （[`crate::collapsible::content`] の `hidden` パターンを踏襲）。子要素は
/// 持たない（`img` は空要素のため `children` 引数を公開しない）。
#[must_use]
pub fn image<'a>(
    status: ImageStatus,
    src: &'a str,
    alt: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
) -> Node {
    let visible = status.is_image_visible();
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("src", src),
        ("alt", alt),
        data_state(visibility_str(visible)),
    ];
    if !visible {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("image", "img", merged, Vec::new())
}

/// Fallback パーツ（`span`）。
///
/// [`image`] とは逆の可視性を持つ（`status` が `Loaded` のときのみ非表示）。
/// イニシャル・アイコン等の `children` は呼び出し側が組み立てる。
#[must_use]
pub fn fallback<'a>(
    status: ImageStatus,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let visible = !status.is_image_visible();
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(visibility_str(visible))];
    if !visible {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("fallback", "span", merged, children)
}

/// Avatar のアクション（WASM 境界の文字列 dispatch と
/// [`Avatar::decode_action`] で接続する）。payload は使用しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarAction {
    /// 画像読み込み成功へ遷移する。
    Loaded,
    /// 画像読み込み失敗へ遷移する。
    Error,
    /// 読み込み中へ戻す（`src` 差し替え時にクライアント側から発火する想定）。
    Reset,
}

/// Avatar の画像読み込みステータス状態機械。
///
/// `data-state`/`data-hydrate-status` と実際の読み込みステータスの整合を
/// 型レベルで保証する入口として、各パーツ関数（[`root`]/[`image`]/
/// [`fallback`]）へ `self.status()` を注入する利便メソッドを提供する。SSR
/// での自由関数直接利用（本型を経由しない構成）も引き続き可能。`Default` は
/// [`ImageStatus::Loading`]（SSR の状態なし初期描画に対応する既定値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Avatar {
    status: ImageStatus,
}

impl Avatar {
    /// `data-hydrate-status` 属性名のフィールド部分
    /// （`docs/api/hydration-state-format.md` の `<field>` 命名規約に従う）。
    pub const FIELD_STATUS: &'static str = "status";

    /// 指定した初期ステータスで Avatar を生成する。
    #[must_use]
    pub fn new(initial: ImageStatus) -> Self {
        Self { status: initial }
    }

    /// 現在の画像読み込みステータス。
    #[must_use]
    pub fn status(&self) -> ImageStatus {
        self.status
    }

    /// [`root`] へ委譲する利便メソッド（現在の状態は使わない。ark-ui 準拠で
    /// Root は `data-state` を持たないため引数として不要）。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(attrs, children)
    }

    /// [`image`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn image<'a>(&self, src: &'a str, alt: &'a str, attrs: Vec<(&'a str, &'a str)>) -> Node {
        image(self.status, src, alt, attrs)
    }

    /// [`fallback`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn fallback<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        fallback(self.status, attrs, children)
    }
}

impl Component for Avatar {
    type Action = AvatarAction;

    fn update(&mut self, action: AvatarAction) {
        self.status = match action {
            AvatarAction::Loaded => ImageStatus::Loaded,
            AvatarAction::Error => ImageStatus::Error,
            AvatarAction::Reset => ImageStatus::Loading,
        };
    }

    /// 共通契約（`data-state` 整合・hydration ルート）のみを表す最小正準
    /// ビュー（root > image + fallback、`src`/`alt` は空文字列・children
    /// 空・呼び出し側 attrs なし）。[`Switch::view`](crate::switch::Switch::view)
    /// と同じ位置付けであり、公開 UI としての利用は想定しない（実際の UI
    /// 構築は §パーツ関数群を呼び出し側が組み合わせる）。
    fn view(&self) -> Node {
        self.root(
            Vec::new(),
            vec![
                image(self.status, "", "", Vec::new()),
                fallback(self.status, Vec::new(), Vec::new()),
            ],
        )
    }

    fn decode_action(name: &str, _payload: &str) -> Option<AvatarAction> {
        match name {
            "loaded" => Some(AvatarAction::Loaded),
            "error" => Some(AvatarAction::Error),
            "reset" => Some(AvatarAction::Reset),
            _ => None,
        }
    }
}

impl Hydrate for Avatar {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![(
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATUS),
            self.status.as_data_status().to_string(),
        )]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let attr_name = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATUS);
        let raw = attrs
            .iter()
            .find(|(k, _)| *k == attr_name)
            .map(|(_, v)| v.as_str())
            .ok_or_else(|| HydrateError::MissingAttr(attr_name.clone()))?;
        let status =
            ImageStatus::from_data_status(raw).ok_or_else(|| HydrateError::InvalidValue {
                attr: attr_name.clone(),
                reason: "expected \"loading\", \"loaded\", or \"error\"".to_string(),
            })?;
        Ok(Self { status })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part/data-state 出力 ---

    #[test]
    fn root_outputs_scope_and_part_but_no_data_state() {
        let html = render(&root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains("<div"));
        assert!(!html.contains("data-state"));
    }

    #[test]
    fn image_loaded_is_visible_and_not_hidden() {
        let html = render(&image(ImageStatus::Loaded, "/a.png", "avatar", vec![]));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="image""#));
        assert!(html.contains(r#"src="/a.png""#));
        assert!(html.contains(r#"alt="avatar""#));
        assert!(html.contains(r#"data-state="visible""#));
        assert!(!html.contains("hidden"));
    }

    #[test]
    fn image_loading_is_hidden() {
        let html = render(&image(ImageStatus::Loading, "/a.png", "avatar", vec![]));
        assert!(html.contains(r#"data-state="hidden""#));
        assert!(html.contains(r#"hidden="""#));
    }

    #[test]
    fn image_error_is_hidden() {
        let html = render(&image(ImageStatus::Error, "/a.png", "avatar", vec![]));
        assert!(html.contains(r#"data-state="hidden""#));
        assert!(html.contains(r#"hidden="""#));
    }

    #[test]
    fn fallback_loaded_is_hidden() {
        let html = render(&fallback(ImageStatus::Loaded, vec![], vec![text("NM")]));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="fallback""#));
        assert!(html.contains(r#"data-state="hidden""#));
        assert!(html.contains(r#"hidden="""#));
        assert!(html.contains("NM"));
    }

    #[test]
    fn fallback_loading_and_error_are_visible() {
        for status in [ImageStatus::Loading, ImageStatus::Error] {
            let html = render(&fallback(status, vec![], vec![]));
            assert!(html.contains(r#"data-state="visible""#));
            assert!(!html.contains("hidden"));
        }
    }

    // --- ImageStatus 変換 ---

    #[test]
    fn data_status_round_trips_for_known_values() {
        for status in [
            ImageStatus::Loading,
            ImageStatus::Loaded,
            ImageStatus::Error,
        ] {
            let s = status.as_data_status();
            assert_eq!(ImageStatus::from_data_status(s), Some(status));
        }
    }

    #[test]
    fn from_data_status_rejects_unknown_values() {
        for bogus in ["LOADED", "", "<script>alert(1)</script>", "loadedx"] {
            assert_eq!(ImageStatus::from_data_status(bogus), None);
        }
    }

    #[test]
    fn default_status_is_loading() {
        assert_eq!(ImageStatus::default(), ImageStatus::Loading);
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側 attrs の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- Avatar: dispatch 統合 ---

    #[test]
    fn avatar_default_is_loading() {
        assert_eq!(Avatar::default().status(), ImageStatus::Loading);
    }

    #[test]
    fn avatar_dispatch_loaded_error_reset() {
        let mut a = Avatar::default();
        assert!(dispatch(&mut a, "loaded", ""));
        assert_eq!(a.status(), ImageStatus::Loaded);
        assert!(render(&a.image("/a.png", "avatar", vec![])).contains(r#"data-state="visible""#));
        assert!(render(&a.fallback(vec![], vec![])).contains(r#"data-state="hidden""#));

        assert!(dispatch(&mut a, "error", ""));
        assert_eq!(a.status(), ImageStatus::Error);
        assert!(render(&a.image("/a.png", "avatar", vec![])).contains(r#"data-state="hidden""#));
        assert!(render(&a.fallback(vec![], vec![])).contains(r#"data-state="visible""#));

        assert!(dispatch(&mut a, "reset", ""));
        assert_eq!(a.status(), ImageStatus::Loading);
    }

    #[test]
    fn avatar_dispatch_ignores_unknown_action() {
        let mut a = Avatar::new(ImageStatus::Loaded);
        assert!(!dispatch(&mut a, "no_such_action", "x"));
        assert_eq!(a.status(), ImageStatus::Loaded);
    }

    // --- Avatar: SSR 状態なし初期描画 ---

    #[test]
    fn avatar_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Avatar::default().view());
        assert!(!rendered.contains("data-hydrate-"));
        assert!(rendered.contains(r#"data-state="hidden""#));
    }

    #[test]
    fn avatar_view_root_is_element_node() {
        assert!(matches!(Avatar::default().view(), Node::Element { .. }));
    }

    // --- Avatar: hydration 経路 ---

    #[test]
    fn avatar_hydration_round_trip() {
        let a = Avatar::new(ImageStatus::Loaded);
        let rendered = render(&render_for_hydration(&a));
        assert!(rendered.contains(r#"data-hydrate-status="loaded""#));

        let restored = Avatar::from_hydration_attrs(&a.hydration_attrs()).unwrap();
        assert_eq!(restored, a);
    }

    #[test]
    fn avatar_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Avatar::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-status".to_string())
        );
    }

    #[test]
    fn avatar_from_hydration_attrs_invalid_value_does_not_panic() {
        for bogus in ["LOADED", "<script>alert(1)</script>", ""] {
            let attrs = vec![("data-hydrate-status".to_string(), bogus.to_string())];
            let err = Avatar::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: src/alt/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn image_src_alt_payload_is_escaped_on_render() {
        let html = render(&image(
            ImageStatus::Loaded,
            ATTR_BREAK_PAYLOAD,
            ATTR_BREAK_PAYLOAD,
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn fallback_children_text_is_escaped_on_render() {
        let html = render(&fallback(
            ImageStatus::Loading,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn avatar_xss_payload_in_hydration_status_is_rejected_not_rendered() {
        // data-hydrate-status はサーバーが as_data_status() から生成する固定
        // 語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が未知値を
        // 拒否することを Avatar 経由でも固定する。
        let attrs = vec![(
            "data-hydrate-status".to_string(),
            "<script>alert(1)</script>".to_string(),
        )];
        let err = Avatar::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
