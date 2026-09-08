//! Item（shadcn/ui `Item` 相当）headless コンポーネント（イシュー #2065、
//! 親 #2064、Phase 4 #2057、祖父トラッキング参照軸 #2001）。
//!
//! `docs/design/component-coverage-map.md`（イシュー #937/#2004）の shadcn/ui
//! 参照軸にのみ存在し、他 3 参照軸（ark-ui / chakra-ui / Radix）に対応する
//! 部品がない汎用リスト行部品を埋める。media（アイコン・画像・アバター）+
//! title/description + actions からなる 1 行のレイアウトコンテナを表現する
//! [`root`] / [`media`] / [`content`] / [`title`] / [`description`] /
//! [`actions`] / [`header`] / [`footer`] / [`group`] / [`separator`] の
//! 10 anatomy パーツを提供する。[`mod@crate::button_group`]/
//! [`mod@crate::input_group`] と同型で状態機械（[`crate::state`]）を持たない
//! （すべて SSR 時点で決まる静的な props のみで完結する自由関数）。
//!
//! # `group` は `role="group"`（shadcn の `role="list"` からの意図的差分）
//!
//! shadcn/ui の `ItemGroup` は `role="list"` だが、WAI-ARIA の `list` ロールは
//! `listitem` 子を必須とする（`aria-required-children`）一方、`a[href]` は
//! ARIA in HTML の許容ロール外で `listitem` ロールを持てない。本モジュールは
//! [`root`] を `a` として描画できる（下記「`root` を `a` として描画する経路」
//! 参照）ため、`list`/`listitem` 対を成立させられない。よって [`group`] は
//! [`crate::button_group::root`] と同じ `role="group"` + 任意 `aria-label` と
//! し、リスト意味論が必要な利用者は `ul`/`li` の合成や [`mod@crate::nav_list`]
//! を使う想定とする。
//!
//! # `separator` は `group` 内専用の水平固定パーツ
//!
//! イシュー題名の列挙（root/media/content/title/description/actions/header/
//! footer/group の 9 パーツ）に加え、親 #2064 の anatomy 定義（「`group`
//! （複数 item の縦並び。`separator` を挟める）」）と shadcn/ui の
//! `ItemSeparator` に基づき、本モジュールは [`separator`] を 10 個目の
//! パーツとして提供する。[`group`] は常に縦並びのコンテナであるため、
//! [`separator`] は向き引数を取らず水平（`aria-orientation="horizontal"`）
//! に固定する（[`crate::button_group::separator`] の「グループの向きと
//! 直交」規則とは異なり、`group` 自体が向きの選択肢を持たないための単純化）。
//!
//! # `root` を `a` として描画する経路（link / link-overlay 方針の適用）
//!
//! [`ItemRootProps::href`] が `Some` のとき [`root`] は `div` ではなく `a` を
//! 描画する（[`mod@crate::link`]/[`mod@crate::link_overlay`] と同型）。
//!
//! - `href` は出力属性列の先頭へ固定付与し、`drop_reserved` で呼び出し側
//!   `attrs` からの同名なりすまし（大文字小文字混在を含む）を除去する
//!   （[`crate::link_overlay::overlay`] と同型、イシュー #1650）。
//! - URL スキーム検証は独自実装しない。`javascript:`/`data:`/`vbscript:` 等は
//!   [`fandhe_frontend_core::render`] の許可スキーム deny-by-default が属性
//!   ごと拒否する（[`mod@crate::link`] と同じ整理）。
//! - [`ItemRootProps::external`] が `true` のとき `target="_blank"` +
//!   `rel="noopener noreferrer"` を**不可分に**付与する（reverse tabnabbing
//!   対策、[`mod@crate::link`] と同じ判断。片方のみを付与できる API は公開
//!   しない）。`href` が `None` のときは `external` を無視する（`div` に
//!   `target`/`rel` を出さない）。
//! - `role` は付与しない（`a` の暗黙 `link` ロールに委ねる。`div` のときも
//!   `listitem` を付けない、上記「`group` は `role="group"`」参照）。
//! - hover / focus-visible の見た目は CSS の `:hover`/`:focus-visible`
//!   疑似クラスで `fandhe-frontend-pre-styled-ui` 側（#2066、本イシューの
//!   スコープ外）が表現する。本モジュールは `data-focus-visible` 等を出力
//!   しない（同フラグはネイティブ疑似クラスで表現できない hidden-input
//!   パターン専用であり、本部品には該当しない）。
//!
//! # `data-variant`/`data-size`/`media` の `data-variant`
//!
//! [`root`] は [`ItemVariant`]（`default`/`outline`/`muted`）+
//! [`ItemSize`]（`default`/`sm`）を `data-variant`/`data-size` として固定
//! 出力する（値語彙は shadcn/ui `Item` の `variant`/`size` props に対応、
//! [`crate::input_group::InputGroupAlign`] と同型の列挙型 + `as_str()`
//! パターン）。[`media`] は独立した [`ItemMediaVariant`]（`default`/`icon`/
//! `image`）を持ち、装飾要素の `aria-hidden` 付与は呼び出し側の判断とする
//! （shadcn も `ItemMedia` 自体には付与しない）。
//!
//! # 呼び出し文脈
//!
//! 上層の [`crate::anatomy::Anatomy`]・[`crate::aria`]・[`crate::data_attrs`]
//! へ薄く委譲するのみ。`fandhe-frontend-pre-styled-ui` が本モジュールを呼んで
//! スタイル済み Item（recipe・golden）を組み立てる想定（#2066、本イシューの
//! スコープ外）。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`role`/`aria-*`/`data-variant`/`data-size`/`href`/`target`/
//!   `rel`）はすべて `&'static str` リテラルで固定しており、動的値が
//!   属性名スロットへ混入する経路はない。
//! - 動的値（`href`/`label`/呼び出し側 `attrs`/`children`）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する
//!   （REQ-1）。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - **呼び出し側による予約キーのなりすまし除去**: `drop_reserved` が
//!   パート別の `*_RESERVED` 定数（ASCII 大文字小文字無視の完全一致）を
//!   使い、呼び出し側 `attrs` から本モジュールが固定付与する属性名を除去
//!   してから固定値を合成する（`crate::button_group::drop_reserved` と
//!   同型）。`data-scope`/`data-part` の偽装は
//!   [`crate::anatomy::Anatomy::part`] が別途除去する。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `fandhe-frontend-pre-styled-ui` 側の recipe・golden テスト
//!   （`tests/item_css.rs`）・`site/themes/item.md`・Themes ページ・
//!   `docs/design/component-coverage-map.md` の「実装済み」化は #2066。
//! - wasm-full 側の配線は不要（静的部品。キーボード操作はネイティブ
//!   `a[href]`/子 `button` の Tab 順序のみで完結する）。
//! - `asChild` 相当機能（`docs/policy/intentional-non-adoption.md` §3.25
//!   Slot 行は保留継続）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_label, aria_orientation, role};
use crate::data_attrs::{data_orientation, Orientation};
use fandhe_frontend_core::Node;

