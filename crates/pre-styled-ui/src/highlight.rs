//! Highlight（イシュー #775）: 単一 recipe styled 静的部品。テキスト中の
//! 一致語句を `<mark>` で強調する（ark-ui `utilities/highlight.md` /
//! chakra-ui `typography/highlight.md` 相当）。skeleton（#764）/
//! separator（#772）と同型の、headless 状態機械を要しない静的部品
//! （`docs/design/component-coverage-map.md` highlight 行、2 件）。
//!
//! # 一致判定は決定的な文字列検索のみ（ReDoS 対策・REQ-1 隣接）
//!
//! `query` はユーザー入力由来の**素朴な部分文字列**としてのみ扱い、正規表現
//! としては解釈しない（本文中への埋め込みはすべて [`fandhe_frontend_core::text`]
//! 経由で既定エスケープを通し、`query` 自体も一致箇所以外は決して HTML へ
//! 混入しない）。走査は最悪計算量 O(text バイト長 × query 数 × query バイト長)
//! の線形探索であり、入力依存で指数的に悪化する経路を持たない
//! （`.claude/rules/security.md` A05 系）。
//!
//! アルゴリズムは以下のとおり決定的に固定する:
//! 1. `query` から空文字列要素を除外する（無限ループ防止。空クエリはどの
//!    位置にも「一致」してしまい走査が進まなくなるため）。
//! 2. 現在の走査位置から各クエリを素朴な部分文字列検索で探し、開始位置が
//!    最も小さい一致（最左一致）を採用する。同一開始位置に複数クエリが
//!    一致する場合は**最長のクエリ**を優先し、なお同点なら `query` 配列の
//!    先頭側を優先する（すべて入力順序のみに依存する決定的なタイブレーク）。
//! 3. `match_all: false`（既定）は最初の 1 件を見つけた時点で走査を打ち切る。
//!
//! # `ignore_case` は ASCII 限定（Unicode ケースフォールディング非対応）
//!
//! 大文字小文字を無視する一致判定は [`str::eq_ignore_ascii_case`] による
//! バイト単位比較のみで行う。Unicode 全体のケースフォールディング（例:
//! ドイツ語 `ß`/`SS`）はバイト長が変化しうり、一致長がクエリのバイト長と
//! 一致するという本モジュールの座標計算の前提を壊すため意図的に非対応と
//! する。バイト単位比較でも、一致長を常にクエリのバイト長に固定し
//! 非 ASCII バイトは完全一致のみ許容するため、UTF-8 の文字境界は自動的に
//! 保たれる（マルチバイト文字の途中で分割された不正な部分列が一致として
//! 採用されることはない）。
//!
//! `mark` 内のテキストは走査で見つかった**原文**（大文字小文字を保持した
//! まま）をそのまま [`fandhe_frontend_core::text`] へ渡す。`ignore_case` は
//! 一致判定のみに影響し、出力される文字列を変形しない。
//!
//! # イシュー #1435 の参照サイト比較（7 軸チェック）
//!
//! chakra-ui（`typography/highlight.md`）とスクリーンショット比較した結果を
//! 記録する（Radix Themes に Highlight 相当なし、ark-ui は headless
//! utility のみで独自スタイルを持たない）。chakra Highlight 自体は
//! `styles` prop による自由な CSS-in-JS 指定でプリセット variant 体系を
//! 持たないため、任意 CSS 注入面を持ち込まず既存語彙（[`crate::mark`]、
//! イシュー #1711 で 6 役割 palette 化済み）のプリセットへ写像する。
//!
//! - **サイズ**: 軸を新設しない（現状維持）。
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4(c)
//!   が本部品を「size 軸を持たない Typography 周辺部品」と規約確定済み。
//! - **バリアント**: [`HighlightVariant`]（`Subtle`〔既定〕/`Solid`/`Text`/
//!   `Plain`）を新設した。[`crate::mark::MarkVariant`] と同一語彙であり、
//!   chakra の淡色背景・濃色背景・文字装飾のみの 3 用例を既存語彙の
//!   プリセットで写像する。
//! - **色**: [`ColorPalette`] 軸（6 値、既定 `Accent`）を新設した。
//!   [`crate::recipe::palette_scale_declarations`] 経由でトークン参照のみ
//!   （生色リテラルなし）。
//! - **`data-*` 状態**: 変更なし。headless 状態機械を持たない静的部品の
//!   ため `data-scope`/`data-part` のみを維持する。
//! - **ダーク**: 全宣言が `--fandhe-*` トークン参照のみのため
//!   `write_dark_declarations` の一元機構に自動追従する。
//! - **フォーカス / hover / disabled / transition**: 適用しない
//!   （意図的）。非インタラクティブな表示専用部品であり、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md`
//!   （hover はインタラクティブ slot のみ）・
//!   `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
//!   （フォーカスリングはフォーカス対象部品のみ）のいずれの適用対象にも
//!   当たらない（[`crate::code`] #1432 と同一判断）。
//! - **余白・角丸**: `padding-inline` を [`crate::mark`] と同じ `0.25em`
//!   （chakra `px: 0.5` 相当の視覚量）へ統一した。角丸は
//!   `--fandhe-radius-sm` を維持する。

