//! CSS 変数プレフィックスが `data-scope` と一致することを全部品横断で
//! 契約テスト化する（イシュー #1061）。
//!
//! # 背景
//!
//! `crates/docs-site/src/component_page.rs::collect_css_vars_for_scope` は
//! `--fandhe-<data-scope>-` を前方一致で走査して `/themes/<kebab>/` の
//! 「CSS Variables」表を機械導出する。部品側の CSS custom property 名が
//! 自身の `data-scope` と食い違うと、この表が黙って空になる
//! （`image-cropper` が `--fandhe-cropper-*` を使っていた実例、イシュー
//! #1061）。本テストは [`showcase::stylesheet`] が返す全部品集約 CSS
//! （`crate::showcase::stylesheet` を経由し、リポジトリ内すべての
//! pre-styled-ui 部品の recipe CSS を 1 本の文字列へ集約したもの）を
//! 走査し、`[data-scope="X"]` を含むルール内で使われる `--fandhe-*` が
//! `--fandhe-X-*`（または完全一致 `--fandhe-X`）であることを固定する。
//!
//! # 免除面（2 つの literal 完全一致表のみ、カテゴリ免除は行わない）
//!
//! - [`SHARED_VARS`]: 複数部品が共有するテーマ非依存の custom property
//!   （colorPalette 系・positioning 系・z-index・未登録の共有色トークン
//!   2 件）。
//! - [`KNOWN_DEVIATIONS`]: `(data-scope, 変数名)` の既知の未是正逸脱。
//!   現在は `angle-slider` の `--fandhe-angle` の 1 件のみ。
//!   `crates/pre-styled-ui/src/angle_slider.rs` が動的インライン値として
//!   使う 1 点のみの custom property であり、是正には golden CSS・
//!   モジュール doc・`docs/api` への波及を伴う別個の破壊的変更が必要な
//!   ため本イシュー（#1061）のスコープ外とし、ここへ明示登録して契約から
//!   除外する（後続イシュー起票を提案、PR 本文参照）。
//!
//! いずれも**変数名の literal 完全一致リスト**であり、「動的インライン値
//! は対象外」のようなカテゴリ述語にはしない（カテゴリ免除にすると
//! `--fandhe-cropper-*` もリネーム前後で PASS してしまい、本契約が
//! 本イシューの修正に対して無意味〔vacuous〕になるため）。両表とも
//! stale エントリ（CSS から消えた免除）を検知し、是正後の掃除漏れを防ぐ。
//!
//! # `@keyframes` ブロックの扱い
//!
//! keyframe のブロックキー（`from`/`to`/`N%`）は `data-scope` セレクタを
//! 持たないが、`marquee.rs`/`progress.rs`/`skeleton.rs`/`spinner.rs` の
//! ように `var(--fandhe-marquee-gap, 1rem)` 等スコープ変数を参照する。
//! 単純にスキップすると網羅の穴になるため、keyframe ブロック内の
//! `--fandhe-*` は「テーマトークン ∪ [`SHARED_VARS`] ∪ 収集済み全 scope の
//! いずれかに前方一致」という scope 単位より弱い契約で検証する（この
//! 弱さは keyframe セレクタ自体が scope 情報を持たないという構造的制約に
//! よるものであり、恒久的な設計判断としてここに明記する）。

use std::collections::BTreeSet;

use fandhe_frontend_docs_site::showcase;
use fandhe_frontend_pre_styled_ui::theme::Theme;

/// 複数部品が共有し `data-scope` プレフィックス規約の対象外とする
/// custom property（literal 完全一致のみ）。
///
/// - `--fandhe-palette` / `-emphasized` / `-fg`: colorPalette 系
///   （`crates/pre-styled-ui/src/recipe.rs`）。
/// - `--fandhe-x` / `-y` / `-arrow-x` / `-arrow-y` / `-reference-width`:
///   positioning 系（wasm-full が実行時に設定、イシュー #663。
///   `menu.rs`/`popover.rs`/`select.rs`/`combobox.rs`/`tooltip.rs`/
///   `hover_card.rs`/`toggle_tip.rs`/`floating_panel.rs`）。
/// - `--fandhe-z-index-toast`: レイヤ z-index（`toast.rs`。
///   `Theme::to_css` は宣言しないためテーマトークンではない）。
/// - `--fandhe-color-accent-subtle` / `--fandhe-color-focus-ring`（イシュー
///   #1422 で `Theme::default()` の既定パレットへ正式追加するまで、この表の
///   エントリだった）: `analyze()` は `theme_tokens.contains(&name)` を
///   `SHARED_VARS` 判定より先に見るため、正式なテーマトークンになった時点で
///   このリストへ載せたままだと `shared_var_hits` が更新されず
///   `shared_vars_table_has_no_stale_entries` が stale 検知で FAIL する。
///   よって #1422 でこの 2 件を削除した（`menubar.rs`/`navigation-menu.rs`/
///   `tree-view.rs`/`toolbar.rs`/`date-input.rs` の 5 部品が参照する事実は
///   変わらないが、免除は `theme_tokens` 経由に一本化される。
///   `docs/design/color-token-system.md` §1 参照）。
const SHARED_VARS: &[&str] = &[
    "--fandhe-palette",
    "--fandhe-palette-emphasized",
    "--fandhe-palette-fg",
    "--fandhe-x",
    "--fandhe-y",
    "--fandhe-arrow-x",
    "--fandhe-arrow-y",
    "--fandhe-reference-width",
    "--fandhe-z-index-toast",
];

