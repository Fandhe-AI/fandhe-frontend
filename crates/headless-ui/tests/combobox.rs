//! Combobox（イシュー #749）の統合テスト。
//!
//! `crates/headless-ui/src/combobox.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! `label + control(input, trigger, clear_trigger) +`
//! `positioner(content(item_group(item(item_text, item_indicator))))` という
//! 全体の組み立てにおける data-*/ARIA 対応・dispatch 統合（closeOnSelect/
//! openOnChange 含む）・SSR/hydration 両経路・XSS 回帰をクレート外部から
//! （公開 API のみを使って）固定する（`crates/headless-ui/tests/select.rs`
//! と同じ粒度）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::combobox::{self, Combobox, ComboboxProps};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_controls_labelledby_and_all_parts_appear() {
    let props = ComboboxProps::default();
    let label = combobox::label(
        &props,
        Some("combobox-label-1"),
        Some("combobox-input-1"),
        vec![],
        vec![text("Framework")],
    );

    let input = combobox::input(
        OpenState::Open,
        "vu",
        &props,
        Some("combobox-content-1"),
        Some("item-vue"),
        Some("framework"),
        vec![("id", "combobox-input-1")],
    );
    let trigger = combobox::trigger(
        OpenState::Open,
        &props,
        Some("combobox-content-1"),
        vec![],
        vec![],
    );
    let clear_trigger = combobox::clear_trigger(&props, vec![("aria-label", "Clear")], vec![]);
    let control = combobox::control(
        OpenState::Open,
        &props,
        vec![],
        vec![input, trigger, clear_trigger],
    );

    let item_text = combobox::item_text(Some("item-text-vue"), vec![], vec![text("Vue")]);
    let item_indicator = combobox::item_indicator(OpenState::Open, vec![], vec![text("✓")]);
    let item = combobox::item(
        OpenState::Open,
        false,
        true,
        "vue",
        Some("item-vue"),
        vec![],
        vec![item_text, item_indicator],
    );
    let item_group_label =
        combobox::item_group_label(Some("group-label-1"), vec![], vec![text("Frameworks")]);
    let item_group =
        combobox::item_group(Some("group-label-1"), vec![], vec![item_group_label, item]);
    let content = combobox::content(
        OpenState::Open,
        Some("combobox-content-1"),
        Some("combobox-label-1"),
        vec![],
        vec![item_group],
    );
    let positioner = combobox::positioner(OpenState::Open, vec![], vec![content]);

    let root = combobox::root(
        OpenState::Open,
        &props,
        vec![],
        vec![label, control, positioner],
    );

    let html = render(&root);

    // 全 data-part の出現を固定する。
    for part in [
        "root",
        "label",
        "control",
        "input",
        "trigger",
        "clear-trigger",
        "positioner",
        "content",
        "item-group",
        "item-group-label",
        "item",
        "item-text",
        "item-indicator",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing data-part=\"{part}\" in: {html}"
        );
    }

    // aria-controls/aria-labelledby/label[for] の id 対応。
    assert!(html.contains(r#"aria-controls="combobox-content-1""#));
    assert!(html.contains(r#"id="combobox-content-1""#));
    assert!(html.contains(r#"aria-labelledby="combobox-label-1""#));
    assert!(html.contains(r#"id="combobox-label-1""#));
    assert!(html.contains(r#"for="combobox-input-1""#));
    assert!(html.contains(r#"id="combobox-input-1""#));

    // role / aria-* の付与。
    assert!(html.contains(r#"role="combobox""#));
    assert!(html.contains(r#"aria-haspopup="listbox""#));
    assert!(html.contains(r#"role="listbox""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-selected="true""#));
    assert!(html.contains(r#"aria-autocomplete="list""#));
    assert!(html.contains(r#"autocomplete="off""#));

    // aria-activedescendant は input 側にのみ配線され、content には付与
    // されない（モジュール doc「aria-activedescendant の配線先」参照。
    // Select（content 側）との差異）。
    assert!(html.contains(r#"aria-activedescendant="item-vue""#));
    assert_eq!(html.matches("aria-activedescendant").count(), 1);

    // trigger はタブ順から外れる（フォーカスは input が保持する）。
    assert!(html.contains(r#"tabindex="-1""#));

    // highlight の SSR 表現: item の data-highlighted/id と input の
    // aria-activedescendant が同一 id で対応する。
    assert!(html.contains(r#"data-highlighted="""#));
    assert!(html.contains(r#"id="item-vue""#));

    // input の現在値。
    assert!(html.contains(r#"value="vu""#));

    // open 状態なので positioner/content に hidden 存在属性は付かない。
    assert!(!html.contains(r#" hidden="""#));
}

#[test]
fn positioner_and_content_closed_have_hidden_and_no_role_leak() {
    let content = combobox::content(OpenState::Closed, None, None, vec![], vec![]);
    let positioner = combobox::positioner(OpenState::Closed, vec![], vec![content]);
    let html = render(&positioner);
    assert!(html.contains(r#"data-state="closed""#));
    // positioner と content の両方に hidden が付く。
    assert_eq!(html.matches(r#"hidden="""#).count(), 2);
}

#[test]
fn dispatch_open_close_toggle_flip_data_state_across_parts() {
    let mut c = Combobox::default();
    assert!(!c.is_open());
    assert!(render(&c.content(None, None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut c, "open", ""));
    assert!(c.is_open());
    let props = ComboboxProps::default();
    assert!(render(&c.root(&props, vec![], vec![])).contains(r#"data-state="open""#));
    assert!(render(&c.input(&props, None, None, None, vec![])).contains(r#"aria-expanded="true""#));
    assert!(render(&c.trigger(&props, None, vec![], vec![])).contains(r#"aria-expanded="true""#));
    assert!(render(&c.positioner(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&c.content(None, None, vec![], vec![])).contains("hidden"));

    assert!(dispatch(&mut c, "close", ""));
    assert!(!c.is_open());

    assert!(dispatch(&mut c, "toggle", ""));
    assert!(c.is_open());
    assert!(dispatch(&mut c, "toggle", ""));
    assert!(!c.is_open());
}

#[test]
fn dispatch_select_updates_value_and_closes_listbox_close_on_select() {
    let mut c = Combobox::default();
    dispatch(&mut c, "open", "");
    assert!(c.is_open());

    assert!(dispatch(&mut c, "select", "vue"));
    assert_eq!(c.selected(), Some("vue"));
    assert!(!c.is_open());

    assert!(render(&c.item("vue", false, false, None, vec![], vec![]))
        .contains(r#"aria-selected="true""#));
    assert!(render(&c.item("react", false, false, None, vec![], vec![]))
        .contains(r#"aria-selected="false""#));
}

#[test]
fn dispatch_deselect_clears_selection() {
    let mut c = Combobox::default();
    dispatch(&mut c, "select", "vue");
    assert!(dispatch(&mut c, "deselect", ""));
    assert_eq!(c.selected(), None);
}

#[test]
fn dispatch_input_updates_value_and_opens_listbox_open_on_change() {
    let mut c = Combobox::default();
    assert!(!c.is_open());

    assert!(dispatch(&mut c, "input", "re"));
    assert_eq!(c.input_value(), "re");
    assert!(c.is_open());

    let options = [("vue", "Vue"), ("react", "React")];
    assert_eq!(c.filtered_options(&options), vec![("react", "React")]);
}

#[test]
fn dispatch_clear_clears_input_and_selection() {
    let mut c = Combobox::default();
    dispatch(&mut c, "input", "vu");
    dispatch(&mut c, "select", "vue");

    assert!(dispatch(&mut c, "clear", ""));
    assert_eq!(c.input_value(), "");
    assert_eq!(c.selected(), None);
}

#[test]
fn dispatch_ignores_unknown_action() {
    let mut c = Combobox::default();
    dispatch(&mut c, "select", "vue");
    assert!(!dispatch(&mut c, "no_such_action", "x"));
    assert_eq!(c.selected(), Some("vue"));
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let c = Combobox::default();
    let html = render(&c.view());
    assert!(!html.contains("data-hydrate-"));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let mut c = Combobox::default();
    dispatch(&mut c, "input", "vu");
    dispatch(&mut c, "select", "vue");
    // select は listbox を閉じるため、開いた状態のラウンドトリップを
    // 確認するために再度 open する。
    dispatch(&mut c, "open", "");

    let html = render(&render_for_hydration(&c));
    assert!(html.contains(r#"data-hydrate-state="open""#));
    assert!(html.contains("data-hydrate-selected="));
    assert!(html.contains("data-hydrate-input="));

    let restored = Combobox::from_hydration_attrs(&c.hydration_attrs()).unwrap();
    assert_eq!(restored, c);
}

#[test]
fn hydration_tampered_state_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![
            ("data-hydrate-state".to_string(), bogus.to_string()),
            (
                "data-hydrate-selected".to_string(),
                fandhe_frontend_interactive::codec::encode_list(&[]),
            ),
            (
                "data-hydrate-input".to_string(),
                fandhe_frontend_interactive::codec::encode_list(&[String::new()]),
            ),
        ];
        let err = Combobox::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

#[test]
fn hydration_missing_attrs_returns_error_not_panic() {
    let err = Combobox::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-state".to_string())
    );
}

#[test]
fn hydration_tampered_multiple_selected_returns_error_not_panic() {
    use fandhe_frontend_interactive::codec;
    let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
    let attrs = vec![
        ("data-hydrate-state".to_string(), "closed".to_string()),
        ("data-hydrate-selected".to_string(), bogus),
        (
            "data-hydrate-input".to_string(),
            codec::encode_list(&[String::new()]),
        ),
    ];
    let err = Combobox::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn hydration_tampered_input_value_count_returns_error_not_panic() {
    use fandhe_frontend_interactive::codec;
    let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
    let attrs = vec![
        ("data-hydrate-state".to_string(), "closed".to_string()),
        ("data-hydrate-selected".to_string(), codec::encode_list(&[])),
        ("data-hydrate-input".to_string(), bogus),
    ];
    let err = Combobox::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn controls_labelledby_value_and_activedescendant_payloads_are_escaped_end_to_end() {
    let props = ComboboxProps::default();
    let input = combobox::input(
        OpenState::Closed,
        ATTR_BREAK_PAYLOAD,
        &props,
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
    );
    let trigger = combobox::trigger(
        OpenState::Closed,
        &props,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let item = combobox::item(
        OpenState::Closed,
        false,
        false,
        ATTR_BREAK_PAYLOAD,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = combobox::content(
        OpenState::Closed,
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let html = render(&combobox::root(
        OpenState::Closed,
        &props,
        vec![],
        vec![input, trigger, item, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn dispatch_input_payload_is_escaped_end_to_end() {
    let mut c = Combobox::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut c, "input", payload));

    let html = render(&render_for_hydration(&c));
    assert!(html.contains("data-hydrate-input="));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains(r#""><script"#));
}

#[test]
fn dispatch_select_payload_is_escaped_end_to_end() {
    let mut c = Combobox::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut c, "select", payload));

    let html = render(&render_for_hydration(&c));
    assert!(html.contains("data-hydrate-selected="));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains(r#""><script"#));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&combobox::root(
        OpenState::Closed,
        &ComboboxProps::default(),
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

#[test]
fn filter_options_end_to_end_with_dynamic_query_and_labels() {
    let options = [("vue", "Vue"), ("react", "React"), ("svelte", "Svelte")];
    let mut c = Combobox::default();
    dispatch(&mut c, "input", "re");
    assert_eq!(c.filtered_options(&options), vec![("react", "React")]);

    dispatch(&mut c, "input", "");
    assert_eq!(c.filtered_options(&options), options.to_vec());
}

// ---------------------------------------------------------------------
// ARIA 関連付け契約（イシュー #1067）
//
// `combobox::input` の `controls`/`activedescendant` 引数は opt-in
// （`Option`）であり、呼び出し側が `None` を渡しても型では検知できない
// （`crates/headless-ui/src/combobox.rs` 参照）。本節は「どういう構成なら
// 関連付けが必須か」という条件付き規則（R1〜R4）を、実際に render() した
// HTML 文字列に対して検証する契約テストで固定する。
//
// `crates/docs-site/tests/combobox_aria_association.rs` が同一規則を
// 実出荷マークアップ（Primitives/Themes 全ページ）へ適用する。クレート
// 境界を跨ぐ test helper 共有は Rust の統合テストでは不可能なため、
// 検証ロジックは意図的に重複実装している。**規則を変更するときは両方の
// ファイルを更新すること。**
// ---------------------------------------------------------------------

/// 開始タグ内容 `tag`（例: `r#"input role="combobox" aria-expanded="true""#`）
/// から属性 `name` の値を取り出す。
///
/// 属性値中に生の `"` が現れないという既定エスケープの不変条件（本節冒頭
/// コメント参照）により、`name="` の前方に半角スペースを要求するだけで
/// 他属性名との部分一致誤検出（例: `data-value` の中に `value` が部分
/// 文字列として現れる）を避けられる。
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// HTML 断片から開始タグの内容（`<` と `>` の間、終了タグは除く）を
/// 「どの combobox インスタンスの部分木に属するか」（`scope`）付きで
/// 登場順に切り出す。
///
/// [`fandhe_frontend_core::render`] の既定エスケープにより、テキスト
/// ノードの内容は `<`/`>` を含まない（常に `&lt;`/`&gt;` へエスケープ
/// される）ため、`<...>` の単純な区間探索で「本物のタグ境界」のみを
/// 拾えることに依拠する（この前提が崩れる変更は、本モジュール既存の
/// `*_payloads_are_escaped_end_to_end` 系 XSS 回帰テストが先に検知する）。
/// `render()` は void 要素（`input` 等、HTML Standard 13.1.2）を終了タグ
/// なしで自己終端出力する（イシュー #1139）ため、本関数はスタックへ
/// push した直後にすぐ pop して「開始/終了タグの対応」を模擬する
/// （void 要素は対応する `</...>` を持たないため、無条件 push だと
/// スタックが恒久的に不整合になる）。
///
/// `data-scope="combobox" data-part="root"`（[`combobox::root`] が出力する
/// マーカー）を持つ要素を新しいスコープの起点とし、その部分木配下（自身を
/// 含む）の全タグへ同一の `scope` 番号を割り当てる。祖先にそのような
/// root が無いタグは `scope = None`（ページ chrome 等）。1 ページに複数の
/// combobox インスタンスが共存する場合（docs-site の Demo + Examples 原稿
/// 断片が同一ページへ合成される構成、イシュー #1067 実測: `/primitives/combobox/`）
/// に、一方のインスタンスのハイライト item がもう一方のインスタンス（無関係
/// に closed な別 combobox）の R3 判定へ誤って波及しないよう、この境界で
/// 区切る。R2/R4（`id` 参照の実在確認）はページ全体で `id` が一意である
/// 前提のもとスコープを跨いで解決してよいため、この関数の呼び出し側は
/// `id -> role` の対応表のみページ全体で構築し、R3（ハイライト item の
/// 集合）だけをスコープ単位で構築する。
/// HTML Standard 13.1.2 の void 要素一覧（[`fandhe_frontend_core`] の
/// `VOID_ELEMENTS`（非公開）と同一集合。本テストヘルパーはコアの
/// 出力形式に依存する検証用途のため、独立して固定する）。
const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// `tag`（`<` `>` 間の内容）先頭のタグ名を取り出し、void 要素かどうかを
/// 判定する。
fn is_void_tag(tag: &str) -> bool {
    let name = tag.split(|c: char| c.is_whitespace()).next().unwrap_or("");
    VOID_TAGS.contains(&name)
}

fn scoped_open_tags(html: &str) -> Vec<(&str, Option<usize>)> {
    let mut tags = Vec::new();
    let mut stack: Vec<Option<usize>> = Vec::new();
    let mut next_scope = 0usize;
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        let after_lt = &rest[lt + 1..];
        if after_lt.starts_with('/') {
            // 終了タグは属性を持たない。対応する開始タグのスコープを
            // スタックから取り除く。
            match after_lt.find('>') {
                Some(gt) => {
                    stack.pop();
                    rest = &after_lt[gt + 1..];
                    continue;
                }
                None => break,
            }
        }
        match after_lt.find('>') {
            Some(gt) => {
                let tag = &after_lt[..gt];
                let parent_scope = stack.last().copied().flatten();
                let own_scope = if attr(tag, "data-scope") == Some("combobox")
                    && attr(tag, "data-part") == Some("root")
                {
                    next_scope += 1;
                    Some(next_scope)
                } else {
                    parent_scope
                };
                tags.push((tag, own_scope));
                if !is_void_tag(tag) {
                    // void 要素は render() が終了タグなしで自己終端出力する
                    // （イシュー #1139）ため、対応する `</...>` を待たず
                    // スタックへ push しない（push すると恒久的に
                    // 不整合になる）。
                    stack.push(own_scope);
                }
                rest = &after_lt[gt + 1..];
            }
            None => break,
        }
    }
    tags
}

/// R1〜R4（本節冒頭コメント参照）を検証し、違反があれば規則名付きの
/// `Err` を返す。
///
/// - **R1**: `role="combobox"` かつ `aria-expanded="true"` の要素は
///   `aria-controls` を持つ。
/// - **R2**: `aria-controls="X"` の `X` は同一断片内に `id="X"` かつ
///   `role="listbox"` を持つ要素として実在する（dangling IDREF・誤配線の
///   禁止）。
/// - **R3**: 同一 combobox インスタンス（[`scoped_open_tags`] 参照）内に
///   `role="option"` かつ `data-highlighted` 属性を持つ要素が 1 件以上
///   あるとき、そのインスタンスの combobox 要素は `aria-activedescendant`
///   を持ち、その値がハイライト要素の `id` と一致する（ハイライト要素が
///   `id` を持たない場合は関連付け不能としてエラー）。
/// - **R4**: `aria-activedescendant="Y"` があるとき `id="Y"` が同一断片内に
///   実在する（dangling IDREF の禁止）。
///
/// closed かつ [`combobox::content`] 自体を描画しない構成（3 引数とも
/// `None`）は `aria-expanded="true"` にならず、ハイライト要素も存在しない
/// ため `Ok(())` を返す（ARIA 上正しい構成を誤検知で落とさない）。
fn verify_combobox_aria_association(html: &str) -> Result<(), String> {
    let scoped_tags = scoped_open_tags(html);

    // R2/R4 の id 参照実在確認はページ全体（id は一意である前提）。
    let mut id_role: std::collections::HashMap<&str, Option<&str>> =
        std::collections::HashMap::new();
    for (tag, _) in &scoped_tags {
        if let Some(id) = attr(tag, "id") {
            id_role.insert(id, attr(tag, "role"));
        }
    }

    // R3 のハイライト item 集合はインスタンス（scope）単位で構築する。
    let mut highlighted_ids_by_scope: std::collections::HashMap<Option<usize>, Vec<&str>> =
        std::collections::HashMap::new();
    let mut highlighted_without_id_scopes: std::collections::HashSet<Option<usize>> =
        std::collections::HashSet::new();
    for (tag, scope) in &scoped_tags {
        if attr(tag, "role") == Some("option") && attr(tag, "data-highlighted").is_some() {
            match attr(tag, "id") {
                Some(id) => highlighted_ids_by_scope.entry(*scope).or_default().push(id),
                None => {
                    highlighted_without_id_scopes.insert(*scope);
                }
            }
        }
    }

    for (tag, scope) in &scoped_tags {
        if attr(tag, "role") != Some("combobox") {
            continue;
        }
        let expanded = attr(tag, "aria-expanded") == Some("true");
        let controls = attr(tag, "aria-controls");
        let activedescendant = attr(tag, "aria-activedescendant");

        if expanded && controls.is_none() {
            return Err(format!(
                "R1 violation: role=\"combobox\" element with aria-expanded=\"true\" lacks aria-controls: <{tag}>"
            ));
        }

        if let Some(target) = controls {
            match id_role.get(target) {
                Some(Some("listbox")) => {}
                Some(_) => {
                    return Err(format!(
                        "R2 violation: aria-controls=\"{target}\" target lacks role=\"listbox\""
                    ));
                }
                None => {
                    return Err(format!(
                        "R2 violation: aria-controls=\"{target}\" target id does not exist (dangling IDREF)"
                    ));
                }
            }
        }

        if highlighted_without_id_scopes.contains(scope) {
            return Err(
                "R3 violation: highlighted role=\"option\" element lacks id (cannot be referenced by aria-activedescendant)"
                    .to_string(),
            );
        }
        if let Some(highlighted_ids) = highlighted_ids_by_scope.get(scope) {
            if !highlighted_ids.is_empty() {
                match activedescendant {
                    None => {
                        return Err(
                            "R3 violation: highlighted option exists but combobox lacks aria-activedescendant"
                                .to_string(),
                        );
                    }
                    Some(target) => {
                        if !highlighted_ids.contains(&target) {
                            return Err(format!(
                                "R3 violation: aria-activedescendant=\"{target}\" does not match any highlighted option id {highlighted_ids:?}"
                            ));
                        }
                    }
                }
            }
        }

        if let Some(target) = activedescendant {
            if !id_role.contains_key(target) {
                return Err(format!(
                    "R4 violation: aria-activedescendant=\"{target}\" target id does not exist (dangling IDREF)"
                ));
            }
        }
    }

    Ok(())
}

/// 既存 `full_assembly_wires_aria_controls_labelledby_and_all_parts_appear`
/// と同型の準拠済み組み立てが `Ok(())` になることを固定する（回帰の
/// ベースライン）。
#[test]
fn verify_combobox_aria_association_ok_for_compliant_full_assembly() {
    let input = combobox::input(
        OpenState::Open,
        "vu",
        &ComboboxProps::default(),
        Some("combobox-content-1"),
        Some("item-vue"),
        None,
        vec![],
    );
    let item = combobox::item(
        OpenState::Open,
        false,
        true,
        "vue",
        Some("item-vue"),
        vec![],
        vec![],
    );
    let content = combobox::content(
        OpenState::Open,
        Some("combobox-content-1"),
        None,
        vec![],
        vec![item],
    );
    let root = combobox::root(
        OpenState::Open,
        &ComboboxProps::default(),
        vec![],
        vec![input, content],
    );
    let html = render(&root);

    assert_eq!(verify_combobox_aria_association(&html), Ok(()));
}

/// closed かつ [`combobox::content`] を描画しない・3 引数とも `None` の
/// 構成は誤検知されず `Ok(())` になることを固定する（`primitive_specs`
/// の Examples 原稿と同型の構成、イシュー #1067 計画 §2 実測）。
#[test]
fn verify_combobox_aria_association_ok_for_closed_without_content() {
    let input = combobox::input(
        OpenState::Closed,
        "",
        &ComboboxProps::default(),
        None,
        None,
        None,
        vec![],
    );
    let root = combobox::root(
        OpenState::Closed,
        &ComboboxProps::default(),
        vec![],
        vec![input],
    );
    let html = render(&root);

    assert_eq!(verify_combobox_aria_association(&html), Ok(()));
}

/// open かつ `controls=None` は R1 違反として検知されることを固定する
/// （opt-in 欠落の検知力の証明）。
#[test]
fn verify_combobox_aria_association_detects_missing_controls_when_expanded() {
    let input = combobox::input(
        OpenState::Open,
        "vu",
        &ComboboxProps::default(),
        None,
        None,
        None,
        vec![],
    );
    let content = combobox::content(OpenState::Open, Some("cb-content"), None, vec![], vec![]);
    let root = combobox::root(
        OpenState::Open,
        &ComboboxProps::default(),
        vec![],
        vec![input, content],
    );
    let html = render(&root);

    let result = verify_combobox_aria_association(&html);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("R1 violation"));
}

/// ハイライト item（`id=Some`）が存在するのに `activedescendant=None` は
/// R3 違反として検知されることを固定する。
#[test]
fn verify_combobox_aria_association_detects_missing_activedescendant_when_highlighted() {
    let input = combobox::input(
        OpenState::Open,
        "vu",
        &ComboboxProps::default(),
        Some("cb-content"),
        None,
        None,
        vec![],
    );
    let item = combobox::item(
        OpenState::Open,
        false,
        true,
        "vue",
        Some("item-vue"),
        vec![],
        vec![],
    );
    let content = combobox::content(
        OpenState::Open,
        Some("cb-content"),
        None,
        vec![],
        vec![item],
    );
    let root = combobox::root(
        OpenState::Open,
        &ComboboxProps::default(),
        vec![],
        vec![input, content],
    );
    let html = render(&root);

    let result = verify_combobox_aria_association(&html);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("R3 violation"));
}

/// ハイライト item が `id=None`（関連付け不能）のときも R3 違反として
/// 検知されることを固定する。
#[test]
fn verify_combobox_aria_association_detects_highlighted_item_without_id() {
    let input = combobox::input(
        OpenState::Open,
        "vu",
        &ComboboxProps::default(),
        Some("cb-content"),
        None,
        None,
        vec![],
    );
    let item = combobox::item(OpenState::Open, false, true, "vue", None, vec![], vec![]);
    let content = combobox::content(
        OpenState::Open,
        Some("cb-content"),
        None,
        vec![],
        vec![item],
    );
    let root = combobox::root(
        OpenState::Open,
        &ComboboxProps::default(),
        vec![],
        vec![input, content],
    );
    let html = render(&root);

    let result = verify_combobox_aria_association(&html);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("R3 violation"));
}

/// `activedescendant` が実在しない `id` を指すとき R4 違反として検知
/// されることを固定する（dangling IDREF）。
#[test]
fn verify_combobox_aria_association_detects_dangling_activedescendant() {
    let input = combobox::input(
        OpenState::Open,
        "vu",
        &ComboboxProps::default(),
        Some("cb-content"),
        Some("no-such-id"),
        None,
        vec![],
    );
    let content = combobox::content(OpenState::Open, Some("cb-content"), None, vec![], vec![]);
    let root = combobox::root(
        OpenState::Open,
        &ComboboxProps::default(),
        vec![],
        vec![input, content],
    );
    let html = render(&root);

    let result = verify_combobox_aria_association(&html);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("R4 violation"));
}

/// `aria-controls` が `role="listbox"` を持たない要素の `id` を指すとき
/// R2 違反として検知されることを固定する（誤配線の禁止）。
#[test]
fn verify_combobox_aria_association_detects_controls_target_without_listbox_role() {
    let input = combobox::input(
        OpenState::Open,
        "vu",
        &ComboboxProps::default(),
        Some("not-a-listbox"),
        None,
        None,
        vec![],
    );
    // `role="listbox"` を持たない、無関係な div へ id を付与する。
    let decoy = combobox::label(
        &ComboboxProps::default(),
        Some("not-a-listbox"),
        None,
        vec![],
        vec![],
    );
    let root = combobox::root(
        OpenState::Open,
        &ComboboxProps::default(),
        vec![],
        vec![input, decoy],
    );
    let html = render(&root);

    let result = verify_combobox_aria_association(&html);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("R2 violation"));
}

/// `Combobox` 状態機械経由（`impl Combobox` の利便メソッド）で組み立てた
/// ときも、open 時に `controls=None` を渡せば同じ落とし穴があることを
/// 固定する（自由関数だけでなく利便メソッド経由の opt-in 欠落も検知
/// できることの確認）。
#[test]
fn verify_combobox_aria_association_detects_missing_controls_via_state_machine_convenience_method()
{
    let mut c = Combobox::default();
    assert!(dispatch(&mut c, "open", ""));

    let input = c.input(&ComboboxProps::default(), None, None, None, vec![]);
    let content = c.content(Some("cb-content"), None, vec![], vec![]);
    let root = c.root(&ComboboxProps::default(), vec![], vec![input, content]);
    let html = render(&root);

    let result = verify_combobox_aria_association(&html);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("R1 violation"));
}
