<script lang="ts">
	import { theme } from '$lib/stores/theme';

	const props = $props<{
		name?: string;
		title?: string;
		description?: string | null;
		image?: string | null;
		color?: string;
		manifest?: string | null;
		locale?: string;
		twitterSite?: string | null;
		url?: string | null;
		canonical?: string | null;
		imageAlt?: string | null;
		appleIcon?: string | null;
		children?: () => unknown;
	}>();
	const renderChildren = props.children;

	const name = props.name ?? 'rustytime';
	const title = props.title ?? name;
	const description = props.description ?? null;
	const image = props.image ?? null;
	const color = $derived(props.color ?? ($theme === 'dark' ? '#b4befe' : '#7287fd'));
	const manifest = props.manifest ?? '/manifest.webmanifest';
	const locale = props.locale ?? 'en_US';
	const twitterSite = props.twitterSite ?? '@rustytime';
	const url = props.url ?? null;
	const canonical = props.canonical ?? null;
	const imageAlt = props.imageAlt ?? null;
	const appleIcon = props.appleIcon ?? '/pwa/apple-icon-180.png';

	const makeTitle = (pageTitle: string, siteName: string) =>
		pageTitle === siteName ? pageTitle : `${pageTitle} - ${siteName}`;

	const computedTitle = $derived(makeTitle(title, name));
</script>

<svelte:head>
	<title>{name}</title>
	<meta property="og:locale" content={locale} />
	<meta property="og:type" content="website" />
	<meta property="og:site_name" content={name} />
	<link rel="canonical" href={canonical} />
	{#if twitterSite}
		<meta name="twitter:site" content={twitterSite} />
	{/if}
	{#if url}
		<meta property="og:url" content={url} />
	{/if}
	<meta property="og:title" content={computedTitle} />
	<meta name="application-name" content={name} />
	<meta name="twitter:title" content={computedTitle} />
	{#if description}
		<meta name="description" content={description} />
		<meta property="og:description" content={description} />
		<meta name="twitter:description" content={description} />
	{/if}
	{#if image}
		<meta property="og:image" content={image} />
		<meta name="twitter:card" content="summary" />
		<meta name="twitter:image" content={image} />
		{#if imageAlt}
			<meta property="og:image:alt" content={imageAlt} />
			<meta name="twitter:image:alt" content={imageAlt} />
		{/if}
	{:else}
		<meta name="twitter:card" content="summary" />
	{/if}
	<meta name="theme-color" content={color} />
	<meta name="color-scheme" content="dark light" />
	<meta name="msapplication-TileColor" content={color} />
	<meta name="apple-mobile-web-app-capable" content="yes" />
	<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
	<meta name="apple-mobile-web-app-title" content={name} />
	{#if appleIcon}
		<link rel="apple-touch-icon" sizes="180x180" href={appleIcon} />
	{/if}
	{#if manifest}
		<link rel="manifest" href={manifest} />
	{/if}
</svelte:head>

{#if renderChildren}
	{@render renderChildren()}
{/if}
