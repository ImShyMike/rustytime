<script lang="ts">
	export let is_admin;
</script>

<span
	class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {is_admin
		? 'bg-ctp-red-400 text-ctp-crust'
		: 'bg-ctp-overlay2 text-ctp-crust'} items-center h-6"
>
	{is_admin ? 'Admin' : 'User'}
</span>
