//! layout（FLIP: First-Last-Invert-Play）アニメーションの座標計測・適用
//! （イシュー #2518）。
//!
//! motion.dev の `layout`/`layoutId` 相当のうち、単一要素の layout 変化
//! （keyed list の `Move` による並べ替え等）を対象とする。異要素間の
//! クロスフェード（`layoutId` 相当）はスコープ外
//! （`docs/design/animation-core-architecture.md` §6.2、実装計画 §3.6）。
//!
//! # 2 層構成
//!
//! - 純粋層（[`Rect`]/[`FlipDelta`]/[`invert`]/[`to_transform_css`]）は
//!   DOM 非依存で native `cargo test` から検証できる。
//! - wasm32 限定層（[`measure`]/[`play`]）は `getBoundingClientRect()`
//!   による実測・`window.matchMedia` による reduced-motion 判定・
//!   [`fandhe_animation::spring::Spring`] + [`crate::raf_driver`] による
//!   再生を担う。
//!
//! `data-*` 属性からの呼び出し配線（keyed list の構造変化前後で本モジュール
//! を呼ぶタイミング判定）は `fandhe-frontend-wasm-full` 側
//! （`crate::layout_flip`）の責務であり、本モジュールは持たない
//! （クレート doc「責務境界」節参照）。
//!
//! # 対応範囲（サポートする祖先変換の契約、イシュー #2518）
//!
//! [`play`]（wasm32 限定層）が正しく補正できる**祖先**の CSS `transform`
//! は「正の軸整列スケール」（並進・拡大縮小・その組み合わせ。回転・
//! skew・反転〔負のスケール〕を含まない）に限る。詳細な理由・実測例は
//! `is_axis_aligned_linear`（内部関数）の rustdoc を正とする。祖先が
//! この範囲外の変換を持つ場合、`play` は再生自体を省略し、即座に元の
//! スタイルへ復元した収束済みハンドルを返す（見た目上は瞬間的な移動に
//! なるが、誤った符号・軸の補正を適用するよりは安全という判断、
//! fail-safe）。この制限は意図的にこの範囲に留めている（本イシューは
//! wasm-full 側の配線のみを担い、祖先の回転・反転まで正しく扱うには
//! 矩形全体〔4 頂点〕をローカル座標へ逆変換する必要があり本イシューの
//! 範囲を超えるため）。祖先変換への対応を拡張する場合は、この制限の
//! 再評価から始めること。
//!
//! 一方、**要素自身**が持つ元の `transform`（[`OriginalStyle`] が保持・
//! 合成する対象）には上記の制限は課していない: 回転・skew を含む任意の
//! 可逆な変形を正しく扱える。
//!
//! **CSS `zoom` 非対応**（codex-review 第 8 ラウンド是正、イシュー
//! #2518）: 要素自身・祖先いずれかが CSS `zoom` に恒等（`1`）以外の値を
//! 持つ場合、`play` は再生自体を省略する。`zoom` は CSS ピクセル座標系
//! そのものを拡縮するため `getBoundingClientRect()` の viewport 座標が
//! 拡縮されるが、補正計算は `transform` プロパティのみを読むためこの
//! 拡縮を捕捉できず、放置すると並進補正が過大/過小になる
//! （[`is_non_identity_zoom`] doc 参照）。
//!
//! # 計測・合成の再設計（codex-review 第 4 ラウンド是正、イシュー #2518）
//!
//! 第 2〜3 ラウンドは「要素自身の変形 `O` を代数的に解析し、FLIP 補正の
//! 並進成分へ補正項を加える」方式（`O` の行列を [`parse_matrix_components`]
//! で解析し逆行列を掛ける等）を積み増していたが、要素自身のサイズが
//! 変化する場合に収束しない構造的な欠陥があった（`<flip> <original>` の
//! 合成順では `O` の並進が FLIP のスケールに巻き込まれて拡大され、
//! `<original> <flip>` では逆に `O` 自身の並進が Last の原点を使って
//! 拡縮されてしまう）。本ラウンドはこの欠陥を補正の積み増しではなく
//! **計測・合成そのものの再設計**で解消する。
//!
//! - **First は視覚矩形**（`getBoundingClientRect()`、要素自身の transform
//!   を含む「現在実際に表示されている矩形」）を測る。アニメーション
//!   進行中に呼ばれた場合も、その補正込みの現在表示位置がそのまま正しい
//!   Before になる（連続性・スクロール反映を計測だけで満たす。旧実装の
//!   `current_viewport_delta`/`apply_delta_to_rect` による引き継ぎ計算は
//!   不要になったため削除した）。
//! - **Last は 2 種類測る**: (1) 元の `transform`/`transform-origin` を
//!   復元した状態での視覚矩形（`last_visual`）と、(2) `transform: none`
//!   （[`measure_clearing_transform`]）で一時的に要素自身の変形を無効化
//!   した「layout（未変形）矩形」（`last_layout`）。要素自身が原点固定の
//!   平行移動のみでなくサイズも変化する変形（例: `scale(2)` の中心原点で
//!   幅が変わる）を持つ場合、`last_visual`（変形後）だけでは変形の基準点
//!   `O` の原点が Last のどこにあったかを復元できず、Invert 直後の視覚
//!   矩形が First と一致しない（詳細な数値例は `anchor_correct`
//!   の rustdoc を正とする）。**この 1 箇所（Last の layout 矩形計測）に
//!   限り** `transform: none !important` への一時書き込みを行う
//!   （measure したら直ちに元の値へ戻すため見た目・進行中の状態には
//!   影響しない）。First 側・要素自身の変形が無い/平行移動のみの Last
//!   側では発生しない（`anchor_correct` が `(1 − sx/sy)` を係数に
//!   持つため、サイズが変化しない場合は自動的に無補正となる）。
//! - **合成は要素自身の変形を不透明に扱う**: `O`（[`OriginalStyle::
//!   compose_transform`] が使う `getComputedStyle().transform` の行列
//!   文字列）は一切解析せず、`transform-origin: 0 0 !important` の下で
//!   `"<F> translate(ox,oy) <O> translate(-ox,-oy)"`（`F` が最外側）と
//!   連結するだけで、`translate(ox,oy) <O> translate(-ox,-oy)` の部分が
//!   「`O` を実際の基準点 `(ox, oy)` で適用した場合と同じ効果」を再現し、
//!   その結果へ `F`（FLIP の並進・スケール、`anchor_correct` が
//!   算出する viewport 座標の delta を祖先の逆行列でローカル化したもの）
//!   が外側から適用される。`O` の回転・skew・並進がどのような値であっても
//!   この合成だけで正しく動く（導出は `anchor_correct` 参照）。

/// 要素の矩形（`getBoundingClientRect()` 相当、px 単位）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Invert 段階で算出する補正 transform（`transform-origin: 0 0` 基準）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlipDelta {
    pub tx: f64,
    pub ty: f64,
    pub sx: f64,
    pub sy: f64,
}

/// 恒等変換（Play 段階の収束先）。
pub const IDENTITY: FlipDelta = FlipDelta {
    tx: 0.0,
    ty: 0.0,
    sx: 1.0,
    sy: 1.0,
};

impl fandhe_animation::interpolate::Interpolate for FlipDelta {
    fn interpolate(&self, other: &Self, t: f64) -> Self {
        FlipDelta {
            tx: self.tx.interpolate(&other.tx, t),
            ty: self.ty.interpolate(&other.ty, t),
            sx: self.sx.interpolate(&other.sx, t),
            sy: self.sy.interpolate(&other.sy, t),
        }
    }
}

/// `first`（変化前の矩形）→ `last`（変化後の矩形）の Invert 補正値を返す。
///
/// `transform-origin: 0 0` を前提とした top-left 基準（中心基準より単純で
/// `Rect` のみから決定的に計算できる、実装計画 §3.1）。
///
/// 以下のいずれかで `None` を返す（ライブラリコードで panic しない、
/// `fandhe_animation::spring::Spring::new` と同じ fail-safe 方針）:
/// - 8 成分のいずれかが非有限（NaN/Inf）
/// - `last.width`/`last.height` が 0 以下（`display: none` 等によるゼロ
///   除算・スケール反転を避ける）
#[must_use]
pub fn invert(first: Rect, last: Rect) -> Option<FlipDelta> {
    let components = [
        first.x,
        first.y,
        first.width,
        first.height,
        last.x,
        last.y,
        last.width,
        last.height,
    ];
    if !components.iter().all(|v| v.is_finite()) {
        return None;
    }
    if last.width <= 0.0 || last.height <= 0.0 {
        return None;
    }
    Some(FlipDelta {
        tx: first.x - last.x,
        ty: first.y - last.y,
        sx: first.width / last.width,
        sy: first.height / last.height,
    })
}

/// [`FlipDelta`] を CSS `transform` 値へ変換する。
///
/// # セキュリティ（A03: CSS injection）
///
/// `delta` の 4 成分はいずれも `f64`（`invert()` の出力、呼び出し元が
/// `getBoundingClientRect()` から構築した [`Rect`] の減算・除算のみで
/// 導出される）であり、利用者・DOM 属性由来の文字列を一切混入しない
/// （`dom_target.rs` と同じ不変条件）。
#[must_use]
pub fn to_transform_css(delta: FlipDelta) -> String {
    format!(
        "translate({}px, {}px) scale({}, {})",
        delta.tx, delta.ty, delta.sx, delta.sy
    )
}

/// [`invert`] が算出した viewport 座標の `delta`（`first_visual` →
/// `last_visual`）へ、要素自身の変形の基準点のずれを補正する項を加える
/// （DOM 非依存の純粋関数、`wiring::play_after` 相当の呼び出し元から
/// 呼ばれる。native `cargo test` で検証できる、codex-review 第 4 ラウンド
/// 是正・再設計、イシュー #2518）。
///
/// # 補正が必要な理由
///
/// [`play`](wiring::play) は FLIP の補正（本関数が返す `delta`）を要素自身
/// の変形 `O`（[`OriginalStyle::compose_transform`] が合成する
/// `getComputedStyle().transform`）の**外側**（`transform-origin: 0 0` 基準）
/// に適用する。`O` が並進のみ（原点固定でサイズが変わらない）なら
/// [`invert`] が返す viewport 座標の `delta` をそのまま祖先の逆行列で
/// ローカル化するだけで正しいが、`O` が非自明な基準点（例:
/// `transform-origin: 50% 50%` の既定値）を持ち要素自身のサイズが変化する
/// 場合、Last の視覚矩形（`last_visual`、`O` 適用後）と layout 矩形
/// （`last_layout`、`O` を一時的に無効化して測った未変形の矩形）の間に
/// `O` の基準点由来のずれが生じ、これを無視すると Invert 直後の視覚矩形
/// が First と一致しない。
///
/// # 導出
///
/// `O` を「layout-local 座標 `p` → 現在の視覚位置」を写す不透明な可逆
/// アフィン変換として扱う（回転・skew・並進のいずれを持っていてもよい、
/// 内部を解析しない）。`p = 0`（layout 矩形の左上）における `O` の像は
/// 定義から `anchor = last_visual − last_layout`（両者の差、祖先の線形
/// 変換 `A` の影響を除くため後述のとおり `A⁻¹` を掛けて `anchor_local` を
/// 得る）。FLIP は `F(q) = S·q + t`（`S` は [`invert`] の `sx`/`sy`、
/// `transform-origin: 0 0`）を `O` の結果 `q` へ外側から適用するため、
/// `p = 0` における最終的な viewport 位置は
/// `last_layout + A(S·anchor_local + t)` になる。これが `first_visual`
/// と一致する条件から
///
/// ```text
/// t = d_local + (1 − S)·anchor_local
/// d_local  = A⁻¹·(first_visual − last_visual)  （[`invert`] の delta を
///             祖先の逆行列でローカル化したもの、[`wiring::to_local_delta`]
///             が既に行う）
/// anchor_local = A⁻¹·(last_visual − last_layout)
/// ```
///
/// が導かれる（`A`・`A⁻¹` は対角、すなわち祖先が「正の軸整列スケール」
/// の場合に限り可換なため、本関数は viewport 座標のまま
/// `t_viewport = d_viewport + (1 − S)·anchor_viewport` を計算し、その後
/// [`wiring::to_local_delta`] が一括で `A⁻¹` を掛ける設計にできる
/// （対角行列の可換性により順序を入れ替えても数学的に等価）。`O` が回転
/// 等の線形部分を持っていても、上記の等式は `p = 0` （原点）でのみ評価
/// しているため `O` の線形部分は両辺から相殺されて消える
/// （`anchor_local` の定義に `O` の効果が丸ごと閉じ込められているため）。
///
/// `sx == 1 && sy == 1`（サイズ変化なし）の場合は係数 `(1 − S)` が 0 に
/// なり本補正は自動的に無効化される（並進のみの変形・要素自身の変形が
/// 無い場合に旧来の挙動と一致する）。
///
/// # 数値例（codex-review 第 4 ラウンド P1 指摘の再現ケース）
///
/// 要素自身が `scale(2)`・`transform-origin: 50% 50%`（既定）、祖先変換
/// なし、レイアウト幅が `100px → 50px`（`left: 0` 固定）に変化した場合:
/// `last_layout = Rect { x: 0, width: 50, .. }`、`last_visual = Rect { x:
/// -25, width: 100, .. }`（中心 25 基準で 2 倍）、`first_visual = Rect {
/// x: -50, width: 200, .. }`（変化前、中心 50 基準で 2 倍）。
/// `sx = 200/100 = 2`、`d = -50 - (-25) = -25`、`anchor = -25 - 0 = -25`、
/// `t = -25 + (1-2)*(-25) = -25 + 25 = 0`。Invert 直後（`F` を丸ごと適用
/// した瞬間）の視覚矩形の左端は `last_layout.x + O(0)*1 + t` を
/// 経由して `-50` に一致する（native テスト
/// `anchor_correct_reproduces_first_rect_under_own_scale` で固定）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
pub fn anchor_correct(delta: FlipDelta, last_visual: Rect, last_layout: Rect) -> FlipDelta {
    let anchor_x = last_visual.x - last_layout.x;
    let anchor_y = last_visual.y - last_layout.y;
    FlipDelta {
        tx: delta.tx + (1.0 - delta.sx) * anchor_x,
        ty: delta.ty + (1.0 - delta.sy) * anchor_y,
        sx: delta.sx,
        sy: delta.sy,
    }
}

/// `"matrix(a, b, c, d, e, f)"` 形式の 2D computed transform 文字列から
/// 6 成分すべてを抽出する（DOM 非依存の純粋関数）。
///
/// `wiring::ancestor_linear_matrix`（線形部分 `a,b,c,d` のみ使用、祖先の
/// 累積変換の逆行列で並進量をローカル座標へ変換する）から呼ばれる
/// （レビュー指摘対応、イシュー #2518 さらなるフォローアップ）。`e`/`f`
/// （並進部分）は現在どの呼び出し元も使わないが、`matrix(...)` の 6 引数
/// 形式を構造的に検証する（引数の個数が異なる形式を弾く）ためそのまま
/// 抽出しておく。`none`・`matrix3d(...)`・引数の個数が異なる・非有限値を
/// 含む形式は `None`。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn parse_matrix_components(value: &str) -> Option<[f64; 6]> {
    let inner = value.trim().strip_prefix("matrix(")?.strip_suffix(')')?;
    let mut parts = inner.split(',').map(|s| s.trim().parse::<f64>());
    let a = parts.next()?.ok()?;
    let b = parts.next()?.ok()?;
    let c = parts.next()?.ok()?;
    let d = parts.next()?.ok()?;
    let e = parts.next()?.ok()?;
    let f = parts.next()?.ok()?;
    if parts.next().is_some() {
        return None;
    }
    let components = [a, b, c, d, e, f];
    if components.iter().all(|v| v.is_finite()) {
        Some(components)
    } else {
        None
    }
}

