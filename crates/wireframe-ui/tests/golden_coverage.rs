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
/// - icon（表示部品、#2652。`icon` モジュール自体は #2606 の SVG
///   アイコン基盤として既に存在するが、`PARTS` へ登録された
///   `icon::ICON_GLYPH_CSS` はグリフ基底 class であり #2652 が追加する
///   表示部品ではない。ルートセレクタは定数名ではなく `.fw-wire-icon {`
///   の出現有無で判定する）
/// - brand（#2653）
/// - list（#2657）
/// - card-basic（#2658）
/// - media（#2661）
/// - table（#2662）
/// - map（#2664）
const PENDING: &[&str] = &[
    "icon",
    "brand",
    "list",
    "card-basic",
    "media",
    "table",
    "map",
];

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

fn all_golden_source() -> String {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut combined = String::new();
    for entry in fs::read_dir(&tests_dir).expect("tests/ ディレクトリの読み取りに失敗")
    {
        let entry = entry.expect("tests/ エントリの読み取りに失敗");
        let path = entry.path();
        let is_css_golden = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with("_css.rs"))
            .unwrap_or(false);
        if is_css_golden {
            combined.push_str(
                &fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("{} の読み込みに失敗: {e}", path.display())),
            );
            combined.push('\n');
        }
    }
    combined
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
    let golden_source = all_golden_source();

    let mut missing = Vec::new();
    for const_name in &consts {
        // `contains(const_name)` という素朴な部分文字列一致だと、
        // 例えば `TEXT_CSS` は `RICH_TEXT_CSS` の部分文字列であるため
        // `tests/text_css.rs` を削除しても `tests/rich_text_css.rs`
        // 内の `crate::rich_text::RICH_TEXT_CSS` 参照にヒットしてしまい
        // 欠落検知が fail-open になる（イシュー #2666 レビュー指摘）。
        // `::` 区切りを含めて照合し、`::TEXT_CSS` が `::RICH_TEXT_CSS`
        // の部分文字列にならないようにする。
        let needle = format!("::{const_name}");
        if !golden_source.contains(needle.as_str()) {
            missing.push(const_name.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "以下の CSS 定数が golden ファイル（tests/*_css.rs）に見つからない。\
         対応する `tests/<snake>_css.rs`（基盤 4 件は tests/base_css.rs）を追加すること: {missing:?}"
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
