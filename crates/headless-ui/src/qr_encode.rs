//! QR Code Model 2（ISO/IEC 18004）の byte モード専用エンコーダ（非公開実装、
//! イシュー #774）。
//!
//! `crate::qr_code` からのみ呼ばれる内部実装であり、公開 API は
//! [`crate::qr_code::encode`] のみ（本モジュールの型・関数はすべて
//! `pub(crate)`）。外部依存は一切追加しない（`fandhe-frontend-core` は
//! 描画にのみ使い、本モジュールは `core`/標準ライブラリのみで完結する。
//! REQ-3 不変条件）。
//!
//! # 実装範囲（意図的な非対応、rustdoc スコープ外）
//!
//! - **byte モードのみ**: numeric/alphanumeric/kanji モードによる容量最適化
//!   は行わない。入力 UTF-8 バイト列をそのまま byte モードで符号化する
//!   （ark-ui/uqr も既定は byte 相当の扱いであり、本実装は「常に安全に
//!   符号化できる最小構成」を優先する）。
//! - ECI（拡張チャネル解釈）・構造的連接（複数シンボル分割）は非対応。
//! - マスクは 8 種全パターンのペナルティスコア評価で決定的に選択する
//!   （ISO/IEC 18004 附属書 8.8.2 準拠）。
//!
//! # 決定性・fail-closed 方針
//!
//! - 乱数・時刻・環境変数を一切参照しない。同一 `(value, ecc)` 入力からは
//!   常に同一のモジュール行列を返す。
//! - 容量超過時は `panic!`/`unwrap()` せず [`QrEncodeError::TooLong`] を
//!   返す（`.claude/rules/coding-rust.md` 「ライブラリコードでの unwrap /
//!   panic を避ける」に従う）。

/// QR エンコード時のエラー（fail-closed。呼び出し側に容量超過を通知する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QrEncodeError {
    /// 入力バイト列が ECC レベル指定下でバージョン 40 の最大容量を超過した。
    TooLong,
}

/// 誤り訂正レベル（`crate::qr_code::ErrorCorrectionLevel` の内部表現）。
/// 数値は ISO/IEC 18004 附属書 C のフォーマット情報 2 ビット指示子
/// （L=01, M=00, Q=11, H=10）ではなく、本モジュール内のテーブル添字
/// （0=L, 1=M, 2=Q, 3=H）として使う（[`format_info_bits`] 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ecc {
    Low,
    Medium,
    Quartile,
    High,
}

impl Ecc {
    const fn table_index(self) -> usize {
        match self {
            Ecc::Low => 0,
            Ecc::Medium => 1,
            Ecc::Quartile => 2,
            Ecc::High => 3,
        }
    }

    /// フォーマット情報 2 ビット指示子値（ISO/IEC 18004 表 25）。
    const fn format_indicator(self) -> u32 {
        match self {
            Ecc::Low => 1,
            Ecc::Medium => 0,
            Ecc::Quartile => 3,
            Ecc::High => 2,
        }
    }
}

/// バージョンごとの「ブロックあたり」誤り訂正 codeword 数
/// （行 = ECC レベル [L,M,Q,H]、列 = バージョン 1..=40、添字 0 は未使用）。
/// ISO/IEC 18004 附属書 D の標準テーブル。
#[rustfmt::skip]
const ECC_CODEWORDS_PER_BLOCK: [[i32; 41]; 4] = [
    // Low
    [-1, 7,10,15,20,26,18,20,24,30,18,20,24,26,30,22,24,28,30,28,28,28,28,30,30,26,28,30,30,30,30,30,30,30,30,30,30,30,30,30,30],
    // Medium
    [-1,10,16,26,18,24,16,18,22,22,26,30,22,22,24,24,28,28,26,26,26,26,28,28,28,28,28,28,28,28,28,28,28,28,28,28,28,28,28,28,28],
    // Quartile
    [-1,13,22,18,26,18,24,18,22,20,24,28,26,24,20,30,24,28,28,26,30,28,30,30,30,30,28,30,30,30,30,30,30,30,30,30,30,30,30,30,30],
    // High
    [-1,17,28,22,16,22,28,26,26,24,28,24,28,22,24,24,30,28,28,26,28,30,24,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],
];

