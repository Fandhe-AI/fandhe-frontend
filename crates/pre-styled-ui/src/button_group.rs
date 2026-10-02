//! styled Button Group（イシュー #2060、親 #2058、祖父トラッキング参照軸
//! #2001。headless 側 anatomy は #2059）。
//!
//! `fandhe_frontend_headless_ui::button_group`（#2059）が出力する
//! `data-scope="button-group"` の `root`/`separator`/`text` 3 slot へ、
//! shadcn/ui の Button Group（`docs/design/component-coverage-map.md`
//! shadcn/ui 参照軸）の意匠（先頭・末尾だけ角丸を残す連結表示、隣接要素の
//! 境界線が二重に描かれない処理）を重ねる薄い委譲層である。
//!
//! # 選択的 re-export（`pub use ...::*` を使わない理由）
//!
//! 本モジュールは 3 パーツ（[`root`]/[`separator`]/[`text`]）をすべて
//! 同名再定義する（見た目クラスは付与しないが、呼び出し側 `class` の除去は
//! 本モジュールの責務のため）。[`Orientation`] のみを選択的に再エクスポート
//! する（規約 A、`crate::lib` 「headless 再エクスポートの形式規約
//! （イシュー #1062）」節。同型の先例は [`crate::tabs`]/[`crate::radio_card`]/
//! [`crate::menubar`]）。
//!
//! # 状態機械を持たない理由
//!
//! headless [`fandhe_frontend_headless_ui::button_group`] 自身が「props から
//! 決定的にマークアップを組み立てる純粋関数群」（状態機械なし、headless 側
//! モジュール doc 参照）として実装されているため、本モジュールもその設計を
//! そのまま継承する（[`crate::input_group`] モジュール doc と同型の判断）。
//!
//! # 責務境界（`docs/policy/intentional-non-adoption.md` §3.25 規則 1）
//!
//! アプリケーションロジック（クリックハンドラ・状態管理）は実装しない。
//! headless が出力する `data-orientation` を CSS セレクタとして参照する
//! だけで見た目を切り替える。本モジュール自身は独自の `data-*` を一切
//! 出力しない。
//!
//! # variant 軸: 持たない
//!
//! `size`/`variant`/`color-palette` いずれの軸も提供しない
//! （`docs/design/pre-styled-ui-focus-ring-and-size-conventions.md` §4 (d)
//! 「子の寸法に従属するレイアウト部品」に該当。高さ・文字サイズは内側の
//! [`crate::button`](mod@crate::button)/[`crate::input`](mod@crate::input) 等の `size` に従属する）。3 パーツ
//! とも見た目クラスを一切付与しない（呼び出し側 `class` は
//! `drop_class_attr` で除去のみ行う）。
//!
//! # raw CSS 追記の理由（[`SlotRecipe`] が子結合子を表現できないため）
//!
//! [`SlotRecipe`] はコンポーネント自身の slot にしか宣言を登録できず、
//! 子孫（内側の [`crate::button`](mod@crate::button)/[`crate::input`](mod@crate::input)/[`crate::menu`]/
//! [`crate::select`]）を対象にした宣言を組めない。このため [`stylesheet`]
//! は [`crate::input_group::stylesheet`]/[`crate::toggle_group::stylesheet`]
//! と同型のパターンで、`recipe().css()` の出力へ [`crate::css::serialize_rule`]
//! を使った素の子結合子（`>`）セレクタを追記する。
//!
//! 対象となる「直接の子」は汎用セレクタ（`> *:not(:first-child)`）ではなく
//! **明示列挙**する: 汎用セレクタは特異度 (0,3,0) となり、[`crate::button`](mod@crate::button)
//! の `.fd-button--variant-*`/`.fd-button--size-*` クラス規則（同じく
//! (0,3,0)）と同順位になり、呼び出し側の `push_css` 順序に勝敗が依存して
//! しまう（decision unstable）。明示列挙した子セレクタは属性 5 個 +
//! `:not(:first-child)` = 特異度 (0,6,0) 以上とし、`button`/`select`/`menu`
//! の variant・base 規則を確実に上書きする。
//!
//! 列挙する子は 4 種:
//! - `[data-scope="button"][data-part="root"]`（[`crate::button`](mod@crate::button)）
//! - `[data-scope="field"][data-part="input"]`（[`crate::input`](mod@crate::input)。`width:
//!   100%` を持つため `flex: 1 1 auto; min-width: 0` も併せて付与する。
//!   [`crate::input_group`] は root が `width: 100%`（定幅）のため
//!   `flex: 1 1 0%` で伸長・縮小の双方が成立するが、本部品の root は
//!   `width: fit-content`（shrink-to-fit）であり、`flex-basis: 0%` は
//!   fit-content コンテナの内在サイズ計算へ寄与せず Input 幅が実質ゼロへ
//!   潰れる（Cursor Bugbot 指摘、PR #2228）。`flex-basis: auto` は
//!   `width` が percentage かつコンテナが indefinite main size のとき
//!   内在サイズ計算上は content ベースへフォールバックする仕様のため、
//!   fit-content ルートでも自然な内在幅から出発しつつ `flex-shrink: 1`
//!   による縮小は維持できる）
//! - `[data-scope="button-group"][data-part="text"]`（自分自身の [`text`]）
//! - `[data-scope="menu"][data-part="root"]`（[`crate::menu`]。menu は
//!   `root(div, position: relative)` → `trigger(button)` の 2 段構成のため、
//!   グループの直接の子になる `root` を `display: inline-flex; align-self:
//!   stretch` で伸長させ、角丸・境界線の規則は 1 段深い `trigger` へ書く）
//! - `[data-scope="select"][data-part="root"]`（[`crate::select`]。select は
//!   `root` → `control(inline-flex)` → `trigger(button)` の 3 段構成のため、
//!   同様に `root` を伸長させ、角丸・境界線は `control` を経由した
//!   `trigger` へ書く）
//!
//! horizontal/vertical それぞれで、先頭以外の開始側 border 幅を 0 にし
//! （shadcn の `rounded-l-none border-l-0` 相当。負マージンではなく border
//! 幅を 0 にすることで `border: none` の variant でも no-op で無害になる）、
//! 末尾以外の終了側角丸を 0 にする。論理プロパティ（`border-start-start-
//! radius` 等）を使うため RTL でも自然に反転する。
//!
//! **位置擬似クラスは直接の子へ付与する**: `:not(:first-child)`/
//! `:not(:last-child)` は「直接の子であるか」を判定する擬似クラスのため、
//! menu/select のように子孫（`trigger`）へ降りるセレクタでは、擬似クラスを
//! **直接の子（`root`）側**に付け、そこから `>` で子孫へ降りる形
//! （`root:not(:first-child) > trigger`）にする。子孫セレクタの末尾へ
//! 付けてしまう（`root > trigger:not(:first-child)`）と、`trigger` は
//! 常にその親 `root` の唯一の子であるため `:first-child`/`:last-child` が
//! 常に真になり、規則が never-match の dead CSS になる（実装時に発見・
//! 修正した回帰）。
//!
//! # フォーカスリングの重なり回避
//!
//! 実フォーカス対象（button/input/menu-trigger/select-trigger）へ
//! `<target>:focus-visible { position: relative; z-index: 1; }` を付与し、
//! 隣接要素の border に外側リングが隠れないようにする（shadcn
//! `[&>*]:focus-visible:z-10 relative` 相当）。`text` はネイティブ `<div>`
//! でありフォーカス不能なため対象から除外する（dead CSS を書かない）。
//!
//! # ネスト（親 `gap` の `:has()` 不採用と代替）
//!
//! shadcn は `:has(>[data-slot=button-group])` で親に `gap` を与えるが、
//! 親側の `:has()` 判定は #2226 で不採用とした。代替として、内側 [`root`]
//! が直接の子として並ぶときのみ `margin-inline-start`（horizontal）/
//! `margin-block-start`（vertical）でグループ間隔を表現する。内側 `root`
//! 自身は上記の角丸連結対象へは**含めない**（内側グループは自分の角丸を
//! 保つ）。
//!
//! ## `data-attached` opt-in（イシュー #3135、入力欄の縦横混在連結）
//!
//! 呼び出し側が内側 [`root`] の `attrs` へ値なし属性 `data-attached`
//! （chakra-ui の `attached` prop に合わせた命名、先例は #3127 の
//! `data-subtle`）を付与すると、既定の margin 間隔を打ち消し、内側
//! グループの接続辺の角丸・境界線幅を外側 root と共有する 1 つの連結
//! 表示にする。付与しない既定の挙動（margin 間隔のまま）は不変（opt-in
//! のみ、既存出力は変えない）。縦の外側グループでは内側グループ自身も
//! `align-self: stretch` で外側の幅まで伸びる（下段の横並び入力欄は既存
//! の `flex: 1 1 auto` で幅を分け合う）。既知の上限: 横の外側グループの
//! 中に縦の attached グループを置き、兄弟の方が高いとき、内側の子は高さ
//! 方向へ伸びず下端に隙間が出る（未実装、イシュー #3135 スコープ外）。
//!
//! ## 接続辺の判定規則（PR #3569 レビュー対応）
//!
//! 外側 root の直接の子は 3 種に分かれる: 通常の子 N（button/input/text/
//! menu/select、[`CONNECTABLE_CHILDREN`]）・attached 内側 root A
//! （[`ATTACHED_ROOT`]）・間隔を保つ通常の内側 root D（[`DETACHED_ROOT`]）。
//! 隣接する 2 兄弟の辺が**接続される（角丸・境界線幅を解除し margin を
//! 打ち消す）のは、どちらも D でないときに限る**。D だけが接続を切る。
//!
//! | 前 \ 後 | N | A | D |
//! |---------|---|---|---|
//! | N | 接続 | 接続 | 間隔 |
//! | A | 接続 | 接続 | 間隔 |
//! | D | 間隔 | 間隔 | 間隔 |
//!
//! N と A は同じ規則で判定する（`:not(:first-child):not(D + 自分)` が
//! 先頭辺、`:not(:last-child):not(:has(+ D))` が末尾辺）。「隣が A か」
//! を条件にすると N–A が切れ（Cursor Bugbot 指摘）、位置だけを条件に
//! すると A–D が接続されてしまう（codex 指摘）ため、両者を満たす条件は
//! 「隣が D でない」である。末尾辺の判定は後続兄弟を参照するため
//! `:has(+ D)`（Selectors Level 4）を使う。上記の親 `gap` 不採用は
//! 親側で子の有無を判定する用法に対するもので、兄弟結合子を引数に取る
//! 本用法とは別。CSS に兄弟の先読み手段が他になく、DOM 側で接続有無を
//! 属性化すると呼び出し側が兄弟構成を二重管理することになるため採用。
//!
//! # セキュリティ不変条件
//!
//! - 全出力は headless [`fandhe_frontend_headless_ui::button_group`] →
//!   `fandhe_frontend_core::render` の既定エスケープ（REQ-1）を必ず
//!   経由する。`raw_html()` は使用しない。
//! - 呼び出し側 `class` は `drop_class_attr` で除去してから headless
//!   関数へ委譲する。
//! - [`stylesheet`] が組み立てる CSS 宣言・selector 断片はすべて
//!   コンパイル時静的リテラルであり、[`crate::css::decl`]/
//!   [`crate::css::serialize_rule`] の `is_valid_value`/`is_valid_identifier`
//!   検証を通る値のみを使う（動的値を混入させない）。
//!
//! # スコープ外
//!
//! - wasm-full 側の配線（不要。静的グループでキーボード操作はネイティブ
//!   `button` の Tab 順序のみ）。
//! - [`text`] の variant/size 軸。
//! - `:has()` 依存のネスト gap（上記代替 margin で表現）。

