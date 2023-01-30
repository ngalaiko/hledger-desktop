<script lang="ts">
    import { mostCommon } from "$lib/utils";

    export let name: string = "";
    export let value: string = "";
    export let required: boolean = false;
    let className = "";
    export { className as class };

    let input: HTMLInputElement;

    export let sources: ((value?: string) => string[]) | string[] = () => [];
    export let placeholder: string | undefined = undefined;

    $: suggestion =
        typeof sources === "function"
            ? mostCommon(
                  sources(value).filter((source) =>
                      source.toLowerCase().startsWith(value.toLowerCase())
                  )
              )?.slice(value.length) ?? placeholder
            : mostCommon(
                  sources.filter((source) =>
                      source.toLowerCase().startsWith(value.toLowerCase())
                  )
              )?.slice(value.length) ?? placeholder;

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

<div class="relative {className}">
    <input
        use:keydown={{ Tab: complete }}
        type="text"
        {name}
        bind:value
        placeholder={suggestion}
        class="flex-1 hover:outline-none focus:outline-none"
        {required}
        bind:this={input}
        size={value ? value.length : suggestion?.length ?? 0}
    />
    {#if value && suggestion}
        <button
            type="button"
            on:click={() => input.focus()}
            class="whitespace-pre cursor-text flex-1 absolute top-0"
            style:color="darkgrey"
            style:left="{value.length}ch"
        >
            {suggestion}
        </button>
    {/if}
</div>