/// バージョンごとの誤り訂正ブロック総数（行 = ECC レベル [L,M,Q,H]）。
/// ISO/IEC 18004 附属書 D の標準テーブル。
#[rustfmt::skip]
const NUM_ERROR_CORRECTION_BLOCKS: [[i32; 41]; 4] = [
    // Low
    [-1, 1,1,1,1,1,2,2,2,2,4,4,4,4,4,6,6,6,6,7,8,8,9,9,10,12,12,12,13,14,15,16,17,18,19,19,20,21,22,24,25],
    // Medium
    [-1, 1,1,1,2,2,4,4,4,5,5,5,8,9,9,10,10,11,13,14,16,17,17,18,20,21,23,25,26,28,29,31,33,35,37,38,40,43,45,47,49],
    // Quartile
    [-1, 1,1,2,2,4,4,6,6,8,8,8,10,12,16,12,17,16,18,21,20,23,23,25,27,29,34,34,35,38,40,43,45,48,51,53,56,59,62,65,68],
    // High
    [-1, 1,1,2,4,4,4,5,6,8,8,11,11,16,16,18,16,19,21,25,25,25,34,30,32,35,37,40,42,45,48,51,54,57,60,63,66,70,74,77,81],
];

/// バージョン `ver`（1..=40）における「生データモジュール数」（機能パターン・
/// フォーマット/バージョン情報領域を除いた、データ+誤り訂正 codeword が
/// 占有可能なビット数）。ISO/IEC 18004 の閉形式（ver ごとのファインダ・
/// タイミング・アライメントパターン占有ビット数を差し引く式、
/// nayuki QR Code generator と同型の一般に知られた導出式）。
fn num_raw_data_modules(ver: u8) -> usize {
    let v = ver as i64;
    let mut result: i64 = (16 * v + 128) * v + 64;
    if ver >= 2 {
        let numalign = v / 7 + 2;
        result -= (25 * numalign - 10) * numalign - 55;
        if ver >= 7 {
            result -= 36;
        }
    }
    result as usize
}

/// バージョン `ver` かつ ECC レベル `ecc` における実データ codeword 数
/// （誤り訂正 codeword を除いた、payload に使えるバイト数）。
fn num_data_codewords(ver: u8, ecc: Ecc) -> usize {
    let raw = num_raw_data_modules(ver) / 8;
    let ecc_len = ECC_CODEWORDS_PER_BLOCK[ecc.table_index()][ver as usize] as usize;
    let blocks = NUM_ERROR_CORRECTION_BLOCKS[ecc.table_index()][ver as usize] as usize;
    raw - ecc_len * blocks
}

/// アライメントパターンの中心座標一覧（昇順、バージョン 1 は空）。
/// ISO/IEC 18004 附属書 E の生成式（nayuki QR Code generator と同型の
/// 一般に知られた導出式）。
fn alignment_pattern_positions(ver: u8) -> Vec<i32> {
    if ver == 1 {
        return Vec::new();
    }
    let v = ver as i32;
    let num_align = v / 7 + 2;
    let step = if ver == 32 {
        26
    } else {
        (v * 4 + num_align * 2 + 1) / (num_align * 2 - 2) * 2
    };
    let mut result: Vec<i32> = vec![6];
    let mut pos = v * 4 + 10;
    for _ in 0..(num_align - 1) {
        result.insert(1, pos);
        pos -= step;
    }
    result
}

/// GF(256)（生成多項式 x^8+x^4+x^3+x^2+1 = 0x11D）の乗算。
/// Reed-Solomon 符号化・生成多項式構築の唯一の低レベル演算。
fn gf_mul(x: u8, mut y: u8) -> u8 {
    let mut z: u8 = 0;
    for _ in 0..8 {
        z = (z << 1) ^ ((z >> 7) * 0x1D);
        z ^= (y >> 7) * x;
        y <<= 1;
    }
    z
}

/// 次数 `degree` の Reed-Solomon 生成多項式の係数列（先頭が最高次、
/// 各ブロックの誤り訂正 codeword 算出に使う除数多項式）。
fn rs_generator_polynomial(degree: usize) -> Vec<u8> {
    let mut result = vec![0u8; degree];
    result[degree - 1] = 1;
    let mut root: u8 = 1;
    for _ in 0..degree {
        for j in 0..degree {
            result[j] = gf_mul(result[j], root);
            if j + 1 < degree {
                result[j] ^= result[j + 1];
            }
        }
        root = gf_mul(root, 0x02);
    }
    result
}

/// `data` を [`rs_generator_polynomial`] の生成多項式で除算した余り
/// （= 誤り訂正 codeword 列、長さ `divisor.len()`）を計算する。
fn rs_compute_remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for &b in data {
        let factor = b ^ result.remove(0);
        result.push(0);
        for (r, &d) in result.iter_mut().zip(divisor.iter()) {
            *r ^= gf_mul(d, factor);
        }
    }
    result
}

/// ビット列を組み立てる薄いバッファ（`Vec<bool>`、1 要素 1 モジュール/ビット）。
struct BitBuffer(Vec<bool>);

