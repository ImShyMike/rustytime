<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { page } from '$app/state';
	import AuthErrorWarning from '$lib/components/AuthErrorWarning.svelte';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{page.url.pathname === '/' ? '' : page.url.pathname.slice(1) + ' - '}🦀 rustytime</title>
	<meta name="description" content="Blazingly fast time tracking for developers." />
</svelte:head>

<div class="app-container">
	<div class="error-warning-container">
		<AuthErrorWarning />
	</div>

	<main class="page-content">
		{@render children()}
	</main>
</div>
