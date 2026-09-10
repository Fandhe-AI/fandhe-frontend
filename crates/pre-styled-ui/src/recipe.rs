//! slot recipe（chakra-ui の slot recipe 相当）: 複数 anatomy パーツ（slot）を
//! 横断する variant 定義から、クラス名と静的 CSS を決定的に生成する。
//!
//! [`crate::css`] の低レベル宣言・検証を使い、`fandhe-frontend-headless-ui`
//! の `data-scope` / `data-part` セレクタ（`crates/headless-ui/src/anatomy.rs`）
//! と接続する CSS 規則を組み立てる。イシュー #550/#551 の styled 部品
//! （Button・Dialog ラッパー等）はここで定義した [`SlotRecipe`] を通じて
//! 「どの HTML 要素にどのクラスを付けるか」を決定する契約になる。
//!
//! # 順序規約（決定性の根拠）
//!
//! 内部ストレージは `Vec` のみを使い、`HashMap`/`HashSet` は使わない
//! （反復順序がプロセスごとに変わりうる型を持ち込まない）。[`SlotRecipe::css`]
//! の出力順は「base（`slots` の宣言順）→ variants（登録順）→ compound variants
//! （登録順、イシュー #604）→ states（登録順、イシュー #643）」に固定し、
//! 同一 slot・同一 axis/value への複数回登録は「後に登録された規則が CSS 中で
//! 後に出力される」（CSS のカスケードにおいて後勝ちになる）という素直な規約に
//! 従う。この規約より複雑な優先順位判定は行わない。states を最後尾に置くのは、
//! 各 styled 部品が従来 `state_css()`（`serialize_rule` 直呼び）で手書きして
//! いた `data-state` 連動規則を [`SlotRecipe::state`] へ移行した際に、
//! 「`stylesheet() = recipe().css() + state_css()`（状態規則が常に最後）」
//! という既存のカスケード上の性質をそのまま保存するため（イシュー #643）。
//!
//! # 状態条件付き規則（イシュー #643）
//!
//! [`SlotRecipe::state`] は `[data-highlighted]`（virtual focus の highlight
//! 表示）・`:focus-visible`（キーボードフォーカスリング）・`data-state`
//! （開閉等）のような、通常の variant 軸（ユーザーが選択する見た目のバリエー
//! ション）ではなく実行時の状態に応じて切り替わる CSS を recipe 経由で表現
//! するための API。[`StateCondition`] enum に条件の形を限定し、生のセレクタ
//! 文字列を受け取る API は設けない（`slot`/属性名/属性値は他の builder と
//! 同じ [`is_valid_identifier`] 検証を経由し、不正な入力は規則ごと出力から
//! 除外する fail-closed 方針。既存 `state_css()` 群が個別に手書きしていた
//! セレクタ組み立てをここへ一本化し、fail-closed 検証の迂回経路を増やさない）。
//!
//! compound variant（[`SlotRecipe::compound_variant`]）は複数軸の条件を
//! `.fd-<scope>--<a1>-<v1>.fd-<scope>--<a2>-<v2>...` のように連結したセレクタ
//! として出力する。条件 2 個以上なら詳細度が単一 variant セレクタより必ず
//! 高くなるため記述順に依存せず上書きが決まり、条件 1 個の場合でも compound
//! ブロックを variants ブロックより後に出力するため CSS カスケードの後勝ちで
//! 上書きされる（chakra-ui の「compoundVariants は variants を上書きする」
//! 意味論に対応する 2 段の保証）。
//!
//! # colorPalette 軸（イシュー #606）
//!
//! [`ColorPalette`] は `size` と並ぶ標準 variant 軸で、[`crate::theme`] の
//! セマンティック色（`accent`/`info`/`success`/`warning`/`danger`）と 1:1
//! 対応する。[`palette_declarations`] が返す宣言を palette 値ごとの
//! `SlotRecipe::variant` として root slot へ登録すると、選択された palette に
//! 応じて `--fandhe-palette`/`--fandhe-palette-emphasized`/`--fandhe-palette-fg`
//! （chakra-ui の virtual token 方式に相当するローカル custom property）が
//! 切り替わる。styled 部品側の色宣言は `var(--fandhe-palette)` 等を参照する
//! だけでよく、palette 軸の追加を機に既存の `var(--fandhe-color-accent)` 直書き
//! を書き換える（Button/Badge/Spinner/Alert、`crate` rustdoc 参照）。

use crate::css::{decl, is_valid_identifier, serialize_rule, Declaration};

/// クラス名プレフィックス（ライブラリ固定）。変更用の API は設けない
/// （`fd-{scope}--{axis}-{value}` の形式を全 styled 部品で一貫させるため）。
const CLASS_PREFIX: &str = "fd";

/// variant 軸 1 個の値を表す enum が実装するトレイト。
///
/// `Size::Sm` のような具象値から `axis()`（例: `"size"`）と `value()`（例:
/// `"sm"`）を取り出せることを要求する。[`SlotRecipe::variant`] /
/// [`SlotRecipe::variant_class`] はこのトレイトを通じてのみ variant を
/// 受け取るため、styled 部品側は生の文字列ではなく型安全な enum を渡す
/// （chakra-ui の `variants: { size: { sm: {...} } }` に対する型安全な代替）。
pub trait VariantValue: Copy {
    /// この値が属す variant 軸の名前（例: `"size"`）。
    fn axis(self) -> &'static str;
    /// この軸におけるこの値の名前（例: `"sm"`）。
    fn value(self) -> &'static str;
}

/// 標準の `size` 軸。#550 以降の styled 部品が共用する最初の具象 variant。
///
/// イシュー #1678 で `Xs`（先頭）・`Xl`（末尾）を純追加し 3 段から 5 段へ
/// 拡張した（t-shirt 語彙、chakra-ui 系に整合。Radix Themes の数値連番
/// `1`〜`4` は不採用）。既存 3 段（`Sm`/`Md`/`Lg`）の名前・`value()` は
/// 変更していないため、追加前から `Size` を使っている styled 部品の
/// golden CSS はバイト不変（新段を登録しないテーマ・recipe の出力は
/// 変わらない）。`2xl` は追加しない（共通 enum に載せると全部品が空の段を
/// 抱えるため。個別に必要な部品は専用の `VariantValue` 実装で扱う）。
/// 既定値（`Default`）は実装しない（安全側判断。呼び出し元が明示的に
/// 選択する現状の契約を変えない）。判断記録は
/// `docs/design/pre-styled-ui-size-and-color-palette-axes.md` を参照。
///
/// `size` 軸は enum として値をすべて公開するが、各 styled 部品が
/// `SlotRecipe::variant`/`SlotRecipe::size_variants` へ実際に登録する段は
/// レシピごとに異なる（未登録の段を `size` に指定しても class は付くが
/// 宣言は出ない、既存 [`SlotRecipe`] の挙動のまま）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    /// 極小サイズ（イシュー #1678 で追加）。
    Xs,
    /// 小サイズ。
    Sm,
    /// 中サイズ（既定値として使われることが多い）。
    Md,
    /// 大サイズ。
    Lg,
    /// 極大サイズ（イシュー #1678 で追加）。
    Xl,
}

impl VariantValue for Size {
    fn axis(self) -> &'static str {
        "size"
    }

    fn value(self) -> &'static str {
        match self {
            Size::Xs => "xs",
            Size::Sm => "sm",
            Size::Md => "md",
            Size::Lg => "lg",
            Size::Xl => "xl",
        }
    }
}

/// 標準の `color-palette` 軸（chakra-ui の `colorPalette` 相当、イシュー #606）。
///
/// [`crate::theme`] の既定パレット（`accent`/`info`/`success`/`warning`/
/// `danger`/`neutral`）と 1:1 対応する。Button/Badge/Spinner がこの軸を公開
/// API の variant として持ち、[`palette_declarations`] が生成する宣言を通じて
/// `--fandhe-palette` 系のローカル custom property を上書きする。
///
/// イシュー #1678 で `Neutral`（末尾、イシュー #1422 で追加された
/// `neutral*` トークンと 1:1）を純追加した。既存 5 値の名前・`value()`・
/// [`palette_declarations`] の出力は変更していないため、golden CSS は
/// バイト不変。`Gray` という別名は設けない（テーマ側のトークン名が
/// `neutral` のため）。任意色（利用者定義パレット）を受け付ける軸は
/// 設けない（`value()` は `&'static str` の固定語彙のみを返す設計を維持し、
/// 動的文字列を受ける入口を増やさない）。判断記録は
/// `docs/design/pre-styled-ui-size-and-color-palette-axes.md` を参照。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorPalette {
    /// 強調色（既定）。
    #[default]
    Accent,
    /// 情報提供色。
    Info,
    /// 成功色。
    Success,
    /// 警告色。
    Warning,
    /// 危険・エラー色。
    Danger,
    /// 中立色（イシュー #1678 で追加、chakra `gray` colorPalette 相当）。
    Neutral,
}

impl VariantValue for ColorPalette {
    fn axis(self) -> &'static str {
        "color-palette"
    }

    fn value(self) -> &'static str {
        match self {
            ColorPalette::Accent => "accent",
            ColorPalette::Info => "info",
            ColorPalette::Success => "success",
            ColorPalette::Warning => "warning",
            ColorPalette::Danger => "danger",
            ColorPalette::Neutral => "neutral",
        }
    }
}

/// `palette` が選択されたときに root slot へ登録する宣言列を返す
/// （chakra-ui の virtual token 方式の静的 CSS 版、イシュー #606）。
///
/// [`crate::theme`] が生成する `--fandhe-color-*`（テーマ層の名前空間）とは
/// 別の `--fandhe-palette-*` 名前空間へ、選択された palette に対応する
/// `accent`/`info`/`success`/`warning`/`danger` の 3 役割（base/emphasized/fg）
/// を `var()` 参照として束ねる。styled 部品側は `var(--fandhe-palette)` 等を
/// 参照するだけで、どの colorPalette が選択されたかに関わらず同じ宣言で
/// 色を切り替えられる。名前空間を分離しているため、ユーザーがカスタム
/// テーマへ独自の `palette` という名の色トークンを追加しても
/// （`Theme::push_color("palette", ...)`）本ヘルパの出力とは衝突しない。
#[must_use]
pub fn palette_declarations(p: ColorPalette) -> Vec<Declaration> {
    match p {
        ColorPalette::Accent => vec![
            decl("--fandhe-palette", "var(--fandhe-color-accent)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-accent-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-accent-fg)"),
        ],
        ColorPalette::Info => vec![
            decl("--fandhe-palette", "var(--fandhe-color-info)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-info-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-info-fg)"),
        ],
        ColorPalette::Success => vec![
            decl("--fandhe-palette", "var(--fandhe-color-success)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-success-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-success-fg)"),
        ],
        ColorPalette::Warning => vec![
            decl("--fandhe-palette", "var(--fandhe-color-warning)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-warning-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-warning-fg)"),
        ],
        ColorPalette::Danger => vec![
            decl("--fandhe-palette", "var(--fandhe-color-danger)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-danger-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-danger-fg)"),
        ],
        ColorPalette::Neutral => vec![
            decl("--fandhe-palette", "var(--fandhe-color-neutral)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-neutral-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-neutral-fg)"),
        ],
    }
}

