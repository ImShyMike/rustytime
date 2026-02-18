<script lang="ts">
	export let className = '';
</script>

<div class={`bg-mantle rounded-xl shadow p-4 border border-surface0/40 ${className}`.trim()}>
	<slot />
</div>
