<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { browser } from '$app/environment';

	import LucideGithub from '~icons/lucide/github';

	// Handle url changes
	$effect(() => {
		if (browser) {
			handleUrlParams(page.url.searchParams);
		}
	});

	function handleUrlParams(urlParams: URLSearchParams) {
		const sessionId = urlParams.get('session_id');
		const userData = urlParams.get('user');
		const authError = urlParams.get('auth_error');

		// Handle OAuth callback
		if (sessionId && userData) {
			try {
				const user = JSON.parse(decodeURIComponent(userData));
				auth.setSession(user, sessionId);
				const newUrl = new URL(window.location.href);
				newUrl.searchParams.delete('session_id');
				newUrl.searchParams.delete('user');
				window.history.replaceState({}, document.title, newUrl.pathname);
			} catch (error) {
				console.error('Failed to parse OAuth callback data:', error);
			}
		}

		// Handle auth error from server redirects
		if (authError === 'unauthorized') {
			auth.setError('unauthorized', 'Please log in to access that page.');
			const newUrl = new URL(window.location.href);
			newUrl.searchParams.delete('auth_error');
			window.history.replaceState({}, document.title, newUrl.pathname);
		}
	}
</script>

<div class="min-h-screen p-8 bg-mantle">
	<!-- Header -->
	<header class="text-center mb-4 mt-[10vh]">
		<div class="flex text-text items-center justify-center gap-3 mb-4">
			<h1 class="text-5xl font-bold">rustytime</h1>
		</div>
		<p class="text-xl text-subtext0">Blazingly fast time tracking for developers.</p>
	</header>

	<!-- Main Content -->
	<div class="rounded-xl p-8 mb-12">
		{#if $auth.isLoading}
			<!-- Loading State -->
			<div class="text-center">
				<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-ctp-text mx-auto"></div>
				<p class="mt-4 text-subtext0">Loading...</p>
			</div>
		{:else if $auth.isAuthenticated && $auth.user}
			<!-- Authenticated User -->
			<div class="text-center">
				<div class="flex items-center justify-center gap-4 mb-6">
					{#if $auth.user.avatar_url}
						<img
							src={$auth.user.avatar_url}
							alt="Profile"
							class="w-16 h-16 rounded-full border-2 border-ctp-green-500"
						/>
					{/if}
					<div>
						<h2 class="text-2xl text-subtext1 font-bold">
							Welcome, {$auth.user.name || 'User'}!
						</h2>
						<div class="flex flex-row items-center gap-1 align-middle">
							<span
								class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {$auth.user
									.is_admin
									? 'bg-ctp-red-400 text-ctp-crust'
									: 'bg-ctp-overlay2 text-ctp-crust'} items-center h-6"
							>
								{$auth.user.is_admin ? 'Admin' : 'User'}
							</span>
							<p class="text-subtext0">User ID: {$auth.user.id}</p>
						</div>
					</div>
				</div>

				<div class="space-y-4">
					<a
						href={resolve('/dashboard')}
						class="inline-block bg-ctp-peach-500 hover:bg-ctp-peach-600 text-ctp-base font-semibold py-3 px-6 rounded-lg"
					>
						Go to Dashboard
					</a>

					{#if $auth.user.is_admin}
						<a
							href={resolve('/admin')}
							class="inline-block bg-ctp-red-600 hover:bg-ctp-red-700 text-ctp-base font-semibold py-3 px-6 rounded-lg ml-4"
						>
							Admin Panel
						</a>
					{/if}

					<button
						onclick={auth.logout}
						class="cursor-pointer block mx-auto text-ctp-text/70 hover:text-ctp-text/50 underline"
					>
						Logout
					</button>
				</div>
			</div>
		{:else}
			<!-- Not Authenticated -->
			<div class="text-center">
				<button
					onclick={auth.login}
					class="cursor-pointer bg-ctp-surface1 text-ctp-text hover:bg-ctp-surface0 font-semibold py-4 px-8 rounded-lg flex items-center gap-3 mx-auto"
				>
					<LucideGithub class="min-w-6 min-h-6" />
					Sign in with GitHub
				</button>
			</div>
		{/if}
	</div>
</div>
