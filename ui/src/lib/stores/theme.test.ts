import { describe, expect, it } from 'vitest';
import {
	createSystemPreferenceListener,
	DEFAULT_THEME_PREFERENCE,
	readThemePreference,
	resolveTheme,
	THEME_STORAGE_KEY,
	writeThemePreference
} from './theme';

function createStorage(): Storage {
	const data = new Map<string, string>();
	return {
		getItem: (key) => (data.has(key) ? data.get(key)! : null),
		setItem: (key, value) => {
			data.set(key, value);
		},
		removeItem: (key) => {
			data.delete(key);
		},
		clear: () => {
			data.clear();
		},
		key: (index) => Array.from(data.keys())[index] ?? null,
		get length() {
			return data.size;
		}
	} as Storage;
}

describe('theme preference resolution', () => {
	it('resolves explicit preferences without system value', () => {
		expect(resolveTheme('light', true)).toBe('light');
		expect(resolveTheme('dark', false)).toBe('dark');
	});

	it('resolves system preference using system match', () => {
		expect(resolveTheme('system', true)).toBe('dark');
		expect(resolveTheme('system', false)).toBe('light');
	});
});

describe('theme preference persistence', () => {
	it('reads the default when storage is empty', () => {
		const storage = createStorage();
		expect(readThemePreference(storage)).toBe(DEFAULT_THEME_PREFERENCE);
	});

	it('writes and reads persisted preference', () => {
		const storage = createStorage();
		writeThemePreference(storage, 'dark');
		expect(storage.getItem(THEME_STORAGE_KEY)).toBe('dark');
		expect(readThemePreference(storage)).toBe('dark');
	});

	it('falls back to default on invalid values', () => {
		const storage = createStorage();
		storage.setItem(THEME_STORAGE_KEY, 'invalid');
		expect(readThemePreference(storage)).toBe(DEFAULT_THEME_PREFERENCE);
	});
});

describe('system preference listener', () => {
	it('notifies on initial and change events', () => {
		const listeners = new Set<(event: MediaQueryListEvent) => void>();
		const state = { matches: false };
		const media = {
			get matches() { return state.matches; },
			addEventListener: (_: 'change', listener: (event: MediaQueryListEvent) => void) => {
				listeners.add(listener);
			},
			removeEventListener: (_: 'change', listener: (event: MediaQueryListEvent) => void) => {
				listeners.delete(listener);
			}
		} as MediaQueryList;

		const seen: boolean[] = [];
		const stop = createSystemPreferenceListener(media, (prefersDark) => {
			seen.push(prefersDark);
		});

		expect(seen).toEqual([false]);
		state.matches = true;
		listeners.forEach((listener) => listener({ matches: true } as MediaQueryListEvent));
		expect(seen).toEqual([false, true]);

		stop();
		state.matches = false;
		listeners.forEach((listener) => listener({ matches: false } as MediaQueryListEvent));
		expect(seen).toEqual([false, true]);
	});
});