impl BitBuffer {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push_bits(&mut self, value: u32, len: u32) {
        for i in (0..len).rev() {
            self.0.push(((value >> i) & 1) != 0);
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

/// byte モードのビットストリームを組み立てる（モード指示子 + 文字数指示子 +
/// データ + 終端子 + バイト境界パディング + パッドバイト）。
///
/// 戻り値は `num_data_codewords(ver, ecc)` バイトちょうど（[`encode`] が
/// 事前にバージョンを選定済みであることを前提とする内部契約）。
fn build_data_codewords(data: &[u8], ver: u8, ecc: Ecc) -> Vec<u8> {
    let mut bits = BitBuffer::new();
    // モード指示子: byte モード = 0100
    bits.push_bits(0b0100, 4);
    // 文字数指示子: バージョン 1-9 は 8 ビット、10-40 は 16 ビット
    // （byte モードの ISO/IEC 18004 表 3 準拠）。
    let count_bits = if ver <= 9 { 8 } else { 16 };
    bits.push_bits(data.len() as u32, count_bits);
    for &b in data {
        bits.push_bits(b as u32, 8);
    }

    let data_capacity_bits = num_data_codewords(ver, ecc) * 8;
    // 終端子: 残り容量に収まる範囲で最大 4 ビットの 0 を追加する
    // （ISO/IEC 18004 §8.4.9。容量ちょうどの場合は追加しない）。
    let terminator_len = (data_capacity_bits - bits.len()).min(4) as u32;
    bits.push_bits(0, terminator_len);

    // バイト境界までの 0 パディング。
    while !bits.len().is_multiple_of(8) {
        bits.0.push(false);
    }

    // バイト列へ変換。
    let mut bytes: Vec<u8> = bits
        .0
        .chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .fold(0u8, |acc, &bit| (acc << 1) | u8::from(bit))
        })
        .collect();

    // パッドバイト 0xEC/0x11 を交互に埋めて容量ちょうどにする。
    let target_len = data_capacity_bits / 8;
    let mut pad = [0xEC_u8, 0x11_u8].into_iter().cycle();
    while bytes.len() < target_len {
        bytes.push(pad.next().expect("cycle iterator は常に Some"));
    }
    bytes
}

/// データ codeword 列をブロック分割し、各ブロックの誤り訂正 codeword を
/// 付加した上でインターリーブした最終 codeword 列を返す
/// （ISO/IEC 18004 §8.6 準拠）。
fn interleave_blocks(data_codewords: &[u8], ver: u8, ecc: Ecc) -> Vec<u8> {
    let ecc_len = ECC_CODEWORDS_PER_BLOCK[ecc.table_index()][ver as usize] as usize;
    let num_blocks = NUM_ERROR_CORRECTION_BLOCKS[ecc.table_index()][ver as usize] as usize;
    let raw_codewords = num_raw_data_modules(ver) / 8;
    let num_short_blocks = num_blocks - raw_codewords % num_blocks;
    let short_block_data_len = raw_codewords / num_blocks - ecc_len;

    let generator = rs_generator_polynomial(ecc_len);

    let mut blocks_data: Vec<&[u8]> = Vec::with_capacity(num_blocks);
    let mut offset = 0usize;
    for i in 0..num_blocks {
        let len = if i < num_short_blocks {
            short_block_data_len
        } else {
            short_block_data_len + 1
        };
        blocks_data.push(&data_codewords[offset..offset + len]);
        offset += len;
    }

    let blocks_ecc: Vec<Vec<u8>> = blocks_data
        .iter()
        .map(|block| rs_compute_remainder(block, &generator))
        .collect();

    let mut result = Vec::with_capacity(raw_codewords);
    // データ部のインターリーブ。
    for i in 0..(short_block_data_len + 1) {
        for block in &blocks_data {
            if i < block.len() {
                result.push(block[i]);
            }
        }
    }
    // 誤り訂正部のインターリーブ。
    for i in 0..ecc_len {
        for ecc_block in &blocks_ecc {
            result.push(ecc_block[i]);
        }
    }
    result
}

/// QR モジュール行列（`true` = 暗モジュール）。行優先（`grid[y][x]`）。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawMatrix {
    pub(crate) size: usize,
    grid: Vec<bool>,
    is_function: Vec<bool>,
}

impl RawMatrix {
    fn new(size: usize) -> Self {
        Self {
            size,
            grid: vec![false; size * size],
            is_function: vec![false; size * size],
        }
    }

    fn idx(&self, x: i32, y: i32) -> usize {
        y as usize * self.size + x as usize
    }

    pub(crate) fn is_dark(&self, x: usize, y: usize) -> bool {
        self.grid[y * self.size + x]
    }

    fn set_function(&mut self, x: i32, y: i32, dark: bool) {
        let i = self.idx(x, y);
        self.grid[i] = dark;
        self.is_function[i] = true;
    }

    fn get(&self, x: i32, y: i32) -> bool {
        self.grid[self.idx(x, y)]
    }

    fn set(&mut self, x: i32, y: i32, dark: bool) {
        let i = self.idx(x, y);
        self.grid[i] = dark;
    }

