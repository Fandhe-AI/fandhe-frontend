# pre-styled-ui スケールトークン設計（radius / shadow / spacing / z-index、イシュー #1423）

## 1. 位置づけ

親イシュー #1421（Phase 0: スタイル調整の共通基盤）の一環として、
`crates/pre-styled-ui/src/theme.rs` の角丸（radii）・影（shadows）・
余白（spaces）の既定スケールを拡充し、新規に重なり順（z-index）トークン
グループを追加する。Phase 1 以降で 107 部品を chakra-ui / Radix Themes
基準へ個別調整する前提として、部品側が載せられる「段階」を先に固定する
ことが目的であり、**部品ソース側のリテラル値をトークン参照へ置換する
一括作業は本イシューのスコープ外**（§5 の棚卸しが後続 Phase の各部品
issue の消し込み対象）。

兄弟イシュー #1422（色トークン）・#1424（focus ring / size）も同じ
`theme.rs` を変更対象とするため、本イシューは `DEFAULT_COLORS` と
color 系 API（`push_color`/`upsert_color`/`color_var`）には一切触れず、
radii/shadows/spaces の定数ブロックと新設 z-index グループのみを変更して
コンフリクト面を最小化する。

## 2. 参照元と一次記録

Radix Themes 側のトークン体系の一次記録は `docs/design/radix-themes-survey.md`
§4.2〜§4.4（radius/spacing/shadow）にあり、本書では書き写さず参照する。
chakra-ui 側は `.agents/skills/chakra-ui/references/theming/design-tokens/`
配下（本リポジトリのローカル skill 参照ファイル）を一次根拠とする。
両者との対比表は `docs/design/radix-themes-survey.md` §5 に追記済み
（本イシューでの拡充分を含む）。

## 3. スケール定義と設計判断

### 3.1 radius（5 段 → 8 段）

既存 5 段（`sm`/`md`/`lg`/`xl`/`full`）は名前・値とも不変。純追加した
3 段:

| name | value | 根拠 |
|------|-------|------|
| `none` | `0` | 意図的な角無し（mark / textarea attached / avatar square）用 |
| `xs` | `0.125rem` | chakra-ui `xs` 相当。密なインライン部品向け |
| `2xl` | `1rem` | chakra-ui `2xl` 相当。大きめの面パネル向け |

chakra-ui の `2xs`/`3xl`/`4xl`、Radix の `thumb`/`factor` は fandhe 部品に
用途がないため見送った（`docs/design/radix-themes-survey.md` §5 参照）。

Radix `--radius-1..6` との対応: `1`≈`xs`、`2`≈`sm`、`3`≈`md`、`4`≈`lg`、
`5`≈`xl`、`6`≈`2xl`。

**部品カテゴリごとの既定段割り当て方針**（後続 Phase の各部品 issue が
適用する。本イシューでは部品ソースを変更しない）:

| カテゴリ | 段 | 対象例 |
|---------|-----|--------|
| 操作部品 | `md` | button / input / select trigger / textarea / checkbox |
| 密なインライン部品 | `sm` | badge / tag / kbd / code / menu item / tooltip / tree item |
| 面パネル | `lg` | card / dialog / drawer / popover / menu content / hover-card / floating-panel |
| pill・円形 | `full` | avatar / switch / radio / steps indicator / color-swatch circle |
| 意図的な角無し | `none` | mark / textarea attached / avatar square |

`50%`（真円）は radio 系のような真円意図が明示的な箇所のみ許容し、
`full`（`9999px`）への統一は行わない。

### 3.2 shadow（4 段 → 6 段）

既存 4 段（`xs`/`sm`/`md`/`lg`）は不変。純追加:

| name | light | dark |
|------|-------|------|
| `xl` | `0 20px 25px rgba(0, 0, 0, 0.2)` | `0 20px 25px rgba(0, 0, 0, 0.5)` |
| `2xl` | `0 25px 50px rgba(0, 0, 0, 0.25)` | `0 25px 50px rgba(0, 0, 0, 0.55)` |

dark 値は既存規則（light 比で不透明度を上げる）を踏襲した。chakra-ui の
`inner`/Radix の inset 相当は見送った（fandhe の inset 用途は現状すべて
ドット・リング・マスク描画であり「影」ではないため）。

**overlay 系の割り当て方針**（後続 Phase 適用）: dropdown 型（menu /
menubar / select / combobox / popover / hover-card / navigation-menu /
floating-panel / date-picker）= `md`（現行の生値
`0 4px 6px rgba(0, 0, 0, 0.15)` は `md` light 値
`0 4px 6px rgba(0, 0, 0, 0.1)` へ寄せる）、dialog / drawer content = `lg`、
toast / action-bar = `lg`、tooltip = `sm`。

