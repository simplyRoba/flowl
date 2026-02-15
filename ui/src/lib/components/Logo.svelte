<script lang="ts">
	let { size = 20 }: { size?: number } = $props();
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
>
	<!-- Left leaf (bigger) -->
	<path d="M11 9a6 6 0 0 1-6-7 6 6 0 0 1 6 7" fill="currentColor" fill-opacity="0.15"/>
	<!-- Right leaf (smaller) -->
	<path d="M13 9a5 5 0 0 0 5-5 5 5 0 0 0-5 5" fill="currentColor" fill-opacity="0.15"/>
	<!-- Owl head -->
	<ellipse cx="12" cy="14.5" rx="6.5" ry="5.5"/>
	<!-- Ring eyes -->
	<circle cx="9.5" cy="13.5" r="2.3"/>
	<circle cx="14.5" cy="13.5" r="2.3"/>
	<!-- Pupils -->
	<circle cx="9.5" cy="13.5" r="0.9" fill="currentColor" stroke="none"/>
	<circle cx="14.5" cy="13.5" r="0.9" fill="currentColor" stroke="none"/>
	<!-- Beak -->
	<path d="M11.2 16.8L12 17.8l.8-1" fill="currentColor" stroke="none"/>
</svg>
