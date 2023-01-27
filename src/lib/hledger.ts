import { hledgerWeb } from "./tauri";
import type { Account, Transaction } from "./types";

const baseURL = new URL("http://127.0.0.1:5000/");

const instance = ({ filepath }: { filepath: string }) => ({
    isReady: new Promise<void>((resolve) =>
        hledgerWeb(
            "--file",
            filepath,
            "--cors",
            "http://localhost:1420",
            "--serve-api"
        )
            .then(() => new Promise((resolve) => setTimeout(resolve, 500)))
            .then(() => resolve())
    ),
});

const instances: Record<string, ReturnType<typeof instance>> = {};
const getInstance = (filepath: string) => {
    // TODO: store a reference somewhere else to survive window reloads
    if (filepath in instances) {
        return instances[filepath];
    }
    instances[filepath] = instance({ filepath });
    return instances[filepath];
};

export const accounts = async ({
    filepath,
}: {
    filepath: string;
}): Promise<Account[]> =>
    getInstance(filepath).isReady.then(() =>
        fetch(new URL("/accounts", baseURL).toString(), {
            method: "GET",
        }).then((resp) => resp.json())
    );

export const transactions = async ({
    filepath,
}: {
    filepath: string;
}): Promise<Transaction[]> => {
    await getInstance(filepath).isReady;
    const response = await fetch(new URL("/transactions", baseURL).toString(), {
        method: "GET",
    });
    const transactions = await response.json();
    return transactions;
};
