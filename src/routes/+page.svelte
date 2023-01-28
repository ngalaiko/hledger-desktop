<script lang="ts">
    import { hledger, context, type Transaction, type Posting } from "$lib";
    import { Amount } from "$lib/components";
    import { derived, writable } from "svelte/store";
    import { format } from "date-fns";

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
                .slice(0, 100)
    );

    const amountWidth = (postings: Posting[]) =>
        Math.max(
            ...postings.map(
                (p) => p.pamount[0].aquantity.floatingPoint.toString().length
            )
        );

    const mostCommon = <T extends string>(array: T[]) => {
        const hashmap = array.reduce((acc, val) => {
            acc[val] = (acc[val] || 0) + 1;
            return acc;
        }, {} as Record<T, number>);
        return (Object.keys(hashmap) as T[]).reduce((a, b) =>
            hashmap[a] > hashmap[b] ? a : b
        );
    };

    const mostCommonFirstAccount = derived(
        displayTransactions,
        (transactions) => {
            if (transactions.length === 0) return "account";
            const accounts = transactions.flatMap((tx) =>
                tx.tpostings.slice(0, 1).map((p) => p.paccount)
            );
            return mostCommon(accounts);
        }
    );

    const mostCommonCommodity = derived(
        [displayTransactions, mostCommonFirstAccount],
        ([transactions, account]) => {
            if (transactions.length === 0) return undefined;
            const commodities = transactions
                .filter((tx) =>
                    tx.tpostings.some((p) => p.paccount === account)
                )
                .flatMap((tx) =>
                    tx.tpostings.slice(0, 1).map((p) => p.pamount[0].acommodity)
                );
            return mostCommon(commodities);
        }
    );
</script>

{#await $fetchTransactions}
    loading...
{:then _}
    <ul class="flex flex-col gap-2">
        <li class="flex flex-1">
            <form class="flex flex-1 flex-col">
                <fieldgroup class="flex gap-2 whitespace-nowrap">
                    <input
                        name="date"
                        type="text"
                        placeholder={format(new Date(), "yyyy-MM-dd")}
                        class="w-[10ch] font-mono hover:outline-none focus:outline-none"
                        required
                    />
                    <input
                        name="description"
                        type="text"
                        class="flex-1 hover:outline-none focus:outline-none"
                        required
                    />
                </fieldgroup>
                <ul class="ml-4">
                    <li
                        class="flex justify-between text-ellipsis overflow-hidden"
                    >
                        <input
                            name="posting[]"
                            type="text"
                            placeholder={$mostCommonFirstAccount}
                            class="flex-1 hover:outline-none focus:outline-none"
                            required
                        />
                        <input
                            name="amount[]"
                            type="text"
                            placeholder="13 {$mostCommonCommodity}"
                            class="text-right hover:outline-none focus:outline-none"
                            required
                        />
                    </li>
                </ul>
            </form>
        </li>
        {#each $displayTransactions as transaction}
            <li class="flex flex-1">
                <figure class="flex flex-col flex-1">
                    <figcaption class="flex gap-2 whitespace-nowrap">
                        <time class="font-mono" datetime={transaction.tdate}>
                            {transaction.tdate}
                        </time>
                        <span>{transaction.tdescription}</span>
                    </figcaption>
                    <ul class="ml-4">
                        {#each transaction.tpostings as posting}
                            <li
                                class="flex justify-between text-ellipsis overflow-hidden"
                            >
                                <span
                                    title={posting.paccount}
                                    class=" text-ellipsis overflow-hidden"
                                    >{posting.paccount}</span
                                >
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
