<script lang="ts">
    import { hledger } from "$lib";
    import { WithHledgerFile, AccountsTree } from "$lib/components";
</script>

<WithHledgerFile let:filepath>
    {#if filepath !== undefined}
        {#await hledger.accounts(filepath)}
            loading...
        {:then accounts}
            <AccountsTree {accounts} />
        {/await}
    {/if}
</WithHledgerFile>