/// `palette` が選択されたときに root slot へ登録する宣言列（6 役割版、
/// イシュー #1678）。
///
/// [`palette_declarations`]（3 役割: base/emphasized/fg）は既存 styled 部品の
/// golden CSS を守るため出力を変更していない。本関数は同じ 3 役割に加えて
/// イシュー #1422 で追加された `-subtle`/`-muted`/`-fg-subtle` の 3 役割を
/// 加えた 6 役割版で、返す宣言列の先頭 3 件は [`palette_declarations`] と
/// 完全に同一の順序・同一の値になる（`recipe_css.rs` の
/// `palette_scale_declarations_prefix_equals_palette_declarations` で固定）。
/// Phase 1 の各部品 issue が golden 更新時にこちらへ移行する想定であり、
/// イシュー #1679 で `mark` / `blockquote` が、イシュー #1681 で
/// `pagination` / `splitter` / `steps` / `tabs` / `tour`（Interactive）・
/// `badge` / `callout` / `spinner` / `status` / `tag` / `timeline`
/// （Data Display）が既にこちらへ移行済み（他の styled 部品は今後の
/// Phase で順次移行する）。`alert` はイシュー #1553 で移行済み（`status`
/// → [`ColorPalette`] 写像を内部で固定し、`ColorPalette` 軸自体は公開しない
/// 設計を維持したまま `palette_scale_declarations` を使う、`crate::alert`
/// 参照）。
///
/// [`Declaration`] は `&'static str` のみを保持できる設計（`crate::css` の
/// 型レベル不変条件、動的文字列混入経路を塞ぐ）のため、`format!` で値を
/// 組み立てず [`palette_declarations`] と同様に palette 値ごとの `match` で
/// リテラルを列挙する。
#[must_use]
pub fn palette_scale_declarations(p: ColorPalette) -> Vec<Declaration> {
    match p {
        ColorPalette::Accent => vec![
            decl("--fandhe-palette", "var(--fandhe-color-accent)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-accent-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-accent-fg)"),
            decl(
                "--fandhe-palette-subtle",
                "var(--fandhe-color-accent-subtle)",
            ),
            decl("--fandhe-palette-muted", "var(--fandhe-color-accent-muted)"),
            decl(
                "--fandhe-palette-fg-subtle",
                "var(--fandhe-color-accent-fg-subtle)",
            ),
        ],
        ColorPalette::Info => vec![
            decl("--fandhe-palette", "var(--fandhe-color-info)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-info-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-info-fg)"),
            decl("--fandhe-palette-subtle", "var(--fandhe-color-info-subtle)"),
            decl("--fandhe-palette-muted", "var(--fandhe-color-info-muted)"),
            decl(
                "--fandhe-palette-fg-subtle",
                "var(--fandhe-color-info-fg-subtle)",
            ),
        ],
        ColorPalette::Success => vec![
            decl("--fandhe-palette", "var(--fandhe-color-success)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-success-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-success-fg)"),
            decl(
                "--fandhe-palette-subtle",
                "var(--fandhe-color-success-subtle)",
            ),
            decl(
                "--fandhe-palette-muted",
                "var(--fandhe-color-success-muted)",
            ),
            decl(
                "--fandhe-palette-fg-subtle",
                "var(--fandhe-color-success-fg-subtle)",
            ),
        ],
        ColorPalette::Warning => vec![
            decl("--fandhe-palette", "var(--fandhe-color-warning)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-warning-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-warning-fg)"),
            decl(
                "--fandhe-palette-subtle",
                "var(--fandhe-color-warning-subtle)",
            ),
            decl(
                "--fandhe-palette-muted",
                "var(--fandhe-color-warning-muted)",
            ),
            decl(
                "--fandhe-palette-fg-subtle",
                "var(--fandhe-color-warning-fg-subtle)",
            ),
        ],
        ColorPalette::Danger => vec![
            decl("--fandhe-palette", "var(--fandhe-color-danger)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-danger-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-danger-fg)"),
            decl(
                "--fandhe-palette-subtle",
                "var(--fandhe-color-danger-subtle)",
            ),
            decl("--fandhe-palette-muted", "var(--fandhe-color-danger-muted)"),
            decl(
                "--fandhe-palette-fg-subtle",
                "var(--fandhe-color-danger-fg-subtle)",
            ),
        ],
        ColorPalette::Neutral => vec![
            decl("--fandhe-palette", "var(--fandhe-color-neutral)"),
            decl(
                "--fandhe-palette-emphasized",
                "var(--fandhe-color-neutral-emphasized)",
            ),
            decl("--fandhe-palette-fg", "var(--fandhe-color-neutral-fg)"),
            decl(
                "--fandhe-palette-subtle",
                "var(--fandhe-color-neutral-subtle)",
            ),
            decl(
                "--fandhe-palette-muted",
                "var(--fandhe-color-neutral-muted)",
            ),
            decl(
                "--fandhe-palette-fg-subtle",
                "var(--fandhe-color-neutral-fg-subtle)",
            ),
        ],
    }
}

/// フォーカスリング色の参照形（イシュー #1424、規約は
/// `docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` 参照）。
///
/// [`focus_ring_declarations`] の第 1 引数として渡し、リング色を
/// `--fandhe-color-focus-ring` トークンへ固定するか、
/// [`ColorPalette`]（`palette_declarations`）が切り替える
/// `--fandhe-palette` へフォールバック付きで連動させるかを選ぶ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusRingColor {
    /// `var(--fandhe-color-focus-ring, var(--fandhe-color-accent))` を
    /// 参照する（`palette` 軸を持たない部品、または hidden-input パターン
    /// の内側リング等）。`--fandhe-color-focus-ring` 未定義時（`Theme::empty()`
    /// ベースの既存カスタムテーマ）は本イシュー以前から存在する
    /// `--fandhe-color-accent` へフォールバックし、リングが消えない
    /// （イシュー #1424 レビュー指摘対応、[`focus_ring_declarations`] 参照）。
    Token,
    /// `var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)))`
    /// を参照する（`ColorPalette` 軸を公開する部品。選択中の palette が
    /// あればそれを使い、`palette` 未設定の文脈〔`--fandhe-palette` が
    /// root 側で定義されない場合〕では `--fandhe-color-focus-ring`
    /// （さらに未定義なら `--fandhe-color-accent`）へフォールバックする）。
    Palette,
}

/// フォーカスリングのオフセット方向（イシュー #1424）。
///
/// [`focus_ring_declarations`] の第 2 引数として渡し、リングを要素の外側
/// （既定）に描くか、内側（`overflow: hidden` な祖先の中でリングが切れる
/// のを避けたい splitter/scroll-area 等）に描くかを選ぶ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusRingOffset {
    /// 要素の外側（既定）。
    /// `outline-offset: var(--fandhe-focus-ring-offset, 2px)`。
    Outside,
    /// 要素の内側。
    /// `outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px))`
    /// （符号反転のみで表現し、専用トークンを増やさない）。
    Inset,
}

/// フォーカスリングの canonical 宣言列を組み立てる（イシュー #1424）。
///
/// `outline` + `outline-offset` の 2 宣言のみで構成し、`box-shadow` による
/// リング表現は行わない（`forced-colors: active` で `outline` の色は
/// システム色へ強制置換され必ず描画されるが、`box-shadow` は除去される
/// ため。`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
/// §3 参照）。値はすべて [`crate::theme`] のトークン参照
/// （`--fandhe-focus-ring-width`/`--fandhe-focus-ring-offset`/
/// `--fandhe-color-focus-ring`）で構成され、テーマ側 1 箇所で太さ・
/// オフセット・色を変更できる。各 `var()` はイシュー本文の 0.42.0
/// バンプ根拠（「破壊的変更ではない純追加」）を成立させるため、いずれも
/// 第 2 引数へ本イシュー以前の実際の既定値（`2px`・`--fandhe-color-accent`
/// 系）をフォールバックとして持つ（`Theme::empty()` ベースの既存カスタム
/// テーマがこれら新トークンを未定義でもキーボードフォーカス表示が消えない、
/// イシュー #1424 レビュー指摘対応）。
///
/// 呼び出し元は [`SlotRecipe::state`] の `declarations` 引数へそのまま渡す
/// （`StateCondition::FocusVisible`/`FocusWithin`/`Attr("data-focus-visible")`
/// のいずれかと組み合わせる。`:focus`（`-visible`/`-within` を伴わない
/// 直書き）と組み合わせて使わない規約は上記設計文書 §3 参照）。
#[must_use]
pub fn focus_ring_declarations(color: FocusRingColor, offset: FocusRingOffset) -> Vec<Declaration> {
    // フォールバック値（イシュー #1424 レビュー指摘対応）:
    // `--fandhe-focus-ring-width`/`--fandhe-focus-ring-offset`/
    // `--fandhe-color-focus-ring` は本イシューで新設したトークンであり、
    // `Theme::empty()` から構築した既存カスタムテーマはこれらを定義して
    // いない。フォールバックなしで直接参照すると `outline` 宣言全体が
    // computed-value time に無効となりキーボードフォーカス表示が消える
    // （0.42.0 を「破壊的変更ではない純追加」とする Cargo.toml のバンプ
    // 理由と矛盾する）。旧実装（本イシュー以前）が使っていた
    // `2px`/`var(--fandhe-color-accent)`（`--fandhe-color-accent` は
    // `DEFAULT_COLORS` の一員として本イシュー以前から存在するため、既存
    // カスタムテーマが定義している可能性が高い）を第 2 引数のフォール
    // バックに据えることで、新トークン未定義時も旧来の見た目を再現する。
    let outline = match color {
        FocusRingColor::Token => decl(
            "outline",
            "var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent))",
        ),
        FocusRingColor::Palette => decl(
            "outline",
            "var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)))",
        ),
    };
    let outline_offset = match offset {
        FocusRingOffset::Outside => decl("outline-offset", "var(--fandhe-focus-ring-offset, 2px)"),
        FocusRingOffset::Inset => decl(
            "outline-offset",
            "calc(-1 * var(--fandhe-focus-ring-offset, 2px))",
        ),
    };
    vec![outline, outline_offset]
}

/// disabled / hover / transition の共通ビジュアル言語（イシュー #1425、
/// 親 #1421）。Phase 1 以降の各部品 issue が本節のヘルパから選んで使う想定
/// （適用手順・duration 写像表は
/// `docs/design/pre-styled-ui-interaction-visual-language.md` 参照）。
///
/// - **disabled**: [`disabled_declarations`] を `root`（や disabled になり得る
///   slot）へ `.state(slot, StateCondition::Attr("data-disabled"),
///   disabled_declarations())` として登録する。`:disabled` ではなく
///   `[data-disabled]` を正とするのは、`<li>`/`<a>`/`<div>` ベースの
///   item/trigger にも同じ 1 経路で適用できるため（`:disabled` はネイティブ
///   フォーム要素にしか効かない）。
/// - **hover**: インタラクティブ slot（`cursor: pointer` を持つ、または
///   `<button>`/`<a>`/`role=option|menuitem|tab` 等を担う slot）にのみ、
///   各 variant が [`hover_bg_solid`]/[`hover_bg_muted`] のいずれかで
///   `--fandhe-hover-bg` を定義し、`root`（または対象 slot）へ
///   `.state(slot, StateCondition::Hover, hover_surface_declarations())` を
///   1 本登録する（variant ごとの色差は custom property 経由の間接参照で
///   表現し、`SlotRecipe` に「variant × state」の複合条件を持ち込まない）。
///   表示専用の slot（badge/alert/card/stat 等）には付けない。
/// - **transition**: [`transition_declarations`] を base（`root`）へ追加する。
#[must_use]
pub fn disabled_declarations() -> Vec<Declaration> {
    vec![decl("opacity", "0.5"), decl("cursor", "not-allowed")]
}

/// hover 時の背景色を適用する宣言（[`disabled_declarations`] 群の doc 参照）。
///
/// 実際の色は宣言に埋め込まず `var(--fandhe-hover-bg)` を参照するのみとし、
/// どの色になるかは各 variant が [`hover_bg_solid`]/[`hover_bg_muted`] で
/// 定義した `--fandhe-hover-bg` に委ねる（[`SlotRecipe`] が持たない
/// 「variant × state」の複合条件を、custom property の間接参照で代替する
/// 既存パターン。[`crate::table`] の `--fandhe-table-stripe-bg` と同型）。
#[must_use]
pub fn hover_surface_declarations() -> Vec<Declaration> {
    vec![decl("background", "var(--fandhe-hover-bg)")]
}

/// solid 系 variant（面が既に強調色で塗られている）向けの `--fandhe-hover-bg`
/// 定義。既存の `<palette>-emphasized` 段（`palette_declarations` が定義する
/// `var(--fandhe-palette-emphasized)`）を再利用し、hover 専用の新しい色段を
/// 追加しない。
#[must_use]
pub fn hover_bg_solid() -> Declaration {
    decl("--fandhe-hover-bg", "var(--fandhe-palette-emphasized)")
}