/// 既知の未是正逸脱（`(data-scope, 変数名)` の literal 完全一致のみ）。
const KNOWN_DEVIATIONS: &[(&str, &str)] = &[("angle-slider", "--fandhe-angle")];

/// 完全網羅の代表確認に使う固定サンプル（実装時点で存在確認済み）。
const EXPECTED_SAMPLE_SCOPES: &[&str] = &[
    "image-cropper",
    "accordion",
    "tags-input",
    "segment-group",
    "angle-slider",
];

/// CSS 中の 1 ルールブロック（`{ ... }`）の「自身が持つテキスト」。
/// ネストした子ブロック（`@media`/`@keyframes` 配下の個々のルール）は
/// 別要素として分離済みであり、`body` には子ブロックの中身を含まない。
struct Block {
    /// 直前の `{` までの text（トリム済み）。`@media (...)`・
    /// `.class[data-scope="x"][data-part="y"]` 等。
    selector: String,
    /// 子ブロックを除いた宣言テキスト。
    body: String,
    /// 祖先のいずれかが `@keyframes` ブロックであること。
    in_keyframes: bool,
}

/// CSS を波括弧の深さを数えながら走査し、[`Block`] の列へ分解する。
/// 正規表現・外部 CSS パーサクレートは使わない（REQ-3、
/// `component_page.rs::collect_css_vars_for_scope` と同じ素の文字列走査
/// 方針）。`@media`/`@supports`/`@keyframes` のネストを正しく扱うため、
/// 子ブロックを閉じるたびに走査再開位置を更新し、親ブロックの `body` へ
/// 子ブロックの中身が混入しないようにする。
fn parse_blocks(css: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut stack: Vec<(String, bool)> = Vec::new();
    let mut seg_start = 0usize;
    let bytes = css.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                let selector = css[seg_start..i].trim().to_string();
                let in_keyframes = stack.last().map(|(_, kf)| *kf).unwrap_or(false)
                    || selector.contains("@keyframes");
                stack.push((selector, in_keyframes));
                seg_start = i + 1;
            }
            b'}' => {
                if let Some((selector, in_keyframes)) = stack.pop() {
                    let body = css[seg_start..i].to_string();
                    blocks.push(Block {
                        selector,
                        body,
                        in_keyframes,
                    });
                }
                seg_start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    blocks
}

/// `text` 中で `data-scope="..."` として現れるスコープ名をすべて収集する
/// （複数 scope をカンマ連結したセレクタにも対応、`BTreeSet` で決定順）。
fn collect_scopes(text: &str) -> BTreeSet<String> {
    let mut scopes = BTreeSet::new();
    let marker = "data-scope=\"";
    let mut search_from = 0usize;
    while let Some(rel) = text[search_from..].find(marker) {
        let start = search_from + rel + marker.len();
        let Some(end_rel) = text[start..].find('"') else {
            break;
        };
        scopes.insert(text[start..start + end_rel].to_string());
        search_from = start + end_rel + 1;
    }
    scopes
}

/// `text` 中の `--fandhe-<...>` 識別子をすべて収集する（宣言
/// `--fandhe-x: ...` ・参照 `var(--fandhe-x` の双方を同一ロジックで拾う）。
fn collect_fandhe_var_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let marker = "--fandhe-";
    let mut search_from = 0usize;
    let bytes = text.as_bytes();
    while let Some(rel) = text[search_from..].find(marker) {
        let start = search_from + rel;
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-') {
            end += 1;
        }
        names.insert(text[start..end].to_string());
        search_from = end.max(start + marker.len());
    }
    names
}

/// `name` が `scope` の前方一致プレフィックス規約
/// （`--fandhe-<scope>` 完全一致 または `--fandhe-<scope>-` 前方一致）を
/// 満たすか判定する。
fn matches_scope_prefix(name: &str, scope: &str) -> bool {
    let exact = format!("--fandhe-{scope}");
    let prefixed = format!("--fandhe-{scope}-");
    name == exact || name.starts_with(&prefixed)
}

