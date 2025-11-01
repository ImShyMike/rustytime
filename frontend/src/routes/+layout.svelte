<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { auth } from '$lib/stores/auth';
	import { AuthErrorWarning, Meta, SideBar } from '$lib';
	import { PUBLIC_BACKEND_API_URL } from '$env/static/public';
	import { ProgressBar } from '@prgm/sveltekit-progress-bar';
	import { onMount } from 'svelte';

	const props = $props();
	let { children, data } = props;

	const canonicalUrl = $derived(`${PUBLIC_BACKEND_API_URL}${page.url.pathname}${page.url.search}`);

	auth.hydrate(data?.auth);

	// Fallback if not authenticated
	onMount(() => {
		if (!data?.auth?.isAuthenticated) {
			auth.verify();
		}
	});
</script>

<Meta
	name="rustytime"
	description="Blazingly fast time tracking for developers"
	image={`${PUBLIC_BACKEND_API_URL}/og-image.png`}
	url={PUBLIC_BACKEND_API_URL}
	canonical={canonicalUrl}
	imageAlt="rustytime logo"
/>

<ProgressBar class="text-ctp-lavender" />

<div class="app-container flex h-screen bg-mantle">
	<div class="error-warning-container fixed top-0 left-0 right-0 z-50">
		<AuthErrorWarning />
	</div>

	<div class="side-bar relative w-0 md:w-auto md:flex-shrink-0 h-full">
		<SideBar />
	</div>

	<main class="page-content flex-1 overflow-auto h-full">
		{@render children()}
	</main>
</div>