use crate::class_attr::drop_class_attr;
use crate::css::{decl, serialize_rule};
use crate::recipe::{SlotRecipe, StateCondition};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

// headless 型のうち見た目クラスを付与しない本モジュールが必要とするのは
// 向き列挙のみ（規約 A）。パーツ関数 3 件は呼び出し側 `class` の除去を
// 担うため同名再定義する。
pub use fandhe_frontend_headless_ui::data_attrs::Orientation;

/// slot 一覧（headless [`fandhe_frontend_headless_ui::button_group`] の
/// anatomy と 1:1、3 パーツ）。
const SLOTS: &[&str] = &["root", "separator", "text"];

/// この styled Button Group の既定 CSS を組み立てる（内部ヘルパ、
/// [`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("button-group", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "stretch"),
                decl("width", "fit-content"),
                decl("max-width", "100%"),
                decl("min-width", "0"),
                decl("box-sizing", "border-box"),
            ],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("flex-direction", "column")],
        )
        .state(
            "root",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            // headless は `data-orientation` を常に出力する（省略しない）
            // ため、`StyleSheet` への連結順に依存させず横向きも明示する。
            vec![decl("flex-direction", "row")],
        )
        .base(
            "separator",
            vec![
                decl("background", "var(--fandhe-color-border)"),
                decl("align-self", "stretch"),
                decl("flex-shrink", "0"),
                decl("margin", "0"),
            ],
        )
        .state(
            "separator",
            StateCondition::AttrEq("data-orientation", "vertical"),
            vec![decl("width", "1px")],
        )
        .state(
            "separator",
            StateCondition::AttrEq("data-orientation", "horizontal"),
            vec![decl("height", "1px"), decl("width", "100%")],
        )
        .base(
            "text",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
                decl("box-sizing", "border-box"),
                decl("padding", "0 var(--fandhe-space-4)"),
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-weight", "var(--fandhe-font-font-weight-medium)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("box-shadow", "var(--fandhe-shadow-xs)"),
            ],
        )
}

