<script lang="ts">
	import LucideCopy from '~icons/lucide/copy';
	import LucideCopyCheck from '~icons/lucide/copy-check';

	interface Props {
		text: string;
		className?: string;
		size?: 'sm' | 'md' | 'lg';
	}

	let { text, className = '', size = 'md' }: Props = $props();

	let copied = $state(false);

	const sizeClasses = {
		sm: 'h-6 px-1.5',
		md: 'h-8 px-2',
		lg: 'h-10 px-3'
	};

	const iconSizeClasses = {
		sm: 'w-3 h-3',
		md: 'w-4 h-4',
		lg: 'w-5 h-5'
	};

	function handleCopy() {
		navigator.clipboard.writeText(text).then(() => {
			copied = true;
			setTimeout(() => (copied = false), 2000);
		});
	}
</script>

<button
	type="button"
	onclick={handleCopy}
	aria-label="Copy to clipboard"
	class="cursor-pointer text-base text-sm rounded transition-all duration-200 flex items-center gap-2 hover:scale-105 active:scale-100 {copied
		? 'bg-green hover:bg-green/90'
		: 'bg-blue/70 hover:bg-blue'} {sizeClasses[size]} {className}"
>
	{#if copied}
		<LucideCopyCheck class={iconSizeClasses[size]} />
	{:else}
		<LucideCopy class={iconSizeClasses[size]} />
	{/if}
</button>
