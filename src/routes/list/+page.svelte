<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { EStatus } from '$lib/parser/index.js';
	import { isDefined, isDone } from '$lib/util.js';
	import {
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell,
		Toggle
	} from 'flowbite-svelte';
	import type { PageData } from './$types';
	import TaskList from '$lib/components/task-list';

	export let data: PageData;

	// function toggleDone() {
	// 	let query = new URLSearchParams($page.url.searchParams.toString());
	// 	if (showDone) {
	// 		query.delete('showDone');
	// 	} else {
	// 		query.set('showDone', 'true');
	// 	}
	// 	goto(`?${query.toString()}`);
	// }
</script>

<TaskList tasks={data.events} />