**ダーク時の方針の再評価トリガー**: Radix は「ダークでは影を弱め border
で境界を出す」方式だが、fandhe の overlay 系部品は既に全部品が
`border: 1px solid var(--fandhe-color-border)` を持ち境界は border が
担保済みのため、dark 値は既存規則（不透明度を上げる）を維持する。この
判断は色トークン（#1422）確定後に再評価する。

### 3.3 spacing（10 段 → 15 段）

既存 10 段（`1,2,3,4,5,6,8,10,12,16`）は不変。純追加:

| name | value | chakra 相当 |
|------|-------|-------------|
| `0-5` | `0.125rem`（2px） | `0.5` |
| `1-5` | `0.375rem`（6px） | `1.5` |
| `2-5` | `0.625rem`（10px） | `2.5` |
| `20` | `5rem` | `20` |
| `24` | `6rem` | `24` |

[`TokenName`] は `.` を許可しない（CSS custom property 名の一部として
安全な文字集合に制限するため）ため、chakra の `0.5`/`1.5`/`2.5` 相当は
`-` 区切り（`0-5`/`1-5`/`2-5`）で表記する。

4px 格子に載らない既存生値（`0.0625rem`/`0.1rem`/`0.15rem`）は §5 の
棚卸しで「後続 Phase で `0-5` または `1` へ丸める」対象として記載する。
visually-hidden 系の `-1px` はレイアウト外へ意図的に配置するための負値
であり、トークン化対象外とする。

### 3.4 z-index（新規グループ、12 段）

`Theme` にモード非依存の新規グループ `z_indices: Vec<ScaleToken>` を
追加した。出力プレフィックスは既に `toast.rs` が使っていた
`--fandhe-z-index-<name>` を踏襲する
（`crates/docs-site/tests/css_var_scope_prefix.rs` の `collect_fandhe_var_names`
は `--fandhe-` 前方一致で収集するため名前衝突なし）。

既定値（chakra-ui の `hide`〜`max` を参考にした 100 刻み。fandhe の現行
生値からの移行が単調になる順序で選定）:

| name | value | 割り当て予定（後続 Phase） | 現行生値 |
|------|-------|---------------------------|----------|
| `hide` | `-1` | — | — |
| `base` | `0` | link-overlay | `0` |
| `docked` | `10` | 非オーバーレイの浮き要素（sticky header 等） | — |
| `dropdown` | `1000` | menu / menubar / select / combobox / navigation-menu / date-picker positioner | `10` |
| `sticky` | `1100` | action-bar / floating-panel | `900` |
| `popover` | `1200` | popover / hover-card / toggle-tip | `10` |
| `overlay` | `1300` | dialog / drawer backdrop、tour backdrop | `1000`/`1100` |
| `modal` | `1400` | dialog / drawer positioner、tour positioner | `1001`/`1101` |
| `skip-nav` | `1500` | skip-nav | `1200` |
| `toast` | `1600` | toast group | `var(..., 9999)` |
| `tooltip` | `1700` | tooltip | `1100` |
| `max` | `2147483647` | 緊急用 | — |

順序は「dropdown < sticky < popover < overlay < modal < skip-nav < toast <
tooltip」を満たす。dialog と drawer は同段（`overlay`/`modal`）とし、
同時表示時の前後関係は DOM 順に委ねる（chakra も同段）。chakra の
`banner`（1200）は fandhe に対応部品がないため見送った。`skip-nav` は
[`TokenName`] 規則（`[a-z0-9][a-z0-9-]*`）に沿いケバブ表記とした。

**実装結果の注記（イシュー #1550）**: tour の `backdrop`/`spotlight`/
`positioner` トークン化にあたり、上表が予定していた「spotlight →
`tooltip`（1700）」割り当てには従わなかった。spotlight は `box-shadow` で
画面全体を暗くするマスク要素であり、`modal`（1400）の positioner/content
より前面（`tooltip` 段）に置くと tour の content カード（および同段の
dialog content）まで覆ってしまうため、backdrop と同じ `overlay` 段の
`calc(var(--fandhe-z-index-overlay, 1100) + 1)` に固定した（backdrop <
spotlight < positioner の順序を DOM 順に依存せず保証する）。positioner は
上表どおり `modal` 段（`--fandhe-z-index-modal`）へ移行済み。

Radix Themes 自体には `--z-index-*` 相当の公開トークン変数は確認できず
（`docs/design/radix-themes-survey.md` の取得範囲では未確認）、本グループ
は chakra-ui のみを参照元とする。

