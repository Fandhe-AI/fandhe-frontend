//! marquee の JS 駆動拡張（ticker、イシュー #2540）: 実測に基づく複製数
//! 決定・rAF による offset 前進・hover/scroll 速度連動の計算・DOM 適用。
//!
//! # 責務境界
//!
//! `fandhe-frontend-pre-styled-ui::marquee_motion` が供給する opt-in 属性
//! （`data-fandhe-ticker*`）から実際に DOM を駆動する層。属性の探索・
//! イベント委譲登録・`prefers-reduced-motion` 判定は
//! `fandhe-frontend-wasm-full::ticker` の責務であり、本モジュールは
//! 「要素 1 個分の駆動状態（[`Ticker`]）」と、その内部で使う純粋な計算
//! （speed/offset/複製数、native テストで検証可能）のみを持つ
//! （`docs/design/motion-reference-adoption-policy.md` §6 の 3 層構成）。
//!
//! # ponytail 割り切り
//!
//! - 複製数は `2..=16` にクランプする（極端に短いコンテンツでの要素数
//!   爆発を防ぐ、A04 対策）。上限を超える継ぎ目は許容する既知の制約。
//! - 速度は `0..=2000` px/s にクランプする。
//! - scroll 速度は毎フレーム 0.9 倍の素朴な指数減衰のみ（物理的なばね等は
//!   実装しない）。
//! - 動的に追加された ticker 要素は wire 時の `querySelectorAll` の対象外
//!   （`svg_path`/`in_view` と同じ制約）。
//! - 入れ子 ticker（ticker の content 内に別の ticker）では、外側の追加
//!   複製に含まれる内側 ticker のクローンは**駆動しない**（静的、offset
//!   0px 固定）。元の内側 ticker だけが JS 駆動される。クローンごとに
//!   独立駆動すると位相がずれ、外側の継ぎ目が崩れるうえ rAF ループが
//!   複製数倍に増えるため、「クローンは重複駆動しない」ことを保証する
//!   側に倒す（PR #2582 codex-review P1・Cursor Bugbot 指摘）。

/// [`Ticker`] が DOM の CSS カスタムプロパティへ毎フレーム書き込む変数名。
/// `fandhe-frontend-pre-styled-ui::marquee_motion::TICKER_OFFSET_VAR` /
/// `fandhe-frontend-wasm-full::ticker::TICKER_OFFSET_VAR` と同値のリテラル
/// （drift テストで固定、`crates/pre-styled-ui/tests/marquee_motion_attr_drift.rs`）。
pub const TICKER_OFFSET_VAR: &str = "--fandhe-marquee-ticker-offset";

/// opt-in 属性名・JS 駆動中マーカー属性名。
/// `fandhe-frontend-wasm-full::ticker::{TICKER_ATTR, TICKER_ACTIVE_ATTR}` と
/// 同値のリテラル（同じ drift テストで固定）。[`ensure_copies`] が追加複製
/// 内の入れ子 ticker を静的化するために使う。
pub const TICKER_ATTR: &str = "data-fandhe-ticker";
pub const TICKER_ACTIVE_ATTR: &str = "data-fandhe-ticker-active";

/// 速度の既定値（px/s）・上限。
pub const DEFAULT_SPEED_PX_S: f64 = 80.0;
const MAX_SPEED_PX_S: f64 = 2000.0;

/// 複製数のクランプ範囲。
const MIN_COPIES: usize = 2;
const MAX_COPIES: usize = 16;

/// dt（フレーム間隔）の上限（ミリ秒）。タブが非アクティブ化から復帰した
/// 直後等の巨大な dt で offset が飛ばないようにする。
const MAX_DT_MS: f64 = 100.0;

/// scroll 速度の毎フレーム減衰係数（素朴な指数減衰、モジュール doc
/// 「ponytail 割り切り」節参照）。
const SCROLL_DECAY_PER_FRAME: f64 = 0.9;

/// スクロール軸（`data-axis` 属性の値と対応）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// [`Ticker`] 1 個分の設定。
#[derive(Debug, Clone, Copy)]
pub struct TickerConfig {
    /// 基準速度（px/s、正の有限数）。
    pub speed_px_s: f64,
    /// スクロール方向の符号（`1.0` = 通常、`-1.0` = 逆方向。
    /// `marquee::MarqueeDirection` の `Start`/`End` に対応）。
    pub direction_sign: f64,
    pub axis: Axis,
    /// hover 中の速度係数（`0.0` = 停止・`1.0` = 変化なし、既定 `0.0`）。
    pub hover_factor: f64,
    /// scroll 速度連動の係数（`0.0` = 連動なし、既定 `0.0`）。
    pub scroll_factor: f64,
}

/// `data-fandhe-ticker-speed` 等の属性文字列から数値を読む。非有限・負・
/// パース失敗は `default` へ fail-safe し、`max` 超過は `max` へ clamp する
/// （security.md A05、利用者由来の文字列を検証なしに使わない）。
#[must_use]
pub fn parse_speed(raw: &str) -> f64 {
    parse_clamped(raw, DEFAULT_SPEED_PX_S, 0.0, MAX_SPEED_PX_S)
}

/// `data-fandhe-ticker-hover-factor`/`-scroll-factor` 等、`0.0..=max` の
/// 係数属性をパースする。非有限・負・パース失敗は `default` へ fail-safe
/// する。
#[must_use]
pub fn parse_factor(raw: &str, default: f64, max: f64) -> f64 {
    parse_clamped(raw, default, 0.0, max)
}

fn parse_clamped(raw: &str, default: f64, min: f64, max: f64) -> f64 {
    match raw.trim().parse::<f64>() {
        Ok(value) if value.is_finite() && value >= min => value.min(max),
        _ => default,
    }
}

