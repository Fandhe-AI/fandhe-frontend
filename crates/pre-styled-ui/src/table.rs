//! styled Table（イシュー #767）: slot recipe 静的部品。root/header/body/
//! footer/row/column-header/cell/caption の 8 パーツで
//! `table`/`thead`/`tbody`/`tfoot`/`tr`/`th`/`td`/`caption` の HTML 意味論を
//! そのまま尊重する（chakra-ui `data-display/table` 相当）。
//!
//! [`crate::card`]・[`crate::alert`] と同型の「状態機械を持たない静的
//! styled 部品」であり、`fandhe-frontend-headless-ui` 側に対応する anatomy は
//! 存在しない（[`crate::checkbox_card`]/[`crate::radio_card`] と同じく、
//! 本クレートで新規 anatomy `data-scope="table"` を定義する）。コンビニ関数
//! （全部入り `table(...)`）は提供せず、各パーツを個別に呼び出して組み立てる
//! 契約とする（[`crate::card`] と同じ判断、呼び出し例は各関数の rustdoc
//! `# Examples` を参照）。
//!
//! # variant（`variant`/`size`/`striped`/`sticky_header`）について
//!
//! [`crate::card`] と異なり 4 軸の variant を持つ（chakra-ui Table の
//! `variant`/`size`/`interactive`/`stickyHeader` のうち `interactive`・
//! `showColumnBorder` はスコープ外、下記参照。`stickyHeader` はイシュー
//! #1571 で実装した）:
//!
//! - [`TableVariant`]: `Line`（既定、行ごとの下線区切り）/ `Outline`
//!   （外枠 + 角丸）。
//! - `size`（[`crate::recipe::Size`]）: セルの padding・font-size を切り替える。
//! - `striped`（`bool`）: 縞模様表示。有効時は本文行の背景色を交互に変える。
//! - `sticky_header`（`bool`）: 有効時、`column-header`（`th`）を
//!   `position: sticky; top: 0` にする（下記「sticky ヘッダーの実装」節参照）。
//!
//! クラスは `root` パーツのみへ付与する（複合部品の variant 統一方針、
//! `crates/pre-styled-ui/src/lib.rs` §「複合部品の variant 統一方針」参照）。
//! `row`/`cell`/`column-header` への伝搬は `root` の variant 宣言が登録する
//! root スコープの CSS custom property（`--fandhe-table-cell-padding` 等）の
//! 通常の CSS 継承で行い、[`crate::recipe::SlotRecipe`] へ子孫セレクタ機構は
//! 追加しない（[`crate::switch`]/[`crate::breadcrumb`] と同型のパターン）。
//!
//! # striped の実装（イシュー #767・[`crate::recipe::StateCondition::NthChildEven`]）
//!
//! `striped` は常に `false`/`true` いずれかのクラスを `root` へ付与する
//! （決定性維持、[`crate::breadcrumb::BreadcrumbVariant`] と同じ「既定値も
//! 明示的に登録する」判断）。`true` 側は root スコープへ
//! `--fandhe-table-stripe-bg: var(--fandhe-color-bg-subtle)` を設定し、`false`
//! 側は `--fandhe-table-stripe-bg: transparent` を明示設定する。`row` slot の
//! [`crate::recipe::StateCondition::NthChildEven`] 規則が
//! `background: var(--fandhe-table-stripe-bg, transparent)` を消費する。
//!
//! `:nth-child(even)` は親要素（`thead`/`tbody`/`tfoot` それぞれ）内の兄弟を
//! 基準に数えるため、通常構成（1 行の `thead` + 複数行の `tbody`）では
//! `tbody` 内の行のみが交互に縞模様になる。`thead` が複数行の場合は 2 行目
//! 以降も対象になりうるが、`column-header`（`th`）は base 規則で背景色を
//! 明示するため視覚的な影響は小さい。
//!
//! # sticky ヘッダーの実装（イシュー #1571）
//!
//! `sticky_header` は `striped` と同型に常に `false`/`true` いずれかの
//! クラスを `root` へ付与し（決定性維持）、root スコープへ
//! `--fandhe-table-header-position`（`static`/`sticky`）・
//! `--fandhe-table-sticky-offset`（常に `0`）の 2 custom property を設定する。
//! `column-header`（`th`）base 規則がこれを
//! `position: var(--fandhe-table-header-position, static); top:
//! var(--fandhe-table-sticky-offset, 0)` として消費する。
//!
//! `thead`/`tr`（`header`/`row` slot）ではなく `column-header`（`th`）へ
//! `position: sticky` を置く理由: 表セルへの sticky 指定はブラウザ横断で
//! 最も安定した適用対象である。chakra-ui は `tr` を対象にするが、
//! [`crate::recipe::SlotRecipe`] は子孫セレクタ機構を持たず（本モジュール doc
//! 「variant について」節参照）、`row` slot へ base 規則を追加すること自体が
//! PR #811 の不変条件（`separate` border モデル下で `tr` への border が
//! 無視される、上記「striped の実装」節と同じ制約源）と衝突しないよう
//! `column-header`/`cell` に閉じる既存方針を踏襲する。
//!
//! sticky 時も `column-header` の `background: var(--fandhe-color-bg)`
//! （既存の base 規則）は維持する。これが無いと sticky 中の見出しの背後に
//! スクロールしてきた本文行が透けて見えてしまう。
//!
//! `z-index` は `var(--fandhe-z-index-docked, 10)` を使う（[`crate::theme`]
//! の z-index スケール、イシュー #1423）。同スケールの `sticky`（1100）は
//! dropdown/popover 帯を越える値であり、単なる「スクロール内で自身の位置に
//! 留まる」sticky ヘッダーには強すぎるため採用しない。
//!
//! [`TableVariant::Outline`] の角丸クリップ（上記「Outline」variant 参照）は
//! `overflow: hidden` ではなく `clip-path: inset(0 round
//! var(--fandhe-radius-lg))` で行う（codex-review P1 是正、下記
//! 「`Outline` の角丸クリップに `overflow` を使わない理由」節参照）ため、
//! `root` 自身をスクロールコンテナ化せず `sticky_header` は `Outline`
//! でもページスクロールへ追従する。ただし `root` 自身がスクロール可能な
//! コンテナ（`overflow-y: auto` 等のスクロール枠）に包まれていない限り、
//! `sticky_header` はページ全体のスクロールでのみ効果を持つ（これは
//! `position: sticky` 自体の一般的な性質であり `Outline`/`Line` を問わない）。
//! スクロール枠との連携（chakra `ScrollArea` 相当）は兄弟イシュー #1572
//! （2/2）のスコープとする。
//!
//! # `Outline` の角丸クリップに `overflow` を使わない理由（イシュー #1571
//! codex-review P1 是正）
//!
//! 当初 `Outline` variant の `root` は角丸クリップに `overflow: hidden` を
//! 使っていた。しかし CSS 仕様上、`overflow` を `visible` 以外の値にする
//! 要素は自動的に「スクロールコンテナ」となり、`position: sticky` の
//! 子孫にとって最も近い祖先スクロールコンテナとして扱われる。`root`
//! （`<table>`）自身はコンテンツに合わせて伸びるだけで実際にはスクロール
//! しないため、`column-header` の sticky 位置決めがこの祖先の
//! スクロールポート基準に固定されてしまい、ページをスクロールしても
//! `column-header` が追従しない（`root` 自身がスクロール可能なコンテナに
//! 包まれる構成でしか効かない）という契約違反を起こしていた。
//!
//! `clip-path` は `overflow` プロパティを変更せずに描画結果だけを
//! クリップするため、上記のスクロールコンテナ化を引き起こさない。
//! [`crate::rating_group`]/[`crate::stat`] が採用済みの「外部リソースを
//! 参照しないインライン `clip-path`」パターンをここでも踏襲し、
//! 半径は `border-radius` 宣言と同じ `--fandhe-radius-lg` custom property
//! を参照することで両宣言の齟齬を防ぐ。
//!
//! # 意図的に参照サイトへ合わせなかった点（イシュー #1571）
//!
//! - **ヘッダー文字の太さ**: chakra-ui は `medium`、Radix Themes Table は
//!   `bold` を使う。本クレートは Table を chakra-ui 由来の部品として
//!   `docs/design/component-coverage-map.md` に位置づけているため chakra 基準
//!   （`medium`）を採用し、Radix の `bold` は採らない。
//! - **striped の偶奇**: chakra-ui は奇数行（`odd`）に縞模様を付けるが、本
//!   クレートは #767 導入時からの偶数行（`even`、
//!   [`crate::recipe::StateCondition::NthChildEven`]）を維持する。視覚上の
//!   優劣が無い選択であり、`odd` 化には新しい `StateCondition` バリアントの
//!   追加が必要になる（既存 `even` を消費している呼び出し側との互換性を
//!   崩さない判断）。
//! - **行・セルの hover/transition**: chakra-ui `interactive` variant の
//!   行ホバー装飾は非採用（下記「スコープ外」節参照）。行は `cursor:
//!   pointer` を持たず `button`/`a` のような操作可能ロールでもないため、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md`
//!   （インタラクション視覚言語）が定義する「インタラクティブ slot」に
//!   該当しない。同じ理由でフォーカスリングも非該当（セルはフォーカス
//!   対象にならない）。
//! - **`data-selected` 等の状態属性**: chakra-ui は `row._selected` の
//!   ような選択状態の消費側規則を持つが、本クレートは `row` slot に対応する
//!   `data-*` の生産者を持たない静的部品であるため追加しない
//!   （消費側規則だけを追加すると `data_attr_vocabulary.rs` が管理しない
//!   暗黙契約を生む）。
//! - **フッターの区切り線**: chakra-ui は `tfoot` に `border-top` を持つが、
//!   `root` の `border-collapse: separate` モデル下では `tfoot`（`footer`
//!   slot）への border 指定はブラウザに無視される（上記「cell」base の
//!   PR #811 不変条件と同型）。body/footer の視覚的な区切りは body 最終行の
//!   `cell` が持つ `border-bottom` に委ねる。
//!
//! # セキュリティ不変条件
//!
//! - セル値・列見出し・caption はすべて呼び出し側が渡す `children`
//!   （`fandhe_frontend_core::text()` 等）としてノード木経由で受け取り、HTML
//!   文字列の直接組み立ては行わない。出力は `render()` の既定エスケープを
//!   必ず経由する（`raw_html()` は使用しない）。
//! - variant クラス名は [`crate::recipe::SlotRecipe::variant_classes`] が
//!   `&'static str` enum 値から決定的に生成し、動的文字列合成を行わない。
//! - 呼び出し側 `attrs` に含まれる `class` は
//!   [`crate::class_attr::drop_class_attr`] で除去してから recipe 生成
//!   クラスと合成するため、`class` 属性は常に単一（呼び出し側からのクラス
//!   偽装・重複混入を防ぐ）。
//! - [`column_header`] の `scope="col"` は関数側で固定するため、呼び出し側
//!   `attrs` に `scope`（大文字小文字無視）が含まれていても除去する
//!   （[`checkbox_card`](crate::checkbox_card) の `drop_reserved` と同型の
//!   fail-closed 判断。重複 `scope` 属性による無効な HTML・意味論の後勝ち
//!   混乱を防ぐ）。
//!
//! # スコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - chakra-ui の `interactive`（クリック可能行のホバー装飾）・
//!   `showColumnBorder`・`ScrollArea` 連携・`ColumnGroup`（`colgroup`/`col`）
//!   は本イシューのスコープ外（PR 本文に記録）。`stickyHeader` はイシュー
//!   #1571 で実装済み（上記「sticky ヘッダーの実装」節参照）。
//! - `size`（[`crate::recipe::Size`]）の各段階の padding/font-size 実値・
//!   `root` 自身をスクロール可能なコンテナに包む `ScrollArea` 連携
//!   （chakra `ScrollArea` 相当。`Outline`/`Line` を問わず必要になる、上記
//!   「sticky ヘッダーの実装」節参照）は兄弟イシュー #1572（2/2）のスコープ。
//! - `examples/headless-pre-styled-ui` の追随・crates.io への公開は公開
//!   イシュー側のスコープ。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, StateCondition, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="table"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("table");

