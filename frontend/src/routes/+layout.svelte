<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { AuthErrorWarning, Meta, SideBar } from '$lib';
	import { PUBLIC_SITE_URL } from '$env/static/public';
	import { ProgressBar } from '@prgm/sveltekit-progress-bar';

	const props = $props();
	let { children, data } = props;

	const canonicalUrl = $derived(`${PUBLIC_SITE_URL}${page.url.pathname}${page.url.search}`);
</script>

<Meta
	name="rustytime"
	description="Blazingly fast time tracking for developers"
	image={`${PUBLIC_SITE_URL}/og-image.png`}
	url={PUBLIC_SITE_URL}
	canonical={canonicalUrl}
	imageAlt="rustytime logo"
/>

<ProgressBar class="text-ctp-lavender" />

<div class="app-container flex h-screen bg-mantle">
	<div class="error-warning-container fixed top-0 left-0 right-0 z-50">
		<AuthErrorWarning />
	</div>

	<div class="side-bar relative w-0 md:w-auto md:shrink-0 h-full">
		<SideBar serverAuth={data?.auth} />
	</div>

	<main class="page-content flex-1 overflow-auto h-full flex flex-col">
		<div class="flex-1">
			{@render children()}
		</div>
	</main>
</div>