API は既存 `push_scale`/`upsert_scale`（内部共通ヘルパ）を再利用した
`push_z_index`/`upsert_z_index`、および `var(--fandhe-z-index-<name>)`
参照ヘルパ `z_index_var` を追加した（新たな検証迂回路を作らない）。
`Theme::to_css()` は radii の後・shadows の前ではなく **shadows の後
（`:root` ブロック末尾）** に z-indices を出力する。既存出力のバイト
同一性（z-indices を push しないテーマの出力は #1423 前と同一）を
`crates/pre-styled-ui/tests/theme_css.rs` の回帰テストで固定している。

**toast の正式トークン化**: `toast.rs` が使っていた未宣言変数
`--fandhe-z-index-toast`（`var(--fandhe-z-index-toast, 9999)` という
fallback 付き参照）を受けて `Theme::default()` 側に正式トークンとして
`--fandhe-z-index-toast: 1600;` を追加した。これに伴い
`crates/docs-site/tests/css_var_scope_prefix.rs` の `SHARED_VARS`
（免除表）から `--fandhe-z-index-toast` を削除した（`theme_token_names()`
が `Theme::default().to_css()` から自動収集するため、免除を残すと
`shared_vars_table_has_no_stale_entries` が stale エントリとして FAIL
する）。**`toast.rs` 側の `var(--fandhe-z-index-toast, 9999)` fallback
自体は除去せず維持する**（codex-review #1705 P1 指摘・実装時追記）:
`Theme::empty()` から必要トークンのみ構築する既存利用者や
`toast::stylesheet()` を単独利用しテーマ CSS を注入しない利用者では
`--fandhe-z-index-toast` が未定義のままになり得る。CSS カスタム
プロパティが unset の場合 `var()` に fallback がないと宣言全体が無効化
され `z-index` ごと失われ、Toast の重なり順（最前面表示）が壊れる。
公開クレート（crates.io 公開済み）の既存 CSS 契約を壊さないため、正式
トークン化後も fallback は残す。

### 3.5 shadcn 基準値+派生スケールの非採用（イシュー #2005）

shadcn/ui の `--radius` は単一の基準値（既定 `0.625rem`）を宣言し、
`sm`/`md`/`lg`/`xl` の各段を CSS `calc()` で派生させる方式
（`--radius-sm: calc(var(--radius) - 4px)` 等）を採る。本書 §3.1 は
#1423 で「8 段の独立した named scale」を意図的に採用しており、これは
基準値+派生とは異なる設計軸である。

**値ベースで比較すると同名でも段がずれる**:

| 段名 | shadcn（基準 `0.625rem` からの派生） | fandhe（§3.1 の固定値） | 差 |
|------|--------------------------------------|--------------------------|----|
| `sm` | `calc(0.625rem - 4px)` ≈ `0.375rem` | `0.25rem` | fandhe `sm` は shadcn `sm` よりさらに 1 段小さい |
| `md` | `calc(0.625rem - 2px)` ≈ `0.5rem` | `0.375rem` | shadcn `md` ≈ fandhe `lg` |
| `lg` | `0.625rem`（基準値そのもの） | `0.5rem` | shadcn `lg` は fandhe `lg` と `xl` の間 |
| `xl` | `calc(0.625rem + 4px)` ≈ `0.875rem` | `0.75rem` | shadcn `xl` は fandhe `xl` と `2xl` の間 |

同じ段名（`sm`/`md`/`lg`/`xl`）で単純に対応付けると、実際には fandhe
側で 1 段小さい／大きい丸みになる（例: shadcn の `sm` は値としては
fandhe の `md` に近い）。後続 Phase で shadcn 系マークアップを参照する
実装者は、段名の一致だけで置換せず本表の値を確認すること。

**`CssValue::new`（本ファイル）が `calc(`/`var(` を許可文字集合上
「たまたま」通す事実の切り分け**: `CssValue::new` の許可文字は英数字・
空白・`# % . , ( ) - _ +` のみで `:` 等は拒否するが、`calc(` `var(` に
必要な文字はいずれも許可されているため、技術的には利用者がベース
トークン（例: `theme.push_radius("base", "0.625rem")`）を自前で用意し、
各段の値を `calc(var(--fandhe-radius-base) - 2px)` のような文字列として
`push_radius`/`upsert_radius` に渡すこと自体は現状の検証ロジック上
ブロックされない。ただしこれは「たまたま通る」に過ぎず、フレームワーク
側が基準値+派生方式を正式サポートする第一級 API（ベーストークンの出力
順保証・`Theme::to_css` 側の依存順考慮・派生段のバリデーション等）は
用意しない。

**結論: 非採用**。§3.1 で確定した 8 段独立スケールの設計判断
（AI 保守の明示性・機械検証可能性を優先し、Theme provider 前提の
基準値+派生方式を採らない）を維持する。将来的に「1 値で全体を動かす」
利便性 API（例: `radii_from_base()` ヘルパー）が要求された場合の検討は
本書のスコープ外の将来検討事項として記録するに留め、本イシューでは
提案しない。

