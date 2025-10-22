import { browser } from '$app/environment';
import { onMount } from 'svelte';

const DEFAULT_INTERVAL_MS = 5 * 60 * 1000;

export interface VisibilityRefreshOptions {
	refresh: () => Promise<void> | void;
	intervalMs?: number;
	onError?: (error: unknown) => void;
}

export interface VisibilityRefreshController {
	start: () => void;
	dispose: () => void;
	handleVisibilityChange: () => void;
	restart: () => void;
}

export function createVisibilityRefresh({
	refresh,
	intervalMs = DEFAULT_INTERVAL_MS,
	onError
}: VisibilityRefreshOptions): VisibilityRefreshController {
	let timeout: ReturnType<typeof setTimeout> | null = null;
	let nextDeadline: number | null = null;
	let disposed = false;
	let refreshInFlight = false;

	const clearScheduled = (resetDeadline = false) => {
		if (timeout !== null) {
			clearTimeout(timeout);
			timeout = null;
		}
		if (resetDeadline) {
			nextDeadline = null;
		}
	};

	const schedule = (delayMs = intervalMs) => {
		if (disposed) return;

		clearScheduled();

		if (typeof document === 'undefined' || document.hidden) {
			nextDeadline = Date.now() + delayMs;
			return;
		}

		const normalizedDelay = Math.max(0, delayMs);
		nextDeadline = Date.now() + normalizedDelay;
		timeout = setTimeout(async () => {
			timeout = null;

			if (disposed) {
				return;
			}

			if (typeof document === 'undefined' || document.hidden) {
				nextDeadline = Date.now() + intervalMs;
				return;
			}

			nextDeadline = null;
			await runRefresh();
			schedule();
		}, normalizedDelay);
	};

	const runRefresh = async () => {
		if (refreshInFlight || disposed) {
			return;
		}

		refreshInFlight = true;
		try {
			await refresh();
		} catch (error) {
			onError?.(error);
		} finally {
			refreshInFlight = false;
		}
	};

	const handleVisibilityChange = () => {
		if (disposed) return;
		if (typeof document === 'undefined') return;

		if (document.hidden) {
			clearScheduled();
			if (nextDeadline === null) {
				nextDeadline = Date.now() + intervalMs;
			}
			return;
		}

		const deadline = nextDeadline;
		const now = Date.now();

		if (deadline !== null && now >= deadline) {
			nextDeadline = null;
			void runRefresh().finally(() => {
				if (!disposed) {
					schedule();
				}
			});
			return;
		}

		const remaining = deadline !== null ? Math.max(0, deadline - now) : intervalMs;
		schedule(remaining);
	};

	const start = () => {
		if (disposed) return;
		if (typeof document === 'undefined') return;

		if (document.hidden) {
			nextDeadline = Date.now() + intervalMs;
			return;
		}

		schedule();
	};

	const dispose = () => {
		disposed = true;
		clearScheduled(true);
	};

	const restart = () => {
		if (disposed) return;
		clearScheduled(true);
		nextDeadline = null;
		schedule();
	};

	return {
		start,
		dispose,
		handleVisibilityChange,
		restart
	};
}

export interface SetupVisibilityRefreshOptions extends VisibilityRefreshOptions {
	onDispose?: () => void;
	autoStart?: boolean;
}

export function setupVisibilityRefresh({
	refresh,
	intervalMs,
	onError,
	onDispose,
	autoStart = true
}: SetupVisibilityRefreshOptions): void {
	onMount(() => {
		if (!browser || typeof document === 'undefined') {
			return () => {
				onDispose?.();
			};
		}

		const controller = createVisibilityRefresh({
			refresh,
			intervalMs,
			onError
		});

		if (autoStart) {
			controller.start();
		}

		const handleVisibilityChange = () => {
			controller.handleVisibilityChange();
		};

		document.addEventListener('visibilitychange', handleVisibilityChange);

		return () => {
			document.removeEventListener('visibilitychange', handleVisibilityChange);
			controller.dispose();
			onDispose?.();
		};
	});
}
