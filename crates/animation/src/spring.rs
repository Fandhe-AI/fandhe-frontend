//! 物理ベースのばね（spring）ソルバ（motion.dev `spring()` 相当）。
//!
//! 減衰調和振動子の解析解として、任意時刻 `t`（秒）における値・速度を
//! 決定的に計算する。フレームループ・DOM への書き込みは持たない
//! （`fandhe-frontend-animation` の rAF ループ、イシュー #2378/#2403 が
//! 本モジュールの [`Spring::at`] を毎フレーム呼び出す消費者になる想定。
//! CSS `linear()` プリセット生成〔#2381〕は [`Spring::at`] のサンプル列を
//! `easing::sample` へ渡す下流消費者）。
//!
//! 時間単位は秒（`f64`）で統一する（`animation-core-architecture.md` §2.1
//! の `Driver::tick` 契約に合わせる。motion.dev は ms 単位だが本 crate は
//! 秒に統一する）。

/// ばねの物理パラメータ（motion.dev `spring()` の既定値 stiffness=100 /
/// damping=10 / mass=1 を [`Default`] に採用）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
        }
    }
}

/// duration/bounce 変換の Newton 法反復回数。motion.dev `findSpring` の
/// 実測収束回数に対し十分な余裕を持たせた固定回数（DoS 対策として上限を
/// 設ける。固定回数のため入力に関わらず必ず終了する）。
const NEWTON_ITERATIONS: u32 = 12;

/// 包絡線が減衰したとみなす比率（motion.dev `findSpring` の `safeMin` 相当）。
/// 0.001 は「初期振幅の 0.1% まで減衰した時刻を duration とみなす」という
/// motion.dev のヒューリスティックをそのまま踏襲する。
const SAFE_MIN: f64 = 0.001;

impl SpringConfig {
    /// duration（秒）/ bounce（0..=1、1 に近いほど大きくオーバーシュートする）
    /// から物理パラメータへ変換する（motion.dev `findSpring` 相当）。
    ///
    /// - `duration` は `[0.01, 10.0]` 秒へ clamp する
    /// - `bounce` から減衰比 `ζ = clamp(1 - bounce, 0.05, 1.0)` を得る
    /// - Newton 法で `ω0` を求め、`stiffness = ω0²·mass` /
    ///   `damping = ζ·2·sqrt(mass·stiffness)` を返す
    /// - 数値的に発散した場合（NaN 等）は [`SpringConfig::default`] へ
    ///   フォールバックする（fail-safe。パニックしない）
    pub fn from_duration_bounce(duration: f64, bounce: f64, velocity: f64, mass: f64) -> Self {
        // duration/bounce が非有限（NaN 等）なら Newton 法へ進まず既定値へ
        // フォールバックする（velocity/mass は既定 0/1 で代替可能な補助
        // パラメータのため個別に clamp する）。
        if !duration.is_finite() || !bounce.is_finite() {
            return Self::default();
        }
        let duration = duration.clamp(0.01, 10.0);
        let mass = if mass.is_finite() && mass > 0.0 {
            mass
        } else {
            1.0
        };
        let zeta = (1.0 - bounce).clamp(0.05, 1.0);
        let velocity = if velocity.is_finite() { velocity } else { 0.0 };

        let omega0 = solve_omega0(duration, zeta, velocity);
        match omega0 {
            Some(omega0) if omega0.is_finite() && omega0 > 0.0 => {
                let stiffness = omega0 * omega0 * mass;
                // `(mass * stiffness).sqrt()` は極端な mass（例: 1e-200）で
                // 積が浮動小数点の下限を割ってゼロへ丸まり、damping=0 を
                // 誤って返しうる（イシュー #2426 レビュー指摘）。
                // sqrt(a·b) = sqrt(a)·sqrt(b) に分解し、それぞれを先に
                // sqrt してから掛けることで積を直接作らず中間アンダー
                // フロー（過大な mass では逆にオーバーフロー）を避ける。
                let damping = zeta * 2.0 * mass.sqrt() * stiffness.sqrt();
                if stiffness.is_finite() && damping.is_finite() && damping >= 0.0 {
                    return Self {
                        stiffness,
                        damping,
                        mass,
                    };
                }
                Self::default()
            }
            _ => Self::default(),
        }
    }

    /// 減衰比 `ζ = damping / (2·sqrt(stiffness·mass))`。
    ///
    /// `2.0 * self.stiffness.sqrt() * self.mass.sqrt()` を先に計算して
    /// から割ると、両 sqrt が揃って巨大（例: stiffness=mass=1e308）な
    /// ケースでその積自体が Inf に丸まり、本来有限の ζ を 0 だと偽って
    /// 返しうる（イシュー #2426 レビュー指摘）。掛け算をまとめず逐次
    /// 除算にすることで、割るたびに値を小さくしてから次の除算へ進み、
    /// 中間結果が Inf になる経路を作らない。
    pub fn damping_ratio(&self) -> f64 {
        self.damping / 2.0 / self.stiffness.sqrt() / self.mass.sqrt()
    }
}

/// `envelope(ω) = SAFE_MIN` の根を Newton 法で探索し `ω0` を返す。
///
/// ζ < 1（不足減衰）と ζ == 1（臨界減衰）で包絡線の式が異なるため分岐する
/// （§3.4 参照）。過減衰（ζ > 1）は `from_duration_bounce` の zeta clamp
/// 上限が 1.0 のため到達しない。
fn solve_omega0(duration: f64, zeta: f64, velocity: f64) -> Option<f64> {
    let mut omega = 5.0 / duration;
    // 直近反復の残差 |f(omega)|。固定回数（`NEWTON_ITERATIONS`）で
    // 打ち切った時点で収束していない場合に検出するため、ループ末尾で
    // 参照できるよう外側に保持する（初期値は「未収束」とみなされる
    // 大きな値にしておく）。
    let mut last_residual = f64::INFINITY;
    for _ in 0..NEWTON_ITERATIONS {
        if !omega.is_finite() || omega <= 0.0 {
            return None;
        }
        let (f, df) = if (zeta - 1.0).abs() < 1e-9 {
            // ζ == 1（臨界減衰）:
            // envelope(ω) = -SAFE_MIN + e^{-ωT}((ω - v)T + 1)
            let e = (-omega * duration).exp();
            let f = -SAFE_MIN + e * ((omega - velocity) * duration + 1.0);
            let df = e * (velocity - omega) * duration * duration;
            (f, df)
        } else {
            // ζ < 1（不足減衰）:
            // 単位変位（x0=1）の解に対する振幅包絡線は
            // |x(t)| <= sqrt(x0² + b²)·e^{-ζωt}（b は sin 成分の係数）で
            // 上から抑えられる。b だけを包絡線として使うと x0² 分（余弦
            // 成分の寄与）を欠落し偽の根を選びうる（速度がちょうど
            // ζω に近い入力で b ≈ 0 になり、実際は全く減衰していない
            // t を「収束済み」と誤認する。イシュー #2426 レビュー指摘）ため
            // sqrt(1 + b²) を包絡線として使う。
            // envelope(ω) = SAFE_MIN - sqrt(1 + b²)·e^{-ζωT}
            //   where b = (ζω - v) / (ω·sqrt(1-ζ²))
            let omega_d_ratio = (1.0 - zeta * zeta).sqrt();
            let e = (-zeta * omega * duration).exp();
            let b = (zeta * omega - velocity) / (omega * omega_d_ratio);
            let r = (1.0 + b * b).sqrt();
            let f = SAFE_MIN - r * e;
            // 数値微分（解析導関数は式が煩雑になるため、固定小刻みの中心差分で
            // 代替する。Newton 法は導関数の厳密性を必要としないため十分）。
            let h = omega * 1e-6 + 1e-9;
            let eval = |w: f64| -> f64 {
                let e = (-zeta * w * duration).exp();
                let b = (zeta * w - velocity) / (w * omega_d_ratio);
                let r = (1.0 + b * b).sqrt();
                SAFE_MIN - r * e
            };
            let df = (eval(omega + h) - eval(omega - h)) / (2.0 * h);
            (f, df)
        };
        if !f.is_finite() || !df.is_finite() || df == 0.0 {
            return None;
        }
        last_residual = f;
        omega -= f / df;
    }
    // 固定回数のみを保証する Newton 法は、収束が遅い入力（例:
    // `from_duration_bounce(0.5, 0.9, 1e6, 1.0)` のような極端な
    // velocity）で `NEWTON_ITERATIONS` 回に達しても収束しないまま
    // omega を返しうる（イシュー #2426 レビュー指摘）。omega の有限性・
    // 正値だけでは未収束を検知できないため、最終反復の残差を成功条件へ
    // 加える。`f`（envelope 関数値）は SAFE_MIN=0.001 前後のスケールで
    // 評価されるため、その 1/10 を許容残差とする。
    const RESIDUAL_TOLERANCE: f64 = SAFE_MIN / 10.0;
    if omega.is_finite() && omega > 0.0 && last_residual.abs() <= RESIDUAL_TOLERANCE {
        Some(omega)
    } else {
        None
    }
}

