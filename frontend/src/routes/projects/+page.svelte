<script lang="ts">
	import type { PageData } from './$types';
	import { Container, PageHeading, SectionTitle } from '$lib';
	import type { Project } from '$lib/types/projects';
	import LucideGithub from '~icons/lucide/github';
	import LucideExternalLink from '~icons/lucide/external-link';
	import StatCard from '$lib/components/ui/StatCard.svelte';
	import { formatDuration, formatRelativeTime, creationDateFormatter } from '$lib/utils/time';

	interface Props {
		data: PageData;
	}

	type EnhancedProject = Project & {
		createdAtFormatted: string;
		createdAtRelative: string | null;
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

	const formattedProjects = $derived.by((): EnhancedProject[] => {
		if (!projectsData?.projects) {
			return [];
		}

		return projectsData.projects.map((project) => {
			const createdDate = project.created_at ? new Date(project.created_at) : null;
			const isValidDate = createdDate && !Number.isNaN(createdDate.getTime());

			return {
				...project,
				createdAtFormatted:
					isValidDate && createdDate ? creationDateFormatter.format(createdDate) : 'Unknown',
				createdAtRelative: isValidDate && createdDate ? formatRelativeTime(createdDate) : null,
				repoLabel: project.repo_url ? formatRepoLabel(project.repo_url) : null
			} satisfies EnhancedProject;
		});
	});

	const totalTrackedSeconds = $derived(
		formattedProjects.reduce(
			(accumulator, project) => accumulator + (project.total_seconds ?? 0),
			0
		)
	);
	const totalTrackedTime = $derived(formatDuration(totalTrackedSeconds));
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
					<StatCard title="Total tracked time" value={totalTrackedTime} />

					<StatCard
						title="Total projects"
						value={projectCount}
						subvalue="({repoCount} with repositories)"
					/>
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
									{#if project.createdAtRelative}
										<span class="text-xs text-ctp-overlay1">{project.createdAtRelative}</span>
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
