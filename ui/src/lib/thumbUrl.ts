/**
 * Derive a thumbnail URL from a photo URL by inserting a size suffix
 * before the extension and changing the extension to `.jpg`.
 *
 * Example: thumbUrl('/uploads/a1b2c3.png', 200) → '/uploads/a1b2c3_200.jpg'
 */
export function thumbUrl(photoUrl: string, size: number): string {
  const lastDot = photoUrl.lastIndexOf(".");
  if (lastDot === -1) return photoUrl;
  const base = photoUrl.substring(0, lastDot);
  return `${base}_${size}.jpg`;
}

const THUMB_SIZES = [200, 600, 1000] as const;

/**
 * Generate a `srcset` attribute value listing all thumbnail sizes with `w` descriptors.
 *
 * Example: thumbSrcset('/uploads/a1b2c3.png') → '/uploads/a1b2c3_200.jpg 200w, /uploads/a1b2c3_600.jpg 600w, /uploads/a1b2c3_1000.jpg 1000w'
 */
export function thumbSrcset(photoUrl: string): string {
  return THUMB_SIZES.map((s) => `${thumbUrl(photoUrl, s)} ${s}w`).join(", ");
}