/// 角丸連結・境界線二重描画解消の対象となる「直接の子」セレクタ断片。
/// モジュール doc「raw CSS 追記の理由」節参照。
const CONNECTABLE_CHILDREN: &[&str] = &[
    r#"[data-scope="button"][data-part="root"]"#,
    r#"[data-scope="field"][data-part="input"]"#,
    r#"[data-scope="button-group"][data-part="text"]"#,
    r#"[data-scope="menu"][data-part="root"]"#,
    r#"[data-scope="select"][data-part="root"]"#,
];

const ROOT: &str = r#"[data-scope="button-group"][data-part="root"]"#;

/// 入れ子 + `data-attached` opt-in（イシュー #3135）対象の内側 root
/// セレクタ。呼び出し側が値なし属性 `data-attached` を明示付与した
/// ときのみ一致する（モジュール doc「ネスト」節参照）。
const ATTACHED_ROOT: &str = r#"[data-scope="button-group"][data-part="root"][data-attached]"#;

/// 間隔を保つ通常の内側 root（`data-attached` なし）セレクタ。外側リストで
/// 接続を**切る**唯一の兄弟種であり、角丸・境界線幅の解除と margin 打ち消し
/// の `:not()` 除外に使う（モジュール doc「接続辺の判定規則」節参照）。
const DETACHED_ROOT: &str = r#"[data-scope="button-group"][data-part="root"]:not([data-attached])"#;

/// [`CONNECTABLE_CHILDREN`] の各セレクタから、(1) `:not(:first-child)`/
/// `:not(:last-child)` を付与すべき**直接の子**セレクタと、(2) そこから
/// 実際の描画・フォーカス対象（menu/select は 1 段深い `trigger`）へ降りる
/// 子孫セレクタの接尾辞、の 2 つを返す。
///
/// 位置擬似クラス（`:not(:first-child)` 等）は「直接の子である」ことに
/// 対して判定されるため、**必ず直接の子セレクタへ付与し、子孫セレクタの
/// 末尾へ付けてはならない**（menu/select の `trigger`/`control > trigger`
/// は常にその親の唯一の子であり、末尾に付けると `:first-child`/
/// `:last-child` が常に真になって規則自体が never-match になる）。
///
/// 角丸連結規則は `{direct}:not(:first-child){suffix}` の形にする
/// （擬似クラスは `direct` に、子孫降下は `suffix` で表現）。一方
/// `:focus-visible` は実フォーカス対象へ直接判定させたいため
/// `{direct}{suffix}:focus-visible` の形にする（`suffix` が空の
/// button/input/text では両者とも同一の文字列になる）。
fn connectable_target(child: &'static str) -> (&'static str, &'static str) {
    match child {
        r#"[data-scope="menu"][data-part="root"]"# => {
            (child, r#" > [data-scope="menu"][data-part="trigger"]"#)
        }
        r#"[data-scope="select"][data-part="root"]"# => (
            child,
            r#" > [data-scope="select"][data-part="control"] > [data-scope="select"][data-part="trigger"]"#,
        ),
        other => (other, ""),
    }
}

