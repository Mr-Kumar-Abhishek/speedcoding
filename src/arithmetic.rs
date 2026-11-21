//! Phase 3: SpeedCo I floating-binary arithmetic and Set 1 elementary mathematical functions.

use crate::core_types::{
    FRACTION_BITS, MAX_BINARY_EXPONENT, MachineStop, Op1, SpeedError, SpeedFloat,
};

pub const MAX_ADD_SUB_DEC_EXP_DIFF: u32 = 76;
pub const MAX_SQRT_EXP_LN_DEC_EXP: i32 = 39_456;
pub const MAX_SINE_ARTAN_DEC_EXP: i32 = 76;

const FRACTION_MASK: u64 = (1u64 << FRACTION_BITS) - 1;
const MIN_NORM_FRACTION: u64 = 1u64 << (FRACTION_BITS - 1);
const TWO_POW_35_F64: f64 = (1u64 << FRACTION_BITS) as f64;

/// Outcome of executing an arithmetic `OP1` instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOutcome {
    Value(SpeedFloat),
    Stop(MachineStop),
}

fn make_speed_float(negative: bool, fraction: u64, exp: i32) -> Result<SpeedFloat, SpeedError> {
    if exp.unsigned_abs() > MAX_BINARY_EXPONENT {
        return Err(SpeedError::BinaryExponentOutOfRange(exp));
    }
    Ok(SpeedFloat {
        negative,
        fraction: fraction & FRACTION_MASK,
        exp_negative: exp < 0,
        exponent: exp.unsigned_abs(),
    })
}

fn normalize_nonzero(val: SpeedFloat) -> (bool, u64, i32) {
    let mut frac = val.fraction & FRACTION_MASK;
    let mut exp = val.signed_exponent();
    if frac != 0 {
        while frac < MIN_NORM_FRACTION {
            frac <<= 1;
            exp -= 1;
        }
    }
    (val.negative, frac, exp)
}

fn approx_decimal_exponent(val: SpeedFloat) -> i32 {
    if val.is_zero_fraction() {
        ((val.signed_exponent() as f64) * std::f64::consts::LOG10_2).round() as i32
    } else {
        let (_, frac, exp) = normalize_nonzero(val);
        let f = (frac as f64) / TWO_POW_35_F64;
        let log10_val = f.log10() + (exp as f64) * std::f64::consts::LOG10_2;
        if exp.unsigned_abs() >= MAX_BINARY_EXPONENT {
            // Near 2^131071 (~10^39456.3), keep valid SpeedFloat binary exponents within 39,456.
            log10_val.round() as i32
        } else {
            log10_val.floor() as i32 + 1
        }
    }
}

fn ceil_div_2(e: i32) -> i32 {
    e.div_euclid(2) + e.rem_euclid(2)
}

fn add_signed(lhs: SpeedFloat, rhs: SpeedFloat) -> Result<ArithmeticOutcome, SpeedError> {
    if lhs.is_zero_fraction() && rhs.is_zero_fraction() {
        let exp = lhs.signed_exponent().max(rhs.signed_exponent());
        return Ok(ArithmeticOutcome::Value(make_speed_float(false, 0, exp)?));
    }
    if lhs.is_zero_fraction() {
        let (neg, frac, exp) = normalize_nonzero(rhs);
        return Ok(ArithmeticOutcome::Value(make_speed_float(neg, frac, exp)?));
    }
    if rhs.is_zero_fraction() {
        let (neg, frac, exp) = normalize_nonzero(lhs);
        return Ok(ArithmeticOutcome::Value(make_speed_float(neg, frac, exp)?));
    }

    let (neg_a, fa, ea) = normalize_nonzero(lhs);
    let (neg_b, fb, eb) = normalize_nonzero(rhs);

    let bin_diff = ea.abs_diff(eb);
    let dec_diff = ((bin_diff as f64) * std::f64::consts::LOG10_2).floor() as u32;
    if dec_diff > MAX_ADD_SUB_DEC_EXP_DIFF {
        return Err(SpeedError::ExponentDifferenceTooLarge { diff: dec_diff });
    }

    let ((big_neg, big_frac, big_exp), (small_neg, small_frac, small_exp)) = if ea >= eb {
        ((neg_a, fa, ea), (neg_b, fb, eb))
    } else {
        ((neg_b, fb, eb), (neg_a, fa, ea))
    };

    let shift = big_exp.abs_diff(small_exp);
    let big_70 = u128::from(big_frac) << FRACTION_BITS;
    let small_70 = if shift >= 2 * FRACTION_BITS {
        0
    } else {
        (u128::from(small_frac) << FRACTION_BITS) >> shift
    };

    if big_neg == small_neg {
        let mut sum_70 = big_70 + small_70;
        let mut res_exp = big_exp;
        if sum_70 >= (1u128 << (2 * FRACTION_BITS)) {
            sum_70 >>= 1;
            res_exp += 1;
        }
        // SpeedCo I add/sub results are truncated (unrounded) to 35 fraction bits.
        let res_frac = (sum_70 >> FRACTION_BITS) as u64;
        return Ok(ArithmeticOutcome::Value(make_speed_float(
            big_neg, res_frac, res_exp,
        )?));
    }

    // Manual page 3-3: exact cancellation stores F = 0 and retains the operand exponent.
    if big_70 == small_70 {
        return Ok(ArithmeticOutcome::Value(make_speed_float(
            false, 0, big_exp,
        )?));
    }

    let (mut diff_70, res_neg) = if big_70 > small_70 {
        (big_70 - small_70, big_neg)
    } else {
        (small_70 - big_70, small_neg)
    };

    let target_lz = 128 - 2 * FRACTION_BITS;
    let lshift = diff_70.leading_zeros() - target_lz;
    diff_70 <<= lshift;
    let res_exp = big_exp - (lshift as i32);
    let res_frac = (diff_70 >> FRACTION_BITS) as u64;

    Ok(ArithmeticOutcome::Value(make_speed_float(
        res_neg, res_frac, res_exp,
    )?))
}

