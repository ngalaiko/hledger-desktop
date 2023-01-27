import { hledgerWeb } from "./tauri";
import type { Account } from "./types";

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
        }).then((resp) => resp.json())
    );
