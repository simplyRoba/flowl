import { describe, expect, it } from 'vitest';
import { plural } from './index';

describe('plural', () => {
	it('returns the singular form for one', () => {
		expect(plural({ one: '{n} plant', other: '{n} plants' }, 1)).toBe('1 plant');
	});

	it('returns the plural form for many', () => {
		expect(plural({ one: '{n} plant', other: '{n} plants' }, 5)).toBe('5 plants');
	});

	it('uses the plural form for zero', () => {
		expect(plural({ one: '{n} plant', other: '{n} plants' }, 0)).toBe('0 plants');
	});
});
