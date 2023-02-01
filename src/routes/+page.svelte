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

    const collapseAccount = (e: HTMLElement) => {
        if (e.textContent === null) return;
        e.dataset["value"] = e.textContent;

        const collapse = (e: HTMLElement) => {
            if (e.textContent === null) return;
            if (e.dataset["value"]) e.textContent = e.dataset["value"];

            while (e.offsetWidth < e.scrollWidth) {
                const parts: string[] = e.textContent.split(":");
                const firstLongPartIndex = parts.findIndex((p) => p.length > 1);
                if (firstLongPartIndex === -1) return;
                e.textContent = [
                    ...parts.slice(0, firstLongPartIndex),
                    parts[firstLongPartIndex][0],
                    ...parts.slice(firstLongPartIndex + 1),
                ].join(":");
            }
        };

        collapse(e);

        const onResize = () => collapse(e);
        window.addEventListener("resize", onResize);
        return {
            destroy: () => window.removeEventListener("resize", onResize),
        };
    };
</script>

{#await $fetchTransactions}
    loading...
{:then _}
    <ul class="flex flex-col gap-4">
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
                    <ul class="ml-[4ch] flex flex-col gap-1">
                        {#each transaction.tpostings as posting, i}
                            {@const amount = Amount.format(posting.pamount[0])}
                            {@const assertion = assertions[i]}
                            <li class="flex gap-[2ch]">
                                <span
                                    use:collapseAccount
                                    title={posting.paccount}
                                    class="flex-1 text-ellipsis overflow-hidden whitespace-pre"
                                >
                                    {posting.paccount}
                                </span>

                                <span class="whitespace-nowrap">
                                    {amount}
                                </span>

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
