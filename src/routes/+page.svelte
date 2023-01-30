<script lang="ts">
    import { hledger, context, type Transaction, type Posting } from "$lib";
    import { Amount } from "$lib/components";
    import { Amount as AmountType } from "$lib/types";
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

    const amountWidth = (postings: Posting[]) =>
        Math.max(
            ...postings.map((p) => AmountType.format(p.pamount[0]).length)
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
            <li class="flex flex-1">
                <figure class="flex flex-col flex-1">
                    <figcaption class="flex gap-2 whitespace-nowrap">
                        <time class="font-mono" datetime={transaction.tdate}>
                            {transaction.tdate}
                        </time>
                        <span>{transaction.tdescription}</span>
                    </figcaption>
                    <ul class="flex-flex-col ml-4">
                        {#each transaction.tpostings as posting}
                            <li class="flex justify-between">
                                <span
                                    title={posting.paccount}
                                    class="text-ellipsis overflow-hidden"
                                >
                                    {posting.paccount}
                                </span>
                                <Amount
                                    amount={posting.pamount[0]}
                                    width={amountWidth(transaction.tpostings)}
                                />
                            </li>
                        {/each}
                    </ul>
                </figure>
            </li>
        {/each}
    </ul>
{/await}
