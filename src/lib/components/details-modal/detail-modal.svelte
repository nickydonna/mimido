<script>
	import { Badge, Button, ButtonGroup, Modal } from 'flowbite-svelte';
	import { createEventDispatcher } from 'svelte';
	import * as pkg from 'rrule';
	import { formatDuration } from 'date-fns';
	import { getEventColor, importanceToString, loadToString, urgencyToString } from '$lib/util';
	import { AngleLeftOutline, AngleRightOutline, ArrowLeftToBracketOutline, CheckOutline } from 'flowbite-svelte-icons';
	import { EStatus } from '$lib/parser';
	import EventCard from '../event-card/event-card.svelte';
	// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
	const { RRule } = pkg.default || pkg;

	/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

  /**
   * @type {import('svelte').EventDispatcher<{ close: null, statuschange: { status: EStatus }}>}
   */
	const dispatch = createEventDispatcher();

	const onClose = () => dispatch('close');
  /** @param {EStatus} status */
  const onStatusChange = (status) => dispatch('statuschange', { status }) 

	/** @type {TEventSchema | undefined} */
	export let event;
  export let loading = false;

	let open = false;
  let color = '';
	$: open = !!event;
  $: color = event ? getEventColor(event) : '';
  const statuses = Object.values(EStatus);
  let statusIdx = -1;
  $: statusIdx = statuses.indexOf(event?.status ?? '');
</script>

<Modal bind:open dismissable={false} >
	<svelte:fragment slot="header">
		<div class="flex w-full ">
			<div class="flex-1 self-center">
        <span class="mr-1 text-{color}-600">{event?.type.toUpperCase()}:</span>
				{event?.title}
			</div>
			<div>
        <ButtonGroup>
          {#if event?.status !== EStatus.BACK}
          <Button disabled={loading} on:click={() => onStatusChange(EStatus.BACK)}>
            <ArrowLeftToBracketOutline class=" rotate-180 w-4 h-4 me-2" />
          </Button>
          <Button disabled={loading} on:click={() => onStatusChange(statuses[statusIdx - 1])} aria-label="Move to {statuses[statusIdx - 1]}">
            <AngleLeftOutline class="w-4 h-4 me-2" />
          </Button>
          {/if}
          <Button>{event?.status.toUpperCase()}</Button>
          {#if event?.status !== EStatus.DONE}
          <Button disabled={loading} on:click={() => onStatusChange(statuses[statusIdx + 1])} aria-label="Move to {statuses[statusIdx + 1]}">
            <AngleRightOutline class="w-4 h-4 me-2" />
          </Button>
          <Button disabled={loading} on:click={() => onStatusChange(EStatus.DONE)} aria-label="move to done">
            <CheckOutline class="w-4 h-4 me-2" />
          </Button>
          {/if}
        </ButtonGroup>
			</div>
		</div>
	</svelte:fragment>
	<div>
		{#if event?.tag && event.tag.length > 0}
			<div class="mb-1 flex">
				<p class="mr-1 font-semibold">Tags: {' '}</p>
				{#each event.tag as t (t)}
					<Badge rounded class="mr-1" color="purple">{t}</Badge>
				{/each}
			</div>
		{/if}
	</div>
	<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
		{#if event?.recur}
			<div class="flex-0 px-1">
				<div class="mb-1 border-b border-solid border-gray-400">Recur</div>
				<div>
					{RRule.fromString(event.recur).toText()}
				</div>
			</div>
		{/if}
		{#if event?.alarms && event?.alarms.length > 0}
			<div class="flex-0 px-1">
				<div class="mb-1 border-b border-solid border-gray-400">Alarms</div>
				{#each event?.alarms as alarm}
					{formatDuration({ ...alarm.duration }, { format: ['days', 'hours', 'minutes'] })} before |
				{/each}
				<div></div>
			</div>
		{/if}
	</p>
	<p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
		{importanceToString(event?.importance, '|')}
		{urgencyToString(event?.urgency, '|')}
		{loadToString(event?.load)}
	</p>
	<svelte:fragment slot="footer">
    <div class="w-full flex flex-row-reverse">
      <Button disabled={loading} on:click={onClose}>Close</Button>
      <Button disabled={loading} href="/form/{event?.eventId}" class="mr-2" color="alternative">Edit</Button>
    </div>
	</svelte:fragment>
</Modal>