/// hover・scroll 連動を反映した実効速度（px/s、常に 0 以上の有限値）を
/// 計算する。
///
/// - `hovered` なら `base * hover_factor` を基準にする（既定
///   `hover_factor = 0.0` は完全停止、既存 CSS 版の hover 一時停止契約と
///   一致させる）。
/// - `scroll_velocity`（px/s、符号付き）の絶対値に `scroll_factor` を
///   掛けた分を加算する。
#[must_use]
pub fn effective_speed(
    base_speed: f64,
    hovered: bool,
    focused: bool,
    hover_factor: f64,
    scroll_velocity: f64,
    scroll_factor: f64,
) -> f64 {
    if !base_speed.is_finite() || base_speed < 0.0 {
        return 0.0;
    }
    // キーボードフォーカス中は `hover_factor` に関わらず常に完全停止する。
    // 既存 `marquee` の CSS 版は `:hover`/`:focus-within` の両方を
    // `animation-play-state: paused` で同列に扱う（両方とも完全停止）が、
    // `hover_factor` は「ポインタ hover 中の速度」を著者が調整するための
    // opt-in 属性であり、focus をポインタ hover と同一視して
    // `hover_factor` を適用すると `hover_factor` を 0 以外へ設定した途端に
    // キーボード操作者だけ完全停止契約（WCAG 2.2.2）を失う（PR #2582
    // codex-review P1 指摘）。
    if focused {
        return 0.0;
    }
    let scroll_boost = if scroll_velocity.is_finite() && scroll_factor.is_finite() {
        scroll_velocity.abs() * scroll_factor.max(0.0)
    } else {
        0.0
    };
    // hover_factor は base_speed + scroll_boost を加算した合計へ適用する。
    // base_speed にのみ適用すると `hover_factor = 0.0`（既定・完全停止契約）
    // でも scroll_boost 分だけ動き続けてしまい、「hover 中は完全停止」の
    // 契約が scroll 連動時に破れていた（PR #2582 codex-review P1・Cursor
    // Bugbot 指摘）。
    let total = base_speed + scroll_boost;
    let total = if hovered {
        total * hover_factor.clamp(0.0, 1.0)
    } else {
        total
    };
    total.clamp(0.0, MAX_SPEED_PX_S)
}

/// `offset`（px、`(-cycle_len, 0]` に正規化）を `speed * direction_sign` で
/// `dt_ms` 分だけ前進させる。`cycle_len <= 0` や非有限入力は変化なし
/// （`offset` をそのまま返す）で fail-safe する。
#[must_use]
pub fn advance_offset(
    offset: f64,
    speed: f64,
    direction_sign: f64,
    dt_ms: f64,
    cycle_len: f64,
) -> f64 {
    if !offset.is_finite() || !speed.is_finite() || !cycle_len.is_finite() || cycle_len <= 0.0 {
        return offset;
    }
    let dt_ms = dt_ms.clamp(0.0, MAX_DT_MS);
    let direction_sign = if direction_sign < 0.0 { -1.0 } else { 1.0 };
    let delta = speed * direction_sign * (dt_ms / 1000.0);
    // シームレスループの折り返しは常に `[-cycle_len, 0]`（負方向へ進む
    // translate 値）へ正規化する: `direction_sign` が符号を反転させても
    // 折り返し先の範囲自体は変えず、複製列の先頭が root の左（上）端に
    // 常に揃う既存 CSS 版の `@keyframes` と同じ見た目を維持する。
    let next = offset - delta;
    // `rem_euclid` は `[0, cycle_len)` を返すため、そのまま `- cycle_len`
    // すると値域は `[-cycle_len, 0)` になり `0` を含まない。offset=0・
    // dt_ms=0（起動前 hover/focus 停止中や speed=0）でも `next=0` が
    // 毎回 `-cycle_len`（主コピー 1 個分ずれた位置）へ丸められ、移動量が
    // ゼロなのに主コピーが画面外へ移動してしまっていた（PR #2582
    // codex-review P1 指摘）。`r == 0` のときだけ `0` を残すことで、
    // モジュール doc の値域契約 `(-cycle_len, 0]` を満たす。
    let r = next.rem_euclid(cycle_len);
    if r == 0.0 {
        0.0
    } else {
        r - cycle_len
    }
}

/// ビューポート長・コンテンツ 1 個分の長さから、継ぎ目なく循環させるため
/// 必要な複製数（元のコンテンツ含む）を計算する。`content_len <= 0` は
/// [`MIN_COPIES`] を返す。
#[must_use]
pub fn required_copies(viewport_len: f64, content_len: f64) -> usize {
    if !content_len.is_finite() || content_len <= 0.0 || !viewport_len.is_finite() {
        return MIN_COPIES;
    }
    let needed = (viewport_len / content_len).ceil() as i64 + 1;
    needed.clamp(MIN_COPIES as i64, MAX_COPIES as i64) as usize
}

/// scroll 速度（px/s）を毎フレーム指数減衰させる（モジュール doc
/// 「ponytail 割り切り」節）。
#[must_use]
pub fn decay_velocity(velocity: f64) -> f64 {
    if !velocity.is_finite() {
        return 0.0;
    }
    let next = velocity * SCROLL_DECAY_PER_FRAME;
    if next.abs() < 0.01 {
        0.0
    } else {
        next
    }
}

#[cfg(target_arch = "wasm32")]
mod dom {
    use super::{Axis, TickerConfig, TICKER_ACTIVE_ATTR, TICKER_ATTR, TICKER_OFFSET_VAR};
    use crate::raf_driver::AnimationLoop;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::JsCast;
    use web_sys::{Element, HtmlElement, Node};

