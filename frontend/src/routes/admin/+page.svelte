<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { createDataLoader } from '$lib/utils/dataLoader';
	import { handleAuthEffect } from '$lib/utils/authEffect';
	import type { AdminResponse } from '$lib/types/admin';

	const { data: adminData, loading, error, loadData } = createDataLoader<AdminResponse>('/admin');

	$effect(() => {
		handleAuthEffect({
			isAuthLoading: $auth.isLoading,
			isAuthenticated: $auth.isAuthenticated,
			user: $auth.user,
			data: $adminData,
			loading: $loading,
			error: $error,
			loadData,
			requireAdmin: true,
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
		<span>Loading admin data...</span>
	</div>
{:else if $auth.isLoading}
	<div class="min-h-screen flex items-center justify-center">
		<span>Authenticating...</span>
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
{:else if $auth.isAuthenticated && $auth.user && $auth.user.is_admin && $adminData}
	<div class="min-h-screen bg-gray-50">
		<div class="max-w-6xl mx-auto py-12">
			<h1 class="text-3xl font-bold text-orange-600 mb-6 flex items-center gap-2">
				admin dashboard
			</h1>

			<!-- System Statistics -->
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
				<div class="bg-white rounded-xl shadow p-6">
					<h3 class="text-lg font-semibold text-gray-800 mb-2">Total Users</h3>
					<p class="text-3xl font-bold text-blue-600">{$adminData.stats.total_users}</p>
				</div>
				<div class="bg-white rounded-xl shadow p-6">
					<h3 class="text-lg font-semibold text-gray-800 mb-2">Total Heartbeats</h3>
					<p class="text-3xl font-bold text-green-600">
						{$adminData.stats.total_heartbeats.toLocaleString()}
					</p>
				</div>
				<div class="bg-white rounded-xl shadow p-6">
					<h3 class="text-lg font-semibold text-gray-800 mb-2">Last 24h</h3>
					<p class="text-3xl font-bold text-purple-600">
						{$adminData.stats.heartbeats_last_24h.toLocaleString()}
					</p>
				</div>
				<div class="bg-white rounded-xl shadow p-6">
					<h3 class="text-lg font-semibold text-gray-800 mb-2">Requests/sec</h3>
					<p class="text-3xl font-bold text-orange-600">{$adminData.stats.requests_per_second}</p>
				</div>
			</div>

			<!-- Top Lists -->
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
				<!-- Top Languages -->
				<div class="bg-white rounded-xl shadow p-6">
					<h3 class="text-xl font-semibold text-gray-800 mb-4">Top Languages</h3>
					{#if $adminData.stats.top_languages.length > 0}
						<div class="space-y-2">
							{#each $adminData.stats.top_languages.slice(0, 5) as lang (lang.language)}
								<div class="flex justify-between items-center">
									<span class="text-gray-700">{lang.language}</span>
									<span class="text-gray-600 font-mono">{lang.count}</span>
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-gray-500">No language data available</p>
					{/if}
				</div>

				<!-- Top Projects -->
				<div class="bg-white rounded-xl shadow p-6">
					<h3 class="text-xl font-semibold text-gray-800 mb-4">Top Projects</h3>
					{#if $adminData.stats.top_projects.length > 0}
						<div class="space-y-2">
							{#each $adminData.stats.top_projects.slice(0, 5) as project (project.project)}
								<div class="flex justify-between items-center">
									<span class="text-gray-700 truncate">{project.project}</span>
									<span class="text-gray-600 font-mono">{project.count}</span>
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-gray-500">No project data available</p>
					{/if}
				</div>
			</div>

			<!-- Daily Activity Chart -->
			<div class="bg-white rounded-xl shadow p-6 mb-8">
				<h3 class="text-xl font-semibold text-gray-800 mb-4">Daily Activity (Last Week)</h3>
				{#if $adminData.stats.daily_activity.length > 0}
					<div class="flex items-end justify-between gap-2 h-40">
						{#each $adminData.stats.daily_activity as day (day.date)}
							<div class="flex flex-col items-center flex-1">
								<div
									class="bg-blue-500 w-full rounded-t flex"
									style="height: {Math.max(
										(day.count /
											Math.max(
												...$adminData.stats.daily_activity.map(
													(d: { date: string; count: number }) => d.count
												)
											)) *
											120,
										2
									)}px"
								></div>
								<span class="text-xs text-gray-600 mt-2">{day.date}</span>
								<span class="text-xs text-gray-500">{day.count}</span>
							</div>
						{/each}
					</div>
				{:else}
					<p class="text-gray-500">No activity data available</p>
				{/if}
			</div>

			<!-- User Management -->
			<div class="bg-white rounded-xl shadow p-6">
				<h3 class="text-xl font-semibold text-gray-800 mb-4">All Users</h3>
				{#if $adminData.stats.all_users.length > 0}
					<div class="overflow-x-auto">
						<table class="min-w-full divide-y divide-gray-200">
							<thead class="bg-gray-50">
								<tr>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
										>User</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
										>GitHub ID</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
										>Created (UTC)</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
										>Admin</th
									>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase"
										>API Key</th
									>
								</tr>
							</thead>
							<tbody class="bg-white divide-y divide-gray-200">
								{#each $adminData.stats.all_users as user (user.github_id)}
									<tr>
										<td class="px-6 py-4 whitespace-nowrap">
											<div class="flex items-center">
												{#if user.avatar_url}
													<img
														src={user.avatar_url}
														alt="Avatar"
														class="h-8 w-8 rounded-full mr-3"
													/>
												{/if}
												<span class="text-sm font-medium text-gray-900"
													>{user.name || 'Unknown'}</span
												>
											</div>
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500"
											>{user.github_id}</td
										>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500"
											>{new Date(user.created_at).toLocaleString('en-US', { timeZone: 'UTC' })}</td
										>
										<td class="px-6 py-4 whitespace-nowrap">
											<span
												class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {user.is_admin
													? 'bg-red-100 text-red-800'
													: 'bg-gray-100 text-gray-800'}"
											>
												{user.is_admin ? 'Admin' : 'User'}
											</span>
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 font-mono">
											<button
												onclick={() => navigator.clipboard.writeText(user.api_key)}
												class="cursor-pointer hover:bg-gray-100 px-2 py-1 rounded transition-colors"
												title="Click to copy full API key"
											>
												{user.api_key.substring(0, 16)}...
											</button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-gray-500">No users found</p>
				{/if}
			</div>

			<div class="flex justify-between mt-8 px-2">
				<a
					href={resolve('/dashboard')}
					class="bg-gray-600 hover:bg-gray-700 text-white py-2 px-6 rounded">Back to Dashboard</a
				>
				<button
					onclick={handleLogout}
					class="cursor-pointer bg-red-600 hover:bg-red-700 text-white py-2 px-6 rounded"
					>Logout</button
				>
			</div>
		</div>
	</div>
{:else}
	<div class="min-h-screen flex items-center justify-center">
		<div class="text-center">
			<h1 class="text-2xl font-bold text-red-600 mb-4">Access Denied</h1>
			<p class="text-gray-600 mb-4">You need admin privileges to access this page.</p>
			<a href={resolve('/')} class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded"
				>Go Home</a
			>
		</div>
	</div>
{/if}
