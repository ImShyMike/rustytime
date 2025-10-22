import { browser } from '$app/environment';
import { writable } from 'svelte/store';

type Theme = 'light' | 'dark';

interface SetThemeOptions {
	persist?: boolean;
	manual?: boolean;
}

const themeStore = writable<Theme>('light');
let currentTheme: Theme = 'light';
let initialized = false;
let hasManualPreference = false;
let mediaQuery: MediaQueryList | null = null;

themeStore.subscribe((value) => {
	currentTheme = value;
});

function applyTheme(theme: Theme) {
	if (!browser) return;
	document.documentElement.classList.toggle('mocha', theme === 'dark');
	document.documentElement.dataset.theme = theme;
}

function setTheme(theme: Theme, options: SetThemeOptions = {}) {
	const { persist = true, manual = persist } = options;

	hasManualPreference = manual;
	applyTheme(theme);
	themeStore.set(theme);

	if (browser && persist) {
		localStorage.setItem('theme', theme);
	}
}

function handleMediaChange(event: MediaQueryListEvent) {
	if (hasManualPreference) {
		return;
	}

	const theme: Theme = event.matches ? 'dark' : 'light';
	setTheme(theme, { persist: false, manual: false });
}

function handleStorage(event: StorageEvent) {
	if (event.key !== 'theme') {
		return;
	}

	if (event.newValue === 'dark' || event.newValue === 'light') {
		setTheme(event.newValue, { persist: false, manual: true });
	} else if (event.newValue === null) {
		hasManualPreference = false;
		const theme = mediaQuery?.matches ? 'dark' : 'light';
		setTheme(theme as Theme, { persist: false, manual: false });
	}
}

export function initializeTheme() {
	if (initialized || !browser) {
		return;
	}

	let theme: Theme = 'light';
	const stored = localStorage.getItem('theme');

	if (stored === 'dark' || stored === 'light') {
		theme = stored;
		hasManualPreference = true;
	} else {
		mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		hasManualPreference = false;
		theme = mediaQuery.matches ? 'dark' : 'light';
	}

	setTheme(theme, { persist: hasManualPreference, manual: hasManualPreference });

	if (!mediaQuery) {
		mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	}

	mediaQuery.addEventListener('change', handleMediaChange);
	window.addEventListener('storage', handleStorage);
	initialized = true;
}

export function toggleTheme() {
	const next: Theme = currentTheme === 'light' ? 'dark' : 'light';
	setTheme(next, { persist: true, manual: true });
}

export function setThemeFromOutside(theme: Theme) {
	setTheme(theme, { persist: true, manual: true });
}

export const theme = {
	subscribe: themeStore.subscribe
};

export function getCurrentTheme(): Theme {
	return currentTheme;
}

export type { Theme };