/// ghost/outline/subtle 系 variant・list item 等（面を持たない、または
/// 淡い面のみを持つ）向けの `--fandhe-hover-bg` 定義。
#[must_use]
pub fn hover_bg_muted() -> Declaration {
    decl("--fandhe-hover-bg", "var(--fandhe-color-bg-muted)")
}

/// [`hover_bg_solid`] のフォールバック連鎖版。checked/indeterminate 状態の
/// solid 面（`checkbox`/`checkbox-group` の control 系 slot）が、palette 軸
/// 未選択時（`--fandhe-palette-emphasized` 未定義）でも `--fandhe-color-
/// accent-emphasized` へ確実にフォールバックできるよう、CSS の 2 引数
/// `var()` フォールバック構文をそのまま埋め込む。
///
/// [`hover_bg_solid`] を単純に呼ばず本関数を分けて持つ理由: [`hover_bg_solid`]
/// は `SlotRecipe::variant` 経由（変数が必ず何らかの palette variant で
/// 定義された後に参照される文脈）を主用途とし、フォールバックを持たない
/// 素の間接参照のままにしている。一方 checkbox 系の checked/indeterminate
/// 規則は variant 軸を経由せず `state()` から直接 palette 変数を参照するため、
/// 未選択時のフォールバックが必要になる（`crates/pre-styled-ui/src/
/// checkbox.rs` / `checkbox_group.rs` の checked 系 `border-color`/
/// `background` 宣言と同じフォールバック連鎖）。
///
/// イシュー #1741: `checkbox.rs`/`checkbox_group.rs` に同一実装が複製されて
/// いたものをここへ共通化した（両モジュールの「モジュール doc スコープ外」
/// 記述は本共通化により解消済み）。
#[must_use]
pub fn hover_bg_solid_with_fallback() -> Declaration {
    decl(
        "--fandhe-hover-bg",
        "var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized))",
    )
}

/// [`transition_declarations`] が既定 easing として使うトークン
/// （汎用の enter/exit、[`crate::theme`] の `easing-standard` 既定値）。
const TRANSITION_EASING_VAR: &str = "var(--fandhe-motion-easing-standard)";

/// [`transition_declarations`] が受け取る duration の 3 段（イシュー #1425）。
/// [`crate::theme`] の `duration-fast`/`duration-normal`/`duration-slow`
/// トークンに 1:1 対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionDuration {
    /// `var(--fandhe-motion-duration-fast)`（150ms 既定）。ホバー等の軽微な
    /// 表面変化向け。
    Fast,
    /// `var(--fandhe-motion-duration-normal)`（200ms 既定）。開閉等の一般的な
    /// 遷移向け。
    Normal,
    /// `var(--fandhe-motion-duration-slow)`（300ms 既定）。モーダル等の
    /// 強調遷移向け。
    Slow,
}

impl MotionDuration {
    /// `var(--fandhe-motion-duration-<name>)` を返す。[`crate::theme::motion_var`]
    /// と同じトークン名を参照する固定 `&'static str` であり、文字列連結を
    /// 一切行わない（[`Declaration::value`] の `&'static str` 制約を満たす
    /// ための設計、本関数群冒頭 doc 参照）。
    const fn var_ref(self) -> &'static str {
        match self {
            MotionDuration::Fast => "var(--fandhe-motion-duration-fast)",
            MotionDuration::Normal => "var(--fandhe-motion-duration-normal)",
            MotionDuration::Slow => "var(--fandhe-motion-duration-slow)",
        }
    }
}

/// transition 宣言 3 件（`transition-property`/`transition-duration`/
/// `transition-timing-function`）を組み立てる（イシュー #1425）。
///
/// CSS shorthand `transition:` ではなく longhand 3 プロパティへ分解するのは、
/// duration/easing をトークン共通化しつつプロパティ名一覧のみ呼び出し側で
/// 変えられるようにするため。shorthand で複数プロパティへ同一 duration を
/// 割り当てるには各プロパティごとに duration/easing の反復記述が必要になり、
/// [`Declaration::value`] の `&'static str` 制約下では実行時の文字列連結
/// なしに表現できない（本モジュール冒頭のセキュリティ不変条件 - `decl()` は
/// ソースコード中のリテラルからのみ構築される - を保つため、`format!` した
/// 文字列を `Box::leak` する等の回避策は採らない）。
///
/// `properties` はカンマ区切りの CSS プロパティ名列を表すソースコード内
/// リテラル（例: `"background, border-color, color, box-shadow"`）を渡す
/// 想定。easing は常に `easing-standard` に固定する（本イシューでは
/// duration のみを可変にする単純化、`docs/design/
/// pre-styled-ui-interaction-visual-language.md` 参照）。
#[must_use]
pub fn transition_declarations(
    properties: &'static str,
    duration: MotionDuration,
) -> Vec<Declaration> {
    vec![
        decl("transition-property", properties),
        decl("transition-duration", duration.var_ref()),
        decl("transition-timing-function", TRANSITION_EASING_VAR),
    ]
}

/// [`transition_declarations`] の 3 宣言に `transition-behavior:
/// allow-discrete` を加えた 4 宣言を返す（イシュー #2192。案 C
/// 〔`docs/design/collapsible-height-animation.md`〕の中核 DSL）。
///
/// `properties` に `display` を含めるのが典型的な用途（`hidden` 属性による
/// `display: none` の実適用を、遷移完了まで遅延させる。`display:
/// none`/`block` は本来トランジションできないプロパティだが、
/// `allow-discrete` を付けると離散値プロパティも `@starting-style` と
/// 組み合わせて遷移させられる、[MDN `transition-behavior`]）。未対応
/// ブラウザでは `transition-behavior` 宣言自体が無視され、`display` の
/// 切り替えは即時になる（構造的な自然劣化、破綻はしない）。
///
/// [MDN `transition-behavior`]: https://developer.mozilla.org/en-US/docs/Web/CSS/transition-behavior
#[must_use]
pub fn transition_declarations_allow_discrete(
    properties: &'static str,
    duration: MotionDuration,
) -> Vec<Declaration> {
    let mut declarations = transition_declarations(properties, duration);
    declarations.push(decl("transition-behavior", "allow-discrete"));
    declarations
}

/// `fandhe-frontend-wasm-full` が collapsible/accordion 等の content 要素の
/// 実測高さ（px）を書き込む CSS custom property 名（イシュー #2192、案 C
/// 〔`docs/design/collapsible-height-animation.md`〕）。
///
/// # 他クレートとの契約
///
/// 値の出典は `crates/wasm-full/src/content_height.rs::CONTENT_HEIGHT_VAR`
/// であり、本定数はそのリテラルの写し。両者のドリフトは
/// `crates/pre-styled-ui/tests/content_height_var_drift.rs` が実行時に
/// wasm-full 側ソースを読んで fail-closed に検知する（`fandhe-frontend-
/// pre-styled-ui` は `fandhe-frontend-wasm-full` に依存しないため、
/// コンパイル時の型共有はできない）。wasm-full 側は「`hidden` 要素は
/// スキップし、実測 0 なら変数を除去する（`0px` を書き込まない）」契約
/// （CSSOM `set_property`/`remove_property` 経由）を持つため、本クレート
/// 側は必ず [`CONTENT_HEIGHT_VAR_REF`] のようにフォールバック付きで参照
/// する（変数が未設定の環境・JS 無効時でも `auto` へ安全に劣化する）。
pub const CONTENT_HEIGHT_VAR: &str = "--fandhe-content-height";

/// `var(--fandhe-content-height, auto)` 参照リテラル。フォールバックは
/// 常に `auto`（wasm-full 未配線・JS 無効時に content の高さを `auto` へ
/// 落として全文表示を保つ、イシュー #2192）。[`Declaration::value`] の
/// `&'static str` 制約のため `format!(CONTENT_HEIGHT_VAR)` ではなく固定
/// リテラルとして持つ。
const CONTENT_HEIGHT_VAR_REF: &str = "var(--fandhe-content-height, auto)";

/// [`content_height_open_declarations`] 専用の `transition-timing-function`
/// 値（PR #2289 codex レビュー P1 是正、イシュー #2192）。開く遷移
/// （base が終端スタイル）にのみ適用する（下記「開閉で非対称にする理由」
/// 参照。閉じる遷移には [`CONTENT_HEIGHT_TIMING_FUNCTION_CLOSING`] を使う）。
///
/// `transition-property: height, padding-block, margin-block, display,
/// overflow` の 5 項目に位置対応する 5 値のカンマ列。先頭 4 項目
/// （`height`/`padding-block`/`margin-block`/`display`）は従来どおり共通
/// easing（[`TRANSITION_EASING_VAR`]）を使うが、末尾の `overflow` だけは
/// `step-end` を使う。理由: `overflow`（離散値プロパティ）の値切り替え
/// タイミングをトランジション完了の瞬間に固定するため。`step-end` は
/// 「進捗が 1（完了）に達するまで開始値のまま、完了と同時に終了値へ切り
/// 替わる」ため、開く遷移（base が終端スタイル）の要件を満たせる: 終端値は
/// `overflow: visible`。`step-end` によりトランジション進行中は開始値
/// `hidden`（`[hidden]` state/`@starting-style` 側）のまま据え置かれ、
/// 完了の瞬間にだけ `visible` へ切り替わる。これにより開いた定常状態
/// （トランジション完了後）では常に `overflow: visible` となり、content
/// 内の `position: absolute` な Popover 等（`top: 100%` で自身の高さの
/// 外へはみ出す構成）が親のクリップで切り取られる回帰（calc-size() 対応
/// ブラウザで発生、codex レビュー指摘）を解消する。
///
/// # 開閉で非対称にする理由（PR #2289 codex レビュー第 2 ラウンド是正）
///
/// 閉じる遷移（`[hidden]` state が終端スタイル）へも同じ `step-end` を
/// 適用すると、終端値 `overflow: hidden` への切り替えが完了の瞬間まで
/// 遅延される。`calc-size()` 対応ブラウザでは `height`/`padding-block`
/// が先に縮み始める一方 `overflow` は `visible` のまま据え置かれるため、
/// 縮小中の content がクリップされず後続の Accordion 項目や Collapsible
/// 下のコンテンツに重なって表示され、`display: none` 適用の瞬間に突然
/// 消えるという表示回帰が生じる（height トランジション導入前にはなかった
/// 見た目、codex レビュー指摘）。閉じる遷移では代わりに
/// [`CONTENT_HEIGHT_TIMING_FUNCTION_CLOSING`]（`overflow` に `step-start`）
/// を使い、縮小開始の瞬間から `overflow: hidden` を有効にする。
///
/// [MDN `transition-behavior`]: https://developer.mozilla.org/en-US/docs/Web/CSS/transition-behavior
const CONTENT_HEIGHT_TIMING_FUNCTION: &str = "var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), step-end";

/// [`content_height_closed_transition_declarations`] 専用の
/// `transition-timing-function` 値（PR #2289 codex レビュー第 2 ラウンド
/// 是正、イシュー #2192）。閉じる遷移（`[hidden]` state が終端スタイル）
/// にのみ適用する。
///
/// [`CONTENT_HEIGHT_TIMING_FUNCTION`] と先頭 4 値（共通 easing）は同じだが、
/// 末尾の `overflow` だけ `step-start` にする。`step-start` は「進捗が
/// 0（開始）を超えた瞬間に終了値へ切り替わり、以降は終了値のまま」の
/// ため、閉じる遷移の終端値 `overflow: hidden`（[`content_height_closed_
/// declarations`]）が縮小開始の瞬間から有効になる。CSS Transitions Level 2
/// の `transition-behavior: allow-discrete` は「離散値プロパティの値切り
/// 替えを timing function 適用後の進捗に従って行う」仕様であり、
/// `step-start` を使えば `[hidden]` state 規則（開閉遷移のうち閉じる方向の
/// 終端スタイル）自身が宣言する `overflow` を、遷移開始と同時に有効化
/// できる。これにより `height`/`padding-block` が `calc-size()` で縮み
/// 始めるのと同時に `overflow: hidden` も有効になり、縮小中の content が
/// 後続要素に重なって表示されてから突然消える表示回帰
/// （[`CONTENT_HEIGHT_TIMING_FUNCTION`] rustdoc「開閉で非対称にする理由」
/// 参照）を解消する。開く遷移（base が終端スタイル）には従来どおり
/// [`CONTENT_HEIGHT_TIMING_FUNCTION`]（`step-end`）を使う。
const CONTENT_HEIGHT_TIMING_FUNCTION_CLOSING: &str = "var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), step-start";

