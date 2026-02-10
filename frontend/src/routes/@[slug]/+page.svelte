<script lang="ts">
	import { page } from '$app/state';
	import { replaceState } from '$app/navigation';
	import { onMount } from 'svelte';
	import type { PageData } from './$types';
	import { createDeferredData } from '$lib/utils/deferred-data.svelte';
	import { Container, PageScaffold, SectionTitle, StatCard, EmptyState } from '$lib';
	import { formatDuration } from '$lib/utils/time';
	import LucideExternalLink from '~icons/lucide/external-link';
	import LucideGithub from '~icons/lucide/github';
	import { safeText, noUnknownText } from '$lib/utils/text';
	import ProfileSkeleton from './ProfileSkeleton.svelte';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	const deferred = createDeferredData(() => data.profile);

	onMount(() => {
		requestAnimationFrame(() => {
			if (deferred.data?.username && page.params.slug !== deferred.data.username) {
				replaceState(`/@${deferred.data.username}`, {});
			}
		});
	});

	const formatUrlLabel = (projectUrl: string): string => {
		try {
			const url = new URL(projectUrl);
			if (url.hostname.includes('github.com') && url.pathname) {
				return url.pathname.replace(/^\/+/, '').replace(/\.git$/u, '');
			}
			return url.hostname;
		} catch {
			return projectUrl;
		}
	};
</script>

{#if deferred.showSkeleton}
	<ProfileSkeleton />
{:else if deferred.data}
	{@const profileData = deferred.data}
	<PageScaffold title="@{profileData.username}" showLastUpdated={false}>
		<svelte:fragment slot="heading">
			<div class="flex items-center gap-4 mb-6">
				<img
					src={profileData.avatar_url}
					alt={profileData.username}
					class="h-16 w-16 rounded-full border-2 border-ctp-surface0"
				/>
				<div>
					<h1 class="text-2xl font-bold text-ctp-text">{profileData.username}</h1>
					<a
						href="https://github.com/{profileData.username}"
						class="text-sm text-ctp-subtext0 hover:text-ctp-blue flex items-center gap-1"
						target="_blank"
						data-umami-event="github-profile-link"
						data-umami-event-name={profileData.username}
						rel="noopener noreferrer external"
					>
						<LucideGithub class="h-3.5 w-3.5" />
						<span>GitHub</span>
					</a>
				</div>
			</div>
		</svelte:fragment>

		<!-- Time Stats -->
		<div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-4">
			<StatCard title="Today" value={formatDuration(profileData.time.today, false)} />
			<StatCard title="This Week" value={formatDuration(profileData.time.week, false)} />
			<StatCard title="All Time" value={formatDuration(profileData.time.all_time, false)} />
		</div>

		<!-- Projects -->
		<SectionTitle className="mb-3"
			>Top Projects <span class="text-sm text-subtext0">(This Month)</span></SectionTitle
		>
		{#if profileData.projects.length > 0}
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
				{#each profileData.projects as project (project.name)}
					<Container className="flex flex-col gap-2">
						<SectionTitle level="h2" size="md" className="text-ctp-text">
							{noUnknownText(project.name)}
						</SectionTitle>

						<span class="font-semibold text-xl text-ctp-lavender">
							{formatDuration(project.total_seconds, false)}
						</span>

						{#if project.project_url}
							<a
								href={project.project_url}
								class="flex items-center gap-1 text-sm font-medium text-ctp-blue hover:text-ctp-blue-400"
								target="_blank"
								rel="noopener noreferrer external"
							>
								{#if project.project_url.includes('github.com')}
									<LucideGithub class="h-4 w-4" aria-hidden="true" />
								{:else}
									<LucideExternalLink class="h-4 w-4" aria-hidden="true" />
								{/if}
								<span>{safeText(formatUrlLabel(project.project_url))}</span>
							</a>
						{/if}
					</Container>
				{/each}
			</div>
		{:else}
			<EmptyState
				title="No projects this month"
				description="This user hasn't tracked any time this month."
			/>
		{/if}
	</PageScaffold>
{:else if deferred.loadError}
	<PageScaffold title="Profile" showLastUpdated={false}>
		<EmptyState
			title="Failed to load profile"
			description="Something went wrong loading the profile data. Please try again."
			className="mb-4"
		/>
	</PageScaffold>
{/if}
