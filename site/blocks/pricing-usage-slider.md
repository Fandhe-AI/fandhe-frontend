# pricing-usage-slider

`fandhe-frontend-pre-styled-ui` の `slider` / `stat` 部品を合成した、
Motion+ `sections/pricing-sections` に相当する使用量ベース料金の合成例
です。Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例です。docs サイトは JS ハイドレーションを一切行わ
ないため、`slider` は固定の初期値で描画され、対応する価格を `stat` へ
表示するのみです。**実運用でスライダーの値をリアルタイムに `stat` へ
反映する処理（ライブ連動）は実装していません**。それを組み込む場合は、
`fandhe-frontend-wasm-full` のハイドレーション配線を購読して価格表示を
書き換える処理を、利用者自身の Rust コードで実装してください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はバリデーション・データ整形等のアプリケーションロジックを内包
しません）。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::slider::{self, SliderProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 利用量（月間リクエスト数、架空の目盛り）から価格表示を導く決定的な固定
/// テーブル。範囲は 4 段のプリセット目盛り（10/50/100/500）を跨ぐしきい値
/// で区切っており、外部入力・ユーザー入力を一切受け取らない。
#[must_use]
fn price_for(units: u32) -> &'static str {
    match units {
        0..=10 => "$9",
        11..=50 => "$29",
        51..=100 => "$49",
        _ => "$199",
    }
}

/// `pricing-usage-slider` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。スライダーは 4 段のプリセット目盛り（10/50/100/500）の中間
/// （50）に固定した初期値で描画し、対応する価格を [`price_for`] から
/// 求めて `stat` へ表示する。
pub fn demo() -> Node {
    let props = SliderProps::default();
    let selected_units = 50.0_f64;
    let state = Slider::new(0.0, 500.0, 10.0, selected_units, Orientation::Horizontal);

    let slider_node = slider::root(
        Size::Md,
        ColorPalette::Accent,
        &state,
        &props,
        vec![("data-blocks-pricing-usage-slider-slider", "")],
        vec![
            slider::label(
                &props,
                vec![("id", "blocks-pricing-usage-slider-label")],
                vec![text("月間リクエスト数（千件）")],
            ),
            slider::control(
                Orientation::Horizontal,
                &props,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        &props,
                        vec![],
                        vec![slider::range(&state, &props, vec![])],
                    ),
                    slider::thumb_styled(
                        &state,
                        Some("50 千件"),
                        &props,
                        vec![("aria-labelledby", "blocks-pricing-usage-slider-label")],
                    ),
                    slider::marker_group(
                        vec![],
                        vec![
                            slider::marker(&state, 10.0, false, vec![], vec![]),
                            slider::marker(&state, 50.0, false, vec![], vec![]),
                            slider::marker(&state, 100.0, false, vec![], vec![]),
                            slider::marker(&state, 500.0, false, vec![], vec![]),
                        ],
                    ),
                ],
            ),
            slider::hidden_input("usage-units", "50", false, vec![]),
        ],
    );

    let stat_node = stat::root(
        Size::Lg,
        vec![("data-blocks-pricing-usage-slider-stat", "")],
        vec![
            stat::label(vec![], vec![text("想定コスト")]),
            stat::value_text(
                vec![],
                vec![text(price_for(selected_units as u32)), text(" / 月")],
            ),
            stat::help_text(
                vec![],
                vec![text(
                    "スライダーの選択位置（50 千件/月）に対応する固定表示です",
                )],
            ),
        ],
    );

    div(
        vec![("data-blocks-pricing-usage-slider-layout", "")],
        vec![slider_node, stat_node],
    )
}
```

## shadcn / Motion+ 側との構成上の判断

- **`slider` と `stat` を静的な組で示す**: docs サイトは JS ハイドレーション
  を一切行わないため、Demo 上のスライダーは常に固定値で描画されます。
  「slider の値を stat へ反映する」というライブ連動そのものは UI コンポー
  ネント層の責務外（`docs/policy/intentional-non-adoption.md` §3.25）で
  あり、本 block は「同じ値を指し示す静的な組」を提示するのみです。
- **`price_for` は決定的な固定テーブル**: 外部入力・ユーザー入力を一切
  受け取らない範囲テーブル（`match units { 0..=10 => ..., ... }`）であり、
  数値のカウントアップアニメーション（別 issue #2539 の担当）とは無関係
  です。
- **4 段のプリセット目盛り**: `marker_group`/`marker` で 10/50/100/500 の
  4 点を目盛りとして表示し、初期値はその中間（50）に固定しています。
- **利用量・価格・プラン名は架空**: 実企業名・実クレデンシャル・PII を
  含みません。

関連情報: [Slider](../themes/slider.md) / [Stat](../themes/stat.md)
