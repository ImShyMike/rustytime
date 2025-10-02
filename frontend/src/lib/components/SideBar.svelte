<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { auth } from '$lib/stores/auth';
	import LucideLogIn from '~icons/lucide/log-in';
	import LucideHouse from '~icons/lucide/house';
	import LucideLayoutDashboard from '~icons/lucide/layout-dashboard';
	import LucideWrench from '~icons/lucide/wrench';
	import LucideMoon from '~icons/lucide/moon';
	import LucideSunMedium from '~icons/lucide/sun-medium';
	import LucideLogOut from '~icons/lucide/log-out';
	import LucideChevronsRight from '~icons/lucide/chevrons-right';
	import LucideMenu from '~icons/lucide/menu';
	import LucideX from '~icons/lucide/x';
	import { onMount } from 'svelte';
	import UserTag from '$lib/components/UserTag.svelte';

	let currentTheme: 'light' | 'dark' = 'light';
	let collapsed: boolean = false;
	let buttonMode: boolean = false;
	let showMobileSidebar: boolean = false;

	$: sidebarPositionClass = buttonMode
		? showMobileSidebar
			? 'fixed inset-y-0 left-0 top-0 z-50 h-[100dvh] max-h-[100dvh] overflow-y-auto shadow-2xl'
			: 'hidden'
		: 'block relative h-full';

	$: sidebarWidth = buttonMode ? 'min(20rem, 100vw)' : collapsed ? '5rem' : '16rem';

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

	function getNameSizeClass(name: string | null): string {
		if (!name) return 'text-md';
		if (name.length <= 10) return 'text-lg';
		if (name.length <= 20) return 'text-md';
		return 'text-sm';
	}

	const handleLogout = () => {
		auth.logout();
		goto(resolve('/'));
	};

	const closeMobileSidebar = () => {
		showMobileSidebar = false;
	};

	const openMobileSidebar = () => {
		showMobileSidebar = true;
	};

	const updateViewportMode = () => {
		const shouldUseButtonMode = window.innerWidth < 768;
		buttonMode = shouldUseButtonMode;

		if (shouldUseButtonMode) {
			collapsed = false;
		} else {
			showMobileSidebar = false;
		}
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

		updateViewportMode();
		window.addEventListener('resize', updateViewportMode);

		return () => {
			window.removeEventListener('resize', updateViewportMode);
		};
	});
</script>

