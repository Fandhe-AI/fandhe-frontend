//! styled DatePicker（headless ラッパー、イシュー #835、親 #520/#546）。
//!
//! `fandhe_frontend_headless_ui::date_picker` の Root / Label / Control /
//! Input / Trigger / ClearTrigger / Positioner / Content 8 anatomy パーツを
//! 再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。薄い委譲の
//! 根拠は [`crate::calendar`]（本クレート内の同型先行例）の rustdoc と同じ
//! 方針に従う。`content` 内部に [`crate::calendar`] の styled パーツを合成
//! する想定である。
//!
//! # 選択的 re-export
//!
//! `size` variant クラス付与のため styled [`root`] を本モジュールで新設する。
//! 状態機械 [`fandhe_frontend_headless_ui::date_picker::DatePicker`] は
//! **あえて**再エクスポートしない（[`crate::calendar`]/[`crate::select`] と
//! 同じ理由）。
//!
//! # スタイル調整（イシュー #1471、control/input/trigger/clear-trigger
//! パートのみ）
//!
//! 親 #1470（chakra-ui / ark-ui 基準への調整、Phase 2 / ルート #1420）のうち
//! `control`/`input`/`trigger`/`clear-trigger` の 4 パートを担当する分割
//! 1/3。分割 2/3（カレンダーグリッド、#1472）・3/3（ビュー切り替え・
//! ポジショナ、#1473）とはファイルを共有するため、以下は本イシューが
//! 確定した意図的差分である（combobox 1/2、PR #1744・イシュー #1467 と
//! 同型の記録方針）:
//!
//! - **radius トークン化**: `input`/`trigger` の `border-radius`（生
//!   `0.375rem`）を `var(--fandhe-radius-md)` へ置換した。`root`/`label`/
//!   `positioner`/`content` は 2/3・3/3 のスコープのため変更しない
//! - **canonical フォーカスリング**: `input`/`trigger` の
//!   `:focus-visible` を [`crate::recipe::focus_ring_declarations`]
//!   （`FocusRingColor::Token`、date-picker は palette 軸を持たないため）
//!   へ置換した。`trigger` はハードコードの `outline: 2px solid
//!   var(--fandhe-color-accent)` を、`input` は `border-color` のみの
//!   弱い表現をそれぞれ置き換えている
//! - **hover は `trigger`/`clear-trigger` にのみ付ける**: `control` は
//!   レイアウトのみのコンテナ（headless が `data-state` のみを出し、
//!   クリック操作を担わない）で hover 対象としない。`input` はテキスト
//!   入力面であり参照サイト（chakra-ui/ark-ui）もこの面自体への hover
//!   表現を持たないため対象外とする（combobox 1/2 の `control`/`input` と
//!   同じ判断軸）。`trigger`/`clear-trigger`（クリック操作を担う slot）
//!   にのみ [`crate::recipe::hover_bg_muted`]（`--fandhe-hover-bg` 定義）+
//!   `.state(slot, StateCondition::Hover,
//!   crate::recipe::hover_surface_declarations())`（実適用）を付ける
//! - **disabled 視覚は `input`/`trigger` のみに付ける**: 本イシュー
//!   （#1471）着手当時、headless（`crates/headless-ui/src/date_picker.rs`）が
//!   `data-disabled` を出すのは `input`/`trigger` のみで、`control`/
//!   `clear-trigger` へは出さないため本 CSS 側でも対象外としていた
//!   （消費できない属性へ規則を書かない、combobox 1/2 と同じ判断）。
//!   headless-ui 0.41.0（イシュー #1627）以降は root/label/control/
//!   clear-trigger にも `data-disabled`、全 6 パーツに `data-invalid`/
//!   `data-readonly`、label に `data-required` が出るようになったが、
//!   これらの CSS 消費（`control`/`clear-trigger` への disabled 視覚追加を
//!   含む）は Themes 側の後続判断に委ね、本 PR では recipe を変更しない
//!   （イシュー #1470 へコメントで追跡）。
//! - **`control` は変更しない**: border を持たない純レイアウトコンテナで
//!   あり、7 軸チェックリスト上で是正対象がない（上記のとおり `data-*`
//!   消費の追加判断自体は #1470 側へ先送り）
//! - `clear-trigger`（`<button>`）にブラウザ既定のボタン装飾（border・
//!   背景）が露出していた実不具合を是正し、`trigger` と同じリセット
//!   （`display: inline-flex`/`align-items: center`/`justify-content:
//!   center`/`background: transparent`/`border: none`）+
//!   `border-radius: var(--fandhe-radius-sm)` + `hover_bg_muted()` を
//!   追加した（combobox 1/2 の `clear-trigger` 是正と同型）。headless が
//!   `clear-trigger` へ `data-disabled` を出さないため disabled 視覚は
//!   付けない
//! - **transition は純追加**: `input`/`trigger`/`clear-trigger` へ
//!   [`crate::recipe::transition_declarations`] を `base` の 2 個目登録
//!   （同一 slot への複数回 `.base()` 登録は出力順で連結される）として
//!   追加し、既存 `base` ブロックは書き換えない
//! - **variant 軸（chakra の `outline`/`subtle` 相当）の追加は見送る**:
//!   `root()` シグネチャ変更（破壊的）を伴い Forms 家族横断の軸判断で
//!   あるため、部品単独で先行しない（checkbox 1/2・combobox 1/2 と同じ
//!   判断軸）
//! - **size 連動の `font-size` 追加は見送る**: 同上の横断判断のため
//!   見送る
//!
//! # スタイル調整（イシュー #1472、カレンダーグリッド）
//!
//! 分割 2/3。イシュー本文の担当範囲名（`table`/`table-cell`/
//! `table-cell-trigger`）は ark-ui の date-picker anatomy 名であり、本実装の
//! `date_picker` 8 パーツには存在しない。本実装ではカレンダーグリッドを
//! [`crate::calendar`] の styled パーツ（`table`/`table-header`/`table-row`/
//! `table-head-cell`/`table-body`/`table-cell`/`day-trigger`）を `content`
//! 内へ合成する設計であり、ark-ui `table`→calendar `table`、
//! `table-cell`→calendar `table-cell`、`table-cell-trigger`→calendar
//! `day-trigger` に対応する。
//!
//! - **グリッドの状態表現（selected/today/outside-month/disabled/hover/
//!   focus-visible/transition）はカレンダー担当イシュー（#1451 系）で是正
//!   済み**: `day-trigger` の `data-selected`/`data-today`/
//!   `data-outside-month`/`data-disabled`・hover（`@media (hover: hover)`
//!   集約）・[`crate::recipe::focus_ring_declarations`]・transition が
//!   `crate::calendar::recipe` に揃っており、`[data-scope="calendar"]`
//!   セレクタで当たるため date-picker への合成先でも自動的に適用される。
//!   `calendar.rs` 本体は他イシュー担当済みのため本イシューでは変更しない
//!   （二重管理の回避）
//! - **是正対象は date-picker 側の size 連動の欠落のみ**: `calendar::root`
//!   の size variant は `--fandhe-calendar-day-size`（xs〜xl で
//!   `--fandhe-space-4`〜`--fandhe-space-12`）を定義するが、date-picker の
//!   合成では `calendar::root` を使わず `content` 直下へ `calendar::table`
//!   を置くためこの custom property が未定義になり、日セルが常に既定
//!   `var(--fandhe-space-8)`（md 相当）へ固定されていた。本イシューで
//!   `date-picker` の `root` size variant（5 段）へ同スケールの
//!   `--fandhe-calendar-day-size` を追加定義し、CSS custom property の
//!   継承で入れ子の calendar グリッドへ届くようにした
//!   （`--fandhe-date-picker-input-padding` と同型のパターン）
//! - **`content` の padding 等は変更しない**: 3/3（#1473）のスコープのため
//!   触らない
//! - **バリアント軸（chakra の `outline`/`subtle` 相当）の追加は見送る**:
//!   1/3 と同じく `root()` シグネチャ変更を伴う Forms 家族横断の軸判断の
//!   ため部品単独で先行しない
//!
//! # スタイル調整（イシュー #1473、ビュー切り替えとポジショナ）
//!
//! 分割 3/3（最終回）。イシュー本文の担当範囲名 `view-control`/
//! `view-trigger`/月・年ビューは ark-ui の date-picker anatomy 名であり、
//! 本実装の `date_picker`（8 パーツ: root/label/control/input/trigger/
//! clear-trigger/positioner/content）には存在しない。月表示の切り替え UI
//! は本実装では `content` 内へ合成する [`crate::calendar`] の
//! `heading`/`prev-trigger`/`next-trigger` が担い、これらは calendar 担当
//! イシュー（#1451 系）で是正済み（`[data-scope="calendar"]` セレクタで
//! 合成先にも自動適用される）。月・年ビューへの切り替え状態機械そのものは
//! headless 側に存在せず、headless anatomy 突合はオープンイシュー #1627 が
//! 追跡する。したがって本イシューで実際に是正したパートは `positioner`/
//! `content` の 2 つである。
//!
//! - **`content` の色トークン化（角丸・影）**: `border-radius`（生
//!   `0.375rem`）を `var(--fandhe-radius-md)` へ、`box-shadow`（生
//!   `0 4px 6px rgba(0, 0, 0, 0.15)`）を `var(--fandhe-shadow-md)` へ
//!   置換した。`docs/design/pre-styled-ui-scale-tokens.md` §3.2 が
//!   dropdown 型（date-picker 含む）へ shadow `md` を割り当て済み
//!   （light 値の alpha が `0.15`→`0.1` へ寄る想定どおりの変化。
//!   [`crate::combobox`] の `content` が先例）
//! - **`positioner` の z-index トークン化**: `z-index: 10` を
//!   `var(--fandhe-z-index-dropdown, 10)` へ置換した
//!   （`docs/design/pre-styled-ui-scale-tokens.md` §3.4 の割り当て表で
//!   `dropdown` = date-picker positioner と明記済み）。フォールバック値
//!   `10` を残すのは [`crate::toast`] の
//!   `var(--fandhe-z-index-toast, 9999)` と同じ理由 — `date_picker::
//!   stylesheet()` 単独利用でテーマ CSS が注入されない構成での挙動を
//!   変えないため
//! - **view-control/view-trigger/月・年ビューへの anatomy 追加は見送る**:
//!   上記マッピングのとおり現 anatomy に存在せず、headless 側の追跡は
//!   #1627。pre-styled-ui 側で anatomy を先行追加しない
//!   （`docs/policy/intentional-non-adoption.md` §3.25 の責務境界にも
//!   整合する）
//! - **`positioner` への `data-positioned`（fixed 切り替え）規則の追加は
//!   見送る**: `fandhe-frontend-wasm-full` の位置決めロジック
//!   （`position.rs::from_scope`）は `date-picker` scope を受け付けて
//!   おらず、wasm 層が date-picker を位置決めしない。positioning 契約の
//!   拡張は 7 軸チェックリスト外（combobox 2/2 が `content` の
//!   `max-height` を同じ理由で見送った判断と同型）
//! - **`content` の開閉アニメーション追加は見送る**: 開閉の表示制御は
//!   headless が出す `hidden` 存在属性が担い、既存 overlay 系（select/
//!   combobox/popover）のいずれも open/close アニメーションを持たない。
//!   overlay 家族横断の判断であり部品単独で先行しない
//! - **`root`/`label` は 7 軸上の是正対象なし**: `label` は全宣言が
//!   トークン参照済み、`root` は `position: relative` のみで是正の余地
//!   がない
//! - **バリアント軸・size 連動 `font-size` の追加は見送る**: 1/3・2/3 と
//!   同じ理由（`root()` シグネチャ変更を伴う Forms 家族横断判断）で見送り
//!   継続する
//!
//! # スタイル調整（イシュー #2013、shadcn/ui 突合）
//!
//! ルート #2001「shadcn/ui を 4 本目の参照軸にする」ツリー Phase 1（#2008）の
//! 1 部品として、shadcn/ui date-picker
//! (<https://ui.shadcn.com/docs/components/base/date-picker>) と突合した。
//! shadcn/ui は補完参照であり主基準（chakra-ui / Radix Themes）の視覚言語を
//! 置き換えない（#2002 で確定済みの適用原則）。
//!
//! - **是正した項目**: 上記イシュー #1471 の記述のとおり、headless-ui
//!   0.41.0（#1627）以降が全 6 パーツへ出す `data-invalid`/`data-readonly`
//!   の CSS 消費が未実装のまま残っていたギャップを本イシューで埋めた。
//!   `input`/`trigger` の `[data-invalid]` へ `border-color:
//!   var(--fandhe-color-danger)`（`date_input.rs` の
//!   `segment-group[data-invalid]` と同一トークン）を追加した。`state()`
//!   の登録順は invalid → disabled とし、同時に真となる場合でも
//!   `disabled_declarations()` が最終適用されるようにした（`date_input.rs`
//!   の #1469/PR #1746 教訓と同型のカスケード順序配慮）
//! - **`input` の `data-readonly` 視覚化は見送る（PR #2177 Cursor Bugbot
//!   指摘を受けた是正）**: 当初 `date_input.rs::segment`（非ネイティブ
//!   `<span>` セグメント）と同一判断軸で `[data-readonly]` へ
//!   `cursor: default` を追加していたが、date-picker の `input` パートは
//!   ネイティブ `<input type="text" readonly>` であり、選択・キャレット
//!   操作が可能なテキストフィールドに `cursor: default` を適用すると
//!   I-beam カーソルが隠れ操作可能な値を操作不能に見せてしまう。
//!   [`crate::input`](mod@crate::input)・[`crate::number_input`] の「readonly（意図的
//!   非採用）」節と同型の判断軸（ネイティブ `<input>` には
//!   `data-readonly` の視覚宣言を追加せず既定の `cursor: text` のまま
//!   とする）へ揃え、追加した宣言を削除した
//! - **`label` の `data-required` 視覚化は見送る**: 同じ headless-ui
//!   バージョンから `label` に `data-required` が出力されているが、
//!   同クレート内の [`crate::field`] も同種の `data-required` を
//!   「headless の `required_indicator`（`hidden` 属性フリップ）が表示切替を
//!   担い、pre-styled-ui 側は `[hidden]` の CSS のみを持つ」という設計で
//!   扱っており、date-picker の `label` 単体に `*` 等の視覚マーカーを
//!   追加する独立実装は他 Forms 部品の慣行と整合しない。Forms 家族横断の
//!   軸判断が必要なため部品単独では追加しない
//! - **`control`/`clear-trigger` への `data-disabled` 視覚追加は見送る**:
//!   #1471 が「Themes 側の後続判断に委ね #1470 へ追跡」としていた積み残し
//!   であり、chakra/ark 基準からの継続課題であって shadcn/ui との比較で
//!   新たに判明した差分ではないため、scope 混同を避け #1470 側へ委ねる
//!   （#2195 で是正済み、次節参照）
//! - **range 選択・複数月表示・presets 行・自然言語入力・date+time 合成は
//!   実装しない**: shadcn/ui の Examples（Range / Natural Language /
//!   Date & Time / Presets 等）はいずれも (a) headless
//!   [`fandhe_frontend_headless_ui::date_picker::DatePicker`]
//!   の状態機械にない状態（range mode・複数月表示・自由入力パース）の
//!   拡張を要する、または (b) 自然言語入力パースのような
//!   `docs/policy/intentional-non-adoption.md` §3.25 規則 1（アプリケー
//!   ションロジックを内包する部品・機能は実装しない）に該当するため、
//!   本イシュー（Themes 限定）のスコープ外である。presets 行（Today/
//!   Clear 等）は [`crate::calendar`] 担当イシュー #2010 が同型の判断
//!   （専用パート化は不要、既存 `input`/`trigger`/`clear-trigger` と
//!   呼び出し側 `button` の組み合わせで表現可能）を採ったのに倣う
//! - **ダークモード・トークン整合**: 新規トークンを追加していないため
//!   無変更（既存 `--fandhe-color-danger` はダーク再定義済み、
//!   `docs/design/pre-styled-ui-scale-tokens.md` 準拠）
//!
//! # スタイル調整（イシュー #2195、Forms 家族横断の disabled / required 規則）
//!
//! #1471・#2013 が「Themes 側の後続判断に委ねる」として見送っていた
//! `control`/`clear-trigger` への `data-disabled` 視覚追加を、Forms 家族
//! 横断の規則として確定し是正した。詳細な決定根拠・対応表は
//! `docs/design/pre-styled-ui-forms-disabled-required-matrix.md` を正とする
//! （二重管理回避のため本節では結論のみ記す）。
//!
//! - **`control[data-disabled]` は `cursor: not-allowed` のみ**: headless
//!   （`crates/headless-ui/src/date_picker.rs::control`）が
//!   `data-disabled` を出すようになったため消費するが、date-picker は
//!   `input`/`trigger` という「葉」パーツが `disabled_declarations()`
//!   （opacity 0.5 を含む）を既に適用する葉所有型であり、レイアウトのみの
//!   `control` へ重ねて opacity を足すと二重減光になる（`tags_input.rs`
//!   の `control`/`item` と同じ判断軸）
//! - **`clear-trigger[data-disabled]` は `disabled_declarations()`**:
//!   headless（`clear_trigger`）が `data-disabled` を出すようになった。
//!   `clear-trigger` は `trigger` と同じく単独で操作されるクリック可能な
//!   `<button>` （葉）であり、`control` の内側 slot ではなく `input`/
//!   `trigger` と同格の要素であるため `disabled_declarations()` を適用する
//! - **`label` の `data-required` 視覚化は見送る（決定として確定）**:
//!   `field::required_indicator` による表現へ統一する Forms 家族横断規則
//!   （R2）であり、`*` 等の CSS 生成コンテンツは追加しない

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    disabled_declarations, focus_ring_declarations, hover_bg_muted, hover_surface_declarations,
    transition_declarations, FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe,
    StateCondition, VariantValue,
};

