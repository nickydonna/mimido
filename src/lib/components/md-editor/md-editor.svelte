<script lang="ts">
	import { Carta, Markdown, MarkdownEditor } from 'carta-md';
	import 'carta-md/default.css'; /* Default theme */
	import './markdown.pcss';
	import DOMPurify from 'isomorphic-dompurify';

	export let name: string | undefined = undefined;
	export let placeholder: string | undefined = undefined;
	export let value: string | undefined;
	export let editable: boolean = true;

	const carta = new Carta({
		// Remember to use a sanitizer to prevent XSS attacks!
		// More on that below
		sanitizer: DOMPurify.sanitize
	});
</script>

{#if editable}
	<MarkdownEditor mode="tabs" textarea={{ name }} {carta} bind:value {placeholder} />
{:else}
	<Markdown {carta} value={value ?? placeholder ?? ''} />
{/if}