### 3.6 breakpoint（新規グループ、4 段、イシュー #2197）

PR #2142（breadcrumb の shadcn/ui 突合）で、shadcn の `sm:gap-2.5`
（`>= 640px` で gap を広げる）に相当する `@media (min-width: …)` を
`crates/pre-styled-ui/src/recipe.rs` の `SlotRecipe` が表現できず、
横断設計判断として見送られていた（`breadcrumb.rs`/`sidebar.rs` の
モジュール doc に未実装記録あり）。本イシューは `recipe.rs` に
`@media (min-width: ...)` を表現する breakpoint 条件を、`theme.rs` に
対応するブレークポイントトークンを追加する。

**中核制約: CSS custom property は `@media` プレリュードで使えない**。
`@media (min-width: var(--fandhe-breakpoint-sm))` は無効な CSS である。
したがって「テーマトークン」と「recipe が `@media` に埋め込む値」は
必然的に別物になる:

- `recipe.rs` 側: `Breakpoint` enum の `const fn min_width(self) ->
  &'static str` がリテラル（例 `"640px"`）を返し、これが**唯一のリテラル
  定義元**。`SlotRecipe::css` はこの `&'static str` を `@media
  (min-width: {min_width})` へ埋め込む
- `theme.rs` 側: `DEFAULT_BREAKPOINTS` は `Breakpoint::ALL` の
  `value()`/`min_width()`（いずれも `const fn`）から構築し、独立した
  リテラルの手打ちを行わない。`Theme::default()` が `push_breakpoint` で
  `:root` へ `--fandhe-breakpoint-<段>: <px>` を出力する
- テーマ側の値を `upsert_breakpoint` で変えても、各 recipe の `@media`
  出力は変わらない。テーマトークンは**参照専用**（JS の
  `matchMedia`・利用者の独自スタイルシート・
  `getComputedStyle(document.documentElement).getPropertyValue(...)` 用途）
  として提供する、CSS の仕様上の制約に由来する既知の限界である

**API 形状**: `StateCondition` の variant ではなく、並列の enum +
専用 builder として追加した（`Breakpoint` enum・
`SlotRecipe::breakpoint(slot, bp, declarations)`）。既存 `StateCondition`
へ variant を追加すると下流の網羅 `match` を壊すため、「`StateCondition`
と並ぶ条件」という要件を並列の enum で満たす純追加とした。

**スケール値の決定**:

| 段 | chakra-ui v3 | Radix Themes | shadcn/ui（Tailwind v4） | **fandhe 採用** |
|----|-------------|--------------|--------------------------|-----------------|
| xs | — | 520px | — | 不採用 |
| sm | 480px | 768px | 640px | **640px** |
| md | 768px | 1024px | 768px | **768px** |
| lg | 1024px | 1280px | 1024px | **1024px** |
| xl | 1280px | 1640px | 1280px | **1280px** |
| 2xl | 1536px | — | 1536px | 見送り（下記） |

根拠: chakra-ui と shadcn/ui は `sm` 以外で完全一致する。最初の消費者
（#2198 の `sm:gap-2.5`）は shadcn の 640px を前提としており、shadcn は
#2153 で主基準の 1 つ。Radix Themes の名前付き段（`initial`/`xs`〜`xl`、
値が 1 段ずれる）は不採用。`2xl` は `Size`（§3.1〜§3.3 と同型の t-shirt
語彙 enum）の「共通 enum に載せると全部品が空の段を抱える」前例と同じ
判断で見送った。傍証: `crates/wasm-full` の sidebar
`DEFAULT_MOBILE_MEDIA_QUERY = "(max-width: 767px)"`（shadcn
`MOBILE_BREAKPOINT = 768`）は `md − 1px` と一致する。

**出力順序**: `SlotRecipe::css()` の出力順は「base → variants →
compound variants → states → breakpoints（`Breakpoint::ALL` の昇順、
mobile-first）→ hover（`@media (hover: hover)`、常に最後尾）」に固定
した。breakpoint 間は登録順ではなく enum 昇順（`sm` → `xl`）で出力し、
同一 breakpoint 内は登録順（後勝ち）。hover ブロックより前に置く（既存
契約「hover は最後尾」を壊さない）。`@media` ブロックのインデント処理
（2 スペース・規則間の空行保持）は hover ブロック生成処理から
`push_media_block` として抽出し、breakpoint と共用した（hover の
バイト列は不変）。

**詳細度の注意**: breakpoint 規則のセレクタは base と同じ
`[data-scope][data-part]`（0,2,0）のため、同一 slot・同一プロパティを
variant（0,3,0）が宣言していると、`@media` の内外に関わらず variant が
常に勝つ。呼び出し元は対象 slot・プロパティが variant で宣言されて
いないことを確認する（#2198 の breadcrumb `list` は `gap` を base のみで
宣言しており衝突なし、確認済み）。