/// 2x2 線形変換行列 `[[a, c], [b, d]]`（CSS `matrix(a,b,c,d,e,f)` の並進
/// 成分 `e,f` を除いた部分）の逆行列 `(a, b, c, d)` を返す（DOM 非依存の
/// 純粋関数、`wiring::ancestor_linear_matrix`/[`play`](wiring::play) から
/// 呼ばれる）。
///
/// レビュー指摘対応（イシュー #2518 さらなるフォローアップ、
/// codex-review P1「祖先変換の符号と軸方向を保持する」）: 祖先の
/// `scaleX(-1)`・`rotate(90deg)` 等、符号・軸が反転する変換の下でも
/// `getBoundingClientRect()` 由来の viewport 座標の並進量を正しく
/// ローカル座標へ戻すために使う（旧実装は `hypot` でスケールの絶対値
/// のみを抽出しており符号・回転を復元できなかった）。行列式が 0 に潰れる
/// （非可逆）・非有限値になる場合は `None`（呼び出し元は再生を省略する、
/// fail-safe）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn invert_linear_2x2(a: f64, b: f64, c: f64, d: f64) -> Option<(f64, f64, f64, f64)> {
    let det = a * d - b * c;
    if !det.is_finite() || det.abs() <= f64::EPSILON {
        return None;
    }
    let inverse = (d / det, -b / det, -c / det, a / det);
    let (ia, ib, ic, id) = inverse;
    if [ia, ib, ic, id].iter().all(|v| v.is_finite()) {
        Some(inverse)
    } else {
        None
    }
}

/// 2x2 線形変換行列 `(a, b, c, d)`（[`invert_linear_2x2`] と同じ
/// `[[a, c], [b, d]]` の表現）をベクトル `(x, y)` へ適用する（DOM 非依存
/// の純粋関数）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn apply_linear_2x2(matrix: (f64, f64, f64, f64), x: f64, y: f64) -> (f64, f64) {
    let (a, b, c, d) = matrix;
    (a * x + c * y, b * x + d * y)
}

/// 2x2 線形変換行列の合成 `outer * inner` を返す（`inner` が要素に近い
/// 祖先、`outer` がさらに外側の祖先。DOM 非依存の純粋関数、
/// `wiring::ancestor_linear_matrix` から呼ばれる）。
///
/// `inner` が先に適用され、その結果へ `outer` が適用される場合の合成
/// 行列（ネストした CSS `transform` の合成順）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn compose_linear_2x2(
    inner: (f64, f64, f64, f64),
    outer: (f64, f64, f64, f64),
) -> (f64, f64, f64, f64) {
    let (a1, b1, c1, d1) = inner;
    let (a2, b2, c2, d2) = outer;
    (
        a2 * a1 + c2 * b1,
        b2 * a1 + d2 * b1,
        a2 * c1 + c2 * d1,
        b2 * c1 + d2 * d1,
    )
}

/// `is_axis_aligned_linear` が「回転・skew なし」と判定する許容誤差
/// （DOM 非依存の純粋関数、[`is_axis_aligned_linear`] から使う）。
///
/// ブラウザの `getComputedStyle` は `rotate(90deg)`/`rotate(180deg)` 等の
/// computed `matrix(...)` の非対角成分を三角関数の数値誤差により厳密な
/// `0.0` ではなくごく僅かな値（例: `6.123e-17`）として返し得るため、
/// 厳密な等値比較ではなく許容誤差での判定が必要（レビュー指摘対応、
/// イシュー #2518 さらなるフォローアップ）。
#[cfg(any(target_arch = "wasm32", test))]
const AXIS_ALIGNED_EPSILON: f64 = 1e-6;

/// 2x2 線形変換行列 `(a, b, c, d)`（[`invert_linear_2x2`] と同じ
/// `[[a, c], [b, d]]` の表現）が「正の軸整列スケール」（回転・skew・
/// 反転（負のスケール）を含まない、対角成分が両方正）かどうかを判定
/// する（DOM 非依存の純粋関数、`wiring::to_local_delta` から**祖先**の
/// 累積変換に対して呼ばれる）。
///
/// # 対応範囲（祖先変換のサポート契約、イシュー #2518）
///
/// [`play`](wiring::play) が並進・スケール双方を正しく補正できる祖先
/// 変換は「正の軸整列スケール」（`b=c=0` かつ `a>0` かつ `d>0`）に限る。
/// 反転（`scaleX(-1)` 等、対角成分が負）・回転・skew を持つ祖先の下では
/// [`wiring::to_local_delta`] が `None` を返し、再生自体を省略する
/// （fail-safe: 誤った符号・軸の補正を適用するより安全）。この契約は
/// 意図的にこの範囲に限定している（本イシューは wasm-full 側の配線の
/// みを担い、実用上の祖先変換は並進・正スケールのみを想定するため）:
///
/// - **回転・skew を除外する理由**（codex-review P1「祖先の回転を
///   サイズ補正にも反映する」）: [`invert`] が算出する `sx`/`sy`
///   （First/Last の viewport 座標での幅・高さ比）は、祖先が軸整列
///   変換の場合に限りローカル座標での幅・高さ比と一致する（両方の
///   計測が同じ祖先変換を経ており、対角成分のみのスケールはどの軸
///   でも比を変えないため）。祖先が `rotate(90deg)` のような非軸整列
///   変換を持つ場合、viewport 座標の幅・高さは元のローカル軸と入れ替
///   わって現れる（例: ローカル幅が viewport では高さとして現れる）
///   ため、`sx`/`sy` をそのまま使うと違う軸を拡縮してしまい、変更前の
///   矩形を再現できない。
/// - **反転（負の対角成分）を除外する理由**（codex-review P1「反転した
///   祖先の下では拡縮に伴う原点のずれも補正する」）: 祖先が
///   `scaleX(-1)` の場合、viewport の矩形左端は要素のローカル右端に
///   対応するため、サイズ変更に伴う原点（要素自身のどの辺が固定される
///   か）が反転する。並進成分のみを逆行列で変換しても、このサイズ変更
///   による原点のずれを補正できず、再生直後の位置が変更前の位置を
///   再現できない（実測例: 祖先の原点を 0・子の `left` を 0・幅を
///   100px→50px にすると First.x=-100・Last.x=-50 から `tx=-50, sx=2`
///   となるが、ローカル `tx=50` を適用すると再生直後の左端が -150px
///   になり変更前の -100px を再現できない）。反転祖先まで正しく扱うには
///   矩形全体（4 頂点）をローカルへ逆変換して First/Last をローカル
///   空間で比較する必要があり、本関数のような並進のみの逆行列変換では
///   対応できない。
///
/// 祖先の回転・skew・反転に対応する必要が生じた場合は、本関数による
/// スコープ制限の再評価から始めること（`docs/policy/
/// intentional-non-adoption.md` の評価軸に準じる）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn is_axis_aligned_linear(matrix: (f64, f64, f64, f64)) -> bool {
    let (a, b, c, d) = matrix;
    b.abs() <= AXIS_ALIGNED_EPSILON && c.abs() <= AXIS_ALIGNED_EPSILON && a > 0.0 && d > 0.0
}

/// `value`（`getComputedStyle().transform` の生値）が `matrix3d(...)`
/// （3D 変形）かどうかを判定する（DOM 非依存の純粋関数、契約絞り込み、
/// codex-review 第 5 ラウンド是正、イシュー #2518。
/// `wiring::OriginalStyle::capture` から呼ばれる）。
///
/// 本クレートの FLIP 合成（`wiring::OriginalStyle::compose_transform`）は
/// 2D の `translate`/`scale` のみを外側に適用する設計であり、
/// `transform-origin` の z 成分（`wiring::resolve_origin_px` 参照）を
/// 保持・合成しない。要素自身が `matrix3d(...)`（`rotateY`/`rotateX` 等の
/// 3D 変形を含む）を持ち、かつ `transform-origin` の z 成分が非 0 の
/// 場合、再生開始時・復元時に z 成分に由来する視差（perspective の有無
/// 次第で見た目の位置が変わる）を無視することになり、変形を維持する
/// 契約に反する。**本関数による契約絞り込み**として、`matrix3d(...)` を
/// 持つ要素は `wiring::play` が再生自体を省略する
/// （`wiring::OriginalStyle::unsupported` 参照。3D 変形への対応が必要に
/// なった場合は、z 成分を保持した `translate3d` 合成への拡張から検討
/// すること）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn is_matrix3d(value: &str) -> bool {
    value.trim().starts_with("matrix3d(")
}

/// `value`（`getComputedStyle().zoom` の生値）が恒等（`zoom: 1` 相当）
/// でないかどうかを判定する（DOM 非依存の純粋関数、codex-review 第 8
/// ラウンド是正、イシュー #2518。要素自身・祖先いずれも CSS `zoom` を
/// 考慮しない契約絞り込みの判定に使う）。
///
/// CSS `zoom` は要素とその子孫の CSS ピクセル座標系そのものを拡縮する
/// （`transform: scale()` とは異なりレイアウトへも影響する）。
/// `getBoundingClientRect()` は zoom 適用後の viewport 座標を返すため、
/// 祖先に `zoom: 2` があると First/Last 間の viewport 上の差分
/// （[`invert`] の入力）は CSS 座標上の差分の 2 倍になるが、
/// `wiring::ancestor_linear_matrix`/`wiring::resolve_computed_transform`
/// はいずれも `transform` プロパティしか読まないため、この拡縮を補正
/// 計算へ一切反映できない（単位行列のまま扱ってしまい、並進補正が実際
/// の 2 倍過大になる。例: 祖先 `zoom: 2` の下で行が CSS 座標上 100px
/// 移動すると `getBoundingClientRect()` 上の差は 200px だが、補正は
/// `translate(-200px)` として画面上 400px 分適用されてしまう）。
/// **本関数による契約絞り込み**として、要素自身・祖先いずれかが `zoom`
/// に恒等以外の値を持つ場合、`wiring::play` は再生自体を省略する
/// （要素自身は `wiring::OriginalStyle::unsupported`、祖先は
/// `wiring::ancestor_linear_matrix` が `None` を返す経路、いずれも
/// fail-safe）。computed 値は `"1"`（既定）・`"normal"`（`zoom` 未対応
/// ブラウザの `getPropertyValue` フォールバック）・空文字列（プロパティ
/// 自体が未サポート）のいずれも恒等として扱う。
/// `property`（`"scale"`/`"rotate"`/`"translate"`）の computed 値
/// `value` が、そのプロパティの恒等変換（見た目に変形を与えない値）
/// と等価かどうかを判定する（DOM 非依存の純粋関数、codex-review 指摘
/// 是正、イシュー #2544）。空白区切りの各成分が恒等値（`scale` は
/// `1`、`rotate` は `0deg`/`0`、`translate` は `0px`）であれば `true`。
/// 不明な単位・パース不能な成分が 1 つでもあれば安全側（恒等ではない
/// = FLIP 省略）に倒し `false` を返す。
///
/// `wiring::has_independent_transform_property` から呼ばれる。
/// `pre-styled-ui::list_motion::enter_css` は `animation-fill-mode:
/// both` の `to` キーフレームに `scale: 1`（視覚的には恒等変換）を
/// 含むため、enter アニメーション再生済みの行の computed `scale` は
/// `"none"` ではなく `"1"` のまま残留する。この値を非恒等値として
/// 誤検知すると list の並べ替え（FLIP）が無条件で省略されてしまう
/// （FLIP 非干渉契約違反）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn is_identity_independent_transform_value(property: &str, value: &str) -> bool {
    let components: Vec<&str> = value.split_whitespace().collect();
    if components.is_empty() {
        return false;
    }
    components.iter().all(|component| match property {
        "scale" => component.parse::<f64>() == Ok(1.0),
        "rotate" => {
            let numeric = component
                .strip_suffix("deg")
                .or_else(|| component.strip_suffix("grad"))
                .or_else(|| component.strip_suffix("rad"))
                .or_else(|| component.strip_suffix("turn"))
                .unwrap_or(component);
            numeric.parse::<f64>() == Ok(0.0)
        }
        "translate" => component
            .strip_suffix("px")
            .and_then(|numeric| numeric.parse::<f64>().ok())
            .is_some_and(|numeric| numeric == 0.0),
        _ => false,
    })
}

#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn is_non_identity_zoom(value: &str) -> bool {
    let trimmed = value.trim();
    !(trimmed.is_empty() || trimmed == "normal" || trimmed == "1")
}

/// `transform-box` の computed 値が、本クレートが要素自身の
/// `resolved_origin_px`（[`wiring::OriginalStyle::capture`] が読む、
/// [`wiring::OriginalStyle::compose_transform`] の合成に使う border-box
/// 基準の原点）を正しく border-box 基準へ換算できる基準ボックスかどうか
/// を判定する（DOM 非依存の純粋関数、Bugbot 指摘対応、codex-review 第
/// 10 ラウンド、イシュー #2518。「own-transform origin ignores forced
/// box」）。
///
/// # 背景（`transform-box: border-box` を書き込むだけでは直らない理由）
///
/// 第 9 ラウンドは再生中に `transform-box: border-box !important` を
/// 強制したが、`getComputedStyle().transform-origin` の絶対長さ成分
/// （`0px` 等、パーセンテージでない値）は現在有効な `transform-box` に
/// 関わらず**そのままの数値**を返す（`0` は「その基準ボックスの左上から
/// 0」という意味であり、基準ボックスを変えても数値自体は変化しない）。
/// このため `transform-box: content-box; transform-origin: 0 0` の要素
/// で `border-box` を強制してから読んでも `resolve_origin_px` は
/// 引き続き `(0, 0)` を返し、実際の物理的な原点位置（content-box 左上、
/// border-box 左上から `border + padding` 分ずれた点）とは一致しない
/// （実測で確認済み: `border-box` 強制の有無に関わらず数値は不変）。
///
/// 正しい border-box 基準原点は「読み取った生の値 + 基準ボックスと
/// border-box の間のオフセット（border 幅・padding 幅の合計）」であり、
/// [`wiring::border_box_offset`] がこのオフセットを算出する。本関数は
/// `transform-box` の computed 値からオフセットの算出方法（
/// `TransformBoxKind`）を決定する。
///
/// `""`（未サポートブラウザのフォールバック）・`"view-box"`（既定値。
/// SVG 要素以外では border-box として解決される仕様）・`"border-box"`
/// は等価に扱う。`"fill-box"`/`"stroke-box"`/`"view-box"`（SVG 固有の
/// 基準ボックス）のうち `view-box` 以外の SVG 専用値、および未知の値は
/// `None`（契約絞り込み: [`wiring::OriginalStyle::capture`] はこの場合
/// `unsupported` へ倒し、[`wiring::play`] は再生自体を省略する。誤った
/// オフセットで合成するより安全、fail-safe）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn classify_transform_box(value: &str) -> Option<TransformBoxKind> {
    match value.trim() {
        "" | "view-box" | "border-box" => Some(TransformBoxKind::Border),
        "padding-box" => Some(TransformBoxKind::Padding),
        "content-box" => Some(TransformBoxKind::Content),
        _ => None,
    }
}

