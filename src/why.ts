export class OrderedFrozenMap<K, V> extends Map<K, V>
  implements Readonly<Map<K, V>> {
  public constructor(
    pairs: Array<[K, V]>,
    ordering: (a: K, b: K) => number = (a, b) => {
      if (a < b) return -1;
      if (b > a) return +1;
      return 0;
    }
  ) {
    pairs.sort(([aKey, _aValue], [bKey, _bValue]) => ordering(aKey, bKey));
    super(pairs);
    Object.freeze(this);
  }

  public static with<K, V>(
    values: Iterable<V>,
    f: (v: V) => K,
    ordering?: (a: K, b: K) => number
  ) {
    return new OrderedFrozenMap(
      [...values].map((value) => [f(value), value]),
      ordering
    );
  }
}
