use super::UnsignedInteger;
use num_traits::AsPrimitive;

pub trait BarrettBackend<Scalar, ScalarDoubled>
where
    Scalar: UnsignedInteger + AsPrimitive<ScalarDoubled> + AsPrimitive<u128> + 'static,
    u128: AsPrimitive<Scalar>,
    ScalarDoubled: UnsignedInteger + AsPrimitive<Scalar> + 'static,
{
    /// Precomputes modulus specific barrett constant.
    /// We set \alpha = n + 3. Thus \mu = 2^{2*n+3}/modulus
    fn precompute_alpha_and_barrett_constant(modulus: Scalar) -> (usize, Scalar) {
        // TODO (Jay): This is a hack specific to find size of Scalar because num_trait does not seem to have a way to access `BITS` constant.
        let modulus_bits = (std::mem::size_of::<Scalar>() * 8) - modulus.leading_zeros() as usize;

        let mu = (1u128 << (modulus_bits * 2 + 3)) / <Scalar as AsPrimitive<u128>>::as_(modulus);
        (modulus_bits + 3, mu.as_())
    }

    fn modulus(&self) -> Scalar;

    fn modulus_bits(&self) -> usize;

    fn barrett_constant(&self) -> Scalar;

    fn barrett_alpha(&self) -> usize;

    fn add_mod_fast(&self, a: Scalar, b: Scalar) -> Scalar {
        debug_assert!(a < self.modulus(), "Input {a} >= {}", self.modulus());
        debug_assert!(b < self.modulus(), "Input {b} >= {}", self.modulus());

        let mut c = a + b;
        if c >= self.modulus() {
            c -= self.modulus();
        }
        c
    }

    fn sub_mod_fast(&self, a: Scalar, b: Scalar) -> Scalar {
        debug_assert!(a < self.modulus(), "Input {a} >= {}", self.modulus());
        debug_assert!(b < self.modulus(), "Input {b} >= {}", self.modulus());

        if a >= b {
            a - b
        } else {
            (a + self.modulus()) - b
        }
    }

    /// Barrett modular multiplication with pre-compute constant \mu
    ///
    /// Both a and b are < q.
    ///
    /// We implement the generalized barrett reduction
    /// formula described as Algorithm 2 of the this [paper](https://homes.esat.kuleuven.be/~fvercaut/papers/bar_mont.pdf).
    /// Assuming \log(ab) < 2*n + 3, \gamma < n + 3. Since for correctness \alpha should be \ge (\gamma + 1) and \beta <= -2,
    /// we set \alpha as (n + 3) and \beta as -2.
    ///
    /// * [Implementation reference](https://github.com/openfheorg/openfhe-development/blob/c48c41cf7893feb94f09c7d95284a36145ec0d5e/src/core/include/math/hal/intnat/ubintnat.h#L1417)
    /// * Note 1: It is possible to do the same without using `SalarDoubled` (i.e. u128s in case of u64s).
    fn mul_mod_fast(&self, a: Scalar, b: Scalar) -> Scalar {
        debug_assert!(a < self.modulus(), "Input {a} >= {}", self.modulus());
        debug_assert!(b < self.modulus(), "Input {b} >= {}", self.modulus());

        // a*b
        let ab = <Scalar as AsPrimitive<ScalarDoubled>>::as_(a)
            * <Scalar as AsPrimitive<ScalarDoubled>>::as_(b);

        // ab / (2^{n + \beta})
        // note: \beta is assumed to -2
        let tmp = ab >> (self.modulus_bits() - 2);

        // q = ((ab / (2^{n + \beta})) * \mu) / 2^{\alpha - (-2)}
        let q = (tmp * self.barrett_constant().as_()) >> (self.barrett_alpha() + 2);

        // ab - q*p
        let tmp = q * self.modulus().as_();
        let mut res = (ab - tmp).as_();

        if res >= self.modulus() {
            res -= self.modulus();
        }

        res
    }
}
