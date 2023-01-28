<script lang="ts">
    import type { Transaction } from "$lib/types";
    import { derived, type Readable } from "svelte/store";
    import { mostCommon } from "$lib";
    import { format } from "date-fns";
    import AutocompleteTextInput from "./AutocompleteTextInput.svelte";

    export let transactions: Readable<Transaction[]>;

    const accounts = derived(transactions, (transactions) =>
        transactions.flatMap((tx) =>
            tx.tpostings.slice(0, 1).map((p) => p.paccount)
        )
    );

    const mostCommonFirstAccount = derived(transactions, (transactions) => {
        if (transactions.length === 0) return "account";
        const accounts = transactions.flatMap((tx) =>
            tx.tpostings.slice(0, 1).map((p) => p.paccount)
        );
        return mostCommon(accounts);
    });

    const mostCommonCommodity = derived(
        [transactions, mostCommonFirstAccount],
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
            placeholder="Description..."
            class="flex-1 hover:outline-none focus:outline-none"
            required
        />
    </fieldgroup>
    <ul class="ml-4">
        <li class="flex justify-between text-ellipsis overflow-hidden">
            <AutocompleteTextInput
                name="posting[]"
                sources={$accounts}
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
