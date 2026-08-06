// Plookup 基本協定查的是單一 field element，但 ReLU table 每列有兩欄，所以使用 verifier challenge 壓縮：
use ark_bls12_381::Fr;

// 使用 challenge alpha 壓縮一個 `(input, output)` row
// compressed = input + alpha * output
pub fn compress_row(row: &(Fr, Fr), alpha: Fr) -> Fr {
    row.0 + alpha * row.1
}

// 使用相同的 alpha 壓縮所有 rows
pub fn compress_rows(rows: &[(Fr, Fr)], alpha: Fr) -> Vec<Fr> {
    rows.iter()
        .map(|row| compress_row(row, alpha))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        encoding::signed_fr,
        table::build_relu_table,
        witness::{
            build_witness,
            INTEGER_INPUTS,
            INTEGER_OUTPUTS,
            WITNESS_SIZE,
        },
    };

    fn test_alpha() -> Fr {
        Fr::from(7u64)
    }

    #[test]
    fn compresses_negative_relu_row() {
        let row = (
            signed_fr(-2),
            signed_fr(0),
        );

        let compressed =
            compress_row(&row, test_alpha());

        assert_eq!(compressed, signed_fr(-2));
    }

    #[test]
    fn compresses_positive_relu_row() {
        let row = (
            signed_fr(2),
            signed_fr(2),
        );

        let compressed =
            compress_row(&row, test_alpha());

        // 2 + 7 * 2 = 16
        assert_eq!(compressed, Fr::from(16u64));
    }

    #[test]
    fn compression_binds_column_order() {
        let first = (
            signed_fr(1),
            signed_fr(2),
        );

        let second = (
            signed_fr(2),
            signed_fr(1),
        );

        assert_ne!(
            compress_row(&first, test_alpha()),
            compress_row(&second, test_alpha())
        );
    }

    #[test]
    fn compressed_lengths_are_preserved() {
        let table = build_relu_table();

        let witness = build_witness(
            &INTEGER_INPUTS,
            &INTEGER_OUTPUTS,
        )
        .unwrap();

        let compressed_table =
            compress_rows(&table, test_alpha());

        let compressed_witness =
            compress_rows(&witness.rows, test_alpha());

        assert_eq!(compressed_table.len(), 64);
        assert_eq!(
            compressed_witness.len(),
            WITNESS_SIZE
        );
    }

    #[test]
    fn valid_witness_values_exist_in_compressed_table() {
        let table = build_relu_table();

        let witness = build_witness(
            &INTEGER_INPUTS,
            &INTEGER_OUTPUTS,
        )
        .unwrap();

        let compressed_table =
            compress_rows(&table, test_alpha());

        let compressed_witness =
            compress_rows(&witness.rows, test_alpha());

        for value in compressed_witness {
            assert!(compressed_table.contains(&value));
        }
    }

    #[test]
    fn incorrect_positive_output_is_not_in_table() {
        let table = build_relu_table();

        // ReLU(15) 應為 15，故意宣稱為 0。
        let witness =
            build_witness(&[15], &[0]).unwrap();

        let compressed_table =
            compress_rows(&table, test_alpha());

        let compressed_witness =
            compress_rows(&witness.rows, test_alpha());

        assert!(
            !compressed_table.contains(
                &compressed_witness[0]
            )
        );
    }
}