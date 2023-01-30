<script lang="ts">
    import type { Transaction } from "$lib/types";
    import type { Readable } from "svelte/store";
    import { format } from "date-fns";
    import PostingForm from "./PostingForm.svelte";

    export let transactions: Readable<Transaction[]>;
</script>

<form class="flex flex-1 flex-col max-w-full">
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
            placeholder="new transaction"
            class="flex-1 hover:outline-none focus:outline-none"
            required
        />
    </fieldgroup>

    <ul class="ml-4">
        <li>
            <PostingForm {transactions} />
        </li>
    </ul>
</form>
