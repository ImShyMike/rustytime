<script lang="ts">
	import ErrorPage from '$lib/components/ErrorPage.svelte';
	import { page } from '$app/state';
</script>

<ErrorPage status={page.status} error={page.error} />
