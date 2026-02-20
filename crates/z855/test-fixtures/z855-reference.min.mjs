#!/usr/bin/env -S deno run --allow-read --allow-write
import{assert as Z}from"jsr:@std/assert"
import{parseArgs as z}from"jsr:@std/cli/parse-args"
import{readAll as J}from"jsr:@std/io"
const E="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#",B=[...E],j=(new TextEncoder).encode(E),e=(new Map(B.map((Z,z)=>[Z,z])),new Map(j.entries().map(([Z,z])=>[z,Z]))),b=35,I={concatenatable:!1,extraSafeBytes:"_,~;|",maxRawLength:65536,unsafeSequences:[]},i={concatenatable:!0,extraSafeBytes:"_,~;|",maxRawLength:65536,unsafeSequences:[]},P={concatenatable:!0,extraSafeBytes:"_,~;| \"'`\\",maxRawLength:1536,unsafeSequences:["```"]}
function p(Z){let z=BigInt(Z);return z=(0x5555555555555555n&z)<<1n|z>>1n&0x5555555555555555n,z=(0x3333333333333333n&z)<<2n|z>>2n&0x3333333333333333n,z=(0x0f0f0f0f0f0f0f0fn&z)<<4n|z>>4n&0x0f0f0f0f0f0f0f0fn,z=(0x00ff00ff00ff00ffn&z)<<8n|z>>8n&0x00ff00ff00ff00ffn,z=(0x0000ffff0000ffffn&z)<<16n|z>>16n&0x0000ffff0000ffffn,z=z<<32n|z>>32n,z}function N(Z,z){return Z[0]!==z[0]?z[0]>Z[0]?-1:1:Z[1]!==z[1]?z[1]>Z[1]?-1:1:0}function n(Z,z){for(let J=0;4>J;J++){if(z[J]>Z[J])return-1
if(Z[J]>z[J])return 1}return 0}function G(Z){return Math.ceil(5*Z/4)}function g(Z,z){const J=Z.length,E=z.length;let B=0;for(const z of Z)B=85*B+z;const j=Math.pow(85,5-J),e=B*j,b=(B+1)*j;if(0===E)return e>4294967295?-1:e;let I=0;for(const Z of z)I=(I<<8|Z)>>>0
const i=Math.pow(2,8*E),P=e%i;let p=P>I?e-P+i+I:e-P+I;return p>=b||p>4294967295?-1:p}function F(Z,z){const J=Z.length,E=z.length;let B=0;for(const z of Z)B=85*B+z;const j=Math.pow(85,5-J),e=B*j,b=(B+1)*j;if(0===E)return e>4294967295?-1:e;let I=0
for(const Z of z)I=(I<<8|Z)>>>0;const i=Math.pow(2,8*E),P=e%i;let p=P>I?e-P+i+I:e-P+I;return p>=b||p>4294967295?-1:p}function f(Z,z,J){return g(((Z,z)=>{const J=[,,,,,];let E=Z;for(let Z=4;Z>=0;Z--)J[Z]=E%85,E=Math.floor(E/85)
return J.slice(0,z)})(Z,z),J)===Z}function T(Z){if(42>Z)return[B[Z]];const z=[];let J=Z;for(;J>0;)z.push(J%42),J=Math.floor(J/42);return z.reverse(),z.map((Z,z)=>B[0===z?Z:Z+42])}function t(Z,z,J,E){let B=0,j=null
for(let e=0;E>=e;e++){const b=e>0?T(e).length:0;if(b+e>E)continue
const I=Z+b+z+1+e,i=I+J-1,P=Math.floor(4*I/5),N=Math.floor(4*i/5),G=p(P),g=p(N),F=p(I),f=p(i),t=[g>G?G:g,G>g?G:g,f>F?F:f,F>f?F:f];(null===j||0>n(t,j))&&(j=t,B=e)}return B}function A(Z,z,J,E){if(z+J>Z.length)return!1
for(let B=0;J>B;B++)if(!E[Z[z+B]])return!1;return!0}function a(z,J=I){const{concatenatable:E,extraSafeBytes:j,maxRawLength:i}=Object.assign({},I,J),P=Array(256).fill(!1)
for(let z of j??[])"string"==typeof z&&(Z(1===z.length,"extra safe byte must be a single character"),z=z.charCodeAt(0)),Z(Number.isInteger(z)&&Number.isFinite(z)&&z>=0&&255>=z,"extra safe byte must be a byte value"),P[z]=!0
for(const Z of e.keys())P[Z]=!0;const n=P[95]&&i>=4,g=P[44]&&i>=5,F=P[126]&&i>=6,a=P[59]&&i>=7,O=P[124]&&i>=8;let o=new Uint8Array(5*Math.ceil(z.length/4)+16),S=0,s=0;function R(Z){if(S>=o.length){const Z=new Uint8Array(2*o.length)
Z.set(o),o=Z}o[S++]=Z}function r(Z){for(let z=0;Z.length>z;z++)R(Z.charCodeAt(z))}function H(Z,z,J){for(;S+J>o.length;){const Z=new Uint8Array(2*o.length);Z.set(o),o=Z}o.set(Z.subarray(z,z+J),S),S+=J}let l=z.length,D=0
if(E){const Z=G(z.length)%5;Z>0&&1!==Z&&(D=Z-1,l=z.length-D)}Z:for(;l>s;){const Z=l-s;if(4>Z){const J=Z;let E=0;for(let Z=0;J>Z;Z++)E=256*E+z[s+Z];const B=L(E,J+1);for(let Z=0;B.length>Z;Z++)R(B[Z]);s+=J
break}const J=d([z[s],z[s+1],z[s+2],z[s+3]]),j=h(J);if(!P[z[s+3]]){for(let Z=0;5>Z;Z++)R(j[Z]);s+=4;continue}let e=0;for(let Z=3;Z>=0&&P[z[s+Z]];Z--)e++;const b=s+4,I=Math.min(z.length,s+i);let D=0
for(let Z=b;I>Z&&i>D+e&&P[z[Z]];Z++)D++;const C=e+D,c=z.length-b-D;if(O&&C>=8&&4===e){const Z=s,J=E?Math.min(C,l-s):C;if(0===c&&!E)return r(B[0]),r("|"),H(z,Z,J),s=z.length,o.subarray(0,S)
const j=T(J),e=G(J),b=j.length+1+J;if(e>=b){const E=e-b,B=2>E?0:t(S,j.length,J,E),I=J>15&&B>0?T(B):[],i=5>I.length+j.length?I:[],P=i.length>0?B:0;if(E>=i.length+P){const B=E-i.length-P
for(const Z of i)r(Z);for(const Z of j)r(Z);r("|");for(let Z=0;P>Z;Z++)R(46);H(z,Z,J);for(let Z=0;B>Z;Z++)R(46);s=Z+J;continue Z}}}{let Z=!1
for(const J of[7,6,5]){if(!(7===J&&a||6===J&&F||5===J&&g)||J>i)continue;const B=7===J?";":6===J?"~":",",e=z.length-s,b=[]
for(let Z=0;(E?0:3)>=Z;Z++){const E=Z+J;if(s+E>z.length)continue;if((0===Z?1:Z+2)+G(e-E)!==G(e))continue
if(!A(z,s+Z,J,P))continue;const B=s+Z,j=B+J-1,I=p(B),i=p(j)
b.push({p:Z,key:[i>I?I:i,I>i?I:i]})}if(0!==b.length){b.sort((Z,z)=>N(Z.key,z.key))
for(const{p:E}of b){const e=s+E;if(0===E)r(B),H(z,e,J),s=e+J
else{for(let Z=0;E>=Z;Z++)R(j[Z]);r(B),H(z,e,J),s=e+J}Z=!0
break}if(Z)continue Z}}}if(n&&4===e)r("_"),H(z,s,4),s+=4
else{if(n&&!E){const Z=[];for(let J=1;3>=J;J++){const E=s+J
if(E+4>z.length)continue;if(!A(z,E,4,P))continue
if(s+8>z.length)continue;const B=E+3,j=p(E),e=p(B)
Z.push({p:J,key:[e>j?j:e,j>e?j:e]})}Z.sort((Z,z)=>N(Z.key,z.key))
for(const{p:E}of Z){const Z=s+E,B=4-E
if(!f(J,E,Array.from(z.subarray(Z,Z+B))))continue
for(let Z=0;E>Z;Z++)R(j[Z]);r("_"),H(z,Z,4)
const e=[];for(let J=0;E>J;J++)e.push(z[Z+B+J])
for(let Z=0;4-E>Z;Z++)e.push(z[s+4+E+Z])
const b=h(d(e));for(let Z=E;5>Z;Z++)R(b[Z]);s+=8
continue Z}}for(let Z=0;5>Z;Z++)R(j[Z])
s+=4}}if(E&&D>0){Z(S%5==0,"concat mode: unexpected outOff alignment "+S%5)
const J=D+1,E=5-J;for(let Z=0;E>Z;Z++)R(b);let B=0
for(let Z=0;D>Z;Z++)B=256*B+z[l+Z];const j=L(B,J)
for(let Z=0;j.length>Z;Z++)R(j[Z])}else if(E){const Z=S%5
if(Z>0){const z=5-Z,J=o.slice(S-Z,S);S-=Z
for(let Z=0;z>Z;Z++)R(b);for(let Z=0;J.length>Z;Z++)R(J[Z])}}return o.subarray(0,S)}function O(Z,z){let J=0,E=1,B=z,j=0
for(;B>0;){const z=Z[--B];if(j++,42>z){J+=z*E
break}J+=(z-42)*E,E*=42}return{value:J,count:j}}function o(Z){const{value:z,count:J}=O(Z,Z.length)
if(J===Z.length)return{offset:0,length:z};const{value:E}=O(Z,Z.length-J)
return{offset:E,length:z}}function S(Z,z,J){const E=Z.length;let B=0
for(const z of Z)B=256*B+z;const j=B<<8*(4-E)>>>0,e=Math.pow(85,J),b=j%e
return(b>z?j-b+e+z:j-b+z)>>>0}function s(Z){if(0===Z.length)return new Uint8Array(0)
const z=[];let J=0,E=[],B=[];for(;Z.length>J;){const j=Z[J]
if(j===b&&0===E.length&&J%5==0&&Z.length-J>=5){let j=0
for(;3>j&&Z[J+j]===b;)j++;if(j>0&&Z[J+j]!==b){const b=5-j,I=b-1
let i=0;for(let z=0;b>z;z++){const E=e.get(Z[J+j+z])
if(void 0===E)throw Error("invalid char in hash block")
i=85*i+E}if(i>[0,255,65535,16777215][I])throw Error("overflow in hash block")
for(let Z=I-1;Z>=0;Z--)z.push(i>>>8*Z&255);J+=5,E=[],B=[]
continue}}if(124===j){if(0===E.length)throw Error("'|' with no prefix digits")
J++;const{offset:j,length:e}=o(E);if(e>=1&&7>=e)throw Error("invalid | length "+e)
if(0===e){for(;Z.length>J;J++)z.push(Z[J]);E=[],B=[]
break}const I=E.length,i=G(e)-I-1-e-j
if(0>i)throw Error("invalid | offset")
if(J+=j,J+e>Z.length)throw Error("truncated | escape")
for(let E=0;e>E;E++)z.push(Z[J+E])
for(J+=e,J+=i;Z.length>J&&Z[J]===b;)J++;E=[],B=[]
continue}let I=0;if(95===j?I=4:44===j?I=5:126===j?I=6:59===j&&(I=7),I>0){if(J+I>=Z.length)throw Error("incomplete passthrough")
const j=[];for(let z=1;I>=z;z++)j.push(Z[J+z]);if(4===I){const Z=E.length;if(0===Z)z.push(...j),J+=5
else{const e=4-Z,b=g(E,j.slice(0,e));if(0>b)throw Error("non-aligned passthrough: invalid before-block")
z.push(...D(b)),B=j.slice(e),E=[],J+=5}}else{if(0===E.length){z.push(...j),J+=1+I
continue}const Z=E.length-1,e=F(E,j.slice(0,4-Z))
if(0>e)throw Error("extended passthrough: invalid before-block")
const b=D(e);for(let J=0;Z>J;J++)z.push(b[J])
z.push(...j),E=[],B=[],J+=1+I}continue}const i=e.get(j)
if(void 0===i)throw Error("invalid Z85 char 0x"+j.toString(16).toUpperCase())
E.push(i),J++;const P=5-B.length;if(E.length===P){const Z=l(E)
if(null===Z)throw Error("Z85 value overflow");let J
if(J=0===B.length?Z:S(B,Z,P),J>4294967295)throw Error("Z85 value overflow")
z.push(...D(J)),E=[],B=[]}}if(E.length>0){if(1===E.length)throw Error("invalid: single trailing Z85 char")
const Z=l(E);if(null===Z)throw Error("Z85 value overflow in partial block");const J=E.length-1
if(Z>[0,255,65535,16777215][J])throw Error("Z85 value overflow in partial block")
for(let E=J-1;E>=0;E--)z.push(Z>>>8*E&255)}return new Uint8Array(z)}async function R(){if("encode"===Deno.args[0]){const E=z(Deno.args.slice(1),{boolean:["concatenatable"],negatable:["concatenatable"],string:["extra-safe-bytes","max-raw-length"],default:{concatenatable:I.concatenatable,"extra-safe-bytes":"","max-raw-length":""+I.maxRawLength}}),B=Number(E["max-raw-length"])
Z(Number.isInteger(B),"max-raw-length must be an integer"),Z(Number.isFinite(B),"max-raw-length must be finite"),Z(Number.MAX_SAFE_INTEGER>=B,"max-raw-length must be less than or equal to 2^53 - 1"),Z(B>0,"max-raw-length must be greater than 0")
const j=E["extra-safe-bytes"]?new Uint8Array([...E["extra-safe-bytes"]].map(Z=>Z.charCodeAt(0))):void 0,e=await J(Deno.stdin)
await Deno.stdout.write(a(e,{concatenatable:E.concatenatable,extraSafeBytes:j,maxRawLength:B}))}else if("encode-lines"===Deno.args[0]){const Z=r(await J(Deno.stdin),P),z=[]
for(let J=0;Z.length>J;J+=80)z.push(Z.slice(J,J+80));await Deno.stdout.write((new TextEncoder).encode(z.join("\n")+"\n"))}else if("decode"===Deno.args[0]){const Z=await J(Deno.stdin)
await Deno.stdout.write(s(Z))}else{if("decode-lines"!==Deno.args[0])return await Deno.stderr.write((new TextEncoder).encode("Usage: z855 encode|decode|encode-lines|decode-lines < input > output\n")),2
{const Z=await J(Deno.stdin),z=(new TextDecoder).decode(Z).replace(/\n/g,"")
await Deno.stdout.write(H(z))}}}function r(Z,z=I){return(new TextDecoder).decode(a(Z,z))}function H(Z){return s((new TextEncoder).encode(Z))}function h(Z){const z=new Uint8Array(5)
for(let J=4;J>=0;J--)z[J]=j[Z%85],Z=Math.floor(Z/85);return z}function L(Z,z){const J=new Uint8Array(z);for(let E=z-1;E>=0;E--)J[E]=j[Z%85],Z=Math.floor(Z/85)
return J}function l(Z){let z=0;for(let J=0;Z.length>J;J++)z=85*z+Z[J]
return z>4294967295?null:z}function D(Z){return[Z>>>24&255,Z>>>16&255,Z>>>8&255,255&Z]}function d(Z){return(Z[0]<<24|Z[1]<<16|Z[2]<<8|Z[3])>>>0}import.meta.main&&Deno.exit(await R())
export{I as CANONICAL_ENCODING,i as CONCATENATABLE_ENCODING,P as PRINTABLE_ASCII_ENCODING,s as decode,a as encode,R as main,H as textDecode,r as textEncode}