/// Item の anatomy（`data-scope="item"`）。
const ANATOMY: Anatomy = anatomy("item");

/// [`root`] が固定付与する予約キー。
const ROOT_RESERVED: &[&str] = &["href", "target", "rel", "data-variant", "data-size"];

/// [`media`] が固定付与する予約キー。
const MEDIA_RESERVED: &[&str] = &["data-variant"];

/// [`content`]/[`title`]/[`description`]/[`actions`]/[`header`]/[`footer`]
/// は固定属性を持たないが、将来の追加に備え対称性のため空の予約キー定数を
/// 用意する（`drop_reserved` は空スライスでも安全に動作する、
/// `crate::button_group::TEXT_RESERVED` と同型）。
const NO_RESERVED: &[&str] = &[];

/// [`group`] が固定付与する予約キー。
const GROUP_RESERVED: &[&str] = &["role", "aria-label"];

/// [`separator`] が固定付与する予約キー。
const SEPARATOR_RESERVED: &[&str] = &["role", "aria-orientation", "data-orientation"];

/// 呼び出し側 `attrs` から予約キー（本モジュールが固定付与する属性名）を
/// 除去する（ASCII 大文字小文字無視の完全一致）。`fandhe_frontend_core::el`
/// は属性の重複除去をしないため、これを経由しない呼び出しは状態属性の
/// なりすましを許してしまう（`crate::button_group::drop_reserved`/
/// `crate::link_overlay::drop_reserved` と同型のパターン）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// [`root`] の見た目バリアント（shadcn/ui `Item` の `variant` prop に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemVariant {
    /// 既定の見た目。
    Default,
    /// 枠線のみの見た目。
    Outline,
    /// 控えめな背景色の見た目。
    Muted,
}

