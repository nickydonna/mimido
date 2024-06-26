<script lang="ts">
	import FloatingLabelInput from 'flowbite-svelte/FloatingLabelInput.svelte';
	import Button from 'flowbite-svelte/Button.svelte';
	import {
		Accordion,
		AccordionItem,
		Helper,
		Input,
		Label,
		MultiSelect,
		Select,
		Spinner
	} from 'flowbite-svelte';
	import { format, formatISODuration } from 'date-fns/fp';
	import { formatDuration } from 'date-fns';
	import { EStatus, EType, parseTaskText, unparseTaskText } from '$lib/parser/index.js';
	import { isBlock, isDefined, isReminder, isTask } from '$lib/util.js';
	import MdEditor from '$lib/components/md-editor';

	import { enhance } from '$app/forms';
	import { rruleToText } from '$lib/utils/rrule.js';
	import { createEventDispatcher, type EventDispatcher } from 'svelte';
	import '@milkdown/theme-nord/style.css';
	import { page } from '$app/stores';
	import { upsert } from '$lib/stores';

	const typeOptions = Object.values(EType).map((type) => ({ value: type, name: type }));
	const statusOptions = Object.values(EStatus).map((type) => ({ value: type, name: type }));

	const today = new Date();
	const offset = today.getTimezoneOffset();
	const { editing, date } = $upsert;
	const tag = $page.url.searchParams.get('tag') ?? undefined;
	let originalText: string = '';
	if (editing) {
		originalText = unparseTaskText(editing);
	} else if (tag) {
		originalText = `#${tag} `;
	} else if (date) {
		originalText = `(${format("dd 'of' MMM", date)} at ${format('HH:mm', date)}) `;
	}

	const dispatch: EventDispatcher<{ success: null }> = createEventDispatcher();
	const onSuccess = () => dispatch('success');

	let taskText = originalText;
	let description = '';
	let isEditting = isDefined(editing);
	let upserting = false;
	let taskInfo = parseTaskText(taskText);

	// Form variables
	let alarmsValue: { name: string; value: string }[] = [];
	let errors: string[] | undefined;
	let formAction = '/form';
	$: {
		taskInfo = parseTaskText(taskText);
		alarmsValue = taskInfo.alarms.map((alarm) => ({
			name: `${formatDuration({ ...alarm.duration }, { format: ['days', 'hours', 'minutes'] })} before`,
			value: formatISODuration(alarm.duration)
		}));
		formAction = isEditting ? `/form/${editing?.eventId}` : '/form';
	}
</script>

