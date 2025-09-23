<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { createDataLoader } from '$lib/utils/dataLoader';
	import { handleAuthEffect } from '$lib/utils/authEffect';

	import { PieChart } from 'layerchart';
	import { schemeCategory10, schemeSet3 } from 'd3-scale-chromatic';

	const renderContext = 'svg';
	const debug = true;

	const {
		data: dashboardData,
		loading,
		error,
		loadData
	} = createDataLoader<DashboardResponse>('/dashboard');

	interface UsageStat {
		name: string;
		total_seconds: number;
		percent: number;
		text: string;
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
					<!-- Top Projects (Horizontal Bar Chart) -->
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-4">Top Projects</h3>
						{#if $dashboardData.projects.length > 0}
							{#each $dashboardData.projects as project (project.name)}
								<div class="mb-4 last:mb-0">
									<p class="text-sm font-medium text-gray-800 mb-1">{project.name}</p>
									<div class="w-full bg-gray-200 rounded-full h-4">
										<div
											class="bg-green-600 h-4 rounded-full"
											style="width: {project.percent}%; transition: width 0.5s;"
											title={project.text}
										></div>
									</div>
									<p class="text-xs text-gray-600 mt-1">
										{project.text}
									</p>
								</div>
							{/each}
						{:else}
							<p class="text-gray-500">No project data available</p>
						{/if}
					</div>

					<!-- Top Languages (Pie Chart) -->
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-4">Top Languages</h3>
						{#if $dashboardData.languages.length > 0}
							<div class="h-[300px] p-4 rounded-sm overflow-auto">
								<PieChart
									data={$dashboardData.languages}
									key="name"
									value="total_seconds"
									placement="right"
									{renderContext}
									{debug}
									legend={{ placement: 'right', orientation: 'vertical' }}
									cRange={schemeCategory10}
									tooltip
								/>
							</div>
						{:else}
							<p class="text-gray-500">No project data available</p>
						{/if}
					</div>

					<!-- Top Editors (Pie Chart) -->
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-4">Top Editors</h3>
						{#if $dashboardData.editors.length > 0}
							<div class="h-[300px] p-4 rounded-sm overflow-auto">
								<PieChart
									data={$dashboardData.editors}
									key="name"
									value="total_seconds"
									placement="right"
									{renderContext}
									{debug}
									legend={{ placement: 'right', orientation: 'vertical'}}
									cRange={schemeSet3}
									tooltip
								/>
							</div>
						{:else}
							<p class="text-gray-500">No editor data available</p>
						{/if}
					</div>

					<!-- Top Operating Systems (Pie Chart) -->
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-4">Top Operating Systems</h3>
						{#if $dashboardData.operating_systems.length > 0}
							<div class="h-[300px] p-4 rounded-sm overflow-auto">
								<PieChart
									data={$dashboardData.operating_systems}
									key="name"
									value="total_seconds"
									placement="right"
									{renderContext}
									{debug}
									legend={{ placement: 'right', orientation: 'vertical' }}
									cRange={[
										'#ff6b6b',
										'#4ecdc4',
										'#45b7d1',
										'#96ceb4',
										'#feca57',
										'#ff9ff3',
										'#54a0ff',
										'#5f27cd'
									]}
									tooltip
								/>
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