/// `out` へ 1 規則を追記する（空でなければ改行区切り）。
fn append_rule(out: &mut String, rule: Option<String>) {
    if let Some(rule) = rule {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&rule);
    }
}

/// この styled Button Group が生成する静的 CSS 全量を返す（決定的。
/// [`crate::input_group::stylesheet`] と同じ契約）。子孫（button/input/
/// menu/select）を対象にした raw CSS 追記を含む（モジュール doc
/// 「raw CSS 追記の理由」節参照）。
#[must_use]
pub fn stylesheet() -> String {
    let mut out = recipe().css();

    // menu/select は grouping 直接の子が中間コンテナ（root）であり、
    // 角丸連結の対象は 1 段深いボタン相当（trigger）になる。ここではまず
    // 直接の子である root 自体を伸長させる（角丸連結の対象へは含めない）。
    for wrapper in [
        r#"[data-scope="menu"][data-part="root"]"#,
        r#"[data-scope="select"][data-part="root"]"#,
    ] {
        let selector = format!("{ROOT} > {wrapper}");
        append_rule(
            &mut out,
            serialize_rule(
                &selector,
                &[
                    decl("display", "inline-flex"),
                    decl("align-self", "stretch"),
                ],
            ),
        );
    }

    for orientation in ["horizontal", "vertical"] {
        let orientation_root = format!(r#"{ROOT}[data-orientation="{orientation}"]"#);

        // 接続辺の判定（モジュール doc「接続辺の判定規則」節）: 隣接する
        // 兄弟が [`DETACHED_ROOT`]（間隔を保つ通常の内側 root）のときだけ
        // `:not(:first-child)`/`:not(:last-child)` の角丸除去を除外する。
        //
        // - 先頭側は forward 結合子 `{DETACHED_ROOT} + {direct}` で直接表現
        //   できる（左側が直前の兄弟、右側が自分自身）。
        // - 末尾側は `:has(+ {DETACHED_ROOT})`（Selectors Level 4）を使う。
        //   兄弟結合子は後続要素しか選べないため、「直後に特定の兄弟が
        //   続くか」を自分自身の条件へ含める手段が他にない。
        for child in CONNECTABLE_CHILDREN {
            let (direct, suffix) = connectable_target(child);

            let (start_radius, start_width, end_radius) = if orientation == "horizontal" {
                (
                    [
                        decl("border-start-start-radius", "0"),
                        decl("border-end-start-radius", "0"),
                    ],
                    decl("border-inline-start-width", "0"),
                    [
                        decl("border-start-end-radius", "0"),
                        decl("border-end-end-radius", "0"),
                    ],
                )
            } else {
                (
                    [
                        decl("border-start-start-radius", "0"),
                        decl("border-start-end-radius", "0"),
                    ],
                    decl("border-block-start-width", "0"),
                    [
                        decl("border-end-start-radius", "0"),
                        decl("border-end-end-radius", "0"),
                    ],
                )
            };

            // 位置擬似クラスは直接の子（`direct`）へ付与し、menu/select は
            // そこから子孫（`suffix`）へ降りる（`connectable_target` doc
            // 参照。擬似クラスを子孫セレクタの末尾へ付けると、trigger が
            // 常にその親の唯一の子であるため :first-child/:last-child が
            // 常に真になり規則が never-match になる）。
            let not_first = format!(
                "{orientation_root} > {direct}:not(:first-child):not({DETACHED_ROOT} + {direct}){suffix}"
            );
            append_rule(
                &mut out,
                serialize_rule(&not_first, &[start_radius[0], start_radius[1], start_width]),
            );

            let not_last = format!(
                "{orientation_root} > {direct}:not(:last-child):not(:has(+ {DETACHED_ROOT})){suffix}"
            );
            append_rule(
                &mut out,
                serialize_rule(&not_last, &[end_radius[0], end_radius[1]]),
            );
        }

        if orientation == "horizontal" {
            // 横並び時のみ、`crate::input` の `width: 100%` 基底規則が
            // flex 行内で縮小できず幅の狭い親でグループごとはみ出すのを
            // 防ぐため、子 `field/input` へ `flex: 1 1 auto; min-width: 0`
            // を付与する。`crate::input_group::stylesheet` は root が
            // `width: 100%`（定幅）のため `flex-basis: 0%` で成立するが、
            // 本部品の root は `width: fit-content` であり
            // `flex-basis: 0%` は fit-content の内在サイズ計算に寄与せず
            // Input 幅が実質ゼロへ潰れる（Cursor Bugbot 指摘、PR #2228。
            // モジュール doc 冒頭の子セレクタ列挙節も参照）。
            // `flex-basis: auto` は `width` が percentage かつ main size
            // が indefinite なコンテナでは内在サイズ計算上 content
            // ベースへフォールバックする仕様のため、fit-content ルート
            // でも自然な内在幅から出発しつつ `flex-shrink: 1` による
            // 縮小（幅の狭い親での折返し防止）は維持できる。縦積み時は
            // 主軸が block 方向のため `width: 100%` のみで収まり本規則は
            // 不要（適用すると主軸方向の伸長指定になり意味が変わるため
            // orientation で限定する）。
            let input_selector =
                format!(r#"{orientation_root} > [data-scope="field"][data-part="input"]"#);
            append_rule(
                &mut out,
                serialize_rule(
                    &input_selector,
                    &[decl("flex", "1 1 auto"), decl("min-width", "0")],
                ),
            );
        } else {
            // 縦積み時、menu/select の中間ラッパー root への
            // `align-self: stretch`（上記ループ）は root 自身の box しか
            // 伸ばさず、その内側の control/trigger までは伝播しない
            // （`align-self` は「自分がどう配置されるか」であり子孫の
            // サイズには無関係）。境界線・角丸の連結対象である trigger
            // （select は `control` 経由）まで明示的に `width: 100%` を
            // 伝播させ、縦積み時の左右境界を隣接パーツと揃える。
            // trigger/control は UA 既定の `box-sizing: content-box` の
            // ままだと `width: 100%` が padding・border を含まず、
            // stretch した親 root より実際の描画幅が広くなり群の左右端が
            // 揃わなくなる（Cursor Bugbot 指摘、PR #2228）。他部品と同じ
            // `width: 100%` + `box-sizing: border-box` の対で明示する。
            for selector in [
                format!(
                    r#"{orientation_root} > [data-scope="menu"][data-part="root"] > [data-scope="menu"][data-part="trigger"]"#
                ),
                format!(
                    r#"{orientation_root} > [data-scope="select"][data-part="root"] > [data-scope="select"][data-part="control"]"#
                ),
                format!(
                    r#"{orientation_root} > [data-scope="select"][data-part="root"] > [data-scope="select"][data-part="control"] > [data-scope="select"][data-part="trigger"]"#
                ),
            ] {
                append_rule(
                    &mut out,
                    serialize_rule(
                        &selector,
                        &[decl("width", "100%"), decl("box-sizing", "border-box")],
                    ),
                );
            }
        }

        // ネストしたグループ（内側 root）は角丸連結の対象へ含めず、代わりに
        // 先頭以外の内側グループへ間隔を付与する（`:has()` 不採用の代替、
        // モジュール doc「ネスト」節参照）。
        let nested_selector = format!(
            r#"{orientation_root} > [data-scope="button-group"][data-part="root"]:not(:first-child)"#
        );
        let margin = if orientation == "horizontal" {
            decl("margin-inline-start", "var(--fandhe-space-2)")
        } else {
            decl("margin-block-start", "var(--fandhe-space-2)")
        };
        append_rule(&mut out, serialize_rule(&nested_selector, &[margin]));
    }

    // `text` はネイティブ `<div>` でありフォーカス不能なため
    // `:focus-visible` の対象から除外する（書いても dead CSS になる。
    // `[data-scope="button-group"][data-part="text"]` を含めない）。
    for child in [
        r#"[data-scope="button"][data-part="root"]"#,
        r#"[data-scope="field"][data-part="input"]"#,
        r#"[data-scope="menu"][data-part="root"]"#,
        r#"[data-scope="select"][data-part="root"]"#,
    ] {
        // 実フォーカスを受ける要素（menu/select は trigger）へ z-index を
        // 付与する。中間コンテナ（root）はネイティブにフォーカス可能では
        // ないため `:focus-visible` が成立せず、書いても dead CSS になる
        // （角丸連結と同じ `connectable_target` を再利用して揃える。
        // `:focus-visible` は実フォーカス対象へ直接判定させるため
        // `{direct}{suffix}:focus-visible` の形にする）。
        let (direct, suffix) = connectable_target(child);
        let focus_selector = format!("{ROOT} > {direct}{suffix}:focus-visible");
        append_rule(
            &mut out,
            serialize_rule(
                &focus_selector,
                &[decl("position", "relative"), decl("z-index", "1")],
            ),
        );
    }

    // 入れ子 + `data-attached` opt-in（イシュー #3135）: 呼び出し側が内側
    // root へ値なし属性 `data-attached` を付与したときのみ、上記「ネスト」
    // 節の margin 間隔を打ち消し、内側 root の接続辺の角丸・境界線を外側
    // root と共有する連結表示にする（既定の margin 間隔は不変、opt-in
    // のみ）。golden の既存部分を変えないよう本モジュール末尾への純追加
    // とする。
    //
    // 接続辺の判定は上記 connectable child と**同一の規則**（モジュール
    // doc「接続辺の判定規則」節）: 外側リストでの位置（先頭/末尾）に加え、
    // 隣接する兄弟が [`DETACHED_ROOT`]（間隔を保つ通常の内側 root）で
    // **ない**ことを条件にする。入力欄・ボタン等の通常の子や別の attached
    // root とは接続し（Cursor Bugbot 指摘、PR #3569）、間隔を保つ通常の
    // 内側 root に隣接する側だけは位置上先頭/末尾でなくても角丸・境界線・
    // margin を保つ（codex 指摘、PR #3569）。
    for orientation in ["horizontal", "vertical"] {
        let orientation_root = format!(r#"{ROOT}[data-orientation="{orientation}"]"#);

        // attached root の先頭辺/末尾辺が実際に接続される箇所だけを選ぶ
        // セレクタ片。connectable child の `not_first`/`not_last` と同じ
        // 形（先頭側は forward 結合子、末尾側は `:has(+ ...)`）。
        let prev_connected =
            format!("{ATTACHED_ROOT}:not(:first-child):not({DETACHED_ROOT} + {ATTACHED_ROOT})");
        let next_connected =
            format!("{ATTACHED_ROOT}:not(:last-child):not(:has(+ {DETACHED_ROOT}))");

        // 1. 間隔の打ち消し。通常ネストの margin 規則（特異度 (0,6,0)）を
        //    `[data-attached]` と `:not(D + A)`（引数中の最大特異度 (0,5,0)）
        //    の上乗せで確実に上書きする。先頭辺が接続されないとき（直前が
        //    間隔を保つ通常の内側 root）は適用せず、ネスト間隔を残す。
        let margin_reset_selector = format!("{orientation_root} > {prev_connected}");
        let margin_reset = if orientation == "horizontal" {
            decl("margin-inline-start", "0")
        } else {
            decl("margin-block-start", "0")
        };
        append_rule(
            &mut out,
            serialize_rule(&margin_reset_selector, &[margin_reset]),
        );

        // 2. 縦外側での幅揃え（vertical のみ）。base の
        //    `width: fit-content` を外し外側の幅いっぱいに伸ばす。内側の
        //    横並び入力欄は既存の「横並び時のみ」flex 規則
        //    （`flex: 1 1 auto; min-width: 0`）で幅を分け合う。
        if orientation == "vertical" {
            let stretch_selector = format!("{orientation_root} > {ATTACHED_ROOT}");
            append_rule(
                &mut out,
                serialize_rule(
                    &stretch_selector,
                    &[decl("align-self", "stretch"), decl("width", "auto")],
                ),
            );
        }

        // 3. 接続辺の角丸・境界線の解除。宣言の組は上記 §「角丸連結・
        //    境界線二重描画解消」ループと同一のものを、`AR`（内側の
        //    attached root）の位置に対して適用する。子孫へは `>` で降り
        //    る（規約は `connectable_target` doc「位置擬似クラスは直接
        //    の子へ付与する」節と同じだが、ここでの判定条件自体は上記
        //    `prev_connected`/`next_connected`）。接続される辺の側の全子
        //    へ無差別に宣言が当たるが、内側連結の既存規則で既に 0 の組み
        //    合わせは再宣言になるだけで無害であり、外側へ露出する角
        //    （内側グループの最初/最後の子が持つ角）だけが実際に効果を
        //    持つ。
        for child in CONNECTABLE_CHILDREN {
            let (direct, suffix) = connectable_target(child);

            let (start_radius, start_width, end_radius) = if orientation == "horizontal" {
                (
                    [
                        decl("border-start-start-radius", "0"),
                        decl("border-end-start-radius", "0"),
                    ],
                    decl("border-inline-start-width", "0"),
                    [
                        decl("border-start-end-radius", "0"),
                        decl("border-end-end-radius", "0"),
                    ],
                )
            } else {
                (
                    [
                        decl("border-start-start-radius", "0"),
                        decl("border-start-end-radius", "0"),
                    ],
                    decl("border-block-start-width", "0"),
                    [
                        decl("border-end-start-radius", "0"),
                        decl("border-end-end-radius", "0"),
                    ],
                )
            };

            let not_first = format!("{orientation_root} > {prev_connected} > {direct}{suffix}");
            append_rule(
                &mut out,
                serialize_rule(&not_first, &[start_radius[0], start_radius[1], start_width]),
            );

            let not_last = format!("{orientation_root} > {next_connected} > {direct}{suffix}");
            append_rule(
                &mut out,
                serialize_rule(&not_last, &[end_radius[0], end_radius[1]]),
            );
        }
    }

    out
}

/// styled `root` パーツを組み立てる。見た目クラスは付与せず（モジュール doc
/// 「variant 軸: 持たない」節参照）、呼び出し側 `class` を
/// `drop_class_attr` で除去してから
/// [`fandhe_frontend_headless_ui::button_group::root`] へそのまま委譲する。
#[must_use]
pub fn root<'a>(
    orientation: Orientation,
    label: &'a str,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::button_group::root(
        orientation,
        label,
        drop_class_attr(attrs),
        children,
    )
}

/// styled `separator` パーツを組み立てる。[`root`] と同じく見た目クラスを
/// 付与しない。
#[must_use]
pub fn separator<'a>(
    group_orientation: Orientation,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    fandhe_frontend_headless_ui::button_group::separator(
        group_orientation,
        drop_class_attr(attrs),
        children,
    )
}

