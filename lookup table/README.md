# Arkworks 練習 - Plookup（Integer ReLU）

ReLU（Rectified Linear Unit）定義為：

$$
\operatorname{ReLU}(x)=\max(0,x)
$$

本練習使用 Arkworks 0.5，依照 Ariel Gabizon 與 Zachary J. Williamson 的論文 *plookup: A simplified polynomial protocol for lookup tables*，實作整數 ReLU 的 toy Plookup protocol。

第一階段先直接建立並檢查論文 Section 3 的 polynomial identities；第二階段再加入 polynomial commitment 與 opening proof。這樣可以先理解 Plookup 本身，再處理 commitment backend。

## 練習目的

1. 理解有限域中沒有原生 signed integer、大小比較與 `max()`。
2. 將 ReLU 表示成公開的 input-output lookup table。
3. 使用 random linear combination 將雙欄 row 壓縮成單一 field element。
4. 理解 Plookup 的 sorted concatenation。
5. 建立論文中的 $h_1$、$h_2$ 與 grand-product accumulator $Z$。
6. 檢查 Plookup polynomial identities。
7. 確認正確 witness 通過、非法 row 無法通過。
8. 最後使用 polynomial commitment 將 toy protocol 擴充成 committed proof。

## 這次實作的是什麼？

本練習實作的是 Plookup，不是：

- Rust 的 `Vec::contains()`。
- R1CS one-hot selector。
- Groth16 circuit。
- Segment Lookup。
- tLookup。
- 完整的 PLONK proving system。

Plookup 的目標是證明 committed witness polynomial 的 evaluations 全部存在於公開 table 中。核心工具是：

- tuple compression；
- sorted concatenation；
- permutation/grand-product argument；
- polynomial identities；
- polynomial commitment 與 opening proof。

## ReLU lookup table

第一階段使用：

$$
x\in[-32,31]
$$

建立兩欄 table：

$$
T_X=[-32,-31,\ldots,31]
$$

$$
T_Y=[0,0,\ldots,0,1,2,\ldots,31]
$$

每一列是：

```text
(-32, 0)
(-31, 0)
...
(-1, 0)
(0, 0)
(1, 1)
...
(31, 31)
```

Table 共有 64 列，因此在論文 Section 3 的設定中：

```text
d = 64
n + 1 = 64
n = 63
```

所以 witness vector `f` 必須有 63 個 elements。題目只有 9 筆有效查詢，其餘位置使用合法 row padding。

## 題目輸入

```rust
[2, 4, -1, 0, 15, 6, -24, -2, 15]
```

預期輸出：

```rust
[2, 4, 0, 0, 15, 6, 0, 0, 15]
```

輸入對應的 table row indices：

```rust
[34, 36, 31, 32, 47, 38, 8, 30, 47]
```

剩下的位置可以使用 `(0, 0)` padding，直到 witness 長度為 63。

## Signed integer encoding

本練習定義：

```text
 15 -> Fr::from(15u64)
  0 -> Fr::from(0u64)
-24 -> -Fr::from(24u64)
```

負數在 finite field 中是對應正數的加法反元素。Plookup 不會替 field elements 定義大小順序，所以 sorted concatenation 必須依照公開 table 的 row 順序建立，不能直接對 field elements 使用一般數值排序。

## Step 1：壓縮雙欄 row

Plookup 的基本協定先處理單一 field element。對 ReLU 的雙欄 row，verifier 提供隨機挑戰：

$$
\alpha\in\mathbb F
$$

壓縮 table row：

$$
t_i=T_{X,i}+\alpha T_{Y,i}
$$

壓縮 witness row：

$$
f_i=X_i+\alpha Y_i
$$

最後證明：

$$
f\subseteq t
$$

使用同一個 $\alpha$ 綁定 input 與 output，避免 prover 從不同 table rows 分別挑選合法的 input 與 output。

## Step 2：Sorted concatenation

建立：

$$
s=\operatorname{sort}_t(f\Vert t)
$$

這裡的排序不是依 finite-field representative 的數值大小排序，而是依 table rows 原本的順序排列。

如果某個 table row 在 witness 出現兩次，該 row 在 $s$ 中會出現：

```text
1 次 table occurrence + 2 次 witness occurrences
```

因為：

```text
len(f) = 63
len(t) = 64
len(s) = 127 = 2n + 1
```

## Step 3：建立 polynomials

取大小 64 的 radix-2 multiplicative subgroup：

$$
H=\{g,g^2,\ldots,g^{64}=1\}
$$

將 vectors 插值成 polynomials：

- $f(X)$：witness polynomial；
- $t(X)$：table polynomial；
- $h_1(X)$、$h_2(X)$：共同描述長度 127 的 sorted concatenation；
- $Z(X)$：grand-product accumulator。

$h_1$ 與 $h_2$ 各自在 64 點 domain 上表示 $s$ 的一部分，並在交界處共享一個值。

## Step 4：Grand product

Verifier 提供隨機挑戰：

$$
\beta,\gamma\in\mathbb F
$$