impl ItemVariant {
    /// `data-variant` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Outline => "outline",
            Self::Muted => "muted",
        }
    }
}

impl Default for ItemVariant {
    /// shadcn/ui の既定（`default`）と一致させる。
    fn default() -> Self {
        Self::Default
    }
}

/// [`root`] のサイズバリアント（shadcn/ui `Item` の `size` prop に対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSize {
    /// 既定のサイズ。
    Default,
    /// 小さいサイズ。
    Sm,
}

impl ItemSize {
    /// `data-size` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
        }
    }
}

impl Default for ItemSize {
    /// shadcn/ui の既定（`default`）と一致させる。
    fn default() -> Self {
        Self::Default
    }
}

/// [`media`] の見た目バリアント（shadcn/ui `ItemMedia` の `variant` prop に
/// 対応。アイコン・画像・アバターの入れ物の見た目を切り替える）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemMediaVariant {
    /// 既定の見た目。
    Default,
    /// アイコン用の見た目。
    Icon,
    /// 画像用の見た目。
    Image,
}

impl ItemMediaVariant {
    /// `data-variant` の属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Icon => "icon",
            Self::Image => "image",
        }
    }
}

impl Default for ItemMediaVariant {
    /// shadcn/ui の既定（`default`）と一致させる。
    fn default() -> Self {
        Self::Default
    }
}

/// [`root`] の描画引数。将来の非破壊的拡張に備えた props 構造体
/// （[`crate::field::FieldProps`] と同型のライフタイム付き props）。
#[derive(Debug, Clone, Copy, Default)]
pub struct ItemRootProps<'a> {
    /// `Some` なら [`root`] を `a` として描画し、値を `href` へ固定付与する
    /// （モジュール doc「`root` を `a` として描画する経路」参照）。
    pub href: Option<&'a str>,
    /// `href` が `Some` のときのみ意味を持つ。`true` なら
    /// `target="_blank"` + `rel="noopener noreferrer"` を不可分に付与する。
    pub external: bool,
    /// 見た目バリアント。
    pub variant: ItemVariant,
    /// サイズバリアント。
    pub size: ItemSize,
}

/// `root` パーツ（`href` が `None` なら `div`、`Some` なら `a`）。
/// `data-variant`/`data-size` を固定出力する（モジュール doc参照）。
#[must_use]
pub fn root<'a>(
    props: ItemRootProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ROOT_RESERVED);
    let tag: &'static str = if props.href.is_some() { "a" } else { "div" };
    let mut merged: Vec<(&'a str, &'a str)> = Vec::with_capacity(5 + attrs.len());
    if let Some(href) = props.href {
        merged.push(("href", href));
        if props.external {
            merged.push(("target", "_blank"));
            merged.push(("rel", "noopener noreferrer"));
        }
    }
    merged.push(("data-variant", props.variant.as_str()));
    merged.push(("data-size", props.size.as_str()));
    merged.extend(attrs);
    ANATOMY.part("root", tag, merged, children)
}

/// `media` パーツ（`div`）。アイコン・画像・アバターの入れ物。
/// `data-variant` を固定出力する。
#[must_use]
pub fn media<'a>(
    variant: ItemMediaVariant,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, MEDIA_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-variant", variant.as_str())];
    merged.extend(attrs);
    ANATOMY.part("media", "div", merged, children)
}

/// `content` パーツ（`div`）。`title`/`description` の縦積みコンテナ。
#[must_use]
pub fn content<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("content", "div", attrs, children)
}

/// `title` パーツ（`div`）。shadcn `ItemTitle` と同じ `div`（見出しレベルは
/// 呼び出し側が子ノードで決める）。
#[must_use]
pub fn title<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("title", "div", attrs, children)
}

/// `description` パーツ（`p`）。shadcn `ItemDescription` と同じ `p`。
#[must_use]
pub fn description<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("description", "p", attrs, children)
}

/// `actions` パーツ（`div`）。ボタン等の末尾スロット。
#[must_use]
pub fn actions<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("actions", "div", attrs, children)
}