    fn is_function_at(&self, x: i32, y: i32) -> bool {
        self.is_function[self.idx(x, y)]
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.size && (y as usize) < self.size
    }
}

/// ファインダパターン（3 隅、7x7 + セパレータ込みの中心配置）を描画する。
fn draw_finder_pattern(m: &mut RawMatrix, center_x: i32, center_y: i32) {
    for dy in -4..=4 {
        for dx in -4..=4 {
            let x = center_x + dx;
            let y = center_y + dy;
            if !m.in_bounds(x, y) {
                continue;
            }
            let dist = dx.abs().max(dy.abs());
            let dark = dist != 2 && dist != 4;
            m.set_function(x, y, dark);
        }
    }
}

/// アライメントパターン（5x5、中心に暗モジュール 1 個）を描画する。
fn draw_alignment_pattern(m: &mut RawMatrix, center_x: i32, center_y: i32) {
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            let dist = dx.abs().max(dy.abs());
            m.set_function(center_x + dx, center_y + dy, dist != 1);
        }
    }
}

/// フォーマット情報（誤り訂正レベル + マスクパターン、15 ビット BCH(15,5)）
/// を組み立てる（ISO/IEC 18004 附属書 C）。
fn format_info_bits(ecc: Ecc, mask: u8) -> u32 {
    let data = (ecc.format_indicator() << 3) | mask as u32;
    let mut rem = data;
    for _ in 0..10 {
        rem = (rem << 1) ^ ((rem >> 9) * 0x537);
    }
    ((data << 10) | rem) ^ 0x5412
}

/// バージョン情報（バージョン 7 以上のみ、18 ビット BCH(18,6)）を組み立てる
/// （ISO/IEC 18004 附属書 D）。
fn version_info_bits(ver: u8) -> u32 {
    let mut rem = ver as u32;
    for _ in 0..12 {
        rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
    }
    ((ver as u32) << 12) | rem
}

/// フォーマット情報を 2 箇所（左上ファインダ周辺・右上/左下分割配置）へ
/// 描画する（ISO/IEC 18004 図 25）。マスク済みビットを両箇所へ複製する。
fn draw_format_bits(m: &mut RawMatrix, ecc: Ecc, mask: u8) {
    let bits = format_info_bits(ecc, mask);
    let get_bit = |i: i32| ((bits >> i) & 1) != 0;
    let size = m.size as i32;

    for i in 0i32..6 {
        m.set_function(8, i, get_bit(i));
    }
    m.set_function(8, 7, get_bit(6));
    m.set_function(8, 8, get_bit(7));
    m.set_function(7, 8, get_bit(8));
    for i in 9i32..15 {
        m.set_function(14 - i, 8, get_bit(i));
    }

    for i in 0i32..8 {
        m.set_function(size - 1 - i, 8, get_bit(i));
    }
    for i in 8i32..15 {
        m.set_function(8, size - 15 + i, get_bit(i));
    }
    // 常に暗いモジュール（バージョンに依らず固定、フォーマット情報とは独立
    // だが同じ列 8 に位置するため慣例的にここで設定する）。
    m.set_function(8, size - 8, true);
}

/// バージョン情報（バージョン 7 以上）を左下・右上の 2 箇所へ描画する
/// （ISO/IEC 18004 図 26）。
fn draw_version_bits(m: &mut RawMatrix, ver: u8) {
    if ver < 7 {
        return;
    }
    let bits = version_info_bits(ver);
    let size = m.size as i32;
    for i in 0..18 {
        let bit = ((bits >> i) & 1) != 0;
        let a = i / 3;
        let b = i % 3;
        m.set_function(size - 11 + b, a, bit);
        m.set_function(a, size - 11 + b, bit);
    }
}

/// 機能パターン（ファインダ・セパレータ・タイミング・アライメント・
/// ダークモジュール・フォーマット/バージョン情報のプレースホルダ）を
/// すべて描画する。
fn draw_function_patterns(m: &mut RawMatrix, ver: u8) {
    let size = m.size as i32;

    // タイミングパターン（行 6・列 6、交互）。
    for i in 0..size {
        m.set_function(6, i, i % 2 == 0);
        m.set_function(i, 6, i % 2 == 0);
    }

    draw_finder_pattern(m, 3, 3);
    draw_finder_pattern(m, size - 4, 3);
    draw_finder_pattern(m, 3, size - 4);

    // アライメントパターン。ファインダと重なる位置はスキップする。
    let positions = alignment_pattern_positions(ver);
    for &y in &positions {
        for &x in &positions {
            let is_near_finder = (x == 6 && (y == 6 || y == size - 7)) || (x == size - 7 && y == 6);
            if !is_near_finder {
                draw_alignment_pattern(m, x, y);
            }
        }
    }

    // フォーマット情報プレースホルダ（マスク 0 で仮描画、後で確定値を上書き）。
    draw_format_bits(m, Ecc::Low, 0);
    draw_version_bits(m, ver);
}

