<script lang="ts">
	import { Modal } from '$lib';
	import type { Project } from '$lib/types/projects';
	import { auth } from '$lib/stores/auth';
	import { createApi, ApiError } from '$lib/api/api';
	import { setRepoUrl } from '$lib/api/project';

	const props = $props<{
		project: Project;
		onclose?: () => void;
		onsuccess?: (updatedRepoUrl: string | null) => void;
	}>();

	const api = createApi(fetch);

	let projectRepo = $state(props.project.repo_url ?? '');
	let isOpen = $state(true);
	let username = $state($auth.user?.name ?? 'user');
	let isSaving = $state(false);
	let errorMessage = $state<string | null>(null);

	function closeModal() {
		props.onclose?.();
	}

	async function saveChanges() {
		if (isSaving) return;

		isSaving = true;
		errorMessage = null;

		try {
			const newRepoUrl = projectRepo.trim() === '' ? null : projectRepo.trim();
			await setRepoUrl(api, props.project.id, newRepoUrl);

			props.project.repo_url = newRepoUrl;
			props.onsuccess?.(newRepoUrl);

			props.onclose?.();
		} catch (error) {
			if (error instanceof ApiError) {
				if (error.status === 400) {
					errorMessage = `Error: ${error.message}`;
				} else {
					errorMessage = `Failed to save: ${error.message}`;
				}
			} else if (error instanceof Error) {
				errorMessage = `Failed to save: ${error.message}`;
			} else {
				errorMessage = 'Failed to save changes. Please try again.';
			}
		} finally {
			isSaving = false;
		}
	}
</script>

<Modal bind:isOpen onclose={closeModal} title="Edit Project">
	<div class="space-y-6">
		<div>
			<label
				for="project-repo"
				class="mb-1 block text-sm font-medium text-ctp-subtext0 dark:text-ctp-subtext1"
			>
				Project Repository URL
			</label>
			<input
				id="project-repo"
				type="text"
				class="w-full rounded-md border border-ctp-subtext1 bg-ctp-surface0 px-3 py-2 text-sm text-ctp-text focus:outline-none focus:ring-2 focus:ring-ctp-lavender disabled:opacity-50 disabled:cursor-not-allowed"
				bind:value={projectRepo}
				placeholder={`https://github.com/${username}/...`}
				disabled={isSaving}
			/>
		</div>

		{#if errorMessage}
			<div class="rounded-md bg-ctp-red/10 border border-ctp-red/30 px-4 py-3">
				<p class="text-sm text-ctp-red">{errorMessage}</p>
			</div>
		{/if}
	</div>

	{#snippet footer()}
		<div class="flex justify-end space-x-2">
			<button
				onclick={saveChanges}
				disabled={isSaving}
				class="cursor-pointer inline-block rounded-lg bg-ctp-green-400 px-3 py-1 font-semibold text-ctp-base hover:bg-ctp-green-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-ctp-green-400"
			>
				{isSaving ? 'Saving...' : 'Save'}
			</button>
		</div>
	{/snippet}
</Modal>
