//! Phase 3 TDD tests: Floating-binary arithmetic and Set 1 elementary mathematical functions
//! (SpeedCo I manual section 3, section 6, and Appendices 1-2).

use speedcoding::arithmetic::{
    ArithmeticOutcome, MAX_ADD_SUB_DEC_EXP_DIFF, MAX_SINE_ARTAN_DEC_EXP,
    MAX_SQRT_EXP_LN_DEC_EXP, execute_op1_arithmetic, execute_op1_math,
};
use speedcoding::core_types::{MachineStop, Op1, SpeedError, SpeedFloat};

fn sf(val: f64) -> SpeedFloat {
    SpeedFloat::from_f64(val).unwrap()
}

fn unwrap_val(outcome: ArithmeticOutcome) -> SpeedFloat {
    match outcome {
        ArithmeticOutcome::Value(v) => v,
        ArithmeticOutcome::Stop(s) => panic!("unexpected machine stop: {s:?}"),
    }
}

#[test]
fn add_sub_six_variants_with_signed_operands() {
    let a = sf(-3.5);
    let b = sf(-1.25);

    // ADD: Q(A) + Q(B) = -4.75
    let add = unwrap_val(execute_op1_arithmetic(Op1::Add, a, b).unwrap());
    assert!((add.to_f64() - (-4.75)).abs() < 1e-9);

    // SUB: Q(A) - Q(B) = -2.25
    let sub = unwrap_val(execute_op1_arithmetic(Op1::Sub, a, b).unwrap());
    assert!((sub.to_f64() - (-2.25)).abs() < 1e-9);

    // ADDAB: Q(A) + |Q(B)| = -3.5 + 1.25 = -2.25
    let addab = unwrap_val(execute_op1_arithmetic(Op1::AddAb, a, b).unwrap());
    assert!((addab.to_f64() - (-2.25)).abs() < 1e-9);

    // ABADD: |Q(A)| + |Q(B)| = 4.75
    let abadd = unwrap_val(execute_op1_arithmetic(Op1::AbAdd, a, b).unwrap());
    assert!((abadd.to_f64() - 4.75).abs() < 1e-9);

    // SUBAB: Q(A) - |Q(B)| = -3.5 - 1.25 = -4.75
    let subab = unwrap_val(execute_op1_arithmetic(Op1::SubAb, a, b).unwrap());
    assert!((subab.to_f64() - (-4.75)).abs() < 1e-9);

    // ABSUB: |Q(A)| - |Q(B)| = 2.25
    let absub = unwrap_val(execute_op1_arithmetic(Op1::AbSub, a, b).unwrap());
    assert!((absub.to_f64() - 2.25).abs() < 1e-9);
}

#[test]
fn exact_cancellation_preserves_exponent_without_shifting() {
    // Manual page 3-3: when operands are equal and opposite in sign,
    // zero is stored as fractional part and the exponent of the operands becomes result exponent.
    let x = sf(6.0); // 0.75 * 2^3 -> exponent = 3
    let neg_x = sf(-6.0);
    let res = unwrap_val(execute_op1_arithmetic(Op1::Add, x, neg_x).unwrap());
    assert_eq!(res.fraction, 0);
    assert_eq!(res.signed_exponent(), x.signed_exponent());

    let sub_self = unwrap_val(execute_op1_arithmetic(Op1::Sub, x, x).unwrap());
    assert_eq!(sub_self.fraction, 0);
    assert_eq!(sub_self.signed_exponent(), x.signed_exponent());
}

#[test]
fn add_sub_exponent_difference_over_76_rejected() {
    assert_eq!(MAX_ADD_SUB_DEC_EXP_DIFF, 76);

    // Binary exponent difference ~256 corresponds to ~77 decimal orders (> 76).
    let large = SpeedFloat {
        negative: false,
        fraction: 1u64 << 34,
        exp_negative: false,
        exponent: 300,
    };
    let small = SpeedFloat {
        negative: false,
        fraction: 1u64 << 34,
        exp_negative: false,
        exponent: 10,
    };
    assert!(matches!(
        execute_op1_arithmetic(Op1::Add, large, small),
        Err(SpeedError::ExponentDifferenceTooLarge { .. })
    ));
    assert!(matches!(
        execute_op1_arithmetic(Op1::Sub, large, small),
        Err(SpeedError::ExponentDifferenceTooLarge { .. })
    ));
}

