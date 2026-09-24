//! wireframe-ui の全 golden CSS を「値ベースの全単射検証」で固定する
//! 単一の integration test crate。
//!
//! かつての `golden_coverage.rs` はソーステキストを走査するヒューリス
//! ティック（`assert_eq!` 呼び出しの構文的抽出）でカバレッジを判定して
//! いたが、コメント・文字列リテラル中に `assert_eq!(...)` らしき文字列
//! を書くだけでカバー済みと誤判定される迂回が、方式を強化するたびに
//! 繰り返し指摘された（イシュー #2666 codex-review、4 ラウンド）。
//!
//! 本モジュールはソース走査を一切行わない。各エントリの `actual` は
//! `fandhe_frontend_wireframe_ui::<mod>::<CONST>`（または `PARTS` を
//! 経由しない基盤生成関数）への通常の Rust 参照であり、コンパイラが
//! 名前解決する。存在しないシンボルへの参照やコメント中の偽装では
//! 成立し得ず、テキストパーサのヒューリスティックに依存しない。
//!
//! 検証する不変条件（イシュー #2666 の要求どおり）:
//!
//! 1. 各エントリで `actual`（実装が返す実際の CSS）と `expected`
//!    （`tests/golden/<part>.rs` の独立した期待値リテラル）がバイト
//!    一致する（[`each_golden_matches_its_expected_literal`]）。
//! 2. `PARTS` の要素（多重集合として）と、`in_parts` な golden
//!    エントリの `actual` 値（同じく多重集合として）が一致する
//!    （[`parts_values_and_in_parts_golden_actuals_are_the_same_multiset`]）。
//!    これにより「`PARTS` に登録されているのに golden が無い」
//!    （newly added part の golden 追加漏れ）と「golden はあるが
//!    `PARTS` に対応する要素が無い」の両方向を検知する（要求の (2)(3)
//!    を 1 つの多重集合比較として同時に満たす）。
//! 3. `wireframe_css()` の連結順契約は既存の `wireframe_css_assembly.rs`
//!    が独立に検証する（本ファイルでは扱わない、変更なし）。
//!
//! 更新手順は `docs/internal/wireframe-ui-golden-test-update-guide.md`
//! を参照。

mod golden;

use fandhe_frontend_wireframe_ui as w;

/// 1 golden エントリ。
struct GoldenEntry {
    /// 表示名（診断メッセージ用。`tests/golden/<name>.rs` のモジュール名
    /// と一致させる）。
    name: &'static str,
    /// 実装が返す実際の CSS 文字列。`PARTS` 登録済みの `&'static str`
    /// 定数は `.to_string()` で複製し、`size::SCALE` 等から実行時に
    /// 導出する関数（`tokens::css()` 等）は戻り値をそのまま使う。
    actual: String,
    /// `tests/golden/<name>.rs` の独立した期待値リテラル。
    expected: &'static str,
    /// `actual` が `fandhe_frontend_wireframe_ui::css::PARTS` の要素
    /// 由来であれば `true`。`tokens::css()`/`size::css()`/
    /// `frame::frame_padding_css()` の 3 件は `PARTS` を経由しない
    /// 基盤生成 CSS のため `false`。
    in_parts: bool,
}

/// `PARTS` を経由しない基盤生成 CSS の件数（`tokens::css()` /
/// `size::css()` / `frame::frame_padding_css()`）。
const BASE_FN_GOLDEN_COUNT: usize = 3;

