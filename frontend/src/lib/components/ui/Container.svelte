<script lang="ts">
	export let className = '';
</script>

<div class={`bg-ctp-base rounded-xl shadow p-4 border border-ctp-surface0/40 ${className}`.trim()}>
	<slot />
</div>