use fandhe_frontend_headless_ui::fandhe_frontend_core::{text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{palette_scale_declarations, ColorPalette, SlotRecipe, VariantValue};

/// `data-scope="highlight"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("highlight");

/// Highlight の見た目 variant（[`crate::mark::MarkVariant`] と同一語彙。
/// モジュール冒頭 rustdoc「イシュー #1435 の参照サイト比較」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HighlightVariant {
    /// 淡色背景（既定）。
    #[default]
    Subtle,
    /// 塗りつぶし。
    Solid,
    /// 背景なし・文字色のみ。
    Text,
    /// 装飾なし（旧実装相当の中立表示）。
    Plain,
}

impl VariantValue for HighlightVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Subtle => "subtle",
            Self::Solid => "solid",
            Self::Text => "text",
            Self::Plain => "plain",
        }
    }
}

/// [`highlight`] の設定。
///
/// 既定値（`#[derive(Default)]`）は `query: &[]`（一致対象なし）・
/// `ignore_case: false`・`match_all: false`（ark-ui 既定 `matchAll: false`
/// に合わせる）・`variant: HighlightVariant::Subtle`・
/// `palette: ColorPalette::Accent`。
#[derive(Debug, Clone, Copy, Default)]
pub struct HighlightProps<'a> {
    /// 強調する語句（複数可）。空文字列の要素は無視する（無限ループ防止、
    /// モジュール冒頭 rustdoc 参照）。正規表現は受け付けない
    /// （素朴な部分文字列一致のみ）。
    pub query: &'a [&'a str],
    /// 大文字小文字を区別しない一致（ASCII の範囲のみ、既定 `false`。
    /// モジュール冒頭 rustdoc「`ignore_case` は ASCII 限定」節参照）。
    pub ignore_case: bool,
    /// `true` なら全一致箇所、`false`（既定）なら最初の 1 箇所のみ強調する
    /// （ark-ui 既定 `matchAll: false` に合わせる）。
    pub match_all: bool,
    /// 見た目 variant（既定 `Subtle`）。モジュール冒頭 rustdoc「イシュー
    /// #1435 の参照サイト比較」節参照。
    pub variant: HighlightVariant,
    /// colorPalette 軸（既定 `Accent`）。[`crate::theme`] のセマンティック色
    /// から選択する。
    pub palette: ColorPalette,
}

