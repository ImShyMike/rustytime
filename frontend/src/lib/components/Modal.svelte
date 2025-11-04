<script lang="ts">
	import { fly, fade } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';
	import LucideX from '~icons/lucide/x';
	import type { Snippet } from 'svelte';

	interface Props {
		isOpen: boolean;
		title: string;
		onclose?: () => void;
		children?: Snippet;
		footer?: Snippet;
	}

	let { isOpen = $bindable(false), title, onclose, children, footer }: Props = $props();

	function handleClose() {
		isOpen = false;
		onclose?.();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			handleClose();
		}
	}

	function handleBackdropKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && e.target === e.currentTarget && isOpen) {
			handleClose();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && isOpen) {
			handleClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center p-4"
		transition:fade={{ duration: 200 }}
		onclick={handleBackdropClick}
		onkeydown={handleBackdropKeydown}
		role="dialog"
		tabindex="-1"
		aria-modal="true"
		aria-labelledby="modal-title"
	>
		<!-- Backdrop -->
		<div
			class="absolute inset-0 bg-crust/30 backdrop-blur-sm"
			onclick={handleClose}
			onkeydown={handleKeydown}
			role="button"
			tabindex="0"
			aria-label="Close modal backdrop"
		></div>

		<!-- Modal -->
		<div
			class="relative w-full max-w-md bg-ctp-base border border-ctp-surface0/40 rounded-xl shadow p-0"
			transition:fly={{ y: 20, duration: 300, easing: quintOut }}
		>
			<!-- Header -->
			<div class="flex items-center justify-between p-4 border-b border-ctp-surface0">
				<h2 id="modal-title" class="text-lg font-semibold text-ctp-text">
					{title}
				</h2>
				<button
					onclick={handleClose}
					class="cursor-pointer text-ctp-subtext0 hover:text-ctp-text"
					aria-label="Close modal"
				>
					<LucideX class="h-6 w-6" />
				</button>
			</div>

			<!-- Content -->
			<div class="p-4">
				{@render children?.()}
			</div>

			<!-- Footer -->
			{#if footer}
				<div class="p-4 border-t border-ctp-surface0">
					{@render footer()}
				</div>
			{/if}
		</div>
	</div>
{/if}
