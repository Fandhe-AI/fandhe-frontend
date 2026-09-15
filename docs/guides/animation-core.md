# fandhe-animation / fandhe-frontend-animation API ガイド

本ガイドは、`fandhe-animation`（プラットフォーム非依存のアニメーション演算
コア）と `fandhe-frontend-animation`（Web アダプタ）を **Rust コードから
直接呼び出す**利用者向けの解説です。`Spring`/`Keyframes`/`RafDriver`/
`DomTarget`/`animate()` 等の型・関数を自分のコードから呼ぶ使い方を扱い
ます。

pre-styled-ui の presence/keyframes/stagger や wasm-full の in-view・
stagger 配線・View Transitions・FLIP・SVG path・hover/press・scroll と
いった**宣言的な `data-*` 属性を書くだけで動く**機能の使い方は
`docs/guides/animation.md`（アニメーション機能ガイド）を参照してください
（本ガイドでは扱いません）。

設計判断の根拠（trait 境界の選定理由・no_std 評価・切り出し手順）を
知りたい場合は `docs/design/animation-core-architecture.md`（開発者向け
設計記録）を参照してください。本ガイドはその利用者向け API リファレンス
です。

## 1. 3 層構成のおさらい

```
fandhe-animation  ←  fandhe-frontend-animation  ←  wasm-full（optional 依存）
（演算のみ）          （Web アダプタ）               （配線）
```

- **`fandhe-animation`**: イージング・補間・spring・keyframes・timeline
  の**計算のみ**を担う。外部依存ゼロ・`#![forbid(unsafe_code)]`。将来
  別リポジトリへ切り出される前提のため、DOM・requestAnimationFrame・
  Web Animations API には一切触れません。
- **`fandhe-frontend-animation`**: `fandhe-animation` が定義する
  `Driver`/`Target` 等の trait を、wasm-bindgen / web-sys / js-sys を
  使って requestAnimationFrame・DOM・Web Animations API 上に実装する層
  です。`fandhe-frontend-wasm-full`/`-wasm-client`/`-wasm-thin` のいずれ
  にも依存しません。
- **`wasm-full`**: `fandhe-frontend-animation` を optional 依存として
  取り込み、型を再公開する配線層です（§4 参照）。

依存方向は一方向（`fandhe-animation ← fandhe-frontend-animation ←
wasm-full`）で、逆方向の依存はありません。

## 2. fandhe-animation（プラットフォーム非依存）

### 2.1 easing

CSS 相当の決定論的イージング関数を提供します。

- `CubicBezier`: CSS `cubic-bezier(x1, y1, x2, y2)` 相当。`EASE` /
  `EASE_IN` / `EASE_OUT` / `EASE_IN_OUT` の定数を持ちます。
  `CubicBezier::new(x1, y1, x2, y2)` は `x1`/`x2` が `[0, 1]` の範囲外、
  またはいずれかの引数が非有限値のとき `None` を返します。
- `Steps`: CSS `steps(count, position)` 相当。`StepPosition` は
  `JumpStart` / `JumpEnd` / `JumpNone` / `JumpBoth` の 4 種類です。
- `Easing`: 上記 2 種類と `Linear` をまとめた enum です。

```rust
use fandhe_animation::easing::{CubicBezier, Easing};

let ease_in_out = CubicBezier::new(0.42, 0.0, 0.58, 1.0).expect("有効な制御点");
let y = ease_in_out.evaluate(0.5); // t = 0.5 時点の進捗値

let easing = Easing::CubicBezier(ease_in_out);
assert_eq!(easing.evaluate(0.5), y);
```

### 2.2 Interpolate と値型

`Interpolate` trait は「2 値と進捗 `t` から補間値を返す」契約です。
`t` は `[0, 1]` に clamp されず、範囲外の `t` は外挿として扱われます
（spring・オーバーシュート系イージングが範囲外の `t` を渡すため）。

対応する値型:

- `f64` / `f32`: そのまま線形補間。
- `Vec2` / `Vec3` / `Vec4`: 成分ごとに線形補間。
- `Rgba`: 成分ごとに線形補間（premultiplied alpha・oklab 等の色空間指定
  はスコープ外）。
- `Quat`（単位四元数）: **slerp**（球面線形補間）。回転角が大きいほど
  角速度が一定になる点が nlerp（成分ごと線形）と異なります。入力が
  ほぼ同一の場合は数値的に不安定な slerp を避け nlerp + 正規化へ自動
  フォールバックします。