/// styled `text` パーツを組み立てる。[`root`] と同じく見た目クラスを
/// 付与しない。
#[must_use]
pub fn text<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    fandhe_frontend_headless_ui::button_group::text(drop_class_attr(attrs), children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text as core_text};

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="button-group"][data-part="root"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let out = stylesheet();
        assert!(!out.contains("</style"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn root_connects_to_headless_button_group_scope() {
        let html = render(&root(Orientation::Horizontal, "Actions", vec![], vec![]));
        assert!(html.contains(r#"data-scope="button-group" data-part="root""#));
        assert!(html.contains(r#"role="group""#));
    }

    #[test]
    fn separator_and_text_connect_to_headless_button_group_scope() {
        let separator_html = render(&separator(Orientation::Horizontal, vec![], vec![]));
        assert!(separator_html.contains(r#"data-scope="button-group" data-part="separator""#));

        let text_html = render(&text(vec![], vec![core_text("Sort by:")]));
        assert!(text_html.contains(r#"data-scope="button-group" data-part="text""#));
    }

    #[test]
    fn caller_class_is_dropped_on_every_part() {
        let html = render(&root(
            Orientation::Horizontal,
            "",
            vec![("class", "evil")],
            vec![
                separator(Orientation::Horizontal, vec![("class", "evil")], vec![]),
                text(vec![("class", "evil")], vec![core_text("x")]),
            ],
        ));
        assert!(!html.contains("evil"));
        assert_eq!(html.matches("class=\"").count(), 0);
    }

    #[test]
    fn stylesheet_contains_horizontal_and_vertical_orientation_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-scope="button-group"][data-part="root"][data-orientation="horizontal"]"#
        ));
        assert!(out.contains(
            r#"[data-scope="button-group"][data-part="root"][data-orientation="vertical"]"#
        ));
        assert!(out.contains("flex-direction: column;"));
    }

    #[test]
    fn stylesheet_contains_connectable_child_radius_and_border_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-orientation="horizontal"] > [data-scope="button"][data-part="root"]:not(:first-child)"#
        ));
        assert!(out.contains("border-inline-start-width: 0;"));
        assert!(out.contains(
            r#"[data-orientation="vertical"] > [data-scope="field"][data-part="input"]:not(:first-child)"#
        ));
        assert!(out.contains("border-block-start-width: 0;"));
        // 位置擬似クラスは直接の子（`root`）へ付与し、そこから子孫
        // （`trigger`）へ降りる（`connectable_target` doc「位置擬似
        // クラスは直接の子へ付与する」節参照）。`:not(:first-child)` の
        // 直後には、非 attached 内側 root が直前の兄弟のときに除外する
        // `:not({detached} + {direct})` 節（下記「入れ子（非 attached）の
        // margin 間隔に対する角丸復元」テスト参照）が入る。
        assert!(out.contains(
            r#"[data-scope="menu"][data-part="root"]:not(:first-child):not([data-scope="button-group"][data-part="root"]:not([data-attached]) + [data-scope="menu"][data-part="root"]) > [data-scope="menu"][data-part="trigger"]"#
        ));
        assert!(out.contains(
            r#"[data-scope="select"][data-part="root"]:not(:first-child):not([data-scope="button-group"][data-part="root"]:not([data-attached]) + [data-scope="select"][data-part="root"]) > [data-scope="select"][data-part="control"] > [data-scope="select"][data-part="trigger"]"#
        ));
        // 回帰防止: 擬似クラスを子孫（trigger）の末尾へ付けた形（trigger は
        // 常にその親の唯一の子のため never-match の dead CSS になる）は
        // 出力されないことを固定する。
        assert!(!out.contains(r#"[data-scope="menu"][data-part="trigger"]:not(:first-child)"#));
        assert!(!out.contains(r#"[data-scope="select"][data-part="trigger"]:not(:first-child)"#));
    }

    /// 入れ子（非 attached）の margin 間隔に対する角丸復元（PR #3569 codex
    /// レビュー指摘対応、イシュー #3135）: `data-attached` を付与しない
    /// （margin 間隔のままの）内側 button-group root と隣接する connectable
    /// child（button/input/text/menu/select）は、角丸・境界線幅の除去
    /// 対象から除外される。間隔を空けて表示される要素が意図せず「連結
    /// 表示」に見える不具合（レビュー指摘）を防ぐ。
    #[test]
    fn stylesheet_preserves_radius_for_child_adjacent_to_detached_nested_group() {
        let out = stylesheet();
        let detached = r#"[data-scope="button-group"][data-part="root"]:not([data-attached])"#;

        // 末尾側: 直後が非 attached の内側 root のときは除去しない
        // （:has() による後方兄弟の存在判定、forward combinator では
        // 表現できないため）。
        assert!(out.contains(&format!(
            r#"[data-scope="field"][data-part="input"]:not(:last-child):not(:has(+ {detached}))"#
        )));
        assert!(out.contains(&format!(
            r#"[data-scope="button"][data-part="root"]:not(:last-child):not(:has(+ {detached}))"#
        )));

        // 先頭側: 直前が非 attached の内側 root のときは除去しない
        // （forward combinator `{detached} + {direct}` で直接表現できる）。
        assert!(out.contains(&format!(
            r#":not(:first-child):not({detached} + [data-scope="field"][data-part="input"])"#
        )));
        assert!(out.contains(&format!(
            r#":not(:first-child):not({detached} + [data-scope="button"][data-part="root"])"#
        )));

        // data-attached 付きの内側 root は除外対象に含まれない（従来どおり
        // 角丸を除去する、既存の「連結表示」契約は不変）。
        assert!(!out.contains(
            r#":not(:has(+ [data-scope="button-group"][data-part="root"][data-attached]))"#
        ));
    }

    #[test]
    fn stylesheet_contains_focus_visible_z_index_rules() {
        let out = stylesheet();
        assert!(out.contains(r#"[data-scope="button"][data-part="root"]:focus-visible"#));
        assert!(out.contains("z-index: 1;"));
        // `text` はフォーカス不能な div のため :focus-visible の対象に
        // 含めない（dead CSS を書かない）。
        assert!(!out.contains(r#"[data-scope="button-group"][data-part="text"]:focus-visible"#));
    }

    #[test]
    fn stylesheet_contains_nested_root_margin_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-orientation="horizontal"] > [data-scope="button-group"][data-part="root"]:not(:first-child)"#
        ));
        assert!(out.contains("margin-inline-start: var(--fandhe-space-2);"));
        assert!(out.contains(
            r#"[data-orientation="vertical"] > [data-scope="button-group"][data-part="root"]:not(:first-child)"#
        ));
        assert!(out.contains("margin-block-start: var(--fandhe-space-2);"));
    }

    /// 接続辺の判定規則（モジュール doc「接続辺の判定規則」節の表）を
    /// 行ごとに固定する回帰テスト（PR #3569 codex / Cursor Bugbot 指摘対応、
    /// イシュー #3135）。N（通常の子）・A（attached 内側 root）・D（間隔を
    /// 保つ通常の内側 root）の 3 種について、「どちらも D でないときだけ
    /// 接続する」ことをセレクタ生成の有無で検証する。
    #[test]
    fn stylesheet_attached_edges_connect_unless_neighbor_is_detached() {
        let out = stylesheet();
        let a = ATTACHED_ROOT;
        let d = DETACHED_ROOT;
        let n = r#"[data-scope="field"][data-part="input"]"#;

        // A の先頭辺・末尾辺は N と同一の規則（隣が D でない限り接続）。
        let a_prev = format!("{a}:not(:first-child):not({d} + {a})");
        let a_next = format!("{a}:not(:last-child):not(:has(+ {d}))");
        let n_prev = format!("{n}:not(:first-child):not({d} + {n})");
        let n_next = format!("{n}:not(:last-child):not(:has(+ {d}))");
        for orientation in ["horizontal", "vertical"] {
            let root = format!(r#"{ROOT}[data-orientation="{orientation}"]"#);
            // 行 N→A / A→A: A の先頭辺が接続（margin 打ち消し + 角丸解除）。
            assert!(out.contains(&format!("{root} > {a_prev} {{")));
            assert!(out.contains(&format!("{root} > {a_prev} > {n}")));
            // 行 A→N / A→A: A の末尾辺が接続（角丸解除）。
            assert!(out.contains(&format!("{root} > {a_next} > {n}")));
            // 行 A→N / N→A: N 側の辺も接続（D 以外は除外されない）。
            assert!(out.contains(&format!("{root} > {n_prev} {{")));
            assert!(out.contains(&format!("{root} > {n_next} {{")));
        }

        // 行 A→D / D→A / N→D / D→N: D に面した辺は `:not()` 除外で角丸・
        // margin を保つ（D 自身は通常ネストの margin 規則のみで、角丸連結
        // 対象へ含めない）。
        assert!(!out.contains(&format!("{d}:not(:first-child):not(")));
        assert!(!out.contains(&format!("{d}:not(:last-child)")));

        // 退行防止: 「隣が A か」を条件にする形（N–A が切れる、Cursor
        // Bugbot 指摘）と、位置だけを条件にする形（A–D が接続される、
        // codex 指摘）のどちらも生成しない。
        assert!(!out.contains(&format!("{a} + {a}")));
        assert!(!out.contains(&format!(":has(+ {a})")));
        assert!(!out.contains(&format!("{a}:not(:first-child) > ")));
        assert!(!out.contains(&format!("{a}:not(:last-child) > ")));
    }

    #[test]
    fn stylesheet_contains_attached_margin_reset_rules() {
        let out = stylesheet();
        assert!(out.contains(&format!(
            r#"[data-orientation="horizontal"] > {ATTACHED_ROOT}:not(:first-child):not({DETACHED_ROOT} + {ATTACHED_ROOT}) {{"#
        )));
        assert!(out.contains("margin-inline-start: 0;"));
        assert!(out.contains(&format!(
            r#"[data-orientation="vertical"] > {ATTACHED_ROOT}:not(:first-child):not({DETACHED_ROOT} + {ATTACHED_ROOT}) {{"#
        )));
        assert!(out.contains("margin-block-start: 0;"));
    }

    #[test]
    fn stylesheet_contains_attached_vertical_stretch_rule() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-orientation="vertical"] > [data-scope="button-group"][data-part="root"][data-attached] {"#
        ));
        assert!(out.contains("align-self: stretch;"));
    }

    #[test]
    fn stylesheet_contains_attached_connectable_radius_and_border_rules() {
        let out = stylesheet();
        // menu/select は子孫（trigger）へ `>` で降り、接続辺の判定は
        // attached root 側に付く（never-match 回避の既存規約と同型）。
        assert!(out.contains(&format!(
            r#"{ATTACHED_ROOT}:not(:first-child):not({DETACHED_ROOT} + {ATTACHED_ROOT}) > [data-scope="menu"][data-part="root"] > [data-scope="menu"][data-part="trigger"] {{"#
        )));
        assert!(out.contains(&format!(
            r#"{ATTACHED_ROOT}:not(:last-child):not(:has(+ {DETACHED_ROOT})) > [data-scope="select"][data-part="root"] > [data-scope="select"][data-part="control"] > [data-scope="select"][data-part="trigger"] {{"#
        )));
        assert!(out.contains("border-inline-start-width: 0;"));
        assert!(out.contains("border-block-start-width: 0;"));
    }

    #[test]
    fn stylesheet_contains_separator_orientation_rules() {
        let out = stylesheet();
        assert!(out.contains(
            r#"[data-scope="button-group"][data-part="separator"][data-orientation="vertical"]"#
        ));
        assert!(out.contains("width: 1px;"));
        assert!(out.contains(
            r#"[data-scope="button-group"][data-part="separator"][data-orientation="horizontal"]"#
        ));
        assert!(out.contains("height: 1px;"));
    }
}
