<script lang="ts">
    import { Account } from "$lib";
    export let parent: Account | undefined = undefined;
    export let accounts: Account[];

    const byRootAccount = accounts.reduce((acc, account) => {
        const [root, ...rest] = Account.split(account);
        console.log({ root, rest });
        if (root === undefined) {
            throw new Error(`invlid account: '${account}'`);
        }
        if (acc[root] === undefined) {
            acc[root] = [];
        }
        if (rest.length > 0) {
            acc[root].push(Account.join(...rest));
        }
        return acc;
    }, {} as Record<Account, Account[]>);
</script>

<ul>
    {#each Object.entries(byRootAccount) as [root, accounts]}
        <li>
            {#if accounts.length > 0}
                <details class="cursor-pointer">
                    <summary>{root}</summary>
                    <div class="ml-4">
                        <svelte:self parent={root} {accounts} />
                    </div>
                </details>
            {:else}
                <span>
                    {root}
                </span>
            {/if}
        </li>
    {/each}
</ul>