#[test]
fn mpy_and_ngmpy_truncation_and_sub_half_doubling() {
    // 0.5 * 2^1 * 0.5 * 2^1 = 0.25 * 2^2 -> |Fa*Fb| < 0.5 -> doubled to 0.5 * 2^1 = 1.0
    let a = sf(1.0);
    let b = sf(1.0);
    let mpy_one = unwrap_val(execute_op1_arithmetic(Op1::Mpy, a, b).unwrap());
    assert!(mpy_one.is_normalized());
    assert!((mpy_one.to_f64() - 1.0).abs() < 1e-10);
    assert_eq!(mpy_one.signed_exponent(), 1);

    // 0.75 * 0.75 = 0.5625 >= 0.5 -> exponent sum preserved (1 + 1 = 2), result = 2.25
    let mpy_225 = unwrap_val(execute_op1_arithmetic(Op1::Mpy, sf(1.5), sf(-1.5)).unwrap());
    assert!(mpy_225.negative);
    assert_eq!(mpy_225.signed_exponent(), 2);
    assert!((mpy_225.to_f64() - (-2.25)).abs() < 1e-10);

    // NGMPY forms the negative of the product: -(-2.25) = +2.25
    let ngmpy = unwrap_val(execute_op1_arithmetic(Op1::NgMpy, sf(1.5), sf(-1.5)).unwrap());
    assert!(!ngmpy.negative);
    assert!((ngmpy.to_f64() - 2.25).abs() < 1e-10);

    // Manual pages 3-3..3-4: |Fa*Fb| < 0.5 is doubled (exp - 1) and truncated to 35 fraction bits.
    let odd = SpeedFloat {
        negative: false,
        fraction: (1u64 << 34) | 3,
        exp_negative: false,
        exponent: 0,
    };
    let prod = unwrap_val(execute_op1_arithmetic(Op1::Mpy, odd, odd).unwrap());
    assert!(prod.is_normalized());
    assert_eq!(prod.signed_exponent(), -1);
    assert_eq!(prod.fraction, (1u64 << 34) | 6); // doubled because |Fa*Fb| < 0.5, low bits truncated

    // Remainder > 0.5 ULP check: (3*2^33 + 1)^2 = (9*2^31 + 1)*2^35 + (2^34 + 1) -> truncates, not rounds up.
    let trunc_test = SpeedFloat {
        negative: false,
        fraction: (3u64 << 33) | 1,
        exp_negative: false,
        exponent: 0,
    };
    let trunc_prod = unwrap_val(execute_op1_arithmetic(Op1::Mpy, trunc_test, trunc_test).unwrap());
    assert!(trunc_prod.is_normalized());
    assert_eq!(trunc_prod.signed_exponent(), 0);
    assert_eq!(trunc_prod.fraction, (9u64 << 31) | 1);
}

#[test]
fn div_and_ngdiv_rounding_and_exponent_adjustment() {
    let q1 = unwrap_val(execute_op1_arithmetic(Op1::Div, sf(9.0), sf(2.0)).unwrap());
    assert!(q1.is_normalized());
    assert!((q1.to_f64() - 4.5).abs() < 1e-9);

    let q2 = unwrap_val(execute_op1_arithmetic(Op1::NgDiv, sf(-9.0), sf(4.0)).unwrap());
    assert!(!q2.negative);
    assert!((q2.to_f64() - 2.25).abs() < 1e-9);

    // 1 / 3 checks 34/35-bit rounded quotient (page 3-4).
    let third = unwrap_val(execute_op1_arithmetic(Op1::Div, sf(1.0), sf(3.0)).unwrap());
    assert!((third.to_f64() - (1.0 / 3.0)).abs() < 1e-10);
}

#[test]
fn div_by_zero_triggers_divide_check_stop() {
    let res = execute_op1_arithmetic(Op1::Div, sf(5.0), SpeedFloat::zero()).unwrap();
    assert_eq!(res, ArithmeticOutcome::Stop(MachineStop::DivideCheck));

    let ng_res = execute_op1_arithmetic(Op1::NgDiv, sf(5.0), SpeedFloat::zero()).unwrap();
    assert_eq!(ng_res, ArithmeticOutcome::Stop(MachineStop::DivideCheck));
}

