<script lang="ts">
    import { open, type OpenDialogOptions } from "@tauri-apps/api/dialog";
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher<{ select: { filepath: string } }>();

    export let filters: OpenDialogOptions["filters"] = [];

    const onButtonClick = () =>
        open({
            filters,
        }).then((selected) => {
            if (!Array.isArray(selected) && selected !== null) {
                dispatch("select", { filepath: selected });
            }
        });
</script>

<button
    type="button"
    class="shadow-md py-1 px-2 rounded-md transition hover:scale-105"
    on:click={onButtonClick}
>
    <slot>select</slot>
</button>