- `Mat4`（4x4 行列）: 成分ごと線形補間です。**回転を含む行列補間は正確
  ではありません**（180° 回転同士の中点がゼロ行列へ潰れる等）。回転を
  伴う用途は `Vec3`（translate/scale）+ `Quat` を個別に補間してから
  合成することを推奨します。

```rust
use fandhe_animation::interpolate::{Interpolate, Vec3};

let from = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
let to = Vec3 { x: 10.0, y: 20.0, z: 30.0 };
let mid = from.interpolate(&to, 0.5);
assert_eq!(mid, Vec3 { x: 5.0, y: 10.0, z: 15.0 });
```

### 2.3 spring

物理ベースのばねソルバです（motion.dev `spring()` 相当）。時間単位は
**秒**で統一されています。

- `SpringConfig`: `stiffness` / `damping` / `mass` の物理パラメータ。
  `Default` は motion.dev の既定値（`stiffness = 100.0` /
  `damping = 10.0` / `mass = 1.0`）です。
- `SpringConfig::from_duration_bounce(duration, bounce, velocity, mass)`:
  duration（秒）/ bounce（`0..=1`、1 に近いほど大きくオーバーシュート）
  から物理パラメータへ変換します。数値的に発散した場合は `Default` へ
  fail-safe にフォールバックします。
- `Spring::new(config, from, to, initial_velocity) -> Option<Spring>`:
  パラメータが有効範囲外・非有限値のときは `None` を返します（ライブ
  ラリコードで panic しません）。
- `Spring::at(t) -> SpringState { value, velocity, done }`: 時刻 `t`
  （秒）における値・速度・収束済みかどうかを返します。
- `Spring::settle_duration() -> f64`: `done` になる最初の時刻（秒）を
  探索して返します。

```rust
use fandhe_animation::spring::{Spring, SpringConfig};

let spring = Spring::new(SpringConfig::default(), 0.0, 100.0, 0.0)
    .expect("既定値は常に有効なパラメータ");
let state = spring.at(0.5);
assert!(!state.done);
let settle = spring.settle_duration();
assert!(spring.at(settle).done);
```

### 2.4 keyframes

オフセット付きキーフレーム列の補間です（Motion の `animate()` 配列
キーフレーム `[0, 1, 0]` + `ease: [...]` 相当）。

- `Keyframe<T> { offset, value }`: `offset` は `0.0..=1.0` に正規化
  された時刻です。
- `Keyframes::new(frames, easings)` / `Keyframes::evenly(values, easings)`:
  `Result<Keyframes<T>, KeyframesError>` を返します。`easings` が空なら
  全区間 `Easing::Linear` を補います。それ以外は長さが区間数
  （`frames.len() - 1`）と一致していなければなりません。
- `Keyframes::at(t) -> T`（`T: Clone`）: `t` は `[0, 1]` へ clamp され、
  区間外（先頭 offset 未満・末尾 offset 超過）は端点の値を保持します
  （Web Animations API の fill 挙動と同じ）。

```rust
use fandhe_animation::easing::Easing;
use fandhe_animation::keyframes::Keyframes;

let kf = Keyframes::evenly(vec![0.0, 10.0, 0.0], vec![Easing::Linear, Easing::Linear])
    .expect("3 値・2 easing は有効な構成");
assert_eq!(kf.at(0.0), 0.0);
assert_eq!(kf.at(0.5), 10.0);
assert_eq!(kf.at(1.0), 0.0);
```

### 2.5 timeline / stagger

`Stagger` は複数要素へ一律の遅延パターン（Motion の `stagger()` 相当）
を与える純関数、`Timeline` は複数セグメント（開始時刻・duration・対象
値）をラベル付きで束ね、任意時刻 `t` における各セグメントの進行度を
返します。いずれも時間単位は秒です。

```rust
use fandhe_animation::timeline::{At, Stagger, Timeline};

let stagger = Stagger::new(0.5);
let mut tl: Timeline<usize> = Timeline::new();
for i in 0..3 {
    tl.add_at(i, 1.0, At::Absolute(stagger.delay(i, 3))).unwrap();
}

// 各要素は 0.5 秒ずつずれて開始する。
assert_eq!(tl.segments()[0].start, 0.0);
assert_eq!(tl.segments()[1].start, 0.5);
assert_eq!(tl.segments()[2].start, 1.0);

// t = 1.0 時点では要素 0 は完了・要素 2 は開始直後。
let progress: Vec<f64> = tl.progress_at(1.0).map(|(p, _)| p).collect();
assert_eq!(progress, vec![1.0, 0.5, 0.0]);
```