/// 時刻 `t` におけるばねのサンプル値。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringState {
    pub value: f64,
    pub velocity: f64,
    pub done: bool,
}

/// `settle_duration` の探索上限（秒）。10 s 経過しても収束しない設定は
/// 実用上ありえないため、無限ループ防止のための固定上限として置く。
const MAX_SETTLE_DURATION: f64 = 10.0;
/// `settle_duration` の探索刻み幅（秒）。motion.dev `calcGeneratorDuration`
/// の 10 ms 刻みより細かくし、#2381 が `linear()` プリセットを生成する際の
/// サンプリング精度を確保する。
const SETTLE_STEP: f64 = 0.001;

/// `from`/`to`/`initial_velocity` の絶対値上限。
///
/// この上限と [`MIN_PARAM`]/[`MAX_PARAM`] の組み合わせにより、`at()` の
/// 全中間演算（`decay*x0*cos` 等の振幅計算・`self.to + x` の最終加算）が
/// 構成的にオーバーフローしないことを保証する（f64 上限 `1.8e308` に対し
/// 十分な余裕を残す値。詳細は [`MIN_PARAM`] のコメント参照）。
const MAX_ABS_VALUE: f64 = 1e15;

/// `stiffness`/`damping`/`mass` の下限。
///
/// `damping < MIN_PARAM`（`damping == 0.0` を含む）は無減衰・過小減衰の
/// ばねとなり、`at()` が理論上永遠に振動し続け収束判定が成立しない
/// （イシュー #2426 レビュー指摘の根本原因: 無減衰の入力は「静止しない」
/// ため settle 契約と矛盾し、`omega_d * t` が `t` の増大に伴い無制限に
/// 大きくなる唯一の経路でもある）ため、`Spring::new` はこの下限で
/// 一括して拒否する。
///
/// 境界値の選定根拠: 不足減衰領域での減衰係数 `ζω0 = damping / (2·mass)`
/// は `damping >= MIN_PARAM`・`mass <= MAX_PARAM` のとき
/// `>= MIN_PARAM / (2·MAX_PARAM) = 5e-16` 以上を保つ。`decay = e^{-ζω0·t}`
/// は指数部が `-745` を下回ると f64 で厳密に `0.0` へ丸まるため、
/// `t <= 745 / 5e-16 ≈ 1.5e18` の時点までに `at()` は decay=0 の早期
/// 打ち切り経路（本ファイル内の既存分岐）へ到達する。この上限内では
/// `omega_d <= sqrt(MAX_PARAM / MIN_PARAM) ≈ 3.2e7` のため
/// `omega_d * t <= 3.2e7 * 1.5e18 ≈ 4.7e25` に収まり、`sin_cos` へ非有限値が
/// 渡ることはない。振幅項（`x0`・不足減衰の `b` 係数）も
/// `MAX_ABS_VALUE`・`MIN_PARAM`/`MAX_PARAM` の組み合わせで最大でも
/// 概算 `2e34` 程度に収まり（`b = (v0 + ζω0·x0) / ωd` は `ωd` が臨界減衰
/// 境界近傍〔`|ζ-1|` が `1e-9` 未満は `Region::Critical` へ分岐するため
/// 到達しない〕でも下限 `1.4e-12` 程度を保つ）、`f64::MAX`（`1.8e308`）に
/// 対し十分な余裕を残す。
///
/// `MAX_PARAM` は `1e6` ではなく `1e9` とする（イシュー #2426 レビュー
/// 指摘 2 件目: `SpringConfig::from_duration_bounce` は文書化された
/// `duration ∈ [0.01, 10.0]`・`bounce ∈ [0, 1]` の全域で `velocity=0`・
/// `mass=1.0` の典型入力においても `stiffness` が最大 `1.91e8` 程度に
/// 達する〔`duration=0.01`〔下限〕・`bounce≈0.95` 付近で最大化〕ため、
/// 旧上限 `1e6` では変換結果を `Spring::new` へそのまま渡せない入力域が
/// 生じていた。`mass` は本関数のドキュメントに明示的な数値域を持たず
/// `stiffness` に線形寄与するため任意の上限を選んでも際限なく超過させ
/// うるが、典型的な物理質量（`mass ∈ [0.01, 2.0]` 程度、`Default` の
/// `mass: 1.0` を中心とする範囲）まで含めても最悪値は `3.82e8` 程度に
/// 収まる。`1e9` はこの実測最大値に 2 倍強の余裕を持つ値である）。
const MIN_PARAM: f64 = 1e-6;

/// `stiffness`/`damping`/`mass` の上限（[`MIN_PARAM`] 参照）。
const MAX_PARAM: f64 = 1e9;

