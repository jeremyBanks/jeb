//! Re-exports contents of the Math built-in as a modern module.

// Logical
export const absolute = Math.abs;
export const abs = absolute;
export const round = Math.round;
export const roundDown = Math.floor;
export const roundTowardsZero = Math.trunc;
export const roundUp = Math.ceil;
export const floor = roundDown;
export const trunc = roundTowardsZero;
export const ceil = roundUp;
export const sign = Math.sign;
export const greater = Math.max;
export const lesser = Math.min;
export const clamp = (min: number, input: number, max: number): number => {
  if (input < min) {
    return min;
  } else if (input > max) {
    return max;
  } else {
    return input;
  }
};
export const max = greater;
export const min = lesser;

// Magical
export const E = Math.E;
export const LN10 = Math.LN10;
export const LN2 = Math.LN2;
export const LOG10E = Math.LOG10E;
export const LOG2E = Math.LOG2E;
export const PI = Math.PI;
export const SQRT1_2 = Math.SQRT1_2;
export const SQRT2 = Math.SQRT2;
export const acos = Math.acos;
export const acosh = Math.acosh;
export const asin = Math.asin;
export const asinh = Math.asinh;
export const atan = Math.atan;
export const atan2 = Math.atan2;
export const atanh = Math.atanh;
export const cbrt = Math.cbrt;
export const cos = Math.cos;
export const cosh = Math.cosh;
export const exp = Math.exp;
export const expm1 = Math.expm1;
export const hypot = Math.hypot;
export const log = Math.log;
export const log10 = Math.log10;
export const log1p = Math.log1p;
export const log2 = Math.log2;
export const pow = Math.pow;
export const sin = Math.sin;
export const sinh = Math.sinh;
export const sqrt = Math.sqrt;
export const tan = Math.tan;
export const tanh = Math.tanh;
