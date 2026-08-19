use crate::multiset::multiset::MultiSet;
use ark_bls12_381::Fr;
use ark_ff::One;
use ark_poly::{univariate::DensePolynomial, Radix2EvaluationDomain, Polynomial, EvaluationDomain};

fn lagrange_basis(index: usize, domain: &Radix2EvaluationDomain<Fr>) -> DensePolynomial<Fr> {
    let mut evals = vec![Fr::from(0u64); domain.size()];
    evals[index] = Fr::one();
    MultiSet::from_slice(&evals).to_polynomial(domain)
}

pub fn constraint_start(l0: &DensePolynomial<Fr>, z: &DensePolynomial<Fr>, x: Fr) -> Fr {
    l0.evaluate(&x) * (z.evaluate(&x) - Fr::one())
}

pub fn constraint_transition(f: &DensePolynomial<Fr>, t: &DensePolynomial<Fr>, h1: &DensePolynomial<Fr>, h2: &DensePolynomial<Fr>, z: &DensePolynomial<Fr>, x: Fr, group_gen: Fr, last_element: Fr, beta: Fr, gamma: Fr) -> Fr {
    let beta_one = Fr::one() + beta;
    let wx = x * group_gen;

    let lhs = (x - last_element) * z.evaluate(&x) * beta_one * (gamma + f.evaluate(&x)) * ((gamma * beta_one) + t.evaluate(&x) + beta * t.evaluate(&wx));

    let rhs = (x - last_element) * z.evaluate(&wx) * ((gamma * beta_one) + h1.evaluate(&x) + beta * h1.evaluate(&wx)) * ((gamma * beta_one) + h2.evaluate(&x) + beta * h2.evaluate(&wx));

    lhs - rhs
}

pub fn constraint_h1_h2_boundary(ln: &DensePolynomial<Fr>, h1: &DensePolynomial<Fr>, h2: &DensePolynomial<Fr>, x: Fr, group_gen: Fr) -> Fr {
    ln.evaluate(&x) * (h1.evaluate(&x) - h2.evaluate(&(x * group_gen)))
}

pub fn constraint_end(ln: &DensePolynomial<Fr>, z: &DensePolynomial<Fr>, x: Fr) -> Fr {
    ln.evaluate(&x) * (z.evaluate(&x) - Fr::one())
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_poly::EvaluationDomain;
    use crate::lookup::lookup::LookUp;
    use crate::lookup::proof::{compress_column, encode_and_pad_witness};
    use crate::lookup::table::relu::ReLUTable;
    use crate::multiset::multiset_equality::{compute_accumulator_values, compute_h1_h2};

    #[test]
    fn all_constraints_hold_on_every_domain_point_for_a_valid_witness() {
        let table = ReLUTable::new();
        let mut lookup = LookUp::new(ReLUTable::new());
        for input in [2, 4, -1, 0, 15, 6, -24, -2, 15] {
            assert!(lookup.read(input));
        }
        let (encoded_inputs, encoded_outputs) = encode_and_pad_witness(lookup.input_wires(), lookup.output_wires());
        let theta = Fr::from(7u64);
        let compressed_witness = compress_column(&encoded_inputs, &encoded_outputs, theta);
        let compressed_table = compress_column(&table.encode_input_column(), &table.encode_output_column(), theta);

        let f = MultiSet::from_slice(&compressed_witness);
        let t = MultiSet::from_slice(&compressed_table);

        let domain: Radix2EvaluationDomain<Fr> = EvaluationDomain::new(t.len()).unwrap();
        let (h_1, h_2) = compute_h1_h2(&f, &t).unwrap();

        let f_poly = f.to_polynomial(&domain);
        let t_poly = t.to_polynomial(&domain);
        let h1_poly = h_1.to_polynomial(&domain);
        let h2_poly = h_2.to_polynomial(&domain);

        let beta = Fr::from(5u64);
        let gamma = Fr::from(6u64);
        let z_evaluations = compute_accumulator_values(&f, &t, &h_1, &h_2, beta, gamma);
        let z_poly = MultiSet::from_slice(&z_evaluations).to_polynomial(&domain);

        let l0 = lagrange_basis(0, &domain);
        let ln = lagrange_basis(domain.size() - 1, &domain);

        let group_gen = domain.group_gen;
        let last_element = domain.elements().last().unwrap();

        for x in domain.elements() {
            assert_eq!(constraint_start(&l0, &z_poly, x), Fr::from(0u64));
            assert_eq!(constraint_transition(&f_poly, &t_poly, &h1_poly, &h2_poly, &z_poly, x, group_gen, last_element, beta, gamma), Fr::from(0u64));
            assert_eq!(constraint_h1_h2_boundary(&ln, &h1_poly, &h2_poly, x, group_gen), Fr::from(0u64));
            assert_eq!(constraint_end(&ln, &z_poly, x), Fr::from(0u64));
        }
    }
}