/// [`SlotRecipe::new`] に渡す slot 一覧（recipe とレンダリング関数の両方が
/// この配列を共有し、slot 名の乖離を防ぐ）。
const SLOTS: &[&str] = &[
    "root",
    "header",
    "body",
    "footer",
    "row",
    "column-header",
    "cell",
    "caption",
];

/// Table の見た目 variant（chakra-ui Table の `variant` を最小構成へ縮約）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableVariant {
    /// 行ごとの下線区切り（既定）。
    #[default]
    Line,
    /// 外枠 + 角丸。
    Outline,
}

impl VariantValue for TableVariant {
    fn axis(self) -> &'static str {
        "variant"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Outline => "outline",
        }
    }
}

/// striped variant 値（内部専用、公開 API は `bool` のまま。
/// [`crate::table` モジュール doc](self)「striped の実装」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StripedVariant {
    /// 縞模様なし（既定）。
    Off,
    /// 縞模様あり。
    On,
}

impl VariantValue for StripedVariant {
    fn axis(self) -> &'static str {
        "striped"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Off => "false",
            Self::On => "true",
        }
    }
}

impl From<bool> for StripedVariant {
    fn from(b: bool) -> Self {
        if b {
            Self::On
        } else {
            Self::Off
        }
    }
}

/// sticky ヘッダー variant 値（内部専用、公開 API は `bool` のまま。
/// [`crate::table` モジュール doc](self)「sticky ヘッダーの実装」節参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StickyHeaderVariant {
    /// 通常表示（既定）。
    Off,
    /// `column-header` を `position: sticky` にする。
    On,
}

