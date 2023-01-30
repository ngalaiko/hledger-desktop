<script lang="ts">
    import { Amount, type Transaction } from "$lib/types";
    import { derived, type Readable } from "svelte/store";
    import AutocompleteTextInput from "./AutocompleteTextInput.svelte";
    import { mostCommon } from "$lib";

    export let transactions: Readable<Transaction[]>;

    const mostCommonExpenseAccount = derived(transactions, (transactions) => {
        const accounts = transactions.flatMap((tx) =>
            tx.tpostings
                .filter((p) => p.pamount[0].aquantity.floatingPoint > 0)
                .map((p) => p.paccount)
        );
        if (accounts.length === 0) return undefined;
        return mostCommon(accounts);
    });

    let inputAccount: string = "";

    $: amountToSuggest = derived(
        [transactions, mostCommonExpenseAccount],
        ([transactions, account]) => {
            const amounts = transactions.flatMap((tx) =>
                tx.tpostings
                    .filter((p) => {
                        const prefix =
                            inputAccount.length > 0 ? inputAccount : account;
                        return prefix
                            ? p.paccount
                                  .toLowerCase()
                                  .startsWith(prefix.toLowerCase())
                            : false;
                    })
                    .map((p) => p.pamount[0])
            );
            if (amounts.length === 0) return undefined;
            return mostCommon(amounts);
        }
    );

    const accounts = derived(transactions, (transactions) =>
        transactions.flatMap((tx) => tx.tpostings.map((p) => p.paccount))
    );

    const onSubmit = (e: SubmitEvent) => {
        e.preventDefault();
        console.log("test");
    };
</script>

<form on:submit={onSubmit} class="flex justify-between max-w-full">
    <AutocompleteTextInput
        class="flex-1 text-ellipsis overflow-hidden whitespace-nowrap"
        name="posting[]"
        sources={$accounts}
        placeholder="assets"
        bind:value={inputAccount}
    />
    <AutocompleteTextInput
        name="amount[]"
        sources={$amountToSuggest ? [Amount.format($amountToSuggest)] : []}
        placeholder="$100"
    />
    <input type="submit" value="" />
</form>
