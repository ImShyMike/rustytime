<script lang="ts">
	import type { PageData } from './$types';
	import { Container, PageHeading, SectionTitle } from '$lib';
	import type { Project } from '$lib/types/projects';
	import LucideGithub from '~icons/lucide/github';
	import LucideExternalLink from '~icons/lucide/external-link';
	import StatCard from '$lib/components/ui/StatCard.svelte';
	import { formatRelativeTime, creationDateFormatter } from '$lib/utils/time';

	interface Props {
		data: PageData;
	}

	type EnhancedProject = Project & {
		createdAtFormatted: string;
		lastUpdated: string | null;
		lastUpdatedExact: string | null;
		repoLabel: string | null;
	};

	let { data }: Props = $props();

	const projectsData = data.projectsData;

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

	const sortedProjects = $derived.by((): Project[] => {
		if (!projectsData?.projects) {
			return [];
		}

		return [...projectsData.projects].sort((a, b) => {
			const dateA = a.updated_at ? new Date(a.updated_at).getTime() : 0;
			const dateB = b.updated_at ? new Date(b.updated_at).getTime() : 0;
			return dateB - dateA;
		});
	});

	const formattedProjects = $derived.by((): EnhancedProject[] => {
		if (!sortedProjects.length) {
			return [];
		}

		return sortedProjects.map((project) => {
			const createdDate = project.created_at ? new Date(project.created_at) : null;
			const updatedDate = project.updated_at ? new Date(project.updated_at) : null;
			const isCreatedAtValid = createdDate && !Number.isNaN(createdDate.getTime());
			const isUpdatedAtValid = updatedDate && !Number.isNaN(updatedDate.getTime());

			return {
				...project,
				createdAtFormatted:
					isCreatedAtValid && createdDate ? creationDateFormatter.format(createdDate) : 'Unknown',
				lastUpdated: isUpdatedAtValid && updatedDate ? formatRelativeTime(updatedDate) : null,
				lastUpdatedExact:
					isUpdatedAtValid && updatedDate ? creationDateFormatter.format(updatedDate) : null,
				repoLabel: project.repo_url ? formatRepoLabel(project.repo_url) : null
			} satisfies EnhancedProject;
		});
	});

	const lastUpdatedProject = $derived.by<EnhancedProject | null>(() => formattedProjects[0] ?? null);
	const lastUpdatedProjectLabel = $derived.by<string>(() => {
		const project = formattedProjects[0];
		return project?.lastUpdated ? `Updated ${project.lastUpdated}` : 'Awaiting activity';
	});

	const projectCount = $derived(formattedProjects.length);
	const repoCount = $derived(
		formattedProjects.filter((project) => Boolean(project.repo_url)).length
	);
</script>

<svelte:head>
	<title>Projects - rustytime</title>
</svelte:head>

{#if projectsData}
	<div class="bg-ctp-mantle">
		<div class="max-w-6xl mx-auto py-4 md:py-12 px-3">
			<PageHeading title="Projects" />

			<!-- Project Statistics -->
			{#if formattedProjects.length}
				<div class="grid grid-cols-1 gap-4 mb-8 sm:grid-cols-2">
					<StatCard
						title="Total projects"
						value={projectCount}
						subvalue="{repoCount} with repositories"
					/>

					{#if lastUpdatedProject}
						<StatCard
							title="Last updated project"
							value={lastUpdatedProject.name ?? 'Unknown project'}
							subvalue={lastUpdatedProjectLabel}
						/>
					{/if}
					
				</div>

				<!-- Project List -->
				<div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
					{#each formattedProjects as project (project.id)}
						<Container className="flex h-full flex-col gap-2">
							<div class="flex items-start justify-between gap-3">
								<div class="flex items-center gap-3">
									<SectionTitle level="h2" size="md" className="text-ctp-text"
										>{project.name}</SectionTitle
									>
								</div>
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
										rel="noopener noreferrer"
									>
										{#if project.repo_url.includes('github.com')}
											<LucideGithub class="h-4 w-4" aria-hidden="true" />
										{:else}
											<LucideExternalLink class="h-4 w-4" aria-hidden="true" />
										{/if}
										<span>{project.repoLabel}</span>
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
											Last updated {project.lastUpdated}
										</span>
									{/if}
								</div>
							</div>
						</Container>
					{/each}
				</div>
			{:else}
				<Container
					className="flex flex-col items-center gap-4 border border-dashed border-ctp-surface0/80 py-12 text-center"
				>
					<p class="text-lg font-semibold text-ctp-text">No projects tracked yet</p>
					<p class="text-ctp-subtext0">Start tracking your time to see your projects here!</p>
				</Container>
			{/if}
		</div>
	</div>
{/if}