/// 全 golden エントリを構築する。
fn goldens() -> Vec<GoldenEntry> {
    vec![
        GoldenEntry {
            name: "tokens",
            actual: w::tokens::css(),
            expected: golden::tokens::EXPECTED_CSS,
            in_parts: false,
        },
        GoldenEntry {
            name: "size",
            actual: w::size::css(),
            expected: golden::size::EXPECTED_CSS,
            in_parts: false,
        },
        GoldenEntry {
            name: "frame_padding",
            actual: w::frame::frame_padding_css(),
            expected: golden::frame_padding::EXPECTED_CSS,
            in_parts: false,
        },
        GoldenEntry {
            name: "icon_glyph",
            actual: w::icon::ICON_GLYPH_CSS.to_string(),
            expected: golden::icon_glyph::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "accordion",
            actual: w::accordion::ACCORDION_CSS.to_string(),
            expected: golden::accordion::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "alert",
            actual: w::alert::ALERT_CSS.to_string(),
            expected: golden::alert::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "annotation",
            actual: w::annotation::ANNOTATION_CSS.to_string(),
            expected: golden::annotation::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "avatar",
            actual: w::avatar::AVATAR_CSS.to_string(),
            expected: golden::avatar::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "brand",
            actual: w::brand::BRAND_CSS.to_string(),
            expected: golden::brand::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "breadcrumbs",
            actual: w::breadcrumbs::BREADCRUMBS_CSS.to_string(),
            expected: golden::breadcrumbs::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "button",
            actual: w::button::BUTTON_CSS.to_string(),
            expected: golden::button::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "calendar",
            actual: w::calendar::CALENDAR_CSS.to_string(),
            expected: golden::calendar::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "card_basic",
            actual: w::card_basic::CARD_BASIC_CSS.to_string(),
            expected: golden::card_basic::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "chart",
            actual: w::chart::CHART_CSS.to_string(),
            expected: golden::chart::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "checkbox",
            actual: w::checkbox::CHECKBOX_CSS.to_string(),
            expected: golden::checkbox::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "counter",
            actual: w::counter::COUNTER_CSS.to_string(),
            expected: golden::counter::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "cursor",
            actual: w::cursor::CURSOR_CSS.to_string(),
            expected: golden::cursor::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "divider",
            actual: w::divider::DIVIDER_CSS.to_string(),
            expected: golden::divider::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "emoji",
            actual: w::emoji::EMOJI_CSS.to_string(),
            expected: golden::emoji::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "file_drop",
            actual: w::file_drop::FILE_DROP_CSS.to_string(),
            expected: golden::file_drop::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "frame",
            actual: w::frame::FRAME_CSS.to_string(),
            expected: golden::frame::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "grid",
            actual: w::grid::GRID_CSS.to_string(),
            expected: golden::grid::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "icon",
            actual: w::icon::ICON_CSS.to_string(),
            expected: golden::icon::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "image",
            actual: w::image::IMAGE_CSS.to_string(),
            expected: golden::image::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "input",
            actual: w::input::INPUT_CSS.to_string(),
            expected: golden::input::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "link",
            actual: w::link::LINK_CSS.to_string(),
            expected: golden::link::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "list",
            actual: w::list::LIST_CSS.to_string(),
            expected: golden::list::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "map",
            actual: w::map::MAP_CSS.to_string(),
            expected: golden::map::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "media",
            actual: w::media::MEDIA_CSS.to_string(),
            expected: golden::media::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "menu",
            actual: w::menu::MENU_CSS.to_string(),
            expected: golden::menu::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "modal",
            actual: w::modal::MODAL_CSS.to_string(),
            expected: golden::modal::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "nav_item",
            actual: w::nav_item::NAV_ITEM_CSS.to_string(),
            expected: golden::nav_item::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "pagination",
            actual: w::pagination::PAGINATION_CSS.to_string(),
            expected: golden::pagination::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "paragraph",
            actual: w::paragraph::PARAGRAPH_CSS.to_string(),
            expected: golden::paragraph::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "progress",
            actual: w::progress::PROGRESS_CSS.to_string(),
            expected: golden::progress::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "question",
            actual: w::question::QUESTION_CSS.to_string(),
            expected: golden::question::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "radio",
            actual: w::radio::RADIO_CSS.to_string(),
            expected: golden::radio::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "ratings",
            actual: w::ratings::RATINGS_CSS.to_string(),
            expected: golden::ratings::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "rich_text",
            actual: w::rich_text::RICH_TEXT_CSS.to_string(),
            expected: golden::rich_text::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "select",
            actual: w::select::SELECT_CSS.to_string(),
            expected: golden::select::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "slider",
            actual: w::slider::SLIDER_CSS.to_string(),
            expected: golden::slider::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "spinner",
            actual: w::spinner::SPINNER_CSS.to_string(),
            expected: golden::spinner::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "stack",
            actual: w::stack::STACK_CSS.to_string(),
            expected: golden::stack::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "stat",
            actual: w::stat::STAT_CSS.to_string(),
            expected: golden::stat::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "stepper",
            actual: w::stepper::STEPPER_CSS.to_string(),
            expected: golden::stepper::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "switch",
            actual: w::switch::SWITCH_CSS.to_string(),
            expected: golden::switch::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "table",
            actual: w::table::TABLE_CSS.to_string(),
            expected: golden::table::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "tabs",
            actual: w::tabs::TABS_CSS.to_string(),
            expected: golden::tabs::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "tag",
            actual: w::tag::TAG_CSS.to_string(),
            expected: golden::tag::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "text",
            actual: w::text::TEXT_CSS.to_string(),
            expected: golden::text::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "textarea",
            actual: w::textarea::TEXTAREA_CSS.to_string(),
            expected: golden::textarea::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "toast",
            actual: w::toast::TOAST_CSS.to_string(),
            expected: golden::toast::EXPECTED_CSS,
            in_parts: true,
        },
        GoldenEntry {
            name: "tooltip",
            actual: w::tooltip::TOOLTIP_CSS.to_string(),
            expected: golden::tooltip::EXPECTED_CSS,
            in_parts: true,
        },
    ]
}

