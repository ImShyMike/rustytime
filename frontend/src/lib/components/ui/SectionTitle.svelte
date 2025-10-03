<script lang="ts">
	type HeadingLevel = 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
	type SizeVariant = 'sm' | 'md' | 'lg';

	export let level: HeadingLevel = 'h3';
	export let size: SizeVariant = 'md';
	export let className = '';

	const sizeClasses: Record<SizeVariant, string> = {
		sm: 'text-lg',
		md: 'text-xl',
		lg: 'text-2xl'
	};

	$: computedClass = `${sizeClasses[size]} font-semibold text-ctp-text ${className}`.trim();
</script>

<svelte:element this={level} class={computedClass}>
	<slot />
</svelte:element>
