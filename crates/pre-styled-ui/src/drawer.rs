//! styled Drawer（headless ラッパー、イシュー #758、親 #520/#545）。
//!
//! `fandhe_frontend_headless_ui::drawer`（イシュー #758）の Root / Trigger /
//! Backdrop / Positioner / Content / Title / Description / CloseTrigger
//! 8 anatomy パーツを再エクスポートし、[`stylesheet`] で既定 CSS を追加提供する。
//! `crates/pre-styled-ui/src/dialog.rs`（#551/#729）を雛形とする薄い委譲層で
//! あり、設計方針（選択的 re-export・薄い委譲の根拠・`size` variant・
//! キーボード操作系スタイル・overlay の stacking context・closed 時の
//! `positioner` 非表示化）は同ファイルの rustdoc をそのまま継承する。以下は
//! drawer 固有の追加点のみを記す。
//!
//! # 選択的 re-export（`Dialog`/`Drawer` 型・headless `root` を再エクスポート
//! しない理由は [`crate::dialog`] と同一）
//!
//! `size` variant クラス付与のため styled [`root`] を本モジュールで新設する。
//! headless 自由関数 `root` と名前が衝突するため、`pub use ...::*` ではなく
//! 必要な識別子（[`trigger`]/[`backdrop`]/[`positioner`]/[`content`]/[`title`]/
//! [`description`]/[`close_trigger`]）のみを選択的に再エクスポートする。
//! [`fandhe_frontend_headless_ui::dialog::ContentIds`] は drawer の `content`
//! も同じ型を使う（headless 層が [`crate::dialog`] の型をそのまま再利用する
//! 契約、`crates/headless-ui/src/drawer.rs` rustdoc 参照）ため、本モジュールが
//! 独自に再定義せずそのまま再エクスポートする。状態機械
//! [`fandhe_frontend_headless_ui::drawer::Drawer`] はあえて再エクスポートしない
//! （[`crate::dialog`]・[`crate::switch`] と同じ理由）。`Drawer` による状態
//! 管理・hydration が必要な呼び出し側は `fandhe_frontend_headless_ui::drawer`
//! を直接 import すること。
//!
//! # placement による方向別レイアウト（イシュー #758 受け入れ条件）
//!
//! headless 層が `root`/`positioner`/`content` へ出力する `data-placement`
//! （`start`/`end`/`top`/`bottom`、[`fandhe_frontend_headless_ui::drawer::DrawerPlacement`]）
//! を [`StateCondition::AttrEq`] で捕捉し、[`recipe`] の `positioner`（flex 方向・
//! 主軸整列）・`content`（占有する寸法軸）を切り替える。`start`/`end` は
//! CSS の `flex-start`/`flex-end`（row 方向）が `dir` 属性に応じて論理的に
//! 解決される仕様を利用しており、明示的な `margin-inline-*` を追加しなくても
//! RTL 文書で自然に反転する（flexbox の主軸整列は既定で書字方向依存）。
//!
//! # `size` variant（drawer 固有の寸法軸、イシュー #758）
//!
//! `size`（[`Size`]）は [`root`] へのみクラスを付与し、[`recipe`] が登録する
//! `--fandhe-drawer-size`（root スコープの CSS custom property。通常の CSS
//! 継承により `content` へ伝わる）経由で drawer パネルの占有幅（start/end）・
//! 占有高さ（top/bottom）を切り替える。`base` 規則の `var()` には Md 相当の
//! フォールバック値を書き、styled `root` を経由しない headless 直接利用
//! マークアップでも現行外観を維持する（fail-safe、[`crate::dialog`] と同じ
//! 方針）。drawer は dialog と同じく `color-palette` 軸を持たない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - フォーカストラップ・Escape キー閉鎖・外側クリック閉鎖・placement 別の
//!   スライドインアニメーション（`translateX`/`translateY` によるオフセット
//!   遷移）は [`crate::dialog`] のスコープ外方針を継承する。[`SlotRecipe::state`]
//!   は単一条件（`StateCondition`）のみを受け付け、「`data-placement="start"`
//!   かつ `data-state="closed"`」のような複合条件を表現する API
//!   （[`SlotRecipe::compound_variant`] は variant 軸専用で `StateCondition`
//!   を扱えない）が現時点で存在しないため、`content` の開閉スタイルは
//!   dialog と同じ非方向的な `opacity` 切り替えに留める（外観の簡略化。
//!   方向別スライドは別イシューで [`SlotRecipe`] への複合状態条件 API 追加
//!   を検討する）。
//! - `wasm-full` の Drawer 対応（`OverlayKind::from_scope` が `"drawer"` を
//!   未受理）は headless 層のスコープ外と同じ。
//!
//! # 外枠パート（trigger/backdrop/positioner/content）のスタイル調整
//! （イシュー #1694、親 #1521。[`crate::dialog`] の外枠調整（イシュー #1692/
//! PR #1794）と同型の適用）
//!
//! drawer は dialog の薄い委譲変種（モジュール冒頭 rustdoc 参照）であるため、
//! 直近でマージ済みの dialog 外枠調整より前の未是正状態のまま残っていた。
//! 本イシューで dialog と同じ是正を適用する:
//!
//! - **trigger**: `<button type="button">` 実体でありながら枠・背景・角丸・
//!   padding が一切なく UA 既定外観のままだったため、`background`/`border`/
//!   `border-radius`/`padding` を新規追加した（[`crate::dialog`] の trigger
//!   と同型）。hover フィードバックとして
//!   [`crate::recipe::hover_bg_muted`] + [`crate::recipe::transition_declarations`]
//!   （`background, border-color` / [`crate::recipe::MotionDuration::Fast`]）
//!   を base へ追加し、`StateCondition::Hover` に
//!   [`crate::recipe::hover_surface_declarations`] を新規登録した（dialog/
//!   file-upload の trigger と同型）。既存の `StateCondition::FocusVisible`
//!   の直書き `outline`/`outline-offset` は
//!   [`crate::recipe::focus_ring_declarations`]（[`crate::recipe::FocusRingColor::Token`]/
//!   [`crate::recipe::FocusRingOffset::Outside`]）の canonical 形へ置換した
//!   （出力値は従来と同一のトークン参照 + フォールバック形への置換のみで
//!   見た目は不変）。**`close-trigger` の `FocusVisible` は兄弟イシュー
//!   #1695（内部パート・状態遷移）の担当のため触れない**。
//! - **backdrop**: `z-index: 1000`（生値）を
//!   `var(--fandhe-z-index-overlay, 1000)`（イシュー #1423 系トークン）へ、
//!   `background: rgba(0, 0, 0, 0.4)`（生値）を
//!   `var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.4))`（イシュー #1422、
//!   light 0.4 / dark 0.6）へ置換した。いずれも旧生値をフォールバックへ残す
//!   （`drawer::stylesheet()` 単独利用・テーマ CSS 非注入時に暗幕が透明化
//!   する・重なり順を失う事故を避けるため、dialog と同型の安全側判断）。
//! - **positioner**: `z-index: 1001`（生値）を
//!   `var(--fandhe-z-index-modal, 1001)` へ置換した。placement 4 方向の
//!   flex レイアウト規則（`data-placement` に応じた `flex-direction`/
//!   `justify-content`）は点検の上、参照サイト基準と齟齬がないため現状
//!   維持とした。
//! - **content**: 面パネルの影（`docs/design/pre-styled-ui-scale-tokens.md`
//!   §3.2 が dialog/drawer content = lg と割り当て済み）が欠落していたため
//!   `box-shadow: var(--fandhe-shadow-lg)` を新規追加した。
//!
//! **意図的に変更しない点**: `content` への `border-radius` 追加はしない
//!   （drawer パネルは画面端に接する全高/全幅パネルであり、参照サイト
//!   （chakra-ui / ark-ui の Drawer）も角丸を持たない。dialog の面パネルとは
//!   異なり境界の一部が画面端と一致するため角丸は視覚的に不自然になる）。
//!   `positioner` への `overflow: auto` 等の挙動変更はしない（視覚調整を
//!   超えるため、dialog #1692 と同判断）。`root()` シグネチャを変える
//!   variant 軸の追加はしない。title / description / close-trigger の
//!   スタイル・`data-state` 開閉トランジション・`prefers-reduced-motion`
//!   対応は兄弟イシュー #1695 の担当であり本イシューでは触れない。
//!
//! # 内部パートのスタイル調整と状態遷移（イシュー #1695、親 #1521。
//! [`crate::dialog`] の内部パート調整（イシュー #1693/PR #1795）と同型の
//! 適用）
//!
//! headless drawer の anatomy は dialog と同一の 8 パーツ（root/trigger/
//! backdrop/positioner/content/title/description/close-trigger）のみで、
//! header/footer/body パートは存在しない。イシュータイトルの「header 等」は
//! 参照サイト（chakra-ui Drawer の Header/Body/Footer）由来の呼称であり、
//! dialog 先例と同じく「anatomy 変更なしで表現できる範囲」に落とし込む。
//!
//! - **`content` の `position: relative`**: 絶対配置する `close-trigger`
//!   （後述）の配置基準。既存の `overflow-y: auto`（body スクロール担当）は
//!   維持する。**注記**: content 全体がスクロールするため、body スクロール
//!   時に close-trigger も一緒にスクロールする。header/body を分離した
//!   固定ヘッダー化は anatomy に header/body パートがないため実現不可
//!   （下記「本イシューのスコープ外」参照）。
//! - **`close-trigger` を content 右上のゴーストボタン化 + アイコン専用
//!   契約**: dialog 0.59.0（イシュー #1693/PR #1795）と同型の破壊的変更。
//!   `position: absolute` で右上に固定し、hover 時のみ背景が付く
//!   （[`hover_bg_muted`] + [`hover_surface_declarations`]）ghost ボタンの
//!   見た目にする。children は 1〜2 文字のグリフ相当に限定し、支援技術
//!   向けラベルは `("aria-label", "...")` 属性で付与する契約へ変更する
//!   （API シグネチャ自体は変更しないため、静的検知不能な破壊的変更である
//!   点に注意。呼び出し側は `close_trigger(vec![("aria-label", "Close")],
//!   vec![text("×")])` の形へ移行すること）。focus-visible リングは
//!   [`focus_ring_declarations`]（イシュー #1424 共通トークン）へ移行する。
//! - **`title`/`description` の行送り**: [`crate::recipe`] のタイポグラフィ
//!   トークン（`--fandhe-font-line-height-tight`/`-normal`）を追加し、
//!   `description` の下余白を広げて後続のアクション行（footer 相当）との
//!   縦リズムを確保する。
//!
//! ## 開閉トランジションを追加しない理由（dialog #1795 の codex-review P1
//! 確定判断の継承）
//!
//! headless 層（`crates/headless-ui/src/drawer.rs`）も dialog と同じく
//! open/closed の切り替え時に `positioner`/`backdrop`/`content` へ `hidden`
//! 存在属性を**同一フレームで即時**付与・除去する契約になっている。この
//! ため `opacity`/`transform` の遷移前フレームが一切描画されず、開く方向・
//! 閉じる方向のいずれも視覚上トランジションは発火しない。効果のない
//! `transition-property` を「開閉トランジション」として謳うのは契約不整合
//! になるため、`backdrop`/`content` への [`transition_declarations`]
//! 追加は行わず、既存の `data-state` 連動 `opacity` 切り替え（イシュー #758
//! 由来）を維持する。真に機能させる手段（headless 側のタイミング制御 /
//! `@starting-style` + `transition-behavior: allow-discrete` の
//! [`crate::recipe::SlotRecipe`] サポート）は [`crate::dialog`] と同じく
//! 別イシュー・ユーザー承認が必要な対象外事項として記録する
//! （`.claude/rules/out-of-scope-tracking.md` 対応）。イシュー #1425 の
//! `prefers-reduced-motion` 対応は、実際に機能する transition
//! （close-trigger の hover 背景遷移）が motion duration トークン経由
//! （`Theme::to_css` が reduce 時に 0ms へ一括上書き）で
//! 自動充足される。
//!
//! ## 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - **footer 相当のアクション配置**: headless anatomy に `footer` パートが
//!   存在せず、[`crate::recipe::SlotRecipe`] は子孫セレクタ機構を持たない
//!   （イシュー #708 で不採用確定）ため、専用 footer パートの CSS を
//!   pre-styled 側だけで新設することはできない。本イシューでは
//!   `description` の下余白確保までに留め、showcase デモ
//!   （`crates/docs-site/src/showcase.rs::drawer_section`）でアクション行の
//!   掲示例を示す。`drawer` への `footer` anatomy パート追加は headless-ui
//!   の anatomy 変更を伴うため、別イシュー・ユーザー承認が必要な対象外事項
//!   として記録する。
//! - **header/body 分離による固定ヘッダー化**: anatomy に header/body
//!   パートが存在しないため、content スクロール時にヘッダーを固定表示する
//!   ことは本イシューの範囲では実現できない（上記「`content` の
//!   `position: relative`」注記参照）。
//! - 開閉トランジション自体は上記「開閉トランジションを追加しない理由」に
//!   記載のとおり対象外。

