<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve -- External project links */
	import { invalidate } from '$app/navigation';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import type { PageData } from './$types';
	import { Container, PageScaffold, SectionTitle, Button } from '$lib';
	import type { Project } from '$lib/types/projects';
	import LucideGithub from '~icons/lucide/github';
	import LucideExternalLink from '~icons/lucide/external-link';
	import LucidePencilLine from '~icons/lucide/pencil-line';
	import LucideSearch from '~icons/lucide/search';
	import StatCard from '$lib/components/ui/StatCard.svelte';
	import { formatRelativeTime, creationDateFormatter } from '$lib/utils/time';
	import RelativeTime from '$lib/components/ui/RelativeTime.svelte';
	import { safeText, noUnknownText } from '$lib/utils/text';
	import EditProjectModal from '$lib/components/modals/EditProjectModal.svelte';

	interface Props {
		data: PageData;
	}

	type EnhancedProject = Project & {
		createdAt: Date | null;
		createdAtFormatted: string;
		lastUpdated: Date | null;
		lastUpdatedExact: string | null;
		repoLabel: string | null;
	};

	type SortOption = 'time' | 'recentUpdate' | 'recentCreate' | 'name';

	const sortOptions: Array<{ label: string; value: SortOption }> = [
		{ label: 'Most time', value: 'time' },
		{ label: 'Recently updated', value: 'recentUpdate' },
		{ label: 'Recently created', value: 'recentCreate' },
		{ label: 'Name (A-Z)', value: 'name' }
	];

	let { data }: Props = $props();

	let projectsData = $state<PageData>();

	let searchQuery = $state('');
	let sortOption = $state<SortOption>('time');

	const resetFilters = () => {
		searchQuery = '';
		sortOption = 'time';
	};

	const refreshProjectsData = async () => {
		await invalidate('app:projects');
	};

	let editingProject = $state<Project | null>(null);
	let lastUpdatedAt = $state(new Date());

	$effect(() => {
		projectsData = data;
	});

	setupVisibilityRefresh({
		refresh: refreshProjectsData,
		onError: (error) => {
			console.error('Failed to refresh projects data:', error);
		}
	});

	$effect(() => {
		if (data) {
			lastUpdatedAt = new Date();
		}
	});

	const formatRepoLabel = (repoUrl: string): string => {
		try {
			const url = new URL(repoUrl);

			if (url.hostname.includes('github.com') && url.pathname) {
				return url.pathname.replace(/^\/+/, '').replace(/\.git$/u, '');
			}

			return url.hostname;
		} catch {
			return repoUrl;
		}
	};

	const enhancedProjects = $derived.by((): EnhancedProject[] => {
		if (!projectsData?.projects?.length) {
			return [];
		}

		return projectsData.projects.map((project) => {
			const createdDate = project.created_at ? new Date(project.created_at) : null;
			const updatedDate = project.updated_at ? new Date(project.updated_at) : null;
			const isCreatedAtValid = createdDate && !Number.isNaN(createdDate.getTime());
			const isUpdatedAtValid = updatedDate && !Number.isNaN(updatedDate.getTime());

			return {
				...project,
				createdAt: isCreatedAtValid && createdDate ? createdDate : null,
				createdAtFormatted:
					isCreatedAtValid && createdDate ? creationDateFormatter.format(createdDate) : 'Unknown',
				lastUpdated: isUpdatedAtValid && updatedDate ? updatedDate : null,
				lastUpdatedExact:
					isUpdatedAtValid && updatedDate ? creationDateFormatter.format(updatedDate) : null,
				repoLabel: project.repo_url ? formatRepoLabel(project.repo_url) : null
			} satisfies EnhancedProject;
		});
	});

	const toEpoch = (date: Date | null): number => (date ? date.getTime() : 0);

	const sortComparators: Record<SortOption, (a: EnhancedProject, b: EnhancedProject) => number> = {
		time: (a, b) => b.total_seconds - a.total_seconds,
		recentUpdate: (a, b) =>
			toEpoch(b.lastUpdated ?? b.createdAt) - toEpoch(a.lastUpdated ?? a.createdAt),
		recentCreate: (a, b) => toEpoch(b.createdAt) - toEpoch(a.createdAt),
		name: (a, b) =>
			a.name.localeCompare(b.name, undefined, {
				sensitivity: 'base',
				numeric: true
			})
	};

	const formattedProjects = $derived.by((): EnhancedProject[] => {
		if (!enhancedProjects.length) {
			return [];
		}

		const normalizedQuery = searchQuery.trim().toLowerCase();

		const filtered = enhancedProjects.filter((project) => {
			if (!normalizedQuery) {
				return true;
			}

			const fullText = `${project.name} ${project.repoLabel ?? ''}`.toLowerCase();
			return fullText.includes(normalizedQuery);
		});

		const comparator = sortComparators[sortOption] ?? sortComparators.time;
		return filtered.sort(comparator);
	});

	const lastUpdatedProject = $derived.by<EnhancedProject | null>(() => {
		if (!enhancedProjects.length) {
			return null;
		}

		return (
			[...enhancedProjects].sort((a, b) => {
				return toEpoch(b.lastUpdated ?? b.createdAt) - toEpoch(a.lastUpdated ?? a.createdAt);
			})[0] ?? null
		);
	});
	const lastUpdatedProjectLabel = $derived.by<string>(() => {
		if (!lastUpdatedProject) {
			return 'Awaiting activity';
		}

		return lastUpdatedProject.lastUpdated
			? `Updated ${formatRelativeTime(lastUpdatedProject.lastUpdated)}`
			: 'Awaiting activity';
	});

	const projectCount = $derived(enhancedProjects.length);
	const repoCount = $derived(
		enhancedProjects.filter((project) => Boolean(project.repo_url)).length
	);
	const hasProjects = $derived(projectCount > 0);
	const hasActiveFilters = $derived.by(() => {
		return Boolean(searchQuery.trim());
	});
