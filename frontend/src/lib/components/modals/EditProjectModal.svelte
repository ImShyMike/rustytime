<script lang="ts">
	import { Modal, Button, TextInput } from '$lib';
	import type { Project } from '$lib/types/projects';
	import { page } from '$app/state';
	import { createApi, ApiError } from '$lib/api/api';
	import { updateProject } from '$lib/api/project';

	const props = $props<{
		project: Project;
		onclose?: () => void;
		onsuccess?: (updatedProjectUrl: string | null) => void;
	}>();

	const api = createApi(fetch);

	const originalProjectUrl = props.project.project_url ?? '';
	let projectUrl = $state(originalProjectUrl);
	let isOpen = $state(true);
	let username = $state(page.data.auth?.user?.name ?? 'user');
	let isSaving = $state(false);
	let errorMessage = $state<string | null>(null);

	const hasChanges = $derived(projectUrl.trim() !== originalProjectUrl);

	function closeModal() {
		props.onclose?.();
	}

	async function saveChanges() {
		if (isSaving) return;

		const newProjectUrl = projectUrl.trim();

		if (!hasChanges) {
			props.onclose?.();
			return;
		}

		isSaving = true;
		errorMessage = null;

		try {
			await updateProject(api, props.project.id, { project_url: newProjectUrl });

			props.project.project_url = newProjectUrl === '' ? null : newProjectUrl;
			props.onsuccess?.(newProjectUrl === '' ? null : newProjectUrl);

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
			id="project-url"
			type="text"
			label="Project URL"
			bind:value={projectUrl}
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
