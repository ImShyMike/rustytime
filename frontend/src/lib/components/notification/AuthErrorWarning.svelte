<script lang="ts">
	import { auth } from '../../stores/auth.js';
	import type { AuthError } from '../../stores/auth.js';

	import LucideTriangleAlert from '~icons/lucide/triangle-alert';
	import LucideCircleAlert from '~icons/lucide/circle-alert';
	import LucideWifiOff from '~icons/lucide/wifi-off';
	import LucideX from '~icons/lucide/x';

	export let showRetryButton = true;

	function getErrorDisplayMessage(error: AuthError): string {
		switch (error.type) {
			case 'unauthorized':
				return 'Your session has expired. Please log in again.';
			case 'network':
				return 'Network connection issue. Please check your connection and try again.';
			case 'server':
				return 'Server is temporarily unavailable. Please try again later.';
			case 'unknown':
			default:
				return 'An unexpected error occurred. Please try again.';
		}
	}

	function getErrorVariant(error: AuthError): 'destructive' | 'warning' | 'default' {
		switch (error.type) {
			case 'unauthorized':
			case 'server':
				return 'destructive';
			case 'network':
				return 'warning';
			case 'unknown':
			default:
				return 'default';
		}
	}

	function getVariantClasses(variant: 'destructive' | 'warning' | 'default'): string {
		switch (variant) {
			case 'destructive':
				return 'border-red text-red';
			case 'warning':
				return 'border-yellow text-yellow';
			default:
				return 'border-border text-text';
		}
	}

	function getTitleVariant(error: AuthError): string {
		switch (error.type) {
			case 'unauthorized':
				return 'Session Expired';
			case 'network':
				return 'Connection Error';
			case 'server':
				return 'Server Error';
			case 'unknown':
			default:
				return 'Error';
		}
	}

	function getIconVariant(error: AuthError) {
		switch (error.type) {
			case 'unauthorized':
				return LucideCircleAlert;
			case 'network':
				return LucideWifiOff;
			case 'server':
				return LucideTriangleAlert;
			case 'unknown':
			default:
				return LucideTriangleAlert;
		}
	}
</script>

{#if $auth.error}
	<div class="fixed top-4 right-4 z-50 w-full max-w-sm">
		<div
			class="bg-base relative rounded-lg border p-4 shadow-lg backdrop-blur supports-[backdrop-filter]:bg-background/60 text-foreground {getVariantClasses(
				getErrorVariant($auth.error)
			)}"
			role="alert"
			aria-live="assertive"
		>
			<div class="flex items-start gap-3">
				<div class="flex-shrink-0 mt-0.5">
					<svelte:component this={getIconVariant($auth.error)} class="h-6 w-6" />
				</div>
				<div class="flex-1 space-y-1">
					<h5 class="text-sm font-medium leading-none tracking-tight text-text">
						{getTitleVariant($auth.error)}
					</h5>
					<p class="text-sm opacity-90 text-subtext0">
						{$auth.error.message ? $auth.error.message : getErrorDisplayMessage($auth.error)}
					</p>
					<div class="flex items-center justify-between">
						{#if $auth.error.type === 'unauthorized'}
							<button
								type="button"
								class="cursor-pointer bg-mantle border border-overlay0 inline-flex h-8 items-center justify-center rounded-md bg-primary px-3 text-xs font-medium text-primary-foreground ring-offset-background hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
								onclick={auth.login}
								disabled={$auth.isLoading}
							>
								Log in
							</button>
						{:else}
							{#if $auth.error}
								<p class="text-xs opacity-70">
									{$auth.error.timestamp.toLocaleTimeString()}
								</p>
							{/if}
							{#if showRetryButton}
								<button
									type="button"
									class="cursor-pointer inline-flex h-8 items-center justify-center rounded-md bg-primary px-3 text-xs font-medium text-primary-foreground ring-offset-background hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
									onclick={auth.retryVerification}
									disabled={$auth.isLoading}
								>
									{$auth.isLoading ? 'Retrying...' : 'Retry'}
								</button>
							{/if}
						{/if}
					</div>
				</div>
				<button
					type="button"
					class="cursor-pointer absolute right-2 top-2 rounded-md p-1 text-foreground/50 opacity-70 ring-offset-background hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
					onclick={auth.clearError}
					aria-label="Dismiss notification"
				>
					<LucideX class="h-4 w-4" />
				</button>
			</div>
		</div>
	</div>
{/if}