#[test]
fn golden_names_are_unique() {
    let gs = goldens();
    let mut names: Vec<&str> = gs.iter().map(|g| g.name).collect();
    let before = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(
        names.len(),
        before,
        "GOLDENS（goldens()）のエントリ名に重複がある"
    );
}

#[test]
fn golden_count_matches_parts_and_base_functions() {
    let expected = w::css::PARTS.len() + BASE_FN_GOLDEN_COUNT;
    assert_eq!(
        goldens().len(),
        expected,
        "goldens() の件数が `PARTS.len() ({}) + 基盤生成 CSS ({BASE_FN_GOLDEN_COUNT})` \
         と不一致。部品追加時は golden の追加・削除時は goldens() からの除去を \
         忘れずに行うこと",
        w::css::PARTS.len()
    );
}

#[test]
fn each_golden_matches_its_expected_literal() {
    let mismatched: Vec<&'static str> = goldens()
        .into_iter()
        .filter(|g| g.actual != g.expected)
        .map(|g| g.name)
        .collect();
    assert!(
        mismatched.is_empty(),
        "以下の golden エントリで実装の実際の出力と tests/golden/<part>.rs の \
         期待値リテラルが一致しない（意図した変更であれば golden を更新する。\
         `docs/internal/wireframe-ui-golden-test-update-guide.md` 参照）: \
         {mismatched:?}"
    );
}

/// PARTS の値の多重集合と、`in_parts` な golden エントリの `actual` 値の
/// 多重集合が一致することを検証する（純粋関数化して単体テストからも
/// 再利用できるようにしてある。[`bijection_self_test`] 参照）。
fn check_value_multiset_bijection(
    parts_values: &[&str],
    golden_actuals: &[&str],
) -> Result<(), String> {
    let mut a: Vec<&str> = parts_values.to_vec();
    a.sort_unstable();
    let mut b: Vec<&str> = golden_actuals.to_vec();
    b.sort_unstable();
    if a == b {
        Ok(())
    } else {
        let only_in_parts: Vec<&&str> = a.iter().filter(|v| !b.contains(v)).collect();
        let only_in_golden: Vec<&&str> = b.iter().filter(|v| !a.contains(v)).collect();
        Err(format!(
            "PARTS 側にしかない値の件数={}（例: {:?}）、golden 側にしかない値の \
             件数={}（例: {:?}）",
            only_in_parts.len(),
            only_in_parts.first(),
            only_in_golden.len(),
            only_in_golden.first()
        ))
    }
}

#[test]
fn parts_values_and_in_parts_golden_actuals_are_the_same_multiset() {
    let parts_values: Vec<&str> = w::css::PARTS.to_vec();
    let gs = goldens();
    let golden_actuals: Vec<&str> = gs
        .iter()
        .filter(|g| g.in_parts)
        .map(|g| g.actual.as_str())
        .collect();

    assert_eq!(
        golden_actuals.len(),
        parts_values.len(),
        "in_parts な golden エントリの件数（{}）が PARTS.len()（{}）と不一致。\
         部品を PARTS へ登録したら対応する golden エントリ（in_parts: true）\
         も追加すること",
        golden_actuals.len(),
        parts_values.len()
    );

    check_value_multiset_bijection(&parts_values, &golden_actuals).unwrap_or_else(|e| {
        panic!(
            "PARTS の値と in_parts golden エントリの actual 値が値として \
             一対一対応していない（golden の追加漏れ、または PARTS に \
             対応しない不要な in_parts エントリが残っている）: {e}"
        )
    });
}

