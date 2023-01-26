import { invoke } from "@tauri-apps/api/tauri";
import { Command } from "@tauri-apps/api/shell";
import { differenceInMilliseconds } from "date-fns";

class ExecError extends Error {
    code: number;

    constructor(code: number, message: string) {
        super(message);
        this.code = code;
        this.message = message;
    }
}

export const hledger = async (...args: string[]) => {
    const start = new Date();
    const { code, stdout, stderr } = await Command.sidecar(
        "binaries/hledger",
        args
    ).execute();

    console.debug(
        `exec "hledger ${args.join(" ")}" took ${differenceInMilliseconds(
            new Date(),
            start
        )}ms`
    );

    if (code === null) {
        throw new Error("process was terminated");
    } else if (code !== 0) {
        throw new ExecError(code, stderr);
    } else {
        return stdout;
    }
};
export const getFilePath = () => invoke<string | undefined>("get_ledger_file");

export const readFile = (filePath: string) =>
    invoke<string>("read_file", { filePath });

export const resolveGlobPattern = (pattern: string) =>
    invoke<string[]>("resolve_glob_pattern", { pattern });
