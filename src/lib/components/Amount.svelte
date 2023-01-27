<script lang="ts">
    import type { Amount } from "$lib";

    export let amount: Amount;
    export let width = -1;

    const prefixLen =
        width > 0
            ? width - amount.aquantity.floatingPoint.toString().length
            : 0;
</script>

<div
    class="flex flex-row font-mono"
    class:gap-1={amount.astyle.ascommodityspaced}
>
    {#if amount.astyle.ascommodityside === "R"}
        <span style:margin-left={`${prefixLen}ch`}
            >{amount.aquantity.floatingPoint}</span
        >
        {#if amount.acommodity.split(" ").length > 1}
            <span>"{amount.acommodity}"</span>
        {:else}
            <span>{amount.acommodity}</span>
        {/if}
    {:else}
        <span>{amount.acommodity}</span>
        {#if amount.acommodity.split(" ").length > 1}
            <span>"{amount.acommodity}"</span>
        {:else}
            <span>{amount.acommodity}</span>
        {/if}
    {/if}
</div>
