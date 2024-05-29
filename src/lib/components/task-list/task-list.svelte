<script lang="ts">
	import { loading, selectedEvent } from '$lib/stores';
	import { isDefined, isDone } from '$lib/util';
	import { Checkbox } from 'flowbite-svelte';
	import FilterDropwdown from '../filter-dropdown';
	import { page } from '$app/stores';
	import { goto, invalidateAll } from '$app/navigation';
	import type { TTaskSchema, WithId } from '$lib/server/calendar';
	import { createEventDispatcher } from 'svelte';
	import { EStatus } from '$lib/parser';
	import { ArrowLeftOutline, ArrowRightOutline } from 'flowbite-svelte-icons';

	const map: Record<EStatus, number> = {
		[EStatus.DOING]: 1,
		[EStatus.TODO]: 2,
		[EStatus.BACK]: 3,
		[EStatus.DONE]: 4
	};

	function sortBytStatus(a: TTaskSchema, b: TTaskSchema) {
		const aStatus = map[a.status as EStatus];
		const bStatus = map[b.status as EStatus];
		if (aStatus < bStatus) {
			return -1;
		}
		if (aStatus > bStatus) {
			return 1;
		}
		return 0;
	}

	export let tasks: Array<WithId<TTaskSchema>>;
	let tags: Array<string>;
	let tagFilter: string | undefined;
	let internalTasks: typeof tasks;
	let groupedEvents: Array<[EStatus, WithId<TTaskSchema>[]]>;

	$: {
		tags = [...new Set(tasks.map((e) => e.tags).flat())];
		tagFilter = $page.url.searchParams.get('tag') ?? undefined;
		if (isDefined(tagFilter)) {
			internalTasks = tasks.filter((e) => e.tags.includes(tagFilter as string));
		} else {
			internalTasks = tasks;
		}
		groupedEvents = [
			[EStatus.DOING, internalTasks.filter((e) => e.status === EStatus.DOING)],
			[EStatus.TODO, internalTasks.filter((e) => e.status === EStatus.TODO)],
			[EStatus.BACK, internalTasks.filter((e) => e.status === EStatus.BACK)],
			[EStatus.DONE, internalTasks.filter((e) => e.status === EStatus.DONE).slice(0, 5)]
		];
	}

	const dispatch = createEventDispatcher<{
		dragtask: WithId<TTaskSchema>;
	}>();

	function setTag(tag?: string) {
		let query = new URLSearchParams($page.url.searchParams.toString());
		if (tag) {
			query.set('tag', tag);
		} else {
			query.delete('tag');
		}
		goto(`?${query.toString()}`);
	}

	async function handleToggle(task: WithId<TTaskSchema>) {
		loading.increase();
		const status = isDone(task) ? EStatus.TODO : EStatus.DONE;
		await fetch(`/event/${task.eventId}/status`, {
			method: 'PUT',
			body: JSON.stringify({ status })
		});

		// TODO manage error
		loading.decrease();
		await invalidateAll();
	}

	const handleStatusChange = async (task: WithId<TTaskSchema>, direction: 1 | -1) => {
		loading.increase();
		const statuses = Object.values(EStatus);
		if (task.status === EStatus.DONE && direction === 1) return;
		if (task.status === EStatus.BACK && direction === -1) return;
		const statusIdx = Object.values(EStatus).indexOf(task.status as EStatus);
		const status = statuses[statusIdx + direction];

		await fetch(`/event/${task.eventId}/status`, {
			method: 'PUT',
			body: JSON.stringify({ status })
		});

		// TODO manage error
		loading.decrease();
		await invalidateAll();
	};
</script>

<div class="flex items-center gap-2 mb-2">
	<h5
		id="drawer-label"
		class="inline-flex flex-1 text-base font-semibold text-gray-500 dark:text-gray-400"
	>
		Tasks
	</h5>
	<FilterDropwdown {tags} on:select={(e) => setTag(e.detail)} on:clear={() => setTag()} />
</div>

<div class="pr-1">
	{#each groupedEvents as [group, tasks]}
		{#if tasks.length > 0}
			<p class="font-bold border-b border-gray-500">{group}</p>
		{/if}
		{#each tasks as task}
			<div
				class={`group block p-2 rounded-md hover:bg-gray-600 ${isDone(task) ? 'line-through !text-gray-400' : ''}`}
			>
				<Checkbox checked={isDone(task)} on:change={() => handleToggle(task)} class="items-center">
					<button
						class="truncate"
						on:click={() => {
							$selectedEvent = task;
						}}
						draggable="true"
						aria-hidden="true"
						on:dragstart={() => dispatch('dragtask', task)}
						on:dragend
					>
						{task.title}
					</button>
					<span class="ml-1">
						{#if task.status !== EStatus.BACK && !isDone(task)}
							<button
								class="hidden group-hover:inline-block hover:scale-125"
								on:click={() => handleStatusChange(task, -1)}
							>
								<ArrowLeftOutline />
							</button>
						{/if}
						{#if !isDone(task)}
							<button
								class="hidden group-hover:inline-block hover:scale-125"
								on:click={() => handleStatusChange(task, 1)}
							>
								<ArrowRightOutline />
							</button>
						{/if}
					</span>
				</Checkbox>
			</div>
		{/each}
	{/each}
</div>