// 「golden ファイル側で実装定数を再エクスポートするだけの別名」
// （`const EXPECTED_CSS: &str = fandhe_frontend_wireframe_ui::button::
// BUTTON_CSS;` のような、`expected` が独立した文字列リテラルではなく
// 実装定数への単なる参照になっている迂回）を `ptr::eq` で検出する案を
// 実装・実測したが、**採用を見送った**（コーディネータ指示に基づく
// 明記）。
//
// 実測（本ファイルの初期実装時点、49 部品 + 基盤 1 件の全 `in_parts`
// エントリで検証）の結果、`tests/golden/<part>.rs` の独立した文字列
// リテラルと実装側の `pub const <PART>_CSS: &str` は、たとえ両者が
// 別ファイルに書かれた完全に独立したソースコードであっても、バイト列
// が一致する限り常に `ptr::eq` が真になった。これは rustc がコンパイル
// 時（LLVM の最適化パスより前、const 評価・インターン処理の段階）に
// 同一バイト列の `&'static str` リテラルを単一のアロケーションへ
// 統合するためであり、リンク時最適化やビルドプロファイルの設定に
// 依存しない。したがって `ptr::eq` は「値が独立している golden」と
// 「実装定数への別名」を区別する手段として構造的に機能しない
// （両者は golden テストの性質上バイト列が一致するのが正常な状態
// であり、その一致自体が偽陽性の原因になる）。
//
// 「実装定数を期待値として使う自己比較」は、[`GoldenEntry::actual`] と
// `expected` を必ず 2 つの異なる Rust アイテム（実装クレート側の
// `pub const` とテスト側 `tests/golden/<part>.rs` の `pub const`）から
// 独立に取得する本ファイルの構造そのものによって、`assert_eq!(X, X)`
// のような**式レベルの**自己比較（旧 `golden_coverage.rs` が対象と
// していた迂回）は既に構造的に成立しない。golden ファイル側が実装定数
// への `use` 再エクスポートを書く残余リスクはコードレビューに委ねる
// （`tests/golden/<part>.rs` は `pub const EXPECTED_CSS: &str = r#"..."#;`
// 1 行のみを持つ規約とし、`use fandhe_frontend_wireframe_ui` 等の import
// を書かないことを更新手順書で明示する）。

/// [`check_value_multiset_bijection`] 自体の単体テスト。イシュー #2666
/// の要求（「新しく部品を足したときに golden を足し忘れると FAIL する
/// ことを、ローカルの偽 PARTS 配列に対して同じ検証関数を当てて示す」）
/// に対応する。
#[cfg(test)]
mod bijection_self_test {
    use super::check_value_multiset_bijection;

    #[test]
    fn passes_when_value_multisets_match() {
        let parts = ["a", "b", "c"];
        let golden = ["c", "a", "b"];
        assert!(check_value_multiset_bijection(&parts, &golden).is_ok());
    }

    #[test]
    fn passes_when_duplicate_values_appear_on_both_sides_with_matching_multiplicity() {
        let parts = ["a", "a", "b"];
        let golden = ["b", "a", "a"];
        assert!(check_value_multiset_bijection(&parts, &golden).is_ok());
    }

    /// 新しく部品を `PARTS`（ここでは偽の配列）へ追加したのに golden の
    /// 追加を忘れた場合を再現する。golden 側に対応する値が無いため
    /// FAIL しなければならない。
    #[test]
    fn fails_when_a_newly_added_part_has_no_matching_golden() {
        let parts = ["a", "b", "c"]; // "c" が golden 未整備のまま追加された新部品を模す
        let golden = ["a", "b"];
        assert!(check_value_multiset_bijection(&parts, &golden).is_err());
    }

    /// golden 側に `PARTS` から削除された（または対応しない）値が残った
    /// 場合を再現する。
    #[test]
    fn fails_when_golden_has_an_entry_with_no_matching_part() {
        let parts = ["a", "b"];
        let golden = ["a", "b", "c"];
        assert!(check_value_multiset_bijection(&parts, &golden).is_err());
    }

    #[test]
    fn fails_when_a_value_is_reused_with_different_multiplicity() {
        let parts = ["a", "a", "b"];
        let golden = ["a", "b", "b"];
        assert!(check_value_multiset_bijection(&parts, &golden).is_err());
    }
}