**テーマ側の値検証**: `#1423`（z-index）・`#1424`（focus-ring）と同じ
判断軸で、`CssValue` の文字 allowlist は通るがプロパティとして無意味な
値（色・`var()`・`calc()`・CSS-wide keyword・負値）を
`validate_breakpoint_value` が個別に拒否する。許可するのは非負の CSS
`<length>`（数値 + 単位、または単位なしの `0`）のみ。専用エラー
`ThemeError::InvalidBreakpointValue` を追加した。

**本イシューでは実装しないこと**（breakpoint × variant / breakpoint ×
state の複合条件、`max-width`/range 構文、container query）は §7 の
再評価トリガーへ記録する。

### 3.7 container query（新規グループ、4 段、イシュー #2199）

PR #2147（field の shadcn/ui 突合）で、shadcn `Field` の
`orientation="responsive"`（`FieldGroup` の `@container/field-group` を
基準に `@md/field-group` 以上で縦積み→横並びへ切り替える）に相当する
`@container` クエリを `SlotRecipe` が表現できず、「本リポジトリに
`@container` の前例が皆無であり単一部品のための新規 CSS 機構導入は
横断設計判断」として見送られていた（`field.rs` モジュール doc「見送った
もの」に記録あり）。本イシューは `recipe.rs` に `@container` を表現する
container query 条件を追加し、field を最初の消費者として実装する。

**container slot（`container-type` を持つ要素）の決め方**: container は
recipe が宣言する 1 slot（`SlotRecipe::container_slot(slot)`）とする。
名前付き container を必須とし、`fd-<scope>-<slot>` を recipe が
`scope`/`slot`（いずれも `is_valid_identifier` 検証済み）から決定的に
導出する。無名 `@container` は不採用（将来他部品が `container-type` を
持った時点で最近傍 container に束縛され本 recipe の `@container` 規則が
黙って壊れるため）。1 recipe につき container slot は 1 つで、複数回
呼んだ場合は最後の呼び出しが上書きする（builder の素直な意味論）。
`container-type: inline-size` は要素の内在インライン寸法を 0 とみなす
制約を持つため、`width: 100%` 等で解決できる block flow の子でのみ
期待どおり動く（field の `group` は既に `width: 100%` を持つ）。

**`@container` 条件の宣言手段**: 新 enum `ContainerBreakpoint`（`sm`=
384px/`md`=448px/`lg`=512px/`xl`=576px、shadcn/ui Tailwind v4 既定の
コンテナクエリスケールと一致）を `Breakpoint` と並列・独立に追加した
（`StateCondition` へ variant を追加する形は既存の「下流の網羅 `match`
を壊さない純追加」判断（§3.6）を踏襲し不採用）。builder は 2 種:
`SlotRecipe::container(slot, cb, declarations)`（base セレクタ、詳細度
(0,2,0)）と `SlotRecipe::container_variant(v, slot, cb, declarations)`
（variant クラス付きセレクタ、詳細度 (0,3,0)）。`container_variant` が
無いと responsive（container 幅に応じて特定 variant クラスの宣言だけを
切り替える）が実装不能なため、breakpoint（§3.6）では「複合条件は未実装」
としていた判断を、**container × variant に限って**採用へ改めた
（breakpoint × variant / × state・container × state は引き続きスコープ
外のまま、§7 参照）。`container_variant` は `variant()` と同じく事前
登録を要求しない（識別子検証のみ）。

**Theme トークンは追加しない**: breakpoint トークン
（`--fandhe-breakpoint-<段>`）は JS の `matchMedia` 等の参照用途を持つが、
container query には JS 側の等価 API が無く、CSS custom property は
`@container` プレリュードでも使えない制約は breakpoint と同じ（§3.6の
「中核制約」節参照）。参照専用トークンを追加する動機自体が無いため、
container query 用の Theme トークンは新設しない。

**出力順序**: `SlotRecipe::css()` の出力順を「… → breakpoints
（`Breakpoint::ALL` の昇順）→ container query（`ContainerBreakpoint::ALL`
の昇順、`container_slot` が有効な場合のみ）→ hover（常に最後尾）」へ
拡張した。container query も breakpoint と同じ「`@media`/`@container` の
at-rule ブロック群が末尾に集約される」既存契約・「hover は最後尾」契約を
保つ位置。`container-type`/`container-name` の 2 個目 base ブロックは
当該 slot の base ブロック群の直後に中間挿入する（accordion #2192 の
`item-content` 2 個目 base ブロックと同型）。