<div class="flex h-full w-full flex-row items-center align-middle">
	<div class="flex-1 p-10">
		<h5 class="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">
			{isEditting ? 'Edit Event' : 'New Event'}
		</h5>
		<p>For creating tasks we use text rather than controls.</p>
		<Accordion flush>
			<AccordionItem>
				<span slot="header">Explanation</span>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Title:</span>
					The title is what remains after all other modifers have been parsed.
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Type:</span>To type the task, prefix with <code>@</code>,
					can be <code>Block</code>,
					<code>Event</code>, <code>Task</code> or <code>Remainder</code>
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Status:</span>
					The status of task can be indicated by prefixing with <code>%</code>, can be
					<code>back</code>,
					<code>todo</code>, <code>doing</code> or <code>done</code>
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Dates:</span>To set dates of a task/event. It will be
					parsed from the text in parenthesis. After the date you can add a <code>|</code> and a recurrence
					pattern (check rrule.ts).
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					Example: Go shopping (at 21 until 23 | every monday) -> Date: today 9pm until 23:00 [every
					week on Monday]
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Tags:</span>To set tags just add <code>#</code> before a word.
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Importance:</span>
					To set the importance of the task use <code>?</code> for less import or <code>!</code>
					for more important ones. You can add up to 3 of either. <code>??</code> means -2, and
					<code>!</code> means +1
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Urgency:</span>
					Urgency is dictated by <code>^</code>. You can add up to 3.
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Load:</span>
					Load is dictated by <code>^</code>. You can add up to 3.
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Alarms:</span>
					Alarms are set using a <code>*</code> followed by ISO 860 duration. For example *PT1H30M ->
					is alert me 1 hour 30 min before. And *P1DT1H is a day and an hour before
				</p>
			</AccordionItem>
		</Accordion>

		<div>
			<form
				method="POST"
				action={formAction}
				class="mt-2"
				use:enhance={() => {
					upserting = true;
					return async ({ update, result }) => {
						if (result.type === 'failure') {
							upserting = false;
							// @ts-expect-error no svelte template types
							errors = result.data?.errors ?? [];
						} else if (result.type === 'success') {
							await update();
							upserting = false;
							onSuccess();
						}
					};
				}}
			>
				<input type="number" class="hidden" name="offset" value={offset} />
				<div class="flex">
					<div class="mr-2 flex-1">
						<FloatingLabelInput
							disable={upserting}
							id="original_text"
							name="originalText"
							type="text"
							label="Type your event"
							required
							autofocus
							bind:value={taskText}
						>
							Type your task
						</FloatingLabelInput>
					</div>
				</div>
				{#if errors}
					<Helper class="pt-2 text-red-950">
						Please fix the following errors
						<ul class="list-disc">
							{#each errors as e}
								<li>{e}</li>
							{/each}
						</ul>
					</Helper>
				{/if}
				<div class="mt-3">
					<div class="my-2 flex">
						<FloatingLabelInput
							id="title"
							name="title"
							type="text"
							value={taskInfo.title}
							label="Title"
							required
							disabled
						></FloatingLabelInput>
					</div>
					{#if taskInfo.tags.length > 0}
						<Label>
							#Tags:
							<MultiSelect
								class="mt-2"
								disabled
								items={taskInfo.tags.map((t) => ({ value: t, name: t }))}
								value={taskInfo.tags}
								size="md"
							/>
						</Label>
					{/if}
					<div class="mt-3 flex">
						<div class="flex-0 px-1">
							<Label>
								@Type:
								<Select class="mt-2" disabled items={typeOptions} value={taskInfo.type} size="md" />
							</Label>
						</div>
						{#if isTask(taskInfo) || isReminder(taskInfo)}
							<div class="flex-0 px-1">
								<Label>
									%Status:
									<Select
										class="mt-2"
										disabled
										items={statusOptions}
										value={taskInfo.status}
										size="md"
									/>
								</Label>
							</div>
						{/if}
					</div>
					<div class="mt-3 flex">
						{#if taskInfo.date && taskInfo.endDate}
							<div class="flex">
								<div class="mt-2 flex-1 px-1">
									<Label>
										From:
										<Input name="date" value={taskInfo.date} disabled />
									</Label>
								</div>
								<div class="mt-2 flex-1 px-1">
									<Label>
										To:
										<Input name="endDate" value={taskInfo.endDate} disabled />
									</Label>
								</div>
							</div>
						{:else if taskInfo.date}
							<div class="flex-1 px-1">
								<Label>
									Start Date
									<Input name="date" disabled value={taskInfo.date} />
								</Label>
							</div>
						{/if}
					</div>
					<div class="mt-3 flex">
						{#if taskInfo.recur}
							<div class="flex-0 px-1">
								<Label>
									Recur
									<Input
										class="mt-2"
										size="sm"
										name="recur"
										disabled
										value={rruleToText(taskInfo.recur)}
									/>
								</Label>
							</div>
						{/if}
						{#if taskInfo.alarms.length > 0}
							<div class="flex-0 px-1">
								<Label>
									Alarms:
									<MultiSelect
										class="mt-2"
										name="alarms"
										disabled
										items={alarmsValue}
										value={alarmsValue.map((a) => a.value)}
										size="sm"
									/>
								</Label>
							</div>
						{/if}
					</div>
					{#if !isBlock(taskInfo)}
						<div class="mt-3 flex">
							{#if taskInfo.importance !== 0}
								<div class="flex-0 px-1">
									<Label>
										Importance
										<Input
											class="mt-2"
											type="number"
											size="sm"
											name="importance"
											disabled
											value={taskInfo.importance}
										/>
									</Label>
								</div>
							{/if}
							{#if taskInfo.urgency !== 0}
								<div class="flex-0 px-1">
									<Label>
										Urgency
										<Input
											class="mt-2"
											type="number"
											size="sm"
											name="urgency"
											disabled
											value={taskInfo.urgency}
										/>
									</Label>
								</div>
							{/if}
							{#if taskInfo.load !== 0}
								<div class="flex-0 px-1">
									<Label>
										Load
										<Input
											class="mt-2"
											type="number"
											size="sm"
											name="load"
											disabled
											value={taskInfo.load}
										/>
									</Label>
								</div>
							{/if}
						</div>
					{/if}
				</div>
				<Label for="description" class="text-md my-2 block text-gray-500 dark:text-gray-400">
					Description
				</Label>
				<div class="my-3">
					<MdEditor
						name="description"
						bind:value={description}
						placeholder="[Enter an optional Description ...]"
						editable
					/>
				</div>
				<div class="">
					<Button type="submit" disabled={upserting}>
						{#if upserting}
							<Spinner class="me-3" size="4" />
						{/if}
						{isEditting ? 'Update' : 'Create'}
					</Button>
				</div>
			</form>
		</div>
	</div>
</div>
