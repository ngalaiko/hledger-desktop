<script lang="ts">
    import { Amount } from "$lib";

    export let amount: Amount;
    export let width = -1;

    const prefix = " ".repeat(
        width > 0 ? width - Amount.format(amount).length : 0
    );
</script>

<div class="flex flex-row font-mono whitespace-pre">
    {prefix}{Amount.format(amount)}
</div>