impl VariantValue for StickyHeaderVariant {
    fn axis(self) -> &'static str {
        "sticky-header"
    }

    fn value(self) -> &'static str {
        match self {
            Self::Off => "false",
            Self::On => "true",
        }
    }
}

impl From<bool> for StickyHeaderVariant {
    fn from(b: bool) -> Self {
        if b {
            Self::On
        } else {
            Self::Off
        }
    }
}

/// Table の呼び出し側公開 props（`root` の引数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableProps {
    /// 見た目 variant（既定 [`TableVariant::Line`]）。
    pub variant: TableVariant,
    /// サイズ（既定 [`Size::Md`]）。
    pub size: Size,
    /// 縞模様表示の有無（既定 `false`）。
    pub striped: bool,
    /// sticky ヘッダーの有無（既定 `false`）。有効時は `column-header`
    /// （`th`）が `position: sticky; top: 0` になる（[`crate::table`
    /// モジュール doc](self)「sticky ヘッダーの実装」節参照）。
    pub sticky_header: bool,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            variant: TableVariant::default(),
            size: Size::Md,
            striped: false,
            sticky_header: false,
        }
    }
}

/// [`column_header`] が固定する属性名（呼び出し側 `attrs` からの偽装を
/// fail-closed で除去する対象）。
const COLUMN_HEADER_RESERVED: &[&str] = &["scope"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crates/pre-styled-ui/src/checkbox_card.rs` の `drop_reserved`
/// と同型）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Table の recipe（scope `"table"`、[`SLOTS`] の 8 パーツ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("table", SLOTS)
        .base(
            "root",
            vec![
                decl("width", "100%"),
                // `border-collapse: collapse` だと `Outline` variant の
                // `border-radius` がブラウザにより無視される（角丸が効かない）ため、
                // `separate` + `border-spacing: 0` を既定にする（イシュー #767
                // PR #811 Bugbot 指摘）。`separate` でもセル間に境界線の重複は
                // 発生しない（`cell`/`column-header` は `border-bottom` のみを
                // 使い、隣接セル間の縦境界線を持たないため見た目に影響しない）。
                // 注意: `separate` モデルでは `row`（`tr`）への border 指定は
                // ブラウザに無視される（CSS 表モデル仕様上、border の対象は
                // table と cell のみ）。そのため Line variant の行区切り線は
                // `row` ではなく `cell`/`column-header` 側に持たせている
                // （下記 `cell` base 参照、PR #811 Bugbot 追加指摘で是正）。
                decl("border-collapse", "separate"),
                decl("border-spacing", "0"),
                decl("text-align", "left"),
            ],
        )
        .base(
            "caption",
            vec![
                decl("caption-side", "bottom"),
                decl("padding", "0.75rem 0"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "column-header",
            vec![
                decl("padding", "var(--fandhe-table-cell-padding, 0.75rem 1rem)"),
                decl(
                    "font-size",
                    "var(--fandhe-table-font-size, var(--fandhe-font-font-size-sm))",
                ),
                // イシュー #1571: chakra-ui Table のヘッダー太さ（medium）へ
                // 合わせる（semibold から変更。Radix Themes の bold は
                // 不採用、モジュール doc「意図的に参照サイトへ合わせなかった
                // 点」節参照）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                // イシュー #1571: 見出しテキストの色を明示する（chakra-ui /
                // Radix Themes とも既定の `fg` トークンで統一しており、
                // 従来は `color` を宣言せずブラウザ既定色に委ねていた）。
                decl("color", "var(--fandhe-color-fg)"),
                decl("text-align", "inherit"),
                // イシュー #1571: chakra-ui / Radix Themes とも 1px であり
                // `2px` は参照サイトより太い（縦罫線がある表と誤認しやすい）。
                // 行罫線に使う `border-muted` より一段強い `border` トークン
                // でヘッダーを区切る。
                decl("border-bottom", "1px solid var(--fandhe-color-border)"),
                // sticky 中に背後の本文行が透けないよう不透明背景を維持する
                // （sticky_header 有無に関わらず既存どおり必須）。
                decl("background", "var(--fandhe-color-bg)"),
                // イシュー #1571: 数値列の桁揃え（chakra-ui root の
                // `tabular-nums` 相当。root ではなくリーフ側へ置く判断は
                // モジュール doc 参照。size/variant 軸は #1572 の担当のため
                // ここでは触らない）。
                decl("font-variant-numeric", "tabular-nums"),
                // イシュー #1571: sticky_header variant（root スコープ）が
                // 設定する custom property を消費する。既定（Off）は
                // `static`/`0` のため見た目に影響しない。
                decl("position", "var(--fandhe-table-header-position, static)"),
                decl("top", "var(--fandhe-table-sticky-offset, 0)"),
                // dropdown/popover 帯（1000/1200）より下、通常のドキュメント
                // フローより上の `docked` 段を使う（モジュール doc
                // 「sticky ヘッダーの実装」節参照）。
                decl("z-index", "var(--fandhe-z-index-docked, 10)"),
            ],
        )
        .base(
            "cell",
            vec![
                decl("padding", "var(--fandhe-table-cell-padding, 0.75rem 1rem)"),
                decl(
                    "font-size",
                    "var(--fandhe-table-font-size, var(--fandhe-font-font-size-sm))",
                ),
                // `root` の `border-collapse: separate`（Outline variant の
                // `border-radius` 対応、直前コミットで導入）下では、CSS の
                // テーブルモデル仕様上 `tr`（`row` スロット）への border は
                // 描画されない（separate モデルでは table と cell のみが
                // border の対象）。そのため Line variant の行区切り線は
                // `row` ではなくここ（`td`/`th` にあたる `cell`/
                // `column-header` スロット）へ持たせる（イシュー #767
                // PR #811 Bugbot 指摘）。
                decl("border-bottom", "var(--fandhe-table-row-border, none)"),
                // イシュー #1571: column-header と同じ理由で数値列の桁揃え
                // を追加する。
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
        .base(
            "footer",
            vec![
                // イシュー #1571: chakra-ui `tfoot` の `font-weight: medium`
                // 相当。border は付けない（`root` の `border-collapse:
                // separate` モデル下では `tfoot`〔footer slot〕への border
                // 指定はブラウザに無視される、上記 `cell` base の PR #811
                // 不変条件と同型。body/footer の区切りは body 最終行の
                // `cell` が持つ `border-bottom` に委ねる）。
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
            ],
        )
        .variant(
            TableVariant::Line,
            "root",
            vec![decl(
                "--fandhe-table-row-border",
                "1px solid var(--fandhe-color-border-muted)",
            )],
        )
        .variant(
            TableVariant::Outline,
            "root",
            vec![
                decl("--fandhe-table-row-border", "none"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-lg)"),
                // `column-header` の不透明背景・striped 偶数行の背景は
                // `root` の `border-radius` に追従してクリップされない
                // （`border-collapse: separate` 下では子孫が親の角丸の
                // 外側にはみ出して矩形の角のまま描画される）。子孫の描画を
                // 角丸内へ収める必要がある（イシュー #767 PR #811 Bugbot
                // 指摘）。
                //
                // イシュー #1571 codex-review P1 是正: 当初 `overflow:
                // hidden` を使っていたが、`overflow` を `visible` 以外に
                // する宣言は CSS 仕様上その要素を `position: sticky` の
                // 「最も近いスクロール祖先」に仕立ててしまい、`root`
                // 自身はスクロールしない（コンテンツに追従して伸びるだけ）
                // ため `sticky_header` がページスクロールに追従しなくなる
                // （下記「sticky ヘッダーの実装」節参照）。`clip-path` は
                // `overflow` を変更せずに視覚的なクリップだけを行うため
                // スクロール祖先化を起こさず、`sticky_header` と共存できる
                // （[`crate::rating_group`]/[`crate::stat`] が採用済みの
                // 「外部リソースを参照しないインライン `clip-path`」
                // パターンをここでも踏襲する）。半径は `border-radius` と
                // 同じ custom property 値を参照し、両宣言が常に一致する
                // ようにする。
                decl("clip-path", "inset(0 round var(--fandhe-radius-lg))"),
            ],
        )
        .default_variant(TableVariant::Line)
        // イシュー #1681: Xs は cell-padding 0.25rem 刻みの等差進行を外挿
        // した (0.25rem, 0.5rem)。font-size はトークン下限 xs をそのまま
        // 使う（Sm と同一。より小さいトークンが存在しないため）。
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-table-cell-padding", "0.25rem 0.5rem"),
                decl(
                    "--fandhe-table-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-table-cell-padding", "0.5rem 0.75rem"),
                decl(
                    "--fandhe-table-font-size",
                    "var(--fandhe-font-font-size-xs)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-table-cell-padding", "0.75rem 1rem"),
                decl(
                    "--fandhe-table-font-size",
                    "var(--fandhe-font-font-size-sm)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-table-cell-padding", "1rem 1.25rem"),
                decl(
                    "--fandhe-table-font-size",
                    "var(--fandhe-font-font-size-md)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-table-cell-padding", "1.25rem 1.5rem"),
                decl(
                    "--fandhe-table-font-size",
                    "var(--fandhe-font-font-size-lg)",
                ),
            ],
        )
        .default_variant(Size::Md)
        .variant(
            StripedVariant::Off,
            "root",
            vec![decl("--fandhe-table-stripe-bg", "transparent")],
        )
        .variant(
            StripedVariant::On,
            "root",
            vec![decl(
                "--fandhe-table-stripe-bg",
                "var(--fandhe-color-bg-subtle)",
            )],
        )
        .default_variant(StripedVariant::Off)
        // イシュー #1571: sticky ヘッダー variant。既定 Off も明示的に登録
        // する（striped と同じ決定性維持の判断、上記「sticky ヘッダーの
        // 実装」節参照）。
        .variant(
            StickyHeaderVariant::Off,
            "root",
            vec![
                decl("--fandhe-table-header-position", "static"),
                decl("--fandhe-table-sticky-offset", "0"),
            ],
        )
        .variant(
            StickyHeaderVariant::On,
            "root",
            vec![
                decl("--fandhe-table-header-position", "sticky"),
                decl("--fandhe-table-sticky-offset", "0"),
            ],
        )
        .default_variant(StickyHeaderVariant::Off)
        .state(
            "row",
            StateCondition::NthChildEven,
            vec![decl(
                "background",
                "var(--fandhe-table-stripe-bg, transparent)",
            )],
        )
}

