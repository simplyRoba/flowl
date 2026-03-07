import { derived, writable } from 'svelte/store';

export type ThemePreference = 'light' | 'dark' | 'system';
export type ThemeMode = 'light' | 'dark';

export const THEME_STORAGE_KEY = 'flowl.theme';
export const DEFAULT_THEME_PREFERENCE: ThemePreference = 'system';

export function isThemePreference(value: unknown): value is ThemePreference {
	return value === 'light' || value === 'dark' || value === 'system';
}

export function resolveTheme(
	preference: ThemePreference,
	prefersDark: boolean
): ThemeMode {
	if (preference === 'system') return prefersDark ? 'dark' : 'light';
	return preference;
}

export function readThemePreference(storage: Storage | null): ThemePreference {
	if (!storage) return DEFAULT_THEME_PREFERENCE;
	try {
		const stored = storage.getItem(THEME_STORAGE_KEY);
		return isThemePreference(stored) ? stored : DEFAULT_THEME_PREFERENCE;
	} catch {
		return DEFAULT_THEME_PREFERENCE;
	}
}

export function writeThemePreference(
	storage: Storage | null,
	preference: ThemePreference
): void {
	if (!storage) return;
	try {
		storage.setItem(THEME_STORAGE_KEY, preference);
	} catch {
		// Ignore storage write failures.
	}
}

export function createSystemPreferenceListener(
	media: MediaQueryList,
	onChange: (prefersDark: boolean) => void
): () => void {
	onChange(media.matches);
	const handler = (event: MediaQueryListEvent) => {
		onChange(event.matches);
	};
	media.addEventListener('change', handler);
	return () => media.removeEventListener('change', handler);
}

export const themePreference = writable<ThemePreference>(
	readThemePreference(getStorage())
);
export const systemPrefersDark = writable(false);
export const effectiveTheme = derived(
	[themePreference, systemPrefersDark],
	([$themePreference, $systemPrefersDark]) =>
		resolveTheme($themePreference, $systemPrefersDark)
);

function getStorage(): Storage | null {
	if (typeof window === 'undefined') return null;
	try {
		return window.localStorage;
	} catch {
		return null;
	}
}

const THEME_COLORS: Record<ThemeMode, string> = {
	light: '#FAF6F1',
	dark: '#1A1612'
};

function applyTheme(theme: ThemeMode): void {
	if (typeof document === 'undefined') return;
	document.documentElement.dataset.theme = theme;
	let meta = document.querySelector(
		'meta[name="theme-color"]'
	) as HTMLMetaElement | null;
	if (!meta) {
		meta = document.createElement('meta');
		meta.name = 'theme-color';
		document.head.appendChild(meta);
	}
	meta.content = THEME_COLORS[theme];
}

let initialized = false;
let cleanup: (() => void) | null = null;

export function initTheme(serverPreference?: ThemePreference): void {
	if (typeof window === 'undefined' || initialized) return;
	initialized = true;

	const storage = getStorage();
	const preference = serverPreference ?? readThemePreference(storage);
	themePreference.set(preference);
	if (serverPreference) writeThemePreference(storage, serverPreference);

	let stopSystemListener = () => {
		// no-op
	};
	if (window.matchMedia) {
		stopSystemListener = createSystemPreferenceListener(
			window.matchMedia('(prefers-color-scheme: dark)'),
			(prefersDark) => systemPrefersDark.set(prefersDark)
		);
	}

	const stopThemeSubscription = effectiveTheme.subscribe((theme) =>
		applyTheme(theme)
	);
	cleanup = () => {
		stopSystemListener();
		stopThemeSubscription();
	};
}

export function destroyTheme(): void {
	cleanup?.();
	cleanup = null;
	initialized = false;
}

export function setThemePreference(preference: ThemePreference): void {
	themePreference.set(preference);
	writeThemePreference(getStorage(), preference);
	import('$lib/api')
		.then(({ updateSettings }) => updateSettings({ theme: preference }))
		.catch(() => {});
}
