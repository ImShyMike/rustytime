<script lang="ts">
	import RelativeTime from './ui/RelativeTime.svelte';

	export let lastUpdatedAt: Date | null = null;

	const commitHash = import.meta.env.VITE_GIT_COMMIT_HASH || 'unknown';
	const shortHash = commitHash.substring(0, 7);
	const repoUrl = 'https://github.com/ImShyMike/rustytime';
	const commitUrl = `${repoUrl}/commit/${commitHash}`;
</script>

<footer class="w-full pb-2 px-6 text-sm">
	<div
		class="group flex flex-col sm:flex-row items-center justify-center sm:gap-3 text-text/35 transition-colors hover:text-subtext1 text-center"
	>
		{#if lastUpdatedAt}
			<span class="font-mono text-balance" title={lastUpdatedAt.toLocaleString()}>
				data updated <RelativeTime datetime={lastUpdatedAt} />
			</span>
			<span class="hidden sm:inline">•</span>
		{/if}
		{#if commitHash !== 'dev'}
			<a
				href={commitUrl}
				target="_blank"
				rel="noopener noreferrer"
				class="font-mono"
				title="View commit on GitHub"
			>
				build: <span class="underline underline-offset-3 decoration-wavy">#{shortHash}</span>
			</a>
		{:else}
			<span class="font-mono"
				>build: <span class="underline underline-offset-3 decoration-wavy">#{shortHash}</span></span
			>
		{/if}
	</div>
</footer>
