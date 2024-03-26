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
  {#each event.alarms as alarm}
  	<div>
      Alarm:
     {formatDuration({...alarm.duration}, { format: ['days', 'hours', 'minutes']})}
      {alarm.isNegative ? 'before' : 'after'}
    </div>
  {/each}

</div>

