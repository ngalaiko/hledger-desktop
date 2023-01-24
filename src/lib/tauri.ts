import { invoke } from "@tauri-apps/api/tauri";

export const getFilePath = () => invoke<string | undefined>("get_ledger_file");

export const readFile = (filePath: string) =>
    invoke<string>("read_file", { filePath });

export type Journal = {
    includes: string[];
};

export const parseJournal = (filePath: string) =>
    invoke<Journal>("parse_hledger_file", { filePath });

export const resolveGlobPattern = (pattern: string) =>
    invoke<string[]>("resolve_glob_pattern", { pattern });
