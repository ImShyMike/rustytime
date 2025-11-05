<script lang="ts">
	import type { PageData } from './$types';
	import { Container, PageScaffold, SectionTitle } from '$lib';
	import LucideCopy from '~icons/lucide/copy';
	import LucideCopyCheck from '~icons/lucide/copy-check';
	import LucideTrash2 from '~icons/lucide/trash-2';
	import LucidePlus from '~icons/lucide/plus';
	import { onMount } from 'svelte';
	import { createApi } from '$lib/api/api';
	import {
		getProjectAliases,
		getProjects,
		addProjectAlias,
		deleteProjectAlias
	} from '$lib/api/project';
	import { safeText } from '$lib/utils/text';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let settingsData = $state(data);

	const api = createApi(fetch);
	let aliases = $state<Awaited<ReturnType<typeof getProjectAliases>> | null>(null);
	let projects = $state<Awaited<ReturnType<typeof getProjects>> | null>(null);

	let selectedMainProject = $state<number | null>(null);
	let selectedAliasProject = $state<number | null>(null);
	let isAddingAlias = $state(false);

	const usedAsAliasIds = $derived(
		aliases
			? new Set(
					aliases.aliases.flatMap((aliasEntry) =>
						aliasEntry.aliases.map((aliasRecord) => aliasRecord.project_id)
					)
				)
			: new Set()
	);

	const availableProjects = $derived(
		projects ? projects.projects.filter((p) => !usedAsAliasIds.has(p.id)) : []
	);

	async function loadData() {
		aliases = await getProjectAliases(api);
		projects = await getProjects(api);
	}

	async function handleAddAlias() {
		if (!selectedMainProject || !selectedAliasProject) return;
		isAddingAlias = true;
		try {
			await addProjectAlias(api, selectedMainProject, selectedAliasProject);
			await loadData();
			selectedMainProject = null;
			selectedAliasProject = null;
		} catch (error) {
			console.error('Failed to add alias:', error);
		} finally {
			isAddingAlias = false;
		}
	}

	async function handleDeleteAlias(aliasId: number) {
		try {
			await deleteProjectAlias(api, aliasId);
			await loadData();
		} catch (error) {
			console.error('Failed to delete alias:', error);
		}
	}

	onMount(() => {
		loadData();
	});

	let selectedTab = $state<'setup' | 'projects'>('setup');
	const tabs = [
		{ id: 'setup' as const, label: 'Setup' },
		{ id: 'projects' as const, label: 'Projects' }
	];

	let config: string = $state('');
	let copied: boolean = $state(false);
	let commandCopied: boolean = $state(false);
	let os: string = $state('windows');

	function copySetup() {
		if (!config) return;
		navigator.clipboard.writeText(config).then(() => {
			copied = true;
			setTimeout(() => (copied = false), 2000);
		});
	}

	const installCommands: Record<string, string> = {
		linux: `curl -fsSL https://raw.githubusercontent.com/ImShyMike/timesplit/refs/heads/main/install.sh | sudo bash -s -- update`,
		windows: `iwr -useb https://raw.githubusercontent.com/ImShyMike/timesplit/refs/heads/main/install.ps1 -OutFile install.ps1; powershell -ExecutionPolicy Bypass -Command ".\\install.ps1 update"`,
		macos: `curl -fsSL https://raw.githubusercontent.com/ImShyMike/timesplit/refs/heads/main/install_macos.sh | sudo bash -s -- update`
	};

	function copyCommand() {
		const cmd = installCommands[os];
		if (!cmd) return;
		navigator.clipboard.writeText(cmd).then(() => {
			commandCopied = true;
			setTimeout(() => (commandCopied = false), 2000);
		});
	}

	$effect(() => {
		if (settingsData) {
			config = `[settings]
api_url = "https://api-rustytime.shymike.dev/api/v1"
api_key = ${settingsData.api_key ?? 'REDACTED'}`;
		} else {
			config = '';
		}
	});

	onMount(() => {
		const platform = navigator.userAgent.toLowerCase();

		if (platform.includes('win')) os = 'windows';
		else if (platform.includes('mac')) os = 'macos';
		else if (platform.includes('linux')) os = 'linux';
	});
</script>

<svelte:head>
	<title>Settings - rustytime</title>
</svelte:head>

