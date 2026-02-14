export{z855,decode};let Z="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#"

// Canonical Z855 encoder
// Priority: 8+ byte | escape > 5/6/7 byte ;_~ escapes > 4-byte , escape (aligned) > 4-byte , escape (non-aligned with canonical minimum) > standard Z85
let z855=(b)=>{
  if(!b[L])return""

  // Build safe char lookup table
  let S=new Set([...Z+",;|~_"].map(c=>c.charCodeAt()))

  // Check if K consecutive bytes starting at index are all safe
  let isSafe=(i,k)=>{
    if(i+k>b[L])return 0
    for(let j=0;j<k;j++)if(!S.has(b[i+j]))return 0
    return 1
  }

  // Calculate Z855 output length for given input byte count
  let outLen=n=>Math.ceil(n*5/4)

  // Bit reverse a number (for alignment preference)
  let bitRev=n=>{
    let x=BigInt(n)
    x=((x&0x5555555555555555n)<<1n)|((x>>1n)&0x5555555555555555n)
    x=((x&0x3333333333333333n)<<2n)|((x>>2n)&0x3333333333333333n)
    x=((x&0x0f0f0f0f0f0f0f0fn)<<4n)|((x>>4n)&0x0f0f0f0f0f0f0f0fn)
    x=((x&0x00ff00ff00ff00ffn)<<8n)|((x>>8n)&0x00ff00ff00ff00ffn)
    x=((x&0x0000ffff0000ffffn)<<16n)|((x>>16n)&0x0000ffff0000ffffn)
    x=(x<<32n)|(x>>32n)
    return x
  }

  // Generate base-42 prefix for long escape
  let genPrefix=n=>{
    if(n<42)return[Z[n]]
    let d=[]
    while(n>0){d[P](n%42);n=Math.floor(n/42)}
    d.reverse()
    let r=[]
    for(let i=0;i<d[L];i++)r[P](Z[d[i]+(i>0?42:0)])
    return r
  }

  // Get high-order P Z85 digits for a value
  let getHighDigits=(v,p)=>{
    let d=[]
    for(let i=0;i<5;i++){d.unshift(v%E);v=Math.floor(v/E)}
    return d[O](0,p)
  }

  // Get high-order P Z85 chars for a value
  let getHighChars=(v,p)=>{
    let d=[]
    for(let i=0;i<5;i++){d.unshift(v%E);v=Math.floor(v/E)}
    return d[O](0,p).map(x=>Z[x])
  }

  // Get low-order numChars Z85 chars for a value
  let getLowChars=(v,nc)=>{
    let d=[]
    for(let i=0;i<5;i++){d.unshift(v%E);v=Math.floor(v/E)}
    return d[O](5-nc).map(x=>Z[x])
  }

  // Compute canonical minimum for non-aligned passthrough
  let canonMin=(highDigits,knownLowBytes)=>{
    let P=highDigits[L],numKnownBytes=knownLowBytes[L]
    let base=0
    for(let d of highDigits)base=base*E+d
    let power=Math.pow(E,5-P)
    let rangeStart=base*power
    if(numKnownBytes===0)return rangeStart>F?-1:rangeStart
    let knownPart=0
    for(let byte of knownLowBytes)knownPart=(knownPart<<8)|byte
    let modulus=1<<(numKnownBytes*8)
    let startRemainder=rangeStart%modulus
    let candidate=startRemainder<=knownPart?rangeStart-startRemainder+knownPart:rangeStart-startRemainder+modulus+knownPart
    let rangeEnd=(base+1)*power
    return candidate>=rangeEnd||candidate>F?-1:candidate
  }

  // Check if value is canonical minimum for given partial encoding
  let isCanonical=(v,p,knownLowBytes)=>{
    let highDigits=getHighDigits(v,p)
    let cm=canonMin(highDigits,knownLowBytes)
    return v===cm
  }

  // Encode a 4-byte block to 5 Z85 chars
  let enc4=(v)=>{
    let s=""
    for(let j=5;j--;){s=Z[v%E]+s;v=Math.floor(v/E)}
    return s
  }

  let o="",i=0,n=b[L],curOutLen=0

  while(i<n){
    let r=n-i

    // Priority 1: Try 8+ byte long escape (| escape)
    if(r>=8){
      let safeCount=0
      for(let j=i;j<n&&safeCount<65536;j++){
        if(S.has(b[j]))safeCount++
        else break
      }

      if(safeCount>=8){
        let atEnd=i+safeCount===n

        if(atEnd){
          // Use 0| (rest is raw)
          o+=Z[0]+"|"
          for(let j=i;j<n;j++)o+=C(b[j])
          return o
        }else{
          // Length-prefixed escape
          let rawLen=Math.min(safeCount,65536)
          let lengthPrefix=genPrefix(rawLen)
          let totalStdLen=outLen(r)
          let afterLen=outLen(r-rawLen)
          let availChars=totalStdLen-afterLen
          let ourLen=lengthPrefix[L]+1+rawLen

          if(ourLen<=availChars){
            let paddingNeeded=availChars-ourLen

            // Find best offset using bit-reversal
            let bestOffset=0
            let bestKey=null
            for(let offset=0;offset<=paddingNeeded;offset++){
              let offsetPrefixLen=offset>0?genPrefix(offset)[L]:0
              if(offsetPrefixLen+offset>paddingNeeded)continue

              let outStart=curOutLen+offsetPrefixLen+lengthPrefix[L]+1+offset
              let outEnd=outStart+rawLen-1
              let inStart=Math.floor(outStart*4/5)
              let inEnd=Math.floor(outEnd*4/5)
              let revInStart=bitRev(inStart)
              let revInEnd=bitRev(inEnd)
              let revOutStart=bitRev(outStart)
              let revOutEnd=bitRev(outEnd)

              let key=[
                revInStart<revInEnd?revInStart:revInEnd,
                revInStart>revInEnd?revInStart:revInEnd,
                revOutStart<revOutEnd?revOutStart:revOutEnd,
                revOutStart>revOutEnd?revOutStart:revOutEnd
              ]

              if(bestKey===null||
                key[0]<bestKey[0]||
                (key[0]===bestKey[0]&&key[1]<bestKey[1])||
                (key[0]===bestKey[0]&&key[1]===bestKey[1]&&key[2]<bestKey[2])||
                (key[0]===bestKey[0]&&key[1]===bestKey[1]&&key[2]===bestKey[2]&&key[3]<bestKey[3])){
                bestKey=key
                bestOffset=offset
              }
            }

            let offsetPrefix=bestOffset>0?genPrefix(bestOffset):[]
            o+=offsetPrefix.join("")+lengthPrefix.join("")+"|"
            for(let j=0;j<bestOffset;j++)o+="."
            for(let j=0;j<rawLen;j++)o+=C(b[i+j])
            let paddingAfter=paddingNeeded-offsetPrefix[L]-bestOffset
            for(let j=0;j<paddingAfter-1;j++)o+="."
            if(paddingAfter>0)o+="|"

            curOutLen+=availChars
            i+=rawLen
            continue
          }
        }
      }
    }

    // Priority 2: Try 5/6/7 byte extended passthrough (;_~ escapes)
    if(r>=5){
      let found=0
      for(let k of[7,6,5]){
        if(r<k||found)continue

        // Try block-aligned first
        if(isSafe(i,k)){
          let totalRemaining=r
          let remaining=totalRemaining-k
          let passthroughChars=k+1
          if(passthroughChars+outLen(remaining)===outLen(totalRemaining)){
            let esc=k===5?";":k===6?"_":"~"
            o+=esc
            for(let j=0;j<k;j++)o+=C(b[i+j])
            curOutLen+=passthroughChars
            i+=k
            found=1
            break
          }
        }

        if(!found){
          // Try non-aligned positions with bit-reversal sorting
          let candidates=[]
          for(let p=0;p<=3;p++){
            let bytesConsumed=p+k
            if(i+bytesConsumed>n)continue
            let remaining=r-bytesConsumed
            let passthroughChars=bytesConsumed+2
            if(passthroughChars+outLen(remaining)!==outLen(r))continue

            let start=i+p
            let end=start+k-1
            let revStart=bitRev(start)
            let revEnd=bitRev(end)
            let sortKey0=revStart<revEnd?revStart:revEnd
            let sortKey1=revStart<revEnd?revEnd:revStart
            candidates[P]({sortKey0,sortKey1,p})
          }

          candidates.sort((a,b)=>{
            if(a.sortKey0<b.sortKey0)return -1
            if(a.sortKey0>b.sortKey0)return 1
            if(a.sortKey1<b.sortKey1)return -1
            if(a.sortKey1>b.sortKey1)return 1
            return 0
          })

          for(let {p} of candidates){
            let passStart=i+p
            if(!isSafe(passStart,k))continue

            let beforeValue=((b[i]<<24)|(b[i+1]<<16)|(b[i+2]<<8)|b[i+3])>>>0
            let beforeChars=getHighChars(beforeValue,p+1)
            let esc=k===5?";":k===6?"_":"~"
            o+=beforeChars.join("")+esc
            for(let j=0;j<k;j++)o+=C(b[passStart+j])

            curOutLen+=p+k+2
            i+=p+k
            found=1
            break
          }
          if(found)break
        }
      }
      if(found)continue
    }

    // Priority 3: Try 4-byte aligned passthrough
    if(r>=4&&isSafe(i,4)){
      o+=","
      for(let j=0;j<4;j++)o+=C(b[i+j])
      curOutLen+=5
      i+=4
      continue
    }

    // Priority 4: Try 4-byte non-aligned passthrough
    if(r>=4){
      let candidates=[]
      for(let p=1;p<=3;p++){
        let passStart=i+p
        if(passStart+4>n)continue
        if(!isSafe(passStart,4))continue

        let start=passStart
        let end=start+3
        let revStart=bitRev(start)
        let revEnd=bitRev(end)
        candidates[P]({
          sortKey:[revStart<revEnd?revStart:revEnd,revStart<revEnd?revEnd:revStart],
          p
        })
      }

      candidates.sort((a,b)=>{
        if(a.sortKey[0]!==b.sortKey[0])return a.sortKey[0]<b.sortKey[0]?-1:1
        if(a.sortKey[1]!==b.sortKey[1])return a.sortKey[1]<b.sortKey[1]?-1:1
        return 0
      })

      let found=0
      for(let {p} of candidates){
        let passStart=i+p
        let passBytes=[b[passStart],b[passStart+1],b[passStart+2],b[passStart+3]]
        let beforeValue=((b[i]<<24)|(b[i+1]<<16)|(b[i+2]<<8)|b[i+3])>>>0
        let numKnownLowBytes=4-p
        let knownLowBytes=passBytes[O](0,numKnownLowBytes)

        if(!isCanonical(beforeValue,p,knownLowBytes))continue

        let knownHighBytes=passBytes[O](numKnownLowBytes)
        let afterBlockDataStart=i+4
        let afterRemainingStart=afterBlockDataStart+p
        let afterRemainingNeeded=4-p

        if(afterRemainingStart+afterRemainingNeeded>n)continue

        let afterBytes=[]
        for(let j=0;j<p;j++)afterBytes[P](knownHighBytes[j])
        for(let j=0;j<afterRemainingNeeded;j++)afterBytes[P](b[afterRemainingStart+j])

        let afterValue=((afterBytes[0]<<24)|(afterBytes[1]<<16)|(afterBytes[2]<<8)|afterBytes[3])>>>0

        let highChars=getHighChars(beforeValue,p)
        o+=highChars.join("")+","
        for(let byte of passBytes)o+=C(byte)
        let lowChars=getLowChars(afterValue,5-p)
        o+=lowChars.join("")

        curOutLen+=10
        i+=8
        found=1
        break
      }
      if(found)continue
    }

    // Priority 5: Standard Z85 encoding
    if(r>=4){
      let v=((b[i]<<24)|(b[i+1]<<16)|(b[i+2]<<8)|b[i+3])>>>0
      o+=enc4(v)
      curOutLen+=5
      i+=4
    }else{
      // Trailing partial block (1-3 bytes)
      let numBytes=r
      let numChars=numBytes+1
      let value=numBytes===1?b[i]:numBytes===2?(b[i]<<8)|b[i+1]:(b[i]<<16)|(b[i+1]<<8)|b[i+2]
      let s=""
      for(let j=numChars;j--;){s=Z[value%E]+s;value=Math.floor(value/E)}
      o+=s
      curOutLen+=numChars
      i+=numBytes
    }
  }

  return o
}

// Decoder (DO NOT CHANGE - already code-golfed and functional)
let decode=(s)=>{let S=s[L];if(!S)return new U(0);let q=j=>K(s[j]),W=(g,e)=>{let v=0,m=1,p=e,c=0;while(p>0){p--;c++;let d=g[p]
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
n<2&&Q();let v=0;for(let x of g)v=v*E+x;let N=n-1;v>=256**N&&Q();o[P](...B(v)[O](4-N))}return new
U(o)},L='length',P='push',O='slice',E=85,M=255,F=2**32-1,B=v=>[v>>>24&M,v>>>16&M,v>>>8&M,v&M],C=String.fromCharCode,Q=z=>{throw new
T},U=Uint8Array,D=[],K=s=>s.charCodeAt(),H=new Set([...Z+",;|~_"].map(c=>K(c))),$=a=>a.reduce((p,x)=>p*E+x,0),T=TypeError
for(let j=E;j--;)D[K(Z[j])]=j