/// 減衰領域（判別式 `ζ` の符号）ごとの事前計算済み係数。
///
/// 振幅係数（`b`/`c`/`overdamped` の `a`,`b`）は `x0`/`v0` から一度だけ
/// 定まる（`t` に依存しない）ため、`at()` を呼ぶたびに再計算せず
/// `Spring::new()` 時点で確定させる。これにより (1) 呼び出しごとの
/// 再計算コストを避けつつ、(2) 極端な `x0`（例: `from`/`to` の差が
/// 1e308 級）でオーバーフローする組み合わせを構築時に検出できる
/// （イシュー #2426 レビュー指摘: `at()` 側で NaN/Inf を返すのではなく
/// `Spring::new()` が `None` を返すべき）。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Region {
    /// 不足減衰（ζ < 1）: 振動しながら収束する。`b` は sin 成分の振幅係数
    /// （`(v0 + zeta_omega0*x0) / omega_d`）。
    Underdamped {
        omega_d: f64,
        zeta_omega0: f64,
        b: f64,
    },
    /// 臨界減衰（ζ == 1、許容誤差 1e-9）: 振動せず最速に収束する。`c` は
    /// `v0 + omega0*x0`。
    Critical { omega0: f64, c: f64 },
    /// 過減衰（ζ > 1）: 2 つの実数指数の和で振動せず収束する。`a`/`b` は
    /// 初期条件 `x(0)=x0, x'(0)=v0` を解いた振幅係数。
    Overdamped { r1: f64, r2: f64, a: f64, b: f64 },
}

/// from → to へ向かうばね 1 本の解析解ソルバ。
///
/// 値は単位非依存の `f64`（px でも進捗 0..=1 でも可）。不変・`Copy` 可・
/// 同一入力に対し常にビット一致する決定的な出力を返す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    to: f64,
    x0: f64,
    region: Region,
    rest_delta: f64,
    rest_speed: f64,
}

impl Spring {
    /// `stiffness`/`damping`/`mass` が [`MIN_PARAM`]..=[`MAX_PARAM`]
    /// の範囲外（`damping == 0.0` の無減衰も含む）・`from`/`to`/
    /// `initial_velocity` の絶対値が [`MAX_ABS_VALUE`] を超える・
    /// いずれかが非有限値のいずれかを満たす場合は `None` を返す
    /// （ライブラリコードで panic しない）。この入力ドメインの境界化に
    /// より、`at()` 内部の全中間演算が構成的にオーバーフローしないことを
    /// 保証する（イシュー #2426 レビュー指摘: `at()` 側の非有限フォール
    /// バックで収束扱いにするのではなく、構築時に拒否すべき）。
    pub fn new(config: SpringConfig, from: f64, to: f64, initial_velocity: f64) -> Option<Self> {
        let SpringConfig {
            stiffness,
            damping,
            mass,
        } = config;
        if !stiffness.is_finite()
            || !damping.is_finite()
            || !mass.is_finite()
            || !from.is_finite()
            || !to.is_finite()
            || !initial_velocity.is_finite()
        {
            return None;
        }
        if !(MIN_PARAM..=MAX_PARAM).contains(&stiffness)
            || !(MIN_PARAM..=MAX_PARAM).contains(&damping)
            || !(MIN_PARAM..=MAX_PARAM).contains(&mass)
        {
            return None;
        }
        if from.abs() > MAX_ABS_VALUE
            || to.abs() > MAX_ABS_VALUE
            || initial_velocity.abs() > MAX_ABS_VALUE
        {
            return None;
        }

        // `stiffness / mass` や `stiffness * mass` を直接計算すると、両者が
        // 極端な値（例: 1e200 同士）のとき中間結果がオーバーフローして
        // Inf・アンダーフローして 0 になり、ζ を偽って算出しうる
        // （例: stiffness=damping=mass=1e200 は本来 ζ=1 の臨界減衰だが、
        // stiffness*mass=1e400 が Inf に丸まり ζ=0 の無減衰振動と誤判定
        // される。イシュー #2426 レビュー指摘）。各項を先に sqrt してから
        // 掛ける／割ることで積・商を直接作らず中間結果の範囲を抑える。
        // ζ の分母も `2.0 * stiffness.sqrt() * mass.sqrt()` をまとめて
        // 計算すると、両 sqrt が揃って巨大（stiffness=damping=mass=1e308
        // 等）なとき積自体が Inf に丸まり ζ=0（無減衰振動）を誤って
        // 返す（`damping_ratio` と同じ不具合、イシュー #2426 レビュー
        // 指摘）。逐次除算にして中間結果を Inf にしない。
        let omega0 = stiffness.sqrt() / mass.sqrt();
        let zeta = damping / 2.0 / stiffness.sqrt() / mass.sqrt();
        // 上記の安定化後もなお非有限・非正な結果が出た場合（両方の入力が
        // sqrt 後もなお表現範囲を超える極端な値等）は、region 選択以降へ
        // 不正な値を伝播させずここで弾く。
        if !omega0.is_finite() || omega0 <= 0.0 || !zeta.is_finite() || zeta < 0.0 {
            return None;
        }
        let x0 = from - to;
        // `from`/`to` は個別には有限でも、両者が離れた極端な値（例:
        // from=1e308, to=-1e308）だと差が f64 の表現範囲を超えて Inf に
        // なりうる（イシュー #2426 レビュー指摘）。表現不能な変位を後段
        // （`at()` の decay*x0 等）へ伝播させず、ここで弾く。
        if !x0.is_finite() {
            return None;
        }
        let v0 = initial_velocity;

        let region = if (zeta - 1.0).abs() < 1e-9 {
            let c = v0 + omega0 * x0;
            if !c.is_finite() {
                return None;
            }
            Region::Critical { omega0, c }
        } else if zeta < 1.0 {
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let zeta_omega0 = zeta * omega0;
            let b = (v0 + zeta_omega0 * x0) / omega_d;
            if !b.is_finite() {
                return None;
            }
            Region::Underdamped {
                omega_d,
                zeta_omega0,
                b,
            }
        } else {
            // `(zeta * zeta - 1.0).sqrt()` は zeta が巨大（例:
            // stiffness=mass=1, damping=1e200 → zeta=5e199）だと
            // zeta*zeta 自体が Inf へオーバーフローし disc=Inf・r1/r2 が
            // NaN になる（イシュー #2426 レビュー指摘）。
            // disc = zeta·sqrt(1 - 1/zeta²) と変形すると、zeta² が
            // オーバーフローしても 1/Inf = 0 で sqrt(1) = 1 に丸まり
            // disc ≈ zeta（zeta 自体は有限）という正しい近似へフォール
            // バックする。
            let disc = zeta * (1.0 - 1.0 / (zeta * zeta)).sqrt();
            // r1 = -omega0·(zeta - disc) は zeta が大きいほど
            // zeta - disc が真の値（≈ 1/(2·zeta)）に対し桁落ちする
            // （例: stiffness=damping=1e18, mass=1 で r1 が -0 に丸まり
            // 減衰が exp(-t) から消える、イシュー #2426 レビュー指摘）。
            // zeta² - disc² = 1（定義上）を使い
            // zeta - disc = 1 / (zeta + disc) へ有理化することで、
            // 差の桁落ちを経由せず r1 を直接計算する。
            let r1 = -omega0 / (zeta + disc);
            // r2 = -omega0·(zeta + disc) は、r1・disc 単体は有限でも
            // omega0 と (zeta+disc) が共に巨大（例: stiffness=1,
            // damping=1e200, mass=1e-200 → omega0=1e100, zeta+disc=1e300）
            // だと積 1e400 が -Inf へオーバーフローする（イシュー #2426
            // レビュー指摘）。この場合の真の r2 はそもそも f64 の表現
            // 範囲を超えており安定化のしようがないため、非有限値を
            // 後段（`at()` の decay 計算）へ伝播させず構築時に拒否する。
            let r2 = -omega0 * (zeta + disc);
            if !r1.is_finite() || !r2.is_finite() {
                return None;
            }
            // 振幅係数 a は `(v0 - r2*x0) / (r1-r2)` の定義どおり。
            let a = (v0 - r2 * x0) / (r1 - r2);
            // b は `x0 - a`（差し引き）で求めると、a が x0 に近い値へ
            // 丸まる入力（例: stiffness=damping=1e18, mass=1 で a が -1
            // に丸まり b=0 になる）で桁落ちし、`at(0).velocity` が本来の
            // 初速度 v0 を再現しない（イシュー #2426 レビュー指摘）。
            // 初期条件 a+b=x0, a*r1+b*r2=v0 を b について直接解いた
            // `b = (r1*x0 - v0) / (r1-r2)` を使うことで、差し引きによる
            // 精度損失を経由せず b を求める。
            let b = (r1 * x0 - v0) / (r1 - r2);
            if !a.is_finite() || !b.is_finite() {
                return None;
            }
            Region::Overdamped { r1, r2, a, b }
        };

        // rest 閾値の 2 段選択（motion.dev のヒューリスティック）:
        // - 移動距離が 5 未満なら「小レンジ値」（opacity・scale・進捗 0..=1 等）
        //   とみなし granular 閾値を使う。0.5 の絶対差はレンジの半分に相当し
        //   大きすぎるため、1/100 スケールの閾値に切り替える
        //   （motion.dev の `isGranularScale` と同一の判定基準）
        // - それ以外は px 単位の DOM 移動を想定した既定閾値を使う:
        //   rest_delta=0.5px は 60fps でサブピクセル描画により視認できない
        //   差分、rest_speed=2px/s は 1 フレーム換算で約 0.03px の動きに相当し
        //   実用上静止とみなせる
        let (rest_delta, rest_speed) = if (to - from).abs() < 5.0 {
            (0.005, 0.01)
        } else {
            (0.5, 2.0)
        };

        Some(Self {
            to,
            x0,
            region,
            rest_delta,
            rest_speed,
        })
    }

