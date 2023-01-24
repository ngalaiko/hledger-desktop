import { invoke } from "@tauri-apps/api/tauri";

export const getFilePath = () => invoke<string | undefined>("get_ledger_file");

export const readFile = (filePath: string) =>
    invoke<string>("read_file", { filePath });

export const parseLedgerFile = (filePath: string) =>
    invoke<any>("parse_hledger_file", { filePath });