/// データ codeword 配置のジグザグ走査順（ISO/IEC 18004 §8.7.3、右下から
/// 左上へ 2 列ずつ、列 6（タイミング列）はスキップする）。
///
/// [`draw_codewords`]（書き込み）と [`extract_codeword_bits`]（読み出し、
/// 復号側自己検証テスト専用）の双方が本関数の座標列をそのまま使い、走査
/// 順序のロジックを 1 箇所に一元化する（2 箇所で座標計算を重複させると
/// 片方だけ修正されて往復不変条件が壊れるドリフトを防ぐ）。
fn zigzag_order(size: i32) -> Vec<(i32, i32)> {
    let mut order = Vec::new();
    let mut x = size - 1;
    while x >= 1 {
        if x == 6 {
            x -= 1;
        }
        for vert in 0..size {
            for j in 0..2 {
                let xx = x - j;
                // 右から左へ列ペアを処理しつつ、上下方向は列ペアごとに反転する
                // （ジグザグ）。`upward` は列ペアの通し番号の偶奇で決める。
                let upward = ((x + 1) / 2) % 2 == 0;
                let yy = if upward { size - 1 - vert } else { vert };
                order.push((xx, yy));
            }
        }
        x -= 2;
    }
    order
}

/// データ codeword 列を [`zigzag_order`] の走査順で機能モジュール以外へ
/// 描画する。
fn draw_codewords(m: &mut RawMatrix, codewords: &[u8]) {
    let mut bit_index = 0usize;
    let total_bits = codewords.len() * 8;
    for (xx, yy) in zigzag_order(m.size as i32) {
        if m.is_function_at(xx, yy) {
            continue;
        }
        let dark = if bit_index < total_bits {
            let byte = codewords[bit_index / 8];
            ((byte >> (7 - (bit_index % 8))) & 1) != 0
        } else {
            false
        };
        bit_index += 1;
        m.set(xx, yy, dark);
    }
}

/// [`draw_codewords`] の逆操作: 完成済み行列からデータビット列を読み出す
/// （復号側自己検証テスト専用、`#[cfg(test)]` からのみ呼ばれる）。
/// マスクは呼び出し側が事前に解除済みであることを前提とする。
#[cfg(test)]
fn extract_codeword_bits(m: &RawMatrix) -> Vec<bool> {
    zigzag_order(m.size as i32)
        .into_iter()
        .filter(|&(x, y)| !m.is_function_at(x, y))
        .map(|(x, y)| m.get(x, y))
        .collect()
}

/// マスクパターン `mask`（0..=7）適用時の bool（暗にすべきか）を返す
/// （ISO/IEC 18004 表 20）。
fn mask_condition(mask: u8, x: i32, y: i32) -> bool {
    match mask {
        0 => (x + y) % 2 == 0,
        1 => y % 2 == 0,
        2 => x % 3 == 0,
        3 => (x + y) % 3 == 0,
        4 => (x / 3 + y / 2) % 2 == 0,
        5 => (x * y) % 2 + (x * y) % 3 == 0,
        6 => ((x * y) % 2 + (x * y) % 3) % 2 == 0,
        7 => ((x + y) % 2 + (x * y) % 3) % 2 == 0,
        _ => unreachable!("mask は 0..=7 のみを呼び出し側が渡す契約"),
    }
}

/// 機能モジュール以外へマスクを適用する（適用/解除は XOR のため同じ関数で
/// 両方向に使える）。
fn apply_mask(m: &mut RawMatrix, mask: u8) {
    let size = m.size as i32;
    for y in 0..size {
        for x in 0..size {
            if m.is_function_at(x, y) {
                continue;
            }
            if mask_condition(mask, x, y) {
                let cur = m.get(x, y);
                m.set(x, y, !cur);
            }
        }
    }
}

/// マスクパターン `mask` 適用後のペナルティスコアを計算する
/// （ISO/IEC 18004 附属書 8.8.2、N1〜N4 評価規則）。
fn penalty_score(m: &RawMatrix) -> i64 {
    let size = m.size as i32;
    let mut total: i64 = 0;

    // N1: 同色連続 5 以上（行・列それぞれ）。
    for y in 0..size {
        total += run_penalty((0..size).map(|x| m.get(x, y)));
    }
    for x in 0..size {
        total += run_penalty((0..size).map(|y| m.get(x, y)));
    }

    // N2: 2x2 同色ブロック。
    for y in 0..size - 1 {
        for x in 0..size - 1 {
            let c = m.get(x, y);
            if m.get(x + 1, y) == c && m.get(x, y + 1) == c && m.get(x + 1, y + 1) == c {
                total += 3;
            }
        }
    }

    // N3: ファインダ様パターン（1:1:3:1:1 比率 + 片側 4 連続の明モジュール）。
    for y in 0..size {
        total += finder_like_penalty((0..size).map(|x| m.get(x, y)));
    }
    for x in 0..size {
        total += finder_like_penalty((0..size).map(|y| m.get(x, y)));
    }

    // N4: 暗モジュール比率の 50% からの乖離。
    let dark_count = (0..size * size).filter(|&i| m.grid[i as usize]).count() as i64;
    let total_modules = (size * size) as i64;
    let percent = dark_count * 100 / total_modules;
    let prev = (percent / 5) * 5;
    let next = prev + 5;
    let diff = (prev - 50).abs().min((next - 50).abs()) / 5;
    total += diff * 10;

    total
}

