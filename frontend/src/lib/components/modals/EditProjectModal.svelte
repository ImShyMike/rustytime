<script lang="ts">
	import { Modal, Button, TextInput } from '$lib';
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
		<TextInput
			id="project-repo"
			type="text"
			label="Project Repository URL"
			bind:value={projectRepo}
			placeholder={`https://github.com/${username}/...`}
			disabled={isSaving}
		/>

		{#if errorMessage}
			<div class="rounded-md bg-ctp-red/10 border border-ctp-red/30 px-4 py-3">
				<p class="text-sm text-ctp-red">{errorMessage}</p>
			</div>
		{/if}
	</div>

	{#snippet footer()}
		<div class="flex justify-end space-x-2">
			<Button variant="primary" size="sm" onClick={saveChanges} disabled={isSaving}>
				{isSaving ? 'Saving...' : 'Save'}
			</Button>
		</div>
	{/snippet}
</Modal>
