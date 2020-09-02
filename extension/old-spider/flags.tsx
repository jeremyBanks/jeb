export const flag = (countryCode?: string): JSX.Renderable => {
  if (!countryCode) {
    return undefined;
  }

  const letters = countryCode.toLowerCase().replace(/[^a-z]/g, "");
  let flag = "";
  const indicatorA = 0x1f1e6;
  const letterA = "a".codePointAt(0)!;
  for (const letter of letters) {
    flag += String.fromCodePoint(letter.codePointAt(0)! - letterA + indicatorA);
  }
  return flag;
};