</script>

<svelte:head>
	<title>Projects - rustytime</title>
</svelte:head>

{#if editingProject !== null}
	<EditProjectModal
		project={editingProject}
		onclose={() => {
			editingProject = null;
		}}
		onsuccess={(updatedRepoUrl) => {
			// update data locally without fully reloading
			const editingId = editingProject?.id;
			if (!editingId || !projectsData?.projects?.length) {
				return;
			}

			const projectIndex = projectsData.projects.findIndex((p) => p.id === editingId);
			if (projectIndex === -1) {
				return;
			}

			const updatedProjects = projectsData.projects.slice();
			updatedProjects[projectIndex] = {
				...updatedProjects[projectIndex],
				repo_url: updatedRepoUrl,
				updated_at: new Date().toISOString()
			};

			projectsData = {
				...projectsData,
				projects: updatedProjects
			};
		}}
	/>
{/if}

{#if projectsData}
	<PageScaffold title="Projects" {lastUpdatedAt}>
		{#if hasProjects}
			<!-- Project Statistics -->
			<div class="grid grid-cols-1 gap-4 mb-4 sm:grid-cols-2">
				<StatCard
					title="Total projects"
					value={formattedProjects.length}
					subvalue="{repoCount} with repositories"
				/>

				{#if lastUpdatedProject}
					<StatCard
						title="Last updated project"
						value={safeText(lastUpdatedProject.name)}
						subvalue={lastUpdatedProjectLabel}
					/>
				{/if}
			</div>

			<!-- Filters and search -->
			<Container className="flex flex-col mb-4 gap-4">
				<div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
					<label class="flex flex-col gap-1 text-sm text-ctp-subtext0 flex-1">
						<span class="text-xs uppercase tracking-wide text-ctp-overlay1">Search</span>
						<div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:gap-4">
							<label
								class="flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-ctp-surface1 bg-ctp-base/60 px-3 py-2 text-sm text-ctp-text shadow-sm focus-within:border-ctp-blue focus-within:bg-ctp-base focus-within:ring-1 focus-within:ring-ctp-blue/40"
							>
								<span class="sr-only">Search projects</span>
								<LucideSearch class="h-4 w-4 text-ctp-overlay1" aria-hidden="true" />
								<input
									type="search"
									class="w-full bg-transparent text-sm text-ctp-text placeholder:text-ctp-overlay1 focus:outline-none"
									placeholder="Search by name or repo"
									value={searchQuery}
									oninput={(event) => {
										const target = event.currentTarget as HTMLInputElement;
										searchQuery = target.value;
									}}
								/>
							</label>
						</div>
					</label>

					<label class="flex flex-col gap-1 text-sm text-ctp-subtext0">
						<span class="text-xs uppercase tracking-wide text-ctp-overlay1">Sort by</span>
						<select
							class="rounded-lg border border-ctp-surface1 bg-ctp-base px-3 py-2 text-sm text-ctp-text focus:border-ctp-blue focus:outline-none"
							value={sortOption}
							onchange={(event) => {
								const target = event.currentTarget as HTMLSelectElement;
								sortOption = target.value as SortOption;
							}}
						>
							{#each sortOptions as option}
								<option value={option.value}>{option.label}</option>
							{/each}
						</select>
					</label>
				</div>
			</Container>

			<!-- Project List -->
			{#if formattedProjects.length}
				<div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
					{#each formattedProjects as project (project.id)}
						<Container className="flex h-full flex-col gap-2">
							<div class="flex items-start justify-between gap-3">
								<div class="flex items-center gap-3">
									<SectionTitle level="h2" size="md" className="text-ctp-text"
										>{noUnknownText(project.name)}</SectionTitle
									>
								</div>

								<LucidePencilLine
									class="h-5 w-5 text-ctp-subtext1/70 hover:text-ctp-subtext0 cursor-pointer"
									title="Edit Project"
									onclick={() => {
										editingProject = project;
									}}
								/>
							</div>

							<div class="flex flex-col justify-between h-full gap-3 text-sm text-ctp-subtext1">
								<span class="font-semibold text-xl text-ctp-lavender"
									>{project.human_readable_total}</span
								>

								{#if project.repo_url}
									<a
										href={project.repo_url}
										class="group flex items-center gap-1 text-sm font-medium text-ctp-blue hover:text-ctp-blue-400"
										target="_blank"
										rel="noopener noreferrer external"
									>
										{#if project.repo_url.includes('github.com')}
											<LucideGithub class="h-4 w-4" aria-hidden="true" />
										{:else}
											<LucideExternalLink class="h-4 w-4" aria-hidden="true" />
										{/if}
										<span>{safeText(project.repoLabel || '')}</span>
									</a>
								{/if}

								<div class="flex flex-col">
									<span class="text-sm font-semibold text-ctp-subtext0"
										>{project.createdAtFormatted}</span
									>
									{#if project.lastUpdated}
										<span
											class="text-xs text-ctp-overlay1"
											title={project.lastUpdatedExact ?? undefined}
										>
											Last updated <RelativeTime
												datetime={project.lastUpdated}
												updateInterval={60000}
											/>
										</span>
									{/if}
								</div>
							</div>
						</Container>
					{/each}
				</div>
			{:else}
				<Container
					className="flex flex-col items-center gap-2 border border-dashed border-ctp-surface0/80 py-6 text-center"
				>
					<p class="text-lg font-semibold text-ctp-text">No projects match your filters</p>
					<p class="text-ctp-subtext0">Try adjusting your search or reset the filters below.</p>
					{#if hasActiveFilters}
						<Button onClick={resetFilters} className="mt-4">Reset filters</Button>
					{/if}
				</Container>
			{/if}
		{:else}
			<Container
				className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center"
			>
				<p class="text-lg font-semibold text-ctp-text">No projects tracked yet</p>
				<p class="text-ctp-subtext0">Start tracking your time to see your projects here!</p>
			</Container>
		{/if}
	</PageScaffold>
{/if}
