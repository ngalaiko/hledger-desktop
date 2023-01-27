<script lang="ts">
    import type { Account } from "$lib";
    import { createEventDispatcher } from "svelte";
    export let root = "root";
    export let accounts: Account[];

    accounts = accounts.filter((acc) => acc.aname !== "root");

    const dispatch = createEventDispatcher<{ select: { account: string } }>();
</script>

<ul>
    {#each accounts.filter(({ aparent_ }) => aparent_ === root) as account}
        <li>
            {#if account.asubs_.length > 0}
                <details class="cursor-pointer">
                    <summary>
                        <button
                            type="button"
                            on:click={() =>
                                dispatch("select", { account: account.aname })}
                        >
                            {account.aname.split(":").at(-1)}
                        </button>
                    </summary>
                    <div class="ml-4">
                        <svelte:self
                            root={account.aname}
                            {accounts}
                            on:select={({ detail }) =>
                                dispatch("select", detail)}
                        />
                    </div>
                </details>
            {:else}
                <button
                    type="button"
                    on:click={() =>
                        dispatch("select", { account: account.aname })}
                >
                    {account.aname.split(":").at(-1)}
                </button>
            {/if}
        </li>
    {/each}
</ul>
