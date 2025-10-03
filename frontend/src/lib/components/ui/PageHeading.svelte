<script lang="ts">
	export let title: string = '';
	export let className = '';
</script>

<h1
	class={`text-3xl font-bold text-ctp-mauve mb-6 flex items-center gap-2 justify-end text-right md:justify-start md:text-left pr-4 md:pr-0 ${className}`.trim()}
>
	<slot>{title}</slot>
</h1>
