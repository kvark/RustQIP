use crate::errors::CircuitError;
/// Common circuits for general usage.
use crate::{Complex, OpBuilder, Register, UnitaryBuilder};

/// Add some common condition circuits to the UnitaryBuilder structs.
pub trait ConditionCircuits {
    /// A controlled x, using `cr` as control and `r` as input.
    fn cx(&mut self, cr: Register, r: Register) -> (Register, Register);
    /// A controlled y, using `cr` as control and `r` as input.
    fn cy(&mut self, cr: Register, r: Register) -> (Register, Register);
    /// A controlled z, using `cr` as control and `r` as input.
    fn cz(&mut self, cr: Register, r: Register) -> (Register, Register);
    /// A controlled not, using `cr` as control and `r` as input.
    fn cnot(&mut self, cr: Register, r: Register) -> (Register, Register);
    /// Swap `ra` and `rb` controlled by `cr`.
    fn cswap(
        &mut self,
        cr: Register,
        ra: Register,
        rb: Register,
    ) -> Result<(Register, Register, Register), CircuitError>;
    /// Apply a unitary matrix to the register. If mat is 2x2 then can broadcast to all qubits.
    fn cmat(
        &mut self,
        name: &str,
        cr: Register,
        r: Register,
        mat: Vec<Complex<f64>>,
    ) -> Result<(Register, Register), CircuitError>;
    /// Apply a orthonormal matrix to the register. If mat is 2x2 then can broadcast to all qubits.
    fn crealmat(
        &mut self,
        name: &str,
        cr: Register,
        r: Register,
        mat: &[f64],
    ) -> Result<(Register, Register), CircuitError>;
}

impl<B: UnitaryBuilder> ConditionCircuits for B {
    fn cx(&mut self, cr: Register, rb: Register) -> (Register, Register) {
        condition(self, cr, rb, |b, r| b.x(r))
    }
    fn cy(&mut self, cr: Register, rb: Register) -> (Register, Register) {
        condition(self, cr, rb, |b, r| b.y(r))
    }
    fn cz(&mut self, cr: Register, rb: Register) -> (Register, Register) {
        condition(self, cr, rb, |b, r| b.z(r))
    }
    fn cnot(&mut self, cr: Register, rb: Register) -> (Register, Register) {
        condition(self, cr, rb, |b, r| b.not(r))
    }
    fn cswap(
        &mut self,
        cr: Register,
        ra: Register,
        rb: Register,
    ) -> Result<(Register, Register, Register), CircuitError> {
        let (cr, result) = condition(self, cr, (ra, rb), |b, (ra, rb)| b.swap(ra, rb));
        result.map(|(ra, rb)| (cr, ra, rb))
    }
    fn cmat(
        &mut self,
        name: &str,
        cr: Register,
        r: Register,
        mat: Vec<Complex<f64>>,
    ) -> Result<(Register, Register), CircuitError> {
        let (cr, result) = condition(self, cr, r, |b, r| b.mat(name, r, mat));
        result.map(|r| (cr, r))
    }
    fn crealmat(
        &mut self,
        name: &str,
        cr: Register,
        r: Register,
        mat: &[f64],
    ) -> Result<(Register, Register), CircuitError> {
        let (cr, result) = condition(self, cr, r, |b, r| b.real_mat(name, r, mat));
        result.map(|r| (cr, r))
    }
}

/// Condition a circuit defined by `f` using `cr`.
pub fn condition<F, RS, OS>(
    b: &mut dyn UnitaryBuilder,
    cr: Register,
    rs: RS,
    f: F,
) -> (Register, OS)
where
    F: FnOnce(&mut dyn UnitaryBuilder, RS) -> OS,
{
    let mut c = b.with_condition(cr);
    let rs = f(&mut c, rs);
    let r = c.release_register();
    (r, rs)
}

/// Makes a pair of Register in the state `|0n>x|0n> + |1n>x|1n>`
pub fn epr_pair(b: &mut OpBuilder, n: u64) -> (Register, Register) {
    let m = 2 * n;

    let r = b.r(1);
    let rs = b.r(m - 1);

    let r = b.hadamard(r);

    let (r, rs) = condition(b, r, rs, |b, rs| b.not(rs));

    let mut all_rs = vec![r];
    all_rs.extend(b.split_all(rs));

    let back_rs = all_rs.split_off(n as usize);
    let ra = b.merge(all_rs);
    let rb = b.merge(back_rs);

    (ra, rb)
}
