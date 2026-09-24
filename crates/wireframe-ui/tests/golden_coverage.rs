//! `PARTS` ⇔ golden ファイル ⇔ §8 の 49 部品一覧、の 3 方向 fail-closed
//! カバレッジ検証。
//!
//! イシュー #2666 の中心的制約: 実装時点で `main` に存在しない部品の
//! golden はモジュールが無いため書けない。本テストは「今 `PARTS` に
//! 登録されている定数はすべて golden を持つ」ことと「§8 の 49 部品の
//! うち未マージのもの（[`PENDING`]）はまだ CSS を出力していない」こと
//! の両方を検証し、後続の部品 PR が golden を同梱し忘れた場合・
//! `PENDING` を外し忘れた場合の両方を検知する。
//!
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md`
//! を参照。

use std::fs;
use std::path::Path;

/// `docs/design/wireframe-ui-architecture.md` §8 が定める 49 部品一覧
/// （kebab, 子イシュー番号）。本表が §8 の表と一致することを (iii) が
/// 件数・重複なしで確認する。
const ALL_PARTS: [(&str, u32); 49] = [
    // Phase 1: レイアウト骨格
    ("frame", 2609),
    ("stack", 2610),
    ("grid", 2611),
    ("divider", 2612),
    // Phase 2: テキスト・注釈
    ("text", 2614),
    ("paragraph", 2615),
    ("rich-text", 2616),
    ("annotation", 2617),
    ("link", 2618),
    ("tag", 2619),
    // Phase 3: Forms A
    ("button", 2621),
    ("input", 2622),
    ("textarea", 2623),
    ("select", 2624),
    ("checkbox", 2625),
    ("radio", 2626),
    ("switch", 2627),
    ("slider", 2628),
    // Phase 4: Forms B
    ("question", 2630),
    ("ratings", 2631),
    ("calendar", 2632),
    ("file-drop", 2633),
    ("stepper", 2634),
    // Phase 5: Navigation
    ("nav-item", 2636),
    ("menu", 2637),
    ("tabs", 2638),
    ("breadcrumbs", 2639),
    ("pagination", 2640),
    ("accordion", 2641),
    ("cursor", 2642),
    // Phase 6: Overlay・Feedback
    ("tooltip", 2644),
    ("modal", 2645),
    ("alert", 2646),
    ("toast", 2647),
    ("progress", 2648),
    ("spinner", 2649),
    // Phase 7: Data display
    ("avatar", 2651),
    ("icon", 2652),
    ("brand", 2653),
    ("emoji", 2654),
    ("counter", 2655),
    ("stat", 2656),
    ("list", 2657),
    ("card-basic", 2658),
    // Phase 8: Media・Data
    ("image", 2660),
    ("media", 2661),
    ("table", 2662),
    ("chart", 2663),
    ("map", 2664),
];

/// §8 の 49 部品のうち、実装時点で `main` に未マージの部品（kebab）。
/// 各部品が `PARTS`（`wireframe_css()`）へ実体を持つようになったら、
/// この配列から削除し対応する `tests/<snake>_css.rs` golden を追加する
/// こと（(iv) がこの手順の漏れを fail-closed に検知する）。
///
/// イシュー #2666 実装完了時点（49/49 部品が main へマージ済み、golden も
/// 全件整備済み）のため空である。`icon`（表示部品、#2652）は `icon`
/// モジュール自体が #2606 の SVG アイコン基盤として先に存在していたが、
/// `PARTS` へ登録される `icon::ICON_CSS`（`.fw-wire-icon {` ルート
/// セレクタ）は基盤の `icon::ICON_GLYPH_CSS`（グリフ基底 class、
/// `tests/base_css.rs` が担当）とは別の定数であり、`tests/icon_css.rs`
/// が golden を持つ。
const PENDING: &[&str] = &[];

/// `src/css.rs` の `PARTS` 配列本体から `crate::<mod>::<CONST>` を機械
/// 抽出する。パース漏れによる fail-open を防ぐため、抽出件数が
/// 呼び出し側で `PARTS.len()` と一致することも確認させる。
fn extract_parts_consts(css_rs: &str) -> Vec<String> {
    const DECL: &str = "pub const PARTS: &[&str] = &[";
    let start = css_rs
        .find(DECL)
        .expect("`pub const PARTS` 宣言が src/css.rs に見つからない");
    // `DECL` 自体が `&[&str]` という `[` を含むため、配列本体は `DECL` の
    // 直後（宣言全体の末尾）から始まる。
    let rest = &css_rs[start + DECL.len()..];
    let end = rest
        .find("];")
        .expect("`PARTS` 配列の終端 `];` が見つからない");
    let body = &rest[..end];

    body.lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',').trim();
            if line.is_empty() || line.starts_with("//") {
                return None;
            }
            // 期待形式: `crate::<mod>::<CONST>`
            let const_name = line.rsplit("::").next().unwrap_or(line);
            Some(const_name.to_string())
        })
        .collect()
}

fn read_manifest_file(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} の読み込みに失敗: {e}", path.display()))
}

/// `tests/*_css.rs` の 1 ファイル分のソースから、「`#[ignore]` の付いて
/// いない `#[test]` 関数」の本体だけを構造的に抽出する。
///
/// コメント・rustdoc・`#[ignore]` 済みテスト中に定数名の文字列が現れても
/// カバレッジとして誤検知しないための最小限のパーサ（イシュー #2666
/// codex-review P1 指摘対応: `::CONST_NAME` の部分文字列検索だけでは、
/// 各追加ファイル先頭の rustdoc コメントにも同名参照があるため、テスト
/// 関数本体や `assert_eq!` を削除しても・`#[ignore]` を付けてもすり抜ける）。
///
/// golden ファイルのテスト関数本体は `assert_eq!` 呼び出しのみで CSS
/// 文字列リテラル（`{`/`}` を含む）を持たない規約（`EXPECTED_*` は関数外の
/// トップレベル `const`）であるため、素朴な波括弧の対応カウントで安全に
/// 関数本体を切り出せる。
fn active_test_fn_bodies(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut bodies = Vec::new();
    let mut pending_attrs: Vec<String> = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.starts_with("#[") {
            // 属性は `#[cfg_attr(\n    all(),\n    ignore\n)]` のように
            // 複数行へまたがり得る。`[`/`]` の対応が閉じるまで行をまたいで
            // 読み進めたうえで、行ごとに trim した断片を連結し、空白を
            // すべて除去した正規化済み 1 文字列として蓄積する（イシュー
            // #2666 codex-review P1 再指摘対応: 素朴な 1 行判定では
            // `#[cfg_attr(` 開始行だけを見て非 `#[ignore]`/`#[cfg(` 属性と
            // 誤判定し、複数行属性の迂回を見逃す）。
            let mut depth = 0i32;
            let mut started = false;
            let mut raw = String::new();
            let mut j = i;
            while j < lines.len() {
                for ch in lines[j].chars() {
                    match ch {
                        '[' => {
                            depth += 1;
                            started = true;
                        }
                        ']' => depth -= 1,
                        _ => {}
                    }
                }
                raw.push_str(lines[j].trim());
                j += 1;
                if started && depth <= 0 {
                    break;
                }
            }
            let normalized: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
            pending_attrs.push(normalized);
            i = j;
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            // 空行・コメント行は属性の蓄積を崩さない（属性直前の
            // doc コメント等を許容するため）。
            i += 1;
            continue;
        }
        if trimmed.starts_with("fn ") {
            // 許可リスト方式（反転判定）: `#[test]` 単体のみをアクティブと
            // 認め、それ以外の属性が 1 つでも付いていれば「非カバー」側へ
            // 倒す。`#[ignore]`・`#[cfg(...)]` はもちろん、`#[cfg_attr(...)]`・
            // `#[should_panic]`（意図的に不一致な `assert_eq!` を置いて
            // テスト自体は「成功」させつつ golden 比較を無効化する迂回）
            // ・将来追加される未知の属性も、個別に列挙して除外するのでは
            // なく「許可リストに無い属性は非アクティブ」という fail-closed
            // な既定へ倒すことで、列挙漏れによる迂回を構造的に防ぐ
            // （イシュー #2666 codex-review P1 再指摘対応）。
            let is_test = pending_attrs.iter().any(|a| a == "#[test]");
            let has_disallowed_attr = pending_attrs.iter().any(|a| a != "#[test]");
            pending_attrs.clear();

            let mut depth = 0i32;
            let mut started = false;
            let mut body = String::new();
            let mut j = i;
            while j < lines.len() {
                for ch in lines[j].chars() {
                    match ch {
                        '{' => {
                            depth += 1;
                            started = true;
                        }
                        '}' => depth -= 1,
                        _ => {}
                    }
                }
                if started {
                    body.push_str(lines[j]);
                    body.push('\n');
                }
                j += 1;
                if started && depth <= 0 {
                    break;
                }
            }

            if is_test && !has_disallowed_attr {
                bodies.push(body);
            }
            i = j;
            continue;
        }

        // 属性が付いていない他の項目（`const` 宣言等）に来たら蓄積をリセット。
        pending_attrs.clear();
        i += 1;
    }

    bodies
}

/// ソース全体（1 ファイル分）から、関数本体の外＝トップレベルで宣言され
/// ている `const EXPECTED_*: ...` の定数名を抽出する。
///
/// golden ファイルの規約では、比較対象の期待値は関数内ローカル変数では
/// なく関数外のトップレベル `const`（`EXPECTED_CSS` 等、`EXPECTED_` 接頭辞）
/// として書く（`tests/card_basic_css.rs` 等の実例を参照）。Rust の変数命名
/// 規約上ローカル変数は snake_case のため、行頭（インデントなし）から始ま
/// る `const`/`pub const` 宣言だけを対象にする単純な走査で、関数内の記述
/// と安全に区別できる。
///
/// 加えて、初期化子（`=` の右辺）が独立した文字列リテラル（`"..."` または
/// `r#"..."#`/`r"..."` 等の raw string）で**始まっている**ことも要求する。
/// これにより `const EXPECTED_CSS: &str = fandhe_frontend_wireframe_ui::
/// button::BUTTON_CSS;` のような、実装定数への単なる別名（識別子参照）を
/// 装った期待値は「独立した golden」として数えない（イシュー #2666
/// codex-review P1 再指摘対応）。golden ファイルの初期化子は本リポジトリの
/// 実例が示すとおり常に raw string リテラルであり、宣言と同じ行で開始する
/// （複数行にまたがる本体は許容し、閉じクォートの位置までは検証しない）。
fn top_level_expected_consts(source: &str) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for line in source.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // インデントされた行は関数本体内（トップレベルではない）。
            continue;
        }
        let rest = line
            .strip_prefix("pub const ")
            .or_else(|| line.strip_prefix("const "));
        let Some(rest) = rest else { continue };
        let Some((name, ty_and_init)) = rest.split_once(':') else {
            continue;
        };
        let name = name.trim();
        let is_expected_const = name.starts_with("EXPECTED_")
            && name
                .chars()
                .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit());
        if !is_expected_const {
            continue;
        }
        // `ty_and_init` は `&str = r#"..."#` のような型注釈 + 初期化子。
        // `=` の右辺（初期化子）だけを取り出し、識別子参照ではなく文字列
        // リテラルの開始トークンであることを確認する。
        let Some((_ty, init)) = ty_and_init.split_once('=') else {
            continue;
        };
        let init = init.trim_start();
        let is_string_literal = init.starts_with('"')
            || init.starts_with("r\"")
            || (init.starts_with("r#") && init[1..].trim_start_matches('#').starts_with('"'));
        if is_string_literal {
            set.insert(name.to_string());
        }
    }
    set
}

/// テスト関数本体から、`assert_eq!` の第 1 引数として実際に比較されている
/// `crate::<mod>::<CONST>` 形式の参照の `<CONST>` 部分だけを抽出する。
/// `fandhe_frontend_wireframe_ui::<mod>::css()` のような関数呼び出し（定数
/// 比較ではない）は `<CONST>` 相当部分が全大文字にならないため除外される。
///
/// 加えて、`assert_eq!` の**第 2 引数**が同ファイルのトップレベルに独立
/// 宣言された `EXPECTED_*` 定数（`expected_consts`）と一致することも要求
/// する。これにより `assert_eq!(crate::x::X_CSS, crate::x::X_CSS)` のような
/// 自己比較（比較相手が golden ではなく検証対象自身）はカバレッジとして
/// 数えない（イシュー #2666 codex-review P1 再指摘対応: 第 1 引数の形式
/// だけを見る判定は、実質的な golden 比較を伴わないテストも通してしまう）。
fn asserted_const_names(
    body: &str,
    expected_consts: &std::collections::HashSet<String>,
) -> Vec<String> {
    const MACRO: &str = "assert_eq!(";
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(pos) = rest.find(MACRO) {
        rest = &rest[pos + MACRO.len()..];
        let arg_end = rest.find(',').unwrap_or(rest.len());
        let first_arg = rest[..arg_end].trim();

        // 第 2 引数（golden 期待値側）を取り出す。任意の `message` 引数
        // （3 引数形式）が続く場合があるため、次の `,` または `)` の
        // いずれか早い方までを区切りとする。
        let after_first = &rest[arg_end.min(rest.len())..];
        let after_comma = after_first.strip_prefix(',').unwrap_or(after_first);
        let second_end = after_comma.find([',', ')']).unwrap_or(after_comma.len());
        let second_arg = after_comma[..second_end].trim();
        let is_independent_golden = expected_consts.contains(second_arg);

        if let Some(name) = first_arg.rsplit("::").next() {
            let is_const_like = !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit());
            if is_const_like && is_independent_golden {
                out.push(name.to_string());
            }
        }
    }
    out
}

/// `tests/*_css.rs` 全体を走査し、実際に `assert_eq!` で比較されている
/// 定数名の集合を返す（`active_test_fn_bodies` の構造的抽出を経由するため、
/// コメント中の参照・`#[ignore]`／`#[cfg(...)]` 済みテストは含まれず、
/// `top_level_expected_consts` によって比較相手が独立した golden 定数で
/// あることも要求される）。
fn all_asserted_consts() -> std::collections::HashSet<String> {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut set = std::collections::HashSet::new();
    for entry in fs::read_dir(&tests_dir).expect("tests/ ディレクトリの読み取りに失敗")
    {
        let entry = entry.expect("tests/ エントリの読み取りに失敗");
        let path = entry.path();
        let is_css_golden = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with("_css.rs"))
            .unwrap_or(false);
        if !is_css_golden {
            continue;
        }
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} の読み込みに失敗: {e}", path.display()));
        let expected_consts = top_level_expected_consts(&source);
        for body in active_test_fn_bodies(&source) {
            set.extend(asserted_const_names(&body, &expected_consts));
        }
    }
    set
}

#[test]
fn parts_extraction_matches_runtime_parts_len() {
    let css_rs = read_manifest_file("src/css.rs");
    let consts = extract_parts_consts(&css_rs);
    assert_eq!(
        consts.len(),
        fandhe_frontend_wireframe_ui::css::PARTS.len(),
        "src/css.rs から抽出した PARTS 定数の件数がランタイムの PARTS.len() と \
         不一致。extract_parts_consts のパースが崩れている可能性がある"
    );
}

#[test]
fn every_parts_const_has_a_golden_file() {
    let css_rs = read_manifest_file("src/css.rs");
    let consts = extract_parts_consts(&css_rs);
    // 単純な部分文字列検索（コメント・`#[ignore]` 済みテストも拾ってしまい
    // fail-open になる）ではなく、`#[ignore]` の付いていない `#[test]`
    // 関数本体内で実際に `assert_eq!` の第 1 引数として比較されている
    // 定数だけを構造的に集計する（イシュー #2666 codex-review P1 指摘対応）。
    let asserted = all_asserted_consts();

    let missing: Vec<&String> = consts
        .iter()
        .filter(|const_name| !asserted.contains(const_name.as_str()))
        .collect();

    assert!(
        missing.is_empty(),
        "以下の CSS 定数が golden ファイル（tests/*_css.rs）の `#[test]`（`#[ignore]` \
         なし）関数内で `assert_eq!` により実際に比較されていない。コメント中の参照・\
         `#[ignore]` 済みテストは検知対象外。対応する `tests/<snake>_css.rs`\
         （基盤 4 件は tests/base_css.rs）を追加すること: {missing:?}"
    );
}

#[test]
fn all_parts_table_has_49_unique_entries() {
    assert_eq!(
        ALL_PARTS.len(),
        49,
        "ALL_PARTS は §8 の 49 部品と一致しなければならない"
    );

    let mut kebabs: Vec<&str> = ALL_PARTS.iter().map(|(k, _)| *k).collect();
    kebabs.sort_unstable();
    kebabs.dedup();
    assert_eq!(
        kebabs.len(),
        49,
        "ALL_PARTS に kebab の重複がある（重複除去後の件数が 49 未満）"
    );
}

#[test]
fn pending_entries_are_subset_of_all_parts_and_not_yet_shipped() {
    let css = fandhe_frontend_wireframe_ui::wireframe_css();

    for kebab in PENDING {
        assert!(
            ALL_PARTS.iter().any(|(k, _)| k == kebab),
            "PENDING の `{kebab}` が ALL_PARTS（§8 の 49 部品）に存在しない"
        );

        let selector = format!(".fw-wire-{kebab} {{");
        assert!(
            !css.contains(&selector),
            "PENDING の `{kebab}` のルートセレクタ `{selector}` が wireframe_css() に \
             既に現れている。この部品は main にマージ済みなので、golden \
             （tests/<snake>_css.rs）を追加したうえで PENDING から削除すること"
        );
    }
}

#[test]
fn shipped_parts_expose_their_root_selector() {
    let css = fandhe_frontend_wireframe_ui::wireframe_css();

    for (kebab, issue) in ALL_PARTS {
        if PENDING.contains(&kebab) {
            continue;
        }
        let selector = format!(".fw-wire-{kebab} {{");
        assert!(
            css.contains(&selector),
            "カバー済みと扱われている `{kebab}`（#{issue}）のルートセレクタ \
             `{selector}` が wireframe_css() に見つからない。ALL_PARTS の \
             エントリが誤っているか、部品側の実装が変わった可能性がある"
        );
    }
}

#[test]
fn covered_count_matches_parts_minus_icon_glyph_base() {
    let covered = ALL_PARTS.len() - PENDING.len();
    let runtime_parts = fandhe_frontend_wireframe_ui::css::PARTS.len();
    // PARTS には基盤の ICON_GLYPH_CSS（§8 の「icon」部品そのものではない）
    // が 1 件含まれるため、部品としてのカバー数は PARTS.len() - 1。
    assert_eq!(
        covered,
        runtime_parts - 1,
        "ALL_PARTS - PENDING のカバー数（{covered}）が PARTS.len() - 1（{}）と \
         不一致。PENDING の更新漏れ、または ALL_PARTS の不整合の可能性がある",
        runtime_parts - 1
    );
}

/// カバレッジパーサ自体（`active_test_fn_bodies`/`asserted_const_names`/
/// `top_level_expected_consts`）の単体テスト。
///
/// イシュー #2666 codex-review の P1 再指摘（「有効な golden 比較がなくて
/// もカバレッジゲートを通過できる」）が挙げた 2 つの迂回パターンを、本物の
/// `tests/*_css.rs` ファイル群に埋め込まず、直接この最小再現ケースで検証
/// する（実ファイルへ迂回コードを混入させると golden カバレッジ自体を
/// 汚染するため）。
#[cfg(test)]
mod parser_self_test {
    use super::{active_test_fn_bodies, asserted_const_names, top_level_expected_consts};

    /// 正規の golden テスト（`EXPECTED_CSS` という独立定数との比較）は
    /// 引き続きカバレッジとして数えられることを確認する（回帰防止）。
    #[test]
    fn genuine_golden_comparison_is_counted() {
        let source = r#"
const EXPECTED_CSS: &str = "body {}";

#[test]
fn matches_golden() {
    assert_eq!(crate::button::BUTTON_CSS, EXPECTED_CSS);
}
"#;
        let expected = top_level_expected_consts(source);
        let bodies = active_test_fn_bodies(source);
        assert_eq!(bodies.len(), 1, "アクティブなテスト本体は 1 件のはず");
        let names = asserted_const_names(&bodies[0], &expected);
        assert_eq!(names, vec!["BUTTON_CSS".to_string()]);
    }

    /// codex-review 指摘の迂回パターン 1: `assert_eq!` の第 2 引数が
    /// トップレベルの独立した `EXPECTED_*` 定数ではなく検証対象自身の
    /// 自己比較（`assert_eq!(X, X)`）である場合はカバレッジとして数えない。
    #[test]
    fn self_comparison_without_independent_golden_is_not_counted() {
        let source = r#"
#[test]
fn fake_coverage() {
    assert_eq!(crate::button::BUTTON_CSS, crate::button::BUTTON_CSS);
}
"#;
        let expected = top_level_expected_consts(source);
        assert!(
            expected.is_empty(),
            "この迂回パターンはトップレベル EXPECTED_* 定数を持たない"
        );
        let bodies = active_test_fn_bodies(source);
        assert_eq!(bodies.len(), 1);
        let names = asserted_const_names(&bodies[0], &expected);
        assert!(
            names.is_empty(),
            "独立した golden 定数と比較していない自己比較はカバレッジに \
             数えてはならない"
        );
    }

    /// codex-review 指摘の迂回パターン 2: `#[cfg(any())]` 等の条件付き
    /// コンパイル属性が付いたテストはコンパイル対象外になり得るため、
    /// （`EXPECTED_CSS` との正規の比較を装っていても）アクティブなテスト
    /// 本体として抽出されない。
    #[test]
    fn cfg_gated_test_is_not_active() {
        let source = r#"
const EXPECTED_CSS: &str = "body {}";

#[test]
#[cfg(any())]
fn never_compiled() {
    assert_eq!(crate::button::BUTTON_CSS, EXPECTED_CSS);
}
"#;
        let bodies = active_test_fn_bodies(source);
        assert!(
            bodies.is_empty(),
            "#[cfg(any())] が付いたテストはアクティブなテスト本体として \
             抽出してはならない"
        );
    }

    /// `#[ignore]` 済みテストは従来どおり非アクティブ（既存契約の回帰確認）。
    #[test]
    fn ignored_test_is_not_active() {
        let source = r#"
const EXPECTED_CSS: &str = "body {}";

#[test]
#[ignore]
fn skipped() {
    assert_eq!(crate::button::BUTTON_CSS, EXPECTED_CSS);
}
"#;
        let bodies = active_test_fn_bodies(source);
        assert!(bodies.is_empty());
    }

    /// codex-review 指摘の再迂回パターン（`#[cfg_attr(...)]`）:
    /// `#[cfg_attr(all(), ignore)]` は `#[ignore]`/`#[cfg(` のどちらの
    /// 接頭辞にも一致しないため、素朴な接頭辞判定では見逃す。許可リスト
    /// 方式（`#[test]` 以外の属性は非アクティブ）でこれを閉じる。
    #[test]
    fn cfg_attr_ignore_test_is_not_active() {
        let source = r#"
const EXPECTED_CSS: &str = "body {}";

#[test]
#[cfg_attr(all(), ignore)]
fn evades_prefix_check() {
    assert_eq!(crate::button::BUTTON_CSS, EXPECTED_CSS);
}
"#;
        let bodies = active_test_fn_bodies(source);
        assert!(
            bodies.is_empty(),
            "#[cfg_attr(all(), ignore)] が付いたテストはアクティブな \
             テスト本体として抽出してはならない"
        );
    }

    /// codex-review 指摘の再迂回パターン（`#[should_panic]`）: 意図的に
    /// 不一致な `assert_eq!` を置いてもテスト実行自体は「成功」するため、
    /// ソーステキストだけを見る本パーサは `#[should_panic]` を明示的に
    /// 非アクティブ扱いしなければ迂回を許してしまう。
    #[test]
    fn should_panic_test_is_not_active() {
        let source = r#"
const EXPECTED_CSS: &str = "body {}";

#[test]
#[should_panic]
fn intentionally_mismatched() {
    assert_eq!(crate::button::BUTTON_CSS, "definitely not the golden value");
}
"#;
        let bodies = active_test_fn_bodies(source);
        assert!(
            bodies.is_empty(),
            "#[should_panic] が付いたテストはアクティブなテスト本体として \
             抽出してはならない"
        );
    }

    /// codex-review 指摘の再迂回パターン（複数行属性）: `#[cfg_attr(...)]`
    /// を複数行に折り返しても、`[`/`]` の対応を追跡して 1 個の属性として
    /// 認識し、許可リスト（`#[test]` 単体）から外れる属性として除外する。
    #[test]
    fn multiline_cfg_attr_ignore_test_is_not_active() {
        let source = r#"
const EXPECTED_CSS: &str = "body {}";

#[test]
#[cfg_attr(
    all(),
    ignore
)]
fn evades_multiline_check() {
    assert_eq!(crate::button::BUTTON_CSS, EXPECTED_CSS);
}
"#;
        let bodies = active_test_fn_bodies(source);
        assert!(
            bodies.is_empty(),
            "複数行に折り返された #[cfg_attr(...)] もアクティブなテスト \
             本体として抽出してはならない"
        );
    }

    /// 許可リスト方式の回帰防止: `#[test]` 単体（他の属性を伴わない）は
    /// 引き続きアクティブなテスト本体として抽出される。
    #[test]
    fn plain_test_attribute_alone_is_still_active() {
        let source = r#"
const EXPECTED_CSS: &str = "body {}";

#[test]
fn plain() {
    assert_eq!(crate::button::BUTTON_CSS, EXPECTED_CSS);
}
"#;
        let bodies = active_test_fn_bodies(source);
        assert_eq!(
            bodies.len(),
            1,
            "#[test] 単体のテストは引き続きアクティブなはず"
        );
    }

    /// codex-review 指摘の迂回パターン 3: `EXPECTED_*` という名前だけを
    /// 満たし、初期化子が実装定数への識別子参照（別名）になっている場合は
    /// 独立した golden 期待値として数えない（イシュー #2666 codex-review
    /// P1 再指摘対応）。
    #[test]
    fn alias_to_implementation_const_is_not_independent_golden() {
        let source = r#"
const EXPECTED_CSS: &str = fandhe_frontend_wireframe_ui::button::BUTTON_CSS;

#[test]
fn fake_coverage() {
    assert_eq!(crate::button::BUTTON_CSS, EXPECTED_CSS);
}
"#;
        let expected = top_level_expected_consts(source);
        assert!(
            expected.is_empty(),
            "初期化子が文字列リテラルでない EXPECTED_* は独立した golden \
             として収集してはならない"
        );
        let bodies = active_test_fn_bodies(source);
        assert_eq!(bodies.len(), 1);
        let names = asserted_const_names(&bodies[0], &expected);
        assert!(
            names.is_empty(),
            "実装定数への別名との比較はカバレッジに数えてはならない"
        );
    }

    /// raw string リテラル（`r#"..."#`）で書かれた golden 期待値は、
    /// 引き続き独立した golden として認識される（実運用の全 golden ファイル
    /// が採用する記法の回帰防止）。
    #[test]
    fn raw_string_literal_expected_const_is_recognized() {
        // 外側の Rust リテラルは `r##"..."##`（2 段ハッシュ）を使い、
        // 内側（golden ファイル側）の `r#"..."#`（1 段ハッシュ）が誤って
        // 外側の終端として解釈されないようにする。
        let source = r##"
const EXPECTED_CSS: &str = r#".fw-wire-button {
  display: inline-flex;
}
"#;
"##;
        let expected = top_level_expected_consts(source);
        assert!(
            expected.contains("EXPECTED_CSS"),
            "raw string リテラルで始まる EXPECTED_CSS は独立した golden \
             として認識されなければならない"
        );
    }
}