pub use fandhe_frontend_headless_ui::date_picker::{
    clear_trigger, content, control, input, label, positioner, trigger, DatePickerProps,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
pub use fandhe_frontend_headless_ui::state::OpenState;

/// headless `date_picker` anatomy の `data-part` 一覧
/// （`crates/headless-ui/src/date_picker.rs` の `ANATOMY.part(...)` 呼び出しと
/// 同期させる契約）。
const SLOTS: &[&str] = &[
    "root",
    "label",
    "control",
    "input",
    "trigger",
    "clear-trigger",
    "positioner",
    "content",
];

/// この styled DatePicker の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("date-picker", SLOTS)
        .base("root", vec![decl("position", "relative")])
        .base(
            "label",
            vec![
                decl("display", "block"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "control",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "input",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl(
                    "padding",
                    "var(--fandhe-date-picker-input-padding, var(--fandhe-space-2) var(--fandhe-space-3))",
                ),
            ],
        )
        // `base` は同一 slot への複数回登録が許され出力順で連結されるため、
        // 上記 base ブロックを書き換えずに純追加する（combobox 1/2、
        // イシュー #1467 の transition 追加と同型のパターン）。
        .base(
            "input",
            transition_declarations("border-color, background", MotionDuration::Fast),
        )
        .base(
            "trigger",
            vec![
                decl("cursor", "pointer"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("padding", "var(--fandhe-space-2)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "trigger",
            transition_declarations("border-color, background, color", MotionDuration::Fast),
        )
        .base(
            "clear-trigger",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("justify-content", "center"),
                decl("background", "transparent"),
                decl("border", "none"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg-muted)"),
                hover_bg_muted(),
            ],
        )
        .base(
            "clear-trigger",
            transition_declarations("background, color", MotionDuration::Fast),
        )
        .base(
            "positioner",
            vec![
                decl("position", "absolute"),
                decl("top", "100%"),
                decl("left", "0"),
                decl("z-index", "var(--fandhe-z-index-dropdown, 10)"),
                decl("margin-top", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "content",
            vec![
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl(
                    "padding",
                    "var(--fandhe-date-picker-content-padding, var(--fandhe-space-2))",
                ),
            ],
        )
        .state(
            "trigger",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("border-color", "var(--fandhe-color-accent)")],
        )
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        .state(
            "input",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // headless-ui 0.41.0（イシュー #1627）以降が `input`/`trigger` へ
        // 出す `data-invalid` を消費する（イシュー #2013、shadcn/ui 突合。
        // `date_input.rs` の `segment-group[data-invalid]` と同じ
        // `--fandhe-color-danger` トークンで枠線色を切り替える）。
        .state(
            "input",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        .state(
            "trigger",
            StateCondition::Attr("data-invalid"),
            vec![decl("border-color", "var(--fandhe-color-danger)")],
        )
        // `input` の `data-readonly` へは視覚宣言を追加しない（PR #2177
        // Cursor Bugbot 指摘を受けた是正、モジュール rustdoc「`input` の
        // `data-readonly` 視覚化は見送る」節参照）。ネイティブ
        // `<input readonly>` は選択・キャレット操作が可能なため既定の
        // `cursor: text` のままが適切であり、[`crate::input`]・
        // [`crate::number_input`] の「readonly（意図的非採用）」節と
        // 同型の判断。
        //
        // 登録順は invalid → disabled の意図的な並びである。`state()` は
        // 同じ詳細度 `[data-part="..."][data-attr]` (0,2,0) の規則同士を
        // 登録順（後勝ち）で解決するため（`crate::recipe::SlotRecipe::css`
        // rustdoc「LastChild」節、`date_input.rs` の #1469/PR #1746 教訓と
        // 同型）、disabled かつ invalid が同時に真の要素で
        // `disabled_declarations()`（`cursor: not-allowed` 等）が確実に
        // 最終適用されるよう最後に置く。headless
        // （`crates/headless-ui/src/date_picker.rs`）が `input`/`trigger`
        // へ出す `data-disabled` を消費する。
        .state(
            "input",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        .state(
            "trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // `control[data-disabled]`（イシュー #2195、モジュール rustdoc
        // 「スタイル調整（イシュー #2195）」節参照）: headless
        // （`control`）が `data-disabled` を出すようになった
        // （headless-ui 0.41.0、#1627）。`control` は純レイアウトの
        // コンテナで `input`/`trigger` という葉パーツが既に
        // `disabled_declarations()`（opacity 0.5 を含む）を適用する
        // ため、`cursor: not-allowed` のみに留め二重減光を避ける
        // （`tags_input.rs` の `control` と同じ判断軸）。
        .state(
            "control",
            StateCondition::Attr("data-disabled"),
            vec![decl("cursor", "not-allowed")],
        )
        // `clear-trigger[data-disabled]`（イシュー #2195）: headless
        // （`clear_trigger`）が `data-disabled` を出すようになった。
        // `clear-trigger` は `trigger` と同格の単独クリック可能な
        // `<button>`（葉）であるため `disabled_declarations()` を適用する。
        .state(
            "clear-trigger",
            StateCondition::Attr("data-disabled"),
            disabled_declarations(),
        )
        // trigger/clear-trigger の hover 実適用（`--fandhe-hover-bg` の間接
        // 参照経由、モジュール rustdoc「スタイル調整」節参照）。`control`/
        // `input` 自体には付けない。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .state(
            "clear-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        .variant(
            Size::Xs,
            "root",
            vec![
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-4)"),
                decl("--fandhe-date-picker-input-padding", "var(--fandhe-space-0-5) var(--fandhe-space-1)"),
                decl("--fandhe-date-picker-content-padding", "var(--fandhe-space-0-5)"),
            ],
        )
        .variant(
            Size::Sm,
            "root",
            vec![
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-6)"),
                decl(
                    "--fandhe-date-picker-input-padding",
                    "var(--fandhe-space-1) var(--fandhe-space-2)",
                ),
                decl(
                    "--fandhe-date-picker-content-padding",
                    "var(--fandhe-space-1)",
                ),
            ],
        )
        .variant(
            Size::Md,
            "root",
            vec![
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-8)"),
                decl(
                    "--fandhe-date-picker-input-padding",
                    "var(--fandhe-space-2) var(--fandhe-space-3)",
                ),
                decl(
                    "--fandhe-date-picker-content-padding",
                    "var(--fandhe-space-2)",
                ),
            ],
        )
        .variant(
            Size::Lg,
            "root",
            vec![
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-10)"),
                decl(
                    "--fandhe-date-picker-input-padding",
                    "var(--fandhe-space-3) var(--fandhe-space-4)",
                ),
                decl(
                    "--fandhe-date-picker-content-padding",
                    "var(--fandhe-space-3)",
                ),
            ],
        )
        .variant(
            Size::Xl,
            "root",
            vec![
                decl("--fandhe-calendar-day-size", "var(--fandhe-space-12)"),
                decl("--fandhe-date-picker-input-padding", "var(--fandhe-space-4) var(--fandhe-space-5)"),
                decl("--fandhe-date-picker-content-padding", "var(--fandhe-space-4)"),
            ],
        )
        .default_variant(Size::Md)
}

/// この styled DatePicker が生成する静的 CSS 全量を返す（決定的。
/// [`crate::calendar::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ。実体は [`fandhe_frontend_headless_ui::date_picker::root`] へ委譲する。
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    props: &DatePickerProps,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::date_picker::root(state, props, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// `css` 中で `selector_with_brace`（例: `"...[data-disabled] {"`）から
    /// 対応する `}` までの本文を抜き出す（イシュー #2195 の
    /// `disabled_declarations_follow_forms_matrix` テスト専用ヘルパ）。
    /// `size_variants_define_calendar_day_size_for_grid_propagation` の
    /// ブロック抽出パターンと同型。
    fn extract_block<'a>(css: &'a str, selector_with_brace: &str) -> &'a str {
        let block_start = css
            .find(selector_with_brace)
            .unwrap_or_else(|| panic!("selector not found: {selector_with_brace}, css={css}"));
        let body_start = block_start + selector_with_brace.len();
        let body_end = css[body_start..]
            .find('}')
            .map(|offset| body_start + offset)
            .unwrap_or_else(|| panic!("unterminated block for {selector_with_brace}"));
        &css[body_start..body_end]
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="date-picker"][data-part="trigger"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let props = DatePickerProps::default();
        let html = render(&root(Size::Md, OpenState::Closed, &props, vec![], vec![]));
        assert!(html.contains(r#"data-scope="date-picker""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        let props = DatePickerProps::default();
        for size in [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl] {
            let html = render(&root(
                size,
                OpenState::Closed,
                &props,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-date-picker--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open() {
        let css = stylesheet();
        assert!(
            css.contains(r#"[data-scope="date-picker"][data-part="trigger"][data-state="open"]"#)
        );
    }

    #[test]
    fn input_and_trigger_use_tokenized_radius_not_raw_literal() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="input"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);"#
        ));
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="trigger"] {
  cursor: pointer;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);"#
        ));
    }

    #[test]
    fn content_uses_tokenized_radius_and_shadow_not_raw_literal() {
        // content の角丸・影は #1473 でトークン参照へ置換した
        // （モジュール rustdoc「スタイル調整（イシュー #1473）」節参照）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="content"] {
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  box-shadow: var(--fandhe-shadow-md);"#
        ));
        assert!(!css.contains("border-radius: 0.375rem;"));
        assert!(!css.contains("rgba(0, 0, 0, 0.15)"));
    }

    #[test]
    fn positioner_uses_tokenized_z_index_not_raw_literal() {
        // positioner の z-index は #1473 でトークン参照へ置換した
        // （フォールバック値 10 は toast の
        // `var(--fandhe-z-index-toast, 9999)` と同じ理由で維持）。
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="positioner"] {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: var(--fandhe-z-index-dropdown, 10);"#
        ));
        assert!(!css.contains("z-index: 10;"));
    }

    #[test]
    fn input_and_trigger_expose_canonical_focus_ring() {
        let css = stylesheet();
        let ring = "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));";
        assert!(css.contains(ring), "css={css}");
        // ハードコードのアウトラインが残っていないこと。
        assert!(!css.contains("outline: 2px solid var(--fandhe-color-accent);"));
    }

    #[test]
    fn disabled_declarations_follow_forms_matrix() {
        // イシュー #2195（Forms 家族横断の disabled 規則、R1「opacity 単一
        // 階層」）: date-picker は `input`/`trigger` という葉パーツが
        // opacity を所有する「葉所有型」であるため、`control`（純レイアウト
        // コンテナ）は `cursor: not-allowed` のみを持ち `opacity` を含まない
        // （二重減光の回避）。`clear-trigger` は `trigger` と同格の葉
        // （単独操作可能な `<button>`）であるため `disabled_declarations()`
        // （opacity 0.5 + cursor）を適用する。
        // 詳細な対応表は
        // `docs/design/pre-styled-ui-forms-disabled-required-matrix.md` 参照。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="date-picker"][data-part="input"][data-disabled]"#));
        assert!(css.contains(r#"[data-scope="date-picker"][data-part="trigger"][data-disabled]"#));
        assert!(css.contains(r#"[data-scope="date-picker"][data-part="control"][data-disabled]"#));
        assert!(
            css.contains(r#"[data-scope="date-picker"][data-part="clear-trigger"][data-disabled]"#)
        );
        assert!(css.contains("opacity: 0.5"));
        assert!(css.contains("cursor: not-allowed"));

        let control_block = extract_block(
            &css,
            r#"[data-scope="date-picker"][data-part="control"][data-disabled] {"#,
        );
        assert!(
            !control_block.contains("opacity"),
            "control[data-disabled] must not own opacity (input/trigger already do): {control_block}"
        );
        assert!(control_block.contains("cursor: not-allowed"));

        let clear_trigger_block = extract_block(
            &css,
            r#"[data-scope="date-picker"][data-part="clear-trigger"][data-disabled] {"#,
        );
        assert!(
            clear_trigger_block.contains("opacity: 0.5"),
            "clear-trigger[data-disabled] must apply disabled_declarations() (leaf slot): {clear_trigger_block}"
        );
        assert!(clear_trigger_block.contains("cursor: not-allowed"));
    }

    #[test]
    fn invalid_declarations_apply_to_input_and_trigger_only() {
        // イシュー #2013（shadcn/ui 突合）: headless-ui 0.41.0（#1627）以降が
        // 全 6 パーツへ出す `data-invalid` のうち、`input`/`trigger` のみを
        // CSS で消費する（`control`/`clear-trigger`/`root`/`label` は
        // 対象外、モジュール rustdoc「スタイル調整（イシュー #2013）」節
        // 参照）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="date-picker"][data-part="input"][data-invalid]"#));
        assert!(css.contains(r#"[data-scope="date-picker"][data-part="trigger"][data-invalid]"#));
        assert!(!css.contains(r#"[data-scope="date-picker"][data-part="control"][data-invalid]"#));
        assert!(
            !css.contains(r#"[data-scope="date-picker"][data-part="clear-trigger"][data-invalid]"#)
        );
        assert!(css.contains("border-color: var(--fandhe-color-danger)"));
    }

    #[test]
    fn input_does_not_style_data_readonly() {
        // PR #2177 Cursor Bugbot 指摘の回帰テスト: date-picker の `input`
        // パートはネイティブ `<input type="text" readonly>` であり、
        // `data-readonly` へ `cursor: default` 等の視覚宣言を追加すると
        // I-beam カーソルが隠れ操作可能な値を操作不能に見せてしまう
        // （[`crate::input`]・[`crate::number_input`] の「readonly
        // （意図的非採用）」節と同型の判断、モジュール rustdoc「`input` の
        // `data-readonly` 視覚化は見送る」節参照）。
        let css = stylesheet();
        assert!(!css.contains(r#"[data-scope="date-picker"][data-part="input"][data-readonly]"#));
        assert!(!css.contains(r#"[data-scope="date-picker"][data-part="trigger"][data-readonly]"#));
    }

    #[test]
    fn input_disabled_cursor_overrides_invalid_by_source_order() {
        // `date_input.rs` の #1469/PR #1746 教訓と同型: disabled かつ
        // invalid が同一 `input` に共存する場合でも、登録順
        // （invalid → disabled）による後勝ちで `disabled_declarations()`
        // の `cursor: not-allowed` が最終適用されることを固定する。
        let css = stylesheet();
        let invalid_idx = css
            .find(r#"[data-scope="date-picker"][data-part="input"][data-invalid] {"#)
            .expect("input invalid rule must exist");
        let disabled_idx = css
            .find(r#"[data-scope="date-picker"][data-part="input"][data-disabled] {"#)
            .expect("input disabled rule must exist");
        assert!(
            disabled_idx > invalid_idx,
            "input[data-disabled] must be registered after input[data-invalid] so it wins by source order"
        );
    }

    #[test]
    fn hover_rules_are_scoped_to_media_hover_query() {
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        let media_start = css
            .find("@media (hover: hover)")
            .expect("media block present");
        let media_block = &css[media_start..];
        assert!(media_block.contains(
            r#"[data-scope="date-picker"][data-part="trigger"]:hover:not([data-disabled])"#
        ));
        assert!(media_block.contains(
            r#"[data-scope="date-picker"][data-part="clear-trigger"]:hover:not([data-disabled])"#
        ));
    }

    #[test]
    fn size_variants_define_calendar_day_size_for_grid_propagation() {
        // date-picker の合成先（`content` 直下の `calendar::table`）は
        // `calendar::root` を経由しないため、`--fandhe-calendar-day-size` は
        // date-picker 側の root size variant が定義しない限り未定義になり、
        // 入れ子の日セルが calendar 側の既定値（md 相当）に固定される
        // （イシュー #1472、モジュール rustdoc「スタイル調整」節参照）。
        let css = stylesheet();
        let expectations: &[(Size, &str)] = &[
            (Size::Xs, "var(--fandhe-space-4)"),
            (Size::Sm, "var(--fandhe-space-6)"),
            (Size::Md, "var(--fandhe-space-8)"),
            (Size::Lg, "var(--fandhe-space-10)"),
            (Size::Xl, "var(--fandhe-space-12)"),
        ];
        for (size, expected_value) in expectations {
            let selector = format!(
                r#"[data-scope="date-picker"][data-part="root"].fd-date-picker--size-{}"#,
                size.value()
            );
            let block_start = css
                .find(&selector)
                .unwrap_or_else(|| panic!("selector not found: {selector}, css={css}"));
            let block_end = css[block_start..]
                .find('}')
                .map(|offset| block_start + offset)
                .unwrap_or_else(|| panic!("unterminated block for {selector}"));
            let block = &css[block_start..block_end];
            let decl = format!("--fandhe-calendar-day-size: {expected_value};");
            assert!(
                block.contains(&decl),
                "size={size:?} block={block} expected_decl={decl}"
            );
        }
    }

    #[test]
    fn clear_trigger_resets_native_button_chrome() {
        let css = stylesheet();
        assert!(css.contains(
            r#"[data-scope="date-picker"][data-part="clear-trigger"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  cursor: pointer;
  color: var(--fandhe-color-fg-muted);"#
        ));
    }
}
