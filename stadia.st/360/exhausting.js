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
  let players = window.players = []
  let pids = new Set
  let url
  let flush = () => {
    if (!players.length) return;
    if (url) url = URL.revokeObjectURL(url)
    url = window.URL.createObjectURL(
      new Blob([JSON.stringify(players, null, 2)], { type: "application/json;charset=utf-8" })
    );
    let a = document.createElement('a')
    a.href = url
    a.download = `stadians-${players[0].name.toLowerCase()}-${players[players.length - 1].name.toLowerCase()}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    players.splice(0, players.length)
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
      let players = Object.fromEntries(JSON.parse(JSON.parse(line + ']')?.[0]?.[2])?.[1]?.map(x => x[0])?.map(([[name, number], _1, _2, _3, _4, id]) => [id, {
        name,
        number,
        id,
      }]) || [])
      got(players)
    }))
    return result
  }
  let lett = async(`etaoinsrhldcumfpgwybvkxjqz`)
  let lott = async(`0123456789`)
  let length = 15
  let id = 4
  let pp = [...async(`srhldcumfpgwybvkxj`)].flatMap(l => [...lett, ...lott].map(c => l + c)).reverse()
  let errors = [];
  while (length, pp.length) {
    try {
      await new Promise(r => setTimeout(r, 500 * Math.pow(2, errors.length)))
      let p = pp.pop()
      let qq = `${p.slice(0, 1)} ${p.slice(1)}`
      let e = yield `[jsname=hYL8Ff]`
      e.value = qq;
      e.dispatchEvent(new InputEvent('input', {bubbles: true}))
      let ps = async(await get);
      for (let p of ps) {
        if (!pids.has(p.id)) {
          pids.add(p.id);
          players.push(p);
        }
      }
      document.title = `Discovered ${pids.size} Stadians`
      let c = async({separator: '#'})
      if ((players.length >= 8192 && flush()) || ps.length >= 100) {
        if (p.length < length + id + 1) {
          if (p.length !== length) {
            c = async([...lett, ...async(p.length < length ? lott : {})].reverse())
          }
          c.map(c => pp.push(p + c))
        } else {
          console.error(JSON.stringify(`unreachable`))
        }
      }
    } catch (error) {
      errors.push(error)
      let name = `error_${Date.now()}`
      console.error(`${name} = ${error}\n${error.stack}`)
      window[name] = error
      await new Promise(r => setTimeout(r, 8_000 * Math.pow(2, errors.length - 1)))
    }
  }
  flush()
}, e = async = async(), q) => {
  for (; !(q = (await async.next(e))).done; ) for (;
      void q == (e = document.querySelector(q.value));
      await new Promise(r => setTimeout(r, 250)));
})()