    /// `core/src/url.rs` の URL 検証ガード 4 種の crate-local な写し。
    ///
    /// 本クレートは `docs/design/animation-core-architecture.md` の方針
    /// （`fandhe-animation` + wasm-bindgen/web-sys/js-sys のみに依存し、
    /// `fandhe-frontend-core` 等の新規依存は追加しない）に従うため、
    /// `fandhe-frontend-core` を依存追加する代わりに同じ判定ロジックを
    /// このモジュール内へ複製する（`fw gate` の `url_validation_check`
    /// U1 が要求する「DOM 属性 sink 呼び出しファイル内でのガード関数 4 種
    /// 共起」を、新規依存を増やさず満たす）。本モジュールの `set_attribute`
    /// 呼び出しは `aria-hidden`/`inert` の固定リテラルのみで URL 値を扱わ
    /// ないが、fandhe-frontend-wasm-full::tabs_indicator（同型パターン）と同じく
    /// 将来値が動的化した場合の防御として同じガードを経由する。
    mod url_guard {
        const URL_ATTRS: &[&str] = &[
            "href",
            "src",
            "action",
            "formaction",
            "xlink:href",
            "poster",
            "cite",
            "data",
            "background",
            "ping",
            "dynsrc",
            "lowsrc",
        ];

        pub fn is_url_attr(name: &str) -> bool {
            URL_ATTRS.iter().any(|a| a.eq_ignore_ascii_case(name))
        }

        pub fn is_event_handler_attr(name: &str) -> bool {
            name.len() > 2
                && name.as_bytes()[0].eq_ignore_ascii_case(&b'o')
                && name.as_bytes()[1].eq_ignore_ascii_case(&b'n')
        }

        pub fn is_safe_url(value: &str) -> bool {
            let stripped: String = value
                .chars()
                .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
                .collect();
            let trimmed =
                stripped.trim_start_matches(|c: char| c.is_control() || c.is_whitespace());
            match extract_scheme(trimmed) {
                None => true,
                Some(scheme) => {
                    scheme.eq_ignore_ascii_case("http")
                        || scheme.eq_ignore_ascii_case("https")
                        || scheme.eq_ignore_ascii_case("mailto")
                        || scheme.eq_ignore_ascii_case("tel")
                }
            }
        }

        pub fn is_safe_srcset(value: &str) -> bool {
            value.split(',').all(|candidate| {
                let url_part = candidate.split_whitespace().next().unwrap_or("");
                is_safe_url(url_part)
            })
        }

