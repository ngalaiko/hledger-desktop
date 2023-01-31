<script lang="ts">
    import { hledger, context, type Transaction } from "$lib";
    import { Amount } from "$lib/types";
    import TransactionForm from "$lib/components/TransactionForm.svelte";
    import { derived, writable } from "svelte/store";

    const filePath = context.getFilePath();

    const transactions = writable<Transaction[]>([]);

    const fetchTransactions = derived(filePath, (filepath) =>
        filepath
            ? hledger.transactions({ filepath }).then(transactions.set)
            : new Promise(() => {})
    );

    const transactionFilters = writable<((tx: Transaction) => boolean)[]>([]);

    const displayTransactions = derived(
        [transactions, transactionFilters],
        ([transactions, transactionFilters]) =>
            transactions
                .filter((tx) =>
                    transactionFilters.every((filter) => filter(tx))
                )
                .sort((a, b) => b.tindex - a.tindex)
    );
</script>

{#await $fetchTransactions}
    loading...
{:then _}
    <ul class="flex flex-col gap-2">
        <li class="flex flex-1">
            <TransactionForm transactions={displayTransactions} />
        </li>
        {#each $displayTransactions.slice(0, 100) as transaction}
            {@const assertions = transaction.tpostings.map((p) =>
                p.pbalanceassertion
                    ? Amount.format(p.pbalanceassertion.baamount)
                    : ""
            )}
            {@const maxAssetionLength = Math.max(
                ...assertions.map((a) => a.length)
            )}
            <li class="flex flex-1">
                <figure class="flex flex-col flex-1 max-w-full">
                    <figcaption class="flex gap-2 whitespace-nowrap">
                        <time class="font-mono" datetime={transaction.tdate}>
                            {transaction.tdate}
                        </time>
                        <span>{transaction.tdescription}</span>
                    </figcaption>
                    <ul class="flex flex-col gap-2">
                        {#each transaction.tpostings as posting, i}
                            {@const amount =
                                posting.pamount.length > 0
                                    ? Amount.format(posting.pamount[0])
                                    : null}
                            {@const assertion = assertions[i]}
                            <li class="flex gap-[2ch]">
                                <span
                                    title={posting.paccount}
                                    class="flex-1 text-ellipsis overflow-hidden whitespace-pre"
                                >
                                    {posting.paccount}
                                </span>

                                {#if amount}
                                    <span class="whitespace-nowrap">
                                        {amount}
                                    </span>
                                {/if}

                                {#if maxAssetionLength > 0}
                                    <span
                                        style:width="{maxAssetionLength + 2}ch"
                                        class="whitespace-nowrap"
                                    >
                                        {#if assertion}= {assertion}{/if}
                                    </span>
                                {/if}
                            </li>
                        {/each}
                    </ul>
                </figure>
            </li>
        {/each}
    </ul>
{/await}