/// 契約違反 1 件分の情報（メッセージ組み立てに使う）。
struct Violation {
    selector: String,
    scopes: BTreeSet<String>,
    var_name: String,
}

/// 集約 CSS 全体を検証し、違反一覧・免除表の使用状況・収集済み scope 集合
/// を返す。
struct AnalysisResult {
    violations: Vec<Violation>,
    shared_var_hits: BTreeSet<&'static str>,
    known_deviation_hits: BTreeSet<(&'static str, &'static str)>,
    all_scopes: BTreeSet<String>,
}

fn analyze(css: &str, theme_tokens: &BTreeSet<String>) -> AnalysisResult {
    let blocks = parse_blocks(css);
    let all_scopes: BTreeSet<String> = collect_scopes(css);

    let mut violations = Vec::new();
    let mut shared_var_hits = BTreeSet::new();
    let mut known_deviation_hits = BTreeSet::new();

    for block in &blocks {
        let block_scopes = collect_scopes(&block.selector);
        let var_names = collect_fandhe_var_names(&block.body);

        for name in var_names {
            if theme_tokens.contains(&name) {
                continue;
            }
            if let Some(shared) = SHARED_VARS.iter().find(|&&s| s == name) {
                shared_var_hits.insert(*shared);
                continue;
            }

            if block.in_keyframes {
                // keyframe ブロックはセレクタが scope 情報を持たないため、
                // 収集済み全 scope のいずれかへの前方一致で弱く検証する
                // （モジュール doc「`@keyframes` ブロックの扱い」参照）。
                if all_scopes.iter().any(|s| matches_scope_prefix(&name, s)) {
                    continue;
                }
                violations.push(Violation {
                    selector: block.selector.clone(),
                    scopes: block_scopes.clone(),
                    var_name: name,
                });
                continue;
            }

            // KNOWN_DEVIATIONS はこのブロックが該当 scope を持つ場合のみ
            // 免除する（scope を跨いだ誤免除を避ける）。
            let deviation_hit = KNOWN_DEVIATIONS
                .iter()
                .find(|(s, v)| *v == name && block_scopes.contains(*s));
            if let Some(hit) = deviation_hit {
                known_deviation_hits.insert(*hit);
                continue;
            }

            if block_scopes.iter().any(|s| matches_scope_prefix(&name, s)) {
                continue;
            }

            violations.push(Violation {
                selector: block.selector.clone(),
                scopes: block_scopes.clone(),
                var_name: name,
            });
        }
    }

    AnalysisResult {
        violations,
        shared_var_hits,
        known_deviation_hits,
        all_scopes,
    }
}

/// 集約 CSS 中に実在する theme token 名の集合を [`Theme::to_css`] の出力
/// から機械導出する（`{color,space,radius,shadow,font}` 等のグループ名を
/// ハードコードしない）。
fn theme_token_names() -> BTreeSet<String> {
    collect_fandhe_var_names(&Theme::default().to_css())
}

/// 本テストの入力 CSS（全部品集約、`showcase::stylesheet()`）を組み立てる。
/// 構築失敗時は `StylesheetError`（`<`・制御文字混入）であり、pre-styled-ui
/// 側の生成 CSS は構造上到達しないはずだが、fail-closed に `expect` で
/// 落とす（黙って空 CSS を検証したことにしない）。
fn aggregated_css() -> String {
    showcase::stylesheet()
        .expect("showcase::stylesheet() should build without StylesheetError")
        .as_css()
        .to_string()
}

/// 契約本体: `[data-scope="X"]` を含むルール内の `--fandhe-*` はすべて
/// `--fandhe-X-*`（または SHARED_VARS・KNOWN_DEVIATIONS 免除）であること。
///
/// イシュー #1061 でリネーム前は `image-cropper` scope に対する
/// `--fandhe-cropper-x`/`-y`/`-w`/`-h`/`-handle-size` の 5 件ちょうどが
/// 違反として検出されることを確認済み（`--fandhe-angle` は
/// KNOWN_DEVIATIONS 免除済みのため現れない）。
#[test]
fn css_var_prefix_matches_data_scope_for_all_components() {
    let css = aggregated_css();
    let theme_tokens = theme_token_names();
    let result = analyze(&css, &theme_tokens);

    if !result.violations.is_empty() {
        let mut lines = Vec::new();
        for v in &result.violations {
            let scopes: Vec<&str> = v.scopes.iter().map(String::as_str).collect();
            lines.push(format!(
                "  selector={:?} scopes={:?} var={}",
                v.selector, scopes, v.var_name
            ));
        }
        panic!(
            "CSS 変数プレフィックスが data-scope と不一致（{} 件、イシュー #1061 契約違反）:\n{}",
            result.violations.len(),
            lines.join("\n")
        );
    }
}

