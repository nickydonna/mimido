<script lang="ts">
	import { Button, Dropdown, Radio } from 'flowbite-svelte';
	import { ChevronDownOutline } from 'flowbite-svelte-icons';
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher<{
		select: string;
		clear: null;
	}>();

	export let tags: Array<string>;

	let selectedTag: string | undefined;

	$: {
		if (selectedTag) {
			dispatch('select', selectedTag);
		} else {
			dispatch('clear');
		}
	}
</script>

<Button>
	{#if selectedTag}
		#{selectedTag}
	{:else}
		Filter by tag
	{/if}
	<ChevronDownOutline class="w-4 h-4 ml-2 text-white dark:text-white" />
</Button>
<Dropdown containerClass="border border-white" class="w-44 p-3 space-y-3 text-sm">
	{#if selectedTag}
		<li>
			<Button
				on:click={() => {
					selectedTag = undefined;
				}}
			>
				Clear Filter</Button
			>
		</li>
	{/if}
	{#each tags as t}
		<li>
			<Radio bind:group={selectedTag} value={t}>#{t}</Radio>
		</li>
	{/each}
</Dropdown>
