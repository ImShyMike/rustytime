<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/rustytime.svg';
	import { page } from '$app/state';
	import { browser } from '$app/environment';
	import { auth } from '$lib/stores/auth';
	import AuthErrorWarning from '$lib/components/AuthErrorWarning.svelte';
	import SideBar from '$lib/components/SideBar.svelte';

	const props = $props();
	let { children, data } = props;

	type AuthData = App.PageData['auth'];
	const DEFAULT_AUTH: AuthData = {
		isAuthenticated: false,
		sessionId: null,
		user: null
	};

	let lastAuthSnapshot = '';

	const applyAuthState = (authData: AuthData) => {
		auth.set({
			user: authData.user,
			sessionId: authData.sessionId,
			isAuthenticated: authData.isAuthenticated,
			isLoading: false,
			error: null
		});
	};

	const syncClientStorage = (authData: AuthData) => {
		if (!browser) return;

		if (authData.sessionId) {
			localStorage.setItem('rustytime_session_id', authData.sessionId);
		} else {
			localStorage.removeItem('rustytime_session_id');
		}
	};

	const hydrateAuth = (incoming: AuthData | undefined) => {
		const authData = incoming ?? DEFAULT_AUTH;
		const serialized = JSON.stringify(authData);

		if (serialized === lastAuthSnapshot) {
			return;
		}

		lastAuthSnapshot = serialized;
		applyAuthState(authData);
		syncClientStorage(authData);
	};

	hydrateAuth(data?.auth);

	$effect(() => {
		hydrateAuth(data?.auth);
	});
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

	<div class="side-bar relative w-0 md:w-auto md:flex-shrink-0 h-full">
		<SideBar />
	</div>

	<main class="page-content flex-1 overflow-auto h-full">
		{@render children()}
	</main>
</div>
