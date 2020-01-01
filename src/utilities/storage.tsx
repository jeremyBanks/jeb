/**
 * An asynchronous key-value store of strings.
 */
export interface StringStorage {
  /** Returns a boolean indicating whether there is a value with the specified key. */
  has(key: string): Promise<boolean>;
  /** Returns the value stored under the specified key, or undefined. */
  get(key: string): Promise<string | undefined>;
  /** Sets the value stored under a specified key, returning the previous value or undefined. */
  set(key: string, value: string): Promise<string | undefined>;
  /** Deletes the value stored under a specified key there is any, returning it or undefined. */
  delete(key: string): Promise<string | undefined>;
}

class SyncStorage implements StringStorage {
  readonly storage: Storage;

  constructor(storage: Storage) {
    this.storage = storage;
  }

  async has(key: string) {
    return this.storage.getItem(key) !== null;
  }

  async get(key: string) {
    return this.storage.getItem(key) ?? undefined;
  }

  async set(key: string, value: string) {
    const former = this.get(key);
    this.storage.setItem(key, value);
    return former;
  }

  async delete(key: string) {
    const former = this.get(key);
    this.storage.removeItem(key);
    return former;
  }
}

class RemoteStorage implements RemoteStorage {
  readonly privateKey: string;
  readonly publicKey: string;

  async has(_key: string) {
    return false;
  }

  async get(_key: string) {
    return undefined;
  }

  async set(_key: string, _value: string): Promise<string> {
    throw new Error("not implemented");
  }

  async delete() {
    return undefined;
  }
}

export const session: StringStorage = new SyncStorage(window.sessionStorage);
export const local: StringStorage = new SyncStorage(window.localStorage);
export const remote: RemoteStorage = new RemoteStorage();
