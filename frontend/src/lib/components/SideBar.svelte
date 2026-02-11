<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { auth, type User } from '$lib/stores/auth';
	import { initializeTheme, theme, toggleTheme } from '$lib/stores/theme';
	import LucideLogIn from '~icons/lucide/log-in';
	import LucideHouse from '~icons/lucide/house';
	import LucideGauge from '~icons/lucide/gauge';
	import LucideLayoutDashboard from '~icons/lucide/layout-dashboard';
	import LucideWrench from '~icons/lucide/wrench';
	import LucideMoon from '~icons/lucide/moon';
	import LucideSunMedium from '~icons/lucide/sun-medium';
	import LucideLogOut from '~icons/lucide/log-out';
	import LucideUserMinus from '~icons/lucide/user-minus';
	import LucideChevronsRight from '~icons/lucide/chevrons-right';
	import LucideSettings from '~icons/lucide/settings';
	import LucideMenu from '~icons/lucide/menu';
	import LucideX from '~icons/lucide/x';
	import LucideTrophy from '~icons/lucide/trophy';
	import LucideImport from '~icons/lucide/import';
	import LucideBook from '~icons/lucide/book';
	import LucideCircleUser from '~icons/lucide/circle-user';
	import { onMount } from 'svelte';
	import { NavLink, NavButton, IconButton } from '$lib';
	import UserTag from '$lib/components/ui/UserTag.svelte';
	import { impersonateUser } from '$lib/api/admin';
	import Avatar from './ui/Avatar.svelte';
	import { createApi } from '$lib/api/api';

	interface Props {
		serverAuth?: {
			isAuthenticated: boolean;
			user: User | null;
			impersonation: App.ImpersonationInfo | null;
		};
	}

	let { serverAuth }: Props = $props();

	const isAuthenticated = $derived(
		$auth.isLoading ? (serverAuth?.isAuthenticated ?? false) : $auth.isAuthenticated
	);
	const user = $derived($auth.isLoading ? (serverAuth?.user ?? null) : $auth.user);
	const impersonation = $derived(
		$auth.isLoading ? (serverAuth?.impersonation ?? null) : $auth.impersonation
	);

	let collapsed: boolean = $state(false);
	let buttonMode: boolean = $state(false);
	let showMobileSidebar: boolean = $state(false);

	const sidebarPositionClass = $derived(
		buttonMode
			? showMobileSidebar
				? 'fixed inset-y-0 left-0 top-0 z-50 h-[100dvh] max-h-[100dvh] overflow-y-auto shadow-2xl'
				: 'hidden'
			: 'block relative h-full'
	);

	const sidebarWidth = $derived(buttonMode ? 'min(20rem, 100vw)' : collapsed ? '5rem' : '16rem');

	const api = createApi(fetch);

	function toggleCollapse() {
		collapsed = !collapsed;
		localStorage.setItem('sidebar_collapsed', collapsed ? '1' : '0');
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
		initializeTheme();

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
	<IconButton
		variant="default"
		size="xl"
		title="Open sidebar"
		onclick={openMobileSidebar}
		className="md:hidden fixed top-4 left-4 z-50 bg-ctp-base shadow-lg"
	>
		<LucideMenu class="w-7 h-7" />
	</IconButton>
{/if}

{#if buttonMode && showMobileSidebar}
	<button
		type="button"
		class="fixed inset-0 bg-ctp-surface0/60 backdrop-blur-sm z-40 md:hidden"
		onclick={closeMobileSidebar}
		aria-label="Close sidebar overlay"
	></button>
{/if}

<div
	class="{sidebarPositionClass} md:static md:top-auto md:left-auto md:z-auto"
	style="width: {sidebarWidth};"
>
	<div
		class="bg-ctp-base text-ctp-text h-full p-4 border-r border-ctp-surface0 transition-all duration-300 relative flex flex-col justify-start"
	>
		<div
			class="flex items-center gap-4 transition-all duration-300 {user
				? collapsed
					? 'justify-center mb-4'
					: 'justify-start mb-6'
				: ''}"
		>
			{#if user}
				{#if user.avatar_url}
					<Avatar url={user.avatar_url} size={collapsed ? 48 : 64} />
				{/if}
				<div class={collapsed ? 'hidden' : ''}>
					<div class="flex flex-row items-center gap-1 align-middle">
						<UserTag admin_level={user.admin_level} />
					</div>
					<h2 class="{getNameSizeClass(user.name)} text-ctp-subtext1 font-bold">
						{user.name || 'User'}
					</h2>
				</div>
			{/if}
		</div>

		<div class="flex flex-col justify-between transition-all duration-300 mt-0">
			<nav class="space-y-2 flex flex-col transition-all duration-300">
				{#if !isAuthenticated || !user}
					<NavLink
						href="/"
						active={page.url.pathname === '/'}
						{collapsed}
						onclick={() => setTimeout(closeMobileSidebar, 100)}
					>
						{#snippet icon()}<LucideHouse class="w-6 h-6" />{/snippet}
						Home
					</NavLink>
				{/if}

				{#if isAuthenticated && user}
					<NavLink
						href="/dashboard"
						active={page.url.pathname === '/dashboard'}
						{collapsed}
						onclick={() => setTimeout(closeMobileSidebar, 100)}
					>
						{#snippet icon()}<LucideGauge class="w-6 h-6" />{/snippet}
						Dashboard
					</NavLink>

					<NavLink
						href="/projects"
						active={page.url.pathname === '/projects'}
						{collapsed}
						onclick={() => setTimeout(closeMobileSidebar, 100)}
					>
						{#snippet icon()}<LucideLayoutDashboard class="w-6 h-6" />{/snippet}
						Projects
					</NavLink>

					<NavLink
						href={`/@${user.name}`}
						active={page.url.pathname === `/@${user.name}`}
						{collapsed}
						onclick={() => setTimeout(closeMobileSidebar, 100)}
					>
						{#snippet icon()}<LucideCircleUser class="w-6 h-6" />{/snippet}
						Profile
					</NavLink>
				{/if}

				<NavLink
					href="/leaderboard"
					active={page.url.pathname === '/leaderboard'}
					{collapsed}
					onclick={() => setTimeout(closeMobileSidebar, 100)}
				>
					{#snippet icon()}<LucideTrophy class="w-6 h-6" />{/snippet}
					Leaderboard
				</NavLink>

				{#if user?.admin_level && user.admin_level >= 1}
					<NavLink
						href="/admin"
						active={page.url.pathname === '/admin'}
						{collapsed}
						permission="admin"
						onclick={() => setTimeout(closeMobileSidebar, 100)}
					>
						{#snippet icon()}<LucideWrench class="w-6 h-6" />{/snippet}
						Admin
					</NavLink>
				{/if}

				{#if user?.admin_level && user.admin_level >= 2}
					<NavLink
						href="/imports"
						active={page.url.pathname === '/imports'}
						{collapsed}
						permission="owner"
						onclick={() => setTimeout(closeMobileSidebar, 100)}
					>
						{#snippet icon()}<LucideImport class="w-6 h-6" />{/snippet}
						Imports
					</NavLink>
				{/if}

				{#if isAuthenticated && user}
					<NavLink
						href="/settings"
						active={page.url.pathname === '/settings'}
						{collapsed}
						onclick={() => setTimeout(closeMobileSidebar, 100)}
					>
						{#snippet icon()}<LucideSettings class="w-6 h-6" />{/snippet}
						Settings
					</NavLink>
				{/if}

				<NavLink href="/docs" active={false} {collapsed}>
					{#snippet icon()}<LucideBook class="w-6 h-6" />{/snippet}
					Docs
				</NavLink>

				{#if isAuthenticated && user}
					<NavButton
						{collapsed}
						onclick={() => {
							handleLogout();
							setTimeout(closeMobileSidebar, 100);
						}}
					>
						{#snippet icon()}<LucideLogOut class="w-6 h-6" />{/snippet}
						Logout
					</NavButton>
				{:else}
					<NavButton
						variant="primary"
						{collapsed}
						onclick={() => {
							auth.login();
							setTimeout(closeMobileSidebar, 100);
						}}
					>
						{#snippet icon()}<LucideLogIn class="w-6 h-6" />{/snippet}
						Login
					</NavButton>
				{/if}
			</nav>

			<IconButton
				variant="ghost"
				size="lg"
				title="Toggle theme"
				onclick={toggleTheme}
				className="absolute {collapsed ? 'bottom-18' : 'bottom-5'} left-5"
			>
				{#if $theme === 'light'}
					<LucideMoon class="w-8 h-8 text-ctp-subtext0" />
				{:else}
					<LucideSunMedium class="w-8 h-8 text-ctp-subtext0" />
				{/if}
			</IconButton>

			{#if buttonMode}
				<IconButton
					variant="ghost"
					size="lg"
					title="Close sidebar"
					onclick={closeMobileSidebar}
					className="absolute top-5 right-5"
				>
					<LucideX class="w-8 h-8 text-ctp-subtext0" />
				</IconButton>
			{:else}
				<IconButton
					variant="ghost"
					size="lg"
					title="Toggle sidebar collapse"
					onclick={toggleCollapse}
					className="absolute bottom-5 {collapsed ? 'right-5' : 'right-5'}"
				>
					<LucideChevronsRight
						class="w-8 h-8 text-ctp-subtext0 transition-transform duration-400"
						style="transform: rotate({collapsed ? '0deg' : '-180deg'})"
					/>
				</IconButton>
			{/if}
		</div>

		{#if impersonation && user}
			<div
				class="mt-4 rounded-md border border-ctp-yellow/40 bg-ctp-yellow/10 px-3 py-3 text-xs text-ctp-subtext0 transition-all duration-300"
			>
				{#if !collapsed}
					<p class="mb-1 text-sm font-semibold text-ctp-text">
						Impersonating: {user.name || 'User'}
					</p>
					<p class="mb-3 text-xs">
						From: {impersonation.admin_name || 'Admin'}
					</p>
				{/if}
				<NavButton
					variant="warning"
					{collapsed}
					className="w-full justify-center font-semibold text-sm"
					onclick={() => {
						setTimeout(closeMobileSidebar, 100);
						impersonateUser(api, impersonation!.admin_id);
					}}
				>
					{#snippet icon()}<LucideUserMinus class="h-5 w-5" />{/snippet}
					Stop impersonating
				</NavButton>
			</div>
		{/if}
	</div>
</div>