Prover 使用 $f,t,h_1,h_2,\beta,\gamma$ 建立 accumulator $Z$，使其逐列累積 numerator 與 denominator 的比值。

Verifier 檢查：

1. $Z$ 的起點為 1。
2. 每一步 accumulator transition 正確。
3. $h_1$ 與 $h_2$ 在交界處一致。
4. $Z$ 的終點回到 1。

如果 witness 包含 table 中不存在的 row，除了 negligible probability 的隨機挑戰碰撞外，終點與 polynomial identities 無法同時成立。

## 兩階段實作策略

### Phase A：Toy Plookup

先不做 commitments，直接使用 vectors 與 polynomial evaluations 驗證：

```text
table/witness rows
    ↓
tuple compression
    ↓
sorted concatenation
    ↓
polynomial interpolation
    ↓
grand-product accumulator
    ↓
identity checks on H
```

這個階段用來確認協定數學與 indexing 正確。

### Phase B：Committed Plookup

Phase A 通過後，再加入：

- polynomial commitment setup；
- commitments to witness/auxiliary polynomials；
- Fiat-Shamir transcript；
- random evaluation challenge；
- batched polynomial openings；
- verifier opening checks。

## 檔案結構

```text
lookup table/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── encoding.rs
│   ├── table.rs
│   ├── witness.rs
│   ├── compression.rs
│   ├── sorted.rs
│   ├── polynomial.rs
│   ├── protocol.rs
│   ├── commitment.rs      # Phase B
│   └── main.rs
└── tests/
    ├── table.rs
    ├── compression.rs
    ├── sorted.rs
    ├── protocol.rs
    └── invalid_witness.rs
```

## 檔案作用

### `src/encoding.rs`

負責 `i64`、ReLU 與 field encoding：

```rust
pub fn relu(value: i64) -> i64;
pub fn signed_fr(value: i64) -> Fr;
```

目前已完成，保留使用。

### `src/table.rs`

建立 64 列公開 ReLU table，保留 row 的原始順序與 field representation。

### `src/witness.rs`

建立 63 列 witness：

- 前 9 列是題目輸入與輸出；
- 其餘使用合法 `(0,0)` row padding；
- 非法 input 或 output 必須被測試捕捉。

### `src/compression.rs`

使用 verifier challenge $\alpha$ 將 `(x,y)` 壓縮成：

$$
x+\alpha y
$$

### `src/sorted.rs`

依 table row 順序建立 $s=\operatorname{sort}_t(f\Vert t)$，並將長度 127 的 $s$ 拆成 $h_1,h_2$ evaluations。

### `src/polynomial.rs`

建立 radix-2 evaluation domain，執行 interpolation/evaluation，並提供 shifted evaluation 所需工具。

### `src/protocol.rs`

實作 toy Plookup：

- 產生 $\beta,\gamma$；
- 建立 accumulator $Z$；
- 檢查 boundary、transition 與 glue identities；
- 回報 accept/reject。

### `src/commitment.rs`

Phase B 才加入，負責 polynomial commitments、opening proofs 與 verifier checks。

### `src/main.rs`

組合完整 demo，印出：

- table/witness sizes；
- compressed rows；
- sorted vector size；
- domain size；
- identity check 結果；
- 正確 proof 是否通過。

## 實作順序

```text
1. 更新 Cargo.toml
2. 整理 encoding.rs 與 table.rs
3. 建立固定長度 witness
4. 實作 tuple compression
5. 實作 sorted concatenation
6. 建立 evaluation domain 與 polynomials
7. 實作 grand-product accumulator
8. 檢查 Section 3 identities
9. 加入非法 witness 測試
10. 建立 main.rs 端到端 demo
11. 加入 polynomial commitments
12. 加入 transcript 與 opening proofs
```

每完成一步，都先執行對應的 `cargo test`；測試通過後才進入下一步。

## Dependencies

Phase A 主要使用：

```text
ark-bls12-381
ark-ff
ark-poly
ark-std
```

Phase B 再加入：

```text
ark-ec
ark-poly-commit
ark-serialize
```

本練習不使用：

```text
ark-groth16
ark-relations
ark-r1cs-std
ark-segmentlookup
```

## 執行指令

```bash
cd ~/Code/Arkworks-practice

cargo check -p lookup-table
cargo test -p lookup-table
cargo run --release -p lookup-table
```

## 範圍與安全性提醒

1. Phase A 是用來學習論文數學的 toy protocol，沒有 commitment，因此尚不是可部署的 zero-knowledge proof。
2. Phase B 必須正確處理 transcript、challenge ordering、degree bounds 與 opening proofs，才能形成 committed protocol。
3. ReLU table 的排序以原始 signed integer 順序定義；不能使用 field element 的自然大小比較代表 signed ordering。
4. 第一階段只處理整數。完成後才能加入 fixed-point encoding 與小數範圍。
5. 完整 PLONK integration 不在本練習第一階段範圍內；完成 toy Plookup 後，再閱讀 `ZK-Garage/plonk` 的 lookup 模組。
