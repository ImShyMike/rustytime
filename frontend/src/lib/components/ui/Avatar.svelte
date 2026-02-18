<script lang="ts">
	const props = $props<{
		url: string;
		size?: number;
		className?: string;
	}>();

	let size = $derived(() => props.size ?? 80);
	let className = $derived(
		() =>
			props.className ??
			'absolute inset-0 w-full h-full object-cover transition-opacity duration-200'
	);
	let avatarLoaded = $state(false);
</script>

<div
	class="rounded-full relative overflow-hidden border-2 border-ctp-green-500"
	style="width: {size()}px; height: {size()}px;"
>
	<img
		src={props.url}
		alt="Avatar"
		width={size()}
		height={size()}
		class={className()}
		class:opacity-0={!avatarLoaded}
		onload={() => (avatarLoaded = true)}
		decoding="async"
		loading="lazy"
	/>
	{#if !avatarLoaded}
		<div class="absolute inset-0 bg-surface0 animate-pulse"></div>
	{/if}
</div>
