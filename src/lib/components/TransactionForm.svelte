<script lang="ts">
    import type { Transaction } from "$lib/types";
    import { derived, type Readable } from "svelte/store";
    import { format } from "date-fns";
    import PostingForm from "./PostingForm.svelte";
    import AutocompleteTextInput from "./AutocompleteTextInput.svelte";

    export let transactions: Readable<Transaction[]>;

    export const descriptions = derived(transactions, (transactions) =>
        transactions.map((t) => t.tdescription)
    );
</script>

<form class="flex flex-1 flex-col max-w-full">
    <fieldgroup class="flex gap-2 whitespace-nowrap">
        <AutocompleteTextInput
            name="date"
            sources={[format(new Date(), "yyyy-MM-dd")]}
            class="w-[10ch]"
            required
        />
        <AutocompleteTextInput
            name="description"
            sources={$descriptions}
            placeholder="transaction"
            class="flex-q"
            required
        />
    </fieldgroup>

    <ul class="ml-4">
        <li>
            <PostingForm {transactions} />
        </li>
    </ul>
</form>