/// [`SlotRecipe::content_height_transition`] が base（非 `hidden`）状態へ
/// 登録する宣言（イシュー #2192、codex レビュー是正で追補）。
/// `box-sizing: border-box` は wasm-full の実測値（`scrollHeight`、
/// padding 込み）と齟齬なく `height` を適用するため必須。
/// `overflow: visible` を終端値として登録し、`transition-property` に
/// `overflow` を含めて [`CONTENT_HEIGHT_TIMING_FUNCTION`] の `step-end`
/// でトランジション完了の瞬間にのみ切り替える（下記「開いた定常状態での
/// クリップ対策」節参照。PR #2289 codex レビュー P1 是正）。
/// `transition-property` に `display` を含めるため、共通ヘルパー
/// [`transition_declarations_allow_discrete`] は使わず本関数で個別に
/// 4 つの `transition-*` longhand を組み立てる（`overflow` だけ
/// `transition-timing-function` を他と変えるため、[`transition_declarations`]
/// の「単一値をカンマ列全体へ反復適用」という汎用設計に乗せられない）。
/// `hidden` 属性による `display: none` の実適用は遷移完了まで遅延する
/// （`transition-behavior: allow-discrete`）。
///
/// # `--fandhe-content-height: initial` を各 content 要素で明示する理由
///
/// CSS custom property は既定で継承される。wasm-full の測定・書き込みは
/// `[data-scope="collapsible"][data-part="content"]` 等、対象要素**自身**
/// への CSSOM `set_property`（インライン style）であり、祖先要素へ書く
/// ことはない。しかし本 slot の base ブロックがこの宣言を持たないと、
/// 入れ子の disclosure（例: accordion `item-content` の中に別の
/// collapsible が入る構成）で未測定の子要素が `var(--fandhe-content-
/// height, auto)` を評価する際、自身に値が無ければ祖先の実測値を
/// **継承**してしまい、祖先の高さを誤って自身の高さとして適用する
/// （`docs/design/collapsible-height-animation.md` §5.2、PR #2289 codex
/// レビュー指摘）。`initial` をここで明示すると、この宣言はカスケード上
/// 常に「祖先からの継承値」より後（同一要素・同一詳細度の中で最後）に
/// 評価されるため一旦リセットされ、その後 JS が当該要素へ書き込む
/// インライン style（詳細度が常に上回る）だけが実効値として残る。
/// 測定前は `initial` → `var()` のフォールバック `auto` が効き、祖先の
/// 値を誤って継承しない。
///
/// # 開いた定常状態でのクリップ対策（`calc-size()` progressive
/// enhancement）
///
/// `height: var(--fandhe-content-height, auto)` は wasm-full が配線時・
/// `on_update` 直後にしか測定しないため、開いた定常状態で内容が後から
/// 伸びる（ウィンドウ幅変化による折り返し増加・画像の遅延読み込み等）と
/// 固定 px 高さのまま `overflow: hidden` に切り取られる限界があった
/// （`docs/design/collapsible-height-animation.md` §5.2「既知の限界
/// （定常状態のクリップ）」。同節が対策として挙げる
/// `interpolate-size: allow-keywords` の progressive enhancement を
/// 具体化する）。`height: calc-size(auto, size)`
/// を var 参照の直後に追加で登録する: `calc-size()` 未対応ブラウザでは
/// この宣言全体が構文解析時点で無効となり同一規則内の直前の `height`
/// 宣言（var 参照）がそのまま有効のまま残る（CSS の標準的な優雅な
/// 劣化。未対応ブラウザは従来どおり固定 px + JS 測定に留まり、既知の
/// 限界も従来どおり残る）。対応ブラウザでは非遷移時の定常状態の
/// `calc-size(auto, size)` は `height: auto` と等価に評価され（内容の
/// 変化に継続追従し、固定値へピン留めされない）、かつ `0 → 定常値` の
/// トランジションも同時に成立する（`calc-size()` は自動的に
/// `interpolate-size: allow-keywords` を当該宣言へ適用する仕様）。
/// この経路では wasm-full の測定結果を待たずに開閉トランジション自体が
/// 成立するため、対応ブラウザでは実質的に「JS 測定なしでも動く」上位
/// 互換となる。
///
/// `calc-size()` 対応ブラウザで開閉トランジション自体が成立するように
/// なった結果、`overflow: hidden` を base（開いた定常状態も含む）へ
/// 恒常的に残すと新たな回帰が生じる: content 内に `position: absolute;
/// top: 100%` で自身の境界の外へ意図的にはみ出す Popover 等（`portal`
/// 未提供のため親のクリップ領域内に留まらざるを得ない構成）が、開閉
/// 完了後の定常状態でも親のクリップで切り取られてしまう（PR #2289
/// codex レビュー P1 指摘）。`content_height_transition` 導入前
/// （`height: auto` を継続評価し `overflow` 宣言自体を持たなかった状態）
/// と比べた表示回帰にあたるため、本関数は `overflow: hidden` を base の
/// 恒常値としては持たない。代わりに [`CONTENT_HEIGHT_TIMING_FUNCTION`]
/// （`step-end`）で `overflow` の値切り替えをトランジション完了の瞬間
/// だけに限定し、`overflow: visible` を開いた定常状態の終端値として
/// 登録する（このタイミング関数は開く遷移＝本関数が終端スタイルになる
/// 方向にのみ適用する。閉じる遷移側の終端スタイル
/// [`content_height_closed_transition_declarations`] は別の
/// `step-start` タイミング関数を自前で宣言し、閉じる方向の overflow
/// 切り替えタイミングを個別制御する。理由は
/// [`CONTENT_HEIGHT_TIMING_FUNCTION`] rustdoc「開閉で非対称にする理由」
/// 節参照。PR #2289 codex レビュー第 2 ラウンド是正）。トランジション
/// 進行中は開始値（`[hidden]` state/`@starting-style` 側の
/// `overflow: hidden`）が `step-end` により維持されるため、開く方向の
/// 遷移中に内容が切り取られる従来の効果は変わらず保たれる。
#[must_use]
pub fn content_height_open_declarations(duration: MotionDuration) -> Vec<Declaration> {
    vec![
        decl("box-sizing", "border-box"),
        decl("overflow", "visible"),
        decl(CONTENT_HEIGHT_VAR, "initial"),
        decl("height", CONTENT_HEIGHT_VAR_REF),
        decl("height", "calc-size(auto, size)"),
        decl(
            "transition-property",
            "height, padding-block, margin-block, display, overflow",
        ),
        decl("transition-duration", duration.var_ref()),
        decl("transition-timing-function", CONTENT_HEIGHT_TIMING_FUNCTION),
        decl("transition-behavior", "allow-discrete"),
    ]
}

/// [`SlotRecipe::content_height_transition`] が `[hidden]` state・
/// `@starting-style` の両方へ登録する縮小状態の宣言（イシュー #2192、
/// Bugbot 指摘を受け `margin-block: 0` を追補）。
/// `padding-block: 0` も含めるのは、`height: 0` 単独だと閉じる途中で
/// padding 分の高さが残ったまま `display: none` に落ちる見た目のジャンプ
/// を避けるため（`docs/design/collapsible-height-animation.md` §5.2）。
/// `margin-block: 0` も同じ理由: `crate::collapsible` の content slot は
/// `margin-top: var(--fandhe-space-2)` を base で宣言するが、閉状態
/// （`[hidden]`）・`@starting-style` の双方がこれを 0 へ縮めないと、
/// `height`/`padding-block` が 0 へアニメーションし切った後も margin
/// だけが残り、`display: none` 適用の瞬間に隙間が唐突に消える見た目の
/// ジャンプが生じる（PR #2289 Bugbot 指摘）。`content_height_open_
/// declarations` の `transition-property` に `margin-block` を含めて
/// あるため、他部品（`margin-block` を宣言しない accordion
/// `item-content` 等）では実質 no-op のまま安全に共有できる。
///
/// `overflow: hidden` を追補したのは PR #2289 codex レビュー P1 是正
/// （[`content_height_open_declarations`] rustdoc「開いた定常状態での
/// クリップ対策」節参照）の一部: `[hidden]` state・`@starting-style` の
/// 双方をトランジションの「開始値」として使うため、この関数が
/// `overflow: hidden` を明示しないと縮む方向の遷移中に内容が切り取られ
/// なくなってしまう。
///
/// 本関数自体は `transition-property` 等を宣言しない（`@starting-style`
/// はトランジション対象プロパティの開始値スナップショットであり、
/// 自前の `transition-*` 宣言を必要としないため）。`[hidden]` state は
/// 代わりに [`content_height_closed_transition_declarations`] を使い、
/// 閉じる遷移専用の `transition-timing-function`
/// （[`CONTENT_HEIGHT_TIMING_FUNCTION_CLOSING`]）を自前で宣言する
/// （PR #2289 codex レビュー第 2 ラウンド是正、理由は
/// [`CONTENT_HEIGHT_TIMING_FUNCTION`] rustdoc「開閉で非対称にする理由」
/// 節参照）。
#[must_use]
pub fn content_height_closed_declarations() -> Vec<Declaration> {
    vec![
        decl("height", "0"),
        decl("padding-block", "0"),
        decl("margin-block", "0"),
        decl("overflow", "hidden"),
    ]
}

/// [`SlotRecipe::content_height_transition`] が `[hidden]` state
/// （閉じる遷移の終端スタイル）へ登録する宣言（PR #2289 codex レビュー
/// 第 2 ラウンド是正、イシュー #2192）。
///
/// [`content_height_closed_declarations`] の 4 値宣言に加えて、閉じる
/// 遷移専用の `transition-property`/`transition-duration`/
/// `transition-timing-function`（[`CONTENT_HEIGHT_TIMING_FUNCTION_CLOSING`]、
/// `overflow` に `step-start`）/`transition-behavior` を自前で宣言する。
///
/// # `[hidden]` state が独自に `transition-*` を宣言する理由
///
/// CSS Transitions は、値が変化した瞬間に適用される
/// `transition-property`/`transition-duration`/`transition-timing-function`
/// を、その時点でカスケードにより有効な（＝遷移先の）計算値から決定する。
/// [`content_height_open_declarations`]（base）が宣言する
/// `transition-timing-function`（[`CONTENT_HEIGHT_TIMING_FUNCTION`]、
/// `overflow` に `step-end`）は、base が遷移先になる方向（開く遷移）に
/// のみ実効させたい。しかし `[hidden]` セレクタは base より詳細度が高い
/// ため、本関数が `transition-*` を明示しなければ base の宣言がそのまま
/// カスケードを通過し、閉じる遷移（`[hidden]` が遷移先）にも同じ
/// `step-end` が適用されてしまう（表示回帰の原因、
/// [`CONTENT_HEIGHT_TIMING_FUNCTION`] rustdoc 参照）。`[hidden]` 自身が
/// `transition-timing-function` を宣言することで、閉じる遷移の実効値を
/// `step-start` へ個別に固定できる（CSS カスケードは宣言単位＝longhand
/// 単位で決まるため、他の longhand（`height` 等の値）は
/// [`content_height_closed_declarations`] のまま変わらない）。
#[must_use]
pub fn content_height_closed_transition_declarations(duration: MotionDuration) -> Vec<Declaration> {
    let mut declarations = content_height_closed_declarations();
    declarations.push(decl(
        "transition-property",
        "height, padding-block, margin-block, display, overflow",
    ));
    declarations.push(decl("transition-duration", duration.var_ref()));
    declarations.push(decl(
        "transition-timing-function",
        CONTENT_HEIGHT_TIMING_FUNCTION_CLOSING,
    ));
    declarations.push(decl("transition-behavior", "allow-discrete"));
    declarations
}

