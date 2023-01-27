<script lang="ts">
    import { hledger, context } from "$lib";
    import { AccountsTree, Amount } from "$lib/components";
    import VirtualList from "@sveltejs/svelte-virtual-list";

    const filePath = context.getFilePath();
</script>

{#if $filePath !== undefined}
    {#await hledger.accounts({ filepath: $filePath })}
        loading...
    {:then accounts}
        <AccountsTree {accounts} />
    {/await}

    {#await hledger.transactions({ filepath: $filePath })}
        loading...
    {:then transctions}
        <VirtualList height="500px" items={transctions} let:item={transaction}>
            <figure class="flex flex-col">
                <figcaption class="flex gap-2 whitespace-nowrap">
                    <time datetime={new Date(transaction.tdate).toISOString()}>
                        {transaction.tdate}
                    </time>
                    <span>{transaction.tdescription}</span>
                </figcaption>
            </figure>
            <ul class="ml-4">
                {#each transaction.tpostings as posting}
                    <li class="grid grid-cols-2">
                        <span class="whitespace-nowrap">{posting.paccount}</span
                        >
                        <Amount amount={posting.pamount[0]} />
                    </li>
                {/each}
            </ul>
        </VirtualList>
    {/await}
{/if}
