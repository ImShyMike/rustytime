<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/rustytime.svg';
	import { page } from '$app/state';
	import { auth } from '$lib/stores/auth';
	import { AuthErrorWarning, SideBar } from '$lib';

	const props = $props();
	let { children, data } = props;

	const SITE_TITLE = 'rustytime';
	const SITE_DESCRIPTION = 'Blazingly fast time tracking for developers.';
	const SITE_URL = 'https://rustytime.shymike.dev';
	const ogImageUrl = `${SITE_URL}/og-image.png`;

	const canonicalUrl = $derived(`${SITE_URL}${page.url.pathname}${page.url.search}`);

	type AuthData = App.PageData['auth'];
	const DEFAULT_AUTH: AuthData = {
		isAuthenticated: false,
		sessionId: null,
		user: null,
		impersonation: null
	};

	let lastAuthSnapshot = '';

	const hydrateAuth = (incoming: AuthData | undefined) => {
		const authData = incoming ?? DEFAULT_AUTH;
		const serialized = JSON.stringify(authData);

		if (serialized === lastAuthSnapshot) {
			return;
		}

		lastAuthSnapshot = serialized;
		auth.hydrate(authData);
	};

	hydrateAuth(data?.auth);

	$effect(() => {
		hydrateAuth(data?.auth);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<link rel="canonical" href={canonicalUrl} />
	<title>{SITE_TITLE}</title>
	<meta name="description" content={SITE_DESCRIPTION} />

	<meta property="og:type" content="website" />
	<meta property="og:site_name" content={SITE_TITLE} />
	<meta property="og:title" content={SITE_TITLE} />
	<meta property="og:description" content={SITE_DESCRIPTION} />
	<meta property="og:url" content={canonicalUrl} />
	<meta property="og:image" content={ogImageUrl} />
	<meta property="og:image:alt" content="rustytime logo" />

	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content={SITE_TITLE} />
	<meta name="twitter:description" content={SITE_DESCRIPTION} />
	<meta name="twitter:image" content={ogImageUrl} />
</svelte:head>

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