/// [`SHARED_VARS`] の全エントリが集約 CSS 中に実在すること（stale エントリ
/// 防止）。是正済みの免除が居座らないことを保証する。
#[test]
fn shared_vars_table_has_no_stale_entries() {
    let css = aggregated_css();
    let theme_tokens = theme_token_names();
    let result = analyze(&css, &theme_tokens);

    let expected: BTreeSet<&str> = SHARED_VARS.iter().copied().collect();
    assert_eq!(
        result.shared_var_hits, expected,
        "SHARED_VARS に stale エントリまたは未使用がある（実使用と表の乖離）"
    );
}

/// [`KNOWN_DEVIATIONS`] の全エントリが集約 CSS 中に実在すること（stale
/// エントリ防止）。後続イシューで `--fandhe-angle` が是正されたら、この
/// アサーションが落ちて表の削除を強制する。
#[test]
fn known_deviations_table_has_no_stale_entries() {
    let css = aggregated_css();
    let theme_tokens = theme_token_names();
    let result = analyze(&css, &theme_tokens);

    let expected: BTreeSet<(&str, &str)> = KNOWN_DEVIATIONS.iter().copied().collect();
    assert_eq!(
        result.known_deviation_hits, expected,
        "KNOWN_DEVIATIONS に stale エントリまたは未使用がある（是正済みなら表を削除すること）"
    );
}

/// scope 集合が非空であり、代表サンプルをすべて含むこと（`showcase.rs`
/// への `push_css` 登録漏れが黙って契約から外れないための網羅性チェック）。
#[test]
fn collected_scopes_are_non_empty_and_cover_expected_samples() {
    let css = aggregated_css();
    let theme_tokens = theme_token_names();
    let result = analyze(&css, &theme_tokens);

    assert!(
        !result.all_scopes.is_empty(),
        "data-scope が 1 件も収集できていない（集約 CSS の組み立てが壊れている疑い）"
    );
    for sample in EXPECTED_SAMPLE_SCOPES {
        assert!(
            result.all_scopes.contains(*sample),
            "期待される代表 scope `{sample}` が集約 CSS 中に見つからない"
        );
    }
}

/// `/themes/image-cropper/` の生成 HTML に、リネーム後の CSS 変数表
/// （`component_page.rs::collect_css_vars_for_scope` による機械導出）が
/// 5 行・名前順（`BTreeMap` 整列）で出現し、旧名 `--fandhe-cropper-` が
/// 一切残らないことを固定する。リネーム前はこの表が 0 行（表ごと省略）
/// だったことが本イシュー（#1061）の実害であり、目視確認ではなくテストで
/// 固定する（計画 §6）。
#[test]
fn image_cropper_page_shows_renamed_css_variables_table() {
    let page =
        fandhe_frontend_docs_site::component_page::generated_content("/themes/image-cropper/")
            .expect("/themes/image-cropper/ should be a registered component page");
    let html = fandhe_frontend_core::render(&page);

    let heading = "<h3>CSS Variables</h3>";
    let heading_pos = html
        .find(heading)
        .unwrap_or_else(|| panic!("image-cropper ページに CSS Variables 節が出ていない: {html}"));
    // CSS Variables は API Reference 節内で最後の h3 のため、表の走査範囲は
    // 見出し直後から囲む `</section>` までに限定する（Demo 節のインライン
    // `style` 属性〔x, y, w, h の順で埋め込まれる、`selection_style`〕が
    // 全文検索に混入して名前順アサーションを誤検知させないため）。
    let table_start = heading_pos + heading.len();
    let table_end = html[table_start..]
        .find("</section>")
        .map(|rel| table_start + rel)
        .unwrap_or(html.len());
    let table_html = &html[table_start..table_end];

    let expected_order = [
        "--fandhe-image-cropper-h",
        "--fandhe-image-cropper-handle-size",
        "--fandhe-image-cropper-w",
        "--fandhe-image-cropper-x",
        "--fandhe-image-cropper-y",
    ];
    let mut last_pos = 0usize;
    for name in expected_order {
        let pos = table_html
            .find(name)
            .unwrap_or_else(|| panic!("`{name}` が CSS Variables 表に見つからない: {table_html}"));
        assert!(
            pos >= last_pos,
            "CSS Variables 表の名前順（BTreeMap 整列）が崩れている: `{name}` の出現位置が前の行より前"
        );
        last_pos = pos;
    }

    assert!(
        !html.contains("--fandhe-cropper-"),
        "旧名 `--fandhe-cropper-` が生成 HTML に残存している: {html}"
    );
}
