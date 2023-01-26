export type Account = string;

export namespace Account {
  const SEPARATOR = ":";

  export const join = (...parts: string[]): Account => parts.join(SEPARATOR);

  export const split = (account: Account): string[] => account.split(SEPARATOR);

  export const basename = (account: Account): string => {
    const lastSeparator = account.lastIndexOf(SEPARATOR);
    if (lastSeparator === -1) return account;
    return account.slice(lastSeparator + 1);
  };

  export const parent = (account: Account): Account | undefined => {
    const lastSeparator = account.lastIndexOf(SEPARATOR);
    if (lastSeparator === -1) return account;
    return account.slice(0, lastSeparator);
  };
}

export type Commodity = string;

export type Amount = {
  value: number;
  commodity?: Commodity;
};

export type Posting = {
  txnidx: number;
  date: Date;
  description: string;
  account: Account;
  amount: Amount;
  total: Amount[];
};

export type Transaction = {
  idx: number;
  date: Date;
  description: string;
  postings: Posting[];
};
