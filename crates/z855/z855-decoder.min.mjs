#!/usr/bin/env -S deno run
import{assert as Z}from"jsr:@std/assert"
import{readAll as z}from"jsr:@std/io"
const J="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#",E=[...J],B=(new TextEncoder).encode(J),j=(new Map(E.map((Z,z)=>[Z,z])),new Map(B.entries().map(([Z,z])=>[z,Z]))),e=35,b={concatenatable:!1,extraSafeBytes:"_,~;|",maxRawLength:65536,unsafeSequences:[]},I={concatenatable:!0,extraSafeBytes:"_,~;|",maxRawLength:65536,unsafeSequences:[]},i={concatenatable:!0,extraSafeBytes:"_,~;| \"'`\\",maxRawLength:1536,unsafeSequences:["```"]}
function P(Z){let z=BigInt(Z);return z=(0x5555555555555555n&z)<<1n|z>>1n&0x5555555555555555n,z=(0x3333333333333333n&z)<<2n|z>>2n&0x3333333333333333n,z=(0x0f0f0f0f0f0f0f0fn&z)<<4n|z>>4n&0x0f0f0f0f0f0f0f0fn,z=(0x00ff00ff00ff00ffn&z)<<8n|z>>8n&0x00ff00ff00ff00ffn,z=(0x0000ffff0000ffffn&z)<<16n|z>>16n&0x0000ffff0000ffffn,z=z<<32n|z>>32n,z}function p(Z,z){return Z[0]!==z[0]?z[0]>Z[0]?-1:1:Z[1]!==z[1]?z[1]>Z[1]?-1:1:0}function N(Z,z){for(let J=0;4>J;J++){if(z[J]>Z[J])return-1
if(Z[J]>z[J])return 1}return 0}function n(Z){return Math.ceil(5*Z/4)}function G(Z,z){const J=Z.length,E=z.length;let B=0;for(const z of Z)B=85*B+z;const j=Math.pow(85,5-J),e=B*j,b=(B+1)*j;if(0===E)return e>4294967295?-1:e;let I=0;for(const Z of z)I=(I<<8|Z)>>>0
const i=Math.pow(2,8*E),P=e%i;let p=P>I?e-P+i+I:e-P+I;return p>=b||p>4294967295?-1:p}function g(Z,z){const J=Z.length,E=z.length;let B=0;for(const z of Z)B=85*B+z;const j=Math.pow(85,5-J),e=B*j,b=(B+1)*j;if(0===E)return e>4294967295?-1:e;let I=0
for(const Z of z)I=(I<<8|Z)>>>0;const i=Math.pow(2,8*E),P=e%i;let p=P>I?e-P+i+I:e-P+I;return p>=b||p>4294967295?-1:p}function F(Z,z,J){return G(((Z,z)=>{const J=[,,,,,];let E=Z;for(let Z=4;Z>=0;Z--)J[Z]=E%85,E=Math.floor(E/85)
return J.slice(0,z)})(Z,z),J)===Z}function f(Z){if(42>Z)return[E[Z]];const z=[];let J=Z;for(;J>0;)z.push(J%42),J=Math.floor(J/42);return z.reverse(),z.map((Z,z)=>E[0===z?Z:Z+42])}function T(Z,z,J,E){let B=0,j=null
for(let e=0;E>=e;e++){const b=e>0?f(e).length:0;if(b+e>E)continue
const I=Z+b+z+1+e,i=I+J-1,p=Math.floor(4*I/5),n=Math.floor(4*i/5),G=P(p),g=P(n),F=P(I),T=P(i),t=[g>G?G:g,G>g?G:g,T>F?F:T,F>T?F:T];(null===j||0>N(t,j))&&(j=t,B=e)}return B}function t(Z,z,J,E){if(z+J>Z.length)return!1
for(let B=0;J>B;B++)if(!E[Z[z+B]])return!1;return!0}function A(z,J=b){const{concatenatable:B,extraSafeBytes:I,maxRawLength:i}=Object.assign({},b,J),N=Array(256).fill(!1)
for(let z of I??[])"string"==typeof z&&(Z(1===z.length,"extra safe byte must be a single character"),z=z.charCodeAt(0)),Z(Number.isInteger(z)&&Number.isFinite(z)&&z>=0&&255>=z,"extra safe byte must be a byte value"),N[z]=!0
for(const Z of j.keys())N[Z]=!0;const G=N[95]&&i>=4,g=N[44]&&i>=5,A=N[126]&&i>=6,a=N[59]&&i>=7,O=N[124]&&i>=8;let o=new Uint8Array(5*Math.ceil(z.length/4)+16),S=0,s=0;function R(Z){if(S>=o.length){const Z=new Uint8Array(2*o.length)
Z.set(o),o=Z}o[S++]=Z}function h(Z){for(let z=0;Z.length>z;z++)R(Z.charCodeAt(z))}function L(Z,z,J){for(;S+J>o.length;){const Z=new Uint8Array(2*o.length);Z.set(o),o=Z}o.set(Z.subarray(z,z+J),S),S+=J}let D=z.length,d=0
if(B){const Z=n(z.length)%5;Z>0&&1!==Z&&(d=Z-1,D=z.length-d)}Z:for(;D>s;){const Z=D-s;if(4>Z){const J=Z;let E=0;for(let Z=0;J>Z;Z++)E=256*E+z[s+Z];const B=H(E,J+1);for(let Z=0;B.length>Z;Z++)R(B[Z]);s+=J
break}const J=l([z[s],z[s+1],z[s+2],z[s+3]]),j=r(J);if(!N[z[s+3]]){for(let Z=0;5>Z;Z++)R(j[Z]);s+=4;continue}let e=0;for(let Z=3;Z>=0&&N[z[s+Z]];Z--)e++;const b=s+4,I=Math.min(z.length,s+i);let d=0
for(let Z=b;I>Z&&i>d+e&&N[z[Z]];Z++)d++;const C=e+d,c=z.length-b-d;if(O&&C>=8&&4===e){const Z=s,J=B?Math.min(C,D-s):C,j=B?4*Math.floor(J/4):J
if(j>=8){if(0===c&&!B)return h(E[0]),h("|"),L(z,Z,j),s=z.length,o.subarray(0,S);const J=f(j),e=n(j),b=J.length+1+j
if(e>=b){const E=e-b,B=2>E?0:T(S,J.length,j,E),I=j>15&&B>0?f(B):[],i=5>I.length+J.length?I:[],P=i.length>0?B:0
if(E>=i.length+P){const B=E-i.length-P;for(const Z of i)h(Z);for(const Z of J)h(Z);h("|");for(let Z=0;P>Z;Z++)R(46)
L(z,Z,j);for(let Z=0;B>Z;Z++)R(46);s=Z+j;continue Z}}}}{let Z=!1
for(const J of[7,6,5]){if(!(7===J&&a||6===J&&A||5===J&&g)||J>i)continue
const E=7===J?";":6===J?"~":",",e=z.length-s,b=[]
for(let Z=0;(B?0:3)>=Z;Z++){const E=Z+J
if(s+E>z.length)continue;if((0===Z?1:Z+2)+n(e-E)!==n(e))continue
if(!t(z,s+Z,J,N))continue;const B=s+Z,j=B+J-1,I=P(B),i=P(j)
b.push({p:Z,key:[i>I?I:i,I>i?I:i]})}if(0!==b.length){b.sort((Z,z)=>p(Z.key,z.key))
for(const{p:B}of b){const e=s+B;if(0===B)h(E),L(z,e,J),s=e+J
else{for(let Z=0;B>=Z;Z++)R(j[Z]);h(E),L(z,e,J),s=e+J}Z=!0
break}if(Z)continue Z}}}if(G&&4===e)h("_"),L(z,s,4),s+=4
else{if(G&&!B){const Z=[];for(let J=1;3>=J;J++){const E=s+J
if(E+4>z.length)continue;if(!t(z,E,4,N))continue
if(s+8>z.length)continue;const B=E+3,j=P(E),e=P(B)
Z.push({p:J,key:[e>j?j:e,j>e?j:e]})}Z.sort((Z,z)=>p(Z.key,z.key))
for(const{p:E}of Z){const Z=s+E,B=4-E
if(!F(J,E,Array.from(z.subarray(Z,Z+B))))continue
for(let Z=0;E>Z;Z++)R(j[Z]);h("_"),L(z,Z,4)
const e=[];for(let J=0;E>J;J++)e.push(z[Z+B+J])
for(let Z=0;4-E>Z;Z++)e.push(z[s+4+E+Z])
const b=r(l(e));for(let Z=E;5>Z;Z++)R(b[Z]);s+=8
continue Z}}for(let Z=0;5>Z;Z++)R(j[Z])
s+=4}}if(B&&d>0){Z(S%5==0,"concat mode: unexpected outOff alignment "+S%5)
const J=d+1,E=5-J;for(let Z=0;E>Z;Z++)R(e);let B=0
for(let Z=0;d>Z;Z++)B=256*B+z[D+Z];const j=H(B,J)
for(let Z=0;j.length>Z;Z++)R(j[Z])}else if(B){const Z=S%5
if(Z>0){const z=5-Z,J=o.slice(S-Z,S);S-=Z
for(let Z=0;z>Z;Z++)R(e);for(let Z=0;J.length>Z;Z++)R(J[Z])}}return o.subarray(0,S)}function a(Z,z){let J=0,E=1,B=z,j=0
for(;B>0;){const z=Z[--B];if(j++,42>z){J+=z*E
break}J+=(z-42)*E,E*=42}return{value:J,count:j}}function O(Z){const{value:z,count:J}=a(Z,Z.length)
if(J===Z.length)return{offset:0,length:z};const{value:E}=a(Z,Z.length-J)
return{offset:E,length:z}}function o(Z,z,J){const E=Z.length;let B=0
for(const z of Z)B=256*B+z;const j=B<<8*(4-E)>>>0,e=Math.pow(85,J),b=j%e
return(b>z?j-b+e+z:j-b+z)>>>0}function S(Z){if(0===Z.length)return new Uint8Array(0)
const z=[];let J=0,E=[],B=[];for(;Z.length>J;){const b=Z[J]
if(b===e&&0===E.length&&J%5==0&&Z.length-J>=5){let b=0
for(;3>b&&Z[J+b]===e;)b++;if(b>0&&Z[J+b]!==e){const e=5-b,I=e-1
let i=0;for(let z=0;e>z;z++){const E=j.get(Z[J+b+z])
if(void 0===E)throw Error("invalid char in hash block")
i=85*i+E}if(i>[0,255,65535,16777215][I])throw Error("overflow in hash block")
for(let Z=I-1;Z>=0;Z--)z.push(i>>>8*Z&255);J+=5,E=[],B=[]
continue}}if(124===b){if(0===E.length)throw Error("'|' with no prefix digits")
J++;const{offset:j,length:b}=O(E);if(b>=1&&7>=b)throw Error("invalid | length "+b)
if(0===b){for(;Z.length>J;J++)z.push(Z[J]);E=[],B=[]
break}const I=E.length,i=n(b)-I-1-b-j
if(0>i)throw Error("invalid | offset")
if(J+=j,J+b>Z.length)throw Error("truncated | escape")
for(let E=0;b>E;E++)z.push(Z[J+E])
for(J+=b,J+=i;Z.length>J&&Z[J]===e;)J++;E=[],B=[]
continue}let I=0;if(95===b?I=4:44===b?I=5:126===b?I=6:59===b&&(I=7),I>0){if(J+I>=Z.length)throw Error("incomplete passthrough")
const j=[];for(let z=1;I>=z;z++)j.push(Z[J+z]);if(4===I){const Z=E.length;if(0===Z)z.push(...j),J+=5
else{const e=4-Z,b=G(E,j.slice(0,e));if(0>b)throw Error("non-aligned passthrough: invalid before-block")
z.push(...L(b)),B=j.slice(e),E=[],J+=5}}else{if(0===E.length){z.push(...j),J+=1+I
continue}const Z=E.length-1,e=g(E,j.slice(0,4-Z))
if(0>e)throw Error("extended passthrough: invalid before-block")
const b=L(e);for(let J=0;Z>J;J++)z.push(b[J])
z.push(...j),E=[],B=[],J+=1+I}continue}const i=j.get(b)
if(void 0===i)throw Error("invalid Z85 char 0x"+b.toString(16).toUpperCase())
E.push(i),J++;const P=5-B.length;if(E.length===P){const Z=h(E)
if(null===Z)throw Error("Z85 value overflow");let J
if(J=0===B.length?Z:o(B,Z,P),J>4294967295)throw Error("Z85 value overflow")
z.push(...L(J)),E=[],B=[]}}if(E.length>0){if(1===E.length)throw Error("invalid: single trailing Z85 char")
const Z=h(E);if(null===Z)throw Error("Z85 value overflow in partial block");const J=E.length-1
if(Z>[0,255,65535,16777215][J])throw Error("Z85 value overflow in partial block")
for(let E=J-1;E>=0;E--)z.push(Z>>>8*E&255)}return new Uint8Array(z)}async function s(){if("decode"===Deno.args[0]){const Z=await z(Deno.stdin)
await Deno.stdout.write(S(Z))}else{if("decode-lines"!==Deno.args[0])return await Deno.stderr.write((new TextEncoder).encode("Usage: z855 encode|decode|encode-lines|decode-lines < input > output\n")),2
{const Z=await z(Deno.stdin),J=(new TextDecoder).decode(Z).replace(/\n/g,"");await Deno.stdout.write(R(J))}}}function R(Z){return S((new TextEncoder).encode(Z))}function r(Z){const z=new Uint8Array(5)
for(let J=4;J>=0;J--)z[J]=B[Z%85],Z=Math.floor(Z/85);return z}function H(Z,z){const J=new Uint8Array(z);for(let E=z-1;E>=0;E--)J[E]=B[Z%85],Z=Math.floor(Z/85);return J}function h(Z){let z=0
for(let J=0;Z.length>J;J++)z=85*z+Z[J];return z>4294967295?null:z}function L(Z){return[Z>>>24&255,Z>>>16&255,Z>>>8&255,255&Z]}function l(Z){return(Z[0]<<24|Z[1]<<16|Z[2]<<8|Z[3])>>>0}import.meta.main&&Deno.exit(await s())
export{b as CANONICAL_ENCODING,I as CONCATENATABLE_ENCODING,i as PRINTABLE_ASCII_ENCODING,S as decode,A as encode,s as main,R as textDecode}