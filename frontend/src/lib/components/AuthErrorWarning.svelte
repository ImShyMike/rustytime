<script lang="ts">
	import { auth } from '../stores/auth.js';
	import type { AuthError } from '../stores/auth.js';

	import MaterialSymbolsWarningRounded from '~icons/material-symbols/warning-rounded';
	import MaterialSymbolsErrorRounded from '~icons/material-symbols/error-rounded';
	import MaterialSymbolsAndroidWifi3BarOffRounded from '~icons/material-symbols/android-wifi-3-bar-off-rounded';
	import MaterialSymbolsCloseRounded from '~icons/material-symbols/close-rounded';

	export let showRetryButton = true;

	let authState: typeof $auth;
	$: authState = $auth;

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
				return 'border-destructive/50 dark:border-destructive [&>svg]:text-destructive';
			case 'warning':
				return 'border-yellow-500/50 dark:border-yellow-500 [&>svg]:text-yellow-600';
			default:
				return 'border-border [&>svg]:text-foreground';
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
				return MaterialSymbolsErrorRounded;
			case 'network':
				return MaterialSymbolsAndroidWifi3BarOffRounded;
			case 'server':
				return MaterialSymbolsWarningRounded;
			case 'unknown':
			default:
				return MaterialSymbolsWarningRounded;
		}
	}
</script>

{#if authState.error}
	<div class="fixed top-4 right-4 z-50 w-full max-w-sm">
		<div
			class="relative rounded-lg border p-4 shadow-lg backdrop-blur supports-[backdrop-filter]:bg-background/60 text-foreground {getVariantClasses(
				getErrorVariant(authState.error)
			)}"
			role="alert"
			aria-live="assertive"
		>
			<div class="flex items-start gap-3">
				<div class="flex-shrink-0 mt-0.5">
					<svelte:component this={getIconVariant(authState.error)} class="h-6 w-6" />
				</div>
				<div class="flex-1 space-y-1">
					<h5 class="text-sm font-medium leading-none tracking-tight">
						{getTitleVariant(authState.error)}
					</h5>
					<p class="text-sm opacity-90">
						{getErrorDisplayMessage(authState.error)}
					</p>
					<div class="flex items-center justify-between">
						{#if authState.error.type !== 'unauthorized'}
							<p class="text-xs opacity-70">
								{authState.error.timestamp.toLocaleTimeString()}
							</p>
						{/if}
						{#if showRetryButton && authState.error.type !== 'unauthorized'}
							<button
								type="button"
								class="cursor-pointer inline-flex h-8 items-center justify-center rounded-md bg-primary px-3 text-xs font-medium text-primary-foreground ring-offset-background transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
								on:click={auth.retryVerification}
								disabled={authState.isLoading}
							>
								{authState.isLoading ? 'Retrying...' : 'Retry'}
							</button>
						{/if}
					</div>
				</div>
				<button
					type="button"
					class="cursor-pointer absolute right-2 top-2 rounded-md p-1 text-foreground/50 opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
					on:click={auth.clearError}
					aria-label="Dismiss notification"
				>
					<MaterialSymbolsCloseRounded class="h-4 w-4" />
				</button>
			</div>
		</div>
	</div>
{/if}
