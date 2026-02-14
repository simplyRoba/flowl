/**
 * Convert an emoji character to its Noto Color Emoji SVG path.
 * e.g. "🪴" → "/emoji/emoji_u1fab4.svg"
 */
export function emojiToSvgPath(emoji: string): string {
	const codepoint = emoji.codePointAt(0);
	if (!codepoint) return '/emoji/emoji_u1fab4.svg';
	return `/emoji/emoji_u${codepoint.toString(16)}.svg`;
}
