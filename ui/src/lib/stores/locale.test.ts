import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import { de, en, es } from '$lib/i18n';
import {
	DEFAULT_LOCALE,
	LOCALE_STORAGE_KEY,
	locale,
	readLocale,
	translations,
	writeLocale
} from './locale';

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

beforeEach(() => {
	locale.set(DEFAULT_LOCALE);
});

describe('locale persistence', () => {
	it('reads the default when storage is empty', () => {
		const storage = createStorage();
		expect(readLocale(storage)).toBe(DEFAULT_LOCALE);
	});

	it('reads stored locale values', () => {
		const storage = createStorage();
		storage.setItem(LOCALE_STORAGE_KEY, 'de');
		expect(readLocale(storage)).toBe('de');
	});

	it('falls back to default on invalid values', () => {
		const storage = createStorage();
		storage.setItem(LOCALE_STORAGE_KEY, 'fr');
		expect(readLocale(storage)).toBe(DEFAULT_LOCALE);
	});

	it('writes locale values to storage', () => {
		const storage = createStorage();
		writeLocale(storage, 'es');
		expect(storage.getItem(LOCALE_STORAGE_KEY)).toBe('es');
	});
});

describe('translations store', () => {
	it('resolves dictionaries based on locale', () => {
		locale.set('en');
		expect(get(translations)).toBe(en);
		locale.set('de');
		expect(get(translations)).toBe(de);
		locale.set('es');
		expect(get(translations)).toBe(es);
	});
});
