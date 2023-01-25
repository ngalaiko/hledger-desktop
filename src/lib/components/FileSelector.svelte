<script lang="ts">
    import { open, type OpenDialogOptions } from "@tauri-apps/api/dialog";

    export let placeholder: string = "select file...";
    export let value: string | undefined = undefined;
    export let filters: OpenDialogOptions["filters"] = [];

    const selectFile = () =>
        open({
            filters,
        }).then((selected) => {
            if (!Array.isArray(selected) && selected !== null) {
                value = selected;
            }
        });
</script>

<form
    on:submit={selectFile}
    class="flex flex-row gap-2 border justify-between p-2"
>
    <input class="flex-1" type="text" bind:value {placeholder} disabled />
    <button
        type="submit"
        class="shadow-md py-1 px-2 rounded-md transition hover:scale-105"
    >
        {value ? "select another" : "select"}
    </button>
</form>
