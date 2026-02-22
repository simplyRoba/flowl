import { derived, writable } from 'svelte/store';
import { en } from '$lib/i18n/en';
import { de } from '$lib/i18n/de';
import { es } from '$lib/i18n/es';
import type { Translations } from '$lib/i18n/en';

export type Locale = 'en' | 'de' | 'es';

export const LOCALE_STORAGE_KEY = 'flowl.locale';
export const DEFAULT_LOCALE: Locale = 'en';

const dictionaries: Record<Locale, Translations> = { en, de, es };

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

export const locale = writable<Locale>(DEFAULT_LOCALE);

export const translations = derived(locale, ($locale) => dictionaries[$locale]);

let initialized = false;

export function initLocale(): void {
	if (typeof window === 'undefined' || initialized) return;
	initialized = true;
	const storage = getStorage();
	locale.set(readLocale(storage));
}

export function destroyLocale(): void {
	initialized = false;
}

export function setLocale(l: Locale): void {
	locale.set(l);
	writeLocale(getStorage(), l);
}