/// Table の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// root パーツ（`<table>`）を組み立てる。`variant`/`size`/`striped` に応じた
/// クラスを付与する唯一のパーツ（[`drop_class_attr`] により呼び出し側の
/// `class` は除去してから合成する）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
///
/// let node = table::root(TableProps::default(), vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="table" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(props: TableProps, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let recipe = recipe();
    let striped: StripedVariant = props.striped.into();
    let sticky_header: StickyHeaderVariant = props.sticky_header.into();
    let class = recipe.variant_classes(&[
        ("variant", props.variant.value()),
        ("size", props.size.value()),
        ("striped", striped.value()),
        ("sticky-header", sticky_header.value()),
    ]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    ANATOMY.part("root", "table", merged, children)
}

/// header パーツ（`<thead>`）を組み立てる。variant を持たないため `class` は
/// 付与せず、呼び出し側 `attrs` をそのまま連結する。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("header", "thead", attrs, children)
}

/// body パーツ（`<tbody>`）を組み立てる。
#[must_use]
pub fn body<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("body", "tbody", attrs, children)
}

/// footer パーツ（`<tfoot>`）を組み立てる。
#[must_use]
pub fn footer<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("footer", "tfoot", attrs, children)
}

/// row パーツ（`<tr>`）を組み立てる。
#[must_use]
pub fn row<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("row", "tr", attrs, children)
}

