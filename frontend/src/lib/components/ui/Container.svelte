<script lang="ts">
	export let className = '';
</script>

<div class={`bg-ctp-base rounded-xl shadow p-4 ${className}`.trim()}>
	<slot />
</div>