    /// 時刻 `t`（秒）における値・速度・収束済みかを返す。
    ///
    /// `t` が負・NaN の場合は `t = 0.0` として扱う。収束判定は
    /// `|to - value| <= rest_delta && |velocity| <= rest_speed` の
    /// AND 条件（motion.dev と同一）。片方のみでは「目標を高速通過中」や
    /// 「遠方で静止中」を誤って完了扱いにしてしまうため両方を要求する。
    /// `done == true` のとき `value` は `to` に完全一致し `velocity` は
    /// `0.0` にスナップする（残差を後続フレームへ漏らさず、#2381 の
    /// `linear()` 末尾値が正確に 1 になる契約を満たすため）。
    pub fn at(&self, t: f64) -> SpringState {
        let t = if t.is_finite() && t > 0.0 { t } else { 0.0 };
        let (x, v) = match self.region {
            Region::Underdamped {
                omega_d,
                zeta_omega0,
                b,
            } => {
                let decay = (-zeta_omega0 * t).exp();
                // decay が 0 まで下振れした時点で以後は収束済み（x=v=0）として
                // 扱い、sin_cos への以後の計算を行わない。`t` が極端に大きい
                // 場合（例: t=1e308）は `omega_d * t` 自体が Inf へオーバー
                // フローし sin_cos(Inf) が NaN を返すため、`decay` が先に
                // 0 へ下振れするこの時点で打ち切ることで NaN の伝播を防ぐ
                // （イシュー #2426 レビュー指摘）。
                if decay == 0.0 {
                    (0.0, 0.0)
                } else {
                    let (sin, cos) = (omega_d * t).sin_cos();
                    // decay を先に係数へ掛けてから t 依存の三角関数と合成する
                    // （`decay * (self.x0 * cos + b * sin)` の順だと桁は変わら
                    // ないが、下記 Critical と同型の安定化のため統一する）。
                    let x = decay * self.x0 * cos + decay * b * sin;
                    // 解析微分: x' = -ζω0·x + decay·(-x0·ωd·sin + b·ωd·cos)
                    let v = -zeta_omega0 * x + decay * omega_d * (-self.x0 * sin + b * cos);
                    (x, v)
                }
            }
            Region::Critical { omega0, c } => {
                let decay = (-omega0 * t).exp();
                if decay == 0.0 {
                    (0.0, 0.0)
                } else {
                    // `self.x0 + c * t` を先に評価すると、`x0`/`c` が極端に
                    // 大きい値（例: 1e308）のとき decay 適用前に和が f64 の
                    // 表現範囲を超えて Inf になり、`decay * Inf` が Inf/NaN
                    // を返してしまう（イシュー #2426 レビュー指摘）。
                    // decay を先に `c` へ掛けてから `t` を掛けることで、
                    // 各中間項を decay 分だけ縮小してから合成し、最終的に
                    // 表現可能な値であれば途中でオーバーフローしない。
                    let x = decay * self.x0 + (decay * c) * t;
                    // 解析微分: x' = -ω0·decay·(x0 + c·t) + decay·c = -ω0·x + decay·c
                    let v = -omega0 * x + decay * c;
                    (x, v)
                }
            }
            Region::Overdamped { r1, r2, a, b } => {
                let e1 = (r1 * t).exp();
                let e2 = (r2 * t).exp();
                let x = a * e1 + b * e2;
                let v = a * r1 * e1 + b * r2 * e2;
                (x, v)
            }
        };
        // `Spring::new` の入力ドメイン境界化（[`MIN_PARAM`]/[`MAX_PARAM`]/
        // [`MAX_ABS_VALUE`]）により、ここに到達する時点で `x`/`v`/
        // `self.to + x` はいずれも構成的に有限であることが保証されている
        // （証明の要旨は [`MIN_PARAM`] のコメント参照）。以前はここで
        // 非有限値を収束済み（x=v=0）や `value = self.to` へ丸めていたが、
        // その丸めは「計算失敗」と「真の収束」を区別できず、収束判定
        // （`done`）を誤って成立させてしまう副作用があった（イシュー #2426
        // レビュー指摘）。デバッグビルドでのみ不変条件を検証し、リリース
        // ビルドでは値をそのまま用いる。
        debug_assert!(
            x.is_finite() && v.is_finite(),
            "at() produced non-finite x/v: x={x} v={v}"
        );
        let value = self.to + x;
        debug_assert!(value.is_finite(), "at() produced non-finite value: {value}");
        let done = (self.to - value).abs() <= self.rest_delta && v.abs() <= self.rest_speed;
        if done {
            SpringState {
                value: self.to,
                velocity: 0.0,
                done: true,
            }
        } else {
            SpringState {
                value,
                velocity: v,
                done: false,
            }
        }
    }

