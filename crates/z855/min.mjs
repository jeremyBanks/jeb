#!/usr/bin/env -S deno run
export{z as z855,b as decode};let Z="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#",z=z=>{if(!z[p])return""
let J=(Z,J)=>{if(Z+J>z[p])return 0;for(let E=0;J>E;E++)if(!o.has(z[Z+E]))return 0;return 1},E=Z=>{let z=[];for(let J=0;5>J;J++)z.unshift(Z%G),Z=Z/G|0
return z},j=Z=>(Z=(Z=(I&(Z=(e&(Z=(b&(Z=(2&Z)<<1|Z>>>1&e))<<2|Z>>>2&b))<<4|Z>>>4&e))<<8|Z>>>8&I)<<16|Z>>>16)>>>0,g=z=>{if(42>z)return Z[z];let J=[]
for(;z>0;)J[N](z%42),z=z/42|0;J.reverse();let E="";for(let z=0;z<J[p];z++)E+=Z[J[z]+(z>0?42:0)]
return E},f=(Z,z)=>E(Z)[n](0,z),t=(z,J)=>E(z)[n](0,J).map(z=>Z[z]).join(""),A=(z,J)=>E(z)[n](5-J).map(z=>Z[z]).join(""),a=(Z,z)=>{let J=Z[p],E=z[p],B=0
for(let z of Z)B=B*G+z;let j=G**(5-J),e=B*j;if(!E)return e>F?-1:e;let b=0;for(let Z of z)b=b<<8|Z;let I=1<<8*E,i=e%I,P=i>b?e-i+I+b:e-i+b
return P>=(B+1)*j||P>F?-1:P},O="",s=0,R=z[p],r=0;for(;R>s;){let Z=R-s;if(Z>=8){let J=0;for(let Z=s;R>Z&&65536>J&&o.has(z[Z]);Z++)J++
if(J>=8){let E=g(J),B=i(5*Z/4)-i(5*(Z-J)/4),e=E[p]+1+J;if(B>=e){let Z=B-e,b=0,I=null;for(let z=0;Z>=z;z++){let B=z>0?g(z)[p]:0
if(B+z>Z)continue;let e=r+B+E[p]+1+z,i=e+J-1,P=4*i/5|0,N=j(4*e/5|0),n=j(P),G=j(e),F=j(i),f=[n>N?N:n,N>n?N:n,F>G?G:F,G>F?G:F];(null===I||f[0]<I[0]||f[0]===I[0]&&f[1]<I[1]||f[0]===I[0]&&f[1]===I[1]&&f[2]<I[2]||f[0]===I[0]&&f[1]===I[1]&&f[2]===I[2]&&f[3]<I[3])&&(I=f,b=z)}let i=b>0?g(b):""
O+=i+E+"|"+".".repeat(b);for(let Z=0;J>Z;Z++)O+=T(z[s+Z]);O+=".".repeat(Z-i[p]-b),r+=B,s+=J;continue}}}if(Z>=5){let E=0;for(let B of[7,6,5])if(Z>=B&&!E){if(J(s,B)&&B+1+i(5*(Z-B)/4)==i(5*Z/4)){O+=6>B?";":7>B?"_":"~";for(let Z=0;B>Z;Z++)O+=T(z[s+Z])
r+=B+1,s+=B,E=1;break}if(!E){let e=[];for(let z=0;3>=z;z++){let J=z+B;if(s+J>R)continue;if(J+2+i(5*(Z-J)/4)!=i(5*Z/4))continue;let E=s+z,b=E+B-1,I=j(E),P=j(b);e[N]({z:P>I?I:P,t:P>I?P:I,p:z})}e.sort(P);for(let{p:Z}of e){let j=s+Z
if(J(j,B)){O+=t(S(z,s),Z+1)+(6>B?";":7>B?"_":"~");for(let Z=0;B>Z;Z++)O+=T(z[j+Z]);r+=Z+B+2,s+=Z+B,E=1;break}}if(E)break}}if(E)continue}if(Z>=4&&J(s,4)){O+=",";for(let Z=0;4>Z;Z++)O+=T(z[s+Z]);r+=5,s+=4}else{if(Z>=4){let Z=[]
for(let z=1;3>=z;z++){let E=s+z;if(E+4>R)continue;if(!J(E,4))continue;let B=E+3,e=j(E),b=j(B);Z[N]({z:b>e?e:b,t:b>e?b:e,p:z})}Z.sort(P);let E=0;for(let{p:J}of Z){let Z=s+J,B=[z[Z],z[Z+1],z[Z+2],z[Z+3]],j=S(z,s),e=4-J,b=B[n](0,e)
if(j!=a(f(j,J),b))continue;let I=B[n](e);if(s+8>R)continue;let i=[];for(let Z=0;J>Z;Z++)i[N](I[Z]);for(let Z=0;4-J>Z;Z++)i[N](z[s+4+J+Z]);O+=t(j,J)+",";for(let Z of B)O+=T(Z);O+=A(S(i,0),5-J),r+=10,s+=8,E=1
break}if(E)continue}4>Z?(O+=B(1==Z?z[s]:2==Z?z[s]<<8|z[s+1]:z[s]<<16|z[s+1]<<8|z[s+2],Z+1),r+=Z+1,s+=Z):(O+=B(S(z,s),5),r+=5,s+=4)}}let H=O[p]%5;return H&&(O=O[n](0,-H)+"#".repeat(5-H)+O[n](-H)),O},B=(z,J)=>{let E=""
for(let B=J;B--;)E=Z[z%G]+E,z=z/G|0;return E};J=Z=>Z?Math.ceil(5*Z/4):0,E=Z=>{if(!Z)return 0;let z=4*Z/5|0;for(;z>0&&J(z)>Z;)z--;for(;J(z+1)<=Z;)z++
return z},e=252645135,b=858993459,I=16711935,p="length",N="push",n="slice",b=Z=>{let z=Z[p];if(!z)return new A(0)
let B=z=>O(Z[z]),j=(Z,z)=>{let J=0,E=1,B=z,j=0;for(;B>0;){B--,j++;let z=Z[B];if(z>83&&t(),42>z){J+=z*E
break}J+=(z-42)*E,E*=42}return j&&42>Z[B]||t(),{v:J,c:j}},e=(Z,z)=>{let J=z[p],E=Z.reduce((Z,z)=>Z*G+z,0),B=G**(5-Z[p]),j=E*B,e=j+B
if(!J)return j>F?-1:j;let b=0;for(let Z=0;J>Z;Z++)b=b<<8|z[Z];let I=1<<8*J,i=j%I,P=i>b?j-i+I+b:j-i+b
return P>=e||P>F?-1:P},b=(Z,z)=>{let J=Z[p],E=z[p],B=0;for(let z=0;J>z;z++)B=B*G+Z[z]
let j=G**(5-J),e=B*j,b=e+j;if(!E)return e>F?-1:e;if(4==E){let Z=0
for(let J=0;4>J;J++)Z=Z<<8|z[J];return Z>=e&&b>Z?Z:-1}let I=0
for(let Z=0;E>Z;Z++)I=I<<8|z[Z];let i=2**(8*E),P=e%i,N=P>I?e-P+i+I:e-P+I
return N>=b||N>F?-1:N},I=(Z,z,J)=>{let E=Z[p],B=0;for(let z of Z)B=B<<8|z
let j=8*(4-E),e=B<<j,b=G**J,I=e%b,i=I>z?e-I+b+z:e-I+z
return i>=e+(1<<j)&&t(),i>>>0},i=[],P=[],g=0,T=[]
for(;z>g;){let Z=B(g);if(124==Z){P[p]||t()
let{v:Z,c:e}=j(P,P[p]),b=0;if(e<P[p]){let{v:Z,c:z}=j(P,P[p]-e)
e+z!=P[p]&&t(),b=Z}if(Z>=1&&7>=Z&&t(),!Z){for(g++;z>g;)i[N](B(g++))
return new A(i)}g++;let I=P[p]-e,n=E(z)-i[p],G=n-Z,F=J(n)-J(G)-I-1-Z,f=F-e-b
0>f&&(f=0),0>F&&(F=0),g+=b,g+Z>z&&t();for(let z=0;Z>z;z++)i[N](B(g++))
g+=f,P=[],T=[];continue}let O=44==Z?4:59==Z?5:95==Z?6:126==Z?7:0
if(O){g+O>=z&&t();let Z=[];for(let z=1;O>=z;z++)Z[N](B(g+z))
if(4==O){let z=P[p];if(z){let J=4-z,E=e(P,Z[n](0,J))
0>E&&t(),i[N](...f(E)),T=Z[n](J),P=[],g+=5}else i[N](...Z),g+=5}else{let z=P[p]
if(!z){i[N](...Z),g+=1+O;continue}let J=z-1,E=b(P,Z[n](0,4-J));0>E&&t()
let B=f(E);for(let Z=0;J>Z;Z++)i[N](B[Z])
i[N](...Z),P=[],T=[],g+=1+O}continue}if(!(P[p]||T[p]||g%5||35!=Z)){let Z=0
for(;3>Z&&z>g+Z&&35==B(g+Z);)Z++;if(Z){g+=Z;continue}}let o=a[Z]
!(o>=0)&&t(),P[N](o),g++;let S=5-T[p];if(P[p]==S){let Z
if(T[p]){let z=0;for(let Z of P)z=z*G+Z
Z=I(T,z,S),T=[]}else{Z=0;for(let z of P)Z=Z*G+z}Z>F&&t(),i[N](...f(Z)),P=[]}}if(P[p]){let Z=P[p]
2>Z&&t();let z=0;for(let Z of P)z=z*G+Z;let J=Z-1
z>=256**J&&t(),i[N](...f(z)[n](4-J))}return new A(i)},i=Z=>~~(Z+.99),P=(Z,z)=>Z.z!=z.z?Z.z<z.z?-1:1:Z.t!=z.t?Z.t<z.t?-1:1:0,G=85,g=255,F=2**32-1,f=Z=>[Z>>>24&g,Z>>>16&g,Z>>>8&g,Z&g],T=String.fromCharCode,t=Z=>{throw new TypeError},A=Uint8Array,a=[],O=Z=>Z.charCodeAt(),o=new Set([...Z+",;|~_"].map(Z=>O(Z))),S=(Z,z)=>(Z[z]<<24|Z[z+1]<<16|Z[z+2]<<8|Z[z+3])>>>0
for(let z=G;z--;)a[O(Z[z])]=z;(import.meta.main||import.meta.url==="file://"+process?.argv?.[1])&&(async()=>{let Z="undefined"!=typeof Deno,z=Z?Deno.args:process?.argv||Bun?.argv||[],J=Z?z[0]:z[z[0]?.includes?.("node")||z[0]?.includes?.("bun")?2:1]
J&&["encode","decode"].includes(J)||(console.error("usage: min.mjs <encode|decode> <stdin >stdout"),(process||Deno||Bun).exit(2));let E="",B=new TextEncoder,j=new TextDecoder;if(Z){Deno.stdin.setRaw?.(!1)
for await(let Z of Deno.stdin.readable)E+=j.decode(Z,{stream:!0})}else{process.stdin.setEncoding("utf8");for await(let Z of process.stdin)E+=Z}E=E.trimEnd();let e="encode"===J?z855(B.encode(E)):j.decode(decode(E))
Z?Deno.stdout.writeSync("string"==typeof e?B.encode(e):e):(process?.stdout||Bun?.stdout)?.write?.(e)})()