<script lang="ts">
    import { mostCommon } from "$lib/utils";

    export let name: string = "";
    export let value: string = "";
    export let required: boolean = false;

    export let sources: string[] = [];
    $: suggestion = mostCommon(
        sources.filter((source) => source.startsWith(value))
    )?.slice(value.length);
</script>

<div class="flex flex-1">
    <input
        type="text"
        {name}
        bind:value
        size={value.length}
        placeholder={suggestion}
        class="hover:outline-none focus:outline-none"
        class:flex-1={!value}
        {required}
    />
    {#if value && suggestion}
        <span class="flex-1" style:color="darkgrey">{suggestion}</span>
    {/if}
</div>
