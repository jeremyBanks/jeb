//! Re-exports contents of the Math built-in as a modern module.

// Logical
export let absolute = Math.abs;
export let abs = absolute;
export let round = Math.round;
export let roundDown = Math.floor;
export let roundTowardsZero = Math.trunc;
export let roundUp = Math.ceil;
export let floor = roundDown;
export let trunc = roundTowardsZero;
export let ceil = roundUp;
export let sign = Math.sign;
export let greater = Math.max;
export let lesser = Math.min;
export let clamp = (min: number, input: number, max: number): number => {
  if (input < min) {
    return min;
  } else if (input > max) {
    return max;
  } else {
    return input;
  }
};
export let max = greater;
export let min = lesser;

// Magical
export let E = Math.E;
export let LN10 = Math.LN10;
export let LN2 = Math.LN2;
export let LOG10E = Math.LOG10E;
export let LOG2E = Math.LOG2E;
export let PI = Math.PI;
export let SQRT1_2 = Math.SQRT1_2;
export let SQRT2 = Math.SQRT2;
export let acos = Math.acos;
export let acosh = Math.acosh;
export let asin = Math.asin;
export let asinh = Math.asinh;
export let atan = Math.atan;
export let atan2 = Math.atan2;
export let atanh = Math.atanh;
export let cbrt = Math.cbrt;
export let cos = Math.cos;
export let cosh = Math.cosh;
export let exp = Math.exp;
export let expm1 = Math.expm1;
export let hypot = Math.hypot;
export let log = Math.log;
export let log10 = Math.log10;
export let log1p = Math.log1p;
export let log2 = Math.log2;
export let pow = Math.pow;
export let sin = Math.sin;
export let sinh = Math.sinh;
export let sqrt = Math.sqrt;
export let tan = Math.tan;
export let tanh = Math.tanh;