**fail-closed**: container slot 未宣言（`container_slot` を呼んでいない、
または宣言した slot が `slots` 未宣言・不正識別子）の場合、`container()`/
`container_variant()` に登録された規則は一切出力しない（孤児 `@container`
を出さない）。個別規則の未宣言 slot・不正識別子・有効な宣言ゼロは
breakpoint と同じくスキップする。純追加不変条件（`container_slot`/
`container`/`container_variant` を呼ばない recipe の `css()` はバイト
不変）も同様に維持する。

## 4. 対象ファイル

| パス | 変更内容 |
|------|----------|
| `crates/pre-styled-ui/src/theme.rs` | `DEFAULT_RADII`/`DEFAULT_SHADOWS`/`DEFAULT_SPACES` への純追加、`DEFAULT_Z_INDICES` 新設、`Theme` へ `z_indices` フィールド、`push_z_index`/`upsert_z_index`/`z_index_var`、`to_css()` 末尾出力、ユニットテスト。イシュー #2197 で `breakpoints` フィールド・`DEFAULT_BREAKPOINTS`・`push_breakpoint`/`upsert_breakpoint`/`breakpoint_var`・`ThemeError::InvalidBreakpointValue`・`validate_breakpoint_value` を追加 |
| `crates/pre-styled-ui/src/recipe.rs` | イシュー #2197 で `Breakpoint` enum（`ALL`/`value`/`min_width`）・`SlotRecipe::breakpoint` builder・`css()` の breakpoint ブロック出力（`push_media_block` ヘルパへの hover ブロック共用抽出込み）を追加 |
| `crates/pre-styled-ui/src/toast.rs` | `z-index` を正式トークン参照へ更新（fallback は後方互換のため維持） |
| `crates/pre-styled-ui/tests/toast_css.rs` | golden CSS の z-index 行を追随 |
| `crates/pre-styled-ui/tests/theme_css.rs` | z-index の出力構造 golden・var helper 一致・dark ブロック不在の確認。イシュー #2197 で breakpoint の同種テストを追加 |
| `crates/pre-styled-ui/tests/theme_injection.rs` | `push_z_index`/`upsert_z_index` のインジェクション payload・重複拒否テスト。イシュー #2197 で `push_breakpoint`/`upsert_breakpoint` の同種テストを追加 |
| `crates/pre-styled-ui/tests/recipe_css.rs` | イシュー #2197 で breakpoint の golden・出力順序・fail-closed テストを追加 |
| `crates/pre-styled-ui/Cargo.toml` | `0.40.6` → `0.41.0`（公開 API 追加）。イシュー #2197 で `0.176.0` → `0.177.0`（公開 API 純追加） |
| `crates/docs-site/tests/css_var_scope_prefix.rs` | `SHARED_VARS` から `--fandhe-z-index-toast` を削除 |
| `docs/api/pre-styled-ui-api.md` §4l | 新 API のシグネチャ追記 |
| `docs/design/radix-themes-survey.md` §5 | fandhe 側の段数更新・z-index 行追加。イシュー #2197 で breakpoints 行を更新 |
| `crates/pre-styled-ui/src/recipe.rs` | イシュー #2199 で `ContainerBreakpoint` enum（`ALL`/`value`/`min_width`）・`SlotRecipe::container_slot`/`container`/`container_variant` builder・`css()` の container-type 中間挿入と `@container` ブロック出力を追加 |
| `crates/pre-styled-ui/src/field.rs` | イシュー #2199 で `FieldOrientation::Responsive` を追加し、`group` を container slot・`root` を `container_variant` の対象として登録 |
| `crates/pre-styled-ui/tests/recipe_css.rs` | イシュー #2199 で container query の golden・出力順序・fail-closed・純追加不変条件テストを追加 |
| `crates/pre-styled-ui/tests/field_css.rs` | イシュー #2199 で `orientation="responsive"` の golden 差分・container ブロック位置テストを追加 |
| `crates/pre-styled-ui/Cargo.toml` | イシュー #2199 で `0.180.4` → `0.181.0`（`FieldOrientation` へ新 variant 追加、0.x の破壊的変更） |

## 5. 部品ソースの「トークン外の生の値」棚卸し（後続 Phase の消し込み対象）

以下は本イシュー実装時点（2026-08-31、`crates/pre-styled-ui/src/` 配下）の
実測。Phase 1 以降の各部品 issue がこの一覧を元に生値をトークン参照へ
置換する（本イシューでは部品ソースを変更しない）。再取得コマンドは
各節に記載した `grep` をそのまま使う。

### 5.1 z-index（`decl("z-index", "<生値>")`、`var(--fandhe-z-index-*)` を除く）

再取得: `grep -rn '"z-index"' crates/pre-styled-ui/src/*.rs | grep -v z-index-toast`

25 箇所（`toast.rs` は正式トークン参照へ移行済みのため対象外）:

