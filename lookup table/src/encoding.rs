// 處理資料表示與普通 ReLU 計算，不建立 R1CS constraint
use ark_bls12_381::Fr;

// ReLU(x) = max(0, x)
pub fn relu(value: i64) -> i64 {
    value.max(0)
}

// 將 signed integer 編碼成 BLS12-381 scalar field element。
pub fn signed_fr(value: i64) -> Fr {
    if value >= 0 {
        Fr::from(value as u64)
    } else {
        -Fr::from(value.unsigned_abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relu_keeps_positive_values() {
        assert_eq!(relu(2), 2);
        assert_eq!(relu(15), 15);
        assert_eq!(relu(24), 24);
    }

    #[test]
    fn relu_maps_negative_values_to_zero() {
        assert_eq!(relu(-1), 0);
        assert_eq!(relu(-2), 0);
        assert_eq!(relu(-24), 0);
    }

    #[test]
    fn relu_maps_zero_to_zero() {
        assert_eq!(relu(0), 0);
    }

    #[test]
    fn signed_fr_encodes_positive_integer() {
        assert_eq!(signed_fr(15), Fr::from(15u64));
    }

    #[test]
    fn signed_fr_encodes_zero() {
        assert_eq!(signed_fr(0), Fr::from(0u64));
    }

    #[test]
    fn signed_fr_encodes_negative_integer() {
        assert_eq!(signed_fr(-24), -Fr::from(24u64));
    }
}