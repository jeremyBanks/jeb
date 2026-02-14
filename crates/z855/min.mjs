export{z855,decode};let Z="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#",_=x=>~~(x+.99),_e=k=>k<6?";":k<7?"_":"~",srt=(a,c)=>a.z!==c.z?a.z<c.z?-1:1:a.t!==c.t?a.t<c.t?-1:1:0,b85=(v,n)=>{let s="";for(let j=n;j--;){s=Z[v%E]+s;v=v/E|0}return s},z855=b=>{if(!b[L])return""
let S=new Set([...Z+",;|~_"].map(c=>c.charCodeAt())),f=(i,k)=>{if(i+k>b[L])return 0;for(let j=0;j<k;j++)if(!S.has(b[i+j]))return 0;return 1}
let A=v=>{let d=[];for(let i=0;i<5;i++){d.unshift(v%E);v=v/E|0}return d},R=n=>{n=((n&0x55555555)<<1)|((n>>>1)&0x55555555);n=((n&0x33333333)<<2)|((n>>>2)&0x33333333);n=((n&0x0f0f0f0f)<<4)|((n>>>4)&0x0f0f0f0f);n=((n&0x00ff00ff)<<8)|((n>>>8)&0x00ff00ff);n=(n<<16)|(n>>>16);return n>>>0},G=n=>{if(n<42)return[Z[n]];let d=[]
while(n>0){d[P](n%42);n=n/42|0}d.reverse();let r=[];for(let i=0;i<d[L];i++)r[P](Z[d[i]+(i>0?42:0)]);return r},D=(v,p)=>A(v)[O](0,p),H=(v,p)=>A(v)[O](0,p).map(x=>Z[x]),W=(v,n)=>A(v)[O](5-n).map(x=>Z[x])
let N=(h,l)=>{let p=h[L],m=l[L],s=0;for(let d of h)s=s*E+d;let w=E**(5-p),r=s*w;if(!m)return r>F?-1:r;let k=0
for(let y of l)k=(k<<8)|y;let q=1<<(m*8),z=r%q,c=z<=k?r-z+k:r-z+q+k;return c>=(s+1)*w||c>F?-1:c},J=(v,p,l)=>{let h=D(v,p),c=N(h,l)
return v===c},o="",i=0,n=b[L],u=0;while(i<n){let r=n-i;if(r>=8){let e=0;for(let j=i;j<n&&e<65536;j++)if(S.has(b[j]))e++;else break
if(e>=8){let a=i+e===n;if(a){o+="0|";for(let j=i;j<n;j++)o+=C(b[j]);return o}else{let w=e<65536?e:65536,g=G(w),t=_(r*5/4)
let A=_((r-w)*5/4),V=t-A,B=g[L]+1+w;if(B<=V){let d=V-B,X=0,Y=null;for(let j=0;j<=d;j++){let m=j>0?G(j)[L]:0;if(m+j>d)continue
let s=u+m+g[L]+1+j,I=s+w-1,x=s*4/5|0,y=I*4/5|0,U=R(x),K=R(y),Q=R(s),T=R(I),k=[U<K?U:K,U>K?U:K,Q<T?Q:T,Q>T?Q:T]
if(Y===null||k[0]<Y[0]||k[0]===Y[0]&&k[1]<Y[1]||k[0]===Y[0]&&k[1]===Y[1]&&k[2]<Y[2]||k[0]===Y[0]&&k[1]===Y[1]&&k[2]===Y[2]&&k[3]<Y[3]){Y=k;X=j}}
let q=X>0?G(X):[];o+=q.join("")+g.join("")+"|"+".".repeat(X);for(let j=0;j<w;j++)o+=C(b[i+j]);let p=d-q[L]-X
o+=".".repeat(p);u+=V;i+=w;continue}}}}if(r>=5){let T=0;for(let k of[7,6,5]){if(r<k||T)continue
if(f(i,k)){let h=r,a=h-k,y=k+1;if(y+_(a*5/4)===_(h*5/4)){let j=_e(k);o+=j;for(let l=0;l<k;l++)o+=C(b[i+l])
u+=y;i+=k;T=1;break}}if(!T){let x=[];for(let p=0;p<=3;p++){let m=p+k;if(i+m>n)continue;let a=r-m,y=m+2;if(y+_(a*5/4)!==_(r*5/4))continue
let s=i+p,I=s+k-1,U=R(s),K=R(I),z=U<K?U:K,t=U<K?K:U;x[P]({z,t,p})}x.sort(srt);for(let {p} of x){let s=i+p;if(!f(s,k))continue;let v=$(b,i)
let h=H(v,p+1),j=_e(k);o+=h.join("")+j;for(let l=0;l<k;l++)o+=C(b[s+l]);u+=p+k+2;i+=p+k;T=1;break}if(T)break}}if(T)continue}
if(r>=4&&f(i,4)){o+=",";for(let j=0;j<4;j++)o+=C(b[i+j]);u+=5;i+=4;continue}if(r>=4){let x=[];for(let p=1;p<=3;p++){let s=i+p
if(s+4>n)continue;if(!f(s,4))continue;let I=s+3,U=R(s),K=R(I);x[P]({z:U<K?U:K,t:U<K?K:U,p})}x.sort(srt);let V=0;for(let {p} of x){let s=i+p,y=[b[s],b[s+1],b[s+2],b[s+3]],v=$(b,i)
let m=4-p,l=y[O](0,m);if(!J(v,p,l))continue;let h=y[O](m),a=i+4,w=a+p,e=4-p;if(w+e>n)continue;let z=[];for(let j=0;j<p;j++)z[P](h[j])
for(let j=0;j<e;j++)z[P](b[w+j]);let A=$(z,0),g=H(v,p);o+=g.join("")+",";for(let j of y)o+=C(j)
o+=W(A,5-p).join("");u+=10;i+=8;V=1;break}if(V)continue}if(r>=4){let v=$(b,i);o+=b85(v,5);u+=5;i+=4}else{let m=r,c=m+1,v=m===1?b[i]:m===2?(b[i]<<8)|b[i+1]:(b[i]<<16)|(b[i+1]<<8)|b[i+2];o+=b85(v,c);u+=c;i+=m}}return o},decode=(s)=>{let S=s[L];if(!S)return new U(0);let q=j=>K(s[j]),W=(g,e)=>{let v=0,m=1,p=e,c=0;while(p>0){p--;c++;let d=g[p]
d>83&&Q();if(d>=42){v+=(d-42)*m;m*=42}else{v+=d*m;break}}c&&g[p]<42||Q();return{v,c}},X=(h,l)=>{let m=l[L],b=h.reduce((p,x)=>p*E+x,0),I=E**(5-h[L]),R=b*I,u=R+I
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
n<2&&Q();let v=0;for(let x of g)v=v*E+x;let N=n-1;v>=256**N&&Q();o[P](...B(v)[O](4-N))}return new
U(o)},L='length',P='push',O='slice',E=85,M=255,F=2**32-1,B=v=>[v>>>24&M,v>>>16&M,v>>>8&M,v&M],C=String.fromCharCode,Q=z=>{throw new
TypeError},U=Uint8Array,D=[],K=s=>s.charCodeAt(),H=new Set([...Z+",;|~_"].map(c=>K(c))),$=(a,i)=>((a[i]<<24)|(a[i+1]<<16)|(a[i+2]<<8)|a[i+3])>>>0
for(let j=E;j--;)D[K(Z[j])]=j
