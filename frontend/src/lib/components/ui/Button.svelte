<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger';
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
		primary:
			'bg-ctp-blue/10 text-ctp-crust hover:bg-ctp-blue/90 border-transparent focus-visible:ring-ctp-blue/60',
		secondary:
			'bg-ctp-surface0/10 text-ctp-text hover:bg-ctp-surface1 border-ctp-surface1 focus-visible:ring-ctp-surface1/60',
		outline:
			'bg-ctp-blue/10 text-ctp-blue hover:bg-ctp-blue/10 border-ctp-blue focus-visible:ring-ctp-blue/60',
		ghost:
			'bg-transparent text-ctp-text hover:bg-ctp-surface0/60 border-transparent focus-visible:ring-ctp-surface1/60',
		danger:
			'bg-ctp-red/10 text-ctp-red hover:bg-ctp-red/10 border-ctp-red focus-visible:ring-ctp-red/60'
	};

	const sizeClasses: Record<Size, string> = {
		sm: 'px-3 py-1.5 text-xs',
		md: 'px-4 py-2 text-sm',
		lg: 'px-5 py-2.5 text-base'
	};

	const baseClasses =
		'inline-flex items-center justify-center gap-2 rounded-lg border font-semibold transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:border-ctp-surface1 disabled:bg-ctp-surface0/40 disabled:text-ctp-subtext1';
</script>

<button
	{type}
	class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {className}"
	onclick={onClick}
	{disabled}
>
	{@render children?.()}
</button>
