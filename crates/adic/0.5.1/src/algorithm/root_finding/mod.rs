// ### Rootfinding methods
//
// - [`Polynomial::variety`] - Find the roots of a `Polynomial` using the Hensel lemma
// - [`AdicInteger::nth_root`] - Calculate the n-th root of an `AdicInteger` using the Hensel lemma
// - [`roots_of_unity`] - Calculate the p-1 roots of unity in Z_p (2 roots in Z_2)
// - [`tiechmuller`] - Tiechmuller characters in Z_p (zero plus the roots of unity)

pub (crate) mod hensel;


#[cfg(test)]
mod test_hensel;
