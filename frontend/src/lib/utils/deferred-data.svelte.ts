import { untrack } from 'svelte';

const RELOAD_SKELETON_DELAY_MS = 150;

export interface DeferredDataState<T> {
	readonly data: T | null;
	readonly showSkeleton: boolean;
	readonly loadError: boolean;
}

export function createDeferredData<T>(getPromise: () => Promise<T>): DeferredDataState<T> {
	let data = $state<T | null>(null);
	let showSkeleton = $state(true);
	let loadError = $state(false);

	$effect(() => {
		const promise = getPromise();
		let cancelled = false;
		let timer: ReturnType<typeof setTimeout> | undefined;

		untrack(() => {
			loadError = false;

			if (!data) {
				showSkeleton = true;
			} else {
				timer = setTimeout(() => {
					if (!cancelled) showSkeleton = true;
				}, RELOAD_SKELETON_DELAY_MS);
			}
		});

		promise
			.then((result) => {
				if (!cancelled) {
					clearTimeout(timer);
					data = result;
					showSkeleton = false;
				}
			})
			.catch(() => {
				clearTimeout(timer);
				if (!cancelled && !data) loadError = true;
				showSkeleton = false;
			});

		return () => {
			cancelled = true;
			clearTimeout(timer);
		};
	});

	return {
		get data() {
			return data;
		},
		get showSkeleton() {
			return showSkeleton;
		},
		get loadError() {
			return loadError;
		}
	};
}
