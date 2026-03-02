/**
 * Derive a thumbnail URL from a photo URL by inserting a size suffix
 * before the extension and changing the extension to `.jpg`.
 *
 * Example: thumbUrl('/uploads/a1b2c3.png', 200) → '/uploads/a1b2c3_200.jpg'
 */
export function thumbUrl(photoUrl: string, size: number): string {
	const lastDot = photoUrl.lastIndexOf('.');
	if (lastDot === -1) return photoUrl;
	const base = photoUrl.substring(0, lastDot);
	return `${base}_${size}.jpg`;
}
