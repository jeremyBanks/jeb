(async (async = async function* async() {
  if (document.location.href !== `https://stadia.google.com/home`)
    document.location.assign(`https://stadia.google.com/home`)
  void (yield `[jsname=KiHgxb]`).click()
  void (yield `[jsname=fGwvm] [jscontroller=dIamPb]`).click()
  let async = object => {
    let next = Object.values(object)
    let async = Object.freeze(Object.assign(Object.create(next), object))
    Object.freeze(Object.assign(next, {keys: Object.freeze(Object.keys(object))}))
    return async
  }
  let put, get = new Promise(resolve => { put = resolve }), got = results => {
    put(results)
    get = new Promise(resolve => { put = resolve })
  }
  let users = []
  let url
  let flush = () => {
    if (!users.length) return;
    if (url) url = URL.revokeObjectURL(url)
    url = window.URL.createObjectURL(
      new Blob([JSON.stringify(users, null, 2)], { type: "application/json;charset=utf-8" })
    );
    let a = document.createElement('a')
    a.href = url
    a.download = `stadians-${users[0].userName.toLowerCase()}-${users[users.length - 1].userName.toLowerCase()}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    users = []
  }
  XMLHttpRequest.prototype._send ??= XMLHttpRequest.prototype.send
  XMLHttpRequest.prototype.send = function() {
    let result = this._send(...arguments)
    this.addEventListener('readystatechange', (async () => {
      if (this.readyState !== XMLHttpRequest.DONE) return
      if ('FdyJ0' !== (new URL(this.responseURL)).searchParams.get('rpcids')) return
      let lines = this.responseText.split(/\n/g)
      let length = Math.max(...lines.map(s => s.length))
      let line = lines.filter(s => s.length === length)[0]
      let users = Object.fromEntries(JSON.parse(JSON.parse(line + ']')?.[0]?.[2])?.[1]?.map(x => x[0])?.map(([[userName, userNumber], _1, _2, _3, _4, userId]) => [userId, {
        type: 'user',
        userName,
        userNumber,
        userId,
      }]) || [])
      got(users)
    }))
    return result
  }
  let lett = async(`etaoinsrhldcumfpgwybvkxjqz`)
  let lott = async(`0123456789`)
  let length = 15
  let id = 4
  let pp = [...lett.flatMap(l => [...lett, ...lott].map(c => l + c))].reverse()
  while (length, pp.length) {
    await new Promise(r => setTimeout(r, 12_000))
    let p = pp.pop()
    let qq = `${p.slice(0, 1)} ${p.slice(1)}`
    let e = yield `[jsname=hYL8Ff]`
    e.value = qq;
    e.dispatchEvent(new InputEvent('input', {bubbles: true}))
    let ps = async(await get);
    users.push(...ps)
    let c = async({separator: '#'})
    if ((users.length >= 8192 && flush()) || ps.length >= 100) {
      if (p.length < length + id + 1) {
        if (p.length !== length) {
          c = async([...lett, ...async(p.length < length ? lott : {})].reverse())
        }
        c.map(c => pp.push(p + c))
      } else {
        console.error(JSON.stringify(`unreachable`))
      }
    }
  }
  flush()
}, e = async = async(), q) => {
  for (; !(q = (await async.next(e))).done; ) for (;
      void q == (e = document.querySelector(q.value));
      await new Promise(r => setTimeout(r, 250)));
})()
