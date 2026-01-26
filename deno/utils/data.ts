const dataExtension = ".json";
const dataUrlBase = new URL("https://sfic.s3.amazonaws.com/");
const dataFileBase = "../data/";
const extraFileBase = "../target/";

const cache = [] as [key: string, value: string][];
const cacheItemCapacity = 128;
const cacheCharacterCapacity = cacheItemCapacity * 16 * 1024;

export const load = async (path: string) => {
  if (!path.endsWith(dataExtension)) {
    path += dataExtension;
  }

  const errors: unknown[] = [];

  const text: string = await Promise.resolve(
    cache.find(([k, _v]) => k === path)?.[1] ?? Promise.reject(undefined),
  ).then((text) => {
    console.debug(`cache hit: ${path}`);
    return text;
  }).catch(async () => {
    const text = await Deno.readTextFile(dataFileBase + path);
    console.debug(`local read: ${path} in ${dataFileBase}`);
    return text;
  }).catch(async (error) => {
    errors.push(error);
    const text = await Deno.readTextFile(extraFileBase + path);
    console.debug(`local read: ${path} in ${extraFileBase}`);
    return text;
  }).catch(async (error) => {
    errors.push(error);
    const url = new URL(path, dataUrlBase);
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status} ${response.statusText}`);
    }
    const text = await response.text();
    console.debug(`remote fetch: ${path} from ${url}`);
    return text;
  }).catch((error) => {
    errors.push(error);
    throw new Error(
      `Unable to load(${JSON.stringify(path)})\n${errors.join("\n")}`,
    );
  });

  if (text.length > cacheCharacterCapacity) {
    console.warn(
      `A single response value is larger (${text.length} chars) than our \
      total response cache capacity (${cacheCharacterCapacity} characters)? That's iffy.`,
    );
  } else if (cache.at(-1)?.[0] !== path) {
    cache.push([path, text]);
  }

  // This is not very efficient, but the limit of `cacheItemCapacity`
  // is small enough that it shouldn't be too bad in practice.
  while (
    cache.length > cacheItemCapacity ||
    [...new Set(cache.flat())].reduce((a, b) => a + b.length, 0) >
      cacheCharacterCapacity
  ) {
    cache.shift();
  }

  return JSON.parse(text);
};