| 現行生値 | 出現箇所（抜粋、file:line） |
|----------|------------------------------|
| `0` | `link_overlay.rs:51`、`segment_group.rs:139` |
| `1` | `color_picker.rs:203`、`segment_group.rs:184` |
| `10` | `combobox.rs:180`、`date_picker.rs:100`、`hover_card.rs:105`、`menu.rs:156`、`menubar.rs:145,207`、`navigation_menu.rs:112`、`popover.rs:115`、`select.rs:153` |
| `900` | `action_bar.rs:91`、`floating_panel.rs:106` |
| `1000`/`1001` | `dialog.rs:142,151`、`drawer.rs:103,112` |
| `1100` | `tooltip.rs:98`、`toggle_tip.rs:102` |
| `1200` | `skip_nav.rs:86` |

移行予定は §3.4 の表（`dropdown`/`sticky`/`popover`/`overlay`/`modal`/
`skip-nav`/`tooltip`）を参照。`tour.rs` の `1100`/`1101`/`1102`（旧生値）は
イシュー #1550 でトークン化済み（backdrop/spotlight は
`--fandhe-z-index-overlay`、positioner は `--fandhe-z-index-modal`。
実装結果は §3.4 の注記を参照）のため、本表からは除外した。

### 5.2 box-shadow（`decl("box-shadow", "<生値>")`、`var(--fandhe-shadow-*)` を除く）

再取得: `grep -rn '"box-shadow"' crates/pre-styled-ui/src/*.rs`

トークン参照は `shadow-sm`（`card.rs`/`segment_group.rs` の fallback 付き
参照を含む）・`shadow-md`（`toast.rs`）の計 3 箇所のみ、残り約 25 箇所が
生値。うち overlay 系 9 部品（`menu.rs`/`menubar.rs`×2/`select.rs`/
`combobox.rs`/`popover.rs`/`hover_card.rs`/`navigation_menu.rs`/
`floating_panel.rs`/`date_picker.rs`）が同一値
`0 4px 6px rgba(0, 0, 0, 0.15)` を持ち、§3.2 の割り当て方針で `shadow-md`
（`0 4px 6px rgba(0, 0, 0, 0.1)`）へ寄せる予定。`action_bar.rs:106`
（`0 0.25rem 1rem rgba(0, 0, 0, 0.15)`）は `shadow-lg` へ寄せる予定。
残りは影ではなくドット・リング・マスク描画用途（
`checkbox_card.rs`/`radio_card.rs`/`radio_group.rs` のリング、
`color_picker.rs` の `0 0 0 1px`、`tour.rs`/`image_cropper.rs` のマスク
`0 0 0 max(100vw, 100vh)`/`0 0 0 9999px`、`splitter.rs`、`skip_nav.rs`、
`date_input.rs`）であり、トークン化対象外として棚卸しに残す。

### 5.3 border-radius（`decl("border-radius", "<生値>")`、`var(--fandhe-radius-*)` を除く）

再取得: `grep -rn '"border-radius"' crates/pre-styled-ui/src/*.rs`

トークン参照済みが約 63 箇所、生値が残り約 69 箇所。代表的な生値と
対応予定トークン: `0.375rem`（`date_picker.rs`/`menubar.rs`/
`listbox.rs`/`combobox.rs`/`popover.rs`/`hover_card.rs`/`select.rs`/
`signature_pad.rs` 等）→ `radius-md`、`0.25rem`（`tooltip.rs`/
`tree_view.rs`/`menubar.rs`/`listbox.rs`/`combobox.rs`/
`signature_pad.rs`）→ `radius-sm`、`0.5rem`（`dialog.rs`）→ `radius-lg`、
`999px`/`9999px`（`switch.rs`/`angle_slider.rs`/`steps.rs`/
`color_swatch.rs`）→ `radius-full`、`50%`（`radio_group.rs`/
`radio_card.rs`、真円意図が明示的）→ 対象外のまま許容、`0`
（`avatar.rs`/`color_swatch.rs`/`mark.rs`/`textarea.rs`）→ `radius-none`
（本イシューで新設したトークンへの移行候補）。

### 5.4 padding / gap / margin（4px 格子に載る値・載らない値）

再取得: `grep -rn '"padding"\|"gap"\|"margin"' crates/pre-styled-ui/src/*.rs`

