import { invoke } from "@tauri-apps/api/tauri";
import { Journal } from "./types";

export const getFilePath = () => invoke<string | undefined>("get_ledger_file");

export const readFile = (filePath: string) =>
    invoke<string>("read_file", { filePath });

export const parseJournal = (filePath: string) =>
    invoke<Journal>("parse_hledger_file", { filePath })
        .then((raw) => {
            console.log(raw);
            return raw;
        })
        .then(Journal.fromJSON);

export const resolveGlobPattern = (pattern: string) =>
    invoke<string[]>("resolve_glob_pattern", { pattern });
