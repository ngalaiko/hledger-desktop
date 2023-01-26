import { setContext, getContext } from "svelte";
import type { Readable } from "svelte/store";

export const setFilePath = (filepath: Readable<string | undefined>) =>
    setContext("filePath", filepath);

export const getFilePath = (): Readable<string | undefined> =>
    getContext("filePath");
