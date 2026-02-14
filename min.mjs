export default {encode,decode}
/**@returns {string}*/export function encode(/**@type {Uint8Array}*/input) {
  if(input.length===0)return""

  // Z85 alphabet
  const A="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#"
  // Safe chars (Z85 + ,;|~_)
  const S="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_"
  const safe=new Set([...S].map(c=>c.charCodeAt(0)))

  // Check if all bytes from idx are safe
  const allSafe=(idx,len)=>{
    for(let i=0;i<len;i++)if(!safe.has(input[idx+i]))return false
    return true
  }

  // Encode 4 bytes to 5 chars
  const enc4=(idx)=>{
    let v=((input[idx]<<24)|(input[idx+1]<<16)|(input[idx+2]<<8)|input[idx+3])>>>0
    let s=""
    for(let i=0;i<5;i++){s=A[v%85]+s;v=Math.floor(v/85)}
    return s
  }

  let out="",i=0

  while(i<input.length){
    const rem=input.length-i

    // Check for 0| (rest of input is all safe)
    if(rem>=8&&allSafe(i,rem)){
      out+=A[0]+"|"
      for(let j=i;j<input.length;j++)out+=String.fromCharCode(input[j])
      return out
    }

    // Check for 8| (exactly 8 safe bytes, not at end)
    if(rem>=8&&allSafe(i,8)&&(rem>8||!allSafe(i,rem))){
      out+=A[8]+"|"
      for(let j=0;j<8;j++)out+=String.fromCharCode(input[i+j])
      out+="|"
      i+=8
      continue
    }

    // Check for , (4 safe bytes block-aligned)
    if(rem>=4&&allSafe(i,4)){
      out+=","
      for(let j=0;j<4;j++)out+=String.fromCharCode(input[i+j])
      i+=4
      continue
    }

    // Standard Z85 encoding
    if(rem>=4){
      out+=enc4(i)
      i+=4
    }else{
      // Trailing 1-3 bytes
      let v=0
      for(let j=0;j<rem;j++)v=(v<<8)|input[i+j]
      let s=""
      for(let j=0;j<=rem;j++){s=A[v%85]+s;v=Math.floor(v/85)}
      out+=s
      i+=rem
    }
  }

  return out
}