`At` は `Absolute`（絶対時刻）・`AfterPrevious`（直前セグメント終了時刻
からの相対）・`WithPrevious`（直前セグメント開始時刻からの相対）・
`Label`（ラベル参照）の 4 種類です。`Timeline::label`/`label_time` で
名前付きの時刻マーカーを扱えます。

### 2.6 Driver / Target trait

`Driver` は「フレームループから tick を受け取る」trait、`Target<T>` は
「計算結果 `T` の書き込み先」を抽象化する trait です。いずれも本
クレート自体はフレームループ・DOM への書き込みを持たず、実装を
`fandhe-frontend-animation`（§3）へ委ねます。

- `Driver::tick(&mut self) -> Option<f64>`: 直前の呼び出しからの経過秒
  （pull 型。呼び出し側が毎フレーム 1 回呼ぶ）。供給が尽きた・まだ計測を
  開始していない等の理由で経過時間を返せない場合は `None` を返します。
- `Target<T>::write(&mut self, value: T)`: 計算済みの値を書き込みます。
  毎フレーム呼ばれる想定のため panic しない・エラーを返さないことが
  前提です。

テストでの駆動には `ManualDriver`（あらかじめ積んだ delta 列を FIFO で
払い出す参照実装）と `RecordingTarget`（書き込まれた値を到着順に蓄積
する参照実装）が使えます。crate 外（`fandhe-frontend-animation` 等）の
テストコードから利用するには `fandhe-animation` の `test-utils` feature
を有効にします。

```toml
[dev-dependencies]
fandhe-animation = { version = "0.1", features = ["test-utils"] }
```

## 3. fandhe-frontend-animation（Web アダプタ）

### 3.1 RafDriver + AnimationLoop

`RafDriver` は `window.performance.now()` の差分（秒）を返す `Driver`
実装です。`RafDriver::new() -> Option<Self>` はブラウザ環境（wasm32
ターゲットかつ `window`/`window.performance` が取得できる環境）でのみ
`Some` を返し、それ以外（native/SSR）では panic せず `None` を返します。

`AnimationLoop::start(step: impl FnMut() -> bool + 'static) -> Self` は
`requestAnimationFrame` で `step` を毎フレーム呼び続け、`step` が
`false` を返したら自動停止するループ補助です。**戻り値の
`AnimationLoop` を保持し続ける必要があります**（`Drop` で自動的に
`stop()` されるため、変数を drop するとループが止まります）。

```rust,ignore
use fandhe_frontend_animation::raf_driver::{AnimationLoop, RafDriver};
use fandhe_animation::driver::Driver;

let mut driver = RafDriver::new().expect("ブラウザ環境");
// `loop_` を保持し続ける（drop すると Drop::drop 経由で自動停止する）。
let loop_ = AnimationLoop::start(move || {
    let Some(_dt) = driver.tick() else {
        return true; // 初回 tick は基準時刻の記録のみ
    };
    // ここで spring.at(t) 等を計算し Target へ書き込む
    true // 継続するなら true、止めるなら false
});
```

### 3.2 DomTarget

`DomTarget` は `HtmlElement` の style へ計算値を書き込む `Target<f64>`
実装です。

- `DomTarget::style_property(element, name, unit)`: 標準 CSS プロパティ
  （例: `opacity`）へ書き込みます。`unit` は値へ付与する単位
  （`"px"`/`"deg"` 等）で、単位不要なプロパティは空文字を渡します。
- `DomTarget::custom_property(element, name)`: CSS カスタムプロパティ
  （例: `--fandhe-motion-progress`）へ書き込みます。

いずれも `CSSStyleDeclaration.setProperty(name, value)`（2 引数 API）
のみを使うため、`;` 等で追加の CSS 宣言を注入する経路を構造的に持ち
ません（§5 参照）。

```rust,ignore
use fandhe_frontend_animation::dom_target::DomTarget;
use fandhe_animation::target::Target;

let mut target = DomTarget::style_property(element, "opacity", "");
target.write(0.5); // element.style.opacity = "0.5"

let mut translate_x = DomTarget::style_property(element2, "translate", "px");
translate_x.write(120.0); // element2.style.translate = "120px"
```

### 3.3 統合例: spring を実 DOM で駆動する

`RafDriver`（§3.1）・`DomTarget`（§3.2）・`Spring`（§2.3）を組み合わせ
ると、実 DOM 上で spring アニメーションを駆動できます。