/// `header` パーツ（`div`）。行の上部スロット。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("header", "div", attrs, children)
}

/// `footer` パーツ（`div`）。行の下部スロット。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, NO_RESERVED);
    ANATOMY.part("footer", "div", attrs, children)
}

/// `group` パーツ（`div`）。複数 [`root`] の縦並びコンテナ。`role="group"` を
/// 固定出力する（モジュール doc「`group` は `role="group"`」参照）。`label`
/// は動的値であり [`fandhe_frontend_core::render`] の既定エスケープを経由
/// して `aria-label` へ出力する（空文字列のときは省略する）。
#[must_use]
pub fn group<'a>(label: &'a str, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, GROUP_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("group")];
    if !label.is_empty() {
        merged.push(aria_label(label));
    }
    merged.extend(attrs);
    ANATOMY.part("group", "div", merged, children)
}

/// `separator` パーツ（`div`）。[`group`] 内で複数 [`root`] を区切る水平線
/// （モジュール doc「`separator` は `group` 内専用の水平固定パーツ」参照）。
/// `role="separator"` + `aria-orientation="horizontal"` +
/// `data-orientation="horizontal"` を固定出力する。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, SEPARATOR_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("separator"),
        aria_orientation(Orientation::Horizontal),
        data_orientation(Orientation::Horizontal),
    ];
    merged.extend(attrs);
    ANATOMY.part("separator", "div", merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn root_div_defaults_have_no_href_and_no_role() {
        let node = root(ItemRootProps::default(), vec![], vec![]);
        let html = render(&node);
        assert!(html.starts_with("<div"));
        assert!(html.contains(r#"data-scope="item""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-variant="default""#));
        assert!(html.contains(r#"data-size="default""#));
        assert!(!html.contains("href="));
        assert!(!html.contains("role="));
    }

    #[test]
    fn root_div_variant_and_size_vocabulary() {
        for (variant, expected) in [
            (ItemVariant::Default, "default"),
            (ItemVariant::Outline, "outline"),
            (ItemVariant::Muted, "muted"),
        ] {
            let props = ItemRootProps {
                variant,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-variant="{expected}""#)));
        }
        for (size, expected) in [(ItemSize::Default, "default"), (ItemSize::Sm, "sm")] {
            let props = ItemRootProps {
                size,
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-size="{expected}""#)));
        }
    }

    #[test]
    fn root_a_outputs_href_and_no_role() {
        let props = ItemRootProps {
            href: Some("/docs/item"),
            ..Default::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.starts_with("<a"));
        assert!(html.contains(r#"href="/docs/item""#));
        assert!(!html.contains("role="));
    }

    #[test]
    fn root_a_external_true_adds_target_and_rel_inseparably() {
        let props = ItemRootProps {
            href: Some("https://example.com"),
            external: true,
            ..Default::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains(r#"target="_blank""#));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
    }

    #[test]
    fn root_a_external_false_omits_target_and_rel() {
        let props = ItemRootProps {
            href: Some("/docs"),
            external: false,
            ..Default::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(!html.contains("target="));
        assert!(!html.contains("rel="));
    }

    #[test]
    fn root_div_external_true_without_href_omits_target_and_rel() {
        let props = ItemRootProps {
            href: None,
            external: true,
            ..Default::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.starts_with("<div"));
        assert!(!html.contains("target="));
        assert!(!html.contains("rel="));
    }

    #[test]
    fn root_a_dangerous_url_schemes_are_rejected() {
        let dangerous_urls = [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "vbscript:msgbox(1)",
        ];
        for url in dangerous_urls {
            let props = ItemRootProps {
                href: Some(url),
                ..Default::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(
                !html.contains("href="),
                "危険な URL スキームなのに href 属性が出力されている: url={url:?}, html={html}"
            );
        }
    }

    #[test]
    fn root_a_href_attribute_breakout_payload_is_escaped() {
        let props = ItemRootProps {
            href: Some("/docs\" onmouseover=\"alert(1)"),
            ..Default::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[test]
    fn root_drops_impersonated_reserved_keys_case_insensitively() {
        let node = root(
            ItemRootProps::default(),
            vec![
                ("HREF", "/evil"),
                ("Target", "_self"),
                ("REL", "evil"),
                ("data-variant", "outline"),
                ("DATA-SIZE", "sm"),
                ("data-testid", "kept"),
            ],
            vec![],
        );
        let html = render(&node);
        assert!(!html.contains("/evil"));
        assert!(!html.contains("_self"));
        assert!(html.contains(r#"data-variant="default""#));
        assert!(html.contains(r#"data-size="default""#));
        assert!(html.contains(r#"data-testid="kept""#));
    }

    #[test]
    fn media_variant_vocabulary() {
        for (variant, expected) in [
            (ItemMediaVariant::Default, "default"),
            (ItemMediaVariant::Icon, "icon"),
            (ItemMediaVariant::Image, "image"),
        ] {
            let html = render(&media(variant, vec![], vec![]));
            assert!(html.contains(r#"data-scope="item""#));
            assert!(html.contains(r#"data-part="media""#));
            assert!(html.contains(&format!(r#"data-variant="{expected}""#)));
        }
    }

    /// テスト専用の型エイリアス（`content`/`title`/`description`/`actions`/
    /// `header`/`footer` はいずれも同じシグネチャの自由関数のため、
    /// テーブル駆動テストで一括検証する際の複雑な型を読みやすくする）。
    type NoFixedAttrPartFn = fn(Vec<(&'static str, &'static str)>, Vec<Node>) -> Node;

    #[test]
    fn content_title_description_actions_header_footer_have_no_fixed_attrs() {
        let cases: Vec<(&str, &str, NoFixedAttrPartFn)> = vec![
            ("content", "div", content),
            ("title", "div", title),
            ("description", "p", description),
            ("actions", "div", actions),
            ("header", "div", header),
            ("footer", "div", footer),
        ];
        for (part, tag, f) in cases {
            let html = render(&f(vec![("data-testid", "kept")], vec![]));
            assert!(
                html.starts_with(&format!("<{tag}")),
                "part={part}, html={html}"
            );
            assert!(html.contains(r#"data-scope="item""#));
            assert!(html.contains(&format!(r#"data-part="{part}""#)));
            assert!(html.contains(r#"data-testid="kept""#));
        }
    }

    #[test]
    fn group_has_role_group_and_optional_aria_label() {
        let html = render(&group("Recent items", vec![], vec![]));
        assert!(html.contains(r#"role="group""#));
        assert!(html.contains(r#"aria-label="Recent items""#));
        assert!(!html.contains("aria-orientation"));

        let html = render(&group("", vec![], vec![]));
        assert!(!html.contains("aria-label"));
    }

    #[test]
    fn group_drops_impersonated_reserved_keys() {
        let node = group(
            "Real",
            vec![("role", "list"), ("aria-label", "Fake")],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"role="group""#));
        assert!(!html.contains(r#"role="list""#));
        assert!(html.contains(r#"aria-label="Real""#));
        assert!(!html.contains("Fake"));
    }

    #[test]
    fn separator_is_fixed_horizontal() {
        let html = render(&separator(vec![], vec![]));
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
        assert!(html.contains(r#"data-orientation="horizontal""#));
    }

    #[test]
    fn separator_drops_impersonated_reserved_keys() {
        let node = separator(
            vec![("role", "button"), ("aria-orientation", "vertical")],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"role="separator""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
        assert!(!html.contains(r#"role="button""#));
    }

    #[test]
    fn anatomy_scope_and_part_spoofing_is_removed() {
        let node = root(
            ItemRootProps::default(),
            vec![("data-scope", "evil"), ("data-part", "evil")],
            vec![],
        );
        let html = render(&node);
        assert!(html.contains(r#"data-scope="item""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("evil"));
    }

    #[test]
    fn no_data_hydrate_attributes_are_emitted() {
        let node = group(
            "g",
            vec![],
            vec![root(ItemRootProps::default(), vec![], vec![])],
        );
        let html = render(&node);
        assert!(!html.contains("data-hydrate"));
    }

    #[test]
    fn xss_payload_in_label_href_and_children_is_escaped_on_render() {
        let payload = "\"><script>alert(1)</script>";
        let node = group(
            payload,
            vec![("id", payload)],
            vec![root(
                ItemRootProps {
                    href: Some(payload),
                    ..Default::default()
                },
                vec![],
                vec![title(vec![], vec![text(payload)])],
            )],
        );
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
