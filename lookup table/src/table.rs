use ark_bls12_381::Fr;

use crate::encoding::{relu, signed_fr};

// ReLU Table 的整數範圍
pub const TABLE_MIN: i64 = -32;
pub const TABLE_MAX: i64 = 31;

// 建立 ReLU Table
pub fn build_relu_table() -> Vec<(Fr, Fr)> {
    let mut table = Vec::new();

    for i in TABLE_MIN..=TABLE_MAX {
        let y = relu(i);

        table.push((signed_fr(i), signed_fr(y)));
    }

    table // 回傳
}

// 將一般整數轉成 table row index
pub fn table_index(value: i64) -> Option<usize> {
    if !(TABLE_MIN..=TABLE_MAX).contains(&value) {
        return None;
    }
    Some((value - TABLE_MIN) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_64_rows() {
        let table = build_relu_table();

        assert_eq!(table.len(), 64);
    }

    #[test]
    fn table_contains_correct_negative_rows() {
        let table = build_relu_table();

        assert_eq!(
            table[table_index(-32).unwrap()],
            (signed_fr(-32), signed_fr(0))
        );

        assert_eq!(
            table[table_index(-1).unwrap()],
            (signed_fr(-1), signed_fr(0))
        );
    }

    #[test]
    fn table_contains_correct_zero_row() {
        let table = build_relu_table();

        assert_eq!(
            table[table_index(0).unwrap()],
            (signed_fr(0), signed_fr(0))
        );
    }

    #[test]
    fn table_contains_correct_positive_rows() {
        let table = build_relu_table();

        assert_eq!(
            table[table_index(15).unwrap()],
            (signed_fr(15), signed_fr(15))
        );

        assert_eq!(
            table[table_index(31).unwrap()],
            (signed_fr(31), signed_fr(31))
        );
    }

    #[test]
    fn table_index_rejects_out_of_range_values() {
        assert_eq!(table_index(-33), None);
        assert_eq!(table_index(32), None);
    }

    #[test]
    fn table_indices_are_correct() {
        assert_eq!(table_index(-32), Some(0));
        assert_eq!(table_index(-1), Some(31));
        assert_eq!(table_index(0), Some(32));
        assert_eq!(table_index(15), Some(47));
        assert_eq!(table_index(31), Some(63));
    }
}