/**@returns {Uint8Array}*/export function decode(/**@type {string}*/input) {
  if(input.length===0)return new Uint8Array(0)

  const A="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#"
  const D=new Array(256).fill(-1)
  for(let i=0;i<85;i++)D[A.charCodeAt(i)]=i

  const COMMA=44,SEMI=59,UNDER=95,TILDE=126,PIPE=124,DOT=46

  // Read base-42 number backwards from digits array
  const readB42=(digits,end)=>{
    let v=0,m=1,pos=end,cnt=0
    while(pos>0){
      pos--;cnt++
      const d=digits[pos]
      if(d>83)throw new Error("invalid prefix digit")
      if(d>=42){v+=(d-42)*m;m*=42}
      else{v+=d*m;break}
    }
    if(cnt===0||digits[pos]>=42)throw new Error("invalid prefix")
    return{value:v,consumed:cnt}
  }

  // Compute canonical minimum for non-aligned , passthrough
  const canonMin=(highDigits,knownLow)=>{
    const P=highDigits.length,nKnown=knownLow.length
    let base=0
    for(let i=0;i<P;i++)base=base*85+highDigits[i]
    const power=Math.pow(85,5-P)
    const rangeStart=base*power,rangeEnd=(base+1)*power
    if(nKnown===0)return rangeStart>0xffffffff?-1:rangeStart
    let known=0
    for(let i=0;i<nKnown;i++)known=(known<<8)|knownLow[i]
    const mod=1<<(nKnown*8)
    const rem=rangeStart%mod
    let c=rem<=known?rangeStart-rem+known:rangeStart-rem+mod+known
    return c>=rangeEnd||c>0xffffffff?-1:c
  }

  // Compute before block value for extended (;_~) passthrough
  const extendedBefore=(digits,knownLow)=>{
    const n=digits.length,p=n-1,nKnown=knownLow.length
    let base=0
    for(let i=0;i<n;i++)base=base*85+digits[i]
    const power=Math.pow(85,5-n)
    const rs=base*power,re=(base+1)*power
    if(nKnown===0)return rs>0xffffffff?-1:rs
    if(nKnown===4){
      let k=0
      for(let i=0;i<4;i++)k=(k<<8)|knownLow[i]
      return k>=rs&&k<re?k:-1
    }
    let known=0
    for(let i=0;i<nKnown;i++)known=(known<<8)|knownLow[i]
    const mod=Math.pow(2,nKnown*8)
    const rem=rs%mod
    let c=rem<=known?rs-rem+known:rs-rem+mod+known
    return c>=re||c>0xffffffff?-1:c
  }

  // Reconstruct after block from known high bytes + low digits
  const reconAfter=(highBytes,lowVal,numDigits)=>{
    const P=highBytes.length
    let high=0
    for(const b of highBytes)high=(high<<8)|b
    const shift=8*(4-P)
    const rs=high<<shift,rSize=1<<shift
    const mod=Math.pow(85,numDigits)
    const rem=rs%mod
    let c=rem<=lowVal?rs-rem+lowVal:rs-rem+mod+lowVal
    if(c>=rs+rSize)throw new Error("invalid after block")
    return c>>>0
  }

  const out=[],curDigits=[]
  let i=0,blockPos=0,knownHigh=[]

  while(i<input.length){
    const c=input.charCodeAt(i)

    // Check for | escape
    if(c===PIPE){
      if(curDigits.length===0)throw new Error("no prefix before |")
      // Read offset and length
      const{value:len,consumed:lc}=readB42(curDigits,curDigits.length)
      let offset=0
      if(lc<curDigits.length){
        const{value:off,consumed:oc}=readB42(curDigits,curDigits.length-lc)
        if(lc+oc!==curDigits.length)throw new Error("invalid prefix structure")
        offset=off
      }
      if(len>=1&&len<=7)throw new Error("invalid length for | escape")
      if(len===0){
        i++
        for(;i<input.length;i++)out.push(input.charCodeAt(i))
        return new Uint8Array(out)
      }
      // len >= 8
      i++
      // Skip offset padding
      for(let j=0;j<offset;j++){
        if(i>=input.length||input.charCodeAt(i)!==DOT)throw new Error("bad offset padding")
        i++
      }
      if(i+len>input.length)throw new Error("insufficient bytes for | escape")
      for(let j=0;j<len;j++)out.push(input.charCodeAt(i++))
      // Skip trailing padding/terminator
      while(i<input.length){
        const nc=input.charCodeAt(i)
        if(nc===DOT){i++;continue}
        if(nc===PIPE){i++;break}
        break
      }
      curDigits.length=0;blockPos=0;knownHigh=[]
      continue
    }

    // Check for passthrough escapes
    const passLen=c===COMMA?4:c===SEMI?5:c===UNDER?6:c===TILDE?7:0

    if(passLen>0){
      if(i+passLen>=input.length)throw new Error("incomplete passthrough")
      const pass=[]
      for(let j=1;j<=passLen;j++)pass.push(input.charCodeAt(i+j))

      if(passLen===4){
        const P=curDigits.length
        if(P===0){
          // Block-aligned
          out.push(...pass)
          i+=5
        }else{
          // Non-aligned
          const nKnownLow=4-P,knownLow=pass.slice(0,nKnownLow)
          const bv=canonMin(curDigits,knownLow)
          if(bv<0)throw new Error("invalid non-aligned passthrough")
          out.push((bv>>>24)&0xff,(bv>>>16)&0xff,(bv>>>8)&0xff,bv&0xff)
          knownHigh=pass.slice(nKnownLow)
          curDigits.length=0;blockPos=0
          i+=5
        }
      }else{
        // 5/6/7 byte passthrough
        const n=curDigits.length
        if(n===0){
          out.push(...pass)
          i+=1+passLen
          continue
        }
        const p=n-1,nKnownLow=4-p,knownLow=pass.slice(0,nKnownLow)
        const bv=extendedBefore(curDigits,knownLow)
        if(bv<0)throw new Error("invalid extended passthrough")
        const bb=[(bv>>>24)&0xff,(bv>>>16)&0xff,(bv>>>8)&0xff,bv&0xff]
        for(let j=0;j<p;j++)out.push(bb[j])
        out.push(...pass)
        curDigits.length=0;blockPos=0;knownHigh=[]
        i+=1+passLen
      }
      continue
    }

    // Regular Z85 character
    const d=D[c]
    if(d===-1)throw new Error(`invalid character: 0x${c.toString(16)}`)
    curDigits.push(d)
    blockPos++
    i++

    const needed=5-knownHigh.length
    if(curDigits.length===needed){
      let v
      if(knownHigh.length===0){
        v=0
        for(const d of curDigits)v=v*85+d
      }else{
        let low=0
        for(const d of curDigits)low=low*85+d
        v=reconAfter(knownHigh,low,needed)
        knownHigh=[]
      }
      if(v>0xffffffff)throw new Error("Z85 value overflow")
      out.push((v>>>24)&0xff,(v>>>16)&0xff,(v>>>8)&0xff,v&0xff)
      curDigits.length=0;blockPos=0
    }
  }

  // Handle trailing partial block
  if(curDigits.length>0){
    const n=curDigits.length
    if(n===1)throw new Error("invalid Z85 input length")
    let v=0
    for(const d of curDigits)v=v*85+d
    const nb=n-1
    if(nb===1&&v>0xff)throw new Error("Z85 value overflow")
    if(nb===2&&v>0xffff)throw new Error("Z85 value overflow")
    if(nb===3&&v>0xffffff)throw new Error("Z85 value overflow")
    if(nb===1)out.push(v)
    else if(nb===2)out.push((v>>>8)&0xff,v&0xff)
    else out.push((v>>>16)&0xff,(v>>>8)&0xff,v&0xff)
  }

  return new Uint8Array(out)
}