/// N1 評価: 同色連続 5 以上ごとに `3 + (run_len - 5)` を加算する。
fn run_penalty<I: Iterator<Item = bool>>(iter: I) -> i64 {
    let mut total = 0i64;
    let mut run_len = 0i64;
    let mut prev: Option<bool> = None;
    for v in iter {
        match prev {
            Some(p) if p == v => run_len += 1,
            _ => {
                if run_len >= 5 {
                    total += 3 + (run_len - 5);
                }
                run_len = 1;
            }
        }
        prev = Some(v);
    }
    if run_len >= 5 {
        total += 3 + (run_len - 5);
    }
    total
}

/// N3 評価: `1011101` の前後に明モジュール 4 連続が付随するパターンごとに
/// 40 を加算する（両側とも対象になり得る）。
fn finder_like_penalty<I: Iterator<Item = bool>>(iter: I) -> i64 {
    let bits: Vec<bool> = iter.collect();
    let pattern_a = [
        true, false, true, true, true, false, true, false, false, false, false,
    ];
    let pattern_b = [
        false, false, false, false, true, false, true, true, true, false, true,
    ];
    let mut total = 0i64;
    if bits.len() < pattern_a.len() {
        return 0;
    }
    for start in 0..=(bits.len() - pattern_a.len()) {
        let window = &bits[start..start + pattern_a.len()];
        if window == pattern_a || window == pattern_b {
            total += 40;
        }
    }
    total
}

