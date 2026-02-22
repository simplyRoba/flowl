export type { Translations } from './en';
export { en } from './en';
export { de } from './de';
export { es } from './es';

export function plural(forms: { one: string; other: string }, n: number): string {
	const form = n === 1 ? forms.one : forms.other;
	return form.replace('{n}', String(n));
}
