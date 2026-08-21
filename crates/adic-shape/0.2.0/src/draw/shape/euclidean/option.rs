use adic::divisible::Prime;


#[derive(Debug, Clone)]
pub (super) enum EuclideanStructure {
    ScaledHulls(Vec<(f64, f64)>),
    CharacteristicPAdic(Prime),
}