        fn extract_scheme(s: &str) -> Option<&str> {
            let colon_idx = s.find(':')?;
            let candidate = &s[..colon_idx];
            if candidate.contains(['/', '?', '#', '\\']) {
                return None;
            }
            let mut chars = candidate.chars();
            match chars.next() {
                Some(c) if c.is_ascii_alphabetic() => {}
                _ => return None,
            }
            if !chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-') {
                return None;
            }
            Some(candidate)
        }
    }

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （fandhe-frontend-wasm-full::tabs_indicator の `set_dom_attribute` と
    /// 同じ方針・同じ 4 種のガードを経由する）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) {
        if url_guard::is_event_handler_attr(name) {
            return;
        }
        if url_guard::is_url_attr(name) && !url_guard::is_safe_url(value) {
            return;
        }
        if name.eq_ignore_ascii_case("srcset") && !url_guard::is_safe_srcset(value) {
            return;
        }
        let _ = element.set_attribute(name, value);
    }

    /// `element` の内容領域の長さ（`Axis::Horizontal` なら幅・`Vertical`
    /// なら高さ）をレイアウト座標（`offsetWidth`/`offsetHeight`）で読む。
    ///
    /// `getBoundingClientRect()` は祖先の `transform: scale()` 等を反映した
    /// 画面上の寸法を返すが、本モジュールが書き込む offset は content の
    /// `transform: translate()` のローカル px（transform 適用前の座標系）
    /// であり、両者を混ぜると祖先 scale 時に周期が実際のコピー間隔と
    /// ずれて空白・ジャンプが生じる（PR #2582 codex-review P1 指摘）。
    /// `offsetWidth`/`offsetHeight` は transform 非反映のレイアウト値の
    /// ため座標系が揃う。`HtmlElement` でない要素（SVG 等）のみ
    /// `getBoundingClientRect()` へフォールバックする。
    ///
    /// ponytail: `offsetWidth`/`offsetHeight` は整数へ丸められるため小数px
    /// 幅の content では周期に最大 0.5px の誤差が残る。目視できる継ぎ目に
    /// なった場合は `getBoundingClientRect` を `offsetWidth` 比で除して
    /// scale 補正する方式へ置き換える。
    #[must_use]
    pub fn measure_len(element: &Element, axis: Axis) -> f64 {
        if let Some(html) = element.dyn_ref::<HtmlElement>() {
            return match axis {
                Axis::Horizontal => f64::from(html.offset_width()),
                Axis::Vertical => f64::from(html.offset_height()),
            };
        }
        let rect = element.get_bounding_client_rect();
        match axis {
            Axis::Horizontal => rect.width(),
            Axis::Vertical => rect.height(),
        }
    }

    /// `element` の複製間 gap（px）を `getComputedStyle` から読む。
    /// `gap` shorthand（`row-gap column-gap` の 2 値になり得る）を自前で
    /// パースせず、軸に対応する longhand（`Horizontal` → `column-gap`、
    /// `Vertical` → `row-gap`）の computed value を直接読む（Cursor Bugbot
    /// 指摘: 2 値のとき gap が 0 扱いになり継ぎ目が見えていた）。
    /// `normal`・取得・パース失敗は `0.0` へ fail-safe する。
    #[must_use]
    pub fn read_gap_px(element: &HtmlElement, axis: Axis) -> f64 {
        let Some(window) = web_sys::window() else {
            return 0.0;
        };
        let Ok(Some(style)) = window.get_computed_style(element) else {
            return 0.0;
        };
        let property = match axis {
            Axis::Horizontal => "column-gap",
            Axis::Vertical => "row-gap",
        };
        let Ok(gap) = style.get_property_value(property) else {
            return 0.0;
        };
        gap.trim()
            .strip_suffix("px")
            .and_then(|n| n.parse::<f64>().ok())
            .filter(|n| n.is_finite())
            .unwrap_or(0.0)
    }

    /// `content`（既存 SSR 複製 1 個を含む）を `needed` 個になるまで
    /// `cloneNode(true)` で追加する（既存 2 個を超えて増やす場合のみ）。
    /// 追加した複製には `aria-hidden="true"`・`inert` を付与し、
    /// スクリーンリーダー二重読み上げ・タブ順序混入を防ぐ
    /// （`marquee` 既存複製と同じ契約）。`root` から現在の子要素数を数え、
    /// 冪等（再呼び出しでは増えない）。
    pub fn ensure_copies(root: &Element, content_template: &Element, needed: usize) {
        let needed = needed.clamp(super::MIN_COPIES, super::MAX_COPIES);
        let mut current = root.child_element_count() as usize;
        while current < needed {
            let Ok(clone) = content_template.clone_node_with_deep(true) else {
                break;
            };
            let Ok(clone_element) = clone.dyn_into::<Element>() else {
                break;
            };
            set_dom_attribute(&clone_element, "aria-hidden", "true");
            set_dom_attribute(&clone_element, "inert", "");
            // `cloneNode(true)` は radio の checked 状態も複製する。`inert`/
            // `aria-hidden` は radio button group からの除外条件ではないため、
            // そのまま挿入すると元の可視 radio の選択が解除され操作不能な
            // 複製へ選択が移る（PR #2582 codex-review P1 指摘）。
            // `presence::isolate_radio_groups` と同じく挿入**前**に `name` を
            // 除去してグループ membership を断つ。
            crate::presence::isolate_radio_groups(&clone_element);
            // 同じく `inert`/`aria-hidden` は送信・制約検証（`required` 等）の
            // 除外条件ではないため、フォーム内 ticker の追加複製は同名の
            // 送信値を複製数分増やし、required 未入力の複製が送信を阻む
            // （PR #2582 codex-review P1 指摘）。`presence::disable_form_controls`
            // で複製内のフォーム部品を `disabled` にし、送信・検証から除外する。
            crate::presence::disable_form_controls(&clone_element);
            neutralize_nested_tickers(&clone_element);
            if root.append_child(&clone_element as &Node).is_err() {
                break;
            }
            current += 1;
        }
    }

    /// `element` が marquee/ticker の **2 番目以降の content コピー**
    /// （SSR が出力する `aria-hidden` 付きの既存複製、または
    /// [`ensure_copies`] の追加複製）の配下にあるか。
    ///
    /// 判定は祖先チェーン上の `[data-part="content"]` のうち、親が
    /// marquee/ticker root（`[data-fandhe-ticker]` または
    /// `[data-scope="marquee"]`）であるものについて「親の最初の content
    /// 子ではない」または `aria-hidden="true"` を持つ、のいずれかが成立
    /// するかで行う（`tabs` 等の別 scope が持つ `data-part="content"` を
    /// 誤って複製扱いしないため親の scope を要求する）。
    ///
    /// `wasm-full::ticker::wire_ticker*` は SSR 由来の 2 コピー目に含まれる
    /// 入れ子 ticker を本関数で起動対象から外し
    /// [`neutralize_nested_tickers`] で静的化する（Cursor Bugbot 指摘:
    /// SSR コピー内の内側 ticker が独立 rAF で駆動され、元の内側・追加
    /// 複製の静的な内側と位相がずれていた）。
    #[must_use]
    pub fn is_in_secondary_copy(element: &Element) -> bool {
        const CONTENT_SELECTOR: &str = "[data-part=\"content\"]";
        const COPY_HOST_SELECTOR: &str = "[data-fandhe-ticker], [data-scope=\"marquee\"]";
        let mut cursor = element.parent_element();
        while let Some(ancestor) = cursor {
            if ancestor.matches(CONTENT_SELECTOR).unwrap_or(false) {
                if let Some(host) = ancestor.parent_element() {
                    if host.matches(COPY_HOST_SELECTOR).unwrap_or(false) {
                        let aria_hidden =
                            ancestor.get_attribute("aria-hidden").as_deref() == Some("true");
                        let first = host
                            .query_selector(&format!(":scope > {CONTENT_SELECTOR}"))
                            .ok()
                            .flatten();
                        let is_first = first.is_some_and(|f| f.is_same_node(Some(&ancestor)));
                        if aria_hidden || !is_first {
                            return true;
                        }
                    }
                }
            }
            cursor = ancestor.parent_element();
        }
        false
    }

    /// `clone`（自身 + 子孫）に含まれる入れ子 ticker（[`TICKER_ATTR`]）を
    /// 静的化する: [`TICKER_ACTIVE_ATTR`] を付与して CSS `@keyframes`
    /// 駆動を止め、[`TICKER_OFFSET_VAR`] をインラインで `0px` に固定する
    /// （外側 root へ書き込まれる外側の offset を継承して二重に translate
    /// されないようにする）。
    ///
    /// 外側 ticker の追加複製は wire 時の走査（`wasm-full::ticker`）の
    /// 後に生成されるため、複製内の内側 ticker は `Ticker` を持たない。
    /// 放置すると「元の内側は JS 駆動・複製内は CSS 駆動（または複製時点の
    /// offset で静止）」と位相がばらばらになり、外側の継ぎ目が崩れる
    /// （PR #2582 codex-review P1・Cursor Bugbot 指摘）。複製ごとに
    /// `Ticker` を起動する案は rAF ループが複製数倍に増え、かつ位相同期を
    /// 別途要するため採らず、「複製内の入れ子 ticker は駆動しない」設計に
    /// 固定する（モジュール doc「ponytail 割り切り」節）。
    pub fn neutralize_nested_tickers(clone: &Element) {
        let selector = format!("[{TICKER_ATTR}]");
        let mut targets: Vec<Element> = Vec::new();
        if clone.matches(&selector).unwrap_or(false) {
            targets.push(clone.clone());
        }
        if let Ok(nodes) = clone.query_selector_all(&selector) {
            for i in 0..nodes.length() {
                if let Some(el) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                    targets.push(el);
                }
            }
        }
        for nested in targets {
            set_dom_attribute(&nested, TICKER_ACTIVE_ATTR, "");
            if let Some(html) = nested.dyn_ref::<HtmlElement>() {
                let _ = html.style().set_property(TICKER_OFFSET_VAR, "0px");
            }
        }
    }

    /// `element` の style へ `offset`（px）を [`TICKER_OFFSET_VAR`] として
    /// 書き込む。`CSSStyleDeclaration.setProperty` の 2 引数 API のみを
    /// 使う（A03 対策、固定プロパティ名・`f64` 演算結果のみ）。
    pub fn write_offset(element: &HtmlElement, offset_px: f64) {
        let _ = element
            .style()
            .set_property(TICKER_OFFSET_VAR, &format!("{offset_px}px"));
    }

    /// 実ポインタ hover を実行できるデバイスかどうか
    /// （`(hover: hover) and (pointer: fine)`）。`Ticker::start` の初期
    /// `:hover` 読み取りがタッチデバイスの sticky `:hover` を誤検知しない
    /// ためのガード（`Ticker::start` 内コメント参照）。判定失敗は `false`
    /// へ fail-safe する。
    fn supports_real_hover() -> bool {
        web_sys::window()
            .and_then(|window| {
                window
                    .match_media("(hover: hover) and (pointer: fine)")
                    .ok()
                    .flatten()
            })
            .map(|list| list.matches())
            .unwrap_or(false)
    }

    /// 1 ticker 要素分の駆動状態。`root`/`content` を握り、`AnimationLoop`
    /// が毎フレーム offset を前進・DOM へ書き込む。`Drop` でループを停止
    /// する。
    pub struct Ticker {
        _loop: AnimationLoop,
        hovered: Rc<Cell<bool>>,
        focused: Rc<Cell<bool>>,
        scroll_distance: Rc<Cell<f64>>,
        resize_pending: Rc<Cell<bool>>,
        on_disconnect: DisconnectHook,
    }

    /// `root` の切断を rAF ループが検知したとき 1 回だけ呼ぶフック
    /// （[`Ticker::set_on_disconnect`]）。`wasm-full::ticker` が保持一覧
    /// からの除去と window 購読の解放をここへ結びつける。
    type DisconnectHook = Rc<RefCell<Option<Box<dyn FnOnce()>>>>;

    impl Ticker {
        /// `root`（`data-fandhe-ticker` 要素）・`content`（最初の
        /// `[data-part="content"]`）を駆動開始する。開始時点で
        /// [`ensure_copies`] を 1 回実行して必要な複製数を満たす。
        #[must_use]
        pub fn start(root: Element, content: Element, config: TickerConfig) -> Self {
            // offset は `root`（CSS 側 `[data-fandhe-ticker-active] [data-part="content"]`
            // の共通祖先）へ書き込む。CSS カスタムプロパティは継承プロパティであり、
            // `content`（複製の 1 個目）へ書き込むと他の兄弟複製へ伝播せず、2 個目
            // 以降が静止したままになる不具合があった（イシュー #2540 レビュー指摘）。
            let (Some(content_html), Some(root_html)) = (
                content.dyn_ref::<HtmlElement>().cloned(),
                root.dyn_ref::<HtmlElement>().cloned(),
            ) else {
                // `HtmlElement` へダウンキャストできない場合は駆動しない
                // no-op（`RafDriver::new` と同じ fail-safe 方針）。
                return Self {
                    _loop: AnimationLoop::start(|| false),
                    hovered: Rc::new(Cell::new(false)),
                    focused: Rc::new(Cell::new(false)),
                    scroll_distance: Rc::new(Cell::new(0.0)),
                    resize_pending: Rc::new(Cell::new(false)),
                    on_disconnect: Rc::new(RefCell::new(None)),
                };
            };

            let viewport_len = measure_len(&root, config.axis);
            let content_len =
                measure_len(&content, config.axis) + read_gap_px(&content_html, config.axis);
            ensure_copies(
                &root,
                &content,
                super::required_copies(viewport_len, content_len),
            );

            // 遅延 hydrate（SSR 表示から JS 駆動開始までに間がある構成）では
            // 利用者が起動前から既に root をポインタ hover・キーボード
            // フォーカス中である場合がある。常に `false` から開始すると、
            // 起動直後の 1 フレームだけ「実際は hover/focus 中なのに動き出す」
            // 停止契約違反が発生する（PR #2582 codex-review P1 指摘）。
            // `matches()` 失敗（対応ブラウザ差異等）は `false` へ fail-safe。
            //
            // ただし `:hover` の初期読み取りは実ポインタ hover を実行できる
            // デバイス（`(hover: hover) and (pointer: fine)`）に限定する:
            // タッチデバイスは tap 後に `:hover` が sticky に残ることがあり
            // （多くのモバイルブラウザの既知挙動）、`pointerover`/`pointerout`
            // 委譲はタッチ由来のポインタを無視するため（`is_touch_pointer`）
            // hydrate 時に `:hover` を true として読むと以後 hover が解除
            // されず ticker が停止したままになる（Cursor Bugbot 指摘、
            // イシュー #2540）。
            let hovered = Rc::new(Cell::new(
                supports_real_hover() && root.matches(":hover").unwrap_or(false),
            ));
            let focused = Rc::new(Cell::new(root.matches(":focus-within").unwrap_or(false)));
            // scroll_velocity 自体は self へは保持しない（AnimationLoop の
            // クロージャが Rc::clone を捕捉して生存させれば十分で、self 経由の
            // 再アクセス手段は不要なため、保持すると `dead_code` になる）。
            let scroll_velocity = Rc::new(Cell::new(0.0));
            let scroll_distance = Rc::new(Cell::new(0.0));
            let resize_pending = Rc::new(Cell::new(false));
            let offset = Rc::new(Cell::new(0.0_f64));
            let last_ms: Rc<Cell<Option<f64>>> = Rc::new(Cell::new(None));
            // `mark_resize`（window `resize` 購読）だけを複製数再計算の契機に
            // すると、非アクティブなタブ配下等 `display:none` の下で起動して
            // 実測値 0 のまま複製 2 個に決まった ticker が、後から表示切替
            // されても `resize` イベントは発火しないため複製数が更新されず
            // 不足したまま残る（PR #2582 codex-review P1 指摘）。既に毎フレーム
            // `measure_len` している値（下の rAF ループ）を使い、前フレームの
            // viewport 長と比較して変化を検知する（新規 API・購読を追加せず
            // 既存の実測ループへ相乗りする方針）。
            let last_viewport_len = Rc::new(Cell::new(viewport_len));
            // viewport 長（root）だけでなく content 長（+ gap）の変化も
            // 複製数再計算の契機にする。固定サイズの root 内でもフォント
            // 読み込み・内容変更で content が短くなる（＝周期が縮む）と、
            // viewport 長は変化しないため上の判定だけでは複製不足を見逃し、
            // 空白を伴って循環し続ける（PR #2582 codex-review P1 指摘）。
            let last_content_len = Rc::new(Cell::new(content_len));

            let step_root = root.clone();
            let step_root_html = root_html;
            let step_content = content;
            let step_content_html = content_html;
            let step_hovered = Rc::clone(&hovered);
            let step_focused = Rc::clone(&focused);
            let step_scroll_velocity = Rc::clone(&scroll_velocity);
            let step_scroll_distance = Rc::clone(&scroll_distance);
            let step_resize_pending = Rc::clone(&resize_pending);
            let step_offset = Rc::clone(&offset);
            let step_last_ms = Rc::clone(&last_ms);
            let step_last_viewport_len = Rc::clone(&last_viewport_len);
            let step_last_content_len = Rc::clone(&last_content_len);
            let on_disconnect: DisconnectHook = Rc::new(RefCell::new(None));
            let step_on_disconnect = Rc::clone(&on_disconnect);

            let animation_loop = AnimationLoop::start(move || {
                // `root` が DOM から切断済み（SPA のルート遷移・コンポーネント
                // 破棄等）なら以後の計測・書き込みは無意味なうえ rAF ループが
                // 無期限に走り続けて CPU を消費し続ける。`wire_ticker` は
                // window にリスナーを `forget()` するため呼び出し元が
                // `Ticker` を明示的に stop できず、マウント/破棄を繰り返す
                // ほどループが累積するリークになっていた（PR #2582
                // codex-review P1 指摘）。`false` を返して `AnimationLoop` を
                // 自己停止させる（`crate::raf_driver::AnimationLoop::start`
                // doc「`step` が `false` を返したら自動停止」契約）。
                if !step_root.is_connected() {
                    // 保持側（`wasm-full::ticker` の `active` 一覧・window
                    // 購読）へ切断を通知する。フックは本 rAF コールバックの
                    // 内側で呼ばれるため、受け手が同期的に本 `Ticker` を
                    // drop すると実行中の `Closure` を解放する use-after-free
                    // になる（`raf_driver::AnimationLoop` doc）。受け手は
                    // 解放を `setTimeout(0)` 等で次のタスクへ遅延させる契約
                    // （[`Ticker::set_on_disconnect`] doc）。
                    if let Some(hook) = step_on_disconnect.borrow_mut().take() {
                        hook();
                    }
                    return false;
                }

                let now_ms = web_sys::window()
                    .and_then(|w| w.performance())
                    .map(|p| p.now());
                let Some(now_ms) = now_ms else {
                    return true;
                };
                let dt_ms = step_last_ms.get().map(|last| now_ms - last).unwrap_or(0.0);
                step_last_ms.set(Some(now_ms));

                let current_viewport_len = measure_len(&step_root, config.axis);
                // 毎フレーム content 長も実測する（後段の offset 前進計算でも
                // 使う値と同一の測り方のため、`ensure_copies` 判定と offset
                // 計算で二重に測って値が食い違うことはない）。
                let current_content_len = measure_len(&step_content, config.axis)
                    + read_gap_px(&step_content_html, config.axis);
                // `resize` イベント経由の明示要求に加え、viewport 長・content
                // 長のいずれかが前フレームから変化していれば（`display:none`
                // → 表示等、`resize` が発火しない経路も含む）複製数を
                // 再計算する（上の `last_viewport_len`/`last_content_len`
                // 初期化コメント参照）。
                let viewport_changed =
                    (current_viewport_len - step_last_viewport_len.get()).abs() > 0.5;
                let content_changed =
                    (current_content_len - step_last_content_len.get()).abs() > 0.5;
                if step_resize_pending.take() || viewport_changed || content_changed {
                    step_last_viewport_len.set(current_viewport_len);
                    step_last_content_len.set(current_content_len);
                    ensure_copies(
                        &step_root,
                        &step_content,
                        super::required_copies(current_viewport_len, current_content_len),
                    );
                }

                // 1 フレーム内で届いた scroll イベントの合計移動距離
                // （`push_scroll_delta` が蓄積）を、このフレームの実経過時間
                // （`dt_ms`）で割って今フレーム分の速度成分を得る。イベント
                // ごとの瞬間速度（delta/イベント間隔）を単純合算する旧実装は
                // 同一フレーム内の細切れイベント数に比例して速度が過大評価
                // される不具合があった（Cursor Bugbot 指摘、イシュー #2540）。
                let frame_distance = step_scroll_distance.replace(0.0);
                // フレーム内にスクロールがあれば今フレームの実測速度をそのまま
                // 採用し、無ければ前回値を減衰させる。「減衰した前回値へ今
                // フレーム分を毎回加算」する実装は、継続スクロール中は
                // 定常状態で `frame_scroll_velocity / (1 - decay)`（decay=0.9
                // なら実測の 10 倍、約 1000px/s）へ発散的に収束する不具合が
                // あった（PR #2582 codex-review P1 指摘）。
                let scroll_velocity_now = if frame_distance != 0.0 && dt_ms > 0.0 {
                    frame_distance / (dt_ms / 1000.0)
                } else {
                    super::decay_velocity(step_scroll_velocity.get())
                };
                step_scroll_velocity.set(scroll_velocity_now);

                // hover（ポインタ）・keyboard focus は一時停止条件（WCAG
                // 2.2.2）だが同列ではない: focus は `hover_factor` に関わらず
                // 常に完全停止（`effective_speed` 側で強制）、hover のみ
                // `hover_factor` で速度調整可能（PR #2582 codex-review P1
                // 指摘）。JS 駆動時は `animation: none` へ切り替わり既存 CSS
                // の `:focus-within` 一時停止が効かなくなるため、`wire_ticker`
                // 側で `focusin`/`focusout` も配線する（`set_focused` 参照。
                // Cursor Bugbot・codex-review 指摘）。
                let speed = super::effective_speed(
                    config.speed_px_s,
                    step_hovered.get(),
                    step_focused.get(),
                    config.hover_factor,
                    scroll_velocity_now,
                    config.scroll_factor,
                );
                let next_offset = super::advance_offset(
                    step_offset.get(),
                    speed,
                    config.direction_sign,
                    dt_ms,
                    current_content_len.max(1.0),
                );
                step_offset.set(next_offset);
                write_offset(&step_root_html, next_offset);
                true
            });

            Self {
                _loop: animation_loop,
                hovered,
                focused,
                scroll_distance,
                resize_pending,
                on_disconnect,
            }
        }

        /// `root` が DOM から切断されたことを rAF ループが検知した時点で
        /// 1 回だけ呼ばれるフックを登録する（後勝ち）。
        ///
        /// フックは rAF コールバックの内側から呼ばれるため、**フック内で
        /// この `Ticker` を同期的に drop してはならない**（実行中の rAF
        /// `Closure` を解放する use-after-free になる）。保持一覧からの
        /// 除去は `setTimeout(0)` 等で次のタスクへ遅延させること
        /// （`wasm-full::ticker` の `schedule_prune` が先例）。
        pub fn set_on_disconnect(&self, hook: impl FnOnce() + 'static) {
            *self.on_disconnect.borrow_mut() = Some(Box::new(hook));
        }

        pub fn set_hovered(&self, hovered: bool) {
            self.hovered.set(hovered);
        }

        /// キーボードフォーカス（`focusin`/`focusout`）による一時停止を
        /// 設定する。JS 駆動時（`animation: none` + `transform` 駆動）は
        /// 既存 CSS の `root:focus-within` 一時停止規則が効かないため、
        /// `wire_ticker` がこのメソッド経由で hover と同じ一時停止契約を
        /// 満たす（モジュール doc・`Ticker::start` 内コメント参照）。
        pub fn set_focused(&self, focused: bool) {
            self.focused.set(focused);
        }

        /// `scroll` イベントのデルタ（px）を今フレーム分の移動距離として
        /// 蓄積する。実際の速度計算は毎フレーム 1 回、`Ticker::start` の
        /// アニメーションループがこの蓄積距離をフレームの実経過時間で
        /// 割って行う（同型パターンは同ループ内コメント参照）。
        pub fn push_scroll_delta(&self, delta_px: f64, dt_ms: f64) {
            if !delta_px.is_finite() || dt_ms <= 0.0 {
                return;
            }
            self.scroll_distance
                .set(self.scroll_distance.get() + delta_px);
        }

        /// 次フレームでの再計測・複製数再調整を要求する。
        pub fn mark_resize(&self) {
            self.resize_pending.set(true);
        }

        pub fn stop(&self) {
            self._loop.stop();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use dom::{
    ensure_copies, is_in_secondary_copy, measure_len, neutralize_nested_tickers, read_gap_px,
    write_offset, Ticker,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_speed_uses_default_for_invalid_input() {
        assert_eq!(parse_speed("abc"), DEFAULT_SPEED_PX_S);
        assert_eq!(parse_speed("-5"), DEFAULT_SPEED_PX_S);
        assert_eq!(parse_speed(""), DEFAULT_SPEED_PX_S);
    }

    #[test]
    fn parse_speed_clamps_to_max() {
        assert_eq!(parse_speed("1e9"), MAX_SPEED_PX_S);
    }

    #[test]
    fn parse_speed_accepts_valid_value() {
        assert_eq!(parse_speed("120"), 120.0);
    }

    #[test]
    fn parse_factor_uses_default_for_invalid_input() {
        assert_eq!(parse_factor("nope", 0.0, 1.0), 0.0);
        assert_eq!(parse_factor("-1", 0.5, 1.0), 0.5);
    }

    #[test]
    fn parse_factor_clamps_to_max() {
        assert_eq!(parse_factor("5", 0.0, 1.0), 1.0);
    }

    #[test]
    fn effective_speed_stops_on_hover_with_default_factor() {
        assert_eq!(effective_speed(80.0, true, false, 0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn effective_speed_halves_on_hover_with_half_factor() {
        assert_eq!(effective_speed(80.0, true, false, 0.5, 0.0, 0.0), 40.0);
    }

    #[test]
    fn effective_speed_adds_scroll_boost() {
        assert_eq!(effective_speed(80.0, false, false, 0.0, 100.0, 0.2), 100.0);
    }

    #[test]
    fn effective_speed_hover_stop_applies_to_scroll_boost_too() {
        // hover 中は既定 hover_factor=0.0 で「完全停止」契約。scroll_boost が
        // 加算後に無視され動き続ける回帰を防ぐ（PR #2582 codex-review P1・
        // Cursor Bugbot 指摘）。
        assert_eq!(effective_speed(80.0, true, false, 0.0, 100.0, 0.2), 0.0);
    }

    #[test]
    fn effective_speed_is_zero_for_invalid_base() {
        assert_eq!(effective_speed(f64::NAN, false, false, 0.0, 0.0, 0.0), 0.0);
        assert_eq!(effective_speed(-1.0, false, false, 0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn effective_speed_focused_always_fully_stops_regardless_of_hover_factor() {
        // キーボードフォーカス中は hover_factor をどう設定しても常に
        // 完全停止する（PR #2582 codex-review P1 指摘）。
        assert_eq!(effective_speed(80.0, false, true, 1.0, 0.0, 0.0), 0.0);
        assert_eq!(effective_speed(80.0, true, true, 1.0, 100.0, 0.2), 0.0);
    }

    #[test]
    fn advance_offset_moves_negative_and_wraps() {
        // speed=100px/s, dt=50ms（MAX_DT_MS(100ms) 未満）→ 5px 前進。cycle_len=200。
        let offset = advance_offset(0.0, 100.0, 1.0, 50.0, 200.0);
        assert!((offset - (-5.0)).abs() < 1e-9);
    }

    #[test]
    fn advance_offset_wraps_within_cycle() {
        // 近い offset からさらに進めて -cycle_len を跨いでも [-cycle, 0] に収まる。
        let offset = advance_offset(-190.0, 100.0, 1.0, 500.0, 200.0);
        assert!((-200.0..=0.0).contains(&offset));
    }

    #[test]
    fn advance_offset_reverses_direction_sign() {
        let forward = advance_offset(-50.0, 100.0, 1.0, 100.0, 200.0);
        let backward = advance_offset(-50.0, 100.0, -1.0, 100.0, 200.0);
        assert!(forward < -50.0);
        assert!(backward > -50.0);
    }

    #[test]
    fn advance_offset_keeps_offset_for_non_positive_cycle_len() {
        // rustdoc の fail-safe 契約（`cycle_len <= 0` は `offset` をそのまま
        // 返す）を、0 以外の `offset` でも検証する。
        assert_eq!(advance_offset(-42.0, 100.0, 1.0, 100.0, 0.0), -42.0);
        assert_eq!(advance_offset(-42.0, 100.0, 1.0, 100.0, -1.0), -42.0);
    }

    #[test]
    fn advance_offset_clamps_huge_dt() {
        // dt が MAX_DT_MS(100ms) にクランプされるため、1 フレームで進む距離は
        // speed * 100ms/1000 を超えない。
        let offset = advance_offset(0.0, 100.0, 1.0, 100_000.0, 1_000_000.0);
        assert!((offset - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn advance_offset_stays_at_zero_when_no_movement() {
        // offset=0・dt_ms=0（起動前 hover/focus 停止中や speed=0 相当）は
        // 移動量ゼロのため `0` のまま（`(-cycle_len, 0]` 契約、PR #2582
        // codex-review P1 指摘）。旧実装は常に `-cycle_len` へ丸めていた。
        assert_eq!(advance_offset(0.0, 0.0, 1.0, 0.0, 200.0), 0.0);
        assert_eq!(advance_offset(0.0, 100.0, 1.0, 0.0, 200.0), 0.0);
    }

    #[test]
    fn required_copies_covers_viewport_with_margin() {
        // viewport=500, content=120 → ceil(500/120)+1 = 6
        assert_eq!(required_copies(500.0, 120.0), 6);
    }

    #[test]
    fn required_copies_clamps_to_min() {
        assert_eq!(required_copies(10.0, 1000.0), MIN_COPIES);
    }

    #[test]
    fn required_copies_clamps_to_max() {
        assert_eq!(required_copies(100_000.0, 1.0), MAX_COPIES);
    }

    #[test]
    fn required_copies_is_min_for_non_positive_content_len() {
        assert_eq!(required_copies(500.0, 0.0), MIN_COPIES);
        assert_eq!(required_copies(500.0, -10.0), MIN_COPIES);
    }

    #[test]
    fn required_copies_increases_when_content_shrinks_at_fixed_viewport() {
        // PR #2582 codex-review P1 指摘の再現数値: viewport=500（root の
        // 表示領域長は固定サイズ・不変）のまま、フォント読み込み・内容
        // 変更で content の周期が 300px → 100px へ縮んだ場合、必要な
        // 複製数は 3 個（ceil(500/300)+1）では表示領域を覆えず、6 個
        // （ceil(500/100)+1）へ増やす必要がある。`Ticker::start` の rAF
        // ループは本テストの純粋な計算結果を、viewport 長だけでなく
        // content 長の変化でも再取得するよう修正した（`step_last_content_len`
        // 比較、モジュール `Ticker::start` 参照）。
        assert_eq!(required_copies(500.0, 300.0), 3);
        assert_eq!(required_copies(500.0, 100.0), 6);
    }

    #[test]
    fn decay_velocity_shrinks_and_settles_to_zero() {
        let v = decay_velocity(100.0);
        assert!((v - 90.0).abs() < 1e-9);
        let mut velocity = 0.02;
        for _ in 0..50 {
            velocity = decay_velocity(velocity);
        }
        assert_eq!(velocity, 0.0);
    }

    #[test]
    fn decay_velocity_is_zero_for_non_finite() {
        assert_eq!(decay_velocity(f64::NAN), 0.0);
        assert_eq!(decay_velocity(f64::INFINITY), 0.0);
    }

    #[test]
    fn offset_var_is_stable_literal() {
        assert_eq!(TICKER_OFFSET_VAR, "--fandhe-marquee-ticker-offset");
    }

    #[test]
    fn ticker_attr_literals_are_stable() {
        assert_eq!(TICKER_ATTR, "data-fandhe-ticker");
        assert_eq!(TICKER_ACTIVE_ATTR, "data-fandhe-ticker-active");
    }
}
