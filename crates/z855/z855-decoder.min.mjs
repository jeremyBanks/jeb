#!/usr/bin/env -S deno run
import{readAll as Z}from"jsr:@std/io"
const z="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#",J=[...z],E=(new TextEncoder).encode(z),B=(new Map(J.map((Z,z)=>[Z,z])),new Map(E.entries().map(([Z,z])=>[z,Z])))
function j(Z){return Math.ceil(5*Z/4)}function e(Z,z){const J=Z.length,E=z.length;let B=0;for(const z of Z)B=85*B+z;const j=Math.pow(85,5-J),e=B*j,b=(B+1)*j;if(0===E)return e>4294967295?-1:e;let I=0
for(const Z of z)I=(I<<8|Z)>>>0;const i=Math.pow(2,8*E),P=e%i;let p=P>I?e-P+i+I:e-P+I;return p>=b||p>4294967295?-1:p}function b(Z,z){const J=Z.length,E=z.length;let B=0;for(const z of Z)B=85*B+z
const j=Math.pow(85,5-J),e=B*j,b=(B+1)*j;if(0===E)return e>4294967295?-1:e;let I=0;for(const Z of z)I=(I<<8|Z)>>>0;const i=Math.pow(2,8*E),P=e%i;let p=P>I?e-P+i+I:e-P+I
return p>=b||p>4294967295?-1:p}function I(Z,z){let J=0,E=1,B=z,j=0;for(;B>0;){const z=Z[--B];if(j++,42>z){J+=z*E
break}J+=(z-42)*E,E*=42}return{value:J,count:j}}function i(Z){const{value:z,count:J}=I(Z,Z.length)
if(J===Z.length)return{offset:0,length:z};const{value:E}=I(Z,Z.length-J)
return{offset:E,length:z}}function P(Z,z,J){const E=Z.length;let B=0
for(const z of Z)B=256*B+z;const j=B<<8*(4-E)>>>0,e=Math.pow(85,J),b=j%e
return(b>z?j-b+e+z:j-b+z)>>>0}function p(Z){if(0===Z.length)return new Uint8Array(0)
const z=[];let J=0,E=[],I=[];for(;Z.length>J;){const p=Z[J]
if(35===p&&0===E.length&&J%5==0&&Z.length-J>=5){let j=0
for(;3>j&&35===Z[J+j];)j++;if(j>0&&35!==Z[J+j]){const e=5-j,b=e-1
let i=0;for(let z=0;e>z;z++){const E=B.get(Z[J+j+z])
if(void 0===E)throw Error("invalid char in hash block")
i=85*i+E}if(i>[0,255,65535,16777215][b])throw Error("overflow in hash block")
for(let Z=b-1;Z>=0;Z--)z.push(i>>>8*Z&255);J+=5,E=[],I=[]
continue}}if(124===p){if(0===E.length)throw Error("'|' with no prefix digits")
J++;const{offset:B,length:e}=i(E);if(e>=1&&7>=e)throw Error("invalid | length "+e)
if(0===e){for(;Z.length>J;J++)z.push(Z[J]);E=[],I=[]
break}const b=E.length,P=j(e)-b-1-e-B
if(0>P)throw Error("invalid | offset")
if(J+=B,J+e>Z.length)throw Error("truncated | escape")
for(let E=0;e>E;E++)z.push(Z[J+E])
for(J+=e,J+=P;Z.length>J&&35===Z[J];)J++;E=[],I=[]
continue}let N=0;if(95===p?N=4:44===p?N=5:126===p?N=6:59===p&&(N=7),N>0){if(J+N>=Z.length)throw Error("incomplete passthrough")
const B=[];for(let z=1;N>=z;z++)B.push(Z[J+z]);if(4===N){const Z=E.length;if(0===Z)z.push(...B),J+=5
else{const j=4-Z,b=e(E,B.slice(0,j));if(0>b)throw Error("non-aligned passthrough: invalid before-block")
z.push(...g(b)),I=B.slice(j),E=[],J+=5}}else{if(0===E.length){z.push(...B),J+=1+N
continue}const Z=E.length-1,j=b(E,B.slice(0,4-Z))
if(0>j)throw Error("extended passthrough: invalid before-block")
const e=g(j);for(let J=0;Z>J;J++)z.push(e[J])
z.push(...B),E=[],I=[],J+=1+N}continue}const n=B.get(p)
if(void 0===n)throw Error("invalid Z85 char 0x"+p.toString(16).toUpperCase())
E.push(n),J++;const F=5-I.length;if(E.length===F){const Z=G(E)
if(null===Z)throw Error("Z85 value overflow");let J
if(J=0===I.length?Z:P(I,Z,F),J>4294967295)throw Error("Z85 value overflow")
z.push(...g(J)),E=[],I=[]}}if(E.length>0){if(1===E.length)throw Error("invalid: single trailing Z85 char")
const Z=G(E);if(null===Z)throw Error("Z85 value overflow in partial block");const J=E.length-1
if(Z>[0,255,65535,16777215][J])throw Error("Z85 value overflow in partial block")
for(let E=J-1;E>=0;E--)z.push(Z>>>8*E&255)}return new Uint8Array(z)}async function N(){if("decode"===Deno.args[0]){const z=await Z(Deno.stdin)
await Deno.stdout.write(p(z))}else{if("decode-lines"!==Deno.args[0])return await Deno.stderr.write((new TextEncoder).encode("Usage: z855 encode|decode|encode-lines|decode-lines < input > output\n")),2
{const z=await Z(Deno.stdin),J=(new TextDecoder).decode(z).replace(/\n/g,"");await Deno.stdout.write(n(J))}}}function n(Z){return p((new TextEncoder).encode(Z))}function G(Z){let z=0
for(let J=0;Z.length>J;J++)z=85*z+Z[J];return z>4294967295?null:z}function g(Z){return[Z>>>24&255,Z>>>16&255,Z>>>8&255,255&Z]}import.meta.main&&Deno.exit(await N())
export{p as decode,N as main,n as textDecode}