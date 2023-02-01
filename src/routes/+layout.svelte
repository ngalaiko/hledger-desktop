<script lang="ts">
    import "../app.postcss";

    import { tauri, context } from "$lib";
    import { onMount } from "svelte";
    import { derived, writable } from "svelte/store";
    import { FileSelector } from "$lib/components";

    const selectedFile = writable<string | undefined>(undefined);
    const defaultFile = writable<string | undefined>(undefined);
    const displayFile = derived(
        [defaultFile, selectedFile],
        ([defaultFile, selectedFile]) => selectedFile ?? defaultFile
    );
    context.setFilePath(displayFile);

    onMount(async () => defaultFile.set(await tauri.getFilePath()));
</script>

<div class="h-full flex flex-col">
    <header data-tauri-drag-region class="h-8" />

    <main class="h-full font-mono overflow-y-scroll">
        <header class="flex-1 flex justify-between">
            <input disabled class="flex-1" value={$displayFile} />

            <FileSelector
                on:select={({ detail }) => selectedFile.set(detail.filepath)}
                filters={[
                    { name: "hledger", extensions: ["ledger", "journal"] },
                ]}
            >
                {$displayFile ? "select another" : "select"}
            </FileSelector>
        </header>

        <slot />
    </main>
</div>
