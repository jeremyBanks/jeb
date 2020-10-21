(async (async = async function* async() {
  let letters = "etaoinsrhldcumfpgwybvkxjqz";
  let digits = "0123456789"
  let nameMaxLength = 15;
  let numberDigits = 4;

  let array = object => {
    let values = Object.values(object);
    let keys = Object.freeze(Object.keys(object));
    object = Object.freeze(Object.assign(Object.create(values), object));
    Object.freeze(Object.assign(values, {keys}));
    return object;
  }

  let prefixPool = [...letters];
  while (prefixPool.length > 0) {
    let prefix = prefixPool.pop();

    let query = `${prefix.slice(0, 1)} ${prefix.slice(1)}`;
    let results = array({
      "123": "Joe#1234",
    });

    if (results.length >= 100) {
      if (prefix.length < nameMaxLength) {
        for (const letter of [...letters]) {
          prefixPool.push(`${prefix}${letter}`);
        }
        for (const digit of [...digits]) {
          prefixPool.push(`${prefix}${digit}`);
        }
      } else if (prefix.length === nameMaxLength) {
        prefixPool.push(`${prefix}#`)
      } else if (prefix.length < nameMaxLength + numberDigits + 1) {
        for (const digit of [...digits]) {
          prefixPool.push(`${prefix}${digit}`);
        }
      } else {
        console.error("sanity failure: 100 results for a unique id");
      }
    }
  }

  console.log(await (yield async () => 2));
  console.log(yield () => 2 + 10);
}, next = async = async()) => {
  while (next = (await async.next(next)).value?.());
})();