fn multiply_signed(
    a: SpeedFloat,
    b: SpeedFloat,
    negate: bool,
) -> Result<ArithmeticOutcome, SpeedError> {
    if a.is_zero_fraction() || b.is_zero_fraction() {
        return Ok(ArithmeticOutcome::Value(SpeedFloat::zero()));
    }

    let (neg_a, fa, ea) = normalize_nonzero(a);
    let (neg_b, fb, eb) = normalize_nonzero(b);
    let res_neg = (neg_a ^ neg_b) ^ negate;

    let prod = u128::from(fa) * u128::from(fb);
    let half_threshold = 1u128 << (2 * FRACTION_BITS - 1);

    // Manual pages 3-3..3-4: double product if |Fa*Fb| < 0.5, and truncate to 35 bits.
    let (res_frac, res_exp) = if prod < half_threshold {
        ((prod >> (FRACTION_BITS - 1)) as u64, ea + eb - 1)
    } else {
        ((prod >> FRACTION_BITS) as u64, ea + eb)
    };

    Ok(ArithmeticOutcome::Value(make_speed_float(
        res_neg, res_frac, res_exp,
    )?))
}

fn divide_signed(
    a: SpeedFloat,
    b: SpeedFloat,
    negate: bool,
) -> Result<ArithmeticOutcome, SpeedError> {
    if b.is_zero_fraction() {
        return Ok(ArithmeticOutcome::Stop(MachineStop::DivideCheck));
    }
    if a.is_zero_fraction() {
        return Ok(ArithmeticOutcome::Value(SpeedFloat::zero()));
    }

    let (neg_a, fa, ea) = normalize_nonzero(a);
    let (neg_b, fb, eb) = normalize_nonzero(b);
    let res_neg = (neg_a ^ neg_b) ^ negate;

    let (num, mut res_exp) = if fa >= fb {
        (u128::from(fa) << (FRACTION_BITS - 1), ea - eb + 1)
    } else {
        (u128::from(fa) << FRACTION_BITS, ea - eb)
    };
    let den = u128::from(fb);
    let mut q = num / den;
    let rem = num % den;

    // Manual page 3-4: form a 35-bit rounded quotient.
    if rem * 2 >= den {
        q += 1;
        if q == (1u128 << FRACTION_BITS) {
            q >>= 1;
            res_exp += 1;
        }
    }

    Ok(ArithmeticOutcome::Value(make_speed_float(
        res_neg, q as u64, res_exp,
    )?))
}

/// Executes binary `OP1` arithmetic (`ADD`, `SUB`, `ADDAB`, `ABADD`, `SUBAB`, `ABSUB`, `MPY`, `NGMPY`, `DIV`, `NGDIV`).
pub fn execute_op1_arithmetic(
    op: Op1,
    a: SpeedFloat,
    b: SpeedFloat,
) -> Result<ArithmeticOutcome, SpeedError> {
    match op {
        Op1::Add => add_signed(a, b),
        Op1::Sub => add_signed(a, b.neg()),
        Op1::AddAb => add_signed(a, b.abs()),
        Op1::AbAdd => add_signed(a.abs(), b.abs()),
        Op1::SubAb => add_signed(a, b.abs().neg()),
        Op1::AbSub => add_signed(a.abs(), b.abs().neg()),
        Op1::Mpy => multiply_signed(a, b, false),
        Op1::NgMpy => multiply_signed(a, b, true),
        Op1::Div => divide_signed(a, b, false),
        Op1::NgDiv => divide_signed(a, b, true),
        _ => Err(SpeedError::InvalidOp1Code(op.code())),
    }
}

