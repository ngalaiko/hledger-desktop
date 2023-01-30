<script lang="ts">
    import { Amount, type Transaction } from "$lib/types";
    import { derived, type Readable } from "svelte/store";
    import AutocompleteTextInput from "./AutocompleteTextInput.svelte";
    import { mostCommon } from "$lib";

    export let transactions: Readable<Transaction[]>;

    const mostCommonExpenseAccount = derived(transactions, (transactions) => {
        if (transactions.length === 0) return undefined;
        const accounts = transactions.flatMap((tx) =>
            tx.tpostings
                .filter((p) => p.pamount[0].aquantity.floatingPoint > 0)
                .map((p) => p.paccount)
        );
        return mostCommon(accounts);
    });

    const mostCommonAmount = derived(
        [transactions, mostCommonExpenseAccount],
        ([transactions, account]) => {
            if (transactions.length === 0) return undefined;
            const commodities = transactions.flatMap((tx) =>
                tx.tpostings
                    .filter((p) => p.paccount === account)
                    .map((p) => p.pamount[0])
            );
            return mostCommon(commodities);
        }
    );

    const suggestPostingAccount = () => $accounts;
    const suggestPostingAmount = () =>
        $mostCommonAmount ? [`${Amount.format($mostCommonAmount)}`] : [];

    const accounts = derived(transactions, (transactions) =>
        transactions.flatMap((tx) =>
            tx.tpostings.slice(0, 1).map((p) => p.paccount)
        )
    );

    const onSubmit = (e: SubmitEvent) => {
        e.preventDefault();
        console.log("test");
    };
</script>

<form
    on:submit={onSubmit}
    class="flex justify-between text-ellipsis overflow-hidden"
>
    <AutocompleteTextInput
        class="flex-1"
        name="posting[]"
        sources={suggestPostingAccount}
    />
    <AutocompleteTextInput name="amount[]" sources={suggestPostingAmount} />
    <input type="submit" value="" />
</form>