/// [`classify_transform_box`] の戻り値（`transform-box` の基準ボックス
/// 種別、border-box 換算に必要な分類のみ）。
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformBoxKind {
    /// border-box 自身（オフセットなし）。
    Border,
    /// padding-box（border 幅分だけ border-box よりオフセット）。
    Padding,
    /// content-box（border 幅 + padding 幅分だけ border-box よりオフセット）。
    Content,
}

/// `value`（`transition-property`/`-duration`/`-delay`/`-timing-function`/
/// `-behavior` いずれかの computed 値、コンマ区切りリスト）を各項目へ
/// 分割する（DOM 非依存の純粋関数、codex-review 第 7 ラウンド是正・
/// イシュー #2518。P1「抑止を transform / transform-origin に限定する」）。
///
/// 単純な `str::split(',')` は `transition-timing-function` の
/// `cubic-bezier(0.1, 0.7, 1, 0.1)` や `steps(4, end)` のような関数記法
/// 内部のコンマを誤って区切りとして扱ってしまうため、丸括弧の深さを
/// 追跡し、深さ 0 のコンマのみを区切りとする（トップレベルのみ分割）。
/// 各項目は前後の空白を取り除く。空文字列の入力は空リストを返す。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn split_transition_list(value: &str) -> Vec<String> {
    if value.trim().is_empty() {
        return Vec::new();
    }
    let mut items = Vec::new();
    let mut depth: i32 = 0;
    let mut current = String::new();
    for ch in value.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                items.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    items.push(current.trim().to_string());
    items.into_iter().filter(|item| !item.is_empty()).collect()
}

