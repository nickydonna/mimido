<script>
	import Badge from 'flowbite-svelte/Badge.svelte';
	import Toast from 'flowbite-svelte/Toast.svelte';
	import FloatingLabelInput from 'flowbite-svelte/FloatingLabelInput.svelte';
	import Button from 'flowbite-svelte/Button.svelte';
	import { slide } from 'svelte/transition';
	import { invalidateAll } from '$app/navigation';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';

	import { parseTaskText } from './parser';
	import { format, isSameDay } from 'date-fns/fp';
	import { formatRelative } from 'date-fns';

	/** @type {string} */
	export let content = 'something today at 10 until 13 @event #mine #project !! ^^^';

	let taskText = content + ''; // Duplicate to avoid chainging the prop
	const today = new Date();
	/** @type {string | undefined}*/
	let str;
	let loading = false;
	/** @type {string | undefined} */
	let successToast;
	let showingToast = false;
	/** @type {number | undefined} */
	let clearSuccessToast;
	/** @type {ReturnType<parseTaskText>}*/
	let taskInfo;
	$: {
		showingToast = !!successToast;
		taskInfo = parseTaskText(taskText, today);
		str = JSON.stringify(taskInfo, null, 2);
	}

	/** @param {string} text */
	function showSuccessToast(text) {
		if (clearSuccessToast) clearTimeout(clearSuccessToast);
		console.log(text);
		successToast = text
		clearSuccessToast = setTimeout(() => {
			successToast = undefined;
			clearSuccessToast = undefined;
		}, 3000);
	}

	/** @param {{ currentTarget: EventTarget & HTMLFormElement}} event */
	async function handleSubmit(event) {
		loading = true;

		const response = await fetch('/events', {
			method: 'POST',
			body: JSON.stringify(taskInfo),
			headers: {
				'Content-Type': 'application/json'
			},
		});

		const result = await response.json();
		loading = false;
		taskText = '';

		showSuccessToast(taskInfo.title);
		// rerun all `load` functions, following the successful update
		await invalidateAll();
	}
</script>

<div>
	<h5 class="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">New Event</h5>
	<div>
		<form method="POST" class="flex-1" on:submit|preventDefault={handleSubmit}>
			<FloatingLabelInput
				id="floating_standard"
				name="floating_standard"
				type="text"
				label="Type your event"
				disabled={loading}
				bind:value={taskText}
			>
				Type your task
			</FloatingLabelInput>
			<div class="mt-3 flex">
				<div class="flex-1">
					<div>
						{taskInfo.status.toUpperCase()}: {taskInfo.title}
					</div>
					<div class="mt-2">
						<Badge large color="green">@{taskInfo.type}</Badge>
						{#if taskInfo.importance !== 0}
							<Badge large color="red">
								{new Array(Math.abs(taskInfo.importance))
									.fill(taskInfo.importance > 0 ? '!' : '?')
									.join('')}
							</Badge>
						{/if}
						{#if taskInfo.urgency > 0}
							<Badge large color="yellow">
								{new Array(Math.abs(taskInfo.urgency)).fill('^').join('')}
							</Badge>
						{/if}
						{#if taskInfo.date}
							<Badge color="indigo" large>
								{#if taskInfo.hasStartTime}
									{formatRelative(taskInfo.date, new Date(), { weekStartsOn: 1 })}
								{:else}
									{format('dd MMM yy', taskInfo.date)}
								{/if}
								{#if taskInfo.endDate}
									until
									{#if !isSameDay(taskInfo.endDate, taskInfo.date)}
										{format('dd MMM yy', taskInfo.endDate)}
									{/if}
									{#if taskInfo.hasEndTime}
										at {format('HH:mm', taskInfo.endDate)}
									{/if}
								{/if}
							</Badge>
						{/if}
						{#each taskInfo.tag as t}
							<Badge class="mr-1" color="pink" large>#{t}</Badge>
						{/each}
					</div>
				</div>
				<div class="flex-0">
					<Button type="submit">Create</Button>
				</div>
			</div>
		</form>
		<Toast dismissable={false} transition={slide} bind:open={showingToast} position="top-right">
    	<CheckCircleSolid slot="icon" class="w-4 h-4" />
			Created event: {successToast}
  	</Toast>

	</div>
</div>