/// Highlight の recipe（scope `"highlight"`、slot `"root"`/`"mark"`）。
///
/// `root` は素通しのコンテナのため規則を持たない。`mark` slot へ
/// variant/palette の宣言を登録する（[`crate::mark::recipe`] と同型）。
fn recipe() -> SlotRecipe {
    let mut recipe = SlotRecipe::new("highlight", &["root", "mark"])
        .base(
            "mark",
            vec![
                decl("padding-inline", "0.25em"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
            ],
        )
        .variant(
            HighlightVariant::Subtle,
            "mark",
            vec![
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("color", "var(--fandhe-palette)"),
            ],
        )
        .variant(
            HighlightVariant::Solid,
            "mark",
            vec![
                decl("background", "var(--fandhe-palette)"),
                decl("color", "var(--fandhe-palette-fg)"),
            ],
        )
        .variant(
            HighlightVariant::Text,
            "mark",
            vec![
                decl("background", "transparent"),
                decl("color", "var(--fandhe-palette)"),
            ],
        )
        .variant(
            HighlightVariant::Plain,
            "mark",
            vec![
                decl("background", "transparent"),
                decl("color", "inherit"),
                decl("padding-inline", "0"),
                decl("border-radius", "0"),
            ],
        )
        .default_variant(HighlightVariant::Subtle)
        .default_variant(ColorPalette::Accent);

    for palette in [
        ColorPalette::Accent,
        ColorPalette::Info,
        ColorPalette::Success,
        ColorPalette::Warning,
        ColorPalette::Danger,
        ColorPalette::Neutral,
    ] {
        recipe = recipe.variant(palette, "mark", palette_scale_declarations(palette));
    }
    recipe
}

/// Highlight の静的 CSS 全文。`root` は素通しのコンテナのため規則を持たず、
/// `mark` の淡色強調のみを出力する。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// 1 回分の一致（テキスト中のバイト範囲）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Match {
    start: usize,
    end: usize,
}

/// `text[from..]` の範囲から、モジュール冒頭 rustdoc のタイブレーク規則
/// （最左一致 → 最長クエリ優先 → `queries` 先頭優先）に従って次の一致を
/// 1 件探す。`queries` に空文字列は含まれない前提（呼び出し元
/// [`find_matches`] が事前に除外する）。
fn find_next_match(text: &str, from: usize, queries: &[&str], ignore_case: bool) -> Option<Match> {
    let mut best: Option<Match> = None;

    for &q in queries {
        let q_len = q.len();
        if q_len == 0 || from + q_len > text.len() {
            continue;
        }
        // 素朴な部分文字列検索: from 以降の各開始位置でクエリと同じ長さの
        // 窓を切り出し比較する。正規表現エンジンを使わないため ReDoS の
        // 経路を構造的に持たない（モジュール冒頭 rustdoc 参照）。
        // `text.get(..)`（バイト範囲の char 境界チェック付き）を使い、
        // マルチバイト文字の途中に落ちる開始位置は `None` としてスキップ
        // する（`&text[..]` 直接インデックスは境界違反で panic するため
        // 使わない。非 ASCII 本文回帰テスト参照）。
        let mut start = from;
        while start + q_len <= text.len() {
            let matched = text.get(start..start + q_len).is_some_and(|window| {
                if ignore_case {
                    window.eq_ignore_ascii_case(q)
                } else {
                    window == q
                }
            });
            if matched {
                let candidate = Match {
                    start,
                    end: start + q_len,
                };
                best = Some(match best {
                    None => candidate,
                    Some(current) => {
                        if candidate.start < current.start {
                            candidate
                        } else if candidate.start == current.start && candidate.end > current.end {
                            // 同一開始位置なら最長クエリを優先する
                            // （タイブレーク規則）。`queries` の先頭優先は
                            // 外側ループの走査順（先に見つかったクエリが
                            // `best` に残る、`>` であって `>=` でないこと）
                            // で担保する。
                            candidate
                        } else {
                            current
                        }
                    }
                });
                break;
            }
            start += 1;
        }
    }

    best
}

/// `text` 全体を走査し、[`HighlightProps`] の規則に従って一致区間の一覧を
/// 決定的に求める。`match_all: false` なら最初の 1 件で打ち切る
/// （モジュール冒頭 rustdoc 参照）。
fn find_matches(text: &str, props: &HighlightProps<'_>) -> Vec<Match> {
    let queries: Vec<&str> = props
        .query
        .iter()
        .copied()
        .filter(|q| !q.is_empty())
        .collect();
    if queries.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        match find_next_match(text, cursor, &queries, props.ignore_case) {
            Some(m) => {
                cursor = m.end;
                matches.push(m);
                if !props.match_all {
                    break;
                }
            }
            None => break,
        }
    }
    matches
}

