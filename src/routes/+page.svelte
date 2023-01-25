<script lang="ts">
    import { tauri } from "$lib";
    import { dirname, join } from "@tauri-apps/api/path";
    import { FileSelector } from "$lib/components";
</script>

{#await tauri.getFilePath() then filepath}
    <FileSelector
        value={filepath}
        filters={[{ name: "hledger", extensions: ["ledger", "journal"] }]}
    />

    {#if filepath !== undefined}
        {#await tauri.parseJournal(filepath) then journal}
            <ul>
                <li>
                    <figure>
                        <figcaption>{filepath}</figcaption>
                        <code>{JSON.stringify(journal)}</code>
                    </figure>
                </li>
                {#each journal.includes as include}
                    {#await dirname(filepath)
                        .then((dir) => join(dir, include))
                        .then(tauri.resolveGlobPattern) then filepaths}
                        {#each filepaths as filepath}
                            <li>
                                <figure>
                                    <figcaption>{filepath}</figcaption>
                                    {#await tauri.parseJournal(filepath) then journal}
                                        <code>{JSON.stringify(journal)}</code>
                                    {:catch error}
                                        <div>
                                            error: <span class="text-red-600"
                                                >{error}</span
                                            >
                                        </div>
                                    {/await}
                                </figure>
                            </li>
                        {/each}
                    {/await}
                {/each}
            </ul>
        {:catch error}
            <div>error: <span class="text-red-600">{error}</span></div>
        {/await}
    {/if}
{/await}
