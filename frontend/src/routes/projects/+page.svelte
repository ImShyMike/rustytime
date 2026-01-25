<script lang="ts">
	import { invalidate } from '$app/navigation';
	import { setupVisibilityRefresh } from '$lib/utils/refresh';
	import type { PageData } from './$types';
	import {
		Container,
		PageScaffold,
		SectionTitle,
		Button,
		SearchInput,
		Select,
		EmptyState,
		IconButton
	} from '$lib';
	import type { Project } from '$lib/types/projects';
	import LucideGithub from '~icons/lucide/github';
	import LucideExternalLink from '~icons/lucide/external-link';
	import LucidePencilLine from '~icons/lucide/pencil-line';
	import LucideEye from '~icons/lucide/eye';
	import LucideEyeOff from '~icons/lucide/eye-off';
	import { createApi } from '$lib/api/api';
	import { updateProject } from '$lib/api/project';
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
		urlLabel: string | null;
	};

	type SortOption = 'time' | 'recentUpdate' | 'recentCreate' | 'name';

	const sortOptions: Array<{ label: string; value: SortOption }> = [
		{ label: 'Most time', value: 'time' },
		{ label: 'Recently updated', value: 'recentUpdate' },
		{ label: 'Recently created', value: 'recentCreate' },
		{ label: 'Name (A-Z)', value: 'name' }
	];

	let { data }: Props = $props();

	let projectsData = $derived(data);

	let searchQuery = $state('');
	let sortOption = $state<SortOption>('time');
	let showHidden = $state(false);

	const resetFilters = () => {
		searchQuery = '';
		sortOption = 'time';
		showHidden = true;
	};

	const refreshProjectsData = async () => {
		await invalidate('app:projects');
	};

	let editingProject = $state<Project | null>(null);
	let lastUpdatedAt = $state(new Date());

	const api = createApi(fetch);

	async function toggleProjectHidden(project: Project) {
		await updateProject(api, project.id, { hidden: !project.hidden });
		await refreshProjectsData();
	}

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
				urlLabel: project.project_url ? formatUrlLabel(project.project_url) : null
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
			if (!showHidden && project.hidden) {
				return false;
			}

			if (!normalizedQuery) {
				return true;
			}

			const fullText = `${project.name} ${project.urlLabel ?? ''}`.toLowerCase();
			return fullText.includes(normalizedQuery);
		});

		const comparator = sortComparators[sortOption] ?? sortComparators.time;

		return filtered.sort((a, b) => {
			if (a.hidden !== b.hidden) {
				return a.hidden ? 1 : -1;
			}
			return comparator(a, b);
		});
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

	const projectCount = $derived(formattedProjects.length);
	const urlCount = $derived(
		formattedProjects.filter((project) => Boolean(project.project_url)).length
	);
	const hiddenCount = $derived(enhancedProjects.filter((project) => project.hidden).length);
	const hasProjects = $derived(projectCount > 0);
	const hasActiveFilters = $derived.by(() => {
		return Boolean(searchQuery.trim()) || !showHidden;
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
		onsuccess={async () => {
			await refreshProjectsData();
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
					value={projectCount}
					subvalue="{urlCount} with project URLs"
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
				<div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
					<label class="flex flex-col gap-1 text-sm text-ctp-subtext0 flex-1">
						<span class="text-xs uppercase tracking-wide text-ctp-overlay1">Search</span>
						<SearchInput
							bind:value={searchQuery}
							placeholder="Search by name or URL"
							oninput={(val) => {
								searchQuery = val;
							}}
						/>
					</label>

					<div class="flex flex-col sm:flex-row items-stretch sm:items-end gap-4">
						{#if hiddenCount > 0}
							<div class="flex flex-col gap-1 flex-1">
								<span class="text-xs uppercase tracking-wide text-ctp-overlay1">Visibility</span>
								<Button
									variant="secondary"
									size="md"
									className="min-w-44"
									onClick={() => {
										showHidden = !showHidden;
									}}
								>
									{#if showHidden}
										<LucideEyeOff class="h-4 w-4" />
										<span class="whitespace-nowrap">Hide hidden ({hiddenCount})</span>
									{:else}
										<LucideEye class="h-4 w-4" />
										<span class="whitespace-nowrap">Show hidden ({hiddenCount})</span>
									{/if}
								</Button>
							</div>
						{/if}

						<div class="flex-1">
							<Select
								bind:value={sortOption}
								options={sortOptions}
								label="Sort by"
								onchange={(val) => {
									sortOption = val;
								}}
							/>
						</div>
					</div>
				</div>
			</Container>

			<!-- Project List -->
			{#if formattedProjects.length}
				<div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
					{#each formattedProjects as project (project.id)}
						<Container className="flex h-full flex-col gap-2 {project.hidden ? 'opacity-50' : ''}">
							<div class="flex items-start justify-between gap-3">
								<div class="flex items-center gap-3">
									<SectionTitle level="h2" size="md" className="text-ctp-text"
										>{noUnknownText(project.name)}</SectionTitle
									>
									{#if project.hidden}
										<span class="text-xs px-2 py-0.5 rounded bg-ctp-surface0 text-ctp-subtext0"
											>Hidden</span
										>
									{/if}
								</div>

								<div class="flex items-center gap-1">
									<IconButton
										variant="ghost"
										size="sm"
										title={project.hidden ? 'Unhide Project' : 'Hide Project'}
										onclick={() => toggleProjectHidden(project)}
									>
										{#if project.hidden}
											<LucideEye class="h-4 w-4" />
										{:else}
											<LucideEyeOff class="h-4 w-4" />
										{/if}
									</IconButton>
									<IconButton
										variant="ghost"
										size="sm"
										title="Edit Project"
										onclick={() => {
											editingProject = project;
										}}
									>
										<LucidePencilLine class="h-4 w-4" />
									</IconButton>
								</div>
							</div>

							<div class="flex flex-col justify-between h-full gap-3 text-sm text-ctp-subtext1">
								<span class="font-semibold text-xl text-ctp-lavender"
									>{project.human_readable_total}</span
								>

								{#if project.project_url}
									<a
										href={project.project_url}
										class="group flex items-center gap-1 text-sm font-medium text-ctp-blue hover:text-ctp-blue-400"
										target="_blank"
										rel="noopener noreferrer external"
									>
										{#if project.project_url.includes('github.com')}
											<LucideGithub class="h-4 w-4" aria-hidden="true" />
										{:else}
											<LucideExternalLink class="h-4 w-4" aria-hidden="true" />
										{/if}
										<span>{safeText(project.urlLabel || '')}</span>
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
				<EmptyState
					title="No projects match your filters"
					description="Try adjusting your search or reset the filters below."
					className="py-6"
				>
					{#if hasActiveFilters}
						<Button onClick={resetFilters}>Reset filters</Button>
					{/if}
				</EmptyState>
			{/if}
		{:else}
			<EmptyState
				title="No projects tracked yet"
				description="Start tracking your time to see your projects here!"
			/>
		{/if}
	</PageScaffold>
{/if}