```rust,ignore
use fandhe_animation::driver::Driver;
use fandhe_animation::spring::{Spring, SpringConfig};
use fandhe_animation::target::Target;
use fandhe_frontend_animation::dom_target::DomTarget;
use fandhe_frontend_animation::raf_driver::{AnimationLoop, RafDriver};
use web_sys::HtmlElement;

/// `element` の `opacity` を 0 → 1 へ spring で駆動するループを開始する。
/// 戻り値の `AnimationLoop` を呼び出し側が保持し続けること。
fn start_fade_in(element: HtmlElement) -> AnimationLoop {
    let spring = Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0)
        .expect("既定値は常に有効なパラメータ");
    let mut driver = RafDriver::new().expect("ブラウザ環境");
    let mut target = DomTarget::style_property(element, "opacity", "");
    let mut elapsed = 0.0_f64;

    AnimationLoop::start(move || {
        let Some(dt) = driver.tick() else {
            return true; // 初回 tick は基準時刻の記録のみ
        };
        elapsed += dt;
        let state = spring.at(elapsed);
        target.write(state.value);
        !state.done // 収束したら停止する
    })
}
```

より詳細な結合例は `crates/frontend-animation/tests/spring_via_raf_dom_browser.rs`
（実ブラウザでの動作確認テスト）を参照してください。

### 3.4 animate()（WAAPI 薄いラッパ）

`element.animate()`（Web Animations API、以下 WAAPI）の薄いラッパです
（wasm32 ターゲット限定）。

- `WaapiKeyframe { offset, easing, properties }`: WAAPI へ渡す 1
  keyframe です。
- `AnimateOptions { duration_ms, easing, fill, iterations }`:
  `element.animate()` の第 2 引数に相当します。**`duration_ms` は
  ミリ秒**です（`fandhe-animation` 全体が秒単位で統一されている点と
  異なるので注意してください）。
- `easing_to_css(easing: Easing) -> String`: `fandhe-animation::easing`
  は CSS 文字列を一切生成しない不変条件を持つため、CSS easing 文字列
  （`"cubic-bezier(...)"`/`"steps(...)"` 等）への変換はこの関数が担い
  ます。native（非 wasm32）からも呼び出せます。
- `keyframes_to_waapi(keyframes, property, to_css_value) -> Vec<WaapiKeyframe>`:
  `Keyframes<T>` を単一 CSS プロパティの WAAPI keyframes 列へ変換し
  ます。`Keyframes::at` が区間外を端点値で保持する契約と、WAAPI が
  offset 0/1 の keyframe 欠落を要素の「underlying value」で補完する
  仕様との差を埋めるため、先頭が offset 0 でない・末尾が offset 1 で
  ない入力は端点値を保持したまま offset 0/1 の keyframe を自動的に
  補います。
- `animate(element, keyframes, options) -> Result<AnimationHandle, JsValue>`
  （wasm32 限定）: `element.animate(keyframes, options)` を呼び出します。
- `AnimationHandle::finished(&self) -> Result<(), JsValue>`（wasm32
  限定、`async`）: WAAPI `Animation.finished` Promise を待ちます。

```rust
use fandhe_animation::easing::{CubicBezier, Easing};
use fandhe_animation::keyframes::{Keyframe, Keyframes};
use fandhe_frontend_animation::animate::{easing_to_css, keyframes_to_waapi};

let keyframes = Keyframes::new(
    vec![
        Keyframe { offset: 0.0, value: 0.0_f64 },
        Keyframe { offset: 1.0, value: 1.0_f64 },
    ],
    vec![Easing::CubicBezier(CubicBezier::EASE_IN)],
)
.unwrap();

let waapi = keyframes_to_waapi(&keyframes, "opacity", |v| v.to_string());
assert_eq!(waapi.len(), 2);
assert_eq!(
    waapi[0].easing.as_deref(),
    Some(easing_to_css(Easing::CubicBezier(CubicBezier::EASE_IN)).as_str())
);
```

```rust,ignore
use fandhe_frontend_animation::animate::{animate, AnimateOptions};

let handle = animate(&element, &waapi, &AnimateOptions {
    duration_ms: 300.0,
    easing: None, // 各 keyframe の easing を使うため全体既定は指定しない
    fill: Some("forwards".to_string()),
    iterations: Some(1.0),
})?;
handle.finished().await?;
```

## 4. wasm-full から利用する（fandhe-frontend 固有部分）