/// QR Model 2 の byte モードエンコード本体（[`crate::qr_code::encode`] の
/// 内部実装）。
///
/// 1. `data` が収まる最小バージョン（1..=40）を決定的に選択する。
/// 2. データ codeword 列を組み立て、ブロック分割 + Reed-Solomon 誤り訂正
///    codeword を付加してインターリーブする。
/// 3. 機能パターンを描画し、データ codeword をジグザグ配置する。
/// 4. 8 種のマスクパターンをすべて試し、ペナルティスコア最小のものを選ぶ
///    （同点の場合は若い番号を優先、`min_by_key` の安定性に依存する）。
/// 5. 確定したマスクでフォーマット情報を描画する。
pub(crate) fn encode(data: &[u8], ecc: Ecc) -> Result<(RawMatrix, u8), QrEncodeError> {
    let mut chosen_ver: Option<u8> = None;
    for ver in 1..=40u8 {
        let capacity_bits = num_data_codewords(ver, ecc) * 8;
        let count_bits = if ver <= 9 { 8 } else { 16 };
        let needed = 4 + count_bits + data.len() * 8;
        if needed <= capacity_bits {
            chosen_ver = Some(ver);
            break;
        }
    }
    let ver = chosen_ver.ok_or(QrEncodeError::TooLong)?;

    let data_codewords = build_data_codewords(data, ver, ecc);
    let final_codewords = interleave_blocks(&data_codewords, ver, ecc);

    let size = 17 + 4 * ver as usize;
    let mut base = RawMatrix::new(size);
    draw_function_patterns(&mut base, ver);
    draw_codewords(&mut base, &final_codewords);

    let mut best_mask = 0u8;
    let mut best_score = i64::MAX;
    let mut best_grid: Option<Vec<bool>> = None;
    for mask in 0..8u8 {
        let mut candidate = RawMatrix {
            size: base.size,
            grid: base.grid.clone(),
            is_function: base.is_function.clone(),
        };
        apply_mask(&mut candidate, mask);
        draw_format_bits(&mut candidate, ecc, mask);
        draw_version_bits(&mut candidate, ver);
        let score = penalty_score(&candidate);
        if score < best_score {
            best_score = score;
            best_mask = mask;
            best_grid = Some(candidate.grid);
        }
    }

    let mut result = RawMatrix::new(size);
    result.grid = best_grid.expect("mask 0..=7 は必ず 1 回以上評価される");
    result.is_function = base.is_function;
    Ok((result, best_mask))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gf_mul_matches_known_identities() {
        assert_eq!(gf_mul(0, 5), 0);
        assert_eq!(gf_mul(1, 5), 5);
        // 2 * 2 = 4 in GF(256)（生成多項式の次数未満のため単純な整数乗算と一致）。
        assert_eq!(gf_mul(2, 2), 4);
    }

    #[test]
    fn num_raw_data_modules_matches_known_totals() {
        // バージョン 1: 208 ビット = 26 codeword（remainder bits = 0）。
        assert_eq!(num_raw_data_modules(1) / 8, 26);
        // バージョン 2: remainder bits = 7、総 codeword 数 44。
        assert_eq!(num_raw_data_modules(2) / 8, 44);
        // バージョン 40: 総 codeword 数 3706（公知の標準値）。
        assert_eq!(num_raw_data_modules(40) / 8, 3706);
    }

    #[test]
    fn num_data_codewords_matches_known_values() {
        assert_eq!(num_data_codewords(1, Ecc::Low), 19);
        assert_eq!(num_data_codewords(1, Ecc::Medium), 16);
        assert_eq!(num_data_codewords(1, Ecc::Quartile), 13);
        assert_eq!(num_data_codewords(1, Ecc::High), 9);
        assert_eq!(num_data_codewords(40, Ecc::Low), 2956);
    }

    #[test]
    fn alignment_pattern_positions_match_known_tables() {
        assert_eq!(alignment_pattern_positions(1), Vec::<i32>::new());
        assert_eq!(alignment_pattern_positions(2), vec![6, 18]);
        assert_eq!(alignment_pattern_positions(3), vec![6, 22]);
        assert_eq!(alignment_pattern_positions(7), vec![6, 22, 38]);
        assert_eq!(alignment_pattern_positions(14), vec![6, 26, 46, 66]);
    }

    #[test]
    fn format_info_bits_known_value() {
        // ECC L (indicator=1) + mask 0 の既知フォーマット情報値
        // （ISO/IEC 18004 附属書 C 例、広く再現される公知値）。
        assert_eq!(format_info_bits(Ecc::Low, 0), 0b111011111000100);
    }

    #[test]
    fn too_long_input_is_fail_closed() {
        // バージョン 40-L の最大バイト容量（2953 バイト）を超える入力。
        let data = vec![0x41u8; 2954];
        assert_eq!(encode(&data, Ecc::Low), Err(QrEncodeError::TooLong));
    }

    #[test]
    fn max_capacity_input_succeeds() {
        let data = vec![0x41u8; 2953];
        assert!(encode(&data, Ecc::Low).is_ok());
    }

    #[test]
    fn empty_input_succeeds() {
        let (matrix, _) = encode(&[], Ecc::Low).expect("空文字列は許容される");
        assert_eq!(matrix.size, 21);
    }

    #[test]
    fn encoding_is_deterministic() {
        let a = encode(b"https://fandhe.example/", Ecc::Medium).expect("エンコード成功");
        let b = encode(b"https://fandhe.example/", Ecc::Medium).expect("エンコード成功");
        assert_eq!(a.1, b.1);
        assert_eq!(a.0.grid, b.0.grid);
    }

    /// `bytes`（ジグザグ順で読み出した codeword 列）の `start_bit` から
    /// `len` ビットを MSB ファーストで読み取る（[`decode_and_verify`] 専用の
    /// ビットリーダ。[`BitBuffer::push_bits`] と対になる読み出し側実装）。
    fn read_bits(bytes: &[u8], start_bit: usize, len: usize) -> u32 {
        let mut v = 0u32;
        for i in 0..len {
            let bit_pos = start_bit + i;
            let byte = bytes[bit_pos / 8];
            let bit = (byte >> (7 - (bit_pos % 8))) & 1;
            v = (v << 1) | u32::from(bit);
        }
        v
    }

    /// エンコード結果を「エンコードとは独立した経路」で復号し、往復整合性を
    /// 検証する（イシュー #774 受け入れ条件「復号側自己検証」）。
    ///
    /// 1. フォーマット/バージョン情報ではなく [`encode`] が返したマスク番号
    ///    を使ってマスクを解除する（往復検証が目的であり、フォーマット情報
    ///    ビット自体の読み取り復号は別途 [`format_info_bits_known_value`]
    ///    が固定値で検証する）。
    /// 2. [`zigzag_order`] を [`draw_codewords`] と共有しているため、走査
    ///    順序そのものにバグがあれば本テストでは検出できない
    ///    （構造不変条件は別途 `crates/headless-ui/tests/qr_code.rs` が
    ///    ファインダ形状・タイミングパターン等の幾何を独立に検証する）。
    /// 3. 各ブロックのデータ codeword から Reed-Solomon 誤り訂正 codeword
    ///    を再計算し、埋め込まれている誤り訂正 codeword と完全一致することを
    ///    確認する（インターリーブ・ブロック分割・RS 符号化にバグがあれば
    ///    ここで不一致になる）。
    /// 4. 再構成したデータ codeword 列からモード指示子・文字数指示子を
    ///    パースし、元の入力バイト列と一致することを確認する。
    fn decode_and_verify(value: &[u8], ecc: Ecc) {
        let (matrix, mask) = encode(value, ecc).expect("エンコード成功");
        let ver = ((matrix.size - 17) / 4) as u8;

        let mut demasked = RawMatrix {
            size: matrix.size,
            grid: matrix.grid.clone(),
            is_function: matrix.is_function.clone(),
        };
        apply_mask(&mut demasked, mask); // XOR は involution なので同じ関数で解除できる。

        // 非機能モジュール総数（[`num_raw_data_modules`]）はバイト境界に揃う
        // とは限らない（remainder bits、ISO/IEC 18004 表 1。例: バージョン
        // 2-6 は 7 ビット）。[`draw_codewords`] は codeword 分のビットだけを
        // 書き込み、残りは `false` 埋めのまま zigzag 末尾に残すため、
        // 先頭 `raw_codewords * 8` ビットのみを codeword として解釈する。
        let bits = extract_codeword_bits(&demasked);
        let ecc_len = ECC_CODEWORDS_PER_BLOCK[ecc.table_index()][ver as usize] as usize;
        let num_blocks = NUM_ERROR_CORRECTION_BLOCKS[ecc.table_index()][ver as usize] as usize;
        let raw_codewords = num_raw_data_modules(ver) / 8;
        assert_eq!(bits.len(), num_raw_data_modules(ver));
        let codewords: Vec<u8> = bits[..raw_codewords * 8]
            .chunks(8)
            .map(|c| c.iter().fold(0u8, |acc, &b| (acc << 1) | u8::from(b)))
            .collect();
        assert_eq!(codewords.len(), raw_codewords);
        let num_short_blocks = num_blocks - raw_codewords % num_blocks;
        let short_block_data_len = raw_codewords / num_blocks - ecc_len;

        let mut blocks_data: Vec<Vec<u8>> = (0..num_blocks)
            .map(|i| {
                let len = if i < num_short_blocks {
                    short_block_data_len
                } else {
                    short_block_data_len + 1
                };
                Vec::with_capacity(len)
            })
            .collect();
        let mut blocks_ecc: Vec<Vec<u8>> = (0..num_blocks)
            .map(|_| Vec::with_capacity(ecc_len))
            .collect();

        let mut cursor = 0usize;
        for i in 0..(short_block_data_len + 1) {
            for (b, block) in blocks_data.iter_mut().enumerate() {
                let len = if b < num_short_blocks {
                    short_block_data_len
                } else {
                    short_block_data_len + 1
                };
                if i < len {
                    block.push(codewords[cursor]);
                    cursor += 1;
                }
            }
        }
        for _ in 0..ecc_len {
            for block in blocks_ecc.iter_mut() {
                block.push(codewords[cursor]);
                cursor += 1;
            }
        }
        assert_eq!(cursor, codewords.len());

        let generator = rs_generator_polynomial(ecc_len);
        for (data_block, ecc_block) in blocks_data.iter().zip(blocks_ecc.iter()) {
            let remainder = rs_compute_remainder(data_block, &generator);
            assert_eq!(
                &remainder, ecc_block,
                "ブロックの Reed-Solomon 誤り訂正 codeword が再計算値と一致しない"
            );
        }

        let data_codewords: Vec<u8> = blocks_data.into_iter().flatten().collect();
        let mode = read_bits(&data_codewords, 0, 4);
        assert_eq!(mode, 0b0100, "byte モード指示子であること");
        let count_bits = if ver <= 9 { 8 } else { 16 };
        let count = read_bits(&data_codewords, 4, count_bits) as usize;
        assert_eq!(count, value.len());
        let mut decoded = Vec::with_capacity(count);
        for i in 0..count {
            decoded.push(read_bits(&data_codewords, 4 + count_bits + i * 8, 8) as u8);
        }
        assert_eq!(decoded, value);
    }

    #[test]
    fn decode_round_trip_self_verification() {
        decode_and_verify(b"", Ecc::Low);
        decode_and_verify(b"A", Ecc::Low);
        decode_and_verify(b"https://ark-ui.com", Ecc::Low);
        decode_and_verify(b"https://ark-ui.com", Ecc::Medium);
        decode_and_verify(b"fandhe-frontend headless-ui QrCode #774", Ecc::Quartile);
        decode_and_verify(&[0x41u8; 200], Ecc::High);
        // バージョン 7 以上（バージョン情報ビット付き）を確実に踏むための
        // 大きめの入力。
        decode_and_verify(&[0x42u8; 500], Ecc::Medium);
    }
}
