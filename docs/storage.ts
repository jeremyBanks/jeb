declare const chrome: {
  storage: {
    local: any;
    sync: any;
    onChanged: any;
  };
  runtime: {
    lastError: any;
  };
};

const pCall = (
  namespace: any,
  method: string,
  ...args: Array<unknown>
): Promise<any> =>
  new Promise((resolve, reject) => {
    namespace[method](...args, (result: any) => {
      if (chrome.runtime.lastError) {
        reject(chrome.runtime.lastError);
      } else {
        resolve(result);
      }
    });
  });

class Storage {
  async getMany(keys: Array<string>): Promise<Record<string, any>> {
    return pCall(chrome.storage.local, "get", keys);
  }
  async get(key: string): Promise<any> {
    return (await this.getMany([key]))[key];
  }
  async set(key: string, value: any): Promise<unknown> {
    return this.setMany({ [key]: value });
  }
  async setMany(pairs: Record<string, any>): Promise<unknown> {
    return pCall(chrome.storage.local, "set", pairs);
  }
  async removeMany(keys: Array<string>): Promise<unknown> {
    return pCall(chrome.storage.local, "remove", keys);
  }
  async remove(key: string): Promise<unknown> {
    return this.removeMany([key]);
  }
  async usage(keys: Array<string> | null = null): Promise<number> {
    return pCall(chrome.storage.local, "getBytesInUse", keys);
  }
  async clear(): Promise<unknown> {
    return pCall(chrome.storage.local, "clear");
  }
  onChange(
    callback: (
      changes: Record<string, { newValue?: string; oldValue?: string }>
    ) => unknown
  ): void {
    chrome.storage.local.onChanged.addListener(callback);
  }
}

export type { Storage };
export const storage = new Storage();