/// Highlight 1 個を組み立てる。
///
/// `text` を非一致区間の [`fandhe_frontend_core::text`] ノードと、一致区間の
/// `<mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">` ノード（子は同じく
/// `text()`）へ交互に分割する。両方とも既定エスケープ経由でのみ HTML へ
/// 出力するため、`text`/`query` のどちらにペイロードを含めても
/// `raw_html()` を経由しない限り実タグとして解釈されない（REQ-1）。
///
/// 呼び出し側 `attrs` の `class`/`data-scope`/`data-part`（大文字小文字を
/// 無視）は除去してから root へ合成する（separator #772 と同型の契約属性
/// 偽装防止）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::highlight::{highlight, HighlightProps};
///
/// let html = render(&highlight(
///     &HighlightProps {
///         query: &["fox"],
///         ..HighlightProps::default()
///     },
///     vec![],
///     "The quick brown fox",
/// ));
/// assert!(html.contains(r#"<mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">fox</mark>"#));
/// ```
#[must_use]
pub fn highlight<'a>(
    props: &HighlightProps<'_>,
    attrs: Vec<(&'a str, &'a str)>,
    text_content: &str,
) -> Node {
    let matches = find_matches(text_content, props);

    let recipe = recipe();
    let mark_class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("color-palette", props.palette.value()),
    ]);

    let mut children: Vec<Node> = Vec::with_capacity(matches.len() * 2 + 1);
    let mut cursor = 0usize;
    for m in &matches {
        if m.start > cursor {
            children.push(text(&text_content[cursor..m.start]));
        }
        children.push(ANATOMY.part(
            "mark",
            "mark",
            vec![("class", mark_class.as_str())],
            vec![text(&text_content[m.start..m.end])],
        ));
        cursor = m.end;
    }
    if cursor < text_content.len() {
        children.push(text(&text_content[cursor..]));
    }

    let contract_keys = ["data-scope", "data-part"];
    let attrs: Vec<(&str, &str)> = drop_class_attr(attrs)
        .into_iter()
        .filter(|(k, _)| !contract_keys.iter().any(|c| k.eq_ignore_ascii_case(c)))
        .collect();
    ANATOMY.part("root", "span", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn single_match_wraps_query_in_mark() {
        let html = render(&highlight(
            &HighlightProps {
                query: &["fox"],
                ..HighlightProps::default()
            },
            vec![],
            "The quick brown fox",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root">The quick brown <mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">fox</mark></span>"#
        );
    }

    #[test]
    fn no_match_renders_plain_text_only() {
        let html = render(&highlight(
            &HighlightProps {
                query: &["zzz"],
                ..HighlightProps::default()
            },
            vec![],
            "The quick brown fox",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root">The quick brown fox</span>"#
        );
        assert!(!html.contains("<mark"));
    }

    #[test]
    fn empty_query_list_does_not_panic_and_renders_no_mark() {
        let html = render(&highlight(
            &HighlightProps::default(),
            vec![],
            "hello world",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root">hello world</span>"#
        );
    }

    #[test]
    fn query_with_only_empty_string_elements_does_not_panic() {
        let html = render(&highlight(
            &HighlightProps {
                query: &["", ""],
                ..HighlightProps::default()
            },
            vec![],
            "hello world",
        ));
        assert!(!html.contains("<mark"));
    }

    #[test]
    fn match_all_true_highlights_every_occurrence() {
        let html = render(&highlight(
            &HighlightProps {
                query: &["o"],
                match_all: true,
                ..HighlightProps::default()
            },
            vec![],
            "foo boo",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root">f<mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">o</mark><mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">o</mark> b<mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">o</mark><mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">o</mark></span>"#
        );
    }

    #[test]
    fn match_all_false_highlights_only_first_occurrence() {
        let html = render(&highlight(
            &HighlightProps {
                query: &["o"],
                match_all: false,
                ..HighlightProps::default()
            },
            vec![],
            "foo boo",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root">f<mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">o</mark>o boo</span>"#
        );
    }

    #[test]
    fn overlapping_queries_prefer_longest_at_same_start_position() {
        // "brown" と "brow" が同じ開始位置で一致する場合、最長の "brown" を採用する。
        let html = render(&highlight(
            &HighlightProps {
                query: &["brow", "brown"],
                ..HighlightProps::default()
            },
            vec![],
            "the brown fox",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root">the <mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">brown</mark> fox</span>"#
        );
    }

    #[test]
    fn ignore_case_false_does_not_match_different_case() {
        let html = render(&highlight(
            &HighlightProps {
                query: &["FOX"],
                ignore_case: false,
                ..HighlightProps::default()
            },
            vec![],
            "the fox",
        ));
        assert!(!html.contains("<mark"));
    }

    #[test]
    fn ignore_case_true_matches_and_preserves_original_casing_in_output() {
        let html = render(&highlight(
            &HighlightProps {
                query: &["FOX"],
                ignore_case: true,
                ..HighlightProps::default()
            },
            vec![],
            "the fox",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root">the <mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">fox</mark></span>"#
        );
    }

    #[test]
    fn rendering_is_deterministic_across_repeated_calls() {
        let props = HighlightProps {
            query: &["brown", "fox"],
            match_all: true,
            ignore_case: false,
            ..HighlightProps::default()
        };
        let first = render(&highlight(&props, vec![], "the brown fox jumps"));
        let second = render(&highlight(&props, vec![], "the brown fox jumps"));
        assert_eq!(first, second);
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (HighlightVariant::Subtle, "fd-highlight--variant-subtle"),
            (HighlightVariant::Solid, "fd-highlight--variant-solid"),
            (HighlightVariant::Text, "fd-highlight--variant-text"),
            (HighlightVariant::Plain, "fd-highlight--variant-plain"),
        ] {
            let props = HighlightProps {
                query: &["fox"],
                variant,
                ..HighlightProps::default()
            };
            let html = render(&highlight(&props, vec![], "the fox"));
            assert!(
                html.contains(&format!(
                    "class=\"{class} fd-highlight--color-palette-accent\""
                )),
                "variant={variant:?} -> {html}"
            );
        }
    }

    #[test]
    fn palette_enumeration_maps_to_expected_classes() {
        for (palette, class) in [
            (ColorPalette::Accent, "fd-highlight--color-palette-accent"),
            (ColorPalette::Info, "fd-highlight--color-palette-info"),
            (ColorPalette::Success, "fd-highlight--color-palette-success"),
            (ColorPalette::Warning, "fd-highlight--color-palette-warning"),
            (ColorPalette::Danger, "fd-highlight--color-palette-danger"),
            (ColorPalette::Neutral, "fd-highlight--color-palette-neutral"),
        ] {
            let props = HighlightProps {
                query: &["fox"],
                palette,
                ..HighlightProps::default()
            };
            let html = render(&highlight(&props, vec![], "the fox"));
            assert!(
                html.contains(&format!("class=\"fd-highlight--variant-subtle {class}\"")),
                "palette={palette:?} -> {html}"
            );
        }
    }

    #[test]
    fn caller_attrs_class_and_data_scope_part_are_dropped() {
        let html = render(&highlight(
            &HighlightProps::default(),
            vec![
                ("class", "attacker"),
                ("data-scope", "attacker"),
                ("Data-Part", "attacker"),
                ("data-testid", "kept"),
            ],
            "hello",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root" data-testid="kept">hello</span>"#
        );
    }

    #[test]
    fn non_ascii_text_with_ascii_query_does_not_panic_and_splits_correctly() {
        let html = render(&highlight(
            &HighlightProps {
                query: &["cat"],
                ..HighlightProps::default()
            },
            vec![],
            "こんにちは cat 世界🐈",
        ));
        assert_eq!(
            html,
            r#"<span data-scope="highlight" data-part="root">こんにちは <mark data-scope="highlight" data-part="mark" class="fd-highlight--variant-subtle fd-highlight--color-palette-accent">cat</mark> 世界🐈</span>"#
        );
    }
}
