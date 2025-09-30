<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { auth } from '$lib/stores/auth';
	import { get } from 'svelte/store';
	import MaterialSymbolsOtherHouses from '~icons/material-symbols/other-houses';
	import MaterialSymbolsDashboard from '~icons/material-symbols/dashboard';
	import MaterialSymbolsToolsWrenchRounded from '~icons/material-symbols/tools-wrench-rounded';
	import MaterialSymbolsDarkModeRounded from '~icons/material-symbols/dark-mode-rounded';
	import MaterialSymbolsLightModeRounded from '~icons/material-symbols/light-mode-rounded';
	import MaterialSymbolsLogoutRounded from '~icons/material-symbols/logout-rounded';
	import { onMount } from 'svelte';

	let currentTheme: 'light' | 'dark' = 'light';

	function switchTheme() {
		if (currentTheme === 'light') {
			document.documentElement.classList.add('mocha');
			currentTheme = 'dark';
			localStorage.setItem('theme', 'dark');
		} else {
			document.documentElement.classList.remove('mocha');
			currentTheme = 'light';
			localStorage.setItem('theme', 'light');
		}
	}

	const handleLogout = () => {
		auth.logout();
		goto(resolve('/'));
	};

	function navIfAuth(url: string, opts: { requireAdmin?: boolean } = {}) {
		const state = get(auth);

		if (state.isAuthenticated && state.user) {
			if (opts.requireAdmin && !state.user.is_admin) {
				auth.update((s) => ({
					...s,
					error: {
						type: 'unauthorized',
						message: 'Admin access required to view that page.',
						timestamp: new Date()
					}
				}));
				return;
			}

            // @ts-expect-error known valid path
            goto(resolve(url));
			return;
		}

		auth.update((s) => ({
			...s,
			error: {
				type: 'unauthorized',
				message: 'You must be logged in to access that page.',
				timestamp: new Date()
			}
		}));
	}

    onMount(() => {
        const savedTheme = localStorage.getItem('theme');
        if (savedTheme === 'dark') {
            currentTheme = 'dark';
        } else {
            currentTheme = 'light';
        }
    });
</script>

<div class="relative bg-base text-text h-full w-64 p-4 border-r border-surface0">
	<h2 class="text-xl font-bold mb-2 text-ctp-mauve">rustytime</h2>
	<div class="flex flex-col justify-between">
		<nav class="space-y-2 flex flex-col">
			<a
				href={resolve('/')}
				class="px-3 py-2 rounded-md items-center inline-flex {page.url.pathname === '/'
					? 'bg-surface0 text-lavender'
					: 'hover:bg-surface1'}"
			>
				<MaterialSymbolsOtherHouses class="w-6 h-6 inline mr-2" />Home
			</a>
			<button
				onclick={() => navIfAuth('/dashboard')}
				class="w-full text-left cursor-pointer px-3 py-2 rounded-md items-center inline-flex {page
					.url.pathname === '/dashboard'
					? 'bg-surface0 text-lavender'
					: 'hover:bg-surface1'}"
			>
				<MaterialSymbolsDashboard class="w-6 h-6 inline mr-2" />Dashboard
			</button>
			{#if $auth.user?.is_admin}
				<button
					onclick={() => navIfAuth('/admin', { requireAdmin: true })}
					class="w-full text-left px-3 py-2 cursor-pointer rounded-md items-center outline-dashed outline-1 outline-yellow inline-flex {page
						.url.pathname === '/admin'
						? 'bg-surface0 text-lavender'
						: 'hover:bg-surface1'}"
				>
					<MaterialSymbolsToolsWrenchRounded class="w-6 h-6 inline mr-2" />Admin
				</button>
			{/if}
			{#if $auth.user}
				<button
					onclick={handleLogout}
					class="px-3 py-2 rounded-md cursor-pointer items-center inline-flex hover:bg-surface1"
				>
					<MaterialSymbolsLogoutRounded class="w-6 h-6 inline mr-2" />Logout
				</button>
			{/if}
		</nav>
		<button
			class="absolute bottom-4 left-4 px-2 py-2 cursor-pointer rounded-md items-center inline-flex hover:text-blue"
			onclick={switchTheme}
			aria-label="Toggle theme"
		>
			{#if currentTheme === 'light'}
				<MaterialSymbolsDarkModeRounded class="w-6 h-6 mr-2" />
			{:else}
				<MaterialSymbolsLightModeRounded class="w-6 h-6 mr-2" />
			{/if}
		</button>
	</div>
</div>
