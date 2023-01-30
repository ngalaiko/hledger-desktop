<script lang="ts">
    import { Amount, type Transaction } from "$lib/types";
    import { derived, type Readable } from "svelte/store";
    import AutocompleteTextInput from "./AutocompleteTextInput.svelte";
    import { mostCommon } from "$lib";

    export let transactions: Readable<Transaction[]>;

    let inputAccount: string = "";

    const accounts = derived(transactions, (transactions) =>
        transactions.flatMap((tx) => tx.tpostings.map((p) => p.paccount))
    );

    $: amounts = derived(
        [transactions, accounts],
        ([transactions, accounts]) => {
            const account = mostCommon(accounts);
            return transactions.flatMap((tx) =>
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
                    .map((p) => Amount.format(p.pamount[0]))
            );
        }
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
        sources={$amounts}
        placeholder="$100"
    />
    <input type="submit" value="" />
</form>
