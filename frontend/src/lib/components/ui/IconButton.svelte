<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'ghost' | 'danger';
	type Size = 'sm' | 'md' | 'lg' | 'xl';

	interface Props {
		variant?: Variant;
		size?: Size;
		disabled?: boolean;
		title?: string;
		className?: string;
		onclick?: () => void;
		children?: Snippet;
	}

	let {
		variant = 'default',
		size = 'md',
		disabled = false,
		title = '',
		className = '',
		onclick,
		children
	}: Props = $props();

	const variantClasses: Record<Variant, string> = {
		default:
			'border border-ctp-surface1 text-ctp-subtext1 hover:text-ctp-text hover:bg-ctp-surface0/60',
		ghost: 'text-ctp-subtext1 hover:text-ctp-text hover:bg-ctp-surface0/40',
		danger: 'text-ctp-red hover:text-ctp-red/80 hover:bg-ctp-red/10'
	};

	const sizeClasses: Record<Size, string> = {
		sm: 'h-7 w-7',
		md: 'h-8 w-8',
		lg: 'h-10 w-10',
		xl: 'h-12 w-12'
	};
</script>

<button
	type="button"
	class="inline-flex cursor-pointer items-center justify-center rounded-lg transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ctp-blue/60 disabled:cursor-not-allowed disabled:opacity-50 {variantClasses[
		variant
	]} {sizeClasses[size]} {className}"
	{disabled}
	{title}
	{onclick}
	aria-label={title}
>
	{@render children?.()}
</button>
