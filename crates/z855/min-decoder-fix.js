// Position-based padding skip for min.mjs long escape decoder
// This replaces the content-checking logic

// Helper: calculate total escape space
const calcSpace = (rawLen) => {
  if (rawLen < 8) return 0;
  const z85Len = (n) => Math.ceil(n * 5 / 4);
  return z85Len(rawLen) - z85Len(rawLen - 8);
};

// In the long escape handler (after reading offset H and length p):
// OLD (content-checking):
// for(let j=0;j<p;j++){i<S&&q(i)==46||Q();i++}  // skip dots before
// for(let j=0;j<H;j++)o[P](q(i++))  // read raw bytes
// while(i<S){let e=q(i);if(e==46){i++;continue}if(e==124){i++;break}break}  // skip dots/pipe after

// NEW (position-based):
const lenPrefixLen = g.length - I;  // g.length is total digits, I is offset digits
const avail = calcSpace(H);  // H is rawLen
const ourLen = lenPrefixLen + 1 + H;
const padNeeded = Math.max(0, avail - ourLen);
const padBefore = p;  // p is offset
const padAfter = Math.max(0, padNeeded - I - padBefore);  // I is offsetDigitsUsed

i++;  // skip |
i += padBefore;  // skip padding before (any content)
for(let j=0;j<H;j++)o[P](q(i++));  // read raw bytes
i += padAfter;  // skip padding after (any content)

// Minified version:
// let lpLen=g[L]-I,av=(r=>r<8?0:Math.ceil(r*5/4)-Math.ceil((r-8)*5/4))(H),pn=av-lpLen-1-H,pb=p,pa=pn-I-pb;pa<0&&(pa=0);pn<0&&(pn=0);i++;i+=pb;for(let j=0;j<H;j++)o[P](q(i++));i+=pa
