<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'primary' | 'secondary' | 'outline' | 'ghost' | 'confirm' | 'danger';
	type Size = 'sm' | 'md' | 'lg';

	interface Props {
		variant?: Variant;
		size?: Size;
		disabled?: boolean;
		type?: 'button' | 'submit' | 'reset';
		className?: string;
		onClick?: () => void;
		children?: Snippet;
	}

	let {
		variant = 'outline',
		size = 'md',
		disabled = false,
		type = 'button',
		className = '',
		onClick = () => {},
		children
	}: Props = $props();

	const variantClasses: Record<Variant, string> = {
		primary: 'bg-blue/10 text-text hover:bg-base border-surface0 focus-visible:ring-blue/60',
		secondary:
			'bg-base/10 text-text hover:bg-surface0 border-surface1 focus-visible:ring-surface1/60',
		outline: 'bg-blue/10 text-blue hover:bg-blue/10 border-blue focus-visible:ring-blue/60',
		ghost:
			'bg-transparent text-text hover:bg-base/60 border-transparent focus-visible:ring-surface1/60',
		confirm: 'bg-green/10 text-green hover:bg-green/10 border-green focus-visible:ring-green/60',
		danger: 'bg-red/10 text-red hover:bg-red/10 border-red focus-visible:ring-red/60'
	};

	const sizeClasses: Record<Size, string> = {
		sm: 'px-3 py-1.5 text-xs',
		md: 'px-4 py-2 text-sm',
		lg: 'px-5 py-2.5 text-base'
	};

	const baseClasses =
		'inline-flex items-center justify-center gap-2 rounded-lg border font-semibold transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:border-surface1 disabled:bg-base/40 disabled:text-subtext1';
</script>

<button
	{type}
	class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
	onclick={onClick}
	{disabled}
>
	{@render children?.()}
</button>