{#if settingsData}
	<PageScaffold title="Settings">
		<!-- Top selector -->
		<div class="flex gap-1 border-b bg-surface0/50 rounded-xl p-1 border border-surface1 mb-4">
			{#each tabs as tab (tab.id)}
				<button
					class="grow cursor-pointer px-4 py-2 rounded-md font-medium transition-colors {selectedTab ===
					tab.id
						? 'bg-blue/75 text-crust'
						: 'text-subtext0 hover:text-text'}"
					onclick={() => (selectedTab = tab.id)}
				>
					{tab.label}
				</button>
			{/each}
		</div>

		{#if selectedTab === 'setup'}
			<!-- Setup stuff -->
			<Container>
				<SectionTitle level="h2" className="mb-3">Setup</SectionTitle>
				<div class="space-y-4">
					<div>
						<label for="api-setup" class="block text-sm font-medium text-ctp-text mb-2"
							>Copy this into your <code class="bg-ctp-surface1 p-1">~/.wakatime.cfg</code> file:</label
						>
						<div class="relative w-full">
							<div
								id="api-setup"
								class="resize-none text-text block w-full pr-14 px-2 py-2 border border-ctp-surface1 rounded-md bg-ctp-surface0/40 text-sm font-mono"
							>
								<p class="text-mauve">[<span class="text-text">settings</span>]</p>
								<p class="text-blue">
									api_url <span class="text-text">=</span>
									<span class="text-green">"https://api-rustytime.shymike.dev/api/v1"</span>
								</p>
								<p class="text-blue">
									api_key <span class="text-text">=</span>
									<span class="text-{settingsData.api_key ? 'yellow' : 'red'}"
										>{settingsData.api_key ?? 'REDACTED'}</span
									>
								</p>
							</div>
							<button
								onclick={() => copySetup()}
								aria-label="Copy setup to clipboard"
								class={`absolute top-2 right-2 cursor-pointer h-8 px-2 text-ctp-base text-sm rounded transition-transform duration-200 transform flex items-center gap-2 hover:scale-105 active:scale-100 ` +
									(copied
										? 'bg-ctp-green-600 hover:bg-ctp-green-700'
										: 'bg-ctp-blue/70 hover:bg-ctp-blue')}
							>
								{#if copied}
									<LucideCopyCheck class="w-4 h-4 inline" />
								{:else}
									<LucideCopy class="w-4 h-4 inline" />
								{/if}
							</button>
						</div>
					</div>
				</div>
			</Container>

			<!-- Timesplit -->
			<Container className="mt-4">
				<SectionTitle level="h2" className="mb-3">Timesplit</SectionTitle>
				<div class="space-y-4">
					<p class="text-ctp-text">
						Want to use multiple WakaTime servers at once? Check out
						<a
							href="https://github.com/ImShyMike/timesplit"
							target="_blank"
							rel="noopener noreferrer"
							class="text-ctp-blue underline hover:text-ctp-blue/80">Timesplit</a
						>!
						<br />
						Or simply use the quick install command for your operating system below.
					</p>
					<!-- OS selector + install command -->
					<div class="mt-3">
						<div class="flex items-center gap-3">
							<div class="inline-flex rounded-md bg-ctp-surface0/40 p-1 border border-surface1">
								<button
									onclick={() => (os = 'linux')}
									class={`cursor-pointer px-3 py-1 text-sm rounded ${os === 'linux' ? 'bg-ctp-blue/80 text-ctp-base' : 'text-ctp-text hover:bg-ctp-surface0/60'}`}
								>
									Linux
								</button>
								<button
									onclick={() => (os = 'macos')}
									class={`cursor-pointer px-3 py-1 text-sm rounded ${os === 'macos' ? 'bg-ctp-blue/80 text-ctp-base' : 'text-ctp-text hover:bg-ctp-surface0/60'}`}
								>
									macOS
								</button>
								<button
									onclick={() => (os = 'windows')}
									class={`cursor-pointer px-3 py-1 text-sm rounded ${os === 'windows' ? 'bg-ctp-blue/80 text-ctp-base' : 'text-ctp-text hover:bg-ctp-surface0/60'}`}
								>
									Windows
								</button>
							</div>
						</div>

						{#key os}
							<div class="relative mt-3">
								<pre
									class="text-text resize-none block w-full pr-14 px-3 py-3 border border-ctp-surface1 rounded-md bg-ctp-surface0/40 text-sm font-mono whitespace-pre-wrap wrap-break-word">{installCommands[
										os
									]}</pre>
								<button
									onclick={() => copyCommand()}
									aria-label="Copy install command"
									class={`absolute top-2 right-2 cursor-pointer h-8 px-2 text-ctp-base text-sm rounded transition-transform duration-200 transform flex items-center gap-2 hover:scale-105 active:scale-100 ` +
										(commandCopied
											? 'bg-ctp-green-600 hover:bg-ctp-green-700'
											: 'bg-ctp-blue/70 hover:bg-ctp-blue')}
								>
									{#if commandCopied}
										<LucideCopyCheck class="w-4 h-4 inline" />
									{:else}
										<LucideCopy class="w-4 h-4 inline" />
									{/if}
								</button>
							</div>
						{/key}

						{#if os === 'windows'}
							<p class="mt-2 text-xs text-ctp-text/80">
								Note: This command must be run from an elevated PowerShell window.
							</p>
						{/if}
					</div>
				</div>
			</Container>
		{:else if selectedTab === 'projects'}
			<!-- Projects aliases -->
			<Container>
				<SectionTitle level="h2" className="mb-3">Project Aliases</SectionTitle>
				<p class="text-ctp-text mb-4">
					Here you can add aliases for projects. For example, adding "myapp" with "myapp-backend" as an alias will combine them into the "myapp" project.
				</p>

				{#if aliases && projects}
					<!-- Add new alias form -->
					<div class="bg-ctp-surface0/40 border border-surface1 rounded-lg p-4 mb-4">
						<h3 class="text-sm font-semibold text-text mb-3">Add New Alias</h3>
						<div class="flex flex-col sm:flex-row gap-3">
							<div class="flex-1">
								<label for="main-project" class="block text-xs text-subtext0 mb-1"
									>Main Project</label
								>
								<select
									id="main-project"
									bind:value={selectedMainProject}
									class="w-full px-3 py-2 bg-surface1/40 border border-surface2 rounded-md text-text text-sm focus:outline-none focus:ring-2 focus:ring-blue"
								>
									<option value={null}>Select main project...</option>
									{#each availableProjects as project (project.id)}
										<option value={project.id}>{safeText(project.name)}</option>
									{/each}
								</select>
							</div>
							<div class="flex-1">
								<label for="alias-project" class="block text-xs text-subtext0 mb-1">Alias</label>
								<select
									id="alias-project"
									bind:value={selectedAliasProject}
									class="w-full px-3 py-2 bg-ctp-surface1/40 border border-surface2 rounded-md text-text text-sm focus:outline-none focus:ring-2 focus:ring-blue"
								>
									<option value={null}>Select alias project...</option>
									{#each availableProjects as project (project.id)}
										<option value={project.id}>{safeText(project.name)}</option>
									{/each}
								</select>
							</div>
							<div class="flex items-end">
								<button
									onclick={handleAddAlias}
									disabled={!selectedMainProject || !selectedAliasProject || isAddingAlias}
									class="cursor-pointer pl-3 pr-4 py-2 bg-blue hover:bg-blue/80 disabled:bg-surface2 disabled:cursor-not-allowed text-crust rounded-md text-sm font-medium transition-colors flex items-center gap-2"
								>
									<LucidePlus class="w-4 h-4" />
									Add
								</button>
							</div>
						</div>
					</div>

					<!-- Existing aliases list -->
					<div class="space-y-3">
						<h3 class="text-sm font-semibold text-text">Existing Aliases</h3>
						{#if aliases.aliases.length === 0}
							<p class="text-subtext0 text-sm italic">No aliases configured yet.</p>
						{:else}
							<div class="space-y-2">
								{#each aliases.aliases as aliasEntry (aliasEntry.project_id)}
									{@const mainProject = projects.projects.find(
										(p) => p.id === aliasEntry.project_id
									)}
									{#if mainProject}
										<div class="bg-ctp-surface0/40 border border-surface1 rounded-lg p-3">
											<div class="flex items-center justify-between mb-2">
												<h4 class="font-semibold text-text">{safeText(mainProject.name)}</h4>
												<span class="text-xs text-subtext0"
													>{aliasEntry.aliases.length} alias{aliasEntry.aliases.length !== 1
														? 'es'
														: ''}</span
												>
											</div>
											<div class="space-y-1">
												{#each aliasEntry.aliases as aliasRecord (aliasRecord.id)}
													{@const aliasProject = projects.projects.find(
														(p) => p.id === aliasRecord.project_id
													)}
													{#if aliasProject}
														<div
															class="flex items-center justify-between bg-surface1/40 border border-surface1 rounded px-3 py-2"
														>
															<span class="text-sm text-text">{safeText(aliasProject.name)}</span>
															<button
																onclick={() => handleDeleteAlias(aliasRecord.id)}
																class="cursor-pointer text-red hover:text-red/80 transition-colors"
																aria-label="Remove alias"
															>
																<LucideTrash2 class="w-4 h-4" />
															</button>
														</div>
													{/if}
												{/each}
											</div>
										</div>
									{/if}
								{/each}
							</div>
						{/if}
					</div>
				{:else}
					<div class="flex items-center justify-center py-8">
						<p class="text-subtext0">Loading projects...</p>
					</div>
				{/if}
			</Container>
		{/if}
	</PageScaffold>
{/if}
