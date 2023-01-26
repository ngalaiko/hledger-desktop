import { hledger, hledgerWeb } from "./tauri";
import type { Account, Amount, Commodity, Posting, Transaction } from "./types";
import csv from "csvtojson";

const baseURL = new URL("http://127.0.0.1:5000/");

let stop = (): Promise<void> => Promise.resolve();
let serverFilepath: string | undefined = undefined;

const start = async ({ filepath }: { filepath: string }) => {
    if (filepath !== serverFilepath) {
        serverFilepath = filepath;
        await stop();
        stop = await hledgerWeb(
            "--file",
            filepath,
            "--cors",
            "http://localhost:1420",
            "--serve-api"
        );
        // TODO: is there a better way to wait for server start?
        return new Promise((resolve) => setTimeout(resolve, 500));
    }
};

export const accounts = async ({
    filepath,
}: {
    filepath: string;
}): Promise<Account[]> =>
    start({ filepath }).then(() =>
        fetch(new URL("/accounts", baseURL).toString(), {
            method: "GET",
        })
            .then((resp) => resp.json())
            .then((v: any[]) => v.map((a) => a.aname))
    );

export const commodities = ({
    filepath,
}: {
    filepath: string;
}): Promise<Commodity[]> =>
    hledger("commodities", "--file", filepath).then((out) => out.split("\n"));

const parseAmountString = (raw: string): Amount => {
    if (raw === "0") return { value: 0 };
    const [valueString, ...rest] = raw.split(" ");
    return {
        value: parseFloat(valueString),
        commodity: rest.join(" "),
    };
};

const parsePostingJSON = (v: any): Posting => ({
    txnidx: parseInt(v["txnidx"]),
    date: new Date(v["date"]),
    description: v["description"] as string,
    account: v["account"] as Account,
    amount: parseAmountString(v["amount"]),
    total: (v["total"] as string).split(",").map((amt) => parseAmountString(amt)),
});

export const transactions = async ({
    filepath,
}: {
    filepath: string;
}): Promise<Transaction[]> => {
    const output = await hledger(
        "register",
        "--output-format",
        "csv",
        "--file",
        filepath,
        "--historical"
    );
    const rows = await csv().fromString(output);
    return rows.reduce((acc: Transaction[], row: any) => {
        const posting = parsePostingJSON(row);
        if (acc.length === 0 || acc.at(-1)?.idx !== posting.txnidx) {
            acc.push({
                idx: posting.txnidx,
                date: posting.date,
                description: posting.description,
                postings: [posting],
            });
        } else {
            acc.at(-1)?.postings.push(posting);
        }
        return acc;
    }, [] as Transaction[]);
};
