use crate::multiset::multiset::{MultiSet, MultiSetError};
use ark_bls12_381::Fr;
use ark_ff::One;

pub fn compute_h1_h2(f: &MultiSet, t:&MultiSet) -> Result<(MultiSet, MultiSet), MultiSetError> {
    // Compute s
    let sorted_s = f.concatenate_and_sort(t)?;

    // Compute h_1, h_2
    let (h_1, h_2) = sorted_s.halve();

    // assert that the last element of h_1 is equal to the first element of h_2
    assert_eq!(h_1.last(), h_2.as_slice()[0]);
    Ok((h_1, h_2))
}

fn compute_f_i(i: usize, f: &MultiSet, t: &MultiSet, beta: Fr, gamma: Fr) -> Fr {
    let gamma_beta_one = gamma * (beta + Fr::one());
    // (gamma + f_i) * [gamma *(1 + beta) + t_i + beta * t_{i+1}]
    (gamma + f.as_slice()[i]) * (gamma_beta_one + t.as_slice()[i] + (beta * t.as_slice()[i + 1]))
}

fn compute_g_i(i: usize, h_1: &MultiSet, h_2: &MultiSet, beta: Fr, gamma: Fr) -> Fr {
    let gamma_beta_one = gamma * (beta + Fr::one());
    // gamma * (1 + beta) + s_j + beta * s_{j+1}
    let first = gamma_beta_one + h_1.as_slice()[i] + beta * h_1.as_slice()[i + 1];
    // gamma * (1 + beta) + s_{n+j} + beta * s_{n+j+1}
    let second = gamma_beta_one + h_2.as_slice()[i] + beta * h_2.as_slice()[i + 1];
    first * second
}

pub fn compute_accumulator_values(f: &MultiSet, t: &MultiSet, h_1: &MultiSet, h_2: &MultiSet, beta: Fr, gamma: Fr) -> Vec<Fr> {
    let n = f.len();

    // F(beta, gamma)
    let mut numerator = Vec::with_capacity(n + 1);
    // G(beta, gamma)
    let mut denominator = Vec::with_capacity(n + 1);

    // Z evaluated at the first root of unity is 1
    numerator.push(Fr::one());
    denominator.push(Fr::one());

    let beta_one = Fr::one() + beta;

    // Compute value for Z(X)
    for i in 0..n {
        let f_i = beta_one * compute_f_i(i, f, t, beta, gamma);
        let g_i = compute_g_i(i, h_1, h_2, beta, gamma);
        numerator.push(f_i * numerator.last().unwrap());
        denominator.push(g_i * denominator.last().unwrap());
    }

    // Check that Z(g^{n+1}) = 1
    let last_numerator = *numerator.last().unwrap();
    let last_denominator = *denominator.last().unwrap();
    assert_eq!(last_numerator / last_denominator, Fr::one());

    // Combine numerator and denominator
    assert_eq!(numerator.len(), denominator.len());
    assert_eq!(numerator.len(), n + 1);
    let mut evaluations = Vec::with_capacity(numerator.len());
    for (n, d) in numerator.into_iter().zip(denominator) {
        evaluations.push(n / d)
    }
    evaluations
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn accumulator_matches_recursive_definition_step_by_step() {
        let t = MultiSet::from_slice(&[1, 2, 3, 4, 5, 6, 7, 8].map(Fr::from));
        let f = MultiSet::from_slice(&[2, 2, 5, 8, 1, 1, 3].map(Fr::from));
        let beta = Fr::from(8u64);
        let gamma = Fr::from(10u64);

        let (h_1, h_2) = compute_h1_h2(&f, &t).unwrap();
        let z = compute_accumulator_values(&f, &t, &h_1, &h_2, beta, gamma);

        let beta_one = Fr::one() + beta;

        // z[0] is 1 / 1
        assert_eq!(z[0], Fr::one());

        for i in 0..f.len() {
            let f_i = compute_f_i(i, &f, &t, beta, gamma);
            let g_i = compute_g_i(i, &h_1, &h_2, beta, gamma);
            assert_eq!(z[i + 1], beta_one * z[i] * f_i / g_i);
        }

        // the last element should be equal to 1
        assert_eq!(*z.last().unwrap(), Fr::one());
    }
}