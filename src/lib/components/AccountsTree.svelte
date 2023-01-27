<script lang="ts">
    import type { Account } from "$lib";
    export let root = "";
    export let accounts: Account[];
</script>

<ul>
    {#each accounts.filter(({ aparent_ }) => aparent_ === root) as account}
        <li>
            {#if account.asubs_.length > 0}
                <details class="cursor-pointer">
                    <summary> {account.aname} </summary>
                    <div class="ml-4">
                        <svelte:self root={account.aname} {accounts} />
                    </div>
                </details>
            {:else}
                <span> {account.aname} </span>
            {/if}
        </li>
    {/each}
</ul>
