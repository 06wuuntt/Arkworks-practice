# Arkworks 練習 - Lookup Table（Integer ReLU）

ReLU（Rectified Linear Unit）是神經網路常見的啟動函數：

$$
\operatorname{ReLU}(x)=\max(0,x)
$$

本練習先只處理整數，使用 Arkworks 的 R1CS 與 Groth16，證明輸入與輸出確實對應到公開的 ReLU lookup table。

## 目的

1. 理解有限域中的數值沒有原生的正負與大小比較，因此不能直接在電路中寫 `x > 0`。
2. 學習將 Rust 的 signed integer 編碼成 `ark_bls12_381::Fr`。
3. 使用公開 table 表達 `(x, ReLU(x))` 的合法對應關係。
4. 使用 one-hot selector 在 R1CS 中約束 witness 必須選中 table 的其中一列。
5. 區分 Private Witness 與 Public Input：
   - $x$：Private Witness，代表不公開的原始輸入。
   - $y$：Public Input，代表 verifier 可看到的 ReLU 輸出。
6. 使用 Groth16 完成 setup、prove 與 verify，並確認錯誤輸出無法通過驗證。

## 重點

### 1. Signed integer encoding

有限域沒有一般程式語言中的 signed integer。這個練習使用以下方式編碼：

```text
 15 -> Fr::from(15u64)
  0 -> Fr::from(0u64)
-24 -> -Fr::from(24u64)
```

負數在有限域中實際上是模質數後的元素；它的「負數意義」來自我們定義的編碼規則與 lookup table。

### 2. ReLU lookup table

第一階段使用下列整數範圍：

$$
x\in[-32,31]
$$

公開 table 共有 64 列：

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

每列代表：

$$
(T_i.x,T_i.y)=(x,\operatorname{ReLU}(x))
$$

### 3. R1CS lookup constraint

`ark-relations` 的 R1CS 不提供 PLONK/Plookup 的原生 lookup gate。本練習使用 Arkworks gadget 建立 one-hot selectors：

$$
s_i\in\{0,1\}
$$

$$
\sum_i s_i=1
$$

$$
x=\sum_i s_iT_i.x
$$

$$
y=\sum_i s_iT_i.y
$$

因為只有一個 selector 可以等於 1，所以 $(x,y)$ 必須等於 table 中某一列。這不是在 Rust 程式中先查完 table 再把答案交給 verifier，而是把 table membership 寫成 constraint。

## Implementation requirement

請使用以下輸入：

```rust
[2, 4, -1, 0, 15, 6, -24, -2, 15]
```

正確輸出應為：

```rust
[2, 4, 0, 0, 15, 6, 0, 0, 15]
```

需要完成：

1. signed integer 的有限域編碼。
2. 整數 ReLU function。
3. `[-32, 31]` 的公開 ReLU table。
4. one-hot selector lookup constraints。
5. 整組輸入的 `ReluCircuit`。
6. Groth16 setup、proof generation 與 verification。
7. 正確 witness 與錯誤 witness 測試。

## 檔案結構

```text
lookup table/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── encoding.rs
│   ├── lookup.rs
│   ├── circuit.rs
│   └── bin/
│       ├── setup.rs
│       ├── prover.rs
│       └── verifier.rs
├── tests/
│   ├── integer_relu.rs
│   └── invalid_witness.rs
└── artifacts/
    └── .gitkeep
```

## 檔案作用

### `Cargo.toml`

管理本練習使用的 Arkworks crates，主要包括：

- `ark-bls12-381`：提供 BLS12-381 curve 與 scalar field `Fr`。
- `ark-ff`：提供有限域相關 trait。
- `ark-relations`：提供 R1CS constraint system 與 `ConstraintSynthesizer`。
- `ark-r1cs-std`：提供 `Boolean`、`FpVar` 與 constraint gadgets。
- `ark-groth16`：負責 setup、prove 與 verify。
- `ark-serialize`：序列化 proving key、verifying key 與 proof。
- `ark-snark`：提供通用 SNARK trait。
- `ark-std`：提供 Arkworks 使用的標準工具與亂數介面。

### `src/lib.rs`

Library 的入口，宣告並匯出共用模組：

```rust
pub mod circuit;
pub mod encoding;
pub mod lookup;
```

