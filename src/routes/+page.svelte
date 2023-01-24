<script lang="ts">
    import { tauri } from "$lib";

    const readHledgerFile = async () => {
        const filePath = await tauri.getFilePath();
        if (filePath === undefined) throw Error("filepath not set");
        return tauri.parseLedgerFile(filePath).catch((err) => {
            throw Error(`filed to parse ${filePath}: ${err}`);
        });
    };
</script>

{#await readHledgerFile()}
    loading...
{:then contents}
    {JSON.stringify(contents)}
{:catch error}
    <div>error: <span class="text-red-600">{error}</span></div>
{/await}
