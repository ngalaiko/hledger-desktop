import { invoke } from "@tauri-apps/api/tauri";
import { Command } from "@tauri-apps/api/shell";

class ExecError extends Error {
    code: number;

    constructor(code: number, message: string) {
        super(message);
        this.code = code;
        this.message = message;
    }
}

export const hledger = (...args: string[]) =>
    Command.sidecar("binaries/hledger", args)
        .execute()
        .then(({ code, stdout, stderr }) => {
            if (code === null) {
                throw new Error("process was terminated");
            } else if (code !== 0) {
                throw new ExecError(code, stderr);
            } else {
                return stdout;
            }
        });

export const getFilePath = () => invoke<string | undefined>("get_ledger_file");

export const readFile = (filePath: string) =>
    invoke<string>("read_file", { filePath });

export const resolveGlobPattern = (pattern: string) =>
    invoke<string[]>("resolve_glob_pattern", { pattern });