`setup`、`prover`、`verifier` 與 integration tests 都應從 library 匯入共用實作，不要重複定義 circuit。

### `src/encoding.rs`

負責一般整數與有限域元素之間的表示規則，預計包含：

```rust
pub fn relu(value: i64) -> i64;
pub fn signed_fr(value: i64) -> Fr;
```

這個檔案只處理資料表示與普通 ReLU 計算，不建立 R1CS constraint。

### `src/lookup.rs`

負責 lookup table 與單筆 lookup constraint，預計包含：

- table 最小值與最大值。
- 根據 table row 建立 one-hot Boolean selectors。
- 約束 selectors 總和等於 1。
- 約束輸入與輸出等於被選中的 table row。

主要介面預計為：

```rust
pub fn enforce_relu_lookup(
    cs: ConstraintSystemRef<Fr>,
    input: i64,
    output: i64,
) -> Result<(), SynthesisError>;
```

### `src/circuit.rs`

定義完整的 `ReluCircuit` 並實作 `ConstraintSynthesizer<Fr>`。

它的責任是：

1. 保存整組 private inputs 與 public outputs。
2. 依序替每一組 `(input, output)` 呼叫 `enforce_relu_lookup`。
3. 保證 setup 與 prove 使用完全相同的 circuit shape。

### `src/bin/setup.rs`

負責 Groth16 trusted setup：

1. 建立與正式證明相同形狀的 circuit。
2. 呼叫 `generate_random_parameters_with_reduction`。
3. 將 proving key 與 verifying key 序列化。
4. 輸出至：

```text
artifacts/proving_key.bin
artifacts/verifying_key.bin
```

本練習使用 random setup，只適合學習與測試，不代表 production ceremony。

### `src/bin/prover.rs`

負責證明生成：

1. 讀取 `artifacts/proving_key.bin`。
2. 建立題目指定的 private inputs。
3. 建立對應的 public ReLU outputs。
4. 使用 `Groth16::prove` 生成 proof。
5. 將 proof 與 public outputs 序列化。

輸出預計為：

```text
artifacts/proof.bin
artifacts/public_outputs.bin
```

### `src/bin/verifier.rs`

這是一個獨立驗證程式，不應重新實作 `ReluCircuit`。

它的責任是：

1. 讀取 `artifacts/verifying_key.bin`。
2. 呼叫 `prepare_verifying_key`。
3. 讀取 `artifacts/proof.bin`。
4. 讀取公開的 ReLU outputs。
5. 呼叫 `verify_with_processed_vk`。
6. 在 terminal 顯示驗證結果 `true` 或 `false`。

### `tests/integer_relu.rs`

測試題目指定的整數輸入能產生正確輸出，並確認 constraint system satisfied。

### `tests/invalid_witness.rs`

故意使用錯誤輸出，例如：

```text
input  = -1
output = -1
```

或：

```text
input  = 15
output = 0
```

確認 constraint system 不滿足，或 Groth16 verification 回傳 `false`。

### `artifacts/`

存放 setup 與 proof 產物。二進位檔通常不應提交到 Git：

```gitignore
lookup table/artifacts/*.bin
```

## 實作順序

```text
encoding.rs
    ↓
lookup.rs
    ↓
circuit.rs
    ↓
integer_relu.rs
    ↓
invalid_witness.rs
    ↓
setup.rs
    ↓
prover.rs
    ↓
verifier.rs
```

先完成 constraint system 測試，再進入 Groth16 setup、prove 與 verify。

## Terminal 指令

```bash
cargo test -p lookup-table
cargo run --release -p lookup-table --bin setup
cargo run --release -p lookup-table --bin prover
cargo run --release -p lookup-table --bin verifier
```

## 提醒

請注意：寫 ZKP 程式不是只在 Rust 中計算 `relu(x)`，而是要把 $x$、$y$ 與 selector 配置到電路中，並建立它們之間的 constraint 關係。

如果 prover 只在 circuit 外計算 `y = relu(x)`，但沒有在 constraint system 中約束 $(x,y)$ 必須屬於 ReLU table，verifier 並不能確認計算正確。

浮點數與 fixed-point 不在這個階段處理。完成整數版後，再加入第二組小數輸入的量化與定點數表示。