/// slot 1 個への base 宣言登録（内部表現）。
struct BaseRule {
    slot: &'static str,
    declarations: Vec<Declaration>,
}

/// slot 1 個・variant 値 1 個への宣言登録（内部表現）。
struct VariantRule {
    axis: &'static str,
    value: &'static str,
    slot: &'static str,
    declarations: Vec<Declaration>,
}

/// axis ごとの既定 variant 値（内部表現）。
struct DefaultVariant {
    axis: &'static str,
    value: &'static str,
}

/// [`SlotRecipe::state`] が受け付ける状態条件（属性・擬似クラス、イシュー #643）。
///
/// 各 styled 部品（dialog/tabs/accordion/menu/select）が `state_css()` で
/// `serialize_rule` を直接呼び手書きしていたセレクタ（`[data-state="open"]`・
/// `[hidden]` 等）を、ここで型として限定した条件へ移行する。生のセレクタ
/// 文字列を受け取る経路は設けず、`name`/`value` は [`SlotRecipe::css`] が
/// `is_valid_identifier` で検証する（不正な場合は規則ごと除外）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateCondition {
    /// 存在属性 `[<name>]`（例: `data-highlighted`・`hidden`）。
    Attr(&'static str),
    /// 値付き属性 `[<name>="<value>"]`（例: `data-state="open"`）。
    AttrEq(&'static str, &'static str),
    /// `:focus-visible` 擬似クラス（キーボードフォーカスリング用。Menu/Select
    /// の `item` は virtual focus パターンのためフォーカスが `trigger` に
    /// 留まり続け、`data-highlighted`（[`StateCondition::Attr`]）が highlight
    /// 表示を担う契約であり、`item` へ `:focus-visible` は付けない）。
    /// hidden-input パターン（実フォーカスが visually-hidden なネイティブ
    /// `<input>` にあり、リングを見せたい視覚パーツと分離している構成、
    /// 例: Switch の `control`・RadioGroup の `item-control`）は擬似クラスの
    /// 対象要素自体が実フォーカスを受けないため本 variant では表現できず、
    /// 代わりに headless 層の `data-focus-visible` 存在属性
    /// （[`StateCondition::Attr`]）+ クライアントランタイムの付け外しで
    /// 表現する（イシュー #709、`crate::switch`/`crate::radio_group` rustdoc
    /// 参照）。
    FocusVisible,
    /// `:focus-within` 擬似クラス（イシュー #683）。visually-hidden 化した
    /// ネイティブフォーム部品（`<input>` 等）を子孫に内包する要素へ
    /// フォーカスリングを付ける唯一の経路として追加した。実フォーカスは
    /// 隠された子孫要素にあり、その要素自体へ `:focus-visible` を当てても
    /// 視覚的に隠されたままのため意味を成さない。`item`（`<label>`、隠した
    /// `item-hidden-input` の祖先）が最初の消費者（RadioGroup styled
    /// ラッパー、`crate::radio_group` 参照）。
    FocusWithin,
    /// `:nth-child(even)` 擬似クラス（イシュー #767）。[`crate::table`] の
    /// striped 表現（縞模様の背景色）専用の追加。[`SlotRecipe`] は子孫
    /// セレクタ機構を持たない（本モジュール冒頭 doc 「colorPalette 軸」節の
    /// 前段方針、#708 で「追加しない」と確定）ため、striped は root variant
    /// が登録する `--fandhe-table-stripe-bg` custom property と、`row` slot
    /// 自身への本条件付き規則の組み合わせで表現する。`nth-child` は親要素
    /// （実際には `thead`/`tbody`/`tfoot` それぞれ）内の兄弟基準で数えるため、
    /// 複数 `<tr>` を持つ `thead` では 2 行目以降も対象になる（`crate::table`
    /// rustdoc 参照）。
    NthChildEven,
    /// `:last-child` 擬似クラス（イシュー #752 PR #797 Bugbot レビュー
    /// Medium severity 指摘「Last step item still stretches」対応）。
    /// Steps の `item`（`<li>`）は呼び出し側が最後の `separator` を省略
    /// するのが典型的な利用パターンであり、最後の item だけは「後ろに
    /// 伸ばす対象（separator）」を持たない。headless 層は `index`/`count`
    /// を比較した「最後かどうか」の data 属性を持たないため（`item`
    /// メソッドが `count` を保持しない設計、`crates/headless-ui/src/steps.rs`
    /// 参照）、DOM 構造がそのまま表す `:last-child` を使い、最後の item
    /// にのみ伸長・最小高さの指定を打ち消す。
    LastChild,
    /// 複数の値付き属性の AND 条件
    /// `[<name1>="<value1>"][<name2>="<value2>"]...`（イシュー #841 PR #870
    /// Bugbot レビュー Medium severity 指摘「Positioner skips align
    /// fallback」対応）。
    ///
    /// [`crate::tour`] の positioner 静的フォールバックは `data-side` と
    /// `data-align` の組み合わせで配置が決まる（例: `Left`+`Start` と
    /// `Left`+`Center` は異なる表示）。単一属性条件（[`StateCondition::AttrEq`]）
    /// だけでは片方の軸しか捕捉できず、もう片方の軸違いが無視されてしまう
    /// ため、複数属性の AND を 1 セレクタで表現する本 variant を追加した。
    /// 要素は `(name, value)` の組。空スライスは無条件規則（`base` と同義）
    /// になる意味のない規則のため [`SlotRecipe::css`] が除外する。
    AttrEqAll(&'static [(&'static str, &'static str)]),
    /// `:hover` 擬似クラス（イシュー #847）。
    ///
    /// [`crate::charts::tooltip`] のデータ点（`datum` slot）専用の追加。
    /// SVG のデータ点はマウス追従型のリッチツールチップ（JS 必須、
    /// `crate::charts::tooltip` モジュール doc「スコープ外」節参照）を
    /// 持たず、代わりに子 `<title>` 要素によるブラウザネイティブな hover
    /// 表示 + 本条件による視覚的強調（`opacity`/`stroke-width` 変更）の
    /// 組み合わせで「ホバーで詳細が分かる」体験を CSS のみで表現する。
    /// `FocusVisible`/`NthChildEven` 等と同じく「消費者が現れた時点で
    /// 追加する」前例に従う（本モジュール冒頭 doc 参照）。
    Hover,
    /// `:hover:not([data-disabled]):not([<name>="<value>"])`（[`Hover`]
    /// の disabled 除外に加え、値付き属性 1 件も除外する。イシュー #1463
    /// PR #1740 Bugbot レビュー Medium severity 指摘「Hover overrides open
    /// border」対応）。
    ///
    /// [`crate::color_picker`] の `trigger` は `open` 状態でアクセント色の
    /// 枠線（[`StateCondition::AttrEq`]）を表示するが、同じ slot へ素の
    /// [`Hover`] を併用すると `:hover:not([data-disabled])`
    /// （specificity (0,4,0)）が `[data-state="open"]`（specificity
    /// (0,3,0)）より高く、かつ [`SlotRecipe::css`] が hover 規則を
    /// `@media (hover: hover)` として常に末尾へ集約出力するため、open な
    /// trigger にホバーすると open のアクセント枠線が hover の枠線色へ
    /// 上書きされてしまう（specificity・ソース順の両方で hover が勝つ）。
    /// 本 variant は hover 条件の対象から `[<name>="<value>"]` に一致する
    /// 要素そのものを除外する（`:not()` によるマッチ除外であり、
    /// [`Hover`] 自体の specificity や `AttrEq`/`AttrEqAll` 側の
    /// specificity を変更しない）ため、open かつ hover な trigger では
    /// この規則が一切マッチせず、open 側の規則がそのまま残る。
    HoverExcept(&'static str, &'static str),
    /// `:hover:not([data-disabled]):not([<name>])`（[`HoverExcept`] の値付き
    /// 属性版に対する、存在属性（boolean 属性）版。イシュー #1468 PR #1745
    /// codex-review P1 / Bugbot Medium severity 指摘「Hover washes out
    /// highlighted item contrast」対応）。
    ///
    /// [`HoverExcept`] は `[<name>="<value>"]` の値等価で除外対象を絞るが、
    /// headless 層が存在属性としてのみ出す状態（例:
    /// `crates/headless-ui/src/combobox.rs::item` の `data-highlighted`。
    /// 値は常に空文字列 `""` の `data-highlighted=""` で出力される）は
    /// [`is_valid_identifier`] が空文字列を拒否するため `HoverExcept(name,
    /// "")` は条件不成立で規則ごと無音に脱落してしまう（値の妥当性検証を
    /// 弱めて空文字列を許容すると、他の呼び出し元が本来の値付き属性
    /// マッチングを意図せず素通りさせる余地を生むため、専用 variant として
    /// 分離した）。[`crate::combobox`] の `item` hover（`[data-highlighted]`
    /// specificity (0,3,0) に `:hover:not([data-disabled])` (0,4,0) が
    /// 勝ってしまい、ポインタが highlight 中の item に重なると accent 背景
    /// が muted 背景に上書きされる／`hover_surface_declarations()` は
    /// `background` shorthand のみのため文字色 accent-fg だけが取り残され
    /// コントラストが崩れる）が最初の消費者。`:not()` 自体は specificity を
    /// 加算しないが、その引数（属性セレクタ）の specificity はそのまま
    /// 寄与するため、[`HoverExcept`] と同じく `:not([<name>])` の追加分
    /// （属性セレクタ 1 個分 = (0,1,0)）だけ規則全体の specificity が
    /// [`Hover`] 単体より高くなる。
    HoverExceptAttr(&'static str),
    /// `:hover:not([data-disabled]):not([<attr_name>]):not([<eq_name>="<eq_value>"])`
    /// （[`HoverExceptAttr`] の存在属性除外と [`HoverExcept`] の値等価除外を
    /// 1 つの hover 規則で両方適用する複合版。イシュー #1702 PR #1803
    /// Bugbot Medium severity 指摘「Hover washes out open trigger」対応）。
    ///
    /// [`crate::menubar`] の `trigger` は highlight（`data-highlighted`、
    /// specificity (0,3,0)）と open（`[data-state="open"]`、同じく
    /// (0,3,0)）という specificity が等しい 2 状態を持つ。[`HoverExceptAttr`]
    /// は highlighted のみを hover 対象から除外するため、open だが
    /// highlighted ではない trigger（ポインタが乗っているだけで virtual
    /// focus は別の trigger にある状態）に hover すると
    /// `:hover:not([data-disabled]):not([data-highlighted])` (0,4,0) が
    /// open の `accent-subtle` 背景を muted hover 背景で上書きしてしまう
    /// （highlighted のみを除外しても open は除外されないため）。本
    /// variant は両方を `:not()` で除外し、open な trigger・highlighted な
    /// trigger のいずれも hover で洗い流されないようにする。`:not()` 自体は
    /// specificity を加算しないが、その引数（属性セレクタ）の specificity は
    /// そのまま寄与するため、[`HoverExcept`]・[`HoverExceptAttr`] の
    /// `:not()` 節に加えてさらに 1 個分（属性セレクタ 1 個分 = (0,1,0)）
    /// specificity が高くなり、素の [`Hover`] より高い specificity を持つ。
    HoverExceptAttrEq(&'static str, &'static str, &'static str),
}

/// slot 1 個・状態条件 1 個への宣言登録（内部表現、イシュー #643）。
struct StateRule {
    slot: &'static str,
    condition: StateCondition,
    declarations: Vec<Declaration>,
}

/// slot 1 個・任意の状態条件への `@starting-style` 内規則登録（内部表現、
/// イシュー #2192）。`condition` が `None` の場合は無条件（`@starting-style
/// { [data-scope][data-part] { ... } }`）を表し、`Some` の場合は
/// [`SlotRecipe::state`] と同じ [`StateCondition`] 検証・セレクタ組み立て
/// 経路（`state_condition_selector`）を通る。Hover 系条件（[`StateCondition::
/// Hover`] 等）は `@starting-style` 内で意味を持たない（starting style は
/// 遷移開始前のスナップショットであり `:hover` のような動的擬似クラスの
/// 「開始状態」という概念が成立しない）ため [`SlotRecipe::css`] が規則ごと
/// 除外する（fail-closed）。
struct StartingStyleRule {
    slot: &'static str,
    condition: Option<StateCondition>,
    declarations: Vec<Declaration>,
}

/// compound variant の条件 1 件（axis, value の型消去された組）。
///
/// [`when()`] を通じてのみ [`VariantValue`] 実装 enum から構築できる（生の
/// 文字列を受け付けない。既存 `variant()` と同じ enum ベースの型安全性を
/// [`SlotRecipe::compound_variant`] の条件部でも保つ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariantCondition {
    axis: &'static str,
    value: &'static str,
}

/// `VariantValue` の具象値から [`VariantCondition`] を作る（chakra-ui の
/// `compoundVariants: [{ size: "sm", tone: "outline", css: {...} }]` の
/// 条件部 1 軸分に相当）。[`SlotRecipe::compound_variant`] の `conditions`
/// 引数は本関数の戻り値を並べた `Vec` として組み立てる。
#[must_use]
pub fn when<V: VariantValue>(v: V) -> VariantCondition {
    VariantCondition {
        axis: v.axis(),
        value: v.value(),
    }
}

/// 複数 variant 軸の組み合わせ条件が揃ったときの slot への宣言登録
/// （内部表現。chakra-ui の `compoundVariants` 1 件に相当、イシュー #604）。
struct CompoundVariantRule {
    conditions: Vec<VariantCondition>,
    slot: &'static str,
    declarations: Vec<Declaration>,
}

/// slot recipe: `scope`（headless anatomy と同一値）・`slots`・base・variants・
/// defaultVariants を保持し、静的 CSS とクラス名を決定的に生成する。
///
/// # 呼び出し文脈
///
/// `scope` は対応する `fandhe-frontend-headless-ui` の
/// `Anatomy::new(scope)`（例: `crates/headless-ui/src/tabs.rs` の
/// `ANATOMY`）と同じ値を渡す契約とする。これにより [`SlotRecipe::css`] が
/// 生成するセレクタ `[data-scope="<scope>"][data-part="<slot>"]` が、
/// headless 層が実際にレンダリングする属性と一致する（本クレートの
/// `tests/recipe_css.rs` が headless 層の実マークアップと照合して固定する）。
pub struct SlotRecipe {
    scope: &'static str,
    slots: &'static [&'static str],
    base: Vec<BaseRule>,
    variants: Vec<VariantRule>,
    default_variants: Vec<DefaultVariant>,
    compound_variants: Vec<CompoundVariantRule>,
    states: Vec<StateRule>,
    starting_style: Vec<StartingStyleRule>,
    supports_not_calc_size: Vec<BaseRule>,
}

/// [`StateCondition`] 1 個が識別子として妥当かどうかを判定する（内部
/// ヘルパ）。[`state_condition_selector`] から呼ばれ、[`SlotRecipe::css`]
/// の states ループ・`@starting-style` ループの双方が同じ検証経路を
/// 通るようにする（二重実装によるドリフト防止）。
fn state_condition_is_valid(condition: &StateCondition) -> bool {
    match condition {
        StateCondition::Attr(name) => is_valid_identifier(name),
        StateCondition::AttrEq(name, value) => {
            is_valid_identifier(name) && is_valid_identifier(value)
        }
        StateCondition::FocusVisible => true,
        StateCondition::FocusWithin => true,
        StateCondition::NthChildEven => true,
        StateCondition::LastChild => true,
        StateCondition::AttrEqAll(pairs) => {
            !pairs.is_empty()
                && pairs
                    .iter()
                    .all(|(name, value)| is_valid_identifier(name) && is_valid_identifier(value))
        }
        StateCondition::Hover => true,
        StateCondition::HoverExcept(name, value) => {
            is_valid_identifier(name) && is_valid_identifier(value)
        }
        StateCondition::HoverExceptAttr(name) => is_valid_identifier(name),
        StateCondition::HoverExceptAttrEq(attr_name, eq_name, eq_value) => {
            is_valid_identifier(attr_name)
                && is_valid_identifier(eq_name)
                && is_valid_identifier(eq_value)
        }
    }
}

/// [`StateCondition`] 1 個からセレクタ接尾辞（例: `[hidden]`・
/// `:focus-visible`・`:hover:not([data-disabled])`）を組み立てる（内部
/// ヘルパ）。不正な入力（[`state_condition_is_valid`] が `false`）は
/// `None` を返す（fail-closed）。[`SlotRecipe::css`] の states ループ・
/// `@starting-style` ループの双方が本関数を通る（セレクタ組み立てを二重
/// 実装しない、イシュー #2192 で `@starting-style` 追加時に切り出した）。
fn state_condition_selector(condition: &StateCondition) -> Option<String> {
    if !state_condition_is_valid(condition) {
        return None;
    }
    let mut suffix = String::new();
    match condition {
        StateCondition::Attr(name) => suffix.push_str(&format!("[{name}]")),
        StateCondition::AttrEq(name, value) => {
            suffix.push_str(&format!("[{name}=\"{value}\"]"));
        }
        StateCondition::FocusVisible => suffix.push_str(":focus-visible"),
        StateCondition::FocusWithin => suffix.push_str(":focus-within"),
        StateCondition::NthChildEven => suffix.push_str(":nth-child(even)"),
        StateCondition::LastChild => suffix.push_str(":last-child"),
        StateCondition::AttrEqAll(pairs) => {
            for (name, value) in *pairs {
                suffix.push_str(&format!("[{name}=\"{value}\"]"));
            }
        }
        StateCondition::Hover => {
            // タッチ端末での hover 貼り付きを避けるため `@media (hover:
            // hover)` 配下へまとめて出力する（イシュー #1425）。
            // `:not([data-disabled])` で disabled 規則との勝敗を記述順に
            // 依存させない。
            suffix.push_str(":hover:not([data-disabled])");
        }
        StateCondition::HoverExcept(name, value) => {
            suffix.push_str(&format!(
                ":hover:not([data-disabled]):not([{name}=\"{value}\"])"
            ));
        }
        StateCondition::HoverExceptAttr(name) => {
            suffix.push_str(&format!(":hover:not([data-disabled]):not([{name}])"));
        }
        StateCondition::HoverExceptAttrEq(attr_name, eq_name, eq_value) => {
            suffix.push_str(&format!(
                ":hover:not([data-disabled]):not([{attr_name}]):not([{eq_name}=\"{eq_value}\"])"
            ));
        }
    }
    Some(suffix)
}

/// `condition` が Hover 系（[`StateCondition::Hover`]・[`StateCondition::
/// HoverExcept`]・[`StateCondition::HoverExceptAttr`]・[`StateCondition::
/// HoverExceptAttrEq`]）かどうかを判定する（内部ヘルパ）。[`SlotRecipe::css`]
/// の states ループが `@media (hover: hover)` 側バッファへ振り分けるため、
/// `@starting-style` ループが Hover 系規則を除外するために使う（イシュー
/// #2192。理由は [`StartingStyleRule`] rustdoc 参照）。
fn is_hover_family(condition: &StateCondition) -> bool {
    matches!(
        condition,
        StateCondition::Hover
            | StateCondition::HoverExcept(_, _)
            | StateCondition::HoverExceptAttr(_)
            | StateCondition::HoverExceptAttrEq(_, _, _)
    )
}

/// `<selector> { ... }` 規則列を 1 個の at-rule ブロック（`@media (hover:
/// hover) { ... }`・`@starting-style { ... }` 等）へインデント付きで包む
/// （内部ヘルパ）。[`SlotRecipe::css`] の hover ブロック・`@starting-style`
/// ブロックの双方が同じインデント付与ロジックを共有する（イシュー #2192
/// で `@starting-style` 追加時に切り出した。書式は変更しない）。
fn write_at_rule_block(out: &mut String, at_rule: &str, inner_css: &str) {
    out.push_str(at_rule);
    out.push_str(" {\n");
    for line in inner_css.trim_end_matches('\n').lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("}\n");
}

impl SlotRecipe {
    /// `scope` と、この recipe が扱う `slots`（anatomy の part 名一覧）を
    /// 指定して空の recipe を作る。
    #[must_use]
    pub const fn new(scope: &'static str, slots: &'static [&'static str]) -> Self {
        Self {
            scope,
            slots,
            base: Vec::new(),
            variants: Vec::new(),
            default_variants: Vec::new(),
            compound_variants: Vec::new(),
            states: Vec::new(),
            starting_style: Vec::new(),
            supports_not_calc_size: Vec::new(),
        }
    }

    /// 指定した `slot` への base 宣言を登録する（builder、自己消費）。
    ///
    /// `slot` が [`SlotRecipe::new`] で宣言した `slots` に含まれない場合、
    /// この登録は [`SlotRecipe::css`] の出力から除外される（fail-closed。
    /// `slots` 未宣言の slot への意図しない CSS 漏出を防ぐ）。
    #[must_use]
    pub fn base(mut self, slot: &'static str, declarations: Vec<Declaration>) -> Self {
        self.base.push(BaseRule { slot, declarations });
        self
    }

    /// 指定した variant 値 `v` が選択されたときの `slot` への宣言を登録する
    /// （builder、自己消費）。
    ///
    /// `slot` が `slots` に含まれない場合、または `v` の `axis()`/`value()`
    /// が識別子として不正な場合は [`SlotRecipe::css`] の出力から除外される。
    #[must_use]
    pub fn variant<V: VariantValue>(
        mut self,
        v: V,
        slot: &'static str,
        declarations: Vec<Declaration>,
    ) -> Self {
        self.variants.push(VariantRule {
            axis: v.axis(),
            value: v.value(),
            slot,
            declarations,
        });
        self
    }

    /// axis `V` の既定 variant 値を登録する（builder、自己消費）。
    ///
    /// [`SlotRecipe::variant_classes`] は選択で指定されなかった axis を
    /// ここで登録した既定値で補完する。
    #[must_use]
    pub fn default_variant<V: VariantValue>(mut self, v: V) -> Self {
        self.default_variants.push(DefaultVariant {
            axis: v.axis(),
            value: v.value(),
        });
        self
    }

    /// `size` 軸の variant 一式を一括登録する（builder、自己消費。イシュー
    /// #1424 の size 規約における「共通生成手段」）。
    ///
    /// `sizes` に列挙した `(Size, declarations)` の組をそれぞれ
    /// [`SlotRecipe::variant`] へ登録したうえで、`Size::Md` を必ず
    /// [`SlotRecipe::default_variant`] として設定する。size 軸を持つ styled
    /// 部品は「既定は必ず `md`」という規約
    /// （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4）を
    /// 個別に手書きすると既定値の設定漏れが起こり得るため、本メソッドは
    /// その設定漏れを構造的に防ぐ（`sizes` に `Size::Md` の宣言が含まれるか
    /// どうかに関わらず、既定 variant としては常に `Size::Md` を登録する。
    /// `Size::Md` 用の宣言そのものを省略したい呼び出し元は `sizes` から
    /// 単に `Size::Md` の要素を除けばよい）。
    ///
    /// 個別の size だけを他 slot・他条件付きで追加登録したい場合は、従来
    /// どおり [`SlotRecipe::variant`] / [`SlotRecipe::default_variant`] を
    /// 直接呼ぶ経路も残っている（本メソッドは追加の共通手段であり、
    /// 既存 API を置き換えない）。
    #[must_use]
    pub fn size_variants(mut self, slot: &'static str, sizes: &[(Size, Vec<Declaration>)]) -> Self {
        for (size, declarations) in sizes {
            self = self.variant(*size, slot, declarations.clone());
        }
        // 呼び出し元 recipe が本メソッド呼び出し以前に `default_variant(Size::...)`
        // で `size` 軸の既定値を設定済みの場合、単純な末尾追加では
        // `variant_classes` の `find`（axis ごとに最初に登録された
        // `default_variant` を採用、`SlotRecipe::variant_classes` rustdoc
        // 参照）が既存の（`Md` ではない）既定値を優先してしまい、「size
        // 軸を持つ styled 部品は既定が必ず `md`」という規約
        // （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md`
        // §4）を満たせない（codex-review #1707 P1 指摘）。同一 axis の
        // 既存エントリを退避してから追加することで、本メソッドが最後に
        // 呼ばれれば必ず `Md` が既定になることを保証する。
        let axis = Size::Md.axis();
        self.default_variants.retain(|d| d.axis != axis);
        self.default_variant(Size::Md)
    }

    /// 複数 variant 軸の組み合わせ条件が満たされたときの `slot` への宣言を
    /// 登録する（builder、自己消費。chakra-ui の `compoundVariants` 相当、
    /// イシュー #604）。
    ///
    /// `conditions` は [`when()`] で作った [`VariantCondition`] の列（AND
    /// 条件、すべて満たされたときのみ適用される）。以下のいずれかに該当する
    /// 規則は [`SlotRecipe::css`] の出力から除外される（fail-closed。既存
    /// `base`/`variant` と同じ「不正入力は panic せず出力から除外する」方針）:
    ///
    /// - `slot` が `slots` に未宣言、または識別子として不正
    /// - いずれかの条件の `axis`/`value` が識別子として不正
    /// - `conditions` が空（base と同義になる無意味な規則の混入防止）
    /// - `conditions` 内に同一 `axis` が重複（[`SlotRecipe::variant_classes`]
    ///   は 1 軸につき高々 1 クラスしか emit しないため、同一軸に異なる値を
    ///   要求する条件は決して同時に一致しない矛盾条件であり、dead CSS の
    ///   混入を防ぐ）
    /// - 条件の `(axis, value)` の組が [`SlotRecipe::variant`] /
    ///   [`SlotRecipe::default_variant`] のいずれにも未登録（axis/value の
    ///   タイポによる dead CSS の混入を防ぐ。検証は `css()` 呼び出し時に
    ///   行うため builder の呼び出し順には依存しない）
    #[must_use]
    pub fn compound_variant(
        mut self,
        conditions: Vec<VariantCondition>,
        slot: &'static str,
        declarations: Vec<Declaration>,
    ) -> Self {
        self.compound_variants.push(CompoundVariantRule {
            conditions,
            slot,
            declarations,
        });
        self
    }

    /// 状態条件（[`StateCondition`]）が満たされたときの `slot` への宣言を
    /// 登録する（builder、自己消費。イシュー #643。既存 dialog/tabs/accordion/
    /// menu/select の `state_css()` が個別に手書きしていたセレクタ組み立てを
    /// recipe 経由へ一本化する）。
    ///
    /// 以下のいずれかに該当する規則は [`SlotRecipe::css`] の出力から除外される
    /// （fail-closed。既存 `base`/`variant`/`compound_variant` と同じ「不正
    /// 入力は panic せず出力から除外する」方針）:
    ///
    /// - `slot` が `slots` に未宣言、または識別子として不正
    /// - [`StateCondition::Attr`] の属性名が識別子として不正
    /// - [`StateCondition::AttrEq`] の属性名・属性値のいずれかが識別子として不正
    #[must_use]
    pub fn state(
        mut self,
        slot: &'static str,
        condition: StateCondition,
        declarations: Vec<Declaration>,
    ) -> Self {
        self.states.push(StateRule {
            slot,
            condition,
            declarations,
        });
        self
    }

    /// `slot` への `@starting-style` 規則（無条件）を登録する（builder、
    /// 自己消費。イシュー #2192）。
    ///
    /// [MDN `@starting-style`] は「要素がまだ存在しない/表示されていない
    /// 状態から遷移する」ときの開始スタイルを宣言する at-rule で、
    /// `transition-behavior: allow-discrete`（[`transition_declarations_allow_discrete`]）
    /// と組み合わせて `display: none` からの遷移（およびその逆）を可能に
    /// する。`slot` が `slots` に未宣言、または識別子として不正な場合は
    /// [`SlotRecipe::css`] の出力から除外される（fail-closed。既存
    /// `base`/`state` と同じ方針）。
    ///
    /// 状態条件付き（`[data-state="open"]` 等に限定した開始スタイル）が
    /// 必要な場合は [`SlotRecipe::starting_style_state`] を使う。
    ///
    /// [MDN `@starting-style`]: https://developer.mozilla.org/en-US/docs/Web/CSS/@starting-style
    #[must_use]
    pub fn starting_style(mut self, slot: &'static str, declarations: Vec<Declaration>) -> Self {
        self.starting_style.push(StartingStyleRule {
            slot,
            condition: None,
            declarations,
        });
        self
    }

    /// `slot` への `@starting-style` 規則（状態条件付き）を登録する
    /// （builder、自己消費。イシュー #2192）。
    ///
    /// [`SlotRecipe::starting_style`] の一般化で、`condition` が Hover 系
    /// （[`StateCondition::Hover`]・[`StateCondition::HoverExcept`] 等）の
    /// 場合は [`SlotRecipe::css`] が規則ごと除外する（fail-closed。
    /// `@starting-style` は遷移開始前の静的スナップショットであり、
    /// `:hover` のような動的擬似クラスの「開始状態」という概念が成立
    /// しないため）。他の除外条件は [`SlotRecipe::state`] と同じ
    /// （`slot` 未宣言・識別子不正）。
    #[must_use]
    pub fn starting_style_state(
        mut self,
        slot: &'static str,
        condition: StateCondition,
        declarations: Vec<Declaration>,
    ) -> Self {
        self.starting_style.push(StartingStyleRule {
            slot,
            condition: Some(condition),
            declarations,
        });
        self
    }

    /// `slot` へ `@supports not (height: calc-size(auto, size))` 配下の
    /// 上書き宣言を登録する（builder、自己消費。イシュー #2192、PR #2289
    /// codex レビュー P1 是正）。[`SlotRecipe::css`] の出力末尾に
    /// `@starting-style`・`@media (hover: hover)` と同様 1 個のブロックへ
    /// 集約される。セレクタは常に無条件の base セレクタ
    /// （`[data-scope="<scope>"][data-part="<slot>"]`）を用いる（`[hidden]`
    /// state 規則は本 slot への通常の base より詳細度が高いため、閉状態は
    /// 引き続き [hidden] state 規則が優先される。カスケード上「開いた
    /// 定常状態の上書き」としてのみ効く）。
    #[must_use]
    pub fn supports_not_calc_size_height(
        mut self,
        slot: &'static str,
        declarations: Vec<Declaration>,
    ) -> Self {
        self.supports_not_calc_size
            .push(BaseRule { slot, declarations });
        self
    }

    /// `slot`（headless の disclosure 系 content パート）へ
    /// `--fandhe-content-height` 連動の高さトランジションを 1 呼び出しで
    /// 適用する（builder、自己消費。イシュー #2192、案 C
    /// 〔`docs/design/collapsible-height-animation.md`〕の中核 preset）。
    ///
    /// [`SlotRecipe::base`]・[`SlotRecipe::state`]（[`StateCondition::Attr`]
    /// `"hidden"`）・[`SlotRecipe::starting_style`]・
    /// [`SlotRecipe::supports_not_calc_size_height`] の 4 登録を一括で
    /// 行う:
    ///
    /// - base: [`content_height_open_declarations`]（`height:
    ///   var(--fandhe-content-height, auto)` 等）
    /// - `[hidden]` state: [`content_height_closed_transition_declarations`]
    ///   （閉じる遷移専用の `transition-timing-function` を自前で宣言し、
    ///   `overflow` の切り替えタイミングを開く遷移と非対称にする。理由は
    ///   同関数 rustdoc 参照、PR #2289 codex レビュー第 2 ラウンド是正）
    /// - `@starting-style`: [`content_height_closed_declarations`]（開く
    ///   遷移の開始点を `height: 0` に固定する。`transition-*` は宣言
    ///   しない）
    /// - `@supports not (height: calc-size(auto, size))`: `height: auto` /
    ///   `overflow: visible` / `transition: none`（PR #2289 codex レビュー
    ///   P1 是正、下記「calc-size() 未対応ブラウザでの表示回帰対策」参照）
    ///
    /// 閉状態のキーに `[hidden]`（[`StateCondition::Attr`]）を採るのは、
    /// `data-state="closed"` ではなく `hidden` 存在属性が headless の
    /// disclosure 系全部品（collapsible `content`・accordion
    /// `item-content` 等）で保証された契約だから（`crates/headless-ui`
    /// 側の共通契約、`docs/design/collapsible-height-animation.md` §5.1）。
    ///
    /// # calc-size() 未対応ブラウザでの表示回帰対策
    ///
    /// [`content_height_open_declarations`] は `height: calc-size(auto,
    /// size)` の progressive enhancement を持つが、これは対応ブラウザの
    /// みで有効になる宣言であり、未対応ブラウザでは直前の `height:
    /// var(--fandhe-content-height, auto)`（wasm-full の実測 px 固定値）が
    /// そのまま残る。本メソッドが従来これを「既知の限界」として放置して
    /// いたのは、本 PR（イシュー #2192）が height トランジションを導入
    /// する**前**（`height: auto` を継続的に評価していた）と比べて表示
    /// 回帰にあたる（PR #2289 codex レビュー P1 指摘）: 開いた後の画面幅
    /// 縮小・画像の遅延読み込み等で内容が固定 px 高さを超えて伸びると、
    /// `overflow: hidden` により本文・操作要素が切り取られてしまう。
    /// [`SlotRecipe::supports_not_calc_size_height`] で登録する
    /// `@supports not (height: calc-size(auto, size))` ブロックが、
    /// 未対応ブラウザに限って開いた定常状態を `height: auto` /
    /// `overflow: visible` へ強制的に戻し、`transition: none` で
    /// トランジション自体も無効化する（`[hidden]` state 規則は本 base
    /// セレクタより詳細度が高いため閉状態には影響しない）。この結果、
    /// 未対応ブラウザは本 PR 適用前と同じ「`auto` に継続追従し、開閉は
    /// 即時（無アニメーション）」という安全な劣化へ戻る。対応ブラウザ
    /// （[`content_height_open_declarations`] rustdoc 参照）では `@supports`
    /// 条件が不成立のためこのブロックは適用されず、`calc-size()` による
    /// アニメーション付きの高さ追従がそのまま有効になる。
    #[must_use]
    pub fn content_height_transition(self, slot: &'static str, duration: MotionDuration) -> Self {
        self.base(slot, content_height_open_declarations(duration))
            .state(
                slot,
                StateCondition::Attr("hidden"),
                content_height_closed_transition_declarations(duration),
            )
            .starting_style(slot, content_height_closed_declarations())
            .supports_not_calc_size_height(
                slot,
                vec![
                    decl("height", "auto"),
                    decl("overflow", "visible"),
                    decl("transition", "none"),
                ],
            )
    }

    /// この slot に属するかどうかを判定する（`slots` 未宣言の slot を
    /// fail-closed で除外するための内部ヘルパ）。
    fn is_declared_slot(&self, slot: &str) -> bool {
        self.slots.contains(&slot)
    }

    /// `(axis, value)` の組が `variant()`（任意 slot）または
    /// `default_variant()` に既に登録済みかどうかを判定する
    /// （[`SlotRecipe::compound_variant`] の fail-closed 検証専用の内部
    /// ヘルパ。axis/value のタイポによる dead CSS 混入を防ぐ）。
    fn is_registered_variant_value(&self, axis: &str, value: &str) -> bool {
        self.variants
            .iter()
            .any(|rule| rule.axis == axis && rule.value == value)
            || self
                .default_variants
                .iter()
                .any(|d| d.axis == axis && d.value == value)
    }

    /// この recipe が生成する静的 CSS 全量を返す（決定的: 同一の `self` に
    /// 対する複数回の呼び出しは常にバイト単位で同一の文字列を返す）。
    ///
    /// 出力順は「base（`slots` の宣言順）→ variants（登録順）→ compound
    /// variants（登録順、イシュー #604）→ states（登録順、イシュー #643）→
    /// `@starting-style`（登録順、1 個のブロックへ集約、イシュー #2192）→
    /// `@media (hover: hover)`（登録順、1 個のブロックへ集約、イシュー
    /// #1425）」。
    /// セレクタは base が `[data-scope="<scope>"][data-part="<slot>"]`、
    /// variant が
    /// `[data-scope="<scope>"][data-part="<slot>"].fd-<scope>--<axis>-<value>`
    /// （詳細度 (0,3,0) が base の (0,2,0) に必ず勝つため、CSS 記述順に
    /// 依存しない上書きを保証する）、compound variant は条件クラスを
    /// 登録順に連結した
    /// `[data-scope="<scope>"][data-part="<slot>"].fd-<scope>--<a1>-<v1>.fd-<scope>--<a2>-<v2>...`
    /// （条件 2 個以上なら詳細度が単一 variant を必ず上回り、1 個の場合でも
    /// 出力順が variants より後のため CSS カスケードの後勝ちで上書きされる）、
    /// state は [`StateCondition`] に応じて
    /// `[data-scope="<scope>"][data-part="<slot>"][<name>]`（`Attr`）・
    /// `[data-scope="<scope>"][data-part="<slot>"][<name>="<value>"]`
    /// （`AttrEq`）・`[data-scope="<scope>"][data-part="<slot>"]:focus-visible`
    /// （`FocusVisible`）・`[data-scope="<scope>"][data-part="<slot>"]:focus-within`
    /// （`FocusWithin`、イシュー #683）・
    /// `[data-scope="<scope>"][data-part="<slot>"]:last-child`
    /// （`LastChild`、イシュー #752）・
    /// `[data-scope="<scope>"][data-part="<slot>"]:hover:not([data-disabled])`
    /// （`Hover`、イシュー #847。イシュー #1425 でタッチ端末の hover
    /// 貼り付き対策として `@media (hover: hover) { ... }` 配下へまとめて
    /// 出力する形へ変更し、`:not([data-disabled])` を付与して disabled 規則
    /// との勝敗を記述順に依存させない契約にした。この `@media` ブロックは
    /// 通常の state 規則がすべて出力された後、[`SlotRecipe::css`] の
    /// 出力の最後尾に 1 つだけ現れる）のいずれか（`Hover` 以外は出力順が
    /// 最後尾のため CSS カスケードの後勝ちで variant/compound variant を
    /// 上書きする。`LastChild` は同一 slot への他の state 規則より後に
    /// 登録することで詳細度が同じでも記述順の後勝ちで上書きする契約、
    /// `state()` の「登録順」規約参照）。
    ///
    /// `scope`（[`SlotRecipe::new`] に渡した値）が識別子として不正な場合は
    /// 空文字列を返す（fail-closed。`slot`/`axis`/`value` と同様に `scope` も
    /// セレクタ・クラス名へそのまま埋め込まれるため、ここで検証しないと
    /// `</style>` やセレクタ脱出を許す構造破壊文字が CSS 生成経路に残ってしまう）。
    #[must_use]
    pub fn css(&self) -> String {
        if !is_valid_identifier(self.scope) {
            return String::new();
        }

        let mut out = String::new();

        for slot in self.slots {
            for rule in self.base.iter().filter(|rule| rule.slot == *slot) {
                if !is_valid_identifier(rule.slot) {
                    continue;
                }
                let selector = format!(
                    "[data-scope=\"{}\"][data-part=\"{}\"]",
                    self.scope, rule.slot
                );
                if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                    out.push_str(&css);
                    out.push('\n');
                }
            }
        }

        for rule in &self.variants {
            if !self.is_declared_slot(rule.slot)
                || !is_valid_identifier(rule.slot)
                || !is_valid_identifier(rule.axis)
                || !is_valid_identifier(rule.value)
            {
                continue;
            }
            let class_name = format!(
                "{CLASS_PREFIX}-{}--{}-{}",
                self.scope, rule.axis, rule.value
            );
            let selector = format!(
                "[data-scope=\"{}\"][data-part=\"{}\"].{class_name}",
                self.scope, rule.slot
            );
            if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                out.push_str(&css);
                out.push('\n');
            }
        }

        'compound: for rule in &self.compound_variants {
            if rule.conditions.is_empty()
                || !self.is_declared_slot(rule.slot)
                || !is_valid_identifier(rule.slot)
            {
                continue;
            }

            // 同一 axis の重複は矛盾条件（variant_classes は 1 軸につき
            // 高々 1 クラスしか emit しないため決して同時に一致しない）と
            // みなし、dead CSS の混入を防ぐため規則ごと除外する。
            let mut seen_axes: Vec<&str> = Vec::new();
            for cond in &rule.conditions {
                if !is_valid_identifier(cond.axis) || !is_valid_identifier(cond.value) {
                    continue 'compound;
                }
                if seen_axes.contains(&cond.axis) {
                    continue 'compound;
                }
                seen_axes.push(cond.axis);
                if !self.is_registered_variant_value(cond.axis, cond.value) {
                    continue 'compound;
                }
            }

            let mut selector = format!(
                "[data-scope=\"{}\"][data-part=\"{}\"]",
                self.scope, rule.slot
            );
            for cond in &rule.conditions {
                selector.push_str(&format!(
                    ".{CLASS_PREFIX}-{}--{}-{}",
                    self.scope, cond.axis, cond.value
                ));
            }
            if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                out.push_str(&css);
                out.push('\n');
            }
        }

        let mut hover_css = String::new();

        for rule in &self.states {
            if !self.is_declared_slot(rule.slot) || !is_valid_identifier(rule.slot) {
                continue;
            }
            let Some(suffix) = state_condition_selector(&rule.condition) else {
                continue;
            };
            let mut selector = format!(
                "[data-scope=\"{}\"][data-part=\"{}\"]",
                self.scope, rule.slot
            );
            selector.push_str(&suffix);
            // Hover/HoverExcept/HoverExceptAttr は states ループの通常出力先
            // ではなく専用バッファへ集約し、css() 末尾で `@media (hover:
            // hover)` に 1 つだけまとめて出す（イシュー #1425、本関数
            // rustdoc の出力順序節参照）。
            let target = if is_hover_family(&rule.condition) {
                &mut hover_css
            } else {
                &mut out
            };
            if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                target.push_str(&css);
                target.push('\n');
            }
        }

        let mut starting_style_css = String::new();

        for rule in &self.starting_style {
            if !self.is_declared_slot(rule.slot) || !is_valid_identifier(rule.slot) {
                continue;
            }
            let mut selector = format!(
                "[data-scope=\"{}\"][data-part=\"{}\"]",
                self.scope, rule.slot
            );
            if let Some(condition) = &rule.condition {
                // Hover 系条件は starting style として意味を持たないため
                // 規則ごと除外する（fail-closed、`StartingStyleRule` rustdoc
                // 参照）。
                if is_hover_family(condition) {
                    continue;
                }
                match state_condition_selector(condition) {
                    Some(suffix) => selector.push_str(&suffix),
                    None => continue,
                }
            }
            if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                starting_style_css.push_str(&css);
                starting_style_css.push('\n');
            }
        }

        if !starting_style_css.is_empty() {
            write_at_rule_block(&mut out, "@starting-style", &starting_style_css);
            // 他の規則ブロック間と同じ「規則間は空行 1 つ」書式
            // （本関数末尾の最終トリムと対をなす）を @media ブロックとの
            // 間にも適用する。
            out.push('\n');
        }

        // `@supports not (height: calc-size(auto, size))` ブロック
        // （イシュー #2192、PR #2289 codex レビュー P1 是正）。
        // `supports_not_calc_size_height` rustdoc 参照。セレクタは常に
        // 無条件の base セレクタを使うため `state`/`@starting-style` の
        // ような条件分岐を持たない（フィルタリングのみ base ループと同じ
        // `is_declared_slot`/`is_valid_identifier` 検証を通す）。
        let mut supports_not_calc_size_css = String::new();
        for rule in &self.supports_not_calc_size {
            if !self.is_declared_slot(rule.slot) || !is_valid_identifier(rule.slot) {
                continue;
            }
            let selector = format!(
                "[data-scope=\"{}\"][data-part=\"{}\"]",
                self.scope, rule.slot
            );
            if let Some(css) = serialize_rule(&selector, &rule.declarations) {
                supports_not_calc_size_css.push_str(&css);
                supports_not_calc_size_css.push('\n');
            }
        }

        if !supports_not_calc_size_css.is_empty() {
            write_at_rule_block(
                &mut out,
                "@supports not (height: calc-size(auto, size))",
                &supports_not_calc_size_css,
            );
            out.push('\n');
        }

        if !hover_css.is_empty() {
            write_at_rule_block(&mut out, "@media (hover: hover)", &hover_css);
        }

        // 末尾の空行は規則ブロック間の区切りとしてのみ入れるため、
        // 最後の 1 つを削って「規則間は空行 1 つ」書式を保つ。
        if out.ends_with("\n\n") {
            out.pop();
        }
        out
    }

    /// variant 値 1 個に対応するクラス名（`fd-<scope>--<axis>-<value>`）を返す。
    ///
    /// `scope`/`axis()`/`value()` のいずれかが識別子として不正な場合は
    /// 空文字列を返す（呼び出し側が不正なクラスを HTML へ書き出すことを防ぐ
    /// fail-closed 動作。`fandhe_frontend_core::render` 経由でエスケープは
    /// されるが、無効なクラス名を出力に混入させないための追加防御）。
    #[must_use]
    pub fn variant_class<V: VariantValue>(&self, v: V) -> String {
        let axis = v.axis();
        let value = v.value();
        if !is_valid_identifier(self.scope)
            || !is_valid_identifier(axis)
            || !is_valid_identifier(value)
        {
            return String::new();
        }
        format!("{CLASS_PREFIX}-{}--{axis}-{value}", self.scope)
    }

    /// axis 名 → value 名の選択列からクラス文字列を組み立てる。
    ///
    /// `selection` で指定されなかった axis は [`SlotRecipe::default_variant`]
    /// で登録した既定値で補完する。戻り値は axis の登録順（`variant`/
    /// `default_variant` で最初に現れた順）で連結したクラス文字列
    /// （スペース区切り、`class="..."` にそのまま渡せる形式）。
    ///
    /// `scope` が識別子として不正な場合は空文字列を返す（[`SlotRecipe::css`]・
    /// [`SlotRecipe::variant_class`] と同じ fail-closed 方針）。
    #[must_use]
    pub fn variant_classes(&self, selection: &[(&str, &str)]) -> String {
        if !is_valid_identifier(self.scope) {
            return String::new();
        }

        let mut axes: Vec<&'static str> = Vec::new();
        for rule in &self.variants {
            if !axes.contains(&rule.axis) {
                axes.push(rule.axis);
            }
        }
        for d in &self.default_variants {
            if !axes.contains(&d.axis) {
                axes.push(d.axis);
            }
        }

        let mut classes: Vec<String> = Vec::new();
        for axis in axes {
            let value = selection
                .iter()
                .find(|(a, _)| *a == axis)
                .map(|(_, v)| *v)
                .or_else(|| {
                    self.default_variants
                        .iter()
                        .find(|d| d.axis == axis)
                        .map(|d| d.value)
                });
            if let Some(value) = value {
                if is_valid_identifier(axis) && is_valid_identifier(value) {
                    classes.push(format!("{CLASS_PREFIX}-{}--{axis}-{value}", self.scope));
                }
            }
        }
        classes.join(" ")
    }
}
