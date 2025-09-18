<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { createDataLoader } from '$lib/utils/dataLoader';
	import { handleAuthEffect } from '$lib/utils/authEffect';

	const {
		data: dashboardData,
		loading,
		error,
		loadData
	} = createDataLoader<DashboardResponse>('/dashboard');

	interface UsageStat {
		name: string;
		count: number;
		percent: number;
	}

	interface DashboardResponse {
		avatar_url: string;
		username: string;
		user_id: number;
		github_id: number;
		created_at: string;
		expires_at: string;
		api_key: string;
		total_heartbeats: number;
		human_readable_total: string;
		is_admin: boolean;
		dev_mode: boolean;
		projects: UsageStat[];
		editors: UsageStat[];
		operating_systems: UsageStat[];
		languages: UsageStat[];
	}

	$effect(() => {
		handleAuthEffect({
			isAuthLoading: $auth.isLoading,
			isAuthenticated: $auth.isAuthenticated,
			user: $auth.user,
			data: $dashboardData,
			loading: $loading,
			error: $error,
			loadData,
			requireAdmin: false,
			redirectTo: '/'
		});
	});

	const handleLogout = () => {
		auth.logout();
		goto(resolve('/'));
	};
</script>

{#if $loading}
	<div class="min-h-screen flex items-center justify-center">
		<span class="text-gray-500">Loading dashboard...</span>
	</div>
{:else if $auth.isLoading}
	<div class="min-h-screen flex items-center justify-center">
		<span class="text-gray-500">Authenticating...</span>
	</div>
{:else if $error}
	<div class="min-h-screen flex items-center justify-center">
		<div class="text-center">
			<h1 class="text-2xl font-bold text-red-600 mb-4">Error</h1>
			<p class="text-gray-600 mb-4">{$error}</p>
			<button
				onclick={() => window.location.reload()}
				class="cursor-pointer bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded"
				>Retry</button
			>
		</div>
	</div>
{:else if $auth.isAuthenticated && $auth.user && $dashboardData}
	<div class="min-h-screen bg-gray-50">
		<div class="max-w-4xl mx-auto py-12">
			<h1 class="text-3xl font-bold text-green-600 mb-6 flex items-center gap-2">
				<span>🎯</span> rustytime dashboard
			</h1>

			<!-- User Information -->
			<div class="bg-white rounded-xl shadow p-8 mb-8">
				<h2 class="text-xl font-semibold text-gray-800 mb-4">User Information</h2>
				<div class="flex items-center gap-4 mb-4">
					{#if $dashboardData.avatar_url}
						<img
							src={$dashboardData.avatar_url}
							alt="Avatar"
							width="80"
							height="80"
							class="rounded-full border-2 border-green-500"
						/>
					{/if}
					<div>
						<p class="font-bold text-lg text-gray-900">{$dashboardData.username}</p>
						<p class="text-gray-600">GitHub ID: {$dashboardData.github_id}</p>
						<p class="text-gray-600">Member since: {$dashboardData.created_at}</p>
						{#if $dashboardData.is_admin}
							<span
								class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 mt-1"
							>
								Admin
							</span>
						{/if}
					</div>
				</div>
			</div>

			<!-- API Key -->
			<div class="bg-white rounded-xl shadow p-8 mb-8">
				<h2 class="text-xl font-semibold text-gray-800 mb-4">API Configuration</h2>
				<div class="space-y-4">
					<div>
						<label for="api-key" class="block text-sm font-medium text-gray-700 mb-2">API Key</label
						>
						<div class="flex items-center gap-2">
							<input
								id="api-key"
								type="text"
								readonly
								value={$dashboardData.api_key}
								class="block w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-sm font-mono"
							/>
							<button
								onclick={() => navigator.clipboard.writeText($dashboardData!.api_key)}
								class="cursor-pointer px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded transition-colors"
							>
								Copy
							</button>
						</div>
					</div>
					<div class="grid grid-cols-2 gap-4 text-sm text-gray-700">
						<div>
							<span class="font-semibold">Total Heartbeats:</span>
							{$dashboardData.total_heartbeats.toLocaleString()}
						</div>
						<div>
							<span class="font-semibold">Total Time:</span>
							{$dashboardData.human_readable_total}
						</div>
					</div>
				</div>
			</div>

			<!-- Dashboard Statistics -->
			<div class="bg-white rounded-xl shadow p-8 mb-8">
				<h2 class="text-xl font-semibold text-gray-800 mb-6">Dashboard Statistics</h2>
				<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
					<!-- Top Languages -->
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-4">Top Languages</h3>
						{#if $dashboardData.languages.length > 0}
							<div class="space-y-3">
								{#each $dashboardData.languages.slice(0, 5) as lang (lang.name)}
									<div class="flex items-center justify-between">
										<span class="text-gray-700">{lang.name}</span>
										<div class="flex items-center gap-2">
											<div class="w-20 bg-gray-200 rounded-full h-2">
												<div
													class="bg-blue-600 h-2 rounded-full"
													style="width: {lang.percent || 0}%"
												></div>
											</div>
											<span class="text-sm text-gray-500 w-12 text-right"
												>{(lang.percent || 0).toFixed(1)}%</span
											>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<p class="text-gray-500">No language data available</p>
						{/if}
					</div>

					<!-- Top Projects -->
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-4">Top Projects</h3>
						{#if $dashboardData.projects.length > 0}
							<div class="space-y-3">
								{#each $dashboardData.projects.slice(0, 5) as project (project.name)}
									<div class="flex items-center justify-between">
										<span class="text-gray-700 truncate">{project.name}</span>
										<div class="flex items-center gap-2">
											<div class="w-20 bg-gray-200 rounded-full h-2">
												<div
													class="bg-green-600 h-2 rounded-full"
													style="width: {project.percent || 0}%"
												></div>
											</div>
											<span class="text-sm text-gray-500 w-12 text-right"
												>{(project.percent || 0).toFixed(1)}%</span
											>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<p class="text-gray-500">No project data available</p>
						{/if}
					</div>

					<!-- Top Editors -->
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-4">Top Editors</h3>
						{#if $dashboardData.editors.length > 0}
							<div class="space-y-3">
								{#each $dashboardData.editors.slice(0, 5) as editor (editor.name)}
									<div class="flex items-center justify-between">
										<span class="text-gray-700">{editor.name}</span>
										<div class="flex items-center gap-2">
											<div class="w-20 bg-gray-200 rounded-full h-2">
												<div
													class="bg-purple-600 h-2 rounded-full"
													style="width: {editor.percent}%"
												></div>
											</div>
											<span class="text-sm text-gray-500 w-12 text-right"
												>{editor.percent.toFixed(1)}%</span
											>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<p class="text-gray-500">No editor data available</p>
						{/if}
					</div>

					<!-- Top Operating Systems -->
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-4">Top Operating Systems</h3>
						{#if $dashboardData.operating_systems.length > 0}
							<div class="space-y-3">
								{#each $dashboardData.operating_systems.slice(0, 5) as os (os.name)}
									<div class="flex items-center justify-between">
										<span class="text-gray-700">{os.name}</span>
										<div class="flex items-center gap-2">
											<div class="w-20 bg-gray-200 rounded-full h-2">
												<div
													class="bg-orange-600 h-2 rounded-full"
													style="width: {os.percent}%"
												></div>
											</div>
											<span class="text-sm text-gray-500 w-12 text-right"
												>{os.percent.toFixed(1)}%</span
											>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<p class="text-gray-500">No operating system data available</p>
						{/if}
					</div>
				</div>
			</div>

			<div class="flex justify-between mt-8">
				<a href={resolve('/')} class="bg-gray-600 hover:bg-gray-700 text-white py-2 px-6 rounded"
					>Back</a
				>
				<div class="space-x-4">
					{#if $dashboardData.is_admin}
						<a
							href={resolve('/admin')}
							class="bg-red-600 hover:bg-red-700 text-white py-2 px-6 rounded">Admin</a
						>
					{/if}
					<button
						onclick={handleLogout}
						class="cursor-pointer bg-red-600 hover:bg-red-700 text-white py-2 px-6 rounded"
						>Logout</button
					>
				</div>
			</div>
		</div>
	</div>
{:else}
	<div class="min-h-screen flex items-center justify-center">
		<div class="text-center">
			<h1 class="text-2xl font-bold text-red-600 mb-4">Access Denied</h1>
			<p class="text-gray-600 mb-4">You need to be logged in to access the dashboard.</p>
			<a href={resolve('/')} class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded"
				>Go Home</a
			>
		</div>
	</div>
{/if}