/// `element` の computed `transition-property`/`-duration`/`-delay`/
/// `-timing-function`/`-behavior` の 5 リストを受け取り、`transform`/
/// `transform-origin` **のみ**を抑止範囲に持つ新しい 5 リストを算出する
/// （DOM 非依存の純粋関数、codex-review 第 7 ラウンド是正・イシュー
/// #2518。P1「抑止を transform / transform-origin に限定する」）。
///
/// 戻り値は [`TRANSITION_LONGHANDS`](wiring::TRANSITION_LONGHANDS) の
/// 順序に対応する `[Option<String>; 5]`。`None` はそのプロパティを一切
/// 書き込まない（既存の値を触らない）ことを意味する。
///
/// # 設計（CSS の「複数リストの長さ不一致は循環展開」規則の活用）
///
/// CSS Transitions の仕様は、`transition-property` の要素数 N を
/// 遷移の本数と定め、他の longhand（duration/delay/timing-function/
/// behavior）のリストが N より短い場合は先頭から循環して割り当て、
/// 長い場合は末尾を無視する（used value の決定規則）。さらに
/// 「同じプロパティが `transition-property` に複数回現れたら最後が
/// 勝つ」規則があるため、既存のリストを**一切変更せず末尾に
/// `transform`/`transform-origin` を追加**すれば、追加分にだけ効く
/// duration/delay/timing-function/behavior を安全に指定できる
/// （既存のどの項目の値も上書きしない）。
///
/// `transition-property` の computed 値が `none`（単一項目）の場合は
/// 遷移対象が元から 0 件のため、循環展開の基準が存在しない。この場合は
/// コーディネータ指示どおり `transform, transform-origin` と対応する
/// `0s, 0s`（duration）・`0s, 0s`（delay）の 3 longhand のみを書き、
/// `transition-timing-function`/`transition-behavior` は書き込まない
/// （duration が `0s` のため timing-function は実効を持たず、
/// transition-behavior も `transform`/`transform-origin` という
/// 連続値プロパティには無関係であり、放置しても安全）。
#[cfg(any(target_arch = "wasm32", test))]
#[must_use]
fn expand_transition_lists(
    property: &str,
    duration: &str,
    delay: &str,
    timing_function: &str,
    behavior: &str,
) -> [Option<String>; 5] {
    let mut property_items = split_transition_list(property);
    if property_items.len() == 1 && property_items[0] == "none" {
        property_items.clear();
    }
    let n = property_items.len();

    if n == 0 {
        return [
            Some("transform, transform-origin".to_string()),
            Some("0s, 0s".to_string()),
            Some("0s, 0s".to_string()),
            None,
            None,
        ];
    }

    let duration_items = split_transition_list(duration);
    let delay_items = split_transition_list(delay);
    let timing_items = split_transition_list(timing_function);
    let behavior_items = split_transition_list(behavior);

    let expand = |items: &[String], default: &str| -> Vec<String> {
        if items.is_empty() {
            vec![default.to_string(); n]
        } else {
            (0..n).map(|i| items[i % items.len()].clone()).collect()
        }
    };

    let mut final_property = property_items;
    let mut final_duration = expand(&duration_items, "0s");
    let mut final_delay = expand(&delay_items, "0s");
    let mut final_timing = expand(&timing_items, "ease");
    let mut final_behavior = expand(&behavior_items, "normal");

    final_property.extend(["transform".to_string(), "transform-origin".to_string()]);
    final_duration.extend(["0s".to_string(), "0s".to_string()]);
    final_delay.extend(["0s".to_string(), "0s".to_string()]);
    final_timing.extend(["linear".to_string(), "linear".to_string()]);
    final_behavior.extend(["normal".to_string(), "normal".to_string()]);

    [
        Some(final_property.join(", ")),
        Some(final_duration.join(", ")),
        Some(final_delay.join(", ")),
        Some(final_timing.join(", ")),
        Some(final_behavior.join(", ")),
    ]
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{to_transform_css, FlipDelta, Rect, IDENTITY};
    use fandhe_animation::driver::Driver;
    use fandhe_animation::interpolate::Interpolate;
    use fandhe_animation::spring::{Spring, SpringConfig};
    use std::cell::Cell;
    use std::rc::Rc;
    use web_sys::{CssStyleDeclaration, HtmlElement};

    /// `element.get_bounding_client_rect()` から [`Rect`] を構築する
    /// （座標計測、実装計画 §3.2）。
    #[must_use]
    pub fn measure(element: &HtmlElement) -> Rect {
        let rect = element.get_bounding_client_rect();
        Rect {
            x: rect.x(),
            y: rect.y(),
            width: rect.width(),
            height: rect.height(),
        }
    }

    /// `element` の inline `transform`/`transform-origin` を一時的に
    /// クリアしてから [`measure`] し、クリア前の値へ直後に復元してから
    /// 矩形を返す（未変形の「layout 矩形」の計測、codex-review 第 4
    /// ラウンド是正・再設計、イシュー #2518）。
    ///
    /// # 用途は Last の layout 矩形計測 1 箇所に限定される
    ///
    /// 本ラウンドの再設計により、First の計測は要素自身の変形を含む
    /// 「現在の視覚矩形」（[`measure`] を直接使う）へ変わったため、本関数
    /// は `crate::layout_flip::play_after` が Last で `last_layout`
    /// （[`super::anchor_correct`] の基準矩形）を測る 1 箇所でのみ使う
    /// （クレート doc「計測・合成の再設計」節参照）。First 側や要素自身の
    /// 変形が並進のみ（サイズ変化なし）の場合はこの一時的な transform
    /// クリアは発生しない（[`super::anchor_correct`] が `(1 − S)` を係数に
    /// 持つため、サイズ変化がなければ本関数の戻り値は補正に使われない
    /// のと数学的に等価だが、呼び出し自体は無条件に行う点に注意）。
    ///
    /// クリア直後に元の値を復元するため、通常は呼び出し前後で DOM の
    /// 見た目は変化しない（計測用の一時的な reflow のみ）。**ただし**
    /// 要素自身の `transform` に CSS transition が設定・進行中の場合は
    /// 例外で、本関数の呼び出しは進行中の遷移を目標値へ確定
    /// （settle）させる副作用を持つ（codex-review 第 7 ラウンド是正、
    /// 本関数末尾の追記節参照）。`remove_property` は inline 宣言しか除去しないため、CSS
    /// クラス・外部スタイルシート由来の `transform`（例: `transform:
    /// scale(2)`）は計測に残ってしまう。`!important` 付きの `none` を
    /// inline へ強制書き込みし、由来（inline かスタイルシートか）を問わ
    /// ず実際に適用される変形を計測時のみ確実に無効化する（測定直後に
    /// 元の inline 値へ復元、[`restore_property`] 参照。`set_property` は
    /// 同じプロパティへの後続の書き込みで `!important` を含む既存の
    /// priority を丸ごと置き換えるため、[`play`] が後で行う通常優先度の
    /// `set_property` はこの一時的な `!important` を正しく上書きできる）。
    ///
    /// codex-review 第 5 ラウンド是正（イシュー #2518）: 対象要素に
    /// `transition: transform 300ms` 等が設定されていると、`none
    /// !important` の書き込み自体がその transition の対象になり、
    /// 直後の [`measure`] が変形除去途中の中間矩形を返してしまう
    /// （即時反映されない）。計測直前に [`compute_transition_
    /// suppression`]/[`apply_transition_suppression`] で
    /// `transform`/`transform-origin` の抑止計画を書き込み、測定直後に
    /// 元の transition longhand（[`TRANSITION_LONGHANDS`]、値・優先度）
    /// へ復元する。
    ///
    /// codex-review 第 6 ラウンド是正（イシュー #2518。P1「transition
    /// の個別プロパティを保存・復元する」・P1「変形の復元を確定してから
    /// transition を戻す」）: (1) 旧実装は shorthand `transition` を
    /// 保存していたが、longhand のみが inline 指定されている場合に
    /// 保存漏れ・復元時の消失が起きるため longhand ごとに保存・復元する
    /// （[`capture_transition`]/[`restore_transition`] 参照）。(2)
    /// `transform`/`transform-origin` の復元直後に [`flush_style`] で
    /// 同期スタイル計算を強制してから `transition-*` を復元する（[`flush_
    /// style`] doc 参照。これを行わないと復元後の `transform` の変化が
    /// 元の transition 設定で評価され、意図しない遷移が始まる）。(3)
    /// 抑止自体も `transition-property` には触れない
    /// （[`compute_transition_suppression`] doc 参照）。
    ///
    /// codex-review 第 7 ラウンド是正（イシュー #2518。P1「抑止を
    /// transform / transform-origin に限定する」）: 抑止範囲を
    /// `transform`/`transform-origin` のみへ絞ったことに伴い、本関数の
    /// 「呼び出し前後で DOM の見た目・進行中アニメーションの状態は変化
    /// しない」という従来の記述は**要素自身の `transform` に CSS
    /// transition が設定されている場合は成り立たない**よう改める:
    /// `transform: none !important` の書き込みは進行中の transform
    /// transition を即座に中断させ、直後の復元（duration 0s のまま）は
    /// 元の指定値（遷移の**目標値**、遷移中でも `getPropertyValue` が
    /// 返す specified value は最初から目標値である）へ即座にジャンプ
    /// させる。つまり本関数は呼び出しの副作用として「進行中の transform
    /// 遷移を目標値へ確定（settle）させる」——これは欠陥ではなく、
    /// `crate::layout_flip::play_after` がこの確定後に
    /// [`OriginalStyle::capture`] と Last 視覚矩形を取得する新しい捕捉
    /// 順（モジュール doc「計測・合成の再設計」節・[`OriginalStyle::
    /// capture`] doc 参照）が前提とする挙動である。`transform` 以外の
    /// プロパティ（`opacity` 等）の進行中の遷移は本関数の対象外であり
    /// 影響を受けない（[`compute_transition_suppression`] がスコープを
    /// `transform`/`transform-origin` のみに限定するため）。
    #[must_use]
    pub fn measure_clearing_transform(element: &HtmlElement) -> Rect {
        let style = element.style();
        let transform = read_property(&style, "transform");
        let transform_origin = read_property(&style, "transform-origin");
        let transform_box = read_property(&style, "transform-box");
        let transition = capture_transition(&style);
        let suppression = compute_transition_suppression(element);
        apply_transition_suppression(&style, &suppression);
        let _ = style.set_property_with_priority("transform", "none", "important");
        let _ = style.set_property_with_priority("transform-origin", "0 0", "important");
        // codex-review 第 9 ラウンド是正（イシュー #2518。P1「transform-box
        // の参照原点を補正に反映する」）: `transform: none` の下では
        // `transform-box` は視覚に影響しないが、[`write_transform_
        // important`] と同じ抑止の対称性を保つため計測ヘルパ側でも同じ
        // 扱い（強制・保存・復元）にする（`OriginalStyle::compose_
        // transform` doc「transform-box の参照原点」節参照）。
        let _ = style.set_property_with_priority("transform-box", "border-box", "important");
        let rect = measure(element);
        restore_property(&style, "transform", &transform);
        restore_property(&style, "transform-origin", &transform_origin);
        restore_property(&style, "transform-box", &transform_box);
        flush_style(element);
        restore_transition(&style, &transition);
        rect
    }

    /// 複数要素の layout（未変形）矩形を一括計測する（reflow を要素ごとに
    /// 繰り返さないための一括版、codex-review 第 5 ラウンド是正・
    /// codex P2・Bugbot 指摘、イシュー #2518）。
    ///
    /// [`measure_clearing_transform`] を要素数ぶんループ呼び出しすると、
    /// 各呼び出しが「書き込み（`none` 強制）→ 読み取り（計測）→ 復元」を
    /// 個別に行うため、大きな keyed list では要素ごとに reflow が発生
    /// する（`crate::layout_flip::play_after` が本来「クリアと計測を
    /// 交互に行うと reflow が N 回に増える」として避けていたのと同じ
    /// 問題が、内部実装では実は解消されていなかった）。本関数は
    /// **全要素の書き込み → 全要素の読み取り → 全要素の復元**の 3 フェーズ
    /// に分けることで、要素数に関わらず reflow を実質 1 回（書き込み後・
    /// 最初の読み取り時の 1 回）に抑える。`crate::layout_flip::play_after`
    /// はこの関数を呼ぶだけになり、行ごとの手書きループを持たない。
    ///
    /// codex-review 第 6 ラウンド是正（イシュー #2518）: transition の
    /// 保存・抑止・復元は [`measure_clearing_transform`] と同じ理由・
    /// 同じ手段（[`capture_transition`]/[`compute_transition_
    /// suppression`]/[`restore_transition`]/[`flush_style`]）を使う。
    /// フェーズ 3 のフラッシュは全要素の `transform`/`transform-origin`
    /// を復元した**後にまとめて 1 回**行う（`offset_width()` の読み取り
    /// は要素単位ではなく文書全体の保留中スタイル・レイアウト計算を
    /// 強制するため、全要素の復元後に 1 回呼べば全要素分をまとめて確定
    /// できる。要素数に応じてフラッシュ回数を増やす必要はない）。
    ///
    /// codex-review 第 7 ラウンド是正（イシュー #2518。P1「抑止を
    /// transform / transform-origin に限定する」）: 抑止計画
    /// （[`compute_transition_suppression`]）は `getComputedStyle` を
    /// 呼ぶため、書き込み（`transform: none !important` 等）の**後**に
    /// 要素ごと呼ぶと、書き込み済みの要素分の保留中スタイル変更を都度
    /// 確定させる reflow が要素数ぶん発生し、本関数が守る「reflow は
    /// 実質 1 回」契約を壊す。そのため抑止計画は**フェーズ 0（読み取り
    /// 専用、書き込み開始前）**で全要素分まとめて算出し、フェーズ 1
    /// （書き込み）では算出済みの計画を渡すだけにする（[`measure`] を
    /// 全要素まとめて呼ぶフェーズ 2 と同じ「読み取りをまとめる」原則）。
    #[must_use]
    pub fn measure_layout_batch(elements: &[HtmlElement]) -> Vec<Rect> {
        struct Saved {
            transform: Option<(String, String)>,
            transform_origin: Option<(String, String)>,
            transform_box: Option<(String, String)>,
            transition: [Option<(String, String)>; 5],
        }

        // フェーズ 0（読み取り専用）: 全要素の transition 抑止計画を
        // 書き込み開始前にまとめて算出する（本関数 doc 参照）。
        let suppression_plans: Vec<[Option<String>; 5]> = elements
            .iter()
            .map(compute_transition_suppression)
            .collect();

        // フェーズ 1（書き込み）: 全要素の元の値を保存し、`none
        // !important` を一括で書き込む。
        let saved: Vec<Saved> = elements
            .iter()
            .zip(suppression_plans.iter())
            .map(|(element, plan)| {
                let style = element.style();
                let transform = read_property(&style, "transform");
                let transform_origin = read_property(&style, "transform-origin");
                let transform_box = read_property(&style, "transform-box");
                let transition = capture_transition(&style);
                apply_transition_suppression(&style, plan);
                let _ = style.set_property_with_priority("transform", "none", "important");
                let _ = style.set_property_with_priority("transform-origin", "0 0", "important");
                // codex-review 第 9 ラウンド是正（イシュー #2518。P1
                // 「transform-box の参照原点を補正に反映する」）:
                // `measure_clearing_transform` doc 参照。
                let _ =
                    style.set_property_with_priority("transform-box", "border-box", "important");
                Saved {
                    transform,
                    transform_origin,
                    transform_box,
                    transition,
                }
            })
            .collect();

        // フェーズ 2（読み取り）: 全要素をまとめて計測する。
        let rects: Vec<Rect> = elements.iter().map(measure).collect();

        // フェーズ 3（復元）: 全要素の transform/transform-origin/
        // transform-box をまとめて元へ戻し、1 回だけフラッシュしてから、
        // 全要素の transition-* をまとめて元へ戻す。
        for (element, entry) in elements.iter().zip(saved.iter()) {
            let style = element.style();
            restore_property(&style, "transform", &entry.transform);
            restore_property(&style, "transform-origin", &entry.transform_origin);
            restore_property(&style, "transform-box", &entry.transform_box);
        }
        if let Some(first) = elements.first() {
            flush_style(first);
        }
        for (element, entry) in elements.iter().zip(saved.iter()) {
            restore_transition(&element.style(), &entry.transition);
        }

        rects
    }

    /// `style` の `property` を読み取る。未設定（空文字列）は `None`。
    fn read_property(style: &CssStyleDeclaration, property: &str) -> Option<(String, String)> {
        let value = style.get_property_value(property).ok()?;
        if value.is_empty() {
            return None;
        }
        let priority = style.get_property_priority(property);
        Some((value, priority))
    }

    /// `original`（[`read_property`] の結果）へ `style` の `property` を
    /// 戻す。`None`（元々未設定）なら `remove_property` する。
    fn restore_property(
        style: &CssStyleDeclaration,
        property: &str,
        original: &Option<(String, String)>,
    ) {
        match original {
            Some((value, priority)) => {
                let _ = style.set_property_with_priority(property, value, priority);
            }
            None => {
                let _ = style.remove_property(property);
            }
        }
    }

    /// `transition` の保存・復元対象とする longhand プロパティ 5 種
    /// （codex-review 第 6 ラウンド是正、イシュー #2518。P1「transition
    /// の個別プロパティを保存・復元する」）。
    ///
    /// shorthand `transition` を [`read_property`] で保存する旧実装は、
    /// 利用者が inline で longhand のみ（例: `transition-duration:
    /// 300ms` のみで `transition` 自体は明示していない）を指定していた
    /// 場合に問題があった: `CSSStyleDeclaration.getPropertyValue
    /// ("transition")` は shorthand が inline で明示されていない限り
    /// 空文字列を返す仕様のため、[`read_property`] は保存値を `None`
    /// と誤判定し、[`restore_property`] が復元時に `transition`
    /// 全体を `remove_property` してしまい、元の longhand 宣言が
    /// 恒久的に失われる（元のスタイルを復元する契約に反する）。longhand
    /// ごとに個別へ保存・復元することでこの問題を避ける（shorthand
    /// `transition` 自体は保存・復元の対象にしない）。
    const TRANSITION_LONGHANDS: [&str; 5] = [
        "transition-property",
        "transition-duration",
        "transition-delay",
        "transition-timing-function",
        "transition-behavior",
    ];

    /// `style` から [`TRANSITION_LONGHANDS`] の現在値・優先度を保存する。
    fn capture_transition(style: &CssStyleDeclaration) -> [Option<(String, String)>; 5] {
        TRANSITION_LONGHANDS.map(|property| read_property(style, property))
    }

    /// [`capture_transition`] が保存した値へ [`TRANSITION_LONGHANDS`]
    /// を戻す。
    fn restore_transition(style: &CssStyleDeclaration, saved: &[Option<(String, String)>; 5]) {
        for (property, value) in TRANSITION_LONGHANDS.iter().zip(saved.iter()) {
            restore_property(style, property, value);
        }
    }

    /// `element` の computed `transition-property`/`-duration`/`-delay`/
    /// `-timing-function`/`-behavior` を読み取り、
    /// [`super::expand_transition_lists`] へ渡して抑止計画（5 longhand の
    /// 書き込み値、`None` は書き込まない）を算出する（codex-review 第 7
    /// ラウンド是正、イシュー #2518。P1「抑止を transform /
    /// transform-origin に限定する」）。
    ///
    /// # なぜ inline ではなく computed を読むか
    ///
    /// スコープ化の前提は「実際に効いている `transition-property` 等の
    /// リスト」であり、それは inline 宣言だけでなくスタイルシート由来
    /// （例: `.card { transition: opacity 300ms, transform 300ms; }`）も
    /// 含む。[`capture_transition`]（[`restore_transition`] 用、inline
    /// のみを保存・復元する）とは目的が異なる別経路であり、混同しない
    /// （両者の対象・読み取り元は意図的に異なる）。
    ///
    /// # 呼び出しは 1 回・結果を再利用する（tick ごとの再計算をしない）
    ///
    /// 本関数が返す計画は呼び出し元（[`measure_clearing_transform`]・
    /// [`measure_layout_batch`]・[`play`]）が 1 回だけ算出して保持し、
    /// 以降の書き込み（[`play`] の rAF tick 含む）では
    /// [`apply_transition_suppression`] へ same 値を渡して再利用する。
    /// もし tick ごとに本関数を再度呼んで結果を末尾へ追記し続けると、
    /// 前回書き込んだ `"..., transform, transform-origin"` を今回の
    /// computed 値として読み込んでしまい、`transform`/`transform-origin`
    /// が呼び出し回数分重複して肥大化し続ける（無限成長・毎フレーム
    /// 強制スタイル再計算という 2 重の問題になる）。
    #[must_use]
    fn compute_transition_suppression(element: &HtmlElement) -> [Option<String>; 5] {
        super::expand_transition_lists(
            &read_computed_property(element, "transition-property"),
            &read_computed_property(element, "transition-duration"),
            &read_computed_property(element, "transition-delay"),
            &read_computed_property(element, "transition-timing-function"),
            &read_computed_property(element, "transition-behavior"),
        )
    }

    /// `element` の `getComputedStyle()` から `property` の値を読む。
    /// `window`/`get_computed_style` の失敗は空文字列（「リストなし」
    /// として [`super::split_transition_list`] が空リストへ畳む）。
    fn read_computed_property(element: &HtmlElement, property: &str) -> String {
        web_sys::window()
            .and_then(|window| window.get_computed_style(element).ok().flatten())
            .and_then(|computed| computed.get_property_value(property).ok())
            .unwrap_or_default()
    }

    /// [`compute_transition_suppression`] が算出した計画を `style` へ
    /// 書き込む（`None` の longhand は一切触らない、codex-review 第 7
    /// ラウンド是正、イシュー #2518）。
    fn apply_transition_suppression(style: &CssStyleDeclaration, plan: &[Option<String>; 5]) {
        for (property, value) in TRANSITION_LONGHANDS.iter().zip(plan.iter()) {
            if let Some(value) = value {
                let _ = style.set_property_with_priority(property, value, "important");
            }
        }
    }

    /// 直近のスタイル変更をブラウザに確定させる同期スタイル計算を強制
    /// する（codex-review 第 6 ラウンド是正、イシュー #2518。P1「変形の
    /// 復元を確定してから transition を戻す」）。
    ///
    /// `transform`/`transform-origin` の復元と `transition-*` longhand
    /// の復元を同一のフラッシュ（マイクロタスク内の連続書き込み）で
    /// 行うと、ブラウザはスタイル再計算を遅延させ、`transform` の変化を
    /// 「復元後の `transition-*` 設定」で評価してしまい、意図しない新規
    /// の遷移が始まる（例: `none → scale(2)` の変化に元の `transition:
    /// transform 300ms` が適用され、復元直後の表示が元の見た目へ戻らな
    /// い）。`offset_width()` の読み取りはレイアウト・スタイル計算を
    /// 強制する副作用を持つため、これを呼び出し元（[`OriginalStyle::
    /// restore`]・[`measure_clearing_transform`]・
    /// [`measure_layout_batch`]）が `transform`/`transform-origin` の
    /// 復元直後・`transition-*` の復元直前に呼ぶことで、`transform` の
    /// 復元が `transition-duration: 0s !important` が有効なうちに確定
    /// してから `transition-*` を元の値へ戻せる。
    fn flush_style(element: &HtmlElement) {
        let _ = element.offset_width();
    }

    /// `value`（inline `transform` の読み取り値）が恒等変換（変形なし）
    /// と等価かどうかを判定する。`""` は [`read_property`] が既に `None`
    /// へ畳んでいるためここでは扱わず、CSS の恒等キーワード `none` のみを
    /// 見る（前後空白は許容）。
    ///
    /// レビュー指摘対応（イシュー #2518 フォローアップ）: `transform:
    /// none` を他の `transform` 値と同列に扱い FLIP 補正へ文字列連結
    /// すると `"translate(...) scale(...) none"` という無効な CSS 値に
    /// なり `set_property` が黙って無視するため、`none` は「元の
    /// transform が存在しない」場合と同じ扱いにする必要がある。
    fn is_identity_transform(value: &str) -> bool {
        value.trim() == "none"
    }

    /// `computed`（`getComputedStyle()` の結果）が独立した CSS 変形
    /// プロパティ（`scale`/`rotate`/`translate`、`transform` とは別の
    /// CSS Transforms Level 2 プロパティ）のいずれかを持つかどうかを
    /// 判定する（契約絞り込み、codex-review 第 5 ラウンド是正、イシュー
    /// #2518）。
    ///
    /// これらのプロパティは `transform` プロパティとは独立に視覚的な
    /// 変形を作用させるが、[`resolve_computed_transform`]（要素自身、
    /// [`OriginalStyle::compose_transform`] の合成元）・
    /// [`ancestor_linear_matrix`]（祖先、[`to_local_delta`] の逆行列元）
    /// のいずれも `transform` プロパティしか読まないため、これらの
    /// プロパティが値を持つ要素・祖先の下では実際に適用されている変形を
    /// 正しく捕捉できない（祖先に `scale: 2` があると恒等変換と誤判定
    /// し、viewport の移動差をそのままローカルへ適用して補正量が倍に
    /// なる。要素自身に `scale: 2` があると元の変形が合成されず表示が
    /// 跳ぶ）。**本関数による契約絞り込み**として、これらのプロパティを
    /// 検出した場合は要素自身/祖先を問わず [`play`] が再生自体を省略
    /// する（要素自身は [`OriginalStyle::unsupported`]、祖先は
    /// [`ancestor_linear_matrix`] が `None` を返す経路、いずれも
    /// fail-safe）。値が空文字列（ブラウザがそのプロパティ自体を
    /// サポートしない、`getPropertyValue` が未知のプロパティ名に対し
    /// 空文字列を返す仕様）・`"none"`、または恒等変形と等価な値
    /// （[`is_identity_independent_transform_value`] doc 参照）の場合は
    /// 「値を持たない」と判定する。
    ///
    /// codex-review 指摘是正（イシュー #2544）: `pre-styled-ui::
    /// list_motion::enter_css` は `animation-fill-mode: both` の `to`
    /// キーフレームに `scale: 1`（視覚的には恒等変換）を含むため、enter
    /// アニメーション再生済みの行の computed `scale` は `"none"` では
    /// なく `"1"` のまま残留する。旧実装はこれを非恒等値として誤検知し、
    /// list の並べ替え（FLIP）を無条件で省略してしまっていた
    /// （FLIP 非干渉契約違反）。
    fn has_independent_transform_property(computed: &CssStyleDeclaration) -> bool {
        ["scale", "rotate", "translate"].iter().any(|property| {
            let value = computed.get_property_value(property).unwrap_or_default();
            !value.is_empty()
                && value != "none"
                && !super::is_identity_independent_transform_value(property, &value)
        })
    }

    /// `getComputedStyle(element).transform-origin`（ブラウザが百分率・
    /// キーワード・要素サイズを解決した px 座標）を `(x, y, z)` として
    /// 返す。
    ///
    /// レビュー指摘対応（イシュー #2518 フォローアップ）: 元の `transform`
    /// が `rotate`/`scale` 等を含む場合、その基準点（既定は中心
    /// `50% 50%`）を無視して `transform-origin` を一律 `0 0` にすると、
    /// FLIP 再生中だけ元の変形が左上原点で適用され、収束時に中心原点へ
    /// 戻って見た目が跳ぶ。computed style は百分率・要素サイズ解決済みの
    /// 絶対 px 値を返すため、明示 inline 指定の有無を問わず実際に
    /// 使われている基準点をそのまま取得できる（[`OriginalStyle::
    /// compose_transform`] 参照）。
    ///
    /// z 成分（3 番目の `"Lpx"`）は [`OriginalStyle::compose_transform`]
    /// の合成には使わない（2D 合成のみサポート）が、[`OriginalStyle::
    /// capture`] が非 0 かどうかを判定して契約絞り込み（[`is_matrix3d`]
    /// doc 参照）に使う。z 成分を持たない `"Npx Mpx"` 形式の場合は
    /// `0.0` として扱う。
    ///
    /// `window`/`get_computed_style` の取得失敗、または `x`/`y` 成分が
    /// `"Npx"` 形式でパースできない場合は `None`（fail-safe: 呼び出し元は
    /// `None` の場合 `0 0` 基準へフォールバックする近似に留める）。
    fn resolve_origin_px(element: &HtmlElement) -> Option<(f64, f64, f64)> {
        let window = web_sys::window()?;
        let computed = window.get_computed_style(element).ok()??;
        let origin = computed.get_property_value("transform-origin").ok()?;
        let mut parts = origin.split_whitespace();
        let x = parts.next()?.strip_suffix("px")?.parse::<f64>().ok()?;
        let y = parts.next()?.strip_suffix("px")?.parse::<f64>().ok()?;
        let z = parts
            .next()
            .and_then(|p| p.strip_suffix("px"))
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(0.0);
        Some((x, y, z))
    }

    /// [`resolve_origin_px`] の z 成分がゼロとみなせる許容誤差
    /// （ブラウザの浮動小数点誤差を吸収する、`super::AXIS_ALIGNED_EPSILON`
    /// と同じ考え方）。
    const ORIGIN_Z_EPSILON: f64 = 1e-3;

    /// `kind`（[`super::TransformBoxKind`]、`transform-box` の基準
    /// ボックス種別）に応じて、その基準ボックスの左上が `computed`
    /// （`getComputedStyle` の結果）の要素の border-box 左上からどれだけ
    /// オフセットしているか（px, x/y）を返す（Bugbot 指摘対応、
    /// codex-review 第 10 ラウンド、イシュー #2518。`super::classify_
    /// transform_box` doc「`transform-box: border-box` を書き込むだけ
    /// では直らない理由」参照）。
    ///
    /// `[`resolve_origin_px`]` が読む `transform-origin` の絶対長さ成分は
    /// 現在有効な `transform-box` からのオフセットのまま（基準ボックスを
    /// 変えても数値は変わらない）返るため、`OriginalStyle::capture` は
    /// この関数が返すオフセットを `resolve_origin_px` の結果へ**加算**
    /// することで border-box 基準へ換算する（`compose_transform` の
    /// `translate(ox, oy) <O> translate(-ox, -oy)` は border-box 基準の
    /// 座標系を前提とするため）。
    ///
    /// `border-*-width`/`padding-*` の computed 値が `"Npx"` 形式で
    /// パースできない場合は `0.0` として扱う（fail-safe: 該当成分の
    /// オフセットなしとして扱う近似に留める。border/padding が実際には
    /// 存在するのに 0 扱いになるのは、要素に `box-sizing`/単位系の
    /// 非典型的な組み合わせがある稀なケースに限られる）。
    fn border_box_offset(
        computed: &CssStyleDeclaration,
        kind: super::TransformBoxKind,
    ) -> (f64, f64) {
        let px = |property: &str| -> f64 {
            computed
                .get_property_value(property)
                .ok()
                .and_then(|value| value.strip_suffix("px").map(str::to_string))
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(0.0)
        };
        match kind {
            super::TransformBoxKind::Border => (0.0, 0.0),
            super::TransformBoxKind::Padding => (px("border-left-width"), px("border-top-width")),
            super::TransformBoxKind::Content => (
                px("border-left-width") + px("padding-left"),
                px("border-top-width") + px("padding-top"),
            ),
        }
    }

    /// `getComputedStyle(element).transform` を用いて、要素が実際に
    /// 持っている元の変形を取得する（レビュー指摘対応、イシュー #2518
    /// さらなるフォローアップ）。
    ///
    /// [`read_property`] による inline `style.transform` の読み取りだけ
    /// では、CSS クラス・外部スタイルシートで `transform` を指定している
    /// 要素（inline には何も無い）を「元の変形なし」と誤判定し、
    /// [`OriginalStyle::compose_transform`] がその変形を合成せずに FLIP
    /// 補正だけを書き込んでしまう（FLIP 再生中だけ見た目が変わり収束時に
    /// 跳ぶ）。computed style は算出済みの `matrix(...)`/`matrix3d(...)`
    /// （またはキーワード `none`）を返し、由来（inline かスタイルシート
    /// か）を問わず実際に適用されている変形をそのまま表すため、これを
    /// [`OriginalStyle::compose_transform`] の合成元として使う
    /// （inline 値は [`OriginalStyle::restore`] 専用に残し、両者の用途を
    /// 分離する）。
    ///
    /// `window`/`get_computed_style` の取得失敗、値が空文字列、または
    /// [`is_identity_transform`] が恒等変換と判定する場合は `None`
    /// （fail-safe: 呼び出し元は「元の変形なし」として扱う）。
    fn resolve_computed_transform(element: &HtmlElement) -> Option<String> {
        let window = web_sys::window()?;
        let computed = window.get_computed_style(element).ok()??;
        let value = computed.get_property_value("transform").ok()?;
        if value.is_empty() || is_identity_transform(&value) {
            return None;
        }
        Some(value)
    }

    /// FLIP 適用前に要素が持っていた inline `transform`/`transform-origin`
    /// の値と priority を保持する（レビュー指摘対応、イシュー #2518
    /// フォローアップ）。
    ///
    /// [`play`] は FLIP の補正 transform をこの元の値へ**合成**して適用し
    /// （利用者が別の目的で指定していた `transform` を上書きで消さない
    /// ため）、収束・強制停止・`prefers-reduced-motion` のいずれの終了
    /// 経路でも `remove_property` ではなくこの値へ**復元**する（値を
    /// 持たない、すなわち元々未設定だった場合のみ `remove_property` と
    /// 等価になる）。
    #[derive(Debug, Clone, Default)]
    pub struct OriginalStyle {
        /// [`restore`](Self::restore) 専用: 復元は DOM が元々持っていた
        /// inline 宣言（未設定なら `remove_property`）へ戻す必要があり、
        /// スタイルシート由来の computed 値を inline へ書き込んでは
        /// いけない（`restore` 後に inline 宣言が新規に生えてしまう）。
        transform: Option<(String, String)>,
        transform_origin: Option<(String, String)>,
        /// [`restore`](Self::restore) 専用: 元の inline `transform-box`
        /// （codex-review 第 9 ラウンド是正、イシュー #2518。P1
        /// 「transform-box の参照原点を補正に反映する」）。[`write_
        /// transform_important`] が再生中 `transform-box: border-box
        /// !important` を強制する（[`Self::compose_transform`] doc
        /// 「transform-box の参照原点」節参照）ため、収束・強制停止時に
        /// この元の値へ戻す必要がある。
        transform_box: Option<(String, String)>,
        /// [`restore`](Self::restore) 専用: 元の transition longhand 5 種
        /// （[`TRANSITION_LONGHANDS`]、値・優先度）。codex-review 第 5
        /// ラウンド是正（イシュー #2518）: [`play`] は計測・再生中に
        /// transition を抑止する（[`compute_transition_suppression`]
        /// doc 参照）ため、
        /// 収束・強制停止時にこの元の値へ戻す必要がある。第 6 ラウンド
        /// 是正: shorthand `transition` ではなく longhand ごとに保存する
        /// （[`capture_transition`] doc「shorthand を保存する旧実装の
        /// 問題」参照）。
        transition: [Option<(String, String)>; 5],
        /// [`compose_transform`](Self::compose_transform)/
        /// [`origin_css`](Self::origin_css) 専用: inline・スタイルシート
        /// いずれの由来でも実際に適用されている変形を
        /// [`resolve_computed_transform`] で解決した値（`matrix(...)`
        /// 等、恒等変換なら `None`）。レビュー指摘対応（イシュー #2518
        /// さらなるフォローアップ）: `transform`（inline のみ）を合成元に
        /// 使うと、CSS クラス由来の `transform` を持つ要素で合成元が
        /// `None` になり FLIP 再生中に上書きされてしまうため、合成専用に
        /// 分離する。
        computed_transform: Option<String>,
        /// 元の変形が恒等変換でない場合に限り [`capture`]
        /// （[`Self::capture`]）が `getComputedStyle` で解決しておく
        /// 実際の基準点（[`resolve_origin_px`] 参照、`(x, y)` のみ。z 成分
        /// は合成に使わないため保持しない）。元の変形が未設定/恒等変換
        /// なら不要なため取得しない。
        resolved_origin_px: Option<(f64, f64)>,
        /// [`play`] が再生を省略すべきかどうか（契約絞り込み、
        /// codex-review 第 5 ラウンド是正、イシュー #2518）。要素自身が
        /// 3D 変形（[`is_matrix3d`]）を持つ、`transform-origin` の z 成分
        /// が非 0（[`resolve_origin_px`]）、または独立した CSS 変形
        /// プロパティ（`scale`/`rotate`/`translate`、
        /// [`has_independent_transform_property`]）を持つ場合に `true`。
        /// これらは本クレートの 2D 専用の計測・合成契約の対応範囲外
        /// であり、[`play`] は即座に元のスタイルへ復元した収束済み
        /// ハンドルを返す（fail-safe、誤った合成を適用するより安全）。
        unsupported: bool,
    }

    impl OriginalStyle {
        /// `element` の現在の inline `transform`/`transform-origin`/
        /// `transition`（[`restore`](Self::restore) 用）と、computed
        /// `transform`（[`compose_transform`](Self::compose_transform)
        /// 用、スタイルシート由来も含む）を読み取り、[`Self::unsupported`]
        /// を判定する。呼び出しは FLIP がまだ何も書き込んでいない時点
        /// （進行中の旧 FLIP を止めて復元した直後）で行うこと。
        ///
        /// # 呼び出し順（codex-review 第 7 ラウンド是正、イシュー #2518。
        /// P1「計測で確定した変形と FLIP の収束先を一致させる」）
        ///
        /// **Last の layout 矩形計測（[`measure_layout_batch`]/
        /// [`measure_clearing_transform`]）の後**に呼ぶこと。これらの
        /// 関数は要素自身の `transform` を一時的に無効化・復元する過程
        /// で、進行中の CSS transform transition を目標値へ確定
        /// （settle）させる副作用を持つ（各関数 doc 参照）。本メソッドを
        /// それより前に呼ぶと、[`Self::computed_transform`] が settle
        /// 前の中間値のまま保存され、[`play`] が収束させる先
        /// （[`Self::restore`] が書き込む値）と実際にレイアウト計測が
        /// 確定させた目標値が食い違い、収束直後に見た目が跳ぶ
        /// （`crate::layout_flip::play_after` の新しい 4 パスの順序
        /// 「停止 → 対象収集 → layout 矩形一括計測（settle） → 本メソッド
        /// と Last 視覚矩形」参照。Last の**視覚**矩形（[`measure`]）も
        /// 同じ理由で本メソッドと同じタイミング〔layout 矩形計測の後〕
        /// で取得し、`computed_transform` と整合させる）。
        ///
        /// # `resolved_origin_px` は border-box 基準へ換算してから保持する
        /// （Bugbot 指摘対応、codex-review 第 10 ラウンド、イシュー
        /// #2518。「own-transform origin ignores forced box」）
        ///
        /// [`play`] は再生中 `transform-box: border-box !important` を
        /// 強制する（[`write_transform_important`] doc「transform-box
        /// の参照原点」節参照、codex-review 第 9 ラウンド是正）ため、
        /// [`compose_transform`](Self::compose_transform) の `translate
        /// (ox, oy) <O> translate(-ox, -oy)` は `ox`/`oy` が border-box
        /// 基準の座標であることを前提とする。**`transform-box: border-box
        /// !important` を書き込んでから `resolve_origin_px` を読み直す
        /// だけでは直らない**: `getComputedStyle().transform-origin` の
        /// 絶対長さ成分（`0px` 等）は現在有効な基準ボックスに関わらず
        /// そのままの数値を返すため（実測で確認済み、`super::classify_
        /// transform_box` doc「`transform-box: border-box` を書き込む
        /// だけでは直らない理由」参照）、要素自身が `transform-box:
        /// content-box` を持つ場合は border-box を強制しても読み取り値は
        /// 変化しない。本メソッドは `resolve_origin_px` が返す生の値へ、
        /// `transform-box` の基準ボックスと border-box の差分
        /// （[`super::classify_transform_box`]/[`border_box_offset`]、
        /// border 幅・padding 幅の合計）を**加算**して border-box 基準へ
        /// 換算する。`transform-box` が `fill-box`/`stroke-box` 等
        /// 未対応の SVG 専用基準ボックスの場合は換算できないため
        /// [`Self::unsupported`] へ倒す（fail-safe、誤ったオフセットで
        /// 合成するより安全）。
        #[must_use]
        pub fn capture(element: &HtmlElement) -> Self {
            let style = element.style();
            let transform = read_property(&style, "transform");
            let transform_origin = read_property(&style, "transform-origin");
            let transform_box = read_property(&style, "transform-box");
            let transition = capture_transition(&style);
            let computed_transform = resolve_computed_transform(element);
            let mut unsupported = false;
            if let Some(value) = &computed_transform {
                if super::is_matrix3d(value) {
                    unsupported = true;
                }
            }
            // `transform-box`/独立変形プロパティ/`zoom` の判定はいずれも
            // 同一の computed style を参照するため 1 回だけ取得する。
            let computed_style = web_sys::window()
                .and_then(|window| window.get_computed_style(element).ok().flatten());
            let resolved_origin = if computed_transform.is_some() {
                resolve_origin_px(element).and_then(|(x, y, z)| {
                    let box_value = computed_style
                        .as_ref()
                        .and_then(|computed| computed.get_property_value("transform-box").ok())
                        .unwrap_or_default();
                    match super::classify_transform_box(&box_value) {
                        Some(kind) => {
                            let (dx, dy) = computed_style
                                .as_ref()
                                .map(|computed| border_box_offset(computed, kind))
                                .unwrap_or((0.0, 0.0));
                            Some((x + dx, y + dy, z))
                        }
                        None => {
                            unsupported = true;
                            None
                        }
                    }
                })
            } else {
                None
            };
            if let Some((_, _, z)) = resolved_origin {
                if z.abs() > ORIGIN_Z_EPSILON {
                    unsupported = true;
                }
            }
            if let Some(computed) = &computed_style {
                if has_independent_transform_property(computed) {
                    unsupported = true;
                }
                // 契約絞り込み（codex-review 第 8 ラウンド是正、イシュー
                // #2518）: 要素自身が CSS `zoom` に恒等以外の値を持つ
                // 場合の非対応（`super::is_non_identity_zoom` doc 参照）。
                let zoom = computed.get_property_value("zoom").unwrap_or_default();
                if super::is_non_identity_zoom(&zoom) {
                    unsupported = true;
                }
            }
            let resolved_origin_px = resolved_origin.map(|(x, y, _)| (x, y));
            Self {
                transform,
                transform_origin,
                transform_box,
                transition,
                computed_transform,
                resolved_origin_px,
                unsupported,
            }
        }

        /// `element` の `transform`/`transform-origin`/`transform-box`/
        /// transition longhand 5 種をこの元の値へ戻す（元々未設定だった
        /// 場合は `remove_property`）。
        ///
        /// `transform`/`transform-origin`/`transform-box` を先に戻し、
        /// [`flush_style`]
        /// で同期スタイル計算を強制してから `transition-*` を最後に戻す
        /// （codex-review 第 5〜6 ラウンド是正、イシュー #2518）:
        /// [`play`] は再生中 `transform`/`transform-origin` に絞った抑止計画を
        /// 適用している（[`compute_transition_suppression`] doc
        /// 参照）ため、この順序であれば `transform`/`transform-origin`
        /// の復元は即時に確定してから `transition-*` を元へ戻すことに
        /// なり、不要なアニメーションは発生しない（第 6 ラウンド是正:
        /// `flush_style` を挟まずに連続して戻すと、ブラウザが
        /// `transform` の変化を「復元後の transition-* 設定」で評価し
        /// 意図しない遷移を開始してしまう、[`flush_style`] doc 参照）。
        pub fn restore(&self, element: &HtmlElement) {
            let style = element.style();
            restore_property(&style, "transform", &self.transform);
            restore_property(&style, "transform-origin", &self.transform_origin);
            restore_property(&style, "transform-box", &self.transform_box);
            flush_style(element);
            restore_transition(&style, &self.transition);
        }

        /// FLIP 補正の `transform-origin` に使う値を返す。
        ///
        /// 合成後の `transform-origin` は常に `"0 0"` を返す
        /// （codex-review 第 4 ラウンド是正、イシュー #2518）: 新設計では
        /// 要素自身の変形 `O` の基準点は [`Self::compose_transform`] が
        /// `translate(ox, oy) <O> translate(-ox, -oy)` として明示的に
        /// 埋め込むため、CSS の `transform-origin` プロパティ自体は FLIP・
        /// 合成後の変形全体に対して常に `0 0`（左上）で固定してよい
        /// （旧実装は `O` の実際の基準点をそのまま `transform-origin` へ
        /// 転用していたが、これは新しい合成方式とは独立した別の仕組みで
        /// あり、両方を同時に適用すると基準点が二重に効いてしまう）。
        fn origin_css(&self) -> &'static str {
            "0 0"
        }

        /// # transform-box の参照原点（codex-review 第 9 ラウンド是正、
        /// イシュー #2518。P1「transform-box の参照原点を補正に反映
        /// する」）
        ///
        /// [`Self::origin_css`] が返す `"0 0"` は「要素の基準ボックス
        /// （`transform-box`）の左上」を意味するが、`transform-box` の
        /// 既定は `view-box`（非 SVG 要素では概ね `border-box` に解決
        /// される）であり、要素自身が `transform-box: content-box` 等を
        /// 指定していると `"0 0"` は border-box 左上ではなく content-box
        /// 左上（border + padding 分だけ内側）を指すようになる。
        /// [`super::anchor_correct`]/[`super::invert`] は
        /// `getBoundingClientRect()`（常に border-box 基準）の差分を
        /// 前提に並進補正を算出するため、`transform-box` が border-box
        /// でない要素では FLIP 再生中の視覚矩形が border + padding 分
        /// ずれてしまう（収束時点では transform 自体が恒等へ戻るため
        /// 気づきにくいが、再生開始直後の Invert 補正で顕在化する）。
        /// [`write_transform_important`] は本メソッドの呼び出しと対に
        /// `transform-box: border-box !important` を常に強制する
        /// （[`OriginalStyle::transform_box`] が元の値を保持し、
        /// [`Self::restore`] で復元する）ことで、`"0 0"` が常に
        /// border-box 左上を指すようにし、この不整合を構造的に防ぐ。
        ///
        /// FLIP 補正 `delta`（[`super::anchor_correct`] 適用済みの
        /// viewport 座標を [`to_local_delta`] でローカル化した
        /// もの）へ元の変形（[`Self::computed_transform`]）を合成する
        /// （codex-review 第 4 ラウンド是正・再設計、イシュー #2518）。
        ///
        /// 元の変形が未設定/恒等変換（[`capture`](Self::capture) が
        /// [`resolve_computed_transform`] から `None` を得た場合）は FLIP
        /// 補正のみを返す（`none` を他の値と同列に連結すると `"...
        /// none"` という無効な CSS 値になり `set_property` が反映
        /// されないため）。
        ///
        /// # 合成（`transform-origin: 0 0` 前提、[`Self::origin_css`] 参照）
        ///
        /// `"<F> translate(ox, oy) <O> translate(-ox, -oy)"`（`F` が
        /// 最外側）と連結する。CSS の `transform` 関数リストは右から左に
        /// 適用されるため、まず `translate(-ox,-oy)` で `O` の基準点を
        /// 原点へ移し、`O`（[`Self::resolve_computed_transform`] が読んだ
        /// 生の行列。内部を解析せず不透明な文字列として扱う）を適用し、
        /// `translate(ox,oy)` で基準点を戻す——この 3 つの合成
        /// （`translate(ox,oy) <O> translate(-ox,-oy)`）は「`O` を実際の
        /// 基準点 `(ox, oy)` で適用した場合と同じ効果」を再現する。その
        /// 結果へ `F`（FLIP の並進・スケール、`transform-origin: 0 0`）が
        /// 外側から適用される。`F` の並進成分 `tx`/`ty` には
        /// [`super::anchor_correct`] が `O` の基準点のずれを既に折り込ん
        /// でいるため、本メソッドは `delta` をそのまま使うだけでよく
        /// `O` の線形部分（回転・skew）や並進成分がどのような値であって
        /// も正しく合成できる（導出は [`super::anchor_correct`] の
        /// rustdoc を正とする）。
        fn compose_transform(&self, delta: super::FlipDelta) -> String {
            let Some(value) = &self.computed_transform else {
                return to_transform_css(delta);
            };
            let (ox, oy) = self.resolved_origin_px.unwrap_or((0.0, 0.0));
            format!(
                "{} translate({ox}px, {oy}px) {value} translate({}px, {}px)",
                to_transform_css(delta),
                -ox,
                -oy
            )
        }
    }

    /// `element` の親から辿って祖先の CSS `transform` による累積線形
    /// 変換（並進成分を除いた 2x2 行列 `(a, b, c, d)`）を返す（レビュー
    /// 指摘対応、イシュー #2518 さらなるフォローアップ、codex-review P1
    /// 「祖先変換の符号と軸方向を保持する」）。
    ///
    /// `getBoundingClientRect()` で測る Before/Last の位置差
    /// （[`super::invert`]）は画面（viewport）座標であり、祖先に
    /// `transform` が掛かっていると、要素自身へ設定する `translate()` は
    /// その祖先変換でさらに変換されて画面へ反映される（例: 祖先
    /// `scale(2)` 下でローカル `translate(100px)` は画面上 200px の移動
    /// として現れる）。[`to_local_delta`] がこの戻り値の逆行列で並進成分
    /// をローカル座標へ変換する。スケール比（`delta.sx`/`delta.sy`）は
    /// Before/Last の両方が同じ祖先変換を経て計測されるため比を取ると
    /// 相殺され、この行列による補正は**軸整列変換（回転・skew を含まない
    /// 場合）に限り**不要（[`to_local_delta`]・[`super::
    /// is_axis_aligned_linear`] doc 参照。回転を含む場合は再生自体を
    /// 省略する）。
    ///
    /// 各祖先の computed `transform` が `none`/空文字列/取得失敗の場合は
    /// 恒等変換（寄与なし）として扱う。`matrix3d(...)`・不正な形式など
    /// [`super::parse_matrix_components`] がパースできない**非恒等**の
    /// 変換を持つ祖先が 1 つでもあれば `None` を返す（対応できない変換の
    /// 下では並進補正の符号・軸を正しく判定できないため、[`play`] は
    /// この場合再生自体を省略する、fail-safe）。
    fn ancestor_linear_matrix(element: &HtmlElement) -> Option<(f64, f64, f64, f64)> {
        let Some(window) = web_sys::window() else {
            return Some((1.0, 0.0, 0.0, 1.0));
        };
        let mut cumulative = (1.0, 0.0, 0.0, 1.0);
        let mut current = element.parent_element();
        while let Some(el) = current {
            let computed = window.get_computed_style(&el).ok().flatten();
            // 契約絞り込み（codex-review 第 5 ラウンド是正、イシュー
            // #2518）: 祖先が独立した CSS 変形プロパティ（`scale`/
            // `rotate`/`translate`）を持つ場合、その効果は `transform`
            // プロパティの computed 値には現れないため、以降で
            // `transform` のみを読んで構築する累積行列では捕捉できない
            // （`has_independent_transform_property` doc 参照）。この
            // 場合は再生自体を省略する。
            if let Some(computed) = &computed {
                if has_independent_transform_property(computed) {
                    return None;
                }
                // 契約絞り込み（codex-review 第 8 ラウンド是正、イシュー
                // #2518）: 祖先が CSS `zoom` に恒等以外の値を持つ場合、
                // `getBoundingClientRect()` の viewport 座標上の差分が
                // 実際の CSS 座標上の差分より拡縮されてしまい、以降で
                // `transform` のみを読んで構築する累積行列では捕捉できない
                // （`super::is_non_identity_zoom` doc 参照）。この場合は
                // 再生自体を省略する。
                let zoom = computed.get_property_value("zoom").unwrap_or_default();
                if super::is_non_identity_zoom(&zoom) {
                    return None;
                }
            }
            let value = computed
                .and_then(|computed| computed.get_property_value("transform").ok())
                .unwrap_or_default();
            if !value.is_empty() && value.trim() != "none" {
                let components = super::parse_matrix_components(&value)?;
                cumulative = super::compose_linear_2x2(
                    cumulative,
                    (components[0], components[1], components[2], components[3]),
                );
            }
            current = el.parent_element();
        }
        Some(cumulative)
    }

    /// [`ancestor_linear_matrix`] の逆行列で `delta.tx`/`delta.ty` を
    /// 要素ローカル座標へ変換する（レビュー指摘対応、イシュー #2518
    /// さらなるフォローアップ、codex-review P1「祖先変換の符号と軸方向を
    /// 保持する」）。
    ///
    /// 祖先変換が非可逆・パース不能なら `None`（[`play`] はこの場合
    /// 再生を省略する。対応できない変換の下で誤った符号・軸の補正を
    /// 適用するより安全、fail-safe）。
    ///
    /// 祖先変換が[`super::is_axis_aligned_linear`]でない（回転・skew を
    /// 含む）場合も `None` を返す（レビュー指摘対応、イシュー #2518
    /// さらなるフォローアップ、codex-review P1「祖先の回転をサイズ補正
    /// にも反映する」）: `delta.sx`/`sy`（[`super::invert`] が viewport
    /// 座標の First/Last 幅・高さ比から算出）は、祖先が軸整列変換の場合
    /// のみローカル座標の比と一致する（[`super::is_axis_aligned_linear`]
    /// doc 参照）。祖先が回転を含む場合、viewport 座標では元のローカル
    /// 軸が入れ替わって現れるため（例: `rotate(90deg)` 下でローカル幅が
    /// viewport では高さとして現れる）、`sx`/`sy` をそのまま使うと違う
    /// 軸を拡縮してしまい対応不能。並進の符号・軸だけを正しく変換しても
    /// サイズが誤るため、この場合は再生自体を省略する。
    fn to_local_delta(element: &HtmlElement, delta: FlipDelta) -> Option<FlipDelta> {
        let (a, b, c, d) = ancestor_linear_matrix(element)?;
        if !super::is_axis_aligned_linear((a, b, c, d)) {
            return None;
        }
        let inverse = super::invert_linear_2x2(a, b, c, d)?;
        let (tx, ty) = super::apply_linear_2x2(inverse, delta.tx, delta.ty);
        Some(FlipDelta {
            tx,
            ty,
            sx: delta.sx,
            sy: delta.sy,
        })
    }

    /// FLIP 再生の外部ハンドル（[`play`] の戻り値）。
    ///
    /// レビュー指摘対応（イシュー #2518 さらなるフォローアップ、
    /// codex-review P1「公開 play API の途中停止でも元のスタイルを復元
    /// する」）: [`crate::raf_driver::AnimationLoop`] を直接返していた
    /// 旧実装は、収束前に呼び出し元が [`crate::raf_driver::
    /// AnimationLoop::stop`] を呼ぶ・戻り値を drop するいずれの経路でも
    /// rAF のみが止まり、補正 `transform`/`transform-origin` が要素へ
    /// 残り続けた（`OriginalStyle` が文書化する「収束・強制停止・
    /// `prefers-reduced-motion` のいずれの終了経路でも元の値へ復元する」
    /// 契約に反する）。本型は spring 収束（`done`）の有無を共有
    /// `Rc<Cell<bool>>` で追跡し、[`Drop`] で未収束なら [`OriginalStyle::
    /// restore`] を呼ぶことで、公開 API 経由の任意の終了経路（収束・
    /// 明示 [`stop`](Self::stop)・単純な drop）を問わず契約を満たす。
    ///
    /// [`is_done`](Self::is_done) は呼び出し元がアニメーションの収束
    /// 状態（＝既に [`OriginalStyle::restore`] 済みか）を確認するために
    /// 使う。`crate::layout_flip`（wasm-full 側配線）は本ラウンドの
    /// 再設計（[`super::anchor_correct`] doc 参照）により、進行中の
    /// アニメーションを中断する際の補正値引き継ぎ計算
    /// （旧 `current_viewport_delta`/`apply_delta_to_rect`）を必要と
    /// しなくなった: Before は常に「現在実際に表示されている視覚矩形」
    /// （[`measure`]）を直接測るだけで、進行中の FLIP 補正込みの表示
    /// 位置がそのまま正しい値になる（`crate::layout_flip::capture_before`
    /// 参照）。
    pub struct FlipAnimation {
        // `AnimationLoop::drop` が rAF コールバックの `Closure` を解放する
        // ことが目的のフィールド（`raf_driver.rs` doc 参照）。フィールド
        // 宣言順により、`Drop::drop` 本体（下記）が先に走った**後**に
        // このフィールドの `Drop` が実行される。
        #[allow(dead_code)]
        loop_: crate::raf_driver::AnimationLoop,
        done: Rc<Cell<bool>>,
        element: HtmlElement,
        original: OriginalStyle,
    }

    impl FlipAnimation {
        /// spring が収束済み（= 既に [`OriginalStyle::restore`] 済み）
        /// かどうかを返す。
        #[must_use]
        pub fn is_done(&self) -> bool {
            self.done.get()
        }

        /// アニメーションを停止する（[`Drop`] と同じ効果: 未収束なら
        /// 元のスタイルへ復元する）。
        pub fn stop(self) {}
    }

    impl Drop for FlipAnimation {
        fn drop(&mut self) {
            if !self.done.get() {
                self.original.restore(&self.element);
            }
        }
    }

    /// [`crate::reduced_motion::prefers_reduced_motion`]・[`to_local_delta`]・
    /// `Spring::new`・`RafDriver::new` のいずれかが不成立の場合の即時終了経路
    /// （[`play`] 参照）。元の値へ即座に復元し、収束済み
    /// （[`FlipAnimation::is_done`] が `true`）の no-op ハンドルを返す。
    fn finished_animation(element: HtmlElement, original: OriginalStyle) -> FlipAnimation {
        original.restore(&element);
        FlipAnimation {
            loop_: crate::raf_driver::AnimationLoop::start(|| false),
            done: Rc::new(Cell::new(true)),
            element,
            original,
        }
    }

    /// `style` の `transform`/`transform-origin` を `!important` 付きで
    /// 書き込み、あわせて呼び出し元が保持する抑止計画
    /// （[`compute_transition_suppression`]）を適用する
    /// （レビュー指摘対応、イシュー #2518 さらなるフォローアップ、
    /// codex-review P1「再生中もスタイルシートの important 宣言を
    /// 上書きする」・第 5 ラウンド是正「既存の CSS transition を抑止
    /// してから計測・再生する」・第 7 ラウンド是正「抑止を transform /
    /// transform-origin に限定する」）。
    ///
    /// スタイルシートに `transform: scale(2) !important` のような宣言が
    /// ある場合、通常優先度の inline 書き込みはカスケードで負けて描画に
    /// 反映されない（CSS の仕様上、`!important` はオリジン・詳細度を
    /// 問わず通常優先度の宣言に必ず勝つ。一方、inline style 属性は
    /// セレクタベースのどの宣言よりも高い詳細度を持つため、inline の
    /// `!important` は他の起源のどの `!important` にも勝つ）。[`play`]
    /// の初期書き込み・tick ごとの書き込みの両方をこの関数経由にする
    /// ことで、由来（inline かスタイルシートか）を問わず FLIP 補正が
    /// 常に実際の描画へ反映される。
    ///
    /// 対象要素に `transition: transform 300ms` 等が設定されている場合、
    /// 本関数の `transform` 書き込みがその transition の対象になり、
    /// Invert 直後の補正が即座に反映されず、後続の rAF 駆動の書き込みも
    /// CSS transition と競合してしまう（FLIP の計測・即時補正契約に
    /// 反する）。tick ごとに `suppression`（呼び出し元が保持する抑止
    /// 計画）を再度書き込むことで、スタイルシート側が transition を
    /// 再設定してくる場合にも対応する（`transform`/`transform-origin`
    /// と同じ `!important` の論理）。
    ///
    /// codex-review 第 6 ラウンド是正（イシュー #2518。P1「FLIP と無関係
    /// な進行中の遷移をキャンセルしない」）: `transition: none
    /// !important` の一括抑止は `transition-property` も `none` へ
    /// 変えてしまい、`opacity` 等の FLIP と無関係な進行中の遷移まで
    /// キャンセルする副作用があった。
    ///
    /// codex-review 第 7 ラウンド是正（イシュー #2518。P1「抑止を
    /// transform / transform-origin に限定する」）: `transition-duration`/
    /// `-delay` を一律 `0s` にする方式も、FLIP 対象以外のプロパティ
    /// （`opacity` 等）に効いている遷移の duration/delay まで即時化して
    /// しまう副作用があったため、[`compute_transition_suppression`] が
    /// `transform`/`transform-origin` の 2 項目だけを対象に持つ抑止計画
    /// へスコープを絞った。`suppression` は [`play`] が 1 回だけ算出して
    /// 呼び出し元から渡される（本関数自身は `getComputedStyle` を呼ば
    /// ない）: tick ごとに計画を再算出すると、前回の書き込みが今回の
    /// computed 値に含まれてしまい `transform`/`transform-origin` が
    /// 呼び出し回数分重複して肥大化し続ける（[`compute_transition_
    /// suppression`] doc 参照）。
    ///
    /// 終了時の復元（[`OriginalStyle::restore`]）は要素が元々持っていた
    /// inline 宣言の優先度（[`read_property`] が保存した値）へ戻すため、
    /// 本関数の呼び出しが元の宣言の優先度を汚染することはない。
    ///
    /// codex-review 第 9 ラウンド是正（イシュー #2518。P1「transform-box
    /// の参照原点を補正に反映する」）: `origin`（常に `"0 0"`、
    /// [`OriginalStyle::origin_css`] 参照）は要素の `transform-box`
    /// 基準ボックスの左上を指すため、要素自身が `transform-box:
    /// content-box` 等を持っていると `"0 0"` が border-box 左上（
    /// `getBoundingClientRect()`/`anchor_correct` の前提）からずれる
    /// （[`OriginalStyle::compose_transform`] doc「transform-box の
    /// 参照原点」節参照）。`transform-box: border-box !important` を
    /// あわせて強制することでこのずれを防ぐ。
    fn write_transform_important(
        style: &CssStyleDeclaration,
        suppression: &[Option<String>; 5],
        transform: &str,
        origin: &str,
    ) {
        apply_transition_suppression(style, suppression);
        let _ = style.set_property_with_priority("transform-box", "border-box", "important");
        let _ = style.set_property_with_priority("transform-origin", origin, "important");
        let _ = style.set_property_with_priority("transform", transform, "important");
    }

    /// Invert 補正値を即座に適用し、[`crate::raf_driver::AnimationLoop`]
    /// と `Spring` で恒等変換（[`IDENTITY`]）へ収束させる（Play 段階、
    /// 実装計画 §3.2 手順2〜4）。
    ///
    /// `original`（[`OriginalStyle::capture`]）が保持する元の
    /// `transform`/`transform-origin` は FLIP 補正へ合成して適用し、
    /// 収束・強制停止（[`FlipAnimation::stop`]/drop）・
    /// `prefers-reduced-motion` のいずれの終了経路でもこの元の値へ復元
    /// する（レビュー指摘対応: 利用者が指定した `transform` を保存・
    /// 復元する、イシュー #2518 フォローアップ・さらなるフォローアップ、
    /// [`FlipAnimation`] doc 参照）。
    ///
    /// # 呼び出し前提（codex-review 第 7 ラウンド是正、イシュー #2518。
    /// P1「計測で確定した変形と FLIP の収束先を一致させる」）
    ///
    /// `original` は、呼び出し元が Last の**layout**矩形計測
    /// （[`measure_layout_batch`]/[`measure_clearing_transform`]。要素
    /// 自身の transform を一時的に無効化・復元する過程で、進行中の CSS
    /// transform transition を目標値へ確定〔settle〕させる副作用を持つ、
    /// 各関数 doc 参照）を終えた**後**に [`OriginalStyle::capture`] で
    /// 取得したものであることを前提とする。この順序を守らないと、
    /// `original.computed_transform` が settle 前の中間値のままになり、
    /// 本関数が収束させる先（`IDENTITY` 適用後に
    /// [`OriginalStyle::restore`] が書き込む値）と、実際に確定する
    /// レイアウト計測後の目標値が食い違い、収束直後に見た目が跳ぶ
    /// （`crate::layout_flip::play_after` の新しい 4 パスの順序・
    /// モジュール doc「計測・合成の再設計」節参照。本クレート単体で
    /// [`play`] を直接呼ぶ経路（[`OriginalStyle::capture`] を独自に
    /// 呼ぶ利用者）も同じ順序を守ること）。
    #[must_use]
    pub fn play(
        element: HtmlElement,
        delta: FlipDelta,
        config: SpringConfig,
        original: OriginalStyle,
    ) -> FlipAnimation {
        // a11y（実装計画 §3.2 手順 1）: reduced-motion 判定はクレート共通の
        // [`crate::reduced_motion::prefers_reduced_motion`] へ一本化する
        // （Bugbot 指摘、イシュー #2518）。判定不能時は共通ヘルパの
        // fail-safe（`true` = アニメーションを抑止）に従い、FLIP のみが
        // 別の既定で動く二重実装を持たない。
        if crate::reduced_motion::prefers_reduced_motion() {
            return finished_animation(element, original);
        }

        // 契約絞り込み（codex-review 第 5 ラウンド是正、イシュー #2518）:
        // 要素自身が 2D 専用の計測・合成契約の対応範囲外（3D 変形・z 成分
        // 付き transform-origin・独立した CSS 変形プロパティ）を持つ
        // 場合は再生自体を省略する（`OriginalStyle::unsupported` doc・
        // `is_matrix3d`/`has_independent_transform_property` の rustdoc
        // を正とする）。
        if original.unsupported {
            return finished_animation(element, original);
        }

        let Some(local_delta) = to_local_delta(&element, delta) else {
            return finished_animation(element, original);
        };

        // 抑止計画は再生開始時に 1 回だけ算出し、tick ごとに再利用する
        // （[`write_transform_important`] doc「tick ごとの再計算をしない」
        // 参照）。
        let suppression = compute_transition_suppression(&element);

        let style = element.style();
        write_transform_important(
            &style,
            &suppression,
            &original.compose_transform(local_delta),
            original.origin_css(),
        );

        let Some(spring) = Spring::new(config, 0.0, 1.0, 0.0) else {
            return finished_animation(element, original);
        };

        let Some(mut driver) = crate::raf_driver::RafDriver::new() else {
            return finished_animation(element, original);
        };

        let done = Rc::new(Cell::new(false));
        let loop_done = done.clone();
        let loop_element = element.clone();
        let loop_original = original.clone();
        let loop_suppression = suppression.clone();
        let mut elapsed = 0.0_f64;
        let loop_ = crate::raf_driver::AnimationLoop::start(move || {
            let Some(dt) = driver.tick() else {
                return true;
            };
            elapsed += dt;
            let state = spring.at(elapsed);
            if state.done {
                loop_done.set(true);
                loop_original.restore(&loop_element);
                return false;
            }
            let local_value = local_delta.interpolate(&IDENTITY, state.value);
            let loop_style = loop_element.style();
            write_transform_important(
                &loop_style,
                &loop_suppression,
                &loop_original.compose_transform(local_value),
                loop_original.origin_css(),
            );
            true
        });

        FlipAnimation {
            loop_,
            done,
            element,
            original,
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{
    measure, measure_clearing_transform, measure_layout_batch, play, FlipAnimation, OriginalStyle,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invert_computes_translate_and_scale_from_two_rects() {
        let first = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let last = Rect {
            x: 50.0,
            y: 20.0,
            width: 50.0,
            height: 200.0,
        };
        let delta = invert(first, last).expect("有限・非ゼロのため Some");
        assert_eq!(delta.tx, -50.0);
        assert_eq!(delta.ty, -20.0);
        assert_eq!(delta.sx, 2.0);
        assert_eq!(delta.sy, 0.5);
    }

    #[test]
    fn split_transition_list_splits_on_top_level_commas_only() {
        assert_eq!(
            split_transition_list("opacity, transform"),
            vec!["opacity".to_string(), "transform".to_string()]
        );
        // cubic-bezier(...) や steps(...) 内部のコンマは区切りにしない。
        assert_eq!(
            split_transition_list("cubic-bezier(0.1, 0.7, 1, 0.1), linear"),
            vec![
                "cubic-bezier(0.1, 0.7, 1, 0.1)".to_string(),
                "linear".to_string()
            ]
        );
        assert_eq!(split_transition_list(""), Vec::<String>::new());
        assert_eq!(split_transition_list("none"), vec!["none".to_string()]);
    }

    #[test]
    fn expand_transition_lists_appends_transform_scope_without_touching_existing_entries() {
        let result = expand_transition_lists(
            "opacity, color",
            "300ms, 1s",
            "0s, 50ms",
            "ease, linear",
            "normal",
        );
        assert_eq!(
            result,
            [
                Some("opacity, color, transform, transform-origin".to_string()),
                Some("300ms, 1s, 0s, 0s".to_string()),
                Some("0s, 50ms, 0s, 0s".to_string()),
                Some("ease, linear, linear, linear".to_string()),
                Some("normal, normal, normal, normal".to_string()),
            ]
        );
    }

    #[test]
    fn expand_transition_lists_cycles_shorter_lists_per_css_index_match_rule() {
        // duration が 1 件のみ・property が 2 件 → 循環展開で両方に適用。
        let result = expand_transition_lists("opacity, color", "300ms", "0s", "ease", "normal");
        assert_eq!(result[1], Some("300ms, 300ms, 0s, 0s".to_string()));
    }

    #[test]
    fn expand_transition_lists_none_property_writes_only_three_longhands() {
        let result = expand_transition_lists("none", "0s", "0s", "ease", "normal");
        assert_eq!(
            result,
            [
                Some("transform, transform-origin".to_string()),
                Some("0s, 0s".to_string()),
                Some("0s, 0s".to_string()),
                None,
                None,
            ]
        );
    }

    #[test]
    fn expand_transition_lists_empty_property_behaves_like_none() {
        let result = expand_transition_lists("", "", "", "", "");
        assert_eq!(
            result,
            [
                Some("transform, transform-origin".to_string()),
                Some("0s, 0s".to_string()),
                Some("0s, 0s".to_string()),
                None,
                None,
            ]
        );
    }

    #[test]
    fn invert_returns_none_for_zero_width_last() {
        let first = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let last = Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 100.0,
        };
        assert!(invert(first, last).is_none());
    }

    #[test]
    fn invert_returns_none_for_zero_height_last() {
        let first = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let last = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 0.0,
        };
        assert!(invert(first, last).is_none());
    }

    #[test]
    fn invert_returns_none_for_non_finite_input() {
        let first = Rect {
            x: f64::NAN,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let last = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        assert!(invert(first, last).is_none());

        let first_inf = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let last_inf = Rect {
            x: f64::INFINITY,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        assert!(invert(first_inf, last_inf).is_none());
    }

    #[test]
    fn invert_returns_identity_when_rects_are_equal() {
        let rect = Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        let delta = invert(rect, rect).expect("同一矩形は有効な入力");
        assert_eq!(delta, IDENTITY);
    }

    #[test]
    fn to_transform_css_formats_translate_and_scale() {
        let delta = FlipDelta {
            tx: -50.0,
            ty: -20.0,
            sx: 2.0,
            sy: 0.5,
        };
        assert_eq!(
            to_transform_css(delta),
            "translate(-50px, -20px) scale(2, 0.5)"
        );
    }

    #[test]
    fn to_transform_css_identity_is_no_op_transform() {
        assert_eq!(
            to_transform_css(IDENTITY),
            "translate(0px, 0px) scale(1, 1)"
        );
    }

    #[test]
    fn flip_delta_interpolate_matches_endpoints() {
        use fandhe_animation::interpolate::Interpolate;
        let from = FlipDelta {
            tx: -50.0,
            ty: -20.0,
            sx: 2.0,
            sy: 0.5,
        };
        let to = IDENTITY;
        assert_eq!(from.interpolate(&to, 0.0), from);
        assert_eq!(from.interpolate(&to, 1.0), to);
        let mid = from.interpolate(&to, 0.5);
        assert_eq!(mid.tx, -25.0);
        assert_eq!(mid.ty, -10.0);
        assert_eq!(mid.sx, 1.5);
        assert_eq!(mid.sy, 0.75);
    }

    // レビュー指摘対応（イシュー #2518 さらなるフォローアップ、codex-review
    // P1・Cursor Bugbot）: 原点補正の並進量をスケールで割ってはいけない
    // （`translate(...) scale(...)` 形式の CSS では並進成分自体は
    // スケールされないため）。`O=50, s=2` なら補正は `50px`（旧実装は
    // 誤って `50/2=25px` になっていた）。
    // codex-review 第 4 ラウンド是正（イシュー #2518）: `anchor_correct`
    // の再設計に伴い、旧 `adjust_translate_for_shared_origin` の単体
    // テスト群を置き換える。以下は `anchor_correct` doc の数値例
    // （要素自身の `scale(2)`・`transform-origin: 50% 50%`、幅が
    // 100px→50px）を native テストとして固定する。

    #[test]
    fn anchor_correct_is_no_op_when_scale_is_identity() {
        // サイズ変化がない（`sx=sy=1`）場合、`(1-S)` 係数が 0 になり
        // `anchor_correct` は無補正（並進のみ・要素自身の変形なしの
        // 旧来の挙動と一致）。
        let delta = FlipDelta {
            tx: 10.0,
            ty: -5.0,
            sx: 1.0,
            sy: 1.0,
        };
        let last_visual = Rect {
            x: -25.0,
            y: -25.0,
            width: 100.0,
            height: 100.0,
        };
        let last_layout = Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        };
        let corrected = anchor_correct(delta, last_visual, last_layout);
        assert_eq!(corrected, delta);
    }

    #[test]
    fn anchor_correct_reproduces_first_rect_under_own_scale() {
        // `anchor_correct` doc の数値例を固定する: 要素自身
        // `scale(2)`・`transform-origin: 50% 50%`（既定）、祖先変換なし、
        // レイアウト幅が 100px→50px（`left: 0` 固定）に変化。
        // last_layout.x=0, w=50 / last_visual.x=-25, w=100
        // （中心 25 基準で 2 倍）/ first_visual.x=-50, w=200
        // （変更前、中心 50 基準で 2 倍）。
        let first_visual = Rect {
            x: -50.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        };
        let last_visual = Rect {
            x: -25.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let last_layout = Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 100.0,
        };
        let delta = invert(first_visual, last_visual).expect("有限・非ゼロのため Some");
        assert_eq!(delta.sx, 2.0);
        let corrected = anchor_correct(delta, last_visual, last_layout);
        // t = d + (1-S)*anchor = -25 + (1-2)*(-25) = 0（doc の数値例）。
        assert_eq!(corrected.tx, 0.0);

        // Invert 直後（祖先なしなのでローカル座標=viewport 座標）の
        // 視覚矩形の左端が First と一致することまで往復で確認する:
        // p_local=0 における合成結果は
        // `last_layout.x + S*(last_visual.x-last_layout.x) + t`。
        let anchor_x = last_visual.x - last_layout.x;
        let reproduced_left = last_layout.x + corrected.sx * anchor_x + corrected.tx;
        assert_eq!(reproduced_left, first_visual.x);
        // 右端（幅）も一致することを確認する（`p_local=last_layout.width`）。
        let anchor_right = {
            let q_right = last_visual.x - last_layout.x
                + (last_visual.width / last_layout.width) * last_layout.width;
            last_layout.x + corrected.sx * q_right + corrected.tx
        };
        assert_eq!(anchor_right - reproduced_left, first_visual.width);
    }

    #[test]
    fn anchor_correct_is_no_op_for_origin_zero_zero_scale() {
        // 要素自身 `scale(2)`・`transform-origin: 0 0`、`left: 0` 固定で
        // 幅が 50px→100px に変化（thread 1 の再現ケース）。origin が
        // layout 矩形の左上（`transform-origin: 0 0`）と一致するため
        // `last_visual.x == last_layout.x` となり anchor 項は 0
        // （`compose_transform` が使う `resolved_origin_px` も `(0,0)` に
        // 解決される想定と整合する）。
        let first_visual = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 200.0,
        };
        let last_visual = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        let last_layout = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 200.0,
        };
        let delta = invert(first_visual, last_visual).expect("有限・非ゼロのため Some");
        let corrected = anchor_correct(delta, last_visual, last_layout);
        assert_eq!(corrected, delta);
        assert_eq!(corrected.tx, 0.0);
    }

    #[test]
    fn anchor_correct_handles_rotated_ancestor_free_own_transform() {
        // `rotate(90deg)`・中心原点、layout 100×50 → 50×50（thread 3 の
        // 再現ケース）。bbox が First（50×100）と一致することを固定する。
        let first_visual = Rect {
            x: 0.0,
            y: -25.0,
            width: 50.0,
            height: 100.0,
        };
        let last_visual = Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        };
        let last_layout = Rect {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        };
        let delta = invert(first_visual, last_visual).expect("有限・非ゼロのため Some");
        assert_eq!((delta.sx, delta.sy), (1.0, 2.0));
        let corrected = anchor_correct(delta, last_visual, last_layout);
        // 自身の変形が並進のみ（回転はあるが last_visual==last_layout、
        // すなわち bbox 上の原点ずれがない）なので anchor 項は 0 になる。
        assert_eq!(corrected, delta);
    }

    #[test]
    fn parse_matrix_components_extracts_all_six() {
        let components =
            parse_matrix_components("matrix(3, 0, 0, 0.5, 10, -4)").expect("パース成功");
        assert_eq!(components, [3.0, 0.0, 0.0, 0.5, 10.0, -4.0]);
    }

    #[test]
    fn parse_matrix_components_rejects_none_and_matrix3d() {
        assert_eq!(parse_matrix_components("none"), None);
        assert_eq!(
            parse_matrix_components("matrix3d(1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1)"),
            None
        );
    }

    #[test]
    fn parse_matrix_components_rejects_malformed_input() {
        // 引数の個数が異なる不正な形式。
        assert_eq!(parse_matrix_components("matrix(1, 0, 0, 1, 0)"), None);
        assert_eq!(parse_matrix_components("matrix(1, 0, 0, 1, 0, 0, 0)"), None);
        assert_eq!(parse_matrix_components(""), None);
    }

    // レビュー指摘対応（イシュー #2518 さらなるフォローアップ、
    // codex-review P1「祖先変換の符号と軸方向を保持する」）:
    // `invert_linear_2x2`/`apply_linear_2x2` が祖先の `scaleX(-1)`・
    // `rotate(90deg)` 下でも並進量の符号・軸を正しくローカル座標へ
    // 戻せることを native テストで固定する。

    #[test]
    fn invert_linear_2x2_rejects_singular_matrix() {
        // `matrix(0, 0, 0, 0)`（すべて 0 に潰れた変換）は非可逆。
        assert_eq!(invert_linear_2x2(0.0, 0.0, 0.0, 0.0), None);
    }

    #[test]
    fn apply_linear_2x2_reverses_ancestor_scale_x_negative_one() {
        // `scaleX(-1)` の computed 行列は `matrix(-1, 0, 0, 1, 0, 0)`。
        let (a, b, c, d) = (-1.0, 0.0, 0.0, 1.0);
        let inverse = invert_linear_2x2(a, b, c, d).expect("scaleX(-1) は自己逆");
        let (local_tx, local_ty) = apply_linear_2x2(inverse, 100.0, 0.0);
        assert_eq!(local_tx, -100.0, "反転祖先の下では並進の符号が反転するはず");
        assert_eq!(local_ty, 0.0);
    }

    #[test]
    fn apply_linear_2x2_reverses_ancestor_rotate_90deg() {
        // `rotate(90deg)` の computed 行列は `matrix(0, 1, -1, 0, 0, 0)`。
        let (a, b, c, d) = (0.0, 1.0, -1.0, 0.0);
        let inverse = invert_linear_2x2(a, b, c, d).expect("rotate(90deg) は可逆");
        let (local_tx, local_ty) = apply_linear_2x2(inverse, 100.0, 0.0);
        assert_eq!(local_tx, 0.0, "90 度回転の逆変換で軸が入れ替わるはず");
        assert_eq!(local_ty, -100.0);
    }

    #[test]
    fn compose_linear_2x2_multiplies_nested_ancestor_scales() {
        // 内側 `scale(3)` → 外側 `scale(2)` の合成は `scale(6)` と等価。
        let inner = (3.0, 0.0, 0.0, 3.0);
        let outer = (2.0, 0.0, 0.0, 2.0);
        assert_eq!(compose_linear_2x2(inner, outer), (6.0, 0.0, 0.0, 6.0));
    }

    // レビュー指摘対応（イシュー #2518 さらなるフォローアップ、
    // codex-review P1「祖先の回転をサイズ補正にも反映する」）:
    // `is_axis_aligned_linear` が回転・skew を正しく非軸整列と判定し、
    // 純粋な拡大縮小（反転含む）を軸整列と判定することを固定する。

    #[test]
    fn is_axis_aligned_linear_accepts_positive_scale() {
        assert!(is_axis_aligned_linear((2.0, 0.0, 0.0, 0.5)));
    }

    // レビュー指摘対応（イシュー #2518 さらなるフォローアップ、
    // codex-review P1「反転した祖先の下では拡縮に伴う原点のずれも補正
    // する」）: 対角成分が負（反転、`scaleX(-1)` 等）の祖先は、並進のみ
    // の逆行列変換ではサイズ変更に伴う原点のずれを補正できないため、
    // 軸整列変換の許容範囲から除外する（`is_axis_aligned_linear` doc
    // 「対応範囲」参照）。

    #[test]
    fn is_axis_aligned_linear_rejects_negative_diagonal_flip() {
        // `scaleX(-1)` の computed 行列 `matrix(-1,0,0,1,...)`。
        assert!(!is_axis_aligned_linear((-1.0, 0.0, 0.0, 1.0)));
        // `scaleY(-1)` の computed 行列 `matrix(1,0,0,-1,...)`。
        assert!(!is_axis_aligned_linear((1.0, 0.0, 0.0, -1.0)));
    }

    #[test]
    fn is_axis_aligned_linear_rejects_point_reflection_within_epsilon() {
        // `rotate(180deg)` の computed 行列は `matrix(-1,0,0,-1,...)`
        // （scaleX(-1)*scaleY(-1) と等価な点対称・反転）。ブラウザの
        // trig 数値誤差（sin 成分がごく僅かな非ゼロ値になる場合）を
        // 許容誤差で吸収しつつ、対角成分の負号自体は正しく検出できる
        // ことを固定する。
        assert!(!is_axis_aligned_linear((-1.0, 1e-9, -1e-9, -1.0)));
    }

    #[test]
    fn is_axis_aligned_linear_rejects_rotation() {
        // `rotate(90deg)` の computed 行列 `matrix(0,1,-1,0,...)`。
        assert!(!is_axis_aligned_linear((0.0, 1.0, -1.0, 0.0)));
    }

    #[test]
    fn is_axis_aligned_linear_rejects_skew() {
        assert!(!is_axis_aligned_linear((1.0, 0.5, 0.0, 1.0)));
    }

    // codex-review 第 5 ラウンド是正（イシュー #2518、契約絞り込み）:
    // `matrix3d(...)` を持つ要素は `play` が再生を省略する
    // （`OriginalStyle::unsupported`）。`is_matrix3d` はその判定の中核
    // であり、DOM 非依存の純粋関数として native テストで固定できる。

    #[test]
    fn is_matrix3d_accepts_matrix3d_prefix() {
        assert!(is_matrix3d(
            "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)"
        ));
    }

    #[test]
    fn is_matrix3d_rejects_2d_matrix_and_none() {
        assert!(!is_matrix3d("matrix(1, 0, 0, 1, 0, 0)"));
        assert!(!is_matrix3d("none"));
    }

    // codex-review 第 8 ラウンド是正（イシュー #2518、契約絞り込み）:
    // CSS `zoom` に恒等以外の値を持つ要素自身・祖先は `play` が再生を
    // 省略する（`OriginalStyle::unsupported`/`ancestor_linear_matrix` が
    // `None` を返す経路）。`is_non_identity_zoom` はその判定の中核であり、
    // DOM 非依存の純粋関数として native テストで固定できる。

    #[test]
    fn is_non_identity_zoom_treats_one_normal_and_empty_as_identity() {
        assert!(!is_non_identity_zoom("1"));
        assert!(!is_non_identity_zoom("normal"));
        assert!(!is_non_identity_zoom(""));
        assert!(!is_non_identity_zoom("  1  "));
    }

    #[test]
    fn is_non_identity_zoom_detects_scaled_values() {
        assert!(is_non_identity_zoom("2"));
        assert!(is_non_identity_zoom("0.5"));
        assert!(is_non_identity_zoom("150%"));
    }

    // codex-review 指摘是正（イシュー #2544）: `list_motion::enter_css`
    // の `scale: 1`（fill-mode: both で再生後も残留する恒等値）を非恒等
    // 値と誤検知しない回帰テスト。

    #[test]
    fn is_identity_independent_transform_value_treats_identity_as_true() {
        assert!(is_identity_independent_transform_value("scale", "1"));
        assert!(is_identity_independent_transform_value("scale", "1 1"));
        assert!(is_identity_independent_transform_value("rotate", "0deg"));
        assert!(is_identity_independent_transform_value("rotate", "0"));
        assert!(is_identity_independent_transform_value("translate", "0px"));
        assert!(is_identity_independent_transform_value(
            "translate",
            "0px 0px"
        ));
    }

    #[test]
    fn is_identity_independent_transform_value_detects_non_identity() {
        assert!(!is_identity_independent_transform_value("scale", "2"));
        assert!(!is_identity_independent_transform_value("scale", "1 2"));
        assert!(!is_identity_independent_transform_value("rotate", "45deg"));
        assert!(!is_identity_independent_transform_value(
            "translate",
            "10px"
        ));
    }

    #[test]
    fn is_identity_independent_transform_value_empty_is_not_identity() {
        // fail-safe: パース不能・空成分は「恒等ではない」側へ倒す。
        assert!(!is_identity_independent_transform_value("scale", ""));
        assert!(!is_identity_independent_transform_value("scale", "auto"));
    }

    // Bugbot 指摘対応（codex-review 第 10 ラウンド、イシュー #2518。
    // 「own-transform origin ignores forced box」）: `classify_transform_
    // box` は `transform-box` の computed 値から border-box 換算方式を
    // 決定する DOM 非依存の純粋関数であり、native テストで固定できる。

    #[test]
    fn classify_transform_box_treats_empty_view_box_and_border_box_as_border_box() {
        assert_eq!(classify_transform_box(""), Some(TransformBoxKind::Border));
        assert_eq!(
            classify_transform_box("view-box"),
            Some(TransformBoxKind::Border)
        );
        assert_eq!(
            classify_transform_box("border-box"),
            Some(TransformBoxKind::Border)
        );
        assert_eq!(
            classify_transform_box("  border-box  "),
            Some(TransformBoxKind::Border)
        );
    }

    #[test]
    fn classify_transform_box_recognizes_padding_box_and_content_box() {
        assert_eq!(
            classify_transform_box("padding-box"),
            Some(TransformBoxKind::Padding)
        );
        assert_eq!(
            classify_transform_box("content-box"),
            Some(TransformBoxKind::Content)
        );
    }

    #[test]
    fn classify_transform_box_rejects_svg_only_reference_boxes() {
        assert_eq!(classify_transform_box("fill-box"), None);
        assert_eq!(classify_transform_box("stroke-box"), None);
        assert_eq!(classify_transform_box("unknown-value"), None);
    }
}
