// Returns a copy of a record with string keys sorted.
export const sorted = <T extends Record<string, any>>(record: T): T =>
  Object.fromEntries(
    Object.keys(record)
      .sort()
      .map((key) => [key, record[key]])
  );