/// Executes Set 1 unary elementary mathematical `OP1` operations (`SQRT`, `SINE`, `ARTAN`, `EXP`, `LN`).
pub fn execute_op1_math(op: Op1, a: SpeedFloat) -> Result<SpeedFloat, SpeedError> {
    match op {
        Op1::Sqrt => {
            let dec_exp = approx_decimal_exponent(a);
            if dec_exp.abs() > MAX_SQRT_EXP_LN_DEC_EXP {
                return Err(SpeedError::DecimalExponentOutOfRange(dec_exp));
            }
            if a.negative || a.is_zero_fraction() {
                return make_speed_float(false, 0, ceil_div_2(a.signed_exponent()));
            }
            let (_, frac, exp) = normalize_nonzero(a);
            let f = (frac as f64) / TWO_POW_35_F64;
            let k = exp.div_euclid(2);
            let (scaled, out_exp_base) = if exp.rem_euclid(2) == 0 {
                (f, k)
            } else {
                (f * 0.5, k + 1)
            };
            let sf_root = SpeedFloat::from_f64(scaled.sqrt())?;
            make_speed_float(
                false,
                sf_root.fraction,
                sf_root.signed_exponent() + out_exp_base,
            )
        }
        Op1::Sine => {
            if a.is_zero_fraction() {
                return Ok(SpeedFloat {
                    negative: false,
                    fraction: 0,
                    exp_negative: false,
                    exponent: 7,
                });
            }
            let dec_exp = approx_decimal_exponent(a);
            if dec_exp.abs() > MAX_SINE_ARTAN_DEC_EXP {
                return Err(SpeedError::DecimalExponentOutOfRange(dec_exp));
            }
            if dec_exp > 10 {
                return Ok(SpeedFloat {
                    negative: false,
                    fraction: 0,
                    exp_negative: false,
                    exponent: 7,
                });
            }
            SpeedFloat::from_f64(a.to_f64().sin())
        }
        Op1::Artan => {
            if a.is_zero_fraction() {
                return Ok(SpeedFloat::zero());
            }
            let dec_exp = approx_decimal_exponent(a);
            if dec_exp.abs() > MAX_SINE_ARTAN_DEC_EXP {
                return Err(SpeedError::DecimalExponentOutOfRange(dec_exp));
            }
            SpeedFloat::from_f64(a.to_f64().atan())
        }
        Op1::Exp => {
            if a.is_zero_fraction() {
                return SpeedFloat::from_f64(1.0);
            }
            let dec_exp = approx_decimal_exponent(a);
            if dec_exp.abs() > MAX_SQRT_EXP_LN_DEC_EXP {
                return Err(SpeedError::DecimalExponentOutOfRange(dec_exp));
            }
            let (_, _, exp) = normalize_nonzero(a);
            if exp > 20 {
                let overflow_exp = if a.negative {
                    -(MAX_BINARY_EXPONENT as i32) - 1
                } else {
                    (MAX_BINARY_EXPONENT as i32) + 1
                };
                return Err(SpeedError::BinaryExponentOutOfRange(overflow_exp));
            }
            let x = a.to_f64();
            if x.abs() <= 500.0 {
                SpeedFloat::from_f64(x.exp())
            } else {
                let k_f = (x * std::f64::consts::LOG2_E).round();
                let k = k_f as i32;
                let r = x - k_f * std::f64::consts::LN_2;
                let sf_m = SpeedFloat::from_f64(r.exp())?;
                make_speed_float(false, sf_m.fraction, sf_m.signed_exponent() + k)
            }
        }
        Op1::Ln => {
            let dec_exp = approx_decimal_exponent(a);
            if dec_exp.abs() > MAX_SQRT_EXP_LN_DEC_EXP {
                return Err(SpeedError::DecimalExponentOutOfRange(dec_exp));
            }
            // Appendix-2 quirks: negative F acts as |F|; zero F acts as F = 1/2.
            let (f, e) = if a.is_zero_fraction() {
                (0.5_f64, a.signed_exponent())
            } else {
                let (_, frac, exp) = normalize_nonzero(a.abs());
                ((frac as f64) / TWO_POW_35_F64, exp)
            };
            let ln_val = if e.abs() <= 1000 {
                (f * 2.0_f64.powi(e)).ln()
            } else {
                f.ln() + (e as f64) * std::f64::consts::LN_2
            };
            SpeedFloat::from_f64(ln_val)
        }
        _ => Err(SpeedError::InvalidOp1Code(op.code())),
    }
}
