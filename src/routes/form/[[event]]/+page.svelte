<script>
	import FloatingLabelInput from 'flowbite-svelte/FloatingLabelInput.svelte';
	import Button from 'flowbite-svelte/Button.svelte';
	import Popover from 'flowbite-svelte/Popover.svelte';

	import { format, isSameDay } from 'date-fns/fp';
	import { formatISO, formatRelative } from 'date-fns';
	import { parseTaskText, unparseTaskText } from '$lib/parser';

	import * as pkg from 'rrule';
	// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
	const { RRule } =  pkg.default || pkg;
	
	/** @type {import('./$types').PageData}*/
	export let data;

	let taskText = data.event ? unparseTaskText(data.event) : ''; // Duplicate to avoid chainging the prop
	const today = new Date();
	/** @type {boolean} */
	let editting;
	/** @type {ReturnType<parseTaskText>}*/
	let taskInfo;
	$: {
		taskInfo = parseTaskText(taskText, today);
		editting = typeof data.event !== 'undefined'
	}

	// /** @type {import('./$types').Snapshot<string>} */
	// export const snapshot = {
	// 	capture: () => taskText,
	// 	restore: (value) => (taskText = value)
	// };

	// TODO use current date?
	let dateStr = formatISO(today);
</script>

<div class="flex h-full w-full flex-row items-center align-middle">
	<div class="flex-1 p-10">
		<h5 class="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">
			{ editting ? 'Edit Event' : 'New Event' }
		</h5>
		<p>
			For creating tasks we use text rather than controls. You can see the details of each on the
			table below
		</p>
		<p>Hover over each line to see details</p>
		<div>
			<form method="POST" class="mt-2 flex-1">
				<FloatingLabelInput
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
				<input type="text" bind:value={dateStr} readonly class="hidden" />
				<div class="mt-3 flex">
					<div class="flex-1">
						<dl class="divide-y divide-gray-100">
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="title-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Title</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.title}
									<Popover placement="bottom" triggeredBy="#title-info" title="The title of the task">
										The title of the task. It will what remains after parsing all modifiers.
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="type-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">
									Type (@)
									<Popover placement="bottom" triggeredBy="#type-info" title="Type of the task">
										The type of task, prefix with <code>@</code>, can be <code>Block</code>,
										<code>Event</code>, <code>Task</code> or <code>Remainder</code>
									</Popover>
								</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.type}
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="date-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Date (.*)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{#if taskInfo.date}
											{format('dd MMM yyyy HH:mm', taskInfo.date)}
										{#if taskInfo.endDate}
											-
											{#if !isSameDay(taskInfo.endDate, taskInfo.date)}
												{format('dd MMM yyyy HH:mm', taskInfo.endDate)}
											{:else}
												{format('HH:mm', taskInfo.endDate)}
											{/if}
										{/if}
									{/if}
									{#if taskInfo.recur}
									  <p>| {RRule.fromString(taskInfo.recur).toText()}</p>
									{/if}
									<Popover placement="bottom" triggeredBy="#date-info" title="The date of the task">
										The date of this task/event. It will be parsed from the text in parenthesis. 
										After the date you can add a <code>|</code> and a recurrence pattern (check rrule.js)
										Example: Go shopping (at 21 until 23 | every monday) -> Date: today 9pm until 23:00 [every week on Monday]
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="status-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Status (%)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.status}
									<Popover placement="bottom" triggeredBy="#status-info" title="The status of the task">
										The status of task, prefix with <code>%</code>, can be <code>back</code>,
										<code>todo</code>, <code>doing</code> or <code>done</code>
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="tag-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Tags (#)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.tag.join(', ')}
									<Popover placement="bottom" triggeredBy="#tag-info" title="The tags of the task">
										The tags for this task, prefix with <code>#</code>, there can be many of these.
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="importance-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Importance (? or !)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.importance ?? 0}
									<Popover placement="bottom" triggeredBy="#importance-info" title="The importance of the task">
										How important is this task, use <code>?</code> for less import or <code>!</code>
										for more important ones. You can add up to 3 of either. <code>??</code> means
										-2, and <code>!</code> means +1
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="urgency-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Urgency (^)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.urgency ?? 0}
									<Popover placement="bottom" triggeredBy="#urgency-info" title="The urgency of the task">
										How urgent the task is, use <code>^</code>. You can add up to 3.
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="load-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Load ($)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.load ?? 0}
									<Popover placement="bottom" triggeredBy="#load-info" title="The load of the task">
										How hard is the task, use <code>$</code>. You can add up to 3.
									</Popover>
								</dd>
							</div>
						</dl>
					</div>
					<div class="flex-0">
						<Button type="submit">{editting ? 'Update' : 'Create'}</Button>
					</div>
				</div>
			</form>
		</div>
	</div>
</div>
