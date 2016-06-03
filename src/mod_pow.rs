pub trait ModPow {
    type IntType;

    fn mod_pow(base: Self::IntType, exponent: Self::IntType, modulus: Self::IntType) -> Self::IntType;
}