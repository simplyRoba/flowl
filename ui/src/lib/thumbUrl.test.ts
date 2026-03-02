import { describe, it, expect } from 'vitest';
import { thumbUrl } from './thumbUrl';

describe('thumbUrl', () => {
	it('derives 200px thumbnail URL for JPEG', () => {
		expect(thumbUrl('/uploads/a1b2c3.jpg', 200)).toBe('/uploads/a1b2c3_200.jpg');
	});

	it('derives 600px thumbnail URL for JPEG', () => {
		expect(thumbUrl('/uploads/a1b2c3.jpg', 600)).toBe('/uploads/a1b2c3_600.jpg');
	});

	it('changes PNG extension to .jpg', () => {
		expect(thumbUrl('/uploads/d4e5f6.png', 200)).toBe('/uploads/d4e5f6_200.jpg');
	});

	it('changes WebP extension to .jpg', () => {
		expect(thumbUrl('/uploads/abc123.webp', 600)).toBe('/uploads/abc123_600.jpg');
	});

	it('returns original URL if no extension', () => {
		expect(thumbUrl('/uploads/noext', 200)).toBe('/uploads/noext');
	});
});