/// column-header パーツ（`<th scope="col">`）を組み立てる。列見出しの
/// WAI-ARIA/HTML 意味論（`scope="col"`）を既定で担保する。呼び出し側 `attrs`
/// に `scope` を含めても [`drop_reserved`] により除去される（本モジュール
/// doc「セキュリティ不変条件」節参照）。
#[must_use]
pub fn column_header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&str, &str)> = vec![("scope", "col")];
    merged.extend(drop_reserved(attrs, COLUMN_HEADER_RESERVED));
    ANATOMY.part("column-header", "th", merged, children)
}

/// cell パーツ（`<td>`）を組み立てる。
#[must_use]
pub fn cell<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("cell", "td", attrs, children)
}

/// caption パーツ（`<caption>`）を組み立てる。呼び出し側は `<table>` の
/// 直接の子として `root` の `children` 先頭に置く必要がある（HTML 仕様上
/// `caption` は `table` の最初の子でなければならない。本関数自体は順序を
/// 強制しない）。
#[must_use]
pub fn caption<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("caption", "caption", attrs, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};

    #[test]
    fn default_variant_is_line_md_not_striped() {
        let html = render(&root(TableProps::default(), vec![], vec![]));
        assert!(html.contains("fd-table--variant-line"));
        assert!(html.contains("fd-table--size-md"));
        assert!(html.contains("fd-table--striped-false"));
        assert!(html.contains("fd-table--sticky-header-false"));
    }

    #[test]
    fn variant_enumeration_maps_to_expected_classes() {
        for (variant, class) in [
            (TableVariant::Line, "fd-table--variant-line"),
            (TableVariant::Outline, "fd-table--variant-outline"),
        ] {
            let props = TableProps {
                variant,
                ..TableProps::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(class), "variant={variant:?} -> {html}");
        }
    }

    #[test]
    fn size_enumeration_maps_to_expected_classes() {
        for (size, class) in [
            (Size::Sm, "fd-table--size-sm"),
            (Size::Md, "fd-table--size-md"),
            (Size::Lg, "fd-table--size-lg"),
        ] {
            let props = TableProps {
                size,
                ..TableProps::default()
            };
            let html = render(&root(props, vec![], vec![]));
            assert!(html.contains(class), "size={size:?} -> {html}");
        }
    }

    #[test]
    fn striped_true_maps_to_expected_class() {
        let props = TableProps {
            striped: true,
            ..TableProps::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains("fd-table--striped-true"));
    }

    #[test]
    fn sticky_header_true_maps_to_expected_class() {
        let props = TableProps {
            sticky_header: true,
            ..TableProps::default()
        };
        let html = render(&root(props, vec![], vec![]));
        assert!(html.contains("fd-table--sticky-header-true"));
    }

    #[test]
    fn parts_use_expected_tags_and_data_part() {
        assert!(render(&header(vec![], vec![]))
            .starts_with(r#"<thead data-scope="table" data-part="header""#));
        assert!(render(&body(vec![], vec![]))
            .starts_with(r#"<tbody data-scope="table" data-part="body""#));
        assert!(render(&footer(vec![], vec![]))
            .starts_with(r#"<tfoot data-scope="table" data-part="footer""#));
        assert!(
            render(&row(vec![], vec![])).starts_with(r#"<tr data-scope="table" data-part="row""#)
        );
        assert!(
            render(&cell(vec![], vec![])).starts_with(r#"<td data-scope="table" data-part="cell""#)
        );
        assert!(render(&caption(vec![], vec![]))
            .starts_with(r#"<caption data-scope="table" data-part="caption""#));
    }

    #[test]
    fn column_header_fixes_scope_col_and_drops_caller_scope() {
        let html = render(&column_header(vec![("scope", "row")], vec![]));
        assert!(html.starts_with(r#"<th data-scope="table" data-part="column-header""#));
        assert!(html.contains(r#"scope="col""#));
        assert!(!html.contains(r#"scope="row""#));
        assert_eq!(html.matches(r#" scope=""#).count(), 1);
    }

    #[test]
    fn composed_table_snapshot() {
        let node = root(
            TableProps::default(),
            vec![],
            vec![
                caption(vec![], vec![text("Users")]),
                header(
                    vec![],
                    vec![row(vec![], vec![column_header(vec![], vec![text("Name")])])],
                ),
                body(
                    vec![],
                    vec![row(vec![], vec![cell(vec![], vec![text("Alice")])])],
                ),
            ],
        );
        let html = render(&node);
        assert_eq!(
            html,
            concat!(
                r#"<table data-scope="table" data-part="root" class="fd-table--variant-line fd-table--size-md fd-table--striped-false fd-table--sticky-header-false">"#,
                r#"<caption data-scope="table" data-part="caption">Users</caption>"#,
                r#"<thead data-scope="table" data-part="header">"#,
                r#"<tr data-scope="table" data-part="row">"#,
                r#"<th data-scope="table" data-part="column-header" scope="col">Name</th>"#,
                r#"</tr>"#,
                r#"</thead>"#,
                r#"<tbody data-scope="table" data-part="body">"#,
                r#"<tr data-scope="table" data-part="row">"#,
                r#"<td data-scope="table" data-part="cell">Alice</td>"#,
                r#"</tr>"#,
                r#"</tbody>"#,
                r#"</table>"#,
            )
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let html = render(&root(
            TableProps::default(),
            vec![("class", "attacker-controlled")],
            vec![],
        ));
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn xss_payload_in_cell_and_column_header_children_is_escaped() {
        let cell_html = render(&cell(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!cell_html.contains("<script>"));
        assert!(cell_html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));

        let header_html = render(&column_header(
            vec![],
            vec![text("<img src=x onerror=alert(1)>")],
        ));
        assert!(!header_html.contains("<img"));
        assert!(header_html.contains("&lt;img"));

        let caption_html = render(&caption(vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!caption_html.contains("<script>"));
    }

    #[test]
    fn css_output_declares_striped_and_size_tokens() {
        let out = css();
        assert!(out.contains(":nth-child(even)"));
        assert!(out.contains("--fandhe-table-stripe-bg"));
        assert!(out.contains("--fandhe-table-cell-padding"));
        assert!(out.contains("--fandhe-table-header-position"));
        assert!(out.contains("position: var(--fandhe-table-header-position, static);"));
        assert!(!out.contains('<'));
    }

    /// イシュー #1571: `column-header` base 規則が chakra-ui / Radix Themes
    /// 基準の 1px 罫線・medium 太さになっていることを固定する
    /// （旧 2px semibold からの是正、上記モジュール doc「意図的に参照
    /// サイトへ合わせなかった点」節参照）。
    #[test]
    fn column_header_uses_one_pixel_border_and_medium_weight() {
        let out = css();
        assert!(out.contains("border-bottom: 1px solid var(--fandhe-color-border);"));
        assert!(out.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
        assert!(!out.contains("2px solid var(--fandhe-color-border-muted)"));
    }

    /// イシュー #1571: `footer`（`tfoot`）base 規則が medium 太さのみを持ち、
    /// border を持たないことを固定する（`separate` border モデル下では
    /// `tfoot` への border 指定が無効なため、上記モジュール doc「sticky
    /// ヘッダーの実装」節と対をなす PR #811 型の不変条件）。
    #[test]
    fn footer_has_medium_weight_and_no_border_rule() {
        let out = css();
        let footer_rule_start = out
            .find(r#"[data-scope="table"][data-part="footer"] {"#)
            .expect("footer base 規則が css() 出力に存在すること");
        let footer_rule_end = out[footer_rule_start..]
            .find('}')
            .map(|offset| footer_rule_start + offset)
            .expect("footer base 規則が `}` で閉じられていること");
        let footer_rule = &out[footer_rule_start..footer_rule_end];
        assert!(footer_rule.contains("font-weight: var(--fandhe-font-font-weight-medium);"));
        assert!(!footer_rule.contains("border"));
    }
}
