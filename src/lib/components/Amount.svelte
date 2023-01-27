<script lang="ts">
    import type { Amount } from "$lib";

    export let amount: Amount;
</script>

<div
    class="flex flex-row font-mono"
    class:gap-1={amount.astyle.ascommodityspaced}
>
    {#if amount.astyle.ascommodityside === "R"}
        <span>{amount.aquantity.floatingPoint}</span>
        <span>{amount.acommodity}</span>
    {:else}
        <span>{amount.acommodity}</span>
        <span>{amount.aquantity.floatingPoint}</span>
    {/if}
</div>
