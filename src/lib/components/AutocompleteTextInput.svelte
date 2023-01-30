<script lang="ts">
    import { mostCommon } from "$lib/utils";

    export let name: string = "";
    export let value: string = "";
    export let required: boolean = false;

    let input: HTMLInputElement;

    export let sources: string[] = [];

    $: suggestion = mostCommon(
        sources.filter((source) => source.startsWith(value))
    )?.slice(value.length);

    const keydown = (
        node: HTMLElement,
        map: Record<string, (e: KeyboardEvent) => void>
    ) => {
        const listener = (e: KeyboardEvent) => {
            if (map[e.key]) map[e.key](e);
        };
        node.addEventListener("keydown", listener);
        return {
            destroy: () => {
                node.removeEventListener("keydown", listener);
            },
        };
    };

    const complete = (e: KeyboardEvent) => {
        if (suggestion) {
            e.preventDefault();
            value += suggestion;
        }
    };
</script>

<div class="flex flex-1 relative">
    <input
        use:keydown={{ Tab: complete }}
        type="text"
        {name}
        bind:value
        placeholder={suggestion}
        class="flex-1 hover:outline-none focus:outline-none"
        {required}
        bind:this={input}
    />
    {#if value && suggestion}
        <button
            type="button"
            on:click={() => input.focus()}
            class="cursor-text flex-1 absolute top-0"
            style:color="darkgrey"
            style:left="{value.length}ch"
        >
            {suggestion}
        </button>
    {/if}
</div>
