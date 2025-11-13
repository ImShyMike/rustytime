<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { browser } from '$app/environment';
	import { Logo, UserTag } from '$lib';

	import LucideGithub from '~icons/lucide/github';
	import Avatar from '$lib/components/ui/Avatar.svelte';

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
		if (sessionId || userData) {
			const newUrl = new URL(window.location.href);
			newUrl.searchParams.delete('session_id');
			newUrl.searchParams.delete('user');
			window.history.replaceState({}, document.title, newUrl.pathname);
		}

		// Handle auth error from server redirects
		if (authError === 'unauthorized') {
			auth.clear();
			auth.setError('unauthorized', 'Please log in to access that page.');
			const newUrl = new URL(window.location.href);
			newUrl.searchParams.delete('auth_error');
			window.history.replaceState({}, document.title, newUrl.pathname);
		}
	}
</script>

<svelte:head>
	<title>rustytime</title>
</svelte:head>

<div class="bg-mantle">
	<!-- Header -->
	<header class="text-center mb-4 mt-[5vh] sm:mt-[10vh]">
		<Logo
			className="w-32 h-32 mx-auto mb-4 text-ctp-subtext0 dark:text-ctp-lavender-300 drop-shadow-[0_10px_30px_rgba(108,111,133,0.5)] dark:drop-shadow-[0_10px_30px_rgba(198,160,246,0.35)] transition-colors"
		/>
		<div class="flex text-text items-center justify-center gap-3 mb-4">
			<h1 class="text-5xl font-bold">rustytime</h1>
		</div>
		<p class="text-xl text-ctp-subtext1 dark:text-subtext0 px-1">
			Blazingly fast time tracking for developers.
		</p>
	</header>

	<!-- Main Content -->
	<div class="rounded-xl p-8">
		{#if $auth.isAuthenticated && $auth.user}
			<!-- Authenticated User -->
			<div class="text-center">
				<div class="flex items-center justify-center gap-4 mb-6">
					{#if $auth.user.avatar_url}
						<Avatar url={$auth.user.avatar_url} size={64} />
					{/if}
					<div class="self-start text-left wrap-break-word sm:w-auto w-min max-w-full">
						<h2 class="text-2xl text-subtext1 font-bold">
							Welcome, {$auth.user.name || 'User'}!
						</h2>
						<div class="flex flex-row items-center gap-1 align-middle">
							<UserTag admin_level={$auth.user.admin_level} />
							<p class="text-subtext0">User ID: {$auth.user.id}</p>
						</div>
					</div>
				</div>

				<div class="flex flex-col items-center space-y-4">
					<div class="flex flex-col items-center space-y-4 sm:flex-row sm:space-y-0 sm:space-x-4">
						<a
							href={resolve('/dashboard')}
							class="inline-block bg-ctp-mauve-400 hover:bg-ctp-mauve-500 text-ctp-base font-semibold py-3 px-6 rounded-lg"
						>
							Go to Dashboard
						</a>

						{#if $auth.user.admin_level >= 1}
							<a
								href={resolve('/admin')}
								class="inline-block bg-ctp-red-400 hover:bg-ctp-red-500 text-ctp-base font-semibold py-3 px-6 rounded-lg"
							>
								Admin Panel
							</a>
						{/if}
					</div>

					<button
						onclick={auth.logout}
						class="cursor-pointer text-ctp-text/70 hover:text-ctp-text/50 underline"
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
					class="cursor-pointer bg-ctp-surface2 text-ctp-text hover:bg-ctp-surface1 font-semibold py-4 px-8 rounded-lg flex items-center gap-3 mx-auto"
				>
					<LucideGithub class="min-w-6 min-h-6" />
					Sign in with GitHub
				</button>
			</div>
		{/if}
	</div>
</div>
