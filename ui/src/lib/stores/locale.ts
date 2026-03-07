import { get, writable } from 'svelte/store';
import { en } from '$lib/i18n/en';
import type { Translations } from '$lib/i18n/en';

export type Locale = 'en' | 'de' | 'es';

export const LOCALE_STORAGE_KEY = 'flowl.locale';
export const DEFAULT_LOCALE: Locale = 'en';

const dictionaries = new Map<Locale, Translations>([['en', en]]);
const loadingDictionaries = new Map<Locale, Promise<Translations>>();

export function isLocale(value: unknown): value is Locale {
	return value === 'en' || value === 'de' || value === 'es';
}

function getStorage(): Storage | null {
	if (typeof window === 'undefined') return null;
	try {
		return window.localStorage;
	} catch {
		return null;
	}
}

export function readLocale(storage: Storage | null): Locale {
	if (!storage) return DEFAULT_LOCALE;
	try {
		const stored = storage.getItem(LOCALE_STORAGE_KEY);
		return isLocale(stored) ? stored : DEFAULT_LOCALE;
	} catch {
		return DEFAULT_LOCALE;
	}
}

export function writeLocale(storage: Storage | null, locale: Locale): void {
	if (!storage) return;
	try {
		storage.setItem(LOCALE_STORAGE_KEY, locale);
	} catch {
		// Ignore storage write failures.
	}
}

async function loadDictionary(target: Locale): Promise<Translations> {
	if (target === 'en') return en;
	if (target === 'de') {
		const module = await import('$lib/i18n/de');
		return module.de;
	}
	const module = await import('$lib/i18n/es');
	return module.es;
}

function ensureDictionary(target: Locale): Promise<Translations> {
	const cached = dictionaries.get(target);
	if (cached) return Promise.resolve(cached);

	const inFlight = loadingDictionaries.get(target);
	if (inFlight) return inFlight;

	const promise = loadDictionary(target)
		.then((dictionary) => {
			dictionaries.set(target, dictionary);
			loadingDictionaries.delete(target);
			return dictionary;
		})
		.catch((error: unknown) => {
			loadingDictionaries.delete(target);
			throw error;
		});

	loadingDictionaries.set(target, promise);
	return promise;
}

const initialLocale = readLocale(getStorage());

export const locale = writable<Locale>(initialLocale);
export const translations = writable<Translations>(en);

locale.subscribe((target) => {
	const cached = dictionaries.get(target);
	if (cached) {
		translations.set(cached);
		return;
	}

	void ensureDictionary(target)
		.then((dictionary) => {
			if (get(locale) === target) {
				translations.set(dictionary);
			}
		})
		.catch(() => {
			if (get(locale) === target) {
				translations.set(en);
			}
		});
});

let initialized = false;

export function initLocale(serverLocale?: Locale): void {
	if (typeof window === 'undefined' || initialized) return;
	initialized = true;
	const storage = getStorage();
	const value = serverLocale ?? readLocale(storage);
	locale.set(value);
	if (serverLocale) writeLocale(storage, serverLocale);
}

export function destroyLocale(): void {
	initialized = false;
}

export function setLocale(l: Locale): void {
	locale.set(l);
	writeLocale(getStorage(), l);
	import('$lib/api')
		.then(({ updateSettings }) => updateSettings({ locale: l }))
		.catch(() => {});
}
