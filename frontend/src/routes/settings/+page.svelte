<script lang="ts">
	import type { PageData } from './$types';
	import { Container, PageScaffold, SectionTitle } from '$lib';
	import LucideCopy from '~icons/lucide/copy';
	import LucideCopyCheck from '~icons/lucide/copy-check';
	import { onMount } from 'svelte';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let settingsData = $state(data);

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
							class="resize-none text-text block w-full pr-14 px-2 py-2 border border-ctp-surface1 rounded-md bg-ctp-surface0/70 text-sm font-mono"
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
						<div class="inline-flex rounded-md bg-ctp-surface0 p-1">
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
								class="text-text resize-none block w-full pr-14 px-3 py-3 border border-ctp-surface1 rounded-md bg-ctp-surface0/70 text-sm font-mono whitespace-pre-wrap">{installCommands[
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
	</PageScaffold>
{/if}
