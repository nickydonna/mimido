<script>
	import { importanceToString, loadToString, urgencyToString } from "$lib/util";
	import { formatDuration } from "date-fns";
	import { ArrowsRepeatOutline } from "flowbite-svelte-icons";
 	import * as pkg from 'rrule';
	// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
	const { RRule } = pkg.default || pkg;


  /** @type {import("$lib/server/calendar").TEventSchema} */
  export let event;
</script>

<div>
  <p class="text-lg">
	  {event.title}
    {#if event.recur}
      <ArrowsRepeatOutline class="inline-block" />
    {/if}
  </p>
  {importanceToString(event.importance, '|')}
  {urgencyToString(event.urgency, '|')}
  {loadToString(event.load)}
  {#if event.alarm}
	<div>
    Alarm:
    {formatDuration({...event.alarm.duration}, { format: ['days', 'hours', 'minutes']})}
    {event.alarm.isNegative ? 'before' : 'after'}
  </div>
  {/if}
</div>
