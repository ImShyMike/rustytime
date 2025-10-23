<script lang="ts">
	import type { PageData } from './$types';
	import { Container, PageScaffold, SectionTitle } from '$lib';
	import LucideCopy from '~icons/lucide/copy';
	import LucideCopyCheck from '~icons/lucide/copy-check';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();

	let settingsData = $state(data);

	let config: string = $state('');
	let copied: boolean = $state(false);

	function copySetup() {
		if (!config) return;
		navigator.clipboard.writeText(config).then(() => {
			copied = true;
			setTimeout(() => (copied = false), 2000);
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
						<textarea
							id="api-setup"
							readonly
							rows="3"
							class="resize-none text-text block w-full pr-14 px-2 py-2 border border-ctp-surface1 rounded-md bg-ctp-surface0/70 text-sm font-mono"
							>{config}</textarea
						>
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
	</PageScaffold>
{/if}
