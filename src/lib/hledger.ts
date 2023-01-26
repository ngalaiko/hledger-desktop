import { hledger } from "./tauri";

export type Account = string;

const SEPARATOR = ":";

export namespace Account {
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

export const accounts = (filename: string): Promise<Account[]> =>
    hledger("-f", filename, "accounts").then((out) => out.split("\n"));

export const exec = (...args: string[]) => hledger(...args);
