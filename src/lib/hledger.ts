import { hledger } from "./tauri";
import type { Account, Transaction } from "./types";

const instance = ({ filepath }: { filepath: string }) =>
    hledger({
        filePath: filepath,
        cors: "http://localhost:1420",
    }).then((url) => {
        console.log(url);
        return new URL(url);
    });

// instances contains a list of running backends
const instances: Record<string, ReturnType<typeof instance>> = {};
const getInstance = (filepath: string) => {
    if (filepath in instances) return instances[filepath];
    instances[filepath] = instance({ filepath });
    return instances[filepath];
};

export const accounts = async ({
    filepath,
}: {
    filepath: string;
}): Promise<Account[]> => {
    const baseUrl = await getInstance(filepath);
    const response = await fetch(new URL("/accounts", baseUrl).toString(), {
        method: "GET",
    });
    return response.json();
};

export const transactions = async ({
    filepath,
}: {
    filepath: string;
}): Promise<Transaction[]> => {
    const baseUrl = await getInstance(filepath);
    const response = await fetch(new URL("/transactions", baseUrl).toString(), {
        method: "GET",
    });
    return response.json();
};
