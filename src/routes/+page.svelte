<script lang="ts">
    import { hledger, context } from "$lib";
    import { AccountsTree } from "$lib/components";

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
        <ul class="flex flex-col gap-4">
            {#each transctions as transaction}
                <li>
                    <figure class="flex flex-col">
                        <figcaption class="flex gap-2 whitespace-nowrap">
                            <time datetime={transaction.date.toISOString()}>
                                {transaction.date.toLocaleDateString()}
                            </time>
                            <span>{transaction.description}</span>
                        </figcaption>
                    </figure>
                    <ul class="ml-4">
                        {#each transaction.postings as posting}
                            <li class="grid grid-cols-2">
                                <span class="whitespace-nowrap"
                                    >{posting.account}</span
                                >
                                <span
                                    class="flex gap-2"
                                    class:text-green-600={posting.amount.value >
                                        0}
                                    class:text-red-600={posting.amount.value <
                                        0}
                                >
                                    <span>{posting.amount.value}</span>
                                    <span>{posting.amount.commodity}</span>
                                </span>
                            </li>
                        {/each}
                    </ul>
                </li>
            {/each}
        </ul>
    {/await}
{/if}
