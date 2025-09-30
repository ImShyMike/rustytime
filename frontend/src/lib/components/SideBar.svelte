<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { auth } from '$lib/stores/auth';
	import MaterialSymbolsLoginRounded from '~icons/material-symbols/login-rounded';
	import MaterialSymbolsOtherHouses from '~icons/material-symbols/other-houses';
	import MaterialSymbolsDashboard from '~icons/material-symbols/dashboard';
	import MaterialSymbolsToolsWrenchRounded from '~icons/material-symbols/tools-wrench-rounded';
	import MaterialSymbolsDarkModeRounded from '~icons/material-symbols/dark-mode-rounded';
	import MaterialSymbolsLightModeRounded from '~icons/material-symbols/light-mode-rounded';
	import MaterialSymbolsLogoutRounded from '~icons/material-symbols/logout-rounded';
	import MaterialSymbolsDoubleArrowRounded from '~icons/material-symbols/double-arrow-rounded';
	import { onMount } from 'svelte';

	let currentTheme: 'light' | 'dark' = 'light';
	let collapsed: boolean = false;

	function toggleCollapse() {
		collapsed = !collapsed;
		localStorage.setItem('sidebar_collapsed', collapsed ? '1' : '0');
	}

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

	onMount(() => {
		const savedTheme = localStorage.getItem('theme');
		if (savedTheme === 'dark') {
			currentTheme = 'dark';
		} else {
			currentTheme = 'light';
		}

		const savedCollapsed = localStorage.getItem('sidebar_collapsed');
		if (savedCollapsed === '1') {
			collapsed = true;
		}
	});
</script>

<div
	class="relative bg-base text-text h-full p-4 border-r border-surface0 transition-all duration-300"
	style="width: {collapsed ? '5rem' : '16rem'};"
>
	<div
		class="flex items-center justify-center gap-4 transition-all duration-300 {collapsed
			? 'mb-3'
			: 'mb-6'}"
	>
		{#if $auth.user}
			{#if $auth.user.avatar_url}
				<img
					src={$auth.user.avatar_url}
					alt="Profile"
					class="{collapsed
						? 'h-8 w-8'
						: 'h-16 w-16'} transition-all duration-300 rounded-full border-2 border-ctp-green-500"
				/>
			{/if}
			<div class={collapsed ? 'hidden' : ''}>
				<div class="flex flex-row items-center gap-1 align-middle">
					<span
						class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {$auth.user
							.is_admin
							? 'bg-ctp-red-400 text-ctp-crust'
							: 'bg-ctp-surface0 text-ctp-crust'} items-center h-6"
					>
						{$auth.user.is_admin ? 'Admin' : 'User'}
					</span>
				</div>
				<h2 class="text-2xl text-subtext1 font-bold">
					{$auth.user.name || 'User'}
				</h2>
			</div>
		{/if}
	</div>
	<div class="flex flex-col justify-between transition-all duration-300">
		<nav class="space-y-2 flex flex-col transition-all duration-300">
			<a
				href={resolve('/')}
				data-sveltekit-preload-data="hover"
				class="py-2 rounded-md items-center inline-flex {page.url.pathname === '/'
					? 'bg-surface0 text-lavender'
					: 'hover:bg-surface1'} {collapsed ? 'justify-center' : 'px-3'}"
			>
				<MaterialSymbolsOtherHouses class="w-6 h-6 inline" /><span
					class={collapsed ? 'hidden' : 'ml-2'}>Home</span
				>
			</a>
			<a
				href={resolve('/dashboard')}
				data-sveltekit-preload-data="hover"
				class="w-full text-left cursor-pointer py-2 rounded-md items-center inline-flex {page.url
					.pathname === '/dashboard'
					? 'bg-surface0 text-lavender'
					: 'hover:bg-surface1'} {collapsed ? 'justify-center' : 'px-3'}"
			>
				<MaterialSymbolsDashboard class="w-6 h-6 inline" /><span
					class={collapsed ? 'hidden' : 'ml-2'}>Dashboard</span
				>
			</a>
			{#if $auth.user?.is_admin}
				<a
					href={resolve('/admin')}
					data-sveltekit-preload-data="hover"
					class="w-full text-left py-2 cursor-pointer rounded-md items-center outline-dashed bg-yellow/5 outline-1 outline-yellow inline-flex {page
						.url.pathname === '/admin'
						? 'bg-surface0 text-lavender'
						: 'hover:bg-surface1'} {collapsed ? 'justify-center' : 'px-3'}"
				>
					<MaterialSymbolsToolsWrenchRounded class="w-6 h-6 inline" /><span
						class={collapsed ? 'hidden' : 'ml-2'}>Admin</span
					>
				</a>
			{/if}
			{#if $auth.isAuthenticated && $auth.user}
				<button
					onclick={handleLogout}
					class="py-2 rounded-md cursor-pointer items-center inline-flex hover:bg-surface1 {collapsed
						? 'justify-center'
						: 'px-3'}"
				>
					<MaterialSymbolsLogoutRounded class="w-6 h-6 inline" /><span
						class={collapsed ? 'hidden' : 'ml-2'}>Logout</span
					>
				</button>
			{:else}
				<button
					onclick={auth.login}
					class="py-2 rounded-md cursor-pointer items-center inline-flex hover:bg-surface1 {collapsed
						? 'justify-center'
						: 'px-3'}"
				>
					<MaterialSymbolsLoginRounded class="w-6 h-6 inline" /><span
						class={collapsed ? 'hidden' : 'ml-2'}>Login</span
					>
				</button>
			{/if}
		</nav>
		<button
			class="absolute {collapsed
				? 'bottom-18'
				: 'bottom-6'} left-6 cursor-pointer rounded-md items-center inline-flex hover:text-blue"
			onclick={switchTheme}
			aria-label="Toggle theme"
		>
			{#if currentTheme === 'light'}
				<MaterialSymbolsDarkModeRounded class="w-8 h-8 text-subtext0" />
			{:else}
				<MaterialSymbolsLightModeRounded class="w-8 h-8 text-subtext0" />
			{/if}
			<span class={collapsed ? 'hidden' : ''}></span>
		</button>

		<button
			class="absolute bottom-6 right-6 cursor-pointer rounded-md items-center inline-flex hover:text-blue"
			onclick={toggleCollapse}
			aria-label="Toggle sidebar collapse"
		>
			<MaterialSymbolsDoubleArrowRounded
				class="w-8 h-8 text-subtext0 transition-transform duration-400"
				style="transform: rotate({collapsed ? '0deg' : '-180deg'})"
			/>
		</button>
	</div>
</div>