{#if buttonMode && !showMobileSidebar}
	<button
		class="cursor-pointer md:hidden fixed top-4 left-4 z-50 flex items-center gap-2 bg-base text-text border border-surface0 rounded-md px-2 py-2 shadow-lg hover:bg-surface0/60 transition-colors"
		onclick={openMobileSidebar}
		aria-label="Open sidebar"
	>
		<LucideMenu class="w-7 h-7" />
	</button>
{/if}

{#if buttonMode && showMobileSidebar}
	<button
		type="button"
		class="fixed inset-0 bg-surface0/60 backdrop-blur-sm z-40 md:hidden"
		onclick={closeMobileSidebar}
		aria-label="Close sidebar overlay"
	></button>
{/if}

<div
	class="{sidebarPositionClass} md:static md:top-auto md:left-auto md:z-auto"
	style={`width: ${sidebarWidth};`}
>
	<div
		class="bg-base text-text h-full p-4 border-r border-surface0 transition-all duration-300 relative flex flex-col justify-start"
	>
		<div
			class="flex items-center gap-4 transition-all duration-300 {$auth.user
				? collapsed
					? 'justify-center mb-3'
					: 'justify-start mb-6'
				: ''}"
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
						<UserTag is_admin={$auth.user.is_admin} />
					</div>
					<h2 class="{getNameSizeClass($auth.user.name)} text-subtext1 font-bold">
						{$auth.user.name || 'User'}
					</h2>
				</div>
			{/if}
		</div>
		<div class="flex flex-col justify-between transition-all duration-300">
			<nav class="space-y-2 flex flex-col transition-all duration-300">
				<a
					href={resolve('/')}
					onclick={() => setTimeout(closeMobileSidebar, 100)}
					data-sveltekit-preload-data="hover"
					class="py-2 rounded-md items-center inline-flex {page.url.pathname === '/'
						? 'bg-surface0/70 text-lavender'
						: 'hover:bg-surface1/50'} {collapsed ? 'justify-center' : 'px-3'}"
				>
					<LucideHouse class="w-6 h-6 inline" /><span class={collapsed ? 'hidden' : 'ml-2'}
						>Home</span
					>
				</a>
				<a
					href={resolve('/dashboard')}
					onclick={() => setTimeout(closeMobileSidebar, 100)}
					data-sveltekit-preload-data="hover"
					class="w-full text-left cursor-pointer py-2 rounded-md items-center inline-flex {page.url
						.pathname === '/dashboard'
						? 'bg-surface0/70 text-lavender'
						: 'hover:bg-surface1/50'} {collapsed ? 'justify-center' : 'px-3'}"
				>
					<LucideLayoutDashboard class="w-6 h-6 inline" /><span
						class={collapsed ? 'hidden' : 'ml-2'}>Dashboard</span
					>
				</a>
				{#if $auth.user?.is_admin}
					<a
						href={resolve('/admin')}
						onclick={() => setTimeout(closeMobileSidebar, 100)}
						data-sveltekit-preload-data="hover"
						class="w-full text-left py-2 cursor-pointer rounded-md items-center outline-dashed bg-yellow/5 outline-1 outline-yellow inline-flex {page
							.url.pathname === '/admin'
							? 'bg-surface0/70 text-lavender'
							: 'hover:bg-surface1/50'} {collapsed ? 'justify-center' : 'px-3'}"
					>
						<LucideWrench class="w-6 h-6 inline" /><span class={collapsed ? 'hidden' : 'ml-2'}
							>Admin</span
						>
					</a>
				{/if}
				{#if $auth.isAuthenticated && $auth.user}
					<button
						onclick={() => {
							handleLogout();
							setTimeout(closeMobileSidebar, 100);
						}}
						class="py-2 rounded-md cursor-pointer items-center inline-flex hover:bg-surface1/50 {collapsed
							? 'justify-center'
							: 'px-3'}"
					>
						<LucideLogOut class="w-6 h-6 inline" /><span class={collapsed ? 'hidden' : 'ml-2'}
							>Logout</span
						>
					</button>
				{:else}
					<button
						onclick={() => {
							auth.login();
							setTimeout(closeMobileSidebar, 100);
						}}
						class="py-2 rounded-md cursor-pointer items-center bg-ctp-mauve/50 outline outline-mauve inline-flex hover:bg-ctp-mauve/65 {collapsed
							? 'justify-center'
							: 'px-3'}"
					>
						<LucideLogIn class="w-6 h-6 inline" /><span class={collapsed ? 'hidden' : 'ml-2'}
							>Login</span
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
					<LucideMoon class="w-8 h-8 text-subtext0" />
				{:else}
					<LucideSunMedium class="w-8 h-8 text-subtext0" />
				{/if}
				<span class={collapsed ? 'hidden' : ''}></span>
			</button>

			{#if buttonMode}
				<button
					class="absolute top-4 right-4 cursor-pointer rounded-md items-center inline-flex hover:text-blue"
					onclick={closeMobileSidebar}
					aria-label="Close sidebar"
				>
					<LucideX class="w-8 h-8 text-subtext0" />
				</button>
			{:else}
				<button
					class="absolute bottom-6 {collapsed
						? 'right-6'
						: 'right-2'} right-6 cursor-pointer rounded-md items-center inline-flex hover:text-blue"
					onclick={toggleCollapse}
					aria-label="Toggle sidebar collapse"
				>
					<LucideChevronsRight
						class="w-8 h-8 text-subtext0 transition-transform duration-400"
						style="transform: rotate({collapsed ? '0deg' : '-180deg'})"
					/>
				</button>
			{/if}
		</div>
	</div>
</div>