#[test]
fn sqrt_accuracy_and_negative_or_zero_fraction_quirk() {
    assert_eq!(MAX_SQRT_EXP_LN_DEC_EXP, 39_456);

    // Positive inputs: at least 9 significant decimal digits (Appendix-1).
    for x in [0.25_f64, 2.0, 100.0, 12345.6789] {
        let out = execute_op1_math(Op1::Sqrt, sf(x)).unwrap();
        let rel_err = ((out.to_f64() - x.sqrt()) / x.sqrt()).abs();
        assert!(rel_err < 1e-9, "SQRT({x}) rel_err={rel_err}");
    }

    // Appendix-1 quirk: negative or zero fraction yields F = 0 and binary exp ceil(E/2).
    let neg_odd_exp = SpeedFloat {
        negative: true,
        fraction: 1u64 << 34,
        exp_negative: false,
        exponent: 5,
    };
    let q_neg = execute_op1_math(Op1::Sqrt, neg_odd_exp).unwrap();
    assert_eq!(q_neg.fraction, 0);
    assert_eq!(q_neg.signed_exponent(), 3); // ceil(5 / 2)

    let zero_neg_exp = SpeedFloat {
        negative: false,
        fraction: 0,
        exp_negative: true,
        exponent: 5,
    };
    let q_zero = execute_op1_math(Op1::Sqrt, zero_neg_exp).unwrap();
    assert_eq!(q_zero.fraction, 0);
    assert_eq!(q_zero.signed_exponent(), -2); // ceil(-5 / 2) = -2
}

#[test]
fn sine_accuracy_and_large_exponent_zero_quirk() {
    assert_eq!(MAX_SINE_ARTAN_DEC_EXP, 76);

    // Appendix-1: at least 8 decimal places when |decimal_exp| <= 3.
    for x in [0.0_f64, 0.5, 1.0, -1.2345, std::f64::consts::FRAC_PI_2, 10.0] {
        let out = execute_op1_math(Op1::Sine, sf(x)).unwrap();
        let err = (out.to_f64() - x.sin()).abs();
        assert!(err < 1e-8, "SINE({x}) err={err}");
    }

    // Appendix-1 quirk: F == 0 gives F = 0, binary exponent +7.
    let z = execute_op1_math(Op1::Sine, SpeedFloat::zero()).unwrap();
    assert_eq!(z.fraction, 0);
    assert_eq!(z.signed_exponent(), 7);
}

#[test]
fn artan_principal_value_accuracy() {
    // Appendix-1: at least 7 decimal places across positive & negative arguments.
    for x in [-100.0_f64, -1.0, -0.25, 0.0, 0.5, 1.0, 10.0, 500.0] {
        let out = execute_op1_math(Op1::Artan, sf(x)).unwrap();
        let err = (out.to_f64() - x.atan()).abs();
        assert!(err < 1e-7, "ARTAN({x}) err={err}");
    }
}

#[test]
fn exp_and_ln_accuracy_and_ln_quirks() {
    // Appendix-2 EXP: at least 8 significant digits.
    for x in [-5.0_f64, -1.0, 0.0, 1.0, 4.5] {
        let out = execute_op1_math(Op1::Exp, sf(x)).unwrap();
        let expected = x.exp();
        let rel_err = ((out.to_f64() - expected) / expected).abs();
        assert!(rel_err < 1e-8, "EXP({x}) rel_err={rel_err}");
    }

    // Appendix-2 LN: >= 9 decimal places in (1/e, e) and >= 9 significant digits outside.
    for x in [0.1_f64, 0.5, 1.0, 2.0, std::f64::consts::E, 100.0] {
        let out = execute_op1_math(Op1::Ln, sf(x)).unwrap();
        let expected = x.ln();
        let abs_err = (out.to_f64() - expected).abs();
        if (1.0 / std::f64::consts::E) < x && x < std::f64::consts::E {
            assert!(abs_err < 1e-9, "LN({x}) abs_err={abs_err}");
        } else {
            let rel_err = abs_err / expected.abs();
            assert!(rel_err < 1e-9, "LN({x}) rel_err={rel_err}");
        }
    }

    // Appendix-2 LN quirk: negative F yields same result as positive F (i.e. ln(|x|)).
    let ln_neg = execute_op1_math(Op1::Ln, sf(-2.0)).unwrap();
    assert!((ln_neg.to_f64() - 2.0_f64.ln()).abs() < 1e-9);

    // Appendix-2 LN quirk: zero F behaves as binary F = 1/2 (so value = ln(0.5 * 2^E)).
    let zero_with_exp4 = SpeedFloat {
        negative: false,
        fraction: 0,
        exp_negative: false,
        exponent: 4,
    };
    let ln_zero_q = execute_op1_math(Op1::Ln, zero_with_exp4).unwrap();
    assert!((ln_zero_q.to_f64() - 8.0_f64.ln()).abs() < 1e-9);
}