    /// `done` になる最初の時刻（秒）を `SETTLE_STEP` 刻みで探索する
    /// （`#2381` がサンプリング範囲を決定するために使う）。
    /// `MAX_SETTLE_DURATION` 到達まで収束しない場合はその上限値を返す
    /// （無限ループ防止）。
    pub fn settle_duration(&self) -> f64 {
        let mut t = 0.0;
        while t < MAX_SETTLE_DURATION {
            if self.at(t).done {
                return t;
            }
            t += SETTLE_STEP;
        }
        MAX_SETTLE_DURATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    // --- 物理パラメータ系 ---

    #[test]
    fn at_zero_returns_initial_state_in_all_regions() {
        for damping in [5.0, 20.0, 40.0] {
            let config = SpringConfig {
                stiffness: 100.0,
                damping,
                mass: 1.0,
            };
            let spring = Spring::new(config, 0.0, 100.0, 3.0).unwrap();
            let s = spring.at(0.0);
            assert!(approx(s.value, 0.0, 1e-9), "damping={damping}");
            assert!(approx(s.velocity, 3.0, 1e-9), "damping={damping}");
        }
    }

    #[test]
    fn ode_residual_matches_analytic_derivative_via_finite_difference() {
        // x'' + (c/m)x' + (k/m)x = 0（x = value - to）を満たすことを、
        // 解析速度と値の中心差分の一致・二階差分の残差の小ささで確認する。
        for damping in [5.0, 20.0, 40.0] {
            let stiffness = 100.0;
            let mass = 1.0;
            let config = SpringConfig {
                stiffness,
                damping,
                mass,
            };
            let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
            let h = 1e-4;
            for i in 1..20 {
                let t = i as f64 * 0.05;
                let s = spring.at(t);
                if s.done {
                    break;
                }
                let x_minus = spring.at(t - h).value;
                let x_plus = spring.at(t + h).value;
                let central_velocity = (x_plus - x_minus) / (2.0 * h);
                assert!(
                    approx(central_velocity, s.velocity, 1e-3),
                    "damping={damping} t={t} analytic={} central={}",
                    s.velocity,
                    central_velocity
                );

                let accel = (x_plus - 2.0 * s.value + x_minus) / (h * h);
                let x = s.value - spring.to;
                let residual = accel + (damping / mass) * s.velocity + (stiffness / mass) * x;
                assert!(
                    residual.abs() < 1.0,
                    "damping={damping} t={t} residual={residual}"
                );
            }
        }
    }

    #[test]
    fn default_spring_overshoot_matches_theoretical_value() {
        // 既定 spring（100/10/1、ζ=0.5）の 0→1 は
        // overshoot = exp(-ζπ/sqrt(1-ζ²)) だけ 1 を超え、
        // 到達時刻は π/ωd 付近になる（不足減衰の一般的性質）。
        let config = SpringConfig::default();
        assert!(approx(config.damping_ratio(), 0.5, 1e-9));
        let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();

        let omega0 = (config.stiffness / config.mass).sqrt();
        let zeta = 0.5_f64;
        let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
        let expected_peak_time = std::f64::consts::PI / omega_d;
        let expected_overshoot =
            1.0 + (-zeta * std::f64::consts::PI / (1.0 - zeta * zeta).sqrt()).exp();

        let mut max_value = f64::MIN;
        let mut max_time = 0.0;
        let mut t = 0.0;
        while t < 2.0 {
            let v = spring.at(t).value;
            if v > max_value {
                max_value = v;
                max_time = t;
            }
            t += 0.0005;
        }

        assert!(
            approx(max_value, expected_overshoot, 1e-3),
            "max_value={max_value} expected={expected_overshoot}"
        );
        assert!(
            approx(max_time, expected_peak_time, 0.02),
            "max_time={max_time} expected={expected_peak_time}"
        );
    }

    #[test]
    fn critical_and_overdamped_are_monotonic_and_never_overshoot() {
        for damping in [20.0, 40.0] {
            let config = SpringConfig {
                stiffness: 100.0,
                damping,
                mass: 1.0,
            };
            let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
            let mut prev = spring.at(0.0).value;
            let mut t = 0.001;
            while t < 3.0 {
                let v = spring.at(t).value;
                assert!(
                    v >= prev - 1e-9,
                    "damping={damping} t={t} v={v} prev={prev}"
                );
                assert!(v <= 1.0 + 1e-9, "damping={damping} t={t} v={v} overshot");
                prev = v;
                t += 0.01;
            }
        }
    }

    #[test]
    fn settle_duration_marks_done_and_snaps_value() {
        let spring = Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0).unwrap();
        let settle = spring.settle_duration();
        let after = spring.at(settle);
        assert!(after.done);
        assert_eq!(after.value, 1.0);
        assert_eq!(after.velocity, 0.0);

        if settle > 0.001 {
            let before = spring.at(settle - 0.001);
            assert!(!before.done);
        }

        // 0→1（距離 1 < 5）は granular 閾値（rest_delta=0.005）が使われる。
        // 不足減衰の振幅包絡線は |x(t)| = decay·R（R = sqrt(x0² + b²)、
        // decay = e^{-ζω0·t}）で上から抑えられるため、
        // decay·R = rest_delta となる時刻を理論値とする
        // （ζω0=5、b=(v0+ζω0·x0)/ωd=5/ωd、ωd=ω0·sqrt(1-ζ²)）。
        let omega0 = (SpringConfig::default().stiffness / SpringConfig::default().mass).sqrt();
        let zeta = 0.5_f64;
        let zeta_omega0 = zeta * omega0;
        let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
        let b = zeta_omega0 / omega_d;
        let r = (1.0_f64 + b * b).sqrt();
        let theoretical = -(0.005_f64 / r).ln() / zeta_omega0;
        assert!(
            approx(settle, theoretical, 0.05),
            "settle={settle} theoretical={theoretical}"
        );
    }

