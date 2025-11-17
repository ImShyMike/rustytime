<script lang="ts">
    import ErrorPage from '$lib/components/ErrorPage.svelte';

    export let error: Error;
    export let status: number;
</script>

<ErrorPage {status} {error} />