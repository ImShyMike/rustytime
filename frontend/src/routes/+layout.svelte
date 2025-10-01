<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/rustytime.svg';
	import { page } from '$app/state';
	import AuthErrorWarning from '$lib/components/AuthErrorWarning.svelte';
	import SideBar from '$lib/components/SideBar.svelte';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{page.url.pathname === '/' ? '' : page.url.pathname.slice(1) + ' - '} rustytime</title>
	<meta name="description" content="Blazingly fast time tracking for developers." />
</svelte:head>

<div class="app-container flex h-screen bg-mantle">
	<div class="error-warning-container fixed top-0 left-0 right-0 z-50">
		<AuthErrorWarning />
	</div>

	<div class="side-bar relative w-20 md:w-auto flex-shrink-0 h-full">
		<SideBar />
	</div>

	<main class="page-content flex-1 overflow-auto h-full">
		{@render children()}
	</main>
</div>
