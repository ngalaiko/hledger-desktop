import { invoke } from "@tauri-apps/api/tauri";

export const hledger = async (params: { filePath: string; cors: string }) =>
    invoke<string>("hledger_web", params);

export const getFilePath = () => invoke<string | undefined>("get_ledger_file");

export const readFile = (filePath: string) =>
    invoke<string>("read_file", { filePath });

export const resolveGlobPattern = (pattern: string) =>
    invoke<string[]>("resolve_glob_pattern", { pattern });
