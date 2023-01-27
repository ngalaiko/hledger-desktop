<script lang="ts">
    import VirtualList from "@sveltejs/svelte-virtual-list";
    import { hledger, context, type Account, type Transaction } from "$lib";
    import { AccountsTree, Postings } from "$lib/components";
    import { derived, writable } from "svelte/store";

    const filePath = context.getFilePath();

    const accounts = writable<Account[]>([]);
    const transactions = writable<Transaction[]>([]);

    const fetchAccounts = derived(filePath, (filepath) =>
        filepath
            ? hledger.accounts({ filepath }).then(accounts.set)
            : new Promise(() => {})
    );

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

    const onAccountSelected = (account: string) =>
        transactionFilters.set([
            (tx: Transaction) =>
                tx.tpostings.some((posting) =>
                    posting.paccount.startsWith(account)
                ),
        ]);
</script>

<div class="flex flex-row gap-2 h-full">
    <div class="w-1/4">
        {#await $fetchAccounts}
            loading...
        {:then _}
            <AccountsTree
                on:select={({ detail }) => onAccountSelected(detail.account)}
                accounts={$accounts}
            />
        {/await}
    </div>

    <div class="w-full">
        {#await $fetchTransactions}
            loading...
        {:then _}
            <VirtualList items={$displayTransactions} let:item={transaction}>
                <figure class="flex flex-col p-2">
                    <figcaption class="flex gap-2 whitespace-nowrap">
                        <time
                            datetime={new Date(transaction.tdate).toISOString()}
                        >
                            {transaction.tdate}
                        </time>
                        <span>{transaction.tdescription}</span>
                    </figcaption>
                    <Postings postings={transaction.tpostings} />
                </figure>
            </VirtualList>
        {/await}
    </div>
</div>
