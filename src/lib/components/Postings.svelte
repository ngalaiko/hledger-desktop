<script lang="ts">
    import type { Posting } from "$lib";
    import { Amount } from "$lib/components";

    export let postings: Posting[];

    const width = Math.max(
        ...postings.map(
            (p) => p.pamount[0].aquantity.floatingPoint.toString().length
        )
    );
</script>

<ul class="ml-4">
    {#each postings as posting}
        <li class="flex justify-between">
            <span class="whitespace-nowrap">{posting.paccount}</span>
            <Amount amount={posting.pamount[0]} {width} />
        </li>
    {/each}
</ul>
