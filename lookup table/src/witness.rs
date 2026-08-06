// 建立固定長度 witness
use ark_bls12_381::Fr;

use crate::{
    encoding::signed_fr,
    table::table_index,
};

pub const WITNESS_SIZE: usize = 63;
pub const PADDING_VALUE: i64 = 0;

pub const INTEGER_INPUTS: [i64; 9] =
    [2, 4, -1, 0, 15, 6, -24, -2, 15];

pub const INTEGER_OUTPUTS: [i64; 9] =
    [2, 4, 0, 0, 15, 6, 0, 0, 15];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessData {
    // 經過 encoding 的 (input, output)
    pub rows: Vec<(Fr, Fr)>,

    // 每一筆輸入所對應的公開 table row index
    pub table_indices: Vec<usize>,

    // padding 前的實際查詢數量
    pub query_count: usize,
}

// build_witness() 在準備資料階段時使用的錯誤
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WitnessError {
    // 輸入和輸出的長度不一樣
    LengthMismatch {
        inputs: usize,
        outputs: usize,
    },

    // witness 的資料太多組
    TooManyRows {
        provided: usize,
        maximum: usize,
    },

    // 輸入值不在公開 ReLU table 的範圍內
    InputOutOfRange(i64),
}

pub fn build_witness(
    inputs: &[i64],
    outputs: &[i64],
) -> Result<WitnessData, WitnessError> {
    // 輸入和輸出的長度不一樣
    if inputs.len() != outputs.len() {
        return Err(WitnessError::LengthMismatch {
            inputs: inputs.len(),
            outputs: outputs.len(),
        });
    }

    // witness 的資料太多組
    if inputs.len() > WITNESS_SIZE {
        return Err(WitnessError::TooManyRows {
            provided: inputs.len(),
            maximum: WITNESS_SIZE,
        });
    }

    let query_count = inputs.len();

    let mut rows = Vec::with_capacity(WITNESS_SIZE);
    let mut table_indices = Vec::with_capacity(WITNESS_SIZE);

    // 將 (&input, &output) 配對
    for (&input, &output) in inputs.iter().zip(outputs.iter()) {
        // 尋找 input 在公開 table 的 index
        let index = table_index(input)
            .ok_or(WitnessError::InputOutOfRange(input))?;

        rows.push((
            signed_fr(input),
            signed_fr(output),
        ));

        table_indices.push(index);
    }

    // 取得 padding 對應的 table index
    let padding_index = table_index(PADDING_VALUE)
        .expect("padding value must exist in the public table");

    // 補到固定 63 組
    while rows.len() < WITNESS_SIZE {
        rows.push((
            signed_fr(PADDING_VALUE),
            signed_fr(PADDING_VALUE),
        ));

        table_indices.push(padding_index);
    }

    Ok(WitnessData {
        rows,
        table_indices,
        query_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exercise_witness_has_expected_size() {
        let witness = build_witness(
            &INTEGER_INPUTS,
            &INTEGER_OUTPUTS,
        )
        .unwrap();

        assert_eq!(witness.rows.len(), WITNESS_SIZE);
        assert_eq!(
            witness.table_indices.len(),
            WITNESS_SIZE
        );
        assert_eq!(witness.query_count, 9);
    }

    #[test]
    fn exercise_indices_are_correct() {
        let witness = build_witness(
            &INTEGER_INPUTS,
            &INTEGER_OUTPUTS,
        )
        .unwrap();

        assert_eq!(
            &witness.table_indices[..9],
            &[34, 36, 31, 32, 47, 38, 8, 30, 47]
        );
    }

    #[test]
    fn remaining_rows_are_zero_padding() {
        let witness = build_witness(
            &INTEGER_INPUTS,
            &INTEGER_OUTPUTS,
        )
        .unwrap();

        let zero_row = (
            signed_fr(0),
            signed_fr(0),
        );

        assert!(
            witness.rows[9..]
                .iter()
                .all(|row| *row == zero_row)
        );

        assert!(
            witness.table_indices[9..]
                .iter()
                .all(|&index| index == 32)
        );
    }

    #[test]
    fn mismatched_lengths_are_rejected() {
        let result = build_witness(
            &[1, 2],
            &[1],
        );

        assert_eq!(
            result,
            Err(WitnessError::LengthMismatch {
                inputs: 2,
                outputs: 1,
            })
        );
    }

    #[test]
    fn out_of_range_input_is_rejected() {
        let result = build_witness(
            &[32],
            &[32],
        );

        assert_eq!(
            result,
            Err(WitnessError::InputOutOfRange(32))
        );
    }

    #[test]
    fn incorrect_relu_output_is_not_prechecked() {
        let witness = build_witness(
            &[-1],
            &[-1],
        )
        .unwrap();

        assert_eq!(
            witness.rows[0],
            (signed_fr(-1), signed_fr(-1))
        );
    }
}