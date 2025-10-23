<script lang="ts">
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
		maskIconColor?: string | null;
		appleIcon?: string | null;
		favicon32?: string | null;
		favicon32Type?: string | null;
		favicon16?: string | null;
		favicon16Type?: string | null;
		children?: () => unknown;
	}>();
	const renderChildren = props.children;

	const name = props.name ?? 'rustytime';
	const title = props.title ?? name;
	const description = props.description ?? null;
	const image = props.image ?? null;
	const color = props.color ?? '#b4befe';
	const manifest = props.manifest ?? '/manifest.webmanifest';
	const locale = props.locale ?? 'en_US';
	const twitterSite = props.twitterSite ?? '@rustytime';
	const url = props.url ?? null;
	const canonical = props.canonical ?? null;
	const imageAlt = props.imageAlt ?? null;
	const maskIconColor = props.maskIconColor ?? null;
	const appleIcon = props.appleIcon ?? '/pwa/apple-icon-180.png';
	const favicon32 = props.favicon32 ?? '/favicon.svg';
	const favicon32Type = props.favicon32Type ?? 'image/svg+xml';
	const favicon16 = props.favicon16 ?? '/favicon.ico';
	const favicon16Type = props.favicon16Type ?? 'image/x-icon';

	const makeTitle = (pageTitle: string, siteName: string) =>
		pageTitle === siteName ? pageTitle : `${pageTitle} - ${siteName}`;

	const computedTitle = $derived(makeTitle(title, name));
	const maskColor = $derived(maskIconColor ?? color);
</script>

<svelte:head>
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
		<meta name="twitter:card" content="summary_large_image" />
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
	{#if favicon32}
		{#if favicon32Type}
			<link rel="icon" type={favicon32Type} sizes="32x32" href={favicon32} />
		{:else}
			<link rel="icon" sizes="32x32" href={favicon32} />
		{/if}
	{/if}
	{#if favicon16}
		{#if favicon16Type}
			<link rel="icon" type={favicon16Type} sizes="16x16" href={favicon16} />
		{:else}
			<link rel="icon" sizes="16x16" href={favicon16} />
		{/if}
	{/if}
	{#if manifest}
		<link rel="manifest" href={manifest} />
	{/if}
</svelte:head>

{#if renderChildren}
	{@render renderChildren()}
{/if}