`fandhe-animation`・`fandhe-frontend-animation` は `fandhe-frontend-
wasm-full` に依存しない独立クレートですが、`wasm-full` は optional
依存としてこれらを取り込み、直接依存を追加しなくても型へ到達できる
配線点を提供します（依存方向は不変: `fandhe-animation ←
fandhe-frontend-animation ← wasm-full(optional)`）。

### 4.1 feature

- **`animation-driver`**（既定 on）: `dep:fandhe-frontend-animation` を
  有効化し、`animation_driver` モジュール（`RafDriver`/`DomTarget`/
  `AnimationLoop` の薄い再公開）を公開します。`Runtime::mount`/`hydrate`
  からの新規呼び出しは伴いません（型の再公開のみ）。
- **`animate`**（既定 on）: `fandhe-frontend-animation` crate 自体
  （`fandhe_animation` への到達路を含む）を再公開します。

いずれも `crates/dist-server/src/wasm_dist_features.rs` の
`WASM_DIST_FEATURES`（`fandhe-frontend-dist-server` が配布する最小
インタラクティブ構成 6 feature）には含まれていません。つまり、
`fandhe-frontend-dist-server` 経由の配布 WASM にはこれらの機能は
出荷されません（`docs/guides/wasm-full-features.md` を参照）。

`default-features = false` を使う場合は、これらの機能を維持するために
`features` 配列へ明示的に `"animation-driver"`/`"animate"` を含める
必要があります（詳細・完全な feature 列挙は
`docs/guides/wasm-full-features.md` を参照）。

### 4.2 到達パス

wasm-full への依存だけで、次の 2 経路から到達できます（自分のアプリで
`fandhe-frontend-animation` へ直接依存を追加する必要はありません）。

```rust,ignore
// animation-driver feature 経由（RafDriver/DomTarget/AnimationLoop）
use fandhe_frontend_wasm_full::animation_driver::{AnimationLoop, DomTarget, RafDriver};

// animate feature 経由（animate() WAAPI ラッパ、fandhe-animation の型も
// ここから到達できる）
use fandhe_frontend_wasm_full::fandhe_frontend_animation::animate::{animate, AnimateOptions};
use fandhe_frontend_wasm_full::fandhe_frontend_animation::fandhe_animation::spring::{Spring, SpringConfig};
```

### 4.3 data-* 配線との対応関係

**現状（2026-09 時点）: rAF Driver / DOM Target / WAAPI を `data-*` 属性
から自動的に駆動する宣言的配線は存在しません。** `animation_driver`
モジュールは型の再エクスポートのみを行い、`wire_*` 相当の自動呼び出し
を持ちません。

既存の `in_view`/`gesture`/`stagger_index` 等の `data-*` 配線（宣言的
機能側）は IntersectionObserver・Pointer Events 由来の状態を `data-*`
へ書き戻すのみで、`fandhe-animation`/`fandhe-frontend-animation` には
一切依存しない別系統です。本ガイドの API（§2/§3/§4.1/§4.2）と混同しな
いよう注意してください。

今後 `data-*` 属性からの自動配線（stagger 適用・`animate()` 自動
トリガー等）が追加される場合の実装先は親トラッキング #2508 配下です。
宣言的な使い方は `docs/guides/animation.md`（アニメーション機能ガイド）を
参照してください。

## 5. セキュリティ上の注意

- `DomTarget` は `CSSStyleDeclaration.setProperty` の 2 引数 API のみを
  使い、プロパティ名・単位は呼び出し側 Rust コードの固定値です。信頼
  できない DOM 文字列（`data-*` 属性の値等）をそのままプロパティ名に
  使う経路は持ちません。
- `animate()` の `WaapiKeyframe.properties`/`AnimateOptions.easing` は、
  本ガイドの型付き API（`easing_to_css`・呼び出し側の `to_css_value`）
  を経由した信頼できる値を渡してください。未検証の外部入力文字列を
  そのまま渡さないでください。

## 6. 関連ドキュメント

- `docs/design/animation-core-architecture.md` — 3 層構成の設計根拠・
  trait 境界・no_std 評価・切り出し手順（開発者向け）
- `docs/design/motion-reference-adoption-policy.md` §6 — Motion 参照
  方針・3 層構成/ゼロコスト方針の決定記録
- `docs/guides/animation.md`（アニメーション機能ガイド） — 宣言的
  `data-*` 配線の使い方（pre-styled-ui の presence/keyframes/stagger・
  wasm-full の in-view/View Transitions/FLIP 等）
- [pre-styled-ui motion feature ガイド](./pre-styled-ui-motion-feature.md)
- [wasm-full feature 選択ガイド](./wasm-full-features.md)
