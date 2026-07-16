use ark_bls12_381::Fr;
use ark_relations::lc;
use ark_relations::r1cs::{
    ConstraintSynthesizer,
    ConstraintSystemRef,
    SynthesisError,
};

pub type ConstraintF = Fr;

/// 證明存在 w、x、b，使公開的 y 滿足線性轉換。
#[derive(Clone, Debug, Default)]
pub struct ScalarLinearCircuit {
    /// Private witness: weight
    pub w: Option<ConstraintF>,

    /// Private witness: input
    pub x: Option<ConstraintF>,

    /// Private witness: bias
    pub b: Option<ConstraintF>,

    /// Public input: output
    pub y: Option<ConstraintF>,
}

impl ConstraintSynthesizer<ConstraintF> for ScalarLinearCircuit {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<ConstraintF>,
    ) -> Result<(), SynthesisError> {
        // Private witness
        let w_var = cs.new_witness_variable(|| {
            self.w.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Private witness
        let x_var = cs.new_witness_variable(|| {
            self.x.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Private witness
        let b_var = cs.new_witness_variable(|| {
            self.b.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Public input
        let y_var = cs.new_input_variable(|| {
            self.y.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // TODO: 由你建立核心 R1CS constraint。
        cs.enforce_constraint(
            lc!() + w_var,
            lc!() + x_var,
            lc!() + y_var - b_var,
        )?;
        // 你可以使用的 circuit variables：
        // - w_var
        // - x_var
        // - b_var
        // - y_var
        //
        // 目標：限制它們符合 y = w * x + b。
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn scalar_circuit_accepts_correct_output() {
        let circuit = ScalarLinearCircuit {
            w: Some(ConstraintF::from(3u64)),
            x: Some(ConstraintF::from(6u64)),
            b: Some(ConstraintF::from(3u64)),

            // TODO：請手動計算正確的 y，再填入這裡。
            y: Some(ConstraintF::from(21u64)),
        };

        let cs = ConstraintSystem::<ConstraintF>::new_ref();

        circuit
            .generate_constraints(cs.clone())
            .expect("建立 constraints 失敗");

        let is_satisfied = cs
            .is_satisfied()
            .expect("檢查 constraint system 失敗");

        println!("is_satisfied = {is_satisfied}");
        assert!(is_satisfied);
    }

    #[test]
    fn scalar_circuit_rejects_wrong_output() {
        let circuit = ScalarLinearCircuit {
            w: Some(ConstraintF::from(3u64)),
            x: Some(ConstraintF::from(6u64)),
            b: Some(ConstraintF::from(3u64)),

            // TODO：故意填入一個錯誤的 y。
            y: Some(ConstraintF::from(10u64)),
        };

        let cs = ConstraintSystem::<ConstraintF>::new_ref();

        circuit
            .generate_constraints(cs.clone())
            .expect("建立 constraints 失敗");

        let is_satisfied = cs
            .is_satisfied()
            .expect("檢查 constraint system 失敗");

        println!("is_satisfied = {is_satisfied}");
        assert!(!is_satisfied);
    }
}