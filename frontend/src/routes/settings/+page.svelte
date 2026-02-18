<script lang="ts">
	import type { PageData } from './$types';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import {
		Container,
		PageScaffold,
		SectionTitle,
		Button,
		TabsPanel,
		SegmentedControl,
		CodeBlock,
		CopyButton,
		Select,
		TextInput,
		IconButton
	} from '$lib';
	import LucideTrash2 from '~icons/lucide/trash-2';
	import LucideLoader2 from '~icons/lucide/loader-2';
	import { onMount } from 'svelte';
	import { createApi } from '$lib/api/api';
	import {
		getProjectAliases,
		getProjects,
		addProjectAlias,
		deleteProjectAlias
	} from '$lib/api/project';
	import { startImport, getImportStatus } from '$lib/api/import';
	import { updateSettings } from '$lib/api/settings';
	import type { ImportStatusResponse } from '$lib/types/settings';
	import { onDestroy } from 'svelte';
	import { safeText } from '$lib/utils/text';
	import { formatDuration } from '$lib/utils/time';
	import { PUBLIC_BACKEND_API_URL, PUBLIC_SITE_URL } from '$env/static/public';

	interface Props {
		data: PageData;
	}

	const tabParam = $state(page.url.searchParams.get('tab') ?? '');

	let { data }: Props = $props();

	let settingsData = $derived(data);

	const api = createApi(fetch);
	let aliases = $state<Awaited<ReturnType<typeof getProjectAliases>> | null>(null);
	let projects = $state<Awaited<ReturnType<typeof getProjects>> | null>(null);

	let selectedMainProject = $state<number | undefined>(undefined);
	let selectedAliasProject = $state<number | undefined>(undefined);
	let isAddingAlias = $state(false);

	let setupVariant = $state<'unix' | 'windows' | 'custom'>('custom');
	const setupVariantOptions = [
		{ value: 'unix' as const, label: 'macOS/Linux' },
		{ value: 'windows' as const, label: 'Windows' },
		{ value: 'custom' as const, label: 'Custom' }
	];

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

	const projectOptions = $derived(
		availableProjects.map((p) => ({ value: p.id, label: safeText(p.name) }))
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
			selectedMainProject = undefined;
			selectedAliasProject = undefined;
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

	const unixCommand = () => {
		const apiKey = settingsData?.api_key ?? 'REDACTED';
		return `curl -fsSL ${PUBLIC_SITE_URL}/install.sh | RT_API_KEY="${apiKey}" RT_API_URL="${PUBLIC_BACKEND_API_URL}/api/v1" bash`;
	};

	const windowsCommand = () => {
		const apiKey = settingsData?.api_key ?? 'REDACTED';
		return `$env:RT_API_KEY="${apiKey}"; $env:RT_API_URL="${PUBLIC_BACKEND_API_URL}/api/v1"; irm ${PUBLIC_SITE_URL}/install.ps1 | iex`;
	};

	const tabs = [
		{ id: 'setup' as const, label: 'Setup' },
		{ id: 'projects' as const, label: 'Projects' },
		{ id: 'migration' as const, label: 'Migration' }
	];

	let selectedTab = $state<'setup' | 'projects' | 'migration'>('setup');
	$effect(() => {
		const tabId = tabParam as 'setup' | 'projects' | 'migration';
		if (tabId && tabs.some((tab) => tab.id === tabId)) {
			selectedTab = tabId;
		} else if (!tabParam) {
			selectedTab = 'setup';
		}
	});

	function handleTabChange(nextTab: 'setup' | 'projects' | 'migration') {
		const nextUrl = new URL(page.url);
		nextUrl.searchParams.set('tab', nextTab);
		goto(nextUrl, { replaceState: false, keepFocus: true, noScroll: true });
	}

	let config: string = $state('');
	let os = $state<'linux' | 'macos' | 'windows'>('windows');
	const osOptions = [
		{ value: 'linux' as const, label: 'Linux' },
		{ value: 'macos' as const, label: 'macOS' },
		{ value: 'windows' as const, label: 'Windows' }
	];
	let hackatimeApiKey: string = $state('');
	let isStartingImport = $state(false);
	let importError: string | null = $state(null);
	let importStatus: ImportStatusResponse | null = $state(null);
	let pollInterval: ReturnType<typeof setInterval> | null = null;
	const uuidV4Regex =
		/^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$/;
	const isValidHackatimeApiKey = $derived(uuidV4Regex.test(hackatimeApiKey.trim()));

	function checkIsImportActive(status: ImportStatusResponse | null): boolean {
		return status?.status === 'pending' || status?.status === 'running';
	}

	const isImportActive = $derived(checkIsImportActive(importStatus));

	let savedTimezone = $state<string>('UTC');
	let selectedTimezone = $state<string>('UTC');
	let isSaving = $state(false);
	let timezoneError: string | null = $state(null);
	let timezoneSuccess = $state(false);
	const hasTimezoneChanged = $derived(selectedTimezone !== savedTimezone);

	const getAvailableTimezones = (): string[] => {
		try {
			// this should work
			if (typeof Intl !== 'undefined' && 'supportedValuesOf' in Intl) {
				return (
					Intl as unknown as { supportedValuesOf: (key: string) => string[] }
				).supportedValuesOf('timeZone');
			}
		} catch {
			// fallback
		}
		// fallback
		return ['UTC'];
	};

	const timezoneOptions = $derived(
		getAvailableTimezones().map((tz) => ({ value: tz, label: tz.replace(/_/g, ' ') }))
	);

	async function handleTimezoneChange() {
		if (selectedTimezone === savedTimezone) return;

		isSaving = true;
		timezoneError = null;
		timezoneSuccess = false;

		try {
			await updateSettings(api, { timezone: selectedTimezone });
			savedTimezone = selectedTimezone;
			timezoneSuccess = true;
			setTimeout(() => {
				timezoneSuccess = false;
			}, 3000);
		} catch (error) {
			console.error('Failed to update settings:', error);
			timezoneError = error instanceof Error ? error.message : 'Failed to update settings';
		} finally {
			isSaving = false;
		}
	}

	onMount(() => {
		loadData();
		loadImportStatus().then(() => {
			if (checkIsImportActive(importStatus)) {
				startPolling();
			}
		});
		const platform = navigator.userAgent.toLowerCase();

		if (platform.includes('win')) {
			os = 'windows';
			setupVariant = 'windows';
		} else if (platform.includes('mac')) {
			os = 'macos';
			setupVariant = 'unix';
		} else if (platform.includes('linux')) {
			os = 'linux';
			setupVariant = 'unix';
		}

		if (settingsData?.timezone) {
			savedTimezone = settingsData.timezone;
			selectedTimezone = settingsData.timezone;
		} else {
			try {
				const detected = Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
				savedTimezone = detected;
				selectedTimezone = detected;
			} catch {
				savedTimezone = 'UTC';
				selectedTimezone = 'UTC';
			}
		}
	});

	function formatDate(value: string | null) {
		if (!value) return 'N/A';
		const date = new Date(value);
		return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
	}

	async function loadImportStatus() {
		try {
			importStatus = await getImportStatus(api);
		} catch {
			// No import job exists yet
		}
	}

	function startPolling() {
		stopPolling();
		pollInterval = setInterval(async () => {
			await loadImportStatus();
			if (importStatus && !isImportActive) {
				stopPolling();
			}
		}, 5000);
	}

	function stopPolling() {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	}

	onDestroy(() => {
		stopPolling();
	});

	async function handleHackatimeImport() {
		const trimmedKey = hackatimeApiKey.trim();
		if (!trimmedKey || !uuidV4Regex.test(trimmedKey)) {
			importError = 'Enter a valid Hackatime API key to start the import.';
			return;
		}
		isStartingImport = true;
		importError = null;
		try {
			await startImport(api, trimmedKey);
			await loadImportStatus();
			startPolling();
		} catch (error) {
			console.error('Failed to start import:', error);
			importError =
				error instanceof Error ? error.message : 'Something went wrong while starting the import.';
		} finally {
			isStartingImport = false;
		}
	}

	const installCommands: Record<string, string> = {
		linux: `curl -fsSL https://raw.githubusercontent.com/ImShyMike/timesplit/refs/heads/main/install.sh | sudo bash -s -- update && timesplit setup`,
		windows: `iwr -useb https://raw.githubusercontent.com/ImShyMike/timesplit/refs/heads/main/install.ps1 -OutFile install.ps1; powershell -ExecutionPolicy Bypass -Command ".\\install.ps1 update"; if ($?) { timesplit setup }`,
		macos: `curl -fsSL https://raw.githubusercontent.com/ImShyMike/timesplit/refs/heads/main/install_macos.sh | sudo bash -s -- update && timesplit setup`
	};

	$effect(() => {
		if (settingsData) {
			config = `[settings]
api_url = "${PUBLIC_BACKEND_API_URL}/api/v1"
api_key = ${settingsData.api_key ?? 'REDACTED'}`;
		} else {
			config = '';
		}
	});
</script>

{#if settingsData}
	<PageScaffold title="Settings">
		<!-- Top selector -->
		<TabsPanel {tabs} bind:selected={selectedTab} onchange={handleTabChange} className="mb-4" />

		{#if selectedTab === 'setup'}
			<!-- Preferences -->
			<Container>
				<SectionTitle level="h2" className="mb-3">Preferences</SectionTitle>
				<div class="space-y-4">
					<div class="bg-base/40 border border-surface1 rounded-lg p-4 space-y-3">
						<div class="flex flex-col sm:flex-row gap-3 items-end">
							<div class="flex-1 w-full">
								<Select
									id="timezone"
									label="Timezone"
									options={timezoneOptions}
									bind:value={selectedTimezone}
									disabled={isSaving}
									className="w-full"
								/>
							</div>
							<Button
								onClick={handleTimezoneChange}
								disabled={isSaving || !hasTimezoneChanged}
								className="w-full sm:w-auto inline-flex items-center gap-2 whitespace-nowrap"
							>
								{#if isSaving}
									<LucideLoader2 class="w-4 h-4 animate-spin" />
									<span>Saving…</span>
								{:else}
									<span>Save</span>
								{/if}
							</Button>
						</div>
						{#if timezoneError}
							<p class="text-sm text-red">{timezoneError}</p>
						{/if}
						{#if timezoneSuccess}
							<p class="text-sm text-green">Settings saved!</p>
						{/if}
					</div>
				</div>
			</Container>

			<!-- Setup stuff -->
			<Container className="mt-4">
				<SectionTitle level="h2" className="mb-3">Setup</SectionTitle>
				<div class="space-y-4">
					<div class="flex flex-col gap-3">
						<div class="flex flex-wrap items-center justify-between gap-3">
							<label for="setup-content" class="text-sm font-medium text-text">
								{#if setupVariant === 'unix'}
									Run this command on macOS or Linux:
								{:else if setupVariant === 'windows'}
									Run this command in PowerShell:
								{:else}
									Copy this into your <code class="bg-surface0 p-1">~/.wakatime.cfg</code> file:
								{/if}
							</label>
							<SegmentedControl options={setupVariantOptions} bind:selected={setupVariant} />
						</div>
						{#if setupVariant === 'custom'}
							<div class="relative w-full">
								<div
									id="setup-content"
									class="resize-none text-text block w-full pr-14 px-2 py-2 border border-surface1 rounded-md bg-base/40 text-sm font-mono"
								>
									<p class="text-mauve">[<span class="text-text">settings</span>]</p>
									<p class="text-blue">
										api_url <span class="text-text">=</span>
										<span class="text-green">"{PUBLIC_BACKEND_API_URL}/api/v1"</span>
									</p>
									<p class="text-blue">
										api_key <span class="text-text">=</span>
										<span class="text-{settingsData.api_key ? 'yellow' : 'red'}"
											>{settingsData.api_key ?? 'REDACTED'}</span
										>
									</p>
								</div>
								<CopyButton text={config} className="absolute top-2 right-2" />
							</div>
						{:else}
							<CodeBlock code={setupVariant === 'unix' ? unixCommand() : windowsCommand()} />
						{/if}
					</div>
				</div>
			</Container>

			<!-- Timesplit -->
			<Container className="mt-4">
				<SectionTitle level="h2" className="mb-3">Timesplit</SectionTitle>
				<div class="space-y-4">
					<p class="text-text">
						Want to use multiple WakaTime servers at once? Check out
						<a
							href="https://github.com/ImShyMike/timesplit"
							target="_blank"
							rel="noopener noreferrer"
							data-umami-event="timesplit-link"
							data-umami-event-name="Timesplit"
							class="text-blue underline hover:text-blue/80">Timesplit</a
						>!
						<br />
						Or simply use the quick install command for your operating system below.
					</p>
					<!-- OS selector + install command -->
					<div class="flex flex-col gap-3 mt-3">
						<div class="flex flex-wrap items-center justify-between gap-3">
							<label for="timesplit-command" class="text-sm font-medium text-text">
								Quick install command for your OS:
							</label>
							<SegmentedControl options={osOptions} bind:selected={os} />
						</div>

						<CodeBlock code={installCommands[os]} />

						{#if os === 'windows'}
							<p class="mt-2 text-xs text-text/80">
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
				<p class="text-text mb-4">
					Here you can add aliases for projects. For example, adding "myapp" with "myapp-backend" as
					an alias will combine them into the "myapp" project.
				</p>

				{#if aliases && projects}
					<!-- Add new alias form -->
					<div class="bg-base/40 border border-surface1 rounded-lg p-4 mb-4">
						<h3 class="text-sm font-semibold text-text mb-3">Add New Alias</h3>
						<div class="flex flex-col sm:flex-row gap-3">
							<div class="flex-1">
								<Select
									id="main-project"
									label="Main Project"
									options={projectOptions}
									bind:value={selectedMainProject}
									placeholder="Select main project..."
									className="w-full"
								/>
							</div>
							<div class="flex-1">
								<Select
									id="alias-project"
									label="Alias"
									options={projectOptions}
									bind:value={selectedAliasProject}
									placeholder="Select alias project..."
									className="w-full"
								/>
							</div>
							<div class="flex items-end">
								<Button
									onClick={handleAddAlias}
									disabled={!selectedMainProject || !selectedAliasProject || isAddingAlias}
									className="w-full sm:w-auto inline-flex items-center gap-2 whitespace-nowrap"
								>
									Add
								</Button>
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
										<div class="bg-base/40 border border-surface1 rounded-lg p-3">
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
															class="flex items-center justify-between bg-surface0/40 border border-surface1 rounded px-3 py-2"
														>
															<span class="text-sm text-text">{safeText(aliasProject.name)}</span>
															<IconButton
																variant="danger"
																size="sm"
																title="Remove alias"
																onclick={() => handleDeleteAlias(aliasRecord.id)}
															>
																<LucideTrash2 class="w-4 h-4" />
															</IconButton>
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
		{:else if selectedTab === 'migration'}
			<Container>
				<SectionTitle level="h2" className="mb-3">Hackatime Import</SectionTitle>
				<div class="space-y-4">
					<p class="text-text">
						Import your existing Hackatime heartbeats directly into rustytime. Provide a <a
							class="text-blue underline hover:text-blue/80"
							href="https://hackatime.hackclub.com/my/wakatime_setup"
							target="_blank"
							rel="noopener noreferrer external">Hackatime API key</a
						> to begin importing. Your key is only used for this session and is not stored.
					</p>
					<div class="bg-base/40 border border-surface1 rounded-lg p-4 space-y-3">
						<h3 class="text-sm font-semibold text-text mb-3">Hackatime API Key</h3>
						<div class="flex flex-col sm:flex-row gap-3 items-center">
							<TextInput
								id="hackatime-api-key"
								type="password"
								placeholder="XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
								bind:value={hackatimeApiKey}
								disabled={isImportActive}
								className="w-full"
							/>
							<Button
								onClick={handleHackatimeImport}
								disabled={isStartingImport || isImportActive || !isValidHackatimeApiKey}
								className="w-full sm:w-auto inline-flex items-center gap-2 whitespace-nowrap"
							>
								{#if isStartingImport}
									<LucideLoader2 class="w-4 h-4 animate-spin" />
									<span>Starting…</span>
								{:else}
									<span>Start Import</span>
								{/if}
							</Button>
						</div>
						{#if importError}
							<p class="text-sm text-red">{importError}</p>
						{/if}
					</div>

					{#if isImportActive}
						<div class="bg-base/40 border border-blue/50 rounded-lg p-4 space-y-3">
							<div class="flex items-center gap-2">
								<LucideLoader2 class="w-5 h-5 animate-spin text-blue" />
								<h3 class="text-sm font-semibold text-text">
									Import {importStatus?.status === 'pending' ? 'Pending' : 'In Progress'}...
								</h3>
							</div>
							<p class="text-sm text-subtext0">
								Your import is running... You can leave this page and come back later to check the
								status.
							</p>
						</div>
					{/if}

					{#if importStatus && importStatus.status === 'completed'}
						<div class="bg-base/40 border border-green/50 rounded-lg p-4 space-y-4">
							<div class="flex items-center justify-between">
								<h3 class="text-sm font-semibold text-green">Import Completed!</h3>
								<p class="text-xs text-subtext0">
									Started at {formatDate(importStatus.created_at)}
								</p>
							</div>
							<div class="grid gap-3 sm:grid-cols-2">
								<div class="rounded-md border border-surface1 bg-surface0/30 p-3">
									<p class="text-xs text-subtext0 uppercase tracking-wide">Imported Heartbeats</p>
									<p class="text-2xl font-semibold text-green">
										{(importStatus.imported_count ?? 0).toLocaleString()}
									</p>
								</div>
								<div class="rounded-md border border-surface1 bg-surface0/30 p-3">
									<p class="text-xs text-subtext0 uppercase tracking-wide">Processed Heartbeats</p>
									<p class="text-2xl font-semibold text-text">
										{(importStatus.processed_count ?? 0).toLocaleString()}
									</p>
								</div>
								<div class="rounded-md border border-surface1 bg-surface0/30 p-3">
									<p class="text-xs text-subtext0 uppercase tracking-wide">API Requests</p>
									<p class="text-2xl font-semibold text-text">
										{(importStatus.request_count ?? 0).toLocaleString()}
									</p>
								</div>
								<div class="rounded-md border border-surface1 bg-surface0/30 p-3">
									<p class="text-xs text-subtext0 uppercase tracking-wide">Duration</p>
									<p class="text-2xl font-semibold text-text">
										{formatDuration(importStatus.time_taken ?? 0)}
									</p>
								</div>
							</div>
						</div>
					{/if}

					{#if importStatus && importStatus.status === 'failed'}
						<div class="bg-base/40 border border-red/50 rounded-lg p-4 space-y-3">
							<h3 class="text-sm font-semibold text-red">Import Failed</h3>
							<p class="text-sm text-subtext0">
								{importStatus.error_message || 'An unknown error occurred during the import.'}
							</p>
						</div>
					{/if}
				</div>
			</Container>
		{/if}
	</PageScaffold>
{/if}