トークン参照済み（`var(--fandhe-space-*)`）が約 121 箇所、生値が残り
約 140 箇所。4px 格子上の値（`0.25rem`(1)/`0.5rem`(2)/`0.75rem`(3)/
`1rem`(4)/`2rem`(8)/`3rem`(12)/`4rem`(16)）は既存トークンへの単純寄せで
移行できる。格子外の値（`0.0625rem`(1px、`badge.rs`/`kbd.rs`/`code.rs`
の縦 padding)/`0.125rem`(2px)/`0.375rem`(6px)/`0.625rem`(10px)/`0.1rem`/
`0.15rem`）は本イシューで新設した `0-5`/`1-5`/`2-5` トークンへの移行
候補（`0.0625rem` のみ既存スケールに対応段がなく「後続 Phase で `0-5`
または `1` へ丸める」の判断が必要）。`-1px`（visually-hidden 系の意図的な
負値）はトークン化対象外。部品固有 fallback 付き参照（例:
`var(--fandhe-radio-card-padding, 0.75rem)`）は既存トークン
（`space-3`）と同値の fallback であり、後続 Phase での fallback 除去
検討対象として棚卸しに残す。

## 6. 見送った項目とその理由

| 項目 | 参照元 | 見送り理由 |
|------|--------|------------|
| radius `thumb`/`factor` | Radix | fandhe 部品に対応する用途がない（スライダーの thumb 専用 radius 制御・グローバル倍率は §7 の scaling 同様 Theme provider 前提の機構であり非採用方針〔`docs/policy/intentional-non-adoption.md` §3.24〕と整合） |
| radius `2xs`/`3xl`/`4xl` | chakra-ui | fandhe 部品に用途がない |
| shadow `inner`/inset | chakra-ui/Radix | fandhe の inset 用途はすべてドット・リング・マスク描画であり「影」ではない |
| z-index `banner` | chakra-ui | fandhe に対応する部品がない |
| ダーク時の影を弱め border へ寄せる方式 | Radix | overlay 系部品は既に border で境界を担保済み。色トークン（#1422）確定後に再評価 |
| breakpoint `2xl`（1536px） | chakra-ui/shadcn | `Size` の「共通 enum に載せると全部品が空の段を抱える」前例と同じ判断。必要になった時点で純追加できる（§3.6） |
| Radix Themes の名前付き段（`initial`/`xs`〜`xl`） | Radix | 値が chakra/shadcn と 1 段ずれており fandhe の他スケール（radius/shadow/spacing 等）が chakra-ui/shadcn 基準で揃っている整合性を優先（§3.6） |
| 無名 `@container` | — | 将来他部品が `container-type` を持った時点で最近傍 container に束縛され本 recipe の `@container` 規則が黙って壊れるため、名前付き container のみを採用（§3.7） |
| container query の `rem` 表記 | — | `Breakpoint::min_width` のリテラル形式（px）との整合を優先（§3.7） |
| container query `3xs`〜`xs`・`2xl` 以上 | shadcn/ui（Tailwind v4） | `Size`/`Breakpoint` の「共通 enum に載せると全部品が空の段を抱える」前例と同じ判断。必要になった時点で純追加できる（§3.7） |
| container query 用 Theme トークン | — | JS 側に等価 API が無く CSS custom property が `@container` プレリュードで使えない制約は breakpoint と同じで、参照専用トークンを追加する動機自体が無い（§3.7） |

## 7. 再評価トリガー

- 色トークン（#1422）確定後、ダーク時の shadow を「不透明度を上げる」
  から「弱めて border 依存へ寄せる」方式へ切り替えるかどうかを再評価する
  （§3.2）。
- z-index の割り当て（§3.4 の「割り当て予定」列）は後続 Phase の各部品
  issue で実際に適用する際、想定外の重なり順衝突が見つかった場合は
  スケール自体（100 刻み）の見直しを検討する。
- breakpoint（§3.6、イシュー #2197）: 以下は本イシューのスコープ外として
  見送った。複数部品で実際の需要が生じた時点で再評価する。
  - breakpoint × variant / breakpoint × state の複合条件（`@media` 内の
    `.fd-*` クラス・`:hover` 規則）
  - `max-width` / range 構文（`width >= 640px`）
  - docs-site `crates/docs-site/src/site_theme.rs` の 768px/1200px を
    breakpoint トークンと整合させる件
  - `crates/wasm-full` sidebar の `DEFAULT_MOBILE_MEDIA_QUERY` を
    breakpoint トークンと連動させる件（`docs/design/wasm-full-architecture.md`
    既記載）
- container query（§3.7、イシュー #2199）: 以下は本イシューのスコープ外
  として見送った。複数部品で実際の需要が生じた時点で再評価する。
  - container × state の複合条件（container × variant のみ本イシューで
    採用）
  - `max-width` / range 構文、`rem` 表記の container 段、`3xs`〜`xs`・
    `2xl` 以上の段
  - Theme 側 container トークン、1 recipe に複数 container slot
  - `examples/headless-pre-styled-ui`（`fandhe-frontend-pre-styled-ui`
    crates.io バージョン `0.119.1` 固定）の追随・crates.io 公開
