<script lang="ts">
	export let admin_level: number;

	const ADMIN_LEVEL_MAP: Record<number, string[]> = {
		0: ['User', 'bg-ctp-overlay2 text-ctp-crust'],
		1: ['Admin', 'bg-ctp-red-400 text-ctp-crust'],
		2: ['Owner', 'bg-ctp-mauve-400 text-ctp-crust']
	};
</script>

<span
	class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {ADMIN_LEVEL_MAP[
		admin_level
	][1]} items-center h-6"
>
	{ADMIN_LEVEL_MAP[admin_level][0] || 'User'}
</span>