    #[test]
    fn rest_threshold_switches_between_granular_and_default() {
        // 移動距離が大きい（>= 5）場合は既定閾値（rest_delta=0.5）が使われ、
        // わずかな残差（0.3）ではまだ done にならないことを確認する。
        let far = Spring::new(SpringConfig::default(), 0.0, 100.0, 0.0).unwrap();
        let far_settle = far.settle_duration();
        // 収束直前は残差が rest_delta=0.5 に漸近するため、そのわずか手前の
        // 状態はまだ done でないはず。
        if far_settle > 0.001 {
            assert!(!far.at(far_settle - 0.001).done || far_settle < 0.002);
        }

        // 移動距離が小さい（< 5）場合は granular 閾値（rest_delta=0.005）が
        // 使われ、既定閾値なら done 扱いになるはずの残差でもまだ収束しない。
        let near = Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0).unwrap();
        // near の収束時刻は far よりも早くならない設計（同じ ζ・ω0 なので
        // 相対残差の減衰速度は同じだが、閾値が厳しいぶん収束は遅くなる）。
        assert!(near.settle_duration() >= far.settle_duration() - 0.01);
    }

    #[test]
    fn invalid_inputs_return_none() {
        let base = SpringConfig::default();
        assert!(Spring::new(
            SpringConfig {
                stiffness: 0.0,
                ..base
            },
            0.0,
            1.0,
            0.0
        )
        .is_none());
        assert!(Spring::new(
            SpringConfig {
                stiffness: -1.0,
                ..base
            },
            0.0,
            1.0,
            0.0
        )
        .is_none());
        assert!(Spring::new(SpringConfig { mass: 0.0, ..base }, 0.0, 1.0, 0.0).is_none());
        assert!(Spring::new(
            SpringConfig {
                damping: -1.0,
                ..base
            },
            0.0,
            1.0,
            0.0
        )
        .is_none());
        assert!(Spring::new(base, f64::NAN, 1.0, 0.0).is_none());
    }

    #[test]
    fn at_with_nan_or_negative_t_matches_at_zero() {
        let spring = Spring::new(SpringConfig::default(), 0.0, 1.0, 2.0).unwrap();
        let zero = spring.at(0.0);
        let nan = spring.at(f64::NAN);
        let negative = spring.at(-1.0);
        assert_eq!(zero, nan);
        assert_eq!(zero, negative);
    }

    #[test]
    fn at_is_deterministic() {
        let spring = Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0).unwrap();
        let a = spring.at(0.37);
        let b = spring.at(0.37);
        assert_eq!(a, b);
    }

    // --- duration/bounce 系 ---

    #[test]
    fn from_duration_bounce_damping_ratio() {
        let config = SpringConfig::from_duration_bounce(0.8, 0.3, 0.0, 1.0);
        assert!(
            approx(config.damping_ratio(), 0.7, 1e-9),
            "damping_ratio={}",
            config.damping_ratio()
        );
    }

    #[test]
    fn from_duration_bounce_settle_duration_is_close_to_requested() {
        for (duration, bounce) in [(0.8, 0.3), (0.3, 0.3), (2.0, 0.3)] {
            let config = SpringConfig::from_duration_bounce(duration, bounce, 0.0, 1.0);
            let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
            let settle = spring.settle_duration();
            let ratio = settle / duration;
            assert!(
                (0.7..=1.3).contains(&ratio),
                "duration={duration} settle={settle} ratio={ratio}"
            );
        }
    }

    #[test]
    fn from_duration_bounce_settle_duration_respects_nonzero_velocity() {
        // イシュー #2426 レビュー指摘の回帰確認: 包絡線に変位成分
        // （sqrt(1+b²)）を含めない実装は、初速がある入力で偽の根
        // （実際にはほぼ減衰していない ω0）を選び、duration=0.5 秒の
        // 要求に対し settle_duration が約 5.2 秒（10 倍超）になっていた。
        let config = SpringConfig::from_duration_bounce(0.5, 0.9, 1.0, 1.0);
        let spring = Spring::new(config, 0.0, 1.0, 1.0).unwrap();
        let settle = spring.settle_duration();
        let ratio = settle / 0.5;
        assert!(
            (0.5..=2.0).contains(&ratio),
            "settle={settle} ratio={ratio} (修正前は ratio≈10.5 だった)"
        );
    }

    #[test]
    fn from_duration_bounce_zeta_bounds() {
        let critical = SpringConfig::from_duration_bounce(0.5, 0.0, 0.0, 1.0);
        assert!(approx(critical.damping_ratio(), 1.0, 1e-6));

        let bouncy = SpringConfig::from_duration_bounce(0.5, 1.0, 0.0, 1.0);
        assert!(approx(bouncy.damping_ratio(), 0.05, 1e-6));
        // 発散しないことの確認: at() が有限値を返す。
        let spring = Spring::new(bouncy, 0.0, 1.0, 0.0).unwrap();
        assert!(spring.at(0.1).value.is_finite());
    }

    #[test]
    fn from_duration_bounce_clamps_extreme_inputs() {
        let too_short = SpringConfig::from_duration_bounce(0.001, 0.3, 0.0, 1.0);
        assert!(too_short.stiffness.is_finite() && too_short.stiffness > 0.0);

        let too_long = SpringConfig::from_duration_bounce(100.0, 0.3, 0.0, 1.0);
        assert!(too_long.stiffness.is_finite() && too_long.stiffness > 0.0);

        let bad_mass = SpringConfig::from_duration_bounce(0.5, 0.3, 0.0, -1.0);
        assert_eq!(bad_mass.mass, 1.0);
    }

    #[test]
    fn from_duration_bounce_domain_configs_are_usable_by_spring_new() {
        // イシュー #2426 レビュー指摘（P1/Medium 再指摘）の回帰確認:
        // `from_duration_bounce` の文書化された入力域（duration の
        // clamp 範囲 [0.01, 10.0]・bounce の clamp 範囲 [0, 1]）の端点・
        // 代表点を格子状に走査し、`velocity=0`・`mass=1.0` という典型
        // 入力で生成した config が必ず `Spring::new` を通り、
        // `settle_duration` 全域で `at()` が有限値を返すことを固定する。
        // 変換側（本関数）と構築側（`Spring::new`）の許容範囲が乖離すると
        // 「公開 API が許容する入力なのに構築できない」状態に戻るため、
        // この網羅チェックで再発を防ぐ。
        // `mass` は duration/bounce と異なりドキュメントに明示的な数値域が
        // ないパラメータであり（不正値のみ 1.0 へフォールバック）、
        // `stiffness = ω0²·mass` に線形寄与するため理論上どこまでも
        // `stiffness` を押し上げられる（例: mass=100 は本走査の
        // duration/bounce 最悪点で `stiffness` が `MAX_PARAM` を超える）。
        // これは「文書化された duration/bounce 範囲」を全域走査する対象
        // ではなく、典型的な物理質量（motion.dev 既定 `mass: 1` 周辺）の
        // 代表点として 0.01〜2.0 を選ぶ。
        let durations = [0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0];
        let bounces = [0.0, 0.1, 0.3, 0.5, 0.7, 0.9, 0.95, 0.99, 1.0];
        let masses = [0.01, 0.1, 1.0, 2.0];
        for &duration in &durations {
            for &bounce in &bounces {
                for &mass in &masses {
                    let config = SpringConfig::from_duration_bounce(duration, bounce, 0.0, mass);
                    assert!(
                        (MIN_PARAM..=MAX_PARAM).contains(&config.stiffness),
                        "stiffness out of range: duration={duration} bounce={bounce} mass={mass} stiffness={}",
                        config.stiffness
                    );
                    assert!(
                        (MIN_PARAM..=MAX_PARAM).contains(&config.damping),
                        "damping out of range: duration={duration} bounce={bounce} mass={mass} damping={}",
                        config.damping
                    );
                    let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap_or_else(|| {
                        panic!(
                            "Spring::new rejected from_duration_bounce output: duration={duration} bounce={bounce} mass={mass} config={config:?}"
                        )
                    });
                    let settle = spring.settle_duration();
                    assert!(settle.is_finite() && settle > 0.0);
                    let mut t = 0.0;
                    while t <= settle {
                        let state = spring.at(t);
                        assert!(state.value.is_finite());
                        assert!(state.velocity.is_finite());
                        t += settle / 8.0;
                    }
                }
            }
        }
    }

    #[test]
    fn from_duration_bounce_output_is_finite_and_positive() {
        let config = SpringConfig::from_duration_bounce(0.5, 0.3, 0.0, 1.0);
        assert!(config.stiffness.is_finite() && config.stiffness > 0.0);
        assert!(config.damping.is_finite() && config.damping > 0.0);

        // NaN 入力はフォールバックで既定値を返す。
        let fallback = SpringConfig::from_duration_bounce(f64::NAN, f64::NAN, 0.0, 1.0);
        assert_eq!(fallback, SpringConfig::default());
    }

    #[test]
    fn new_avoids_overflow_and_underflow_in_derived_coefficients() {
        // イシュー #2426 レビュー指摘の回帰確認: stiffness・damping・mass が
        // 揃って極端な値のとき、`stiffness * mass` を直接計算すると
        // オーバーフローして Inf に丸まり、本来 ζ=1（臨界減衰）となる
        // べき入力が ζ=0（無減衰振動、Region::Underdamped）と誤判定
        // されていた。入力ドメインの境界化（[`MIN_PARAM`]/[`MAX_PARAM`]）
        // 導入後は当時の値（1e200 等）自体が構築時に拒否されるため、
        // 許容範囲の上限付近で同じ ζ=1 の組み合わせを使い、安定化した
        // 除算・乗算の順序が引き続き正しい region 判定を返すことを確認
        // する。
        let huge = SpringConfig {
            stiffness: MAX_PARAM / 2.0,
            damping: MAX_PARAM,
            mass: MAX_PARAM / 2.0,
        };
        assert!(approx(huge.damping_ratio(), 1.0, 1e-6));
        let spring = Spring::new(huge, 0.0, 1.0, 0.0).unwrap();
        assert!(matches!(spring.region, Region::Critical { .. }));
        let s = spring.at(1e-6);
        assert!(s.value.is_finite() && s.velocity.is_finite());

        // 許容範囲の下限付近（tiny 同士）でも ζ が 0 除算で NaN/Inf に
        // ならず、有効な Spring を構築できる。
        let tiny = SpringConfig {
            stiffness: MIN_PARAM,
            damping: MIN_PARAM,
            mass: MIN_PARAM,
        };
        let spring = Spring::new(tiny, 0.0, 1.0, 0.0).unwrap();
        let s = spring.at(0.0);
        assert!(s.value.is_finite() && s.velocity.is_finite());
    }

    #[test]
    fn damping_ratio_avoids_overflow_with_extreme_uniform_inputs() {
        // イシュー #2426 レビュー指摘の回帰確認: stiffness=damping=mass が
        // 揃って極端な値（1e308）のとき、分母をまとめて計算すると
        // `2.0 * sqrt(stiffness) * sqrt(mass)` 自体が Inf に丸まり、
        // 本来 ζ=0.5 のばねを ζ=0（無減衰振動）と誤判定していた。
        // `damping_ratio()` は `Spring::new` の入力ドメイン境界化とは
        // 独立した純粋な算術メソッドのため、この安定化の回帰確認は
        // 従来どおり極端な値（1e308）で行う。
        let extreme = SpringConfig {
            stiffness: 1e308,
            damping: 1e308,
            mass: 1e308,
        };
        assert!(approx(extreme.damping_ratio(), 0.5, 1e-6));

        // `Spring::new` は入力ドメインの境界化により 1e308 のような値を
        // 構築時に拒否するため、region 判定・at() の有限性確認は許容
        // 範囲内の値（ζ=0.5 を保った組み合わせ）で行う。
        let config = SpringConfig {
            stiffness: MAX_PARAM,
            damping: MAX_PARAM,
            mass: MAX_PARAM,
        };
        assert!(approx(config.damping_ratio(), 0.5, 1e-6));
        let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
        assert!(matches!(spring.region, Region::Underdamped { .. }));
        let s = spring.at(1e-6);
        assert!(s.value.is_finite() && s.velocity.is_finite());
    }

    #[test]
    fn overdamped_roots_avoid_cancellation_and_overflow() {
        // イシュー #2426 レビュー指摘の回帰確認その1: 大きな zeta
        // （桁落ちする素朴な計算 `-omega0*(zeta-disc)` では r1 が -0 に
        // 丸まり、減衰が exp(-t) から消える）を、入力ドメインの境界化
        // 後も許容範囲内の組み合わせ（stiffness=damping=MAX_PARAM,
        // mass=MIN_PARAM、ζ=5e5）で再現する。有理化した計算では r1 が
        // 有限の負値になる。
        let config = SpringConfig {
            stiffness: MAX_PARAM,
            damping: MAX_PARAM,
            mass: MIN_PARAM,
        };
        let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
        let Region::Overdamped { r1, r2, .. } = spring.region else {
            panic!("expected Overdamped region");
        };
        assert!(r1.is_finite() && r1 < 0.0, "r1={r1}");
        assert!(r2.is_finite() && r2 < 0.0, "r2={r2}");
        let s0 = spring.at(0.0);
        let s1 = spring.at(1e-6);
        // from=0 → to=1 なので、時間経過とともに value は to=1 へ単調に
        // 近づく（オーバーシュートしない）。
        assert!(
            s1.value > s0.value && s1.value <= 1.0 + 1e-9,
            "s0={} s1={}",
            s0.value,
            s1.value
        );

        // イシュー #2426 レビュー指摘の回帰確認その2: damping が
        // stiffness/mass に対し極端に大きい（許容範囲の上限
        // `MAX_PARAM`）組み合わせでも `(zeta*zeta - 1.0).sqrt()` が
        // オーバーフローせず at() が有限値を返す。
        let huge_damping = SpringConfig {
            stiffness: 1.0,
            damping: MAX_PARAM,
            mass: 1.0,
        };
        let spring = Spring::new(huge_damping, 0.0, 1.0, 0.0).unwrap();
        let s = spring.at(1e-6);
        assert!(s.value.is_finite() && s.velocity.is_finite());
    }

    #[test]
    fn new_rejects_out_of_range_displacement() {
        // イシュー #2426 レビュー指摘の回帰確認: 変位・初速度の絶対値が
        // [`MAX_ABS_VALUE`] を超える入力は、個別には有限であっても
        // 入力ドメインの境界化により構築時に一括で拒否される（`at()`
        // 側で非有限値を丸めて誤魔化す経路を持たない）。
        assert!(Spring::new(SpringConfig::default(), MAX_ABS_VALUE * 2.0, 0.0, 0.0).is_none());
        assert!(Spring::new(SpringConfig::default(), 0.0, MAX_ABS_VALUE * 2.0, 0.0).is_none());
        assert!(Spring::new(SpringConfig::default(), 0.0, 0.0, MAX_ABS_VALUE * 2.0).is_none());
    }

    #[test]
    fn from_duration_bounce_newton_non_convergence_falls_back_to_default() {
        // イシュー #2426 レビュー指摘の回帰確認: Newton 法が
        // `NEWTON_ITERATIONS` 回で収束しない極端な velocity
        // （例: 1e6）を残差確認なしに成功として返すと、誤った
        // stiffness/damping が生成される。未収束は既定値へ
        // フォールバックする。
        let config = SpringConfig::from_duration_bounce(0.5, 0.9, 1e6, 1.0);
        assert_eq!(config, SpringConfig::default());
    }

    #[test]
    fn from_duration_bounce_avoids_underflow_with_tiny_mass() {
        // イシュー #2426 レビュー指摘の回帰確認: `(mass * stiffness).sqrt()`
        // を直接計算すると mass=1e-200 のとき積がアンダーフローしてゼロに
        // 丸まり、damping=0（無減衰）を誤って返していた。
        let config = SpringConfig::from_duration_bounce(0.5, 0.3, 0.0, 1e-200);
        assert!(config.damping.is_finite() && config.damping > 0.0);
        assert!(approx(config.damping_ratio(), 0.7, 1e-6));
    }

    #[test]
    fn new_rejects_overdamped_root_overflow() {
        // PR #2426 codex レビュー指摘の回帰確認: stiffness=1,
        // damping=1e200, mass=1e-200 のような r1/r2 のオーバーフロー
        // 組み合わせは、入力ドメインの境界化（[`MIN_PARAM`]/
        // [`MAX_PARAM`]）により、damping・mass が個別に範囲外である
        // 時点で構築時に None として拒否される（表現不能な組み合わせを
        // Region 構築後に検出する必要がなくなった）。
        let config = SpringConfig {
            stiffness: 1.0,
            damping: 1e200,
            mass: 1e-200,
        };
        assert!(Spring::new(config, 0.0, 1.0, 0.0).is_none());
    }

    #[test]
    fn overdamped_b_matches_initial_velocity_without_cancellation() {
        // PR #2426 codex レビュー指摘の回帰確認: b を `x0 - a`
        // （差し引き）で求めると、a が -1 へ丸まる入力で b が 0 に
        // 桁落ちし、at(0).velocity が指定した初速度を再現できない
        // （実際には 1 を返していた）。入力ドメインの境界化後も許容
        // 範囲内の値（stiffness=damping=MAX_PARAM, mass=1.0、ζ=500）で
        // 同じ桁落ちが再現することを確認する。b を初期条件から直接
        // `(r1*x0 - v0) / (r1-r2)` で解くことで、この桁落ちを避ける。
        let config = SpringConfig {
            stiffness: MAX_PARAM,
            damping: MAX_PARAM,
            mass: 1.0,
        };
        let spring = Spring::new(config, 0.0, 1.0, 0.0).unwrap();
        let s0 = spring.at(0.0);
        assert!(
            approx(s0.velocity, 0.0, 1e-6),
            "at(0).velocity should reproduce the initial velocity 0, got {}",
            s0.velocity
        );
        assert!(approx(s0.value, 0.0, 1e-9), "at(0).value={}", s0.value);
    }

    #[test]
    fn at_avoids_overflow_in_critical_region_with_extreme_displacement() {
        // PR #2426 codex レビュー P1 指摘の回帰確認: stiffness=1, damping=2,
        // mass=1（ζ=1、Region::Critical）で `from` が入力ドメインの上限
        // （[`MAX_ABS_VALUE`]）に達するとき、構築時の係数 `c` 自体は
        // 有限でも `at()` 内部で `c * t` を先に評価すると表現範囲を
        // 超えて Inf になり、decay を掛けても NaN/Inf が伝播していた。
        // decay を先に適用する安定化後は value/velocity が共に有限で
        // あること。
        let config = SpringConfig {
            stiffness: 1.0,
            damping: 2.0,
            mass: 1.0,
        };
        let spring = Spring::new(config, MAX_ABS_VALUE, 0.0, 0.0).unwrap();
        let state = spring.at(2.0);
        assert!(state.value.is_finite(), "value={}", state.value);
        assert!(state.velocity.is_finite(), "velocity={}", state.velocity);
    }

    #[test]
    fn at_avoids_overflow_in_final_value_addition() {
        // PR #2426 codex レビュー P1 再指摘の回帰確認: x/v 単体は有限でも
        // `self.to + x`（最終 value の加算）自体がオーバーフローしうる
        // 組み合わせ（from/to が入力ドメインの上限付近で符号が異なる）。
        // 入力ドメインの境界化後は from/to の差が高々 2*MAX_ABS_VALUE に
        // 収まるため、settle_duration の探索範囲全体で value/velocity が
        // 有限であることを確認する。
        let config = SpringConfig {
            stiffness: 1.0,
            damping: 0.1,
            mass: 1.0,
        };
        let spring = Spring::new(config, MAX_ABS_VALUE, -MAX_ABS_VALUE, 0.0).unwrap();
        let mut t = 0.0;
        while t <= MAX_SETTLE_DURATION {
            let state = spring.at(t);
            assert!(state.value.is_finite(), "t={t} value={}", state.value);
            assert!(
                state.velocity.is_finite(),
                "t={t} velocity={}",
                state.velocity
            );
            t += 0.5;
        }
    }

    #[test]
    fn new_rejects_reported_zero_damping_overflow_example() {
        // PR #2426 codex レビュー P1 再指摘
        // （discussion_r3996960841）の回帰確認: stiffness=0.01,
        // damping=0, mass=1, from=1.7e308, initial_velocity=1.7e307 は
        // 無減衰（damping=0）ゆえ本来「静止しない」入力であり、
        // t=3π/(4·0.1) 付近で速度計算の中間和が数学的にも f64 表現
        // 範囲を超える。以前は非有限値を (0,0) へ丸めるフォールバックが
        // 誤って done:true を返していたが、入力ドメインの境界化により
        // damping=0（[`MIN_PARAM`] 未満）・from が [`MAX_ABS_VALUE`] を
        // 超えることの双方で構築時に拒否される。
        let config = SpringConfig {
            stiffness: 0.01,
            damping: 0.0,
            mass: 1.0,
        };
        assert!(Spring::new(config, 1.7e308, 0.0, 1.7e307).is_none());

        // damping=0（無減衰）は、他のパラメータが入力ドメイン内でも
        // 単独で拒否される（静止しない入力は settle 契約と矛盾する
        // ため、値の大小に関わらず一律で無効）。
        let undamped = SpringConfig {
            stiffness: 100.0,
            damping: 0.0,
            mass: 1.0,
        };
        assert!(Spring::new(undamped, 0.0, 1.0, 0.0).is_none());
    }

    #[test]
    fn at_stays_finite_across_settle_duration_at_domain_boundary() {
        // 入力ドメインの境界値（MAX_ABS_VALUE・MIN_PARAM・MAX_PARAM）を
        // 同時に使う極端な構成でも、`at()` が settle_duration の探索
        // 範囲全体で有限値を返すことを確認する（イシュー #2426 レビュー
        // 指摘: 境界化そのものの正しさを直接検証する）。
        let config = SpringConfig {
            stiffness: MAX_PARAM,
            damping: MIN_PARAM,
            mass: MIN_PARAM,
        };
        let spring = Spring::new(config, MAX_ABS_VALUE, 0.0, 0.0).unwrap();
        let mut t = 0.0;
        while t <= MAX_SETTLE_DURATION {
            let state = spring.at(t);
            assert!(state.value.is_finite(), "t={t} value={}", state.value);
            assert!(
                state.velocity.is_finite(),
                "t={t} velocity={}",
                state.velocity
            );
            t += 0.25;
        }
    }

    #[test]
    fn at_avoids_nan_in_underdamped_region_at_extreme_time() {
        // PR #2426 codex レビュー P1 指摘の回帰確認: 既定パラメータ
        // （Region::Underdamped）でも `at(1e308)` は `omega_d * t` が Inf へ
        // オーバーフローし sin_cos(Inf) が NaN を返すため、以前は
        // value/velocity が NaN になっていた。decay が 0 まで下振れした
        // 時点で sin_cos の計算自体を打ち切る安定化後は、収束済み
        // （done かつ value == to）を返すこと。
        let spring = Spring::new(SpringConfig::default(), 1.0, 0.0, 0.0).unwrap();
        let state = spring.at(1e308);
        assert!(state.value.is_finite(), "value={}", state.value);
        assert!(state.velocity.is_finite(), "velocity={}", state.velocity);
        assert!(state.done);
    }
}
