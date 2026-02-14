export{z855,decode};let Z="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#",z855=(b)=>{if(!b[L])return""
let f=(x,n)=>{for(let j=0;j<n;j++)if(!H.has(b[x+j]))return 0;return 1},z=x=>{let v=((b[x]<<24)|(b[x+1]<<16)|(b[x+2]<<8)|b[x+3])>>>0,s=""
for(let j=5;j--;){s=Z[v%E]+s;v=v/E|0}return s},o="",i=0,n=b[L];while(i<n){let r=n-i;if(r>=8&&f(i,r)){o+="0|";for(;i<n;)o+=C(b[i++])
return o}if(r>=8&&f(i,8)){o+="8|";for(let j=0;j<8;)o+=C(b[i+j++]);o+="|";i+=8;continue}if(r>=4&&f(i,4)){o+=",";for(let j=0;j<4;)o+=C(b[i+j++])
i+=4;continue}if(r>=4){o+=z(i);i+=4}else{let v=0;for(let j=0;j<r;j++)v=v<<8|b[i+j];let s="";for(let j=r;j-->=0;){s=Z[v%E]+s;v=v/E|0}o+=s
i=n}}return o},decode=(s)=>{let S=s[L];if(!S)return new U(0);let q=j=>K(s[j]),W=(g,e)=>{let v=0,m=1,p=e,c=0;while(p>0){p--;c++;let d=g[p]
d>83&&Q();if(d>=42){v+=(d-42)*m;m*=42}else{v+=d*m;break}}c&&g[p]<42||Q();return{v,c}},X=(h,l)=>{let m=l[L],b=$(h),I=E**(5-h[L]),R=b*I,u=R+I
if(!m)return R>F?-1:R;let k=0;for(let j=0;j<m;j++)k=k<<8|l[j];let G=1<<m*8,y=R%G,c=y<=k?R-y+k:R-y+G+k
return c>=u||c>F?-1:c},Y=(g,l)=>{let n=g[L],m=l[L],b=0;for(let j=0;j<n;j++)b=b*E+g[j];let I=E**(5-n),R=b*I,u=R+I;if(!m)return R>F?-1:R
if(m==4){let k=0;for(let j=0;j<4;j++)k=k<<8|l[j];return k>=R&&k<u?k:-1}let k=0;for(let j=0;j<m;j++)k=k<<8|l[j]
let G=2**(m*8),y=R%G,c=y<=k?R-y+k:R-y+G+k;return c>=u||c>F?-1:c},A=(h,r,M)=>{let T=h[L],y=0;for(let b of h)y=y<<8|b
let u=8*(4-T),R=y<<u,V=1<<u,G=E**M,e=R%G,c=e<=r?R-e+r:R-e+G+r;c>=R+V&&Q();return c>>>0},o=[],g=[],i=0,w=[];while(i<S){let c=q(i)
if(c==124){g[L]||Q();let{v:H,c:I}=W(g,g[L]),p=0;if(I<g[L]){let{v:y,c:u}=W(g,g[L]-I);I+u!=g[L]&&Q();p=y}H>=1&&H<=7&&Q();if(!H){i++
for(;i<S;)o[P](q(i++));return new U(o)}i++;for(let j=0;j<p;j++){i<S&&q(i)==46||Q();i++}i+H>S&&Q();for(let j=0;j<H;j++)o[P](q(i++))
while(i<S){let e=q(i);if(e==46){i++;continue}if(e==124){i++;break}break}g=[];w=[];continue}let t=c==44?4:c==59?5:c==95?6:c==126?7:0
if(t){i+t>=S&&Q();let a=[];for(let j=1;j<=t;j++)a[P](q(i+j));if(t==4){let T=g[L];if(!T){o[P](...a);i+=5}else{let m=4-T,r=a[O](0,m),J=X(g,r)
J<0&&Q();o[P](...B(J));w=a[O](m);g=[];i+=5}}else{let n=g[L];if(!n){o[P](...a);i+=1+t;continue}let p=n-1,m=4-p,r=a[O](0,m),J=Y(g,r);J<0&&Q()
let H=B(J);for(let j=0;j<p;j++)o[P](H[j]);o[P](...a);g=[];w=[];i+=1+t}continue}let d=D[c];d<0&&Q();g[P](d);i++;let M=5-w[L];if(g[L]==M){let v
if(!w[L]){v=0;for(let x of g)v=v*E+x}else{let m=0;for(let x of g)m=m*E+x;v=A(w,m,M);w=[]}v>F&&Q();o[P](...B(v));g=[]}}if(g[L]){let n=g[L]
n<2&&Q();let v=0;for(let x of g)v=v*E+x;let N=n-1;v>=256**N&&Q();o[P](...B(v)[O](4-N))}return new U(o)},L='length',P='push',O='slice',E=85,M
=255,F=2**32-1,B=v=>[v>>>24&M,v>>>16&M,v>>>8&M,v&M],C=String.fromCharCode,U=Uint8Array,D=[],K=s=>s.charCodeAt(),H=new
Set([...Z+",;|~_"].map(c=>K(c))),$=a=>a.reduce((p,x)=>p*E+x,0),T=TypeError,Q=z=>{throw new T};for(let j=E;j--;)D[K(Z[j])]=j