use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, hover_bg_muted, hover_surface_declarations, transition_declarations,
    FocusRingColor, FocusRingOffset, MotionDuration, Size, SlotRecipe, StateCondition,
    VariantValue,
};

// headless 自由関数 `root`・状態機械 `Drawer` はあえて再エクスポートしない
// （本モジュール冒頭の rustdoc「選択的 re-export」節参照）。
pub use fandhe_frontend_headless_ui::dialog::ContentIds;
pub use fandhe_frontend_headless_ui::drawer::{
    backdrop, close_trigger, content, description, positioner, title, trigger, DrawerPlacement,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;
// `trigger`/`backdrop` 等の `state` 引数は `state` モジュール由来で上記選択的
// 再エクスポートでは到達しない。呼び出し側が `fandhe-frontend-pre-styled-ui`
// のみに依存して呼び出せることを保証するための明示再エクスポート
// （イシュー #685、[`crate::dialog`] と同型）。
pub use fandhe_frontend_headless_ui::state::{DisclosureAction, OpenState};

/// headless `drawer` anatomy の `data-part` 一覧（`crates/headless-ui/src/drawer.rs`
/// の `ANATOMY.part(...)` 呼び出しと同期させる契約。ずれると [`stylesheet`] が
/// 一部パーツの CSS を出力しない fail-closed 側の不具合として現れるため、
/// 変更時は両ファイルを合わせて確認する）。
const SLOTS: &[&str] = &[
    "root",
    "trigger",
    "backdrop",
    "positioner",
    "content",
    "title",
    "description",
    "close-trigger",
];

/// この styled Drawer の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("drawer", SLOTS)
        .base(
            "backdrop",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                // イシュー #1694: `--fandhe-z-index-overlay`
                // （Theme::default() では 1300）。単独利用時のフォール
                // バックとして旧生値 1000 を残す（dialog/toast/date_picker
                // と同型）。
                decl("z-index", "var(--fandhe-z-index-overlay, 1000)"),
                // イシュー #1694: `--fandhe-color-bg-overlay`
                // （light 0.4 / dark 0.6）。単独利用時のフォールバックとして
                // 旧生値を残す（透明化して暗幕が消えないための安全側判断）。
                decl(
                    "background",
                    "var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.4))",
                ),
            ],
        )
        .base(
            "positioner",
            vec![
                decl("position", "fixed"),
                decl("inset", "0"),
                // イシュー #1694: `--fandhe-z-index-modal`
                // （Theme::default() では 1400、backdrop の overlay より前面）。
                decl("z-index", "var(--fandhe-z-index-modal, 1001)"),
                decl("display", "flex"),
            ],
        )
        .base(
            "content",
            vec![
                // イシュー #1695: close-trigger の絶対配置基準（枠・影・
                // サイズではないため #1694 側ではなく本イシューで追加。
                // dialog の同型追加（イシュー #1693）と同じ判断）。
                decl("position", "relative"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                // `docs/design/pre-styled-ui-scale-tokens.md` §3.2:
                // dialog/drawer content = lg。参照サイトが共通して持つ
                // 面パネルの影が本モジュールに欠落していたため新規追加
                // （イシュー #1694）。`border-radius` は追加しない
                // （drawer パネルは画面端に接する全高/全幅パネルであり、
                // 参照サイトも角丸を持たない、モジュール冒頭 rustdoc 参照）。
                decl("box-shadow", "var(--fandhe-shadow-lg)"),
                decl(
                    "padding",
                    "var(--fandhe-drawer-content-padding, var(--fandhe-space-6))",
                ),
                // `data-placement` の state が height/width: 100% を content
                // へ指定する（下記 state 参照）。content-box（既定）のままだと
                // padding が 100% の外側に加算され、start/end は viewport 高さ
                // を、top/bottom は viewport 幅を超えて溢れる。border-box で
                // padding を寸法に含めることで overflow を防ぐ。
                decl("box-sizing", "border-box"),
                decl("overflow-y", "auto"),
            ],
        )
        .base(
            "title",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-lg)"),
                decl("font-weight", "var(--fandhe-font-font-weight-semibold)"),
                decl("line-height", "var(--fandhe-font-line-height-tight)"),
                decl("margin", "0 0 var(--fandhe-space-2) 0"),
                // イシュー #1695（dialog #1693/PR #1795 の同型是正）:
                // close-trigger を content 右上へ絶対配置で重ねているため、
                // title 側にインライン終端方向のガターを確保しないと、
                // title が折り返す/長い場合にテキストと close-trigger が
                // 重なる。close-trigger は `box-sizing: border-box` を
                // 明示するため実占有幅は `width`（`--fandhe-space-8`）で
                // 確定するが、絶対配置の基準点は content の inline-end
                // からの `inset-inline-end`（`--fandhe-space-2`）だけ内側に
                // ずれているため、ガターは両者の合計
                // （`calc(width + inset)`）を確保する。
                decl(
                    "padding-inline-end",
                    "calc(var(--fandhe-space-8) + var(--fandhe-space-2))",
                ),
            ],
        )
        .base(
            "description",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("line-height", "var(--fandhe-font-line-height-normal)"),
                // イシュー #1695: footer 相当のアクション行（掲示用、
                // showcase デモ参照）との縦リズムを確保するため下余白を
                // 追加する（dialog #1693 と同型）。
                decl("margin", "0 0 var(--fandhe-space-4) 0"),
            ],
        )
        .base(
            "trigger",
            vec![
                // イシュー #1694: `<button type="button">` 実体でありながら
                // 枠・背景・角丸・padding が一切なく UA 既定外観のままだった
                // ため、操作部品カテゴリ既定段（dialog/file-upload の
                // trigger と同型）を新規追加する。
                decl("background", "var(--fandhe-color-bg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("cursor", "pointer"),
                decl("color", "var(--fandhe-color-fg)"),
                hover_bg_muted(),
            ]
            .into_iter()
            .chain(transition_declarations(
                "background, border-color",
                MotionDuration::Fast,
            ))
            .collect(),
        )
        // イシュー #1695（dialog #1693/PR #1795 の同型適用）: content 右上の
        // ゴーストボタン化（参照サイト標準）。`position: absolute` は本
        // イシューで追加した `content` の `position: relative` を基準と
        // する。位置指定は論理プロパティで統一する（`inset-block-start`/
        // `inset-inline-end`）。
        //
        // `close-trigger` はアイコン専用契約（dialog 0.59.0 と同型の 0.x
        // 破壊的変更）: 絶対配置 + `title` 側の固定ガター
        // （`calc(var(--fandhe-space-8) + var(--fandhe-space-2))`）の
        // 組み合わせでは複数文字テキストが `title` と重なるため、
        // children は 1〜2 文字のグリフ相当に限定し、支援技術向けラベルは
        // `("aria-label", "...")` 属性で付与する契約へ変更する。
        // `width`/`height` の明示固定と `overflow: hidden` により、誤って
        // 長いテキストを渡しても正方形の枠内で切り詰められ `title` への
        // 重なりを防ぐ（緩和策であり正式な使用法ではない）。呼び出し側は
        // `close_trigger(vec![("aria-label", "Close")], vec![text("×")])`
        // の形へ移行すること。
        .base(
            "close-trigger",
            [
                vec![
                    decl("position", "absolute"),
                    decl("inset-block-start", "var(--fandhe-space-2)"),
                    decl("inset-inline-end", "var(--fandhe-space-2)"),
                    decl("display", "inline-flex"),
                    decl("align-items", "center"),
                    decl("justify-content", "center"),
                    // dialog #1795 の codex-review/Bugbot 指摘と同型:
                    // `box-sizing` 未指定だと既定の `content-box` になり、
                    // `padding`（`--fandhe-space-1`）が `width`/`height`
                    // （`--fandhe-space-8`）に加算されて実描画サイズが
                    // documented な 2rem square を超える。`border-box` を
                    // 明示し、`width`/`height` を実占有サイズの確定値にする。
                    decl("box-sizing", "border-box"),
                    decl("width", "var(--fandhe-space-8)"),
                    decl("height", "var(--fandhe-space-8)"),
                    decl("overflow", "hidden"),
                    decl("border", "none"),
                    decl("border-radius", "var(--fandhe-radius-sm)"),
                    decl("background", "transparent"),
                    decl("padding", "var(--fandhe-space-1)"),
                    decl("cursor", "pointer"),
                    decl("color", "var(--fandhe-color-fg-muted)"),
                ],
                vec![hover_bg_muted()],
                // `hover_surface_declarations()`（下記 state 登録）は
                // `background` のみを変更し `color` を変える規則を持たない
                // ため、到達しない宣言を避けて `background` のみ
                // transition 対象にする。
                transition_declarations("background", MotionDuration::Fast),
            ]
            .concat(),
        )
        .state(
            "close-trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #758 受け入れ条件: placement 4 方向。positioner の flex
        // 方向・主軸整列を切り替える（row 方向の flex-start/flex-end は
        // 書字方向依存のため RTL でも自然に反転する、モジュール冒頭 rustdoc
        // 参照）。
        .state(
            "positioner",
            StateCondition::AttrEq("data-placement", "start"),
            vec![
                decl("flex-direction", "row"),
                decl("justify-content", "flex-start"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-placement", "end"),
            vec![
                decl("flex-direction", "row"),
                decl("justify-content", "flex-end"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-placement", "top"),
            vec![
                decl("flex-direction", "column"),
                decl("justify-content", "flex-start"),
            ],
        )
        .state(
            "positioner",
            StateCondition::AttrEq("data-placement", "bottom"),
            vec![
                decl("flex-direction", "column"),
                decl("justify-content", "flex-end"),
            ],
        )
        // content の占有寸法軸（start/end は幅、top/bottom は高さ）。
        .state(
            "content",
            StateCondition::AttrEq("data-placement", "start"),
            vec![
                decl("width", "var(--fandhe-drawer-size, 20rem)"),
                decl("height", "100%"),
            ],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-placement", "end"),
            vec![
                decl("width", "var(--fandhe-drawer-size, 20rem)"),
                decl("height", "100%"),
            ],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-placement", "top"),
            vec![
                decl("height", "var(--fandhe-drawer-size, 20rem)"),
                decl("width", "100%"),
            ],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-placement", "bottom"),
            vec![
                decl("height", "var(--fandhe-drawer-size, 20rem)"),
                decl("width", "100%"),
            ],
        )
        // dialog と同型の開閉状態連動（[`crate::dialog`] 同様、方向別
        // スライドは表現できない API 制約のためスコープ外、モジュール冒頭
        // rustdoc 参照）。
        .state(
            "backdrop",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1")],
        )
        .state(
            "backdrop",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("opacity", "0")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "open"),
            vec![decl("opacity", "1")],
        )
        .state(
            "content",
            StateCondition::AttrEq("data-state", "closed"),
            vec![decl("opacity", "0")],
        )
        // PR #575 Bugbot 指摘対応（dialog 由来、High）: positioner の base
        // 規則が `display: flex` を宣言しており、UA 既定の
        // `[hidden] { display: none }` を詳細度で上書きしてしまう。closed
        // 時に headless 層が付与する `hidden` 属性を確実に非表示化として
        // 機能させるため、より詳細度の高い `[hidden]` 属性セレクタで
        // `display: none` を明示的に上書きする。
        .state(
            "positioner",
            StateCondition::Attr("hidden"),
            vec![decl("display", "none")],
        )
        // イシュー #1694: trigger の hover surface（dialog/file-upload の
        // trigger と同型、`--fandhe-hover-bg` は上記 base の
        // `hover_bg_muted()` が定義する）。
        .state(
            "trigger",
            StateCondition::Hover,
            hover_surface_declarations(),
        )
        // イシュー #643 / #1694: キーボード操作時のみのフォーカスリング。
        // canonical ヘルパへ置換（出力値は従来と同一、トークン参照 +
        // 旧来値フォールバックへの置換のみで見た目は不変）。close-trigger
        // 側は兄弟イシュー #1695 の担当のため直書きのまま変更しない。
        .state(
            "trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #1695: close-trigger の focus-visible をイシュー #1424
        // 共通トークンへ移行する（trigger 側は #1694 で同じ canonical
        // ヘルパへ移行済み、これで両者ともトークン参照形で揃う）。
        .state(
            "close-trigger",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #758: `size` variant（root スコープの CSS custom
        // property。Md はフォールバック値と同一の現行外観を維持する）。
        // イシュー #1681: Xs/Xl は Sm(16)→Md(20)→Lg(28) の非等差進行を、
        // 両端それぞれ隣接差分（Sm-Md=4 / Md-Lg=8）を踏襲して外挿した値。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-drawer-size", "12rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-drawer-size", "16rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-drawer-size", "20rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-drawer-size", "28rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-drawer-size", "36rem")],
        )
        .default_variant(Size::Md)
}

/// この styled Drawer が生成する静的 CSS 全量を返す（決定的。同一プロセス内で
/// 複数回呼んでも常にバイト単位で同一の文字列を返す、[`SlotRecipe::css`](crate::recipe::SlotRecipe::css)
/// の契約をそのまま継承する）。
///
/// 呼び出し元は返り値を静的 `.css` ファイルとして配信する、または
/// [`crate::stylesheet::StyleSheet::push_css`] へ渡して `<style>` 要素へ
/// 埋め込む（#605、[`crate`] 冒頭の不変条件を参照）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// styled root パーツを組み立てる。`size` に応じたクラスを付与する唯一の
/// パーツ（[`drop_class_attr`] により呼び出し側の `class` は除去してから
/// 合成する）。実体は [`fandhe_frontend_headless_ui::drawer::root`] へ
/// 委譲する。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::drawer::{self, DrawerPlacement, OpenState};
/// use fandhe_frontend_pre_styled_ui::Size;
///
/// let node = drawer::root(Size::Md, OpenState::Open, DrawerPlacement::End, vec![], vec![]);
/// assert!(render(&node).contains(r#"data-scope="drawer" data-part="root""#));
/// ```
#[must_use]
pub fn root<'a>(
    size: Size,
    state: OpenState,
    placement: DrawerPlacement,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    fandhe_frontend_headless_ui::drawer::root(state, placement, merged, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_headless_ui::state::OpenState;

    #[test]
    fn stylesheet_is_deterministic_and_targets_data_scope_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="drawer"][data-part="content"]"#));
        assert!(a.contains(r#"[data-scope="drawer"][data-part="backdrop"]"#));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    #[test]
    fn backdrop_and_positioner_declare_stacking_order() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="drawer"][data-part="backdrop"] {"#));
        // イシュー #1694 でトークン参照へ置換（旧生値はフォールバックとして残す）。
        assert!(css.contains("z-index: var(--fandhe-z-index-overlay, 1000);"));
        assert!(css.contains("z-index: var(--fandhe-z-index-modal, 1001);"));
    }

    #[test]
    fn backdrop_uses_bg_overlay_token_with_legacy_fallback() {
        // イシュー #1694: backdrop の暗幕をライト/ダーク対応トークン
        // （イシュー #1422）へ切り替える。`drawer::stylesheet()` 単独利用
        // （テーマ CSS 非注入）でも透明化しないよう旧生値をフォールバック
        // として残す（dialog #1692 と同型）。
        let css = stylesheet();
        assert!(css.contains("background: var(--fandhe-color-bg-overlay, rgba(0, 0, 0, 0.4));"));
    }

    #[test]
    fn content_declares_shadow_lg() {
        // イシュー #1694: `docs/design/pre-styled-ui-scale-tokens.md` §3.2
        // が割り当てる面パネルの影を新規追加する。`border-radius` は
        // 意図的に追加しない（モジュール冒頭 rustdoc 参照）。
        let css = stylesheet();
        assert!(css.contains("box-shadow: var(--fandhe-shadow-lg);"));
    }

    #[test]
    fn trigger_declares_button_chrome_and_hover_and_transition() {
        // イシュー #1694: trigger をボタンとしての枠・背景・角丸・padding
        // を持つ操作部品既定段（dialog #1692/file-upload #1696 と同型）へ
        // 載せ、hover surface + transition を新規登録する。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="drawer"][data-part="trigger"] {"#));
        let trigger_start = css
            .find(r#"[data-scope="drawer"][data-part="trigger"] {"#)
            .expect("trigger base rule must be present");
        let rule_body = &css[trigger_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        let base_rule = &rule_body[..rule_end];
        assert!(base_rule.contains("border: 1px solid var(--fandhe-color-border);"));
        assert!(base_rule.contains("border-radius: var(--fandhe-radius-md);"));
        assert!(base_rule.contains("background: var(--fandhe-color-bg);"));
        assert!(base_rule.contains("padding: var(--fandhe-space-2) var(--fandhe-space-3);"));
        assert!(base_rule.contains("--fandhe-hover-bg: var(--fandhe-color-bg-muted);"));
        assert!(base_rule.contains("transition-property: background, border-color;"));

        assert!(css.contains(
            r#"[data-scope="drawer"][data-part="trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn closed_positioner_hidden_attr_overrides_display_flex() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="drawer"][data-part="positioner"][hidden] {"#));
        let positioner_hidden_rule_start = css
            .find(r#"[data-scope="drawer"][data-part="positioner"][hidden] {"#)
            .expect("positioner[hidden] rule must be present");
        let rule_body = &css[positioner_hidden_rule_start..];
        let rule_end = rule_body.find('}').expect("rule must be closed");
        assert!(rule_body[..rule_end].contains("display: none;"));
    }

    #[test]
    fn reexported_root_renders_with_headless_anatomy_attrs() {
        let html = render(&root(
            Size::Md,
            OpenState::Closed,
            DrawerPlacement::End,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="drawer""#));
        assert!(html.contains(r#"data-part="root""#));
    }

    #[test]
    fn all_four_placements_output_expected_layout_rules() {
        let css = stylesheet();
        for (placement, justify) in [
            ("start", "flex-start"),
            ("end", "flex-end"),
            ("top", "flex-start"),
            ("bottom", "flex-end"),
        ] {
            assert!(css.contains(&format!(
                r#"[data-scope="drawer"][data-part="positioner"][data-placement="{placement}"]"#
            )));
            assert!(css.contains(&format!("justify-content: {justify};")));
            assert!(css.contains(&format!(
                r#"[data-scope="drawer"][data-part="content"][data-placement="{placement}"]"#
            )));
        }
    }

    // --- イシュー #758: size variant ---

    #[test]
    fn size_variant_appends_single_class_to_root_and_drops_caller_class() {
        for size in [Size::Sm, Size::Md, Size::Lg] {
            let html = render(&root(
                size,
                OpenState::Closed,
                DrawerPlacement::End,
                vec![("class", "attacker")],
                vec![],
            ));
            let expected_class = format!("fd-drawer--size-{}", size.value());
            assert!(html.contains(&expected_class), "html={html}");
            assert!(!html.contains("attacker"));
            assert_eq!(html.matches("class=\"").count(), 1);
        }
    }

    #[test]
    fn default_variant_is_md_and_matches_fallback() {
        let css = stylesheet();
        assert!(css.contains("width: var(--fandhe-drawer-size, 20rem);"));
        assert!(css.contains("--fandhe-drawer-size: 20rem;"));
    }

    #[test]
    fn trigger_and_close_trigger_declare_focus_visible_ring() {
        // イシュー #1694/#1695: trigger/close-trigger の双方が共通トークン
        // （#1424 canonical ヘルパ）へ移行済みであることを固定する（dialog
        // #1692/#1693 の base 取り込みと同型）。
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="drawer"][data-part="trigger"]:focus-visible {"#));
        assert!(css.contains(r#"[data-scope="drawer"][data-part="close-trigger"]:focus-visible {"#));
        assert!(css.matches(
            "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
        ).count() == 2);
    }

    #[test]
    fn close_trigger_declares_hover_surface_inside_hover_media_query() {
        // イシュー #1695: close-trigger の hover 規則が `@media (hover:
        // hover)` 内に `:hover:not([data-disabled])` で出力されることを
        // 固定する（#1425 規約、`SlotRecipe::css` の集約契約。dialog #1693
        // と同型）。
        let css = stylesheet();
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(
            r#"[data-scope="drawer"][data-part="close-trigger"]:hover:not([data-disabled]) {"#
        ));
        assert!(css.contains("background: var(--fandhe-hover-bg);"));
    }

    #[test]
    fn content_and_close_trigger_declare_positioning_pair() {
        // イシュー #1695: close-trigger の絶対配置は content の
        // `position: relative` を基準とする（対で出力されることを固定、
        // dialog #1693 と同型）。
        let css = stylesheet();
        let content_start = css
            .find(r#"[data-scope="drawer"][data-part="content"] {"#)
            .expect("content base rule must be present");
        let content_end = css[content_start..].find('}').unwrap() + content_start;
        assert!(css[content_start..content_end].contains("position: relative;"));

        let close_trigger_start = css
            .find(r#"[data-scope="drawer"][data-part="close-trigger"] {"#)
            .expect("close-trigger base rule must be present");
        let close_trigger_end = css[close_trigger_start..].find('}').unwrap() + close_trigger_start;
        assert!(css[close_trigger_start..close_trigger_end].contains("position: absolute;"));
    }

    #[test]
    fn close_trigger_uses_border_box_and_title_gutter_matches_occupied_space() {
        // dialog #1795 の codex-review P1 指摘 + Cursor Bugbot 指摘と同型:
        // close-trigger は `width`/`height`（`--fandhe-space-8`）と
        // `padding`（`--fandhe-space-1`）を併せ持つため、
        // `box-sizing: border-box` がないと content-box の既定で実描画
        // サイズが documented な 2rem square を超える。かつ、`title` 側の
        // ガター（`padding-inline-end`）は close-trigger の実占有幅
        // （`width`）と絶対配置の基準点のずれ（`inset-inline-end`）の
        // 合計を確保しないと、xs drawer のような狭い content で title と
        // 重なり得る。両者を固定する。
        let css = stylesheet();

        let close_trigger_start = css
            .find(r#"[data-scope="drawer"][data-part="close-trigger"] {"#)
            .expect("close-trigger base rule must be present");
        let close_trigger_end = css[close_trigger_start..].find('}').unwrap() + close_trigger_start;
        let close_trigger_rule = &css[close_trigger_start..close_trigger_end];
        assert!(close_trigger_rule.contains("box-sizing: border-box;"));
        assert!(close_trigger_rule.contains("width: var(--fandhe-space-8);"));
        assert!(close_trigger_rule.contains("height: var(--fandhe-space-8);"));
        assert!(close_trigger_rule.contains("inset-inline-end: var(--fandhe-space-2);"));

        let title_start = css
            .find(r#"[data-scope="drawer"][data-part="title"] {"#)
            .expect("title base rule must be present");
        let title_end = css[title_start..].find('}').unwrap() + title_start;
        let title_rule = &css[title_start..title_end];
        assert!(title_rule
            .contains("padding-inline-end: calc(var(--fandhe-space-8) + var(--fandhe-space-2));"));
    }

    #[test]
    fn stylesheet_links_data_state_to_style_open_and_closed() {
        let css = stylesheet();
        assert!(css.contains(r#"[data-scope="drawer"][data-part="backdrop"][data-state="open"]"#));
        assert!(css.contains(r#"[data-scope="drawer"][data-part="backdrop"][data-state="closed"]"#));
        assert!(css.contains(r#"[data-scope="drawer"][data-part="content"][data-state="open"]"#));
    }

    #[test]
    fn ssr_and_hydration_round_trip_via_reexported_drawer_state_machine() {
        // イシュー #758 受け入れ条件: 「SSR / hydration 両経路の動作確認」を
        // headless `Drawer`（headless の Component/Hydrate 実装を継承。
        // 本モジュールから再エクスポートしないため、状態機械を使う呼び出し側
        // と同じくエスケープハッチ経由で直接 import する。モジュール冒頭
        // rustdoc「選択的 re-export」節参照）経由で固定する。
        use fandhe_frontend_headless_ui::drawer::Drawer;
        use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

        let mut d = Drawer::default();
        assert_eq!(d.state(), OpenState::Closed);

        // SSR: 状態なし初期描画には data-hydrate-* が出ない。
        let ssr_html = render(&d.root(vec![], vec![]));
        assert!(ssr_html.contains(r#"data-state="closed""#));

        // dispatch で開閉し、hydration 属性へ反映されることを確認する。
        assert!(dispatch(&mut d, "open", ""));
        let hydrate_html = render(&render_for_hydration(&d));
        assert!(hydrate_html.contains(r#"data-hydrate-state="open""#));
        assert!(hydrate_html.contains(r#"data-hydrate-placement="end""#));

        // クライアント側の改ざん耐性のある復元経路が Drawer 経由でも機能する。
        let restored = Drawer::from_hydration_attrs(&d.hydration_attrs()).unwrap();
        assert_eq!(restored.state(), OpenState::Open);
    }
}
