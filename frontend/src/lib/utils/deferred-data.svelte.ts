import { navigating } from '$app/state';

const SKELETON_DELAY_MS = 150;

export function useNavigationSkeleton() {
	let showSkeleton = $state(false);
	let timer: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		clearTimeout(timer);
		if (navigating.to) {
			timer = setTimeout(() => {
				showSkeleton = true;
			}, SKELETON_DELAY_MS);
		} else {
			showSkeleton = false;
		}
	});

	return {
		get showSkeleton() {
			return showSkeleton;
		}
	};
}
