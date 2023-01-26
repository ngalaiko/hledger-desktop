<script lang="ts">
    import { tauri } from "$lib";
    import { onMount } from "svelte";
    import { derived, writable } from "svelte/store";
    import { FileSelector } from "$lib/components";

    const selectedFile = writable<string | undefined>(undefined);
    const defaultFile = writable<string | undefined>(undefined);
    const displayFile = derived(
        [defaultFile, selectedFile],
        ([defaultFile, selectedFile]) => selectedFile ?? defaultFile
    );

    onMount(async () => defaultFile.set(await tauri.getFilePath()));
</script>

<figure class="flex flex-col gap-2">
    <figcaption class="flex flex-row justify-between">
        <span class="flex-1">{$displayFile}</span>
        <FileSelector
            on:select={({ detail }) => selectedFile.set(detail.filepath)}
            filters={[{ name: "hledger", extensions: ["ledger", "journal"] }]}
        >
            {$displayFile ? "select another" : "select"}
        </FileSelector>
    </figcaption>

    <slot filepath={$displayFile} />
</